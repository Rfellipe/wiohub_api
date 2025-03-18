use bson::doc;
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use mongodb::Database;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use std::sync::Arc;
use warp::filters::ws::{Message, WebSocket};

use crate::handlers::auth_handlers::security::{decode_jwt, JWT_SECRET};
use crate::utils::utils_models;
use crate::{ClientsWorkspace, Clients};

#[derive(Deserialize, Debug)]
struct WsRequest {
    kind: String,
    message: String,
}

// example result structure
#[derive(Serialize, Debug)]
pub struct WsResult {
    pub status: String,
    pub response: String,
}

pub async fn handle_ws_client(
    websocket: warp::ws::WebSocket,
    authorization: String,
    db: Database,
    workspaces: String,
    clients: Clients,
    clients_workspaces: ClientsWorkspace,
) {
    let user_info = decode_jwt(authorization, JWT_SECRET, db.clone())
        .await
        .unwrap();

    let mut conns = ConnectionMap {
        clients,
        clients_workspaces,
    };
    let this_req = serde_qs::from_str::<utils_models::WebSocketQuery>(&workspaces).unwrap();

    let (mut tx, mut rx) = websocket.split();
    let (client_tx, _client_rx) = mpsc::unbounded_channel();

    let client_tx_ptr = Arc::new(client_tx);

    conns.add_client(user_info.client_id.as_ref().unwrap(), this_req.workspace_id.clone(), &client_tx_ptr);

    println!("{:#?}", conns);

    while let Some(body) = rx.next().await {
        let message = match body {
            Ok(msg) => msg,
            Err(e) => {
                println!("error reading message on websocket: {}", e);
                break;
            }
        };

        handle_websocket_message(message, &mut tx).await;
    }

    conns.remove_client(user_info.client_id.unwrap().as_str(), this_req.workspace_id, client_tx_ptr);

    println!("{:#?}", conns);

    println!("client disconnected");
}

pub async fn handle_websocket_message(
    message: Message,
    sender: &mut SplitSink<WebSocket, Message>,
) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = message.to_str() {
        s
    } else {
        println!("ping-pong");
        return;
    };

    let req: WsRequest = serde_json::from_str(msg).unwrap();
    println!("got request {} with body {}", req.kind, req.message);

    match req.kind.as_str() {
        "notification" => println!("test"),
        _ => println!("no"),
    }

    let response = serde_json::to_string(&WsResult {
        status: "success".to_string(),
        response: "awesome message".to_string(),
    })
    .unwrap();
    sender.send(Message::text(response)).await.unwrap();
}

#[derive(Debug)]
struct ConnectionMap {
    clients: Clients,
    clients_workspaces: ClientsWorkspace,
}

impl ConnectionMap {
    fn add_client(&mut self, client_id: &str, workspaces_id: Vec<String>, client_conn: &Arc<mpsc::UnboundedSender<Message>>) {
        self.clients
            .lock()
            .unwrap()
            .entry(client_id.to_string())
            .or_insert_with(Vec::new)
            .push(client_conn.clone());

        for workspace_id in workspaces_id {
            self.clients_workspaces
                .lock()
                .unwrap()
                .entry(workspace_id.clone())
                .or_insert_with(Vec::new)
                .push(client_conn.clone());
        }
    }

    fn remove_client(
        &mut self,
        client_id: &str,
        workspace_ids: Vec<String>,
        client_conn: Arc<mpsc::UnboundedSender<Message>>,
    ) {
        // Remove from clients map
        if let Some(client_connections) = self.clients.lock().unwrap().get_mut(client_id) {
            client_connections.retain(|client_ws| !Arc::ptr_eq(client_ws, &client_conn));
            if client_connections.is_empty() {
                self.clients.lock().unwrap().remove(client_id);
            }
        }

        // Remove from workspaces map
        for workspace_id in workspace_ids {
            if let Some(workspace_connections) = self.clients_workspaces.lock().unwrap().get_mut(&workspace_id) {
                workspace_connections.retain(|workspace_ws| !Arc::ptr_eq(workspace_ws, &client_conn));
                if workspace_connections.is_empty() {
                    self.clients_workspaces.lock().unwrap().remove(&workspace_id);
                }
            }
        }
    }
}
