use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use serde_json::json;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::models::{
    format_timestamp, new_uuid, ConnectionKind, Manager, ManagerStatus, Question, QuestionDefinition,
    QuestionStatus, SocketEvent, Visitor,
};

#[derive(Debug, Clone)]
pub struct WsConnection {
    pub connection_id: String,
    pub kind: ConnectionKind,
    pub target_id: String,
    pub sender: mpsc::UnboundedSender<String>,
}

pub struct AppState {
    pub managers: RwLock<HashMap<String, Manager>>,
    pub visitors: RwLock<HashMap<String, Visitor>>,
    pub questions: RwLock<HashMap<String, Question>>,
    pub ws_connections: RwLock<HashMap<String, WsConnection>>,
    pub question_definitions: Vec<QuestionDefinition>,
}

impl AppState {
    pub fn new() -> Self {
        let managers = vec![
            Manager {
                id: "Yasser Lakrid".to_string(),
                name: "Yasser Lakrid".to_string(),
                status: ManagerStatus::Available,
                bio: "the cool guy ".to_string(),
            },
            Manager {
                id: "Nouha Lounes".to_string(),
                name: "Nouha Lounes".to_string(),
                status: ManagerStatus::Available,
                bio: "PrésidentE".to_string(),
            },
            Manager {
                id: "Ghada Laidisista".to_string(),
                name: "Amine".to_string(),
                status: ManagerStatus::Busy,
                bio: "VPrésidentE".to_string(),
            },
            Manager {
                id: "Mohamed Adem Boudehane".to_string(),
                name: "Mohamed Adem Boudehane".to_string(),
                status: ManagerStatus::Offline,
                bio: "RH manager.".to_string(),
            },
        ];

        let question_definitions = vec![
            QuestionDefinition {
                id: "q1".to_string(),
                text: "What does CSE stand for?".to_string(),
                category: "CSE questions ".to_string(),
            },
            QuestionDefinition {
                id: "q2".to_string(),
                text: "When was CSE created?".to_string(),
                category: "CSE questions ".to_string(),
            },
            QuestionDefinition {
                id: "q3".to_string(),
                text: "What does CSE do?".to_string(),
                category: "CSE questions ".to_string(),
            },
            QuestionDefinition {
                id: "q4".to_string(),
                text: "What departments does CSE have?".to_string(),
                category: "CSE questions".to_string(),
            },
            QuestionDefinition {
                id: "q5".to_string(),
                text: "What kind of events does CSE organize?".to_string(),
                category: "CSE events ".to_string(),
            },
            QuestionDefinition {
                id: "q6".to_string(),
                text: "How can I join CSE?".to_string(),
                category: "CSE lifestyle ? ".to_string(),
            },
            QuestionDefinition {
                id: "q7".to_string(),
                text: "What can I do as a member?".to_string(),
                category: "CSE lifestyle ?".to_string(),
            },
          
        ];

        let managers = managers
            .into_iter()
            .map(|manager| (manager.id.clone(), manager))
            .collect();

        Self {
            managers: RwLock::new(managers),
            visitors: RwLock::new(HashMap::new()),
            questions: RwLock::new(HashMap::new()),
            ws_connections: RwLock::new(HashMap::new()),
            question_definitions,
        }
    }

