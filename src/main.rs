use anyhow::Error;
use axum::{
    error_handling::HandleErrorLayer, extract::Extension, http::StatusCode, response::IntoResponse,
    response::Json, routing::get, routing::post, Router,
};
use ratchet::component_service;

use ratchet::file_service::*;

use futures::FutureExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
};
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::task;
use tower::{BoxError, ServiceBuilder};
use tower_http::add_extension::AddExtensionLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt}; // for `.fuse()`
#[derive(Debug)]
struct AppState {
    channel_sender: Arc<Sender<u64>>,
    total: Mutex<u64>,
}

#[tokio::main]
async fn main() {
    // Setup a simple tracing setup
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "jolt=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (tx, rx) = channel();
    let app_state = Arc::new(AppState {
        channel_sender: Arc::new(tx),
        total: Mutex::new(0),
    });

    let app = Router::new()
        .route("/", get(home))
        .route("/info/tasks", get(diagnose_handler))
        .route("/info/cpu", get(cpu_info_handler))
        .route("/info/memory", get(ram_info_handler))
        .route("/info/network", get(network_info_handler))
        .route("/info/system", get(get_system_information_handler))
        .route("/task/kill", post(kill_task_handler))
        .route("/search", post(search)) // Add middleware to all routes
        .route("/file/largest", post(get_largest_file))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {error}"),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(6000))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .layer(AddExtensionLayer::new(app_state.clone()));

    // Spawn a new task to read from the channel
    tokio::spawn(async move {
        while let Ok(message) = rx.recv() {
            let mut total = app_state.total.lock().unwrap();
            *total += message;
        }
    });

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn home() -> &'static str {
    "Hello, World!"
}

#[derive(Serialize)]
struct SerializableError {
    message: String,
}

impl From<Error> for SerializableError {
    fn from(error: Error) -> Self {
        SerializableError {
            message: error.to_string(),
        }
    }
}

async fn diagnose_handler() -> Json<Value> {
    let resp = component_service::scan_running_proccess();
    match resp {
        Ok(r) => Json(json!(r)),
        Err(err) => {
            let sr = SerializableError::from(err);
            Json(json!(sr))
        }
    }
}
#[derive(Debug, Default, Serialize, Deserialize)]
struct KillTaskRequest {
    pid: u32,
}

async fn kill_task_handler(input: Json<KillTaskRequest>) -> impl IntoResponse {
    let resp = component_service::kill_process(input.pid);
    match resp {
        Ok(r) => Json(json!(r)),
        Err(err) => {
            let sr = SerializableError::from(err);
            Json(json!(sr))
        }
    }
}

async fn get_system_information_handler() -> Json<Value> {
    let resp = component_service::get_system_information();
    match resp {
        Ok(r) => Json(json!(r)),
        Err(err) => {
            let sr = SerializableError::from(err);
            Json(json!(sr))
        }
    }
}


async fn cpu_info_handler() -> Json<Value> {
    let resp = match task::spawn_blocking(component_service::get_current_cpu_usage).await {
        Ok(result) => result,
        Err(e) => {
            return Json(json!({ "error": format!("Error in spawn_blocking: {:?}", e) }));
        }
    };

    Json(json!(resp))
}

async fn ram_info_handler() -> Json<Value> {
    let resp = component_service::get_memory_cpu_usage();
    Json(json!(resp))
}

async fn network_info_handler() -> Json<Value> {
    let resp = component_service::get_network_information();
    Json(json!(resp))
}

// the input to our `create_user` handler
#[derive(serde::Deserialize, Default, Clone, Serialize)]
struct SearchRequest {
    pattern: Option<String>,
    path: String,
    show_full_path: Option<bool>,
}

async fn search(Json(payload): Json<SearchRequest>) -> Json<Value> {
    // TODO, this is not very effecient if we are searching a very large directory, lets
    // think about how we can improve it
    let resp = grep(
        GrepRequest {
            path: &payload.path,
            search_term: &payload.pattern.unwrap_or_default(),
            show_full_path: payload.show_full_path.unwrap_or_default(),
        },
        Arc::new(Mutex::new(Vec::new())),
    )
    .unwrap();
    // We need to derefernece here because we want what the mutex guard is pointing to
    let data_vault = resp.lock().unwrap();
    Json(json!(*data_vault))
}

// LESSON LEARNED https://docs.rs/axum/latest/axum/extract/index.html#the-order-of-extractors
async fn get_largest_file(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(payload): Json<SearchRequest>,
) -> impl IntoResponse {
    let (stop_sender, stop_receiver) = oneshot::channel();
    let mut stop_receiver = stop_receiver.fuse();
    let tracker = app_state.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(2)) => {
                    let processed_files = tracker.total.lock().unwrap();
                    println!("Number of proccessed files so far {}", processed_files);
                }
                _ = &mut stop_receiver => {
                    let mut processed_files = tracker.total.lock().unwrap();
                    *processed_files = 0;
                    break;
                }
            }
        }
    });
    // LESSON LEARNED - If you use regular mutex it blocks causes compile errors, had to use tokio mutex
    let db = app_state.clone();
    let resp = match task::spawn_blocking(move || {
        find_largest_files(
            &payload.path,
            Arc::new(Mutex::new(Vec::new())),
            db.channel_sender.clone(),
        )
    })
    .await
    {
        Ok(Ok(resp)) => resp,
        Ok(Err(e)) => {
            return Json(json!({ "error": format!("Error in find_largest_files: {:?}", e) }));
        }
        Err(e) => {
            return Json(json!({ "error": format!("Error in spawn_blocking: {:?}", e) }));
        }
    };
    let data_vault = match resp.lock() {
        Ok(data) => data,
        Err(e) => {
            return Json(json!({ "error": format!("Error locking mutex: {:?}", e) }));
        }
    };
    let file_total = app_state.total.lock().unwrap();
    let _ = stop_sender.send(());
    Json(json!({"files": *data_vault, "total_files_searched": *file_total}))
    // return Ok("")
}
