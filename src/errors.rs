use argon2::password_hash::Error as ArgonError;
use bson::oid::Error as BsonError;
use mongodb::error::Error as MongoError;
use warp::reject::Reject;

#[derive(Debug)]
pub struct MongoRejection(pub MongoError);

#[derive(Debug)]
pub struct BsonRejection(pub BsonError);

#[derive(Debug)]
pub struct HashRejection(pub ArgonError);

#[derive(Debug)]
pub struct SignInError;

impl Reject for MongoRejection {}
impl Reject for BsonRejection {}
impl Reject for HashRejection {}
impl Reject for SignInError {}

pub async fn handle_rejection(
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
    } else if let Some(HashRejection(e)) = err.find() {
        Ok(warp::reply::with_status(
            format!("Error Encypting/Decrypting password: {}", e),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(SignInError) = err.find() {
        Ok(warp::reply::with_status(
            "Email or password incorrect".to_string(),
            warp::http::StatusCode::FORBIDDEN,
        ))
    } else {
        // Handle other errors
        Ok(warp::reply::with_status(
            "Internal server error".to_string(),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}
