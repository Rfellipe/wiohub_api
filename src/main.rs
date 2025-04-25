mod config;
mod db;
mod errors;
mod handlers;
mod logger;
mod models;
mod mqtt_srv;
mod swagger;
mod utils;
mod websocket_srv;

use crate::handlers::{
    auth_handlers::auth::auth_signin_handler,
    device_handlers::{device_data::devices_data_handler, device_status::device_status_handler},
};
use config::Configs;
use handlers::{
    auth_handlers::session::with_auth,
    device_handlers::{device::device, device_data::device_data_handler},
    mqtt_handlers::{
        entry_data::handle_entry_data, handle_device_registration::handle_device_registration, handle_heartbeats::{read_device_heartbeat, read_device_threads_heartbeat}, real_time_data::handle_real_time_data
    },
};
use mqtt_srv::MqttClient;
use rumqttc::QoS;
use std::{collections::HashMap, net::SocketAddr, path::Path, sync::Arc, time::Instant};
use websocket_srv::{websocket, ClientsConnections, ClientsWorkspaces};

use db::get_db;
use log::{error, info};
use tokio::sync::RwLock;
use utils::utils_functions::send_to_zabbix;
use utils::utils_models;
use utoipa::OpenApi;
use warp::{self, filters::path::FullPath, http::Method, Filter};
use warp_rate_limit::{with_rate_limit, RateLimitConfig};

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    logger::start_log();

    let config_path = Path::new("./config.toml");
    let configs = match Configs::load_from_file(config_path) {
        Ok(c) => {
            info!("Configurations loaded");
            Arc::new(RwLock::new(c))
        }
        Err(e) => {
            info!("Failed to load configurations: {}", e);
            std::process::exit(1);
        }
    };

    let mqtt_settings = Arc::clone(&configs).read().await.mqtt.clone();
    let ws_settings = Arc::clone(&configs).read().await.websocket.clone();
    let db_settings = Arc::clone(&configs).read().await.database.clone();

    let config = swagger::doc_config();
    let db = get_db(db_settings).await?;
    let db_clone = db.clone();

    let server_status: Arc<RwLock<Option<i64>>> = Arc::new(RwLock::new(None));
     let mqtt_client = MqttClient::new(mqtt_settings, server_status.clone()).await;
    let mqtt_client_ptr = Arc::new(mqtt_client.clone());

    let clients_workspaces: ClientsWorkspaces = Arc::new(RwLock::new(HashMap::new()));
    let websocket_connections = Arc::new(RwLock::new(ClientsConnections { clients_workspaces }));
    let ws_connections_clone = Arc::clone(&websocket_connections);

    mqtt_client
        .subscribe("entry/registration", QoS::AtLeastOnce)
        .await
        .ok();
    mqtt_client
        .add_topic_handler("entry/registration", move |payload| {
            let db_clone = db_clone.clone();
            let mqtt_client_clone = Arc::clone(&mqtt_client_ptr);

            tokio::spawn(async move {
                let handler = handle_device_registration(&payload, db_clone).await;
                if let Err(e) = handler {
                    error!("error on entry data: {}", e);
                    let r = mqtt_client_clone
                        .publish("entry/reports", &e.as_str(), QoS::AtLeastOnce, true)
                        .await;
                    if let Err(err) = r {
                        error!("error publishing: {}", err);
                    }
                }
            });
        })
        .await;

    let db_clone = db.clone();
    let mqtt_client_ptr = Arc::new(mqtt_client.clone());
    mqtt_client
        .subscribe("entry/heartbeat", QoS::AtLeastOnce)
        .await
        .ok();
    mqtt_client.add_topic_handler("entry/heartbeat", move |payload| {
        let db_clone = db_clone.clone();
        let mqtt_client_clone = Arc::clone(&mqtt_client_ptr);

        tokio::spawn(async move {
            let handler = read_device_heartbeat(&payload, db_clone).await;
            if let Err(e) = handler {
                error!("error on entry data: {}", e);
                let r = mqtt_client_clone
                    .publish("entry/reports", &e.as_str(), QoS::AtLeastOnce, true)
                    .await;
                if let Err(err) = r {
                    error!("error publishing: {}", err);
                }
            }
        });
    }).await;

    let db_clone = db.clone();
    let mqtt_client_ptr = Arc::new(mqtt_client.clone());
    mqtt_client
        .subscribe("entry/heartbeat/threads", QoS::AtLeastOnce)
        .await
        .ok();
    mqtt_client.add_topic_handler("entry/heartbeat/threads", move |payload| {
        let db_clone = db_clone.clone();
        let mqtt_client_clone = Arc::clone(&mqtt_client_ptr);

        tokio::spawn(async move {
            let handler = read_device_threads_heartbeat(&payload, db_clone).await;
            if let Err(e) = handler {
                error!("error on entry data: {}", e);
                let r = mqtt_client_clone
                    .publish("entry/reports", &e.as_str(), QoS::AtLeastOnce, true)
                    .await;
                if let Err(err) = r {
                    error!("error publishing: {}", err);
                }
            }
        });
    }).await;

    let db_clone = db.clone();
    let mqtt_client_ptr = Arc::new(mqtt_client.clone());
    mqtt_client
        .subscribe("entry/data", QoS::AtLeastOnce)
        .await
        .ok();
    mqtt_client
        .add_topic_handler("entry/data", move |payload| {
            let db_clone = db_clone.clone();
            let mqtt_client_clone = Arc::clone(&mqtt_client_ptr);
            let ws_conns = ws_connections_clone.clone();

            tokio::spawn(async move {
                let handler = handle_entry_data(db_clone, payload.as_str(), ws_conns).await;
                if let Err(e) = handler {
                    error!("error on entry data: {}", e);
                    let r = mqtt_client_clone
                        .publish("entry/reports", &e.as_str(), QoS::AtLeastOnce, true)
                        .await;
                    if let Err(err) = r {
                        error!("error publishing: {}", err);
                    }
                }
            });
        })
        .await;

    let db_clone = db.clone();
    let ws_connections_clone = Arc::clone(&websocket_connections);
    let mqtt_client_ptr = Arc::new(mqtt_client.clone());
    mqtt_client
        .subscribe("sensors/realtime/data", QoS::AtLeastOnce)
        .await
        .ok();
    mqtt_client
        .add_topic_handler("sensors/realtime/data", move |payload| {
            let db_clone = db_clone.clone();
            let ws_conns = ws_connections_clone.clone();
            let mqtt_client_clone = Arc::clone(&mqtt_client_ptr);

            tokio::spawn(async move {
                let handler = handle_real_time_data(db_clone, &payload, ws_conns).await;
                if let Err(e) = handler {
                    error!("error on entry data: {}", e);
                    let r = mqtt_client_clone
                        .publish("entry/reports", &e.as_str(), QoS::AtLeastOnce, true)
                        .await;
                    if let Err(err) = r {
                        error!("error publishing: {}", err);
                    }
                }
            });
        })
        .await;

    let mqtt_client_ptr = Arc::new(mqtt_client.clone());
    let db_clone = db.clone();
    let ws = websocket(ws_settings, websocket_connections, mqtt_client_ptr.clone(), db_clone).await;
    if let Err(e) = ws {
        error!("error starting websocket: {:#?}", e);
        std::process::exit(1);
    } else {
        info!("Websocket started");
    };

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
        .and(warp::query::<utils_models::DeviceControllerQueries>())
        .and(with_db(db.clone()))
        .and_then(device_data_handler);

    let devices_controller_route = warp::path!("devices" / "data")
        .and(warp::get())
        .and(with_auth())
        .and(warp::query::<utils_models::DeviceControllerQueries>())
        .and(with_db(db.clone()))
        .and_then(devices_data_handler);

    let devices_status_route = warp::path!("devices" / "status")
        .and(warp::get())
        .and(with_auth())
        .and(warp::query::<utils_models::DeviceStatusQueries>())
        .and(with_db(db.clone()))
        .and_then(device_status_handler);

    let signin_route = warp::path!("auth" / "signin")
        .and(warp::post())
        .and(with_rate_limit(public_routes_rate_limit.clone()))
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(auth_signin_handler);

    let routes = root
        .or(api_doc)
        .or(swagger_ui)
        .or(signin_route)
        .or(devices_route)
        .or(device_controller_route)
        .or(devices_controller_route)
        .or(devices_status_route)
        .with(with_cors())
        .recover(errors::handle_rejection)
        .with(warp::wrap_fn(monitoring_wrapper));

    info!("starting http srv...");
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
