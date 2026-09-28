use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, patch, post},
    Router,
};
use serde_json::json;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tower_http::services::ServeDir;

use crate::{
    handlers,
    state::AppState,
    websocket::ws_handler,
};

pub fn create_app() -> Router {
    let state = Arc::new(AppState::new());

    let user_service = ServeDir::new("static/user").append_index_html_on_directories(true);
    let manager_service = ServeDir::new("static/manager").append_index_html_on_directories(true);

    Router::new()
        .route("/api/health", get(api_health))
        .route("/api/managers", get(handlers::managers::list_managers))
        .route("/api/visitors", post(handlers::users::create_visitor))
        .route("/api/questions", get(handlers::questions::list_questions))
        .route("/api/questions", post(handlers::questions::submit_question))
        .route("/api/questions/definitions", get(handlers::questions::list_question_definitions))
        .route("/api/questions/:id", get(handlers::questions::get_question))
        .route("/api/questions/:id/status", patch(handlers::questions::update_question_status))
        .route("/api/managers/:id/status", patch(handlers::managers::update_manager_status))
        .route("/ws", get(ws_handler))
        .nest_service("/manager", manager_service)
        .fallback_service(user_service)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn api_health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
