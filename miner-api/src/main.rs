// Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::{HeaderMap, StatusCode},
    middleware,
    response::Json as JsonResponse,
    routing::{get, post},
    Router,
};
use axum::response::IntoResponse;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use miner_core::{
    EmbedClient, Gorev, Kanit, KanitGonderici, GorevAlici, 
    MiningStats, WorkerState, int8_nicele,
};

use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use base64::{engine::general_purpose::STANDARD as B64_STANDARD, Engine as _};

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

use tracing::{info, error, warn};

mod auth;
mod handlers;
mod models;
mod state;

use handlers::*;
use models::*;
use state::AppState;

const JWT_SECRET: &[u8] = b"nemes-jwt-secret-change-in-production";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Database
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:/srv/beyin/beyin.db".to_string());
    let pool = SqlitePool::connect(&db_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Ed25519 keys for command signing
    let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
    let verifying_key = signing_key.verifying_key();

    let state = Arc::new(AppState::new(pool).await?);

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/kayit", post(register_miner))
        .route("/api/heartbeat", post(heartbeat))
        .route("/api/gorev", get(get_gorev))
        .route("/api/kanit", post(submit_kanit))
        .route("/api/rag", get(rag_query))
        .route("/api/status", get(miner_status))
        .route("/api/komut", post(broadcast_komut))
        .layer(middleware::from_fn_with_state(state.clone(), auth::auth_middleware))
        .layer(middleware::from_fn(cors_middleware))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8787").await?;
    info!("NEMES Komuta API listening on http://0.0.0.0:8787");
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    JsonResponse(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn cors_middleware<B>(req: axum::http::Request<B>, next: axum::middleware::Next<B>) -> impl IntoResponse {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    headers.insert("Access-Control-Allow-Methods", "GET, POST, OPTIONS".parse().unwrap());
    headers.insert("Access-Control-Allow-Headers", "Content-Type, Authorization".parse().unwrap());
    response
}