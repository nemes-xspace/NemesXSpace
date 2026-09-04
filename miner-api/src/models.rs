// Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::miner_core::{Gorev, Kanit, MiningStats, WorkerState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub cuzdan: String,
    pub makine_id: String,
    pub eposta: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub miner_id: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub ts: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub ok: bool,
    pub kilitli: bool,
    pub gorev: Option<GorevResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GorevResponse {
    pub tip: String,
    pub aciklama: String,
    pub sira_log: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanitRequest {
    pub gorev_id: String,
    pub madde_id: usize,
    pub v_int8_b64: String,
    pub v_min: f32,
    pub v_max: f32,
    pub imza: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanitResponse {
    pub ok: bool,
    pub pay: u64,
    pub mesaj: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagRequest {
    pub q: String,
    #[serde(default = "default_k")]
    k: usize,
}

fn default_k() -> usize { 5 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagResponse {
    pub q: String,
    pub k: usize,
    pub sonuclar: Vec<RagSonuc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagSonuc {
    pub baslik: String,
    pub ozet: String,
    pub skor: f32,
    pub kaynak: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerStatusResponse {
    pub miner_id: String,
    pub pay: u64,
    pub api_hakki: u64,
    pub bakiye: f64,
    pub bagli: bool,
    pub son_heartbeat: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KomutRequest {
    pub tip: String,           // "embed" | "rag" | "dur" | "model_sec"
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KomutResponse {
    pub ok: bool,
    pub komut_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerStatusResponse {
    pub pay: u64,
    pub api_hakki: u64,
    pub bakiye: f64,
    pub bagli: bool,
    pub son_heartbeat: u64,
    pub miner_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KomutDagitRequest {
    pub corpus: String,
    pub shard: usize,
    pub miners: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KomutDagitResponse {
    pub ok: bool,
    pub dagitilan: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerInfo {
    pub id: String,
    pub cuzdan: String,
    pub makine_id: String,
    pub pay: u64,
    pub api_hakki: u64,
    pub bakiye: f64,
    pub son_heartbeat: u64,
    pub olusturma_zamani: DateTime<Utc>,
}