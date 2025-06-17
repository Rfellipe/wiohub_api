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
    nest( (path = "/api", api = super::WiohubApi) ),
    tags( (name = "Wiohub API", description = "Endpoints usage") )
)]
pub struct WiohubDoc;

pub async fn serve_swagger(
    full_path: FullPath,
    tail: Tail,
    // config: Arc<Config<'static>>,
) -> Result<Box<dyn Reply + 'static>, Rejection> {
    let config = Arc::new(Config::from("/api/api-doc.json"));

    if full_path.as_str() == "/docs" {
        return Ok(Box::new(warp::redirect::found(Uri::from_static("/api/docs/"))));
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
