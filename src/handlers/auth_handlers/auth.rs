use crate::errors::{AuthError, HashRejection, MongoRejection, SignInError};
use crate::handlers::auth_handlers::security::{generate_jwt, JWT_SECRET};
use crate::models::User;
use crate::utils::utils_models::SinginBody;
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use mongodb::{Collection, Database};
use warp::reply::Reply;
use warp_rate_limit::{add_rate_limit_headers, RateLimitInfo};

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
    rate_limit_info: RateLimitInfo,
    body: SinginBody,
    db: Database,
) -> Result<impl warp::Reply, warp::Rejection> {
    let user_coll: Collection<User> = db.collection("User");
    println!("{:#?}", body);
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
                    "role": 1,
                    "tenantId": 1
                })
                .build(),
        )
        .await
        .map_err(|e| warp::reject::custom(MongoRejection(e)))?;
    // .ok_or_else(|| warp::reject::custom(SignInError))?;

    if let Some(user) = user {
        let password = user.clone().password;
        let id = user.clone().id;
        let tenant_id = user.tenant_id.ok_or_else(|| warp::reject::not_found())?;

        let parsed_hash =
            PasswordHash::new(&password).map_err(|e| warp::reject::custom(HashRejection(e)))?;

        let password_match = Argon2::default()
            .verify_password(body.password.as_bytes(), &parsed_hash)
            .is_ok();

        if !password_match {
            return Err(warp::reject::custom(SignInError));
        }

        match generate_jwt(&id.to_string(), &tenant_id.to_string(), JWT_SECRET, 3600) {
            Ok(token) => {
                let mut response =
                    warp::reply::with_status(token, warp::http::StatusCode::OK).into_response();
                let _ = add_rate_limit_headers(response.headers_mut(), &rate_limit_info);
                return Ok(response);
            }
            Err(_e) => Err(warp::reject::reject()),
        }
    } else {
        println!("NO DATA FOUND FOR TYHOS EMAIL");
        Err(warp::reject::custom(AuthError))
    }
}
