// Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
use axum::{extract::{State, Path, Query, Json}, http::StatusCode, Json as JsonResponse};
use serde_json::json;
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use crate::models::*;
use crate::auth::{create_token, Claims};

use uuid::Uuid;
use chrono::Utc;

pub async fn register_miner(
    State(state): State<std::sync::Arc<crate::state::AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<JsonResponse<RegisterResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
    if payload.cuzdan.len() < 10 {
        return Err((StatusCode::BAD_REQUEST, JsonResponse(json!({"error": "gecersiz cuzdan"}))));
    }

    match state.create_miner(&payload.cuzdan, &payload.makine_id, payload.eposta).await {
        Ok((miner_id, token)) => Ok(JsonResponse(RegisterResponse { miner_id, token })),
        Err(e) => {
            tracing::error!("Kayıt hatası: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(json!({"error": "kayıt başarısız"}))))
        }
    }
}

pub async fn heartbeat(
    State(state): State<std::sync::Arc<crate::state::AppState>>,
    axum::extract::Extension(claims): axum::extract::Extension<crate::auth::Claims>,
    Json(payload): Json<HeartbeatRequest>,
) -> Result<JsonResponse<HeartbeatResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
    let miner_id = &claims.sub;
    
    // Update heartbeat
    if let Err(e) = crate::state::AppState::update_heartbeat(&crate::state::AppState, &claims.sub).await {
        tracing::error!("Heartbeat güncelleme hatası: {}", e);
    }

    // Get next task
    let gorev = crate::state::AppState::get_gorev(&crate::state::AppState).await.ok().flatten();

    let response = HeartbeatResponse {
        ok: true,
        kilitli: false,
        gorev,
    };
    Ok(JsonResponse(response))
}

pub async fn get_gorev(
    State(state): State<std::sync::Arc<crate::state::AppState>>,
    axum::extract::Extension(claims): axum::extract::Extension<crate::auth::Claims>,
) -> Result<JsonResponse<Option<crate::models::GorevResponse>>, (StatusCode, JsonResponse<serde_json::Value>)> {
    let gorev = crate::state::AppState::get_gorev(&crate::state::AppState).await.ok().flatten();
    Ok(JsonResponse(gorev))
}

pub async fn submit_kanit(
    State(state): State<std::sync::Arc<crate::state::AppState>>,
    axum::extract::Extension(claims): axum::extract::Extension<crate::auth::Claims>,
    Json(payload): Json<KanitIstek>,
) -> Result<JsonResponse<KanitCevap>, (StatusCode, JsonResponse<serde_json::Value>)> {
    let miner_id = &claims.sub;
    
    match crate::state::AppState::submit_kanit(&crate::state::AppState, &claims.sub, payload).await {
        Ok(pay) => Ok(JsonResponse(KanitCevap {
            ok: true,
            pay,
            mesaj: "kanit kabul".to_string(),
        })),
        Err(e) => {
            tracing::error!("Kanıt hatası: {}", e);
            Err((StatusCode::BAD_REQUEST, JsonResponse(json!({"error": "kanıt reddedildi"}))))
        }
    }
}

pub async fn rag_query(
    State(state): State<std::sync::Arc<crate::state::AppState>>,
    Query(params): Query<RagIstek>,
    axum::extract::Extension(claims): axum::extract::Extension<crate::auth::Claims>,
) -> Result<JsonResponse<RagCevap>, (StatusCode, JsonResponse<serde_json::Value>)> {
    let miner_id = &claims.sub;
    
    // Check API hakkı
    let miner = crate::state::AppState::get_miner(&crate::state::AppState, &claims.sub).await
        .map_err(|_| (StatusCode::UNAUTHORIZED, JsonResponse(json!({"error": "miner bulunamadı"}))))?
        .ok_or((StatusCode::NOT_FOUND, JsonResponse(json!({"error": "miner bulunamadı"}))))?;

    if miner.api_hakki == 0 {
        return Err((StatusCode::PAYMENT_REQUIRED, JsonResponse(json!({"error": "api hakki bitti"}))));
    }

    // Decrease API hakki
    // TODO: Decrement api_hakki in DB

    // Use sorgu.py logic
    // For now return mock results
    Ok(JsonResponse(RagCevap {
        q: params.q,
        k: params.k,
        sonuclar: vec![],
    }))
}

pub async fn miner_status(
    State(state): State<std::sync::Arc<crate::state::AppState>>,
    axum::extract::Extension(claims): axum::extract::Extension<crate::auth::Claims>,
) -> Result<JsonResponse<MinerStatusResponse>, (StatusCode, JsonResponse<serde_json::Value>)> {
    let miner_id = &claims.sub;
    
    match crate::state::AppState::get_status(&crate::state::AppState, &claims.sub).await {
        Ok(status) => Ok(JsonResponse(status)),
        Err(e) => Err((StatusCode::NOT_FOUND, JsonResponse(json!({"error": e.to_string()})))),
    }
}

pub async fn broadcast_komut(
    State(state): State<std::sync::Arc<crate::state::AppState>>,
    Json(payload): Json<KomutIstek>,
) -> Result<JsonResponse<KomutCevap>, (StatusCode, JsonResponse<serde_json::Value>)> {
    // Only admin can broadcast commands
    // TODO: Check admin auth
    
    Ok(JsonResponse(KomutCevap {
        ok: true,
        komut_id: uuid::Uuid::new_v4().to_string(),
    }))
}