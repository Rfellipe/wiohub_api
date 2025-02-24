use warp::Filter;
use cookie::Cookie;

pub fn with_auth_cookie() -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    warp::header::optional::<String>("cookie")
        .and_then(|cookie_header: Option<String>| async move {
            if let Some(cookie_header) = cookie_header {
                let cookies = cookie_header.split("; ");
                for cookie in cookies {
                    if let Ok(parsed_cookie) = Cookie::parse(cookie) {
                        if parsed_cookie.name() == "session" {
                            return Ok(parsed_cookie.value().to_string());
                        }
                    }
                }
            }
            Err(warp::reject::custom(AuthError))
        })
}

#[derive(Debug)]
struct AuthError;

impl warp::reject::Reject for AuthError {}