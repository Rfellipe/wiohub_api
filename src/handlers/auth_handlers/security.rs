use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
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
        client_id: Some("".to_string()),
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

pub fn decode_jwt(
    authorization: String,
    secret: &str,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token = authorization.trim_start_matches("Bearer ");

    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let decoded = decode::<Claims>(token, &decoding_key, &Validation::default())?;

    Ok(decoded.claims)
}
