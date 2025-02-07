mod db;
mod errors;
mod handlers;
mod models;
mod routes;
mod utils;

use std::sync::Arc;
use utoipa::OpenApi ;
use db::get_db;
use errors::{BsonRejection, MongoRejection};
use routes::all_routes;
use utoipa_swagger_ui::Config;
// use warp::{self, Filter};
use warp::{
    http::Uri,
    hyper::{Response, StatusCode},
    path::{FullPath, Tail},
    Filter, Rejection, Reply,
};

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
    let config = Arc::new(Config::from("/api-doc.json"));

    #[derive(OpenApi)]
    #[openapi(
        nest(
            (path = "/", api = handlers::WiohubApi)
        ),
        tags(
            (name = "Wiohub Api", description = "~")
        )
    )]
    struct WiohubDoc;

    let db = get_db().await?;

    let api_doc = warp::path("api-doc.json")
        .and(warp::get())
        .map(|| warp::reply::json(&WiohubDoc::openapi()));

    let swagger_ui = warp::path("docs")
        .and(warp::get())
        .and(warp::path::full())
        .and(warp::path::tail())
        .and(warp::any().map(move || config.clone()))
        .and_then(serve_swagger);

    // let api = all_routes(db).or(api_doc).(swagger_ui);
    let api = api_doc.or(swagger_ui);

    let routes = api.or(all_routes(db)).with(warp::log("devices")).recover(handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    Ok(())
}


async fn serve_swagger(
    full_path: FullPath,
    tail: Tail,
    config: Arc<Config<'static>>,
) -> Result<Box<dyn Reply + 'static>, Rejection> {
    if full_path.as_str() == "/docs" {
        return Ok(Box::new(warp::redirect::found(Uri::from_static(
            "/docs/",
        ))));
    }

    let path = tail.as_str();
    match utoipa_swagger_ui::serve(path, config) {
        Ok(file) => {
            if let Some(file) = file {
                Ok(Box::new(
                    Response::builder()
                        .header("Content-Type", file.content_type)
                        .body(file.bytes),
                ))
            } else {
                Ok(Box::new(StatusCode::NOT_FOUND))
            }
        }
        Err(error) => Ok(Box::new(
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string()),
        )),
    }
}
