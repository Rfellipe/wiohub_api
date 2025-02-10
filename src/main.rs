mod db;
mod errors;
mod handlers;
mod models;
mod swagger;
mod utils;
mod security;

use db::get_db;
// use routes::all_routes;
use utoipa::OpenApi;
use warp::{self, Filter};
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

    let device_controller_route = warp::path!("devices" / "data")
        .and(warp::get())
        .and(warp::header::header("authorization"))
        .and(warp::query::<utils::DeviceControllerQueries>())
        .and(with_db(db.clone()))
        .and_then(handlers::device_data_handler);

    let signin_route = warp::path!("auth" / "signin")
        .and(warp::post())
        .and(with_rate_limit(public_routes_rate_limit.clone()))
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(handlers::auth_signin_handler);

    let routes = root
        .or(api_doc)
        .or(swagger_ui)
        .or(device_controller_route)
        .or(signin_route)
        .recover(errors::handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}

fn with_db(
    db: mongodb::Database,
) -> impl Filter<Extract = (mongodb::Database,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}
