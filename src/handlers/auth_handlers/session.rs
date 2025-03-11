use crate::errors::AuthError;
use cookie::Cookie;
use warp::Filter;

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

    // If neither is found, reject the request
    Err(warp::reject::custom(AuthError))
}

pub fn with_auth() -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    warp::header::optional::<String>("Cookie")
        .and(warp::header::optional::<String>("Authorization"))
        .and_then(extract_headers)
}
