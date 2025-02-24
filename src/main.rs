mod db;
mod errors;
mod handlers;
mod models;
mod swagger;
mod utils;
// mod security;

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::handlers::{
    auth_handlers::auth::auth_signin_handler,
    device_handlers::{device_data::devices_data_handler, device_status::device_status_handler},
};
use handlers::device_handlers::device_data::device_data_handler;
use utils::utils_functions::send_to_zabbix;
use utils::utils_models;

use db::get_db;
// use routes::all_routes;
use utoipa::OpenApi;
use warp::{self, filters::path::FullPath, Filter};
use warp_rate_limit::{with_rate_limit, RateLimitConfig};

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let config = swagger::doc_config();
    let db = get_db().await?;

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

    let ping = warp::path!("ping")
        .and(warp::get())
        .map(|| warp::reply::with_status("Pong", warp::http::StatusCode::OK));

    let device_controller_route = warp::path!("devices" / "data" / String)
        .and(warp::get())
        .and(warp::header::header("authorization"))
        .and(warp::query::<utils_models::DeviceControllerQueries>())
        .and(with_db(db.clone()))
        .and_then(device_data_handler);

    let devices_controller_route = warp::path!("devices" / "data")
        .and(warp::get())
        .and(warp::header::header("authorization"))
        .and(warp::query::<utils_models::DeviceControllerQueries>())
        .and(with_db(db.clone()))
        .and_then(devices_data_handler);

    let devices_status_route = warp::path!("devices" / "status")
        .and(warp::get())
        .and(warp::header::header("authorization"))
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
        .or(ping)
        .or(signin_route)
        .or(device_controller_route)
        .or(devices_controller_route)
        .or(devices_status_route)
        .recover(errors::handle_rejection)
        .with(warp::wrap_fn(monitoring_wrapper));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}

fn with_db(
    db: mongodb::Database,
) -> impl Filter<Extract = (mongodb::Database,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn monitoring_wrapper<F, T>(
    filter: F,
) -> impl Filter<Extract = (T,),> + Clone + Send + Sync + 'static
where
    F: Filter<Extract = (T,), Error = std::convert::Infallible> + Clone + Send + Sync + 'static,
    T: warp::Reply + Send + 'static,
{
    let start_time = Arc::new(Mutex::new(Instant::now()));

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
        .map(move |path: warp::path::FullPath, ip: Option<std::net::SocketAddr>, arg: T| {
            let elapsed = start_time.lock().unwrap().elapsed().as_secs_f64();
            let ip_str = ip.map(|addr| addr.ip().to_string()).unwrap_or_else(|| "unknown".to_string());

            let value = format!("Request to {} from {} took {:.3} seconds", path.as_str(), ip_str, elapsed);

            send_to_zabbix("http_request_duration", value);
            arg
        })
}
