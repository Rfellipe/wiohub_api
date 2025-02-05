mod db;
mod errors;
mod handlers;
mod models;
mod routes;
mod utils;

use db::get_db;
use errors::{BsonRejection, MongoRejection};
use routes::all_routes;
use warp::{self, Filter};

async fn handle_rejection(
    err: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    if let Some(MongoRejection(e)) = err.find() {
        // Handle MongoDB errors
        println!("{}", e);
        Ok(warp::reply::with_status(
            format!("MongoDB error: {}", e),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(BsonRejection(e)) = err.find() {
        // Handle Bson errors
        Ok(warp::reply::with_status(
            format!("Bson error: {}", e),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else {
        // Handle other errors
        Ok(warp::reply::with_status(
            "Internal server error".to_string(),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let db = get_db().await?;

    let api = all_routes(db);

    let routes = api.with(warp::log("devices")).recover(handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}
