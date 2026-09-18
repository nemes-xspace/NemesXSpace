// Copyright 2026 NEMES-X. SPDX-License-Identifier: Apache-2.0.
//! worker.rs — Mining worker (GPU embed, int8 nicele)
//! wiki_embed_par.py:30 ile uyumlu niceleme, Md.112 doğrulama payı

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct GpuInfo {
    pub ad: String,
    pub vram_toplam: String,
    pub vram_bos: String,
    pub oneri_batch: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MiningStatus {
    pub aktif: bool,
    pub pay_saat: f32,
    pub toplam_pay: u64,
    pub son_gorev: Option<String>,
}

#[tauri::command]
pub fn get_gpu_info() -> GpuInfo {
    // TODO: nvidia-smi parse veya wgpu adapter info
    // Şimdilik placeholder — gerçekte `nvidia-smi --query-gpu=name,memory.total,memory.free --format=csv`
    GpuInfo {
        ad: "NVIDIA GeForce GTX 1080 Ti (merkez) / yerel GPU".into(),
        vram_toplam: "11264 MiB".into(),
        vram_bos: "~9GB".into(),
        oneri_batch: 48,
    }
}

#[tauri::command]
pub async fn start_mining(komuta_url: String, token: String) -> Result<String, String> {
    let url = komuta_url.clone();
    tokio::spawn(async move {
        mining_loop(&url, &token).await;
    });
    Ok("mining başlatıldı".into())
}

#[tauri::command]
pub fn stop_mining() -> Result<String, String> {
    // TODO: AtomicBool ile loop’u durdur
    Ok("mining durduruldu".into())
}

async fn mining_loop(url: &str, token: &str) {
    loop {
        // 1. Görev al
        let gorev = match fetch_gorev(url, token).await {
            Ok(g) => g,
            Err(e) => {
                eprintln!("[worker] gorev alınamadı: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                continue;
            }
        };
        // 2. Lokal embed (llama.cpp sidecar veya candle)
        // 3. int8 nicele + kanıt gönder
        let _ = gorev;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

async fn fetch_gorev(url: &str, token: &str) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/api/gorev", url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("gorev http {}", resp.status()));
    }
    resp.json().await.map_err(|e| e.to_string())
}

/// wiki_embed_par.py:30 ile birebir aynı niceleme
pub fn int8_nicele(vektor: &[f32]) -> (Vec<u8>, f32, f32) {
    let lo = vektor.iter().cloned().fold(f32::INFINITY, f32::min);
    let hi = vektor.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let hi = if (hi - lo).abs() < 1e-9 { lo + 1e-9 } else { hi };
    let skala = 255.0 / (hi - lo);
    let q = vektor
        .iter()
        .map(|v| ((v - lo) * skala).round() as u8)
        .collect();
    (q, lo, hi)
}
