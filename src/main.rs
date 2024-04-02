
use axum::{
    error_handling::HandleErrorLayer, extract::Extension, http::StatusCode, response::Json,
    routing::get, routing::post, Router,
    response::{IntoResponse,ErrorResponse}
};
use ratchet::component_service;

use ratchet::file_service::*;
use serde::Deserialize;
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
};
use std::time::Duration;
use tokio::task;
use tower::{BoxError, ServiceBuilder};
use tower_http::add_extension::AddExtensionLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug)]
struct AppState {
    channel_sender: Arc<Sender<u64>>,
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
    });

    let app = Router::new()
        .route("/", get(home))
        .route("/diagnose", get(diagnose_handler))
        .route("/info/cpu", get(cpu_info_handler))
        .route("/info/memory", get(ram_info_handler))
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
        .layer(AddExtensionLayer::new(app_state));

    // Spawn a new task to read from the channel
    let mut file_total = 0;
    let read_task = tokio::spawn(async move {
        while let Ok(message) = rx.recv() {
            file_total += message;
            println!("Total files: {}", file_total);
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
async fn diagnose_handler() -> Json<Value> {
    let resp = component_service::scan_running_proccess();
    component_service::get_network_information();
    component_service::get_system_memory();
    return Json(json!(resp));
}

async fn cpu_info_handler() -> Json<Value> {
    // let app_state = app_state.0;
    // LESSON LEARNED - If you use regular mutex it blocks causes compile errors, had to use tokio mutex
    // let channel_sender = app_state.channel_sender.lock().await;
    // let _ = channel_sender.send("Hi".to_string());
    // channel_sender.send("Hi Wes".to_string());
    // channel_sender.send("Hello from home handler".to_string()).unwrap();
    let resp = match task::spawn_blocking(move || component_service::get_current_cpu_usage()).await
    {
        Ok(result) => result,
        Err(e) => {
            return Json(json!({ "error": format!("Error in spawn_blocking: {:?}", e) }));
        }
    };

    return Json(json!(resp));
}

async fn ram_info_handler() -> Json<Value> {
    let resp = component_service::get_memory_cpu_usage();
    return Json(json!(resp));
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
    return Json(json!(*data_vault));
}

// LESSON LEARNED https://docs.rs/axum/latest/axum/extract/index.html#the-order-of-extractors
async fn get_largest_file(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(payload): Json<SearchRequest>,
) -> impl IntoResponse {
    // LESSON LEARNED - If you use regular mutex it blocks causes compile errors, had to use tokio mutex
    let resp = match task::spawn_blocking(move || {
        let r = find_largest_files(
            &payload.path,
            Arc::new(Mutex::new(Vec::new())),
            app_state.channel_sender.clone(),
        );
        return r;
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
    return Json(json!(*data_vault));
    // return Ok("")
}
