use bson::doc;
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use mongodb::Database;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use warp::filters::ws::{Message, WebSocket};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::handlers::auth_handlers::security::{decode_jwt, JWT_SECRET};
use crate::utils::utils_models;
use crate::ConnectionMap;

#[derive(Deserialize, Debug)]
struct WsRequest {
    kind: String,
    message: String,
}

#[derive(Serialize, Debug)]
pub struct WsResult {
    pub kind: String,
    pub data: String,
}

pub async fn handle_ws_client(
    websocket: warp::ws::WebSocket,
    authorization: String,
    db: Database,
    workspaces: String,
    conns_map: Arc<Mutex<ConnectionMap>>,
) {
    let user_info = decode_jwt(authorization, JWT_SECRET, db.clone())
        .await
        .unwrap();

    let this_req = serde_qs::from_str::<utils_models::WebSocketQuery>(&workspaces).unwrap();
    let (mut tx, mut rx) = websocket.split();
    let (client_tx, client_rx) = mpsc::unbounded_channel::<Message>();
    let client_tx_ptr = Arc::new(client_tx);

    let mut client_rx_stream = UnboundedReceiverStream::new(client_rx);
    tokio::spawn(async move {
        while let Some(message) = client_rx_stream.next().await {
            if tx.send(message.clone()).await.is_err() {
                break;
            }
        }
    });

    {
        let conns = conns_map.lock().await;
        conns
            .add_client(
                user_info.client_id.as_ref().unwrap(),
                this_req.workspace_id.clone(),
                &client_tx_ptr,
            )
            .await;
    }

    while let Some(body) = rx.next().await {
        let _message = match body {
            Ok(msg) => msg,
            Err(e) => {
                println!("error reading message on websocket: {}", e);
                break;
            }
        };

        // handle_websocket_message(message, &mut tx).await;
    }

    {
        let conns = conns_map.lock().await;
        conns
            .remove_client(
                user_info.client_id.unwrap().as_str(),
                this_req.workspace_id,
                client_tx_ptr,
            )
            .await;
    }

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
        "realTimeData" => handle_realtime_data(sender).await, 
        _ => println!("no"),
    }

    let response = serde_json::to_string(&WsResult {
        kind: "success".to_string(),
        data: "awesome message".to_string(),
    })
    .unwrap();
    sender.send(Message::text(response)).await.unwrap();
}

async fn handle_realtime_data(sender: &mut SplitSink<WebSocket, Message> ) {
    let response = serde_json::to_string(&WsResult {
        kind: "ok".to_string(),
        data: "activating realtime data for device".to_string()
    }).unwrap();

    sender.send(Message::text(response)).await.unwrap();
}
