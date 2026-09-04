// Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use axum::body::Body;
use jsonwebtoken::{decode, decode_header, decode_token, decode, DecodingKey, Validation, TokenData};
use jsonwebtoken::TokenData;
use serde::{Deserialize, Serialize};

use crate::state::AppState;

use crate::models::Claims;

pub async fn auth_middleware(
    State(state): State<crate::state::AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token_data = match validate_token(&token) {
        Ok(data) => data,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Add claims to request extensions for handlers to use
    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}

fn validate_token(token: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_secret(b"nemes-jwt-secret-change-in-production");
    let mut validation = Validation::default();
    validation.validate_exp = true;
    
    decode::<Claims>(token, &DecodingKey::from_secret(b"nemes-jwt-secret-change-in-production"), &Validation::default())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // miner_id
    pub cuzdan: String,   // cüzdan adresi
    pub exp: usize,       // expiration time
    pub iat: usize,       // issued at
}

pub fn create_token(miner_id: &str, cuzdan: &str) -> Result<String, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use chrono::{Utc, Duration};
    
    let now = chrono::Utc::now();
    let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
    let iat = Utc::now().timestamp() as usize;

    let claims = Claims {
        sub: miner_id.to_string(),
        cuzdan: miner_id.to_string(),
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &Claims {
            sub: miner_id.to_string(),
            cuzdan: miner_id.to_string(),
            exp: (chrono::Utc::now() + Duration::hours(24)).timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
        },
        &jsonwebtoken::EncodingKey::from_secret(b"nemes-jwt-secret-change-in-production"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_token() {
        let token = create_token("miner123", "TUc1x...").unwrap();
        assert!(!token.is_empty());
    }
}