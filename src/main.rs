mod db;
mod errors;
mod handlers;
mod models;
mod swagger;
mod utils;

use crate::handlers::{
    auth_handlers::auth::auth_signin_handler,
    device_handlers::{device_data::devices_data_handler, device_status::device_status_handler},
};
use futures::stream::SplitSink;
use handlers::{
    auth_handlers::session::with_auth,
    device_handlers::{device::device, device_data::device_data_handler},
    mqtt_handlers::mqtt::run_mqtt,
    websocket_handlers::websocket::handle_ws_client,
};
use rumqttc::{AsyncClient, MqttOptions};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Instant};

use db::get_db;
use tokio::sync::{mpsc, Mutex};
use tokio::sync::RwLock;
use utils::utils_functions::send_to_zabbix;
use utils::utils_models;
use utoipa::OpenApi;
use warp::{
    self,
    filters::{path::FullPath, ws::{Message, WebSocket}},
    http::Method,
    Filter,
};
use warp_rate_limit::{with_rate_limit, RateLimitConfig};

type ClientId = String;
type WorkspaceId = String;
type Sender = Arc<mpsc::UnboundedSender<Message>>;
// type Sender = Arc<SplitSink<WebSocket, Message>>;

pub type Clients = Arc<RwLock<HashMap<ClientId, Vec<Sender>>>>;
pub type ClientsWorkspace = Arc<RwLock<HashMap<WorkspaceId, Vec<Sender>>>>;


#[derive(Debug, Clone)]
pub struct ConnectionMap {
    pub clients: Clients,
    pub clients_workspaces: ClientsWorkspace,
}

impl ConnectionMap {
    async fn add_client(
        &self,
        client_id: &str,
        workspaces_id: Vec<String>,
        client_conn: &Sender,
    ) {
        // Lock `clients` once
        let mut clients = self.clients.write().await;
        clients
            .entry(client_id.to_string())
            .or_insert_with(Vec::new)
            .push(client_conn.clone());
        drop(clients); // Explicitly drop write lock before acquiring another

        // Lock `clients_workspaces` once
        let mut workspaces = self.clients_workspaces.write().await;
        for workspace_id in workspaces_id {
            workspaces
                .entry(workspace_id)
                .or_insert_with(Vec::new)
                .push(client_conn.clone());
        }
        drop(workspaces);
    }

    async fn remove_client(
        &self,
        client_id: &str,
        workspace_ids: Vec<String>,
        client_conn: Sender,
    ) {
        // Lock `clients` once
        let mut clients = self.clients.write().await;
        if let Some(client_connections) = clients.get_mut(client_id) {
            client_connections.retain(|client_ws| !Arc::ptr_eq(client_ws, &client_conn));
            if client_connections.is_empty() {
                clients.remove(client_id);
            }
        }
        drop(clients); // Explicitly drop write lock before acquiring another

        // Lock `clients_workspaces` once
        let mut workspaces = self.clients_workspaces.write().await;
        for workspace_id in workspace_ids {
            if let Some(workspace_connections) = workspaces.get_mut(&workspace_id) {
                workspace_connections
                    .retain(|workspace_ws| !Arc::ptr_eq(workspace_ws, &client_conn));
                if workspace_connections.is_empty() {
                    workspaces.remove(&workspace_id);
                }
            }
        }
        drop(workspaces);
    }

