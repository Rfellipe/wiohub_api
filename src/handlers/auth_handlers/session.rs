use cookie::Cookie;
use warp::Filter;

use crate::errors::AppError;
use crate::errors::ErrorType;

pub async fn extract_headers(
    header_cookie: Option<String>,
    header_auth: Option<String>,
) -> Result<String, warp::Rejection> {
    if let Some(cookie_header) = header_cookie {
        let cookies = cookie_header.split("; ");
        for cookie in cookies {
            if let Ok(parsed_cookie) = Cookie::parse(cookie) {
                if parsed_cookie.name() == "session" {
                    return Ok(parsed_cookie.value().to_string());
                }
            }
        }
    }

    // If no valid cookie, check Authorization header
    if let Some(auth_header) = header_auth {
        if auth_header.starts_with("Bearer ") {
            return Ok(auth_header.to_string());
        }
    }

    let err_str = format!("Authorization token invalid or expired");
    let err = AppError {
        message: err_str,
        err_type: ErrorType::AuthError
    };
    Err(warp::reject::custom(err))
}

pub fn with_auth() -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    warp::header::optional::<String>("Cookie")
        .and(warp::header::optional::<String>("Authorization"))
        .and_then(extract_headers)
}
