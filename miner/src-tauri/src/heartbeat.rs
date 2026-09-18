// Copyright 2026 NEMES-X. SPDX-License-Identifier: Apache-2.0.
//! heartbeat.rs — Komuta kalp atışı (45sn, Md.113)
//! Merkez olmadan miner kilitlenir. Ed25519 imzalı komut doğrular.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use tokio::time::{sleep, Duration};

static KILITLI: AtomicBool = AtomicBool::new(true);
static KOMUTA_BAGLI: AtomicBool = AtomicBool::new(false);
static HEARTBEAT_INTERVAL: Duration = Duration::from_secs(45);
static HEARTBEAT_GUARD: OnceLock<Mutex<bool>> = OnceLock::new();

#[derive(Serialize, Deserialize, Clone)]
pub struct HeartbeatStatus {
    pub bagli: bool,
    pub kilitli: bool,
    pub son_heartbeat: Option<String>,
    pub komut_id: Option<String>,
}

/// UI’dan durum sorgusu
#[tauri::command]
pub fn get_status() -> HeartbeatStatus {
    HeartbeatStatus {
        bagli: KOMUTA_BAGLI.load(Ordering::SeqCst),
        kilitli: KILITLI.load(Ordering::SeqCst),
        son_heartbeat: None,
        komut_id: None,
    }
}

/// Heartbeat döngüsünü başlat (45sn, Md.113) — idempotent, tek loop
#[tauri::command]
pub async fn start_heartbeat(komuta_url: String, token: String) -> Result<String, String> {
    let guard = HEARTBEAT_GUARD.get_or_init(|| Mutex::new(false));
    {
        let mut started = guard.lock().map_err(|_| "lock zehirlendi".to_string())?;
        if *started {
            return Ok("heartbeat zaten çalışıyor".into());
        }
        *started = true;
    }
    let url = komuta_url.trim_end_matches('/').to_string();
    tokio::spawn(async move {
        let mut ardarda_hata: u32 = 0;
        loop {
            match send_heartbeat(&url, &token).await {
                Ok(_) => {
                    KOMUTA_BAGLI.store(true, Ordering::SeqCst);
                    KILITLI.store(false, Ordering::SeqCst);
                    ardarda_hata = 0;
                }
                Err(e) => {
                    ardarda_hata += 1;
                    KOMUTA_BAGLI.store(false, Ordering::SeqCst);
                    KILITLI.store(true, Ordering::SeqCst);
                    eprintln!("[heartbeat] hata {}/3: {}", ardarda_hata, e);
                    if ardarda_hata >= 3 {
                        eprintln!("[heartbeat] 3 ardarda hata — worker durdurulacak (Md.60)");
                        // TODO: crate::worker::stop_mining() çağrısı eklenecek
                    }
                }
            }
            sleep(HEARTBEAT_INTERVAL).await;
        }
    });
    Ok("heartbeat başlatıldı (45sn)".into())
}

async fn send_heartbeat(url: &str, token: &str) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("NEMES-Miner/0.2.0")
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(format!("{}/api/heartbeat", url.trim_end_matches('/')))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "ts": chrono::Utc::now().to_rfc3339() }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("heartbeat http {}", resp.status()));
    }
    // Komut varsa imzasını doğrula (merkez public key ile)
    // Beklenen response: { ok:true, komut:{id, payload, imza}, pubkey }
    if let Ok(json) = resp.json::<serde_json::Value>().await {
        if let Some(komut) = json.get("komut") {
            if let (Some(payload), Some(imza), Some(pubkey)) = (
                komut.get("payload").and_then(|v| v.as_str()),
                komut.get("imza").and_then(|v| v.as_str()),
                json.get("pubkey").and_then(|v| v.as_str()),
            ) {
                if !dogrula_imza(payload.as_bytes(), imza, pubkey) {
                    return Err("komut imzasi dogrulamasi basarisiz".into());
                }
            }
        }
    }
    Ok(())
}

/// Merkez imzasını doğrula (komuta public key ile) — Ed25519 base64
pub fn dogrula_imza(mesaj: &[u8], imza_b64: &str, pubkey_b64: &str) -> bool {
    use base64::{engine::general_purpose::STANDARD as B64, Engine};
    use ed25519_dalek::{Signature, VerifyingKey};

    let sig_bytes = match B64.decode(imza_b64.trim()) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let pub_bytes = match B64.decode(pubkey_b64.trim()) {
        Ok(b) => b,
        Err(_) => return false,
    };
    if sig_bytes.len() != 64 || pub_bytes.len() != 32 {
        return false;
    }
    let sig_arr: [u8; 64] = match sig_bytes.try_into() {
        Ok(a) => a,
        Err(_) => return false,
    };
    let pub_arr: [u8; 32] = match pub_bytes.try_into() {
        Ok(a) => a,
        Err(_) => return false,
    };
    let vk = match VerifyingKey::from_bytes(&pub_arr) {
        Ok(k) => k,
        Err(_) => return false,
    };
    let sig = Signature::from_bytes(&sig_arr);
    vk.verify_strict(mesaj, &sig).is_ok()
}
