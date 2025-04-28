use serde::Serialize;
// use warp::reply::Reply;
// use warp_rate_limit::{add_rate_limit_headers, RateLimitInfo};

use crate::errors::AppError;

pub fn respond<T: Serialize>(
    result: Result<T, AppError>,
    status: warp::http::StatusCode,
) -> Result<impl warp::Reply, warp::Rejection> {
    match result {
        Ok(response) => {
            Ok(warp::reply::with_status(warp::reply::json(&response), status))
        }
        Err(err) => {
            log::error!("Error while trying to respond: {}", err.to_string());
            Err(warp::reject::custom(err))
        }
    }
}
