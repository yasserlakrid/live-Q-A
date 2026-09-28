use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;

use crate::{
    models::{ConnectionKind, Manager, ManagerStatus, SocketEvent, UpdateManagerStatusRequest},
    state::SharedAppState,
};

pub async fn list_managers(State(state): State<SharedAppState>) -> Json<Vec<Manager>> {
    Json(state.list_managers())
}

pub async fn update_manager_status(
    State(state): State<SharedAppState>,
    Path(manager_id): Path<String>,
    Json(payload): Json<UpdateManagerStatusRequest>,
) -> Result<Json<Manager>, (StatusCode, Json<serde_json::Value>)> {
    let manager = state
        .update_manager_status(&manager_id, payload.status)
        .map_err(|err| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": err, "status": "error" })),
            )
        })?;

    let event = SocketEvent::new(
        "manager_status_changed",
        json!({
            "manager_id": manager.id,
            "manager_name": manager.name,
            "status": manager.status,
        }),
    );

    state.send_event_to(ConnectionKind::Manager, &manager.id, event.clone());

    Ok(Json(manager))
}
