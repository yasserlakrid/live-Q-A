use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde_json::json;

use crate::{
    models::{CreateVisitorRequest, Visitor},
    state::SharedAppState,
};

pub async fn create_visitor(
    State(state): State<SharedAppState>,
    Json(payload): Json<CreateVisitorRequest>,
) -> Result<Json<Visitor>, (StatusCode, Json<serde_json::Value>)> {
    let visitor = state
        .create_visitor(payload.name.trim(), payload.manager_id.trim())
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": err, "status": "error" })),
            )
        })?;

    Ok(Json(visitor))
}
