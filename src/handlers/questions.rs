use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;

use crate::{
    models::{
        ConnectionKind, Question, QuestionDefinition, SocketEvent, SubmitQuestionRequest,
        UpdateQuestionStatusRequest,
    },
    state::SharedAppState,
};

pub async fn list_questions(State(state): State<SharedAppState>) -> Json<Vec<Question>> {
    Json(state.list_questions())
}

pub async fn list_question_definitions(State(state): State<SharedAppState>) -> Json<Vec<QuestionDefinition>> {
    Json(state.get_question_definitions())
}

pub async fn submit_question(
    State(state): State<SharedAppState>,
    Json(payload): Json<SubmitQuestionRequest>,
) -> Result<Json<Question>, (StatusCode, Json<serde_json::Value>)> {
    let question = state
        .submit_question(&payload.visitor_id, &payload.manager_id, &payload.question)
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": err, "status": "error" })),
            )
        })?;

    state.send_event_to(
        ConnectionKind::Manager,
        &question.manager_id,
        SocketEvent::new(
            "new_question",
            json!({
                "id": question.id,
                "visitor_name": question.visitor_name,
                "manager_id": question.manager_id,
                "manager_name": question.manager_name,
                "question": question.question,
                "created_at": question.created_at,
                "status": question.status,
            }),
        ),
    );

    state.send_event_to(
        ConnectionKind::Visitor,
        &question.visitor_id,
        SocketEvent::new(
            "question_created",
            json!({
                "id": question.id,
                "manager_id": question.manager_id,
                "manager_name": question.manager_name,
                "question": question.question,
                "created_at": question.created_at,
                "status": question.status,
            }),
        ),
    );

    Ok(Json(question))
}

pub async fn get_question(
    State(state): State<SharedAppState>,
    Path(question_id): Path<String>,
) -> Result<Json<Question>, (StatusCode, Json<serde_json::Value>)> {
    state
        .get_question(&question_id)
        .map(Json)
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Question not found.", "status": "error" })),
        ))
}

pub async fn update_question_status(
    State(state): State<SharedAppState>,
    Path(question_id): Path<String>,
    Json(payload): Json<UpdateQuestionStatusRequest>,
) -> Result<Json<Question>, (StatusCode, Json<serde_json::Value>)> {
    let question = state
        .update_question_status(&question_id, payload.status)
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": err, "status": "error" })),
            )
        })?;

    state.emit_status_change(&question.visitor_id, &question.manager_id, &question);

    Ok(Json(question))
}
