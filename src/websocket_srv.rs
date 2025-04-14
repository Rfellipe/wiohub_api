use futures::FutureExt;
use futures::{stream::SplitSink, SinkExt};
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures_util::StreamExt;
use log::{error, info, warn};
use mongodb::Database;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use std::{collections::HashMap, io::Error as IoError, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::http;
use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::{
        handshake::{client::Request, server::Response},
        protocol::Message,
        Error,
    },
    WebSocketStream,
};
use utils_models::WebSocketQuery;

use crate::handlers::auth_handlers::security::{decode_jwt, JWT_SECRET};
use crate::handlers::websocket_handlers::handle_realtime_data::start_stop_realtime_data;
use crate::mqtt_srv::MqttClient;
use crate::{config::WebsocketConfig, utils::utils_models};

pub type Tx = Arc<RwLock<UnboundedSender<Message>>>;
type WorkspaceId = String;
pub type ClientsWorkspaces = Arc<RwLock<HashMap<WorkspaceId, Vec<Tx>>>>;

#[derive(Deserialize)]
#[allow(dead_code)]
struct WsRequest {
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Serialize, Debug)]
pub struct WsResult {
    #[serde(rename = "type")]
    pub type_: String,
    pub data: String,
}

#[derive(Debug, Clone)]
pub struct ClientsConnections {
    pub clients_workspaces: ClientsWorkspaces,
}

impl ClientsConnections {
    async fn add_client(&self, workspaces_id: Vec<String>, conn: Tx) {
        let mut conns = self.clients_workspaces.write().await;
        for workspace_id in workspaces_id {
            conns
                .entry(workspace_id)
                .or_insert_with(Vec::new)
                .push(conn.clone());
        }
        drop(conns);
    }

    async fn remove_client(&self, workspaces_ids: Vec<String>, conn: Tx) {
        let mut conns = self.clients_workspaces.write().await;
        for workspace_id in workspaces_ids {
            if let Some(client_conn) = conns.get_mut(&workspace_id) {
                client_conn.retain(|workspace_ws| !Arc::ptr_eq(workspace_ws, &conn));
                if client_conn.is_empty() {
                    conns.remove(&workspace_id);
                }
            }
        }
        drop(conns);
    }

    pub async fn send_message(&self, workspaces_id: String, msg: &str) {
        let conns = self.clients_workspaces.read().await;
        if let Some(clients) = conns.get(&workspaces_id) {
            for client in clients {
                let ws = client.write().await;
                let ac = ws.unbounded_send(Message::Text(msg.to_string()));
                if let Err(e) = ac {
                    error!("error sendinf ws message! err: {:#?}", e);
                }
            }
        }
    }
}

async fn handle_incoming_messages(
    mut incoming: impl StreamExt<Item = Result<Message, Error>> + Unpin,
    connections: Arc<RwLock<ClientsConnections>>,
    client_conn: Tx,
    workspace_ids: Vec<String>,
    mqtt_client: Arc<MqttClient>,
) {
    while let Some(msg) = incoming.next().await {
        match msg {
            Ok(message) => {
                if let Ok(text) = message.to_text() {
                    let req: Result<WsRequest, serde_json::Error> = serde_json::from_str(text);

                    match req {
                        Ok(request) => {
                            match request.type_.as_str() {
                                "realTimeData" => {
                                    start_stop_realtime_data(text, mqtt_client.clone()).await
                                }
                                //"entry/heartbeat" ,
                                _ => warn!("no"),
                            }
                        }
                        Err(_) => continue,
                    }
                }
            }
            Err(err) => {
                error!("Client disconnected due to error: {}", err);
                break;
            }
        }
    }

    // Remove client from all workspaces on disconnection
    connections
        .write()
        .await
        .remove_client(workspace_ids, client_conn)
        .await;
    info!("Client disconnected and removed.");
}

