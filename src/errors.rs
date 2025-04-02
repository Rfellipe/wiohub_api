use argon2::password_hash::Error as ArgonError;
// use bson::oid::Error as BsonError;
    use bson::datetime::Error as DateTimeError;
use mongodb::error::Error as MongoError;
use warp::{reject::Reject, reply::Reply};
use warp_rate_limit::{add_rate_limit_headers, get_rate_limit_info, RateLimitRejection};

#[derive(Debug)]
pub struct MongoRejection(pub MongoError);

// #[derive(Debug)]
// pub struct BsonRejection(pub BsonError);

#[derive(Debug)]
pub struct BsonDateTimeRejection(pub DateTimeError);

#[derive(Debug)]
pub struct HashRejection(pub ArgonError);

#[derive(Debug)]
pub struct SignInError;

#[derive(Debug)]
pub struct AuthError;

// #[derive(Debug)]
// pub struct NoRecordFound;

impl Reject for MongoRejection {}
// impl Reject for BsonRejection {}
impl Reject for BsonDateTimeRejection {}
impl Reject for HashRejection {}
impl Reject for SignInError {}
impl Reject for AuthError {}
// impl Reject for NoRecordFound {}

pub async fn handle_rejection(
    err: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    if let Some(MongoRejection(e)) = err.find() {
        // Handle MongoDB errors
        Ok(warp::reply::with_status(
            format!("MongoDB error: {}", e),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response())
    } else if let Some(HashRejection(e)) = err.find() {
        Ok(warp::reply::with_status(
            format!("Error Encypting/Decrypting password: {}", e),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response())
    } else if let Some(SignInError) = err.find() {
        Ok(warp::reply::with_status(
            "Email or password incorrect".to_string(),
            warp::http::StatusCode::FORBIDDEN,
        )
        .into_response())
    } else if let Some(AuthError) = err.find() {
        Ok(warp::reply::with_status(
            "Authorization token invalid or expired".to_string(),
            warp::http::StatusCode::FORBIDDEN,
        )
        .into_response())
    } else if let Some(rate_limit_rejection) = err.find::<RateLimitRejection>() {
        let info = get_rate_limit_info(rate_limit_rejection);

        let message = format!("Rate limit exceeded. Try again after {}.", info.retry_after);

        let mut response =
            warp::reply::with_status(message, warp::http::StatusCode::TOO_MANY_REQUESTS)
                .into_response();

        let _ = add_rate_limit_headers(response.headers_mut(), &info);

        Ok(response)
    }  else if let Some(BsonDateTimeRejection(e)) = err.find() {
       // Handle Bson errors
       Ok(warp::reply::with_status(
           format!("Bson Date Time error: {}", e),
           warp::http::StatusCode::INTERNAL_SERVER_ERROR,
       ).into_response())
   } else {
        // Handle other errors
        Ok(warp::reply::with_status(
            "Internal server error".to_string(),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response())
    }
}

/*
   else if let Some(NoRecordFound) = err.find() {
        Ok(warp::reply::with_status(
            "No record Found".to_string(),
            warp::http::StatusCode::BAD_REQUEST,
        )
        .into_response())
    }

    else if let Some(BsonRejection(e)) = err.find() {
       // Handle Bson errors
       Ok(warp::reply::with_status(
           format!("Bson error: {}", e),
           warp::http::StatusCode::INTERNAL_SERVER_ERROR,
       ).into_response())
    } 
* */
