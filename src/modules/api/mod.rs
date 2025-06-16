mod filters;
mod handlers;
mod models;
mod responder;
mod routes;
mod swagger;

use crate::shared::db::PgPool;
use crate::shared::errors::handle_rejection;
use tokio::task::JoinHandle;
use utoipa::OpenApi;
use warp::Filter;

#[derive(OpenApi)]
#[openapi(paths(handlers::devices_data_handler))]
pub struct WiohubApi;

pub async fn start_api(pool: PgPool) -> JoinHandle<()> {
    let task = tokio::spawn(async move {
        // Generate API docs
        // let config = swagger::doc_config();
        let api_doc = warp::path("api-doc.json")
            .and(warp::get())
            .map(|| warp::reply::json(&swagger::WiohubDoc::openapi()));

        // let swagger_ui = warp::path("docs")
        //     .and(warp::get())
        //     .and(warp::path::full())
        //     .and(warp::path::tail())
        //     .and(warp::any().map(move || config.clone()))
        //     .and_then(swagger::serve_swagger);

        // Generate routes
        let routes = warp::path!("api" / ..)
            .and(
                api_doc
                    // .or(swagger_ui)
                    .or(routes::devices_route(pool.clone())),
            )
            .recover(handle_rejection);

        log::info!("Starting API...");
        warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
    });

    task
}
