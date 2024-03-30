use axum::{
    error_handling::HandleErrorLayer, http::StatusCode, response::Json, routing::get,
    routing::post, Router,
};
use ratchet::component_service;

use ratchet::file_service::*;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task;
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
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
        );

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
    let resp = match task::spawn_blocking(move || component_service::get_current_cpu_usage()).await {
        Ok(result) => result,
        Err(e) => {
            return Json(json!({ "error": format!("Error in spawn_blocking: {:?}", e) }));
        },
    };

    return Json(json!(resp));
}

async fn ram_info_handler() -> Json<Value> {
    let resp = component_service::get_memory_cpu_usage();
    return Json(json!(resp));
}

// the input to our `create_user` handler
#[derive(serde::Deserialize, Default)]
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

async fn get_largest_file(Json(payload): Json<SearchRequest>) -> Json<Value> {
    let resp = match task::spawn_blocking(move || {
        find_largest_files(&payload.path, Arc::new(Mutex::new(Vec::new())))
    }).await {
        Ok(Ok(resp)) => resp,
        Ok(Err(e)) => {
            return Json(json!({ "error": format!("Error in find_largest_files: {:?}", e) }));
        },
        Err(e) => {
            return Json(json!({ "error": format!("Error in spawn_blocking: {:?}", e) }));
        },
    };
    let data_vault = match resp.lock() {
        Ok(data) => data,
        Err(e) => {
            return Json(json!({ "error": format!("Error locking mutex: {:?}", e) }));
        },
    };
    return Json(json!(*data_vault));
}