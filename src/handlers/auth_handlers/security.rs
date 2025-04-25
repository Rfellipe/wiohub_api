use crate::errors::{AuthError, MongoRejection};
use crate::models::User;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use mongodb::{
    bson::doc,
    options::FindOneOptions,
    Collection, Database,
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub static JWT_SECRET: &'static str = "9971dc00-943b-11ec-9a4f-4aabc8e81102";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    pub tenant: Option<String>,
    pub avatar: Option<String>,
    pub role: Option<String>,
    #[serde(rename = "clientId")]
    pub client_id: Option<String>,
    pub expires: Option<String>,
    pub iat: usize,
    pub exp: usize, // Expiration timestamp in seconds
}

pub fn generate_jwt(
    user_id: &str,
    tenant_id: &str,
    client_id: &str,
    secret: &str,
    expires_in: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
        + expires_in;

    let issued = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let claims = Claims {
        id: user_id.to_string(),
        tenant: Some(tenant_id.to_string()),
        avatar: Some("".to_string()),
        role: Some("".to_string()),
        client_id: Some(client_id.to_string()),
        expires: Some("".to_string()),
        iat: issued as usize,
        exp: expiration as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?;

    Ok(token)
}

pub async fn decode_jwt(
    authorization: String,
    secret: &str,
    db: Database,
) -> Result<Claims, warp::Rejection> {
    if authorization.starts_with("Bearer ") {
        let api_key = authorization.trim_start_matches("Bearer ");

        let user_coll: Collection<User> = db.collection("User");
        let user = user_coll
            .find_one(
                doc! {
                    "apiKey": api_key
                },
                FindOneOptions::builder()
                    .projection(doc! {
                        "name": 1,
                        "email": 1,
                        "phone": 1,
                        "password": 1,
                        "role": 1,
                        "clientId": 1,
                        "tenantId": 1
                    })
                    .build(),
            )
            .await
            .map_err(|e| warp::reject::custom(MongoRejection(e)))?;

        if let Some(user) = user {
            let expiration = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
                + 3600;

            let issued = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();

            let claims = Claims {
                id: user.id.to_string(),
                tenant: user.tenant_id.map(|id| id.to_string()),
                avatar: user.avatar.map(|a| a.to_string()),
                role: None,
                client_id: user.client_id.map(|id| id.to_string()),
                expires: None, // Change as needed
                iat: issued as usize,
                exp: expiration as usize,
            };

            Ok(claims)
        } else {
            println!("error1");
            Err(warp::reject::custom(AuthError))
        }
    } else {
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        println!("error2");
        match decode::<Claims>(&authorization, &decoding_key, &Validation::default()) {
            Ok(decoded) => Ok(decoded.claims),
            Err(_) => Err(warp::reject::custom(AuthError)), // Return an authentication error if decoding fails
        }
    }
}