    async fn send_message_to_client(&self, workspace_id: String, message: &str) {
        let msg = Message::text(message.to_string());

        // Lock `clients_workspaces` for reading once
        if let Some(connections_workspaces) = self.clients_workspaces.read().await.get(&workspace_id) {
            for conn in connections_workspaces {
                let _ = conn.send(msg.clone()).unwrap();
            }
        }
    }
}

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let config = swagger::doc_config();
    let db = get_db().await?;
    let db_clone = db.clone();

    let mqttoptions: MqttOptions = MqttOptions::new("rumqtt-async", "127.0.0.1", 1883);
    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);

    let client = Arc::new(client);

    let mqtt_client = Arc::clone(&client);

    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));
    let clients_workspaces: ClientsWorkspace = Arc::new(RwLock::new(HashMap::new()));

    let conn = Arc::new(Mutex::new(ConnectionMap {
        clients,
        clients_workspaces,
    }));
    let conn_clone = conn.clone();

    tokio::spawn(async move {
        run_mqtt(mqtt_client, eventloop, db_clone, conn_clone).await;
    });

    // 60 request per 60 seconds
    let public_routes_rate_limit = RateLimitConfig::max_per_window(5, 5 * 60);

    let root = warp::path::end().map(|| "Welcome to the Wiohub api");

    let api_doc = warp::path("api-doc.json")
        .and(warp::get())
        .map(|| warp::reply::json(&swagger::WiohubDoc::openapi()));

    let swagger_ui = warp::path("docs")
        .and(warp::get())
        .and(warp::path::full())
        .and(warp::path::tail())
        .and(warp::any().map(move || config.clone()))
        .and_then(swagger::serve_swagger);

    let devices_route = warp::path!("devices")
        .and(warp::post())
        .and(with_auth())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(device);

    let device_controller_route = warp::path!("device" / "data" / String)
        .and(warp::get())
        .and(with_auth())
        // .and(warp::header::header("authorization"))
        .and(warp::query::<utils_models::DeviceControllerQueries>())
        .and(with_db(db.clone()))
        .and_then(device_data_handler);

    let devices_controller_route = warp::path!("devices" / "data")
        .and(warp::get())
        .and(with_auth())
        // .and(warp::header::header("authorization"))
        .and(warp::query::<utils_models::DeviceControllerQueries>())
        .and(with_db(db.clone()))
        .and_then(devices_data_handler);

    let devices_status_route = warp::path!("devices" / "status")
        .and(warp::get())
        .and(with_auth())
        // .and(warp::header::header("authorization"))
        .and(warp::query::<utils_models::DeviceStatusQueries>())
        .and(with_db(db.clone()))
        .and_then(device_status_handler);

    let signin_route = warp::path!("auth" / "signin")
        .and(warp::post())
        .and(with_rate_limit(public_routes_rate_limit.clone()))
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(auth_signin_handler);

    let web_socket = warp::path("websocket")
        .and(warp::ws())
        .and(with_auth())
        .and(with_db(db.clone()))
        .and(warp::query::raw())
        .and(with_conn_maps(conn))
        .map(
            |ws: warp::ws::Ws,
             authorization: String,
             db: mongodb::Database,
             workspace_id: String,
             conn: Arc<Mutex<ConnectionMap>>| {
                ws.on_upgrade(move |websocket| {
                    handle_ws_client(websocket, authorization, db, workspace_id, conn)
                })
            },
        );

    let routes = root
        .or(api_doc)
        .or(swagger_ui)
        .or(signin_route)
        .or(devices_route)
        .or(device_controller_route)
        .or(devices_controller_route)
        .or(devices_status_route)
        .or(web_socket)
        .with(with_cors())
        .recover(errors::handle_rejection);
        // .with(warp::wrap_fn(monitoring_wrapper));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}

fn with_db(
    db: mongodb::Database,
) -> impl Filter<Extract = (mongodb::Database,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn with_cors() -> warp::filters::cors::Cors {
    warp::cors()
        .allow_origin("http://localhost:3000")
        .allow_headers(vec!["Content-Type", "Authorization"])
        .allow_methods(&[Method::GET, Method::POST])
        .allow_credentials(true)
        .build()
}

fn with_conn_maps(
    conn: Arc<Mutex<ConnectionMap>>,
) -> impl Filter<Extract = (Arc<Mutex<ConnectionMap>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || conn.clone())
}

fn monitoring_wrapper<F, T>(
    filter: F,
) -> impl Filter<Extract = (T,)> + Clone + Send + Sync + 'static
where
    F: Filter<Extract = (T,), Error = std::convert::Infallible> + Clone + Send + Sync + 'static,
    T: warp::Reply + Send + 'static,
{
    let start_time = Arc::new(std::sync::Mutex::new(Instant::now()));

    let start_time_clone = start_time.clone();

    warp::any()
        .and(warp::filters::path::full())
        .and(warp::filters::addr::remote())
        .map(move |path: FullPath, ip: Option<SocketAddr>| {
            let mut start_time = start_time_clone.lock().unwrap();
            *start_time = Instant::now();
            (path, ip)
        })
        .untuple_one()
        .and(filter)
        .map(
            move |path: warp::path::FullPath, ip: Option<std::net::SocketAddr>, arg: T| {
                let elapsed = start_time.lock().unwrap().elapsed().as_secs_f64();
                let ip_str = ip
                    .map(|addr| addr.ip().to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let value = format!(
                    "Request to {} from {} took {:.3} seconds",
                    path.as_str(),
                    ip_str,
                    elapsed
                );

                send_to_zabbix("http_request_duration", value);
                arg
            },
        )
}
