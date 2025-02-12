use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub static JWT_SECRET: &'static str = "wiohub-secret";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    pub tenant: String,
    pub exp: usize, // Expiration timestamp in seconds
}

pub fn generate_jwt(
    user_id: &str,
    tenant_id: &str,
    secret: &str,
    expires_in: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
        + expires_in;

    let claims = Claims {
        id: user_id.to_string(),
        tenant: tenant_id.to_string(),
        exp: expiration as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?;

    Ok(token)
}

pub fn decode_jwt(authorization: String, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token = authorization.trim_start_matches("Bearer ");

    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let decoded = decode::<Claims>(token, &decoding_key, &Validation::default())?;

    Ok(decoded.claims)
}
