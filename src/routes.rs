use super::handlers::{device_data_handler, hello_handler};
use super::utils::DeviceControllerQueries;
use warp::Filter;

pub fn all_routes(
    db: mongodb::Database,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    device_controller_route(db.clone()).or(hello())
}

fn device_controller_route(
    db: mongodb::Database,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("devices" / "data")
        .and(warp::get())
        .and(warp::query::<DeviceControllerQueries>())
        .and(with_db(db))
        .and_then(device_data_handler)
}

fn hello() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("hello" / String)
        .and(warp::get())
        .and_then(hello_handler)
}

fn with_db(
    db: mongodb::Database,
) -> impl Filter<Extract = (mongodb::Database,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