async fn handle_outgoing_messages(
    mut rx: UnboundedReceiver<Message>,
    mut outgoing: SplitSink<WebSocketStream<TcpStream>, Message>,
    connections: Arc<RwLock<ClientsConnections>>,
    client_conn: Tx,
    workspaces: Vec<String>,
) {
    while let Some(msg) = rx.next().await {
        if let Err(err) = outgoing.send(msg).await {
            error!("Failed to send message: {}. Removing client.", err);
            break;
        }
    }

    // Remove client from all workspaces on disconnection
    connections
        .write()
        .await
        .remove_client(workspaces, client_conn)
        .await;
    error!("Client disconnected and removed.");
}

pub async fn websocket(
    ws_config: WebsocketConfig,
    connections: Arc<RwLock<ClientsConnections>>,
    mqtt_client: Arc<MqttClient>,
    db: Database,
) -> Result<(), IoError> {
    let addr = ws_config.server;

    // Supposed main thread for WebSocket Server
    tokio::spawn(async move {
        // Create the event loop and TCP listener we'll accept connections on.
        let try_socket = TcpListener::bind(&addr).await;
        let listener = try_socket.expect("Failed to bind");

        while let Ok((raw_stream, _)) = listener.accept().await {
            let mut uri = None;
            let mut token = None;

            let ws_stream = accept_hdr_async(raw_stream,  |req: &Request, mut res: Response| {
                if let Some(auth_header) = req.headers().get("Authorization") {
                        if let Ok(auth_str) = auth_header.to_str() {   
                            token = Some(auth_str.to_string());
                            uri = Some(req.uri().clone());
                            println!("token {:#?}, uri {:#?}", token, uri);
                            return Ok(res);
                    }
                }

                *res.status_mut() = http::StatusCode::UNAUTHORIZED;
                let body = Some("Unauthorized".to_string());
                return Err(res.map(|_| body));
            })
            .await;

            let mut ws_stream = match ws_stream {
                Ok(stream) => stream,
                Err(_) => continue, // skip this connection if not authorized
            };

            // Parsee the url to get users workspace ids.
            let token = token.unwrap();
            let claims = match decode_jwt(token, JWT_SECRET, db.clone()).await {
                Ok(c) => c,
                Err(_) => {
                    let unauthorized = CloseFrame{
                        code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Error,
                        reason: std::borrow::Cow::Borrowed("Unauthorized")
                    };
                    let _ = ws_stream.close(Some(unauthorized)).await;
                    continue;
                }
            };
            
            println!("{:#?}", claims);

            let uri = uri.unwrap().to_string();
            let workspace_ids: WebSocketQuery =
                serde_qs::from_str(&uri.trim_start_matches("/?")).unwrap();
            let workspace_ids_clone = workspace_ids.clone();

            let (tx, rx) = unbounded::<Message>();
            let tx_arc = Arc::new(RwLock::new(tx));

            let connections_clone = Arc::clone(&connections);

            // Add incoming connection to connection map.
            {
                // In braces so this task doesn't hold the write rights.
                let conns = connections_clone.write().await;
                conns
                    .add_client(workspace_ids_clone.clone().workspace_id, tx_arc.clone())
                    .await;
            }

            let connections_clone = Arc::clone(&connections);
            let workspace_ids_clone = workspace_ids.clone();
            let tx_clone = tx_arc.clone();

            let (outgoing, incoming) = ws_stream.split();

            // Thread to handle incoming message from clients.
            tokio::spawn(handle_incoming_messages(
                incoming,
                connections_clone,
                tx_clone,
                workspace_ids_clone.clone().workspace_id,
                mqtt_client.clone(),
            ));

            // Thread to handle sending message to clients.
            let connections_clone = Arc::clone(&connections);
            tokio::spawn(handle_outgoing_messages(
                rx,
                outgoing,
                connections_clone,
                tx_arc,
                workspace_ids.workspace_id,
            ));
        }
    });

    Ok(())
}
