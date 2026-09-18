// Copyright 2026 NEMES-X. SPDX-License-Identifier: Apache-2.0.
//! komuta.rs — Komuta merkezi istemcisi (kayıt, auth, durum)

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct KayitIstek {
    pub cuzdan: String,
    pub eposta: Option<String>,
    pub makine_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct KayitCevap {
    pub token: String,
    pub miner_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KomutaStatus {
    pub bagli: bool,
    pub miner_id: Option<String>,
    pub api_hakki: u64,
    pub bakiye: String,
}

#[tauri::command]
pub async fn kayit(
    komuta_url: String,
    cuzdan: String,
    eposta: Option<String>,
) -> Result<KomutaStatus, String> {
    let client = reqwest::Client::new();
    let makine_id = format!("miner-{}", uuid_simple());
    let resp = client
        .post(format!("{}/api/kayit", komuta_url))
        .json(&KayitIstek {
            cuzdan,
            eposta,
            makine_id,
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("kayit http {}", resp.status()));
    }
    let data: KayitCevap = resp.json().await.map_err(|e| e.to_string())?;
    // token’ı store’a yaz (tauri-plugin-store)
    Ok(KomutaStatus {
        bagli: true,
        miner_id: Some(data.miner_id),
        api_hakki: 0,
        bakiye: "0".into(),
    })
}

#[tauri::command]
pub async fn get_komuta_status(komuta_url: String, token: String) -> Result<KomutaStatus, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/api/status", komuta_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("status http {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

fn uuid_simple() -> String {
    // basit makine id — gerçekte `machine-uid` crate
    format!("{:x}", rand_u64())
}

fn rand_u64() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0).hash(&mut h);
    h.finish()
}
