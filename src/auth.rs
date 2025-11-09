use crate::models::Dealer;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub dealer_id: i32,
    pub email: String,
    pub exp: usize,
}

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    hash(password, DEFAULT_COST).map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))
}

pub fn verify_password(password: &str, hash: &str) -> anyhow::Result<bool> {
    verify(password, hash).map_err(|e| anyhow::anyhow!("Failed to verify password: {}", e))
}

pub fn create_token(dealer_id: i32, email: &str) -> anyhow::Result<String> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(30))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        dealer_id,
        email: email.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| anyhow::anyhow!("Failed to create token: {}", e))
}

pub fn verify_token(token: &str) -> anyhow::Result<Claims> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

pub async fn get_dealer_from_token(token: &str, pool: &PgPool) -> anyhow::Result<Dealer> {
    let claims = verify_token(token)?;
    let dealer = sqlx::query_as::<_, Dealer>(
        "SELECT id, name, email, password_hash, zip_code, created_at FROM dealers WHERE id = $1"
    )
    .bind(claims.dealer_id)
    .fetch_one(pool)
    .await?;

    Ok(dealer)
}

