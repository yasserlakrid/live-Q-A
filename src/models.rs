use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ManagerStatus {
    Available,
    Busy,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QuestionStatus {
    Waiting,
    InProgress,
    Answered,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectionKind {
    Visitor,
    Manager,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manager {
    pub id: String,
    pub name: String,
    pub status: ManagerStatus,
    pub bio: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visitor {
    pub id: String,
    pub name: String,
    pub selected_manager_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub visitor_id: String,
    pub visitor_name: String,
    pub manager_id: String,
    pub manager_name: String,
    pub question: String,
    pub created_at: String,
    pub status: QuestionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionDefinition {
    pub id: String,
    pub text: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVisitorRequest {
    pub name: String,
    pub manager_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitQuestionRequest {
    pub visitor_id: String,
    pub manager_id: String,
    pub question: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateQuestionStatusRequest {
    pub status: QuestionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateManagerStatusRequest {
    pub status: ManagerStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketEvent {
    pub r#type: String,
    pub data: serde_json::Value,
}

impl SocketEvent {
    pub fn new(event_type: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            r#type: event_type.into(),
            data,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsQuery {
    pub role: Option<String>,
    pub id: Option<String>,
    pub manager_id: Option<String>,
    pub visitor_id: Option<String>,
}

pub fn new_uuid() -> String {
    Uuid::new_v4().to_string()
}

pub fn format_timestamp() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%H:%M").to_string()
}
