use crate::errors::{AppError, ErrorType};
use crate::handlers::auth_handlers::security::{generate_jwt, JWT_SECRET};
use crate::utils::responder::respond;
use crate::utils::utils_models::SinginBody;
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use bson::oid::ObjectId;
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use serde_json::json;
use warp::http::StatusCode;
use warp_rate_limit::RateLimitInfo;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub password: String,
    pub client_id: ObjectId,
    pub tenant_id: String,
}

#[utoipa::path(
        get,
        path = "auth/signin",
        params(SinginBody),
        responses(
            (status = 200, description = "JWT received", body = String),
            (status = 500, description = "Internal Server Error", body = String),
        )
    )
]
pub async fn auth_signin_handler(
    _rate_limit_info: RateLimitInfo,
    body: SinginBody,
    db: Database,
) -> Result<impl warp::Reply, warp::Rejection> {
    let user_coll: Collection<User> = db.collection("User");
    let user = user_coll
        .find_one(
            doc! {
                "email": body.email
            },
            FindOneOptions::builder()
                .projection(doc! {
                    "name": 1,
                    "email": 1,
                    "phone": 1,
                    "password": 1,
                    "clientId": 1,
                    "tenantId": 1
                })
                .build(),
        )
        .await;

    match user {
        Ok(user) => {
            if let Some(user) = user {
                let password = user.clone().password;
                let user_id = user.clone().id;
                let client_id = user.clone().client_id;
                let tenant_id = user.clone().tenant_id;

                let parsed_hash = PasswordHash::new(&password);

                let hash = match parsed_hash {
                    Ok(hash) => hash,
                    Err(err) => {
                        let err_str = format!("Internal Error: {:#?}", err);
                        let e = AppError {
                            message: err_str,
                            err_type: ErrorType::Internal
                        };
                        return Err(warp::reject::custom(e))
                    }
                };

                let password_match = Argon2::default()
                    .verify_password(body.password.as_bytes(), &hash)
                    .is_ok();

                if !password_match {
                    let err_str = format!("Passwords don't match");
                    let e = AppError {
                        message: err_str,
                        err_type: ErrorType::BadRequest
                    };
                    return Err(warp::reject::custom(e))
                }
                
                match generate_jwt(
                    &ObjectId::to_string(&user_id),
                    &tenant_id,
                    &ObjectId::to_string(&client_id),
                    JWT_SECRET,
                    3600,
                ) {
                    Ok(token) => {
                        let res = json!({
                            "token": token
                        });
                        respond(Ok(res), StatusCode::OK)
                    }
                    Err(err) => {
                        let err_str = format!("Error finding user: {:#?}", err);
                        let e = AppError {
                            err_type: ErrorType::BadRequest,
                            message: err_str,
                        };
                        return Err(warp::reject::custom(e))
                    }
                }
            } else {
                let err_str = format!("User not found!");
                let e = AppError {
                    message: err_str,
                    err_type: ErrorType::BadRequest,
                };
                Err(warp::reject::custom(e))
            }
        }
        Err(err) => {
            let err_str = format!("Internal Error: {:#?}", err);
            let e = AppError {
                message: err_str,
                err_type: ErrorType::Internal,
            };
            Err(warp::reject::custom(e))
        }
    }
}
