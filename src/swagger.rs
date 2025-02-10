use super::handlers;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::Config;
use warp::{
    http::Uri,
    hyper::{Response, StatusCode},
    path::{FullPath, Tail},
    Rejection, Reply,
};

#[derive(OpenApi)]
#[openapi(
        nest(
            (path = "/", api = handlers::WiohubApi)
        ),
        tags(
            (name = "Wiohub Api", description = "~")
        )
    )]
pub struct WiohubDoc;

pub fn doc_config() -> Arc<Config<'static>> {
    let config = Arc::new(Config::from("/api-doc.json"));

    config
}

pub async fn serve_swagger(
    full_path: FullPath,
    tail: Tail,
    config: Arc<Config<'static>>,
) -> Result<Box<dyn Reply + 'static>, Rejection> {
    if full_path.as_str() == "/docs" {
        return Ok(Box::new(warp::redirect::found(Uri::from_static("/docs/"))));
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