    pub fn list_managers(&self) -> Vec<Manager> {
        self.managers
            .read()
            .expect("managers lock poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn get_manager(&self, manager_id: &str) -> Option<Manager> {
        self.managers
            .read()
            .expect("managers lock poisoned")
            .get(manager_id)
            .cloned()
    }

    pub fn create_visitor(&self, name: &str, manager_id: &str) -> Result<Visitor, String> {
        let clean_name = name.trim();
        if clean_name.is_empty() {
            return Err("Name cannot be empty.".to_string());
        }

        let manager = self
            .managers
            .read()
            .expect("managers lock poisoned")
            .get(manager_id)
            .cloned()
            .ok_or_else(|| "Manager not found.".to_string())?;

        if manager.status == ManagerStatus::Offline {
            return Err("This manager is currently unavailable.".to_string());
        }

        let visitor = Visitor {
            id: new_uuid(),
            name: clean_name.to_string(),
            selected_manager_id: manager_id.to_string(),
        };

        self.visitors
            .write()
            .expect("visitors lock poisoned")
            .insert(visitor.id.clone(), visitor.clone());

        Ok(visitor)
    }

    pub fn list_questions(&self) -> Vec<Question> {
        self.questions
            .read()
            .expect("questions lock poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn get_question_definitions(&self) -> Vec<QuestionDefinition> {
        self.question_definitions.clone()
    }

    pub fn submit_question(&self, visitor_id: &str, manager_id: &str, question: &str) -> Result<Question, String> {
        let visitor = self
            .visitors
            .read()
            .expect("visitors lock poisoned")
            .get(visitor_id)
            .cloned()
            .ok_or_else(|| "Visitor not found.".to_string())?;

        let manager = self
            .managers
            .read()
            .expect("managers lock poisoned")
            .get(manager_id)
            .cloned()
            .ok_or_else(|| "Manager not found.".to_string())?;

        let clean_question = question.trim();
        if clean_question.is_empty() {
            return Err("Question cannot be empty.".to_string());
        }

        let question_record = Question {
            id: new_uuid(),
            visitor_id: visitor.id,
            visitor_name: visitor.name,
            manager_id: manager.id.clone(),
            manager_name: manager.name,
            question: clean_question.to_string(),
            created_at: format_timestamp(),
            status: QuestionStatus::Waiting,
        };

        self.questions
            .write()
            .expect("questions lock poisoned")
            .insert(question_record.id.clone(), question_record.clone());

        Ok(question_record)
    }

    pub fn get_question(&self, id: &str) -> Option<Question> {
        self.questions
            .read()
            .expect("questions lock poisoned")
            .get(id)
            .cloned()
    }

    pub fn list_questions_for_manager(&self, manager_id: &str) -> Vec<Question> {
        self.questions
            .read()
            .expect("questions lock poisoned")
            .values()
            .filter(|question| question.manager_id == manager_id)
            .cloned()
            .collect()
    }

    pub fn update_question_status(&self, question_id: &str, status: QuestionStatus) -> Result<Question, String> {
        let mut questions = self.questions.write().expect("questions lock poisoned");
        let question = questions
            .get_mut(question_id)
            .ok_or_else(|| "Question not found.".to_string())?;

        question.status = status;
        Ok(question.clone())
    }

    pub fn update_manager_status(&self, manager_id: &str, status: ManagerStatus) -> Result<Manager, String> {
        let mut managers = self.managers.write().expect("managers lock poisoned");
        let manager = managers
            .get_mut(manager_id)
            .ok_or_else(|| "Manager not found.".to_string())?;

        manager.status = status;
        Ok(manager.clone())
    }

    pub fn register_ws_connection(
        &self,
        kind: ConnectionKind,
        target_id: String,
        sender: mpsc::UnboundedSender<String>,
    ) -> String {
        let id = Uuid::new_v4().to_string();
        self.ws_connections
            .write()
            .expect("ws lock poisoned")
            .insert(
                id.clone(),
                WsConnection {
                    connection_id: id.clone(),
                    kind,
                    target_id,
                    sender,
                },
            );
        id
    }

    pub fn unregister_ws_connection(&self, connection_id: &str) {
        self.ws_connections
            .write()
            .expect("ws lock poisoned")
            .remove(connection_id);
    }

    pub fn send_event_to(&self, kind: ConnectionKind, target_id: &str, event: SocketEvent) {
        let payload = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());

        let stale_ids = {
            let connections = self.ws_connections.read().expect("ws lock poisoned");
            let mut stale = Vec::new();

            for (connection_id, connection) in connections.iter() {
                if connection.kind == kind && connection.target_id == target_id {
                    if connection.sender.send(payload.clone()).is_err() {
                        stale.push(connection_id.clone());
                    }
                }
            }

            stale
        };

        if !stale_ids.is_empty() {
            let mut connections = self.ws_connections.write().expect("ws lock poisoned");
            for stale_id in stale_ids {
                connections.remove(&stale_id);
            }
        }
    }

    pub fn emit_status_change(&self, visitor_id: &str, manager_id: &str, question: &Question) {
        self.send_event_to(
            ConnectionKind::Manager,
            manager_id,
            SocketEvent::new(
                "question_status_changed",
                json!({
                    "id": question.id,
                    "visitor_name": question.visitor_name,
                    "question": question.question,
                    "status": question.status,
                    "created_at": question.created_at,
                }),
            ),
        );

        self.send_event_to(
            ConnectionKind::Visitor,
            visitor_id,
            SocketEvent::new(
                "question_status_changed",
                json!({
                    "id": question.id,
                    "question": question.question,
                    "manager_name": question.manager_name,
                    "status": question.status,
                    "created_at": question.created_at,
                }),
            ),
        );
    }
}

pub type SharedAppState = Arc<AppState>;
