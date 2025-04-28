#[allow(unused_imports)]
#[allow(dead_code)]

use serde::Serialize;
use std::fmt::{self};
use warp::reject::Reject;

#[derive(Debug, Clone)]
pub enum ErrorType {
    // INotFound,
    Internal,
    BadRequest,
    AuthError,
    MongoError,
}

#[derive(Debug, Clone)]
pub struct AppError {
    pub err_type: ErrorType,
    pub message: String,
}

impl AppError {
    // pub fn new(message: &str, err_type: ErrorType) -> AppError {
    //     AppError {
    //         message: message.to_string(),
    //         err_type,
    //     }
    // }

    pub fn to_http_status(&self) -> warp::http::StatusCode {
        match self.err_type {
            // IErrorType::NotFound => warp::http::StatusCode::NOT_FOUND,
            ErrorType::Internal => warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ErrorType::BadRequest => warp::http::StatusCode::BAD_REQUEST,
            ErrorType::AuthError => warp::http::StatusCode::FORBIDDEN,
            ErrorType::MongoError => warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    // fn from_mongo_err(err: mongodb::error::ErrorKind, context: &str) -> AppError {
    //     AppError::new(
    //         format!("{}: {}", context, err.to_string()).as_str(),
    //         match err {
    //             mongodb::error::ErrorKind::InvalidArgument { .. } => ErrorType::BadRequest,
    //             mongodb::error::ErrorKind::BsonDeserialization { .. } => ErrorType::BadRequest,
    //             mongodb::error::ErrorKind::BsonSerialization { .. } => ErrorType::BadRequest,
    //             _ => ErrorType::Internal,
    //         },
    //     )
    // }
}

impl std::error::Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Reject for AppError {}

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

pub async fn handle_rejection(
    err: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = warp::http::StatusCode::NOT_FOUND;
        message = "Not Found";
    } else if let Some(app_err) = err.find::<AppError>() {
        code = app_err.to_http_status();
        message = app_err.message.as_str();
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        code = warp::http::StatusCode::BAD_REQUEST;
        message = "Invalid Body";
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        code = warp::http::StatusCode::METHOD_NOT_ALLOWED;
        message = "Method Not Allowed";
    } else {
        // In case we missed something - log and respond with 500
        eprintln!("unhandled rejection: {:?}", err);
        code = warp::http::StatusCode::INTERNAL_SERVER_ERROR;
        message = "Unhandled rejection";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}

pub fn mongo_error(e: mongodb::error::Error) -> AppError {
    AppError {
        err_type: ErrorType::MongoError,
        message: format!("Mongo Error: {:#?}", e),
    }
}

pub fn bson_datetime_error(e: bson::datetime::Error) -> AppError {
    AppError {
        err_type: ErrorType::BadRequest,
        message: format!("Date format incorrect: {:#?}", e),
    }
}
