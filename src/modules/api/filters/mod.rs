use crate::{
    shared::errors::{AppError, ErrorType},
    shared::{
        db::{DBAccessManager, PgPool},
        zabbix::send_to_zabbix,
    },
};
use reqwest::header::HeaderMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc, time::Instant};
use warp::{self, filters::path::FullPath, http::Method, reply::Response, Filter};

#[allow(unused)]
pub fn with_json_body<T: DeserializeOwned + Send>(
) -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

#[allow(unused)]
pub fn with_cors(origin: &str) -> warp::filters::cors::Cors {
    warp::cors()
        .allow_origin(origin)
        .allow_headers(vec!["Content-Type", "Authorization"])
        .allow_methods(&[Method::GET, Method::POST])
        .allow_credentials(true)
        .build()
}

#[allow(unused)]
pub fn monitoring_wrapper<F, T>(
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
}

impl warp::Reply for UserInfo {
    fn into_response(self) -> warp::reply::Response {
        Response::new(serde_json::to_string::<UserInfo>(&self).unwrap().into())
    }
}

async fn check_user(token: String) -> Result<UserInfo, AppError> {
    let client = reqwest::Client::new();

    let supabase_url = "https://zqvkvfoyhtneccxmrukp.supabase.co/auth/v1/user";
    let mut map_headers = HeaderMap::new();

    map_headers.insert(
        "apikey",
        std::env::var("SUPABASE_PUBLIC").unwrap().parse().unwrap(),
    );
    map_headers.insert(
        "Authorization",
        format!("Bearer {}", token).as_str().parse().unwrap(),
    );
    let res = client
        .get(supabase_url)
        .headers(map_headers)
        .send()
        .await
        .map_err(|e| {
            log::error!("{}", e.to_string());
            AppError::new("Error sending request", ErrorType::Internal)
        })?
        .error_for_status()
        .map_err(|e| {
            log::error!("{}", e.to_string());
            AppError::new(
                "Authorization token invalid or expired",
                ErrorType::AuthError,
            )
        })?
        .json::<UserInfo>()
        .await
        .map_err(|e| {
            log::error!("{}", e.to_string());
            AppError {
                err_type: ErrorType::Internal,
                message: format!("Error deserializing user: {:#?}", e),
            }
        })?;

    Ok(res)
}

pub fn with_auth() -> impl Filter<Extract = (UserInfo,), Error = warp::Rejection> + Clone {
    warp::header::<String>("Authorization").and_then(async move |header_auth: String| {
        if header_auth.clone().starts_with("Bearer ") {
            let token: String = header_auth.trim_start_matches("Bearer ").parse().unwrap();
            let user_info = check_user(token).await;
            match user_info {
                Ok(res) => {
                    log::debug!("Got user: {:#?}", res);
                    return Ok(res);
                }
                Err(e) => {
                    return Err(warp::reject::custom(e));
                }
            }
        } else {
            return Err(warp::reject::custom(AppError::new(
                "Header Error",
                ErrorType::AuthError,
            )));
        };
    })
}

pub fn with_db_access_manager(
    pool: PgPool,
) -> impl Filter<Extract = (DBAccessManager,), Error = warp::Rejection> + Clone {
    warp::any()
        .map(move || pool.clone())
        .and_then(|pool: PgPool| async move {
            match pool.get() {
                Ok(conn) => Ok(DBAccessManager::new(conn)),
                Err(err) => Err(warp::reject::custom(AppError::new(
                    format!("Error getting connection from pool: {}", err.to_string()).as_str(),
                    ErrorType::Internal,
                ))),
            }
        })
}
