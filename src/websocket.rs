use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::mpsc;

use crate::{models::ConnectionKind, state::AppState};

#[derive(Debug, Clone)]
pub struct WsSession {
    pub role: String,
    pub id: String,
}

pub async fn ws_handler(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let role = query.get("role").cloned().unwrap_or_else(|| "visitor".to_string());
    let target_id = query
        .get("id")
        .or_else(|| query.get("manager_id"))
        .or_else(|| query.get("visitor_id"))
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());

    ws.on_upgrade(move |socket| handle_socket(state, socket, role, target_id))
}

async fn handle_socket(state: Arc<AppState>, socket: WebSocket, role: String, target_id: String) {
    let kind = match role.as_str() {
        "manager" => ConnectionKind::Manager,
        _ => ConnectionKind::Visitor,
    };

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let connection_id = state.register_ws_connection(kind.clone(), target_id.clone(), tx);

    let (mut ws_sender, mut ws_receiver) = socket.split();

    let connection_payload = json!({
        "type": "connected",
        "data": {
            "role": role,
            "id": target_id,
        }
    });

    if ws_sender
        .send(Message::Text(serde_json::to_string(&connection_payload).unwrap()))
        .await
        .is_err()
    {
        state.unregister_ws_connection(&connection_id);
        return;
    }

    loop {
        tokio::select! {
            Some(message) = rx.recv() => {
                if ws_sender.send(Message::Text(message)).await.is_err() {
                    state.unregister_ws_connection(&connection_id);
                    return;
                }
            }
            result = ws_receiver.next() => {
                let Some(msg) = result else {
                    break;
                };

                match msg {
                    Ok(Message::Text(text)) => {
                        if text.trim() == "ping" {
                            let _ = ws_sender
                                .send(Message::Text("{\"type\":\"pong\"}".to_string()))
                                .await;
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Ok(Message::Binary(_)) | Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {}
                    Err(_) => break,
                }
            }
        }
    }

    state.unregister_ws_connection(&connection_id);
}
