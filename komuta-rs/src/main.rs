// Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row, query_as};
use sqlx::sqlite::SqlitePoolOptions;
use nemes_core::shard::ShardIlan;
use nemes_p2p::{NetworkEvent, P2PConfig, P2PNode};
use rand::Rng;
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::signal;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info, warn};
use uuid::Uuid;
use base64::{engine::general_purpose::STANDARD, Engine as _};

// --- Config ---
const COIN_UNIT: i64 = 1_000_000;
// Uretim: batch bazli odul. 20 kanit = 1 batch = 1 odul birimi.
const BATCH_ODUL_TABAN_MIKRO: i64 = 2_000; // batch basina 0.002 NEMES (kademe 0)
const HALVING_BATCH: i64 = 5_000_000; // her 5M tamamlanan batch'te odul yariya iner (=100M kanit)
const SPOT_CHECK_YUZDE: u8 = 1; // kanitlarin %1'i rastgele denetime duser
const DENETIM_BATCH: i64 = 5; // bir denetim gorevinde en fazla kac kayit
const DENETIM_ODUL_MIKRO: i64 = 50; // denetim sonucu basina denetci ucreti (0.00005 NEMES)
const DENETIM_ESIK: f32 = 0.98; // kosinus alti = kaldi
const SLASH_MIKRO: i64 = 2_000; // kesinlesen hilede geri alim (1 batch bedeli)
const ITIBAR_CEZA: i64 = 25; // slash basina itibar kesintisi
const ITIBAR_ODUL: i64 = 1; // gecen denetim basina uretici itibari (+1, max 100)
const STRIKE_LIMIT: i64 = 3; // bu sayiya ulasanin gorev akisi kesilir
const SHARD_SURE_SN: i64 = 1800; // shard ilan gecerliligi (30 dk)
const TASK_BATCH: i64 = 20;
const TASK_DEADLINE_SEC: i64 = 600;
const ARA_VARSAYILAN_K: usize = 5;
const ARA_ENBUYUK_K: usize = 20;
const ARA_BOYUT: usize = 768;
const HAVUZ_YENILE_SN: u64 = 120;

/// RAM'deki vektor havuzu: normalize edilmis duz dizi (n x 768).
#[derive(Default)]
struct VektorHavuzu {
    ids: Vec<i64>,
    duz: Vec<f32>,
    n: usize,
}

struct AppState {
    pool: SqlitePool,
    wiki: SqlitePool,
    http: reqwest::Client,
    matris: std::sync::Arc<tokio::sync::RwLock<VektorHavuzu>>,
    /// Dagitim kritik bolumu kilidi: cursor + supurme + claim sabitleme
    /// ayni anda tek gorevde (cift dagitim yarisini bitirir).
    dagitim_kilidi: tokio::sync::Mutex<()>,
    master_pubkey_b64: String,
    corpus: String,
    embed_api: String,
    embed_model: String,
}

#[derive(Deserialize)]
struct RegisterReq {
    cuzdan: String,
    makine_id: String,
}

#[derive(Serialize)]
struct RegisterResp {
    token: String,
    miner_id: String,
}

#[derive(Serialize)]
struct HealthResp {
    ok: bool,
    ts: String,
}

#[derive(Serialize)]
struct GorevResp {
    id: String,
    tip: String,
    corpus: String,
    shard: i64,
    offset: i64,
    limit: i64,
    deadline: i64,
    payload: GorevPayload,
}

#[derive(Serialize)]
struct DenetimRef {
    gorev_id: String,
    madde_id: i64,
}

#[derive(Serialize)]
struct GorevPayload {
    metinler: Vec<String>,
    madde_idler: Vec<i64>,
    // tip="denetim" ise doldurulur; embed gorevlerde None (JSON'a yazilmaz).
    #[serde(skip_serializing_if = "Option::is_none")]
    denetim: Option<Vec<DenetimRef>>,
}

#[derive(Deserialize)]
struct KanitReq {
    gorev_id: String,
    madde_id: i64,
    v_int8_b64: String,
    v_min: f32,
    v_max: f32,
    imza: Option<String>,
}

#[derive(Serialize)]
struct KanitResp {
    kabul: bool,
    // Bu kanitin anlik odulu (batch tamamlanmadiysa 0).
    odul_mikro: i64,
    odul_coin: f64,
    // Bu kanitla batch tamamladiysa bu miner'e dusen pay.
    batch_tamam: bool,
    batch_odul_mikro: i64,
    batch_odul_coin: f64,
    // Bu kanit rastgele denetime dustu mu?
    spot_check: bool,
    miner_id: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct DenetimKaydi {
    gorev_id: String,
    madde_id: i64,
    miner_id: String,
    v_int8_b64: String,
    v_min: f32,
    v_max: f32,
    ts: i64,
}

#[derive(Serialize)]
struct DenetimResp {
    kayitlar: Vec<DenetimKaydi>,
    sayi: usize,
    toplam_bekleyen: i64,
}

#[derive(Deserialize)]
struct DenetimSonuc {
    gorev_id: String,
    madde_id: i64,
    // Onerilen: denetcinin TAZE embedding'i (kosinus sunucuda hesaplanir).
    v_int8_b64: Option<String>,
    v_min: Option<f32>,
    v_max: Option<f32>,
    // Legacy: istemci-hesapli skor (vektor yoksa kullanilir).
    dogrulama: Option<f32>,
    gecerli: Option<bool>,
}

#[derive(Serialize)]
struct StatusResp {
    miner_id: String,
    pay: i64,
    coin_mikro: i64,
    coin: f64,
    strike: i64,
    itibar: i64,
}

#[derive(Serialize)]
struct BakiyeResp {
    miner_id: String,
    pay: i64,
    coin_mikro: i64,
    coin: f64,
    defter_toplam_mikro: i64,
    tutarli: bool,
    strike: i64,
    itibar: i64,
}

#[derive(Serialize, sqlx::FromRow)]
struct LedgerEntry {
    id: i64,
    miner_id: String,
    delta_mikro: i64,
    neden: String,
    epoch: i64,
    ts: i64,
}

#[derive(Serialize)]
struct LedgerResp {
    entries: Vec<LedgerEntry>,
}

#[derive(Serialize, sqlx::FromRow)]
struct ShardKaydi {
    miner_id: String,
    corpus: String,
    baslangic: i64,
    bitis: i64,
    adet: i64,
    kaynak: String,
    durum: String,
    ts: i64,
}

#[derive(Serialize)]
struct ShardListeResp {
    kayitlar: Vec<ShardKaydi>,
    sayi: usize,
}

// --- Helpers ---

fn current_epoch() -> i64 {
    Utc::now().timestamp()
}

async fn token_dogrula(headers: &HeaderMap, pool: &SqlitePool) -> Result<String, (StatusCode, String)> {
    let auth = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "Authorization header missing".to_string()))?;
    let token = auth.strip_prefix("Bearer ").unwrap_or(auth).trim();
    if token.is_empty() {
        return Err((StatusCode::UNAUTHORIZED, "Empty token".to_string()));
    }
    let row = sqlx::query("SELECT miner_id FROM miners WHERE token = ?")
        .bind(token)
        .fetch_optional(pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    row.map(|r| r.get("miner_id"))
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid token".to_string()))
}

/// Kanonik imza formati (nemes-core protocol.rs + `imzala` ile birebir):
/// epoch_le_bytes(8) + expires_le_bytes(8) + payload_bytes.
fn verify_signed_command(msg: &[u8], sig_b64: &str, pubkey_b64: &str) -> bool {
    let pubkey_bytes = match STANDARD.decode(pubkey_b64) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let sig_bytes = match STANDARD.decode(sig_b64) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let verifying_key = match VerifyingKey::from_bytes(&pubkey_bytes.try_into().unwrap_or_default()) {
        Ok(k) => k,
        Err(_) => return false,
    };
    let signature = match Signature::try_from(&sig_bytes[..]) {
        Ok(s) => s,
        Err(_) => return false,
    };
    verifying_key.verify(msg, &signature).is_ok()
}

/// Komut mesaji kur: epoch_le + expires_le + payload.
fn komut_mesaj(epoch: i64, expires: i64, payload: &str) -> Vec<u8> {
    let mut msg = Vec::with_capacity(16 + payload.len());
    msg.extend_from_slice(&epoch.to_le_bytes());
    msg.extend_from_slice(&expires.to_le_bytes());
    msg.extend_from_slice(payload.as_bytes());
    msg
}

/// Uretim odulu: tamamlanan batch sayisina gore batch basina mikro coin.
fn current_batch_reward_micro(tamamlanan_batch: i64) -> i64 {
    let halvings = (tamamlanan_batch / HALVING_BATCH) as u32;
    if halvings >= 31 {
        return 0;
    }
    BATCH_ODUL_TABAN_MIKRO >> halvings
}

/// Deterministik spot-check: ayni (gorev, madde) hep ayni karari verir.
/// blake3(gorev_id + madde_id) ilk bayt % 100 < SPOT_CHECK_YUZDE ise denetim.
fn spot_check_gerekli(gorev_id: &str, madde_id: i64) -> bool {
    let mut h = blake3::Hasher::new();
    h.update(gorev_id.as_bytes());
    h.update(&madde_id.to_le_bytes());
    let digest = h.finalize();
    (digest.as_bytes()[0] % 100) < SPOT_CHECK_YUZDE
}

/// int8 nicelemeyi geri coz: byte -> min + (b/255)*(max-min). 768 degilse None.
fn dequantize_int8(b64: &str, vmin: f32, vmax: f32) -> Option<Vec<f32>> {
    let raw = STANDARD.decode(b64).ok()?;
    if raw.len() != 768 {
        return None;
    }
    let range = vmax - vmin;
    if !range.is_finite() || range <= 0.0 {
        return None;
    }
    Some(raw.iter().map(|b| vmin + (*b as f32 / 255.0) * range).collect())
}

/// Kanit defterinden RAM havuzunu kur: (gorev, madde, b64, min, max)
/// -> normalize vektorler. Sadece bu komutanin corpus'u alinir.
async fn yukle_havuz(pool: &SqlitePool, corpus: &str) -> anyhow::Result<VektorHavuzu> {
    let rows = sqlx::query(
        "SELECT gorev_id, madde_id, v_int8_b64, v_min, v_max FROM kanitlar"
    )
    .fetch_all(pool)
    .await?;
    let mut ids = Vec::with_capacity(rows.len());
    let mut duz = Vec::with_capacity(rows.len() * ARA_BOYUT);
    for r in &rows {
        let gid: String = r.get("gorev_id");
        if gid.split(':').next().unwrap_or("") != corpus {
            continue;
        }
        let b64: String = r.get("v_int8_b64");
        let vmin: f32 = r.get("v_min");
        let vmax: f32 = r.get("v_max");
        let mid: i64 = r.get("madde_id");
        let mut v = match dequantize_int8(&b64, vmin, vmax) {
            Some(v) if v.len() == ARA_BOYUT => v,
            _ => continue,
        };
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in v.iter_mut() {
                *x /= norm;
            }
        }
        ids.push(mid);
        duz.extend_from_slice(&v);
    }
    let n = ids.len();
    Ok(VektorHavuzu { ids, duz, n })
}

#[derive(Deserialize)]
struct AraReq {
    sorgu: String,
    k: Option<usize>,
}

#[derive(Serialize)]
struct AraSonuc {
    madde_id: i64,
    baslik: String,
    skor: f32,
    ozet: String,
}

#[derive(Serialize)]
struct AraResp {
    sorgu: String,
    sonuclar: Vec<AraSonuc>,
    k: usize,
    toplam_vektor: usize,
    sure_ms: u64,
}

/// Sorguyu embed et (search_query oneiyle) -> normalize vektor.
async fn embed_sorgu(state: &Arc<AppState>, sorgu: &str) -> Result<Vec<f32>, (StatusCode, String)> {
    let url = format!("{}/v1/embeddings", state.embed_api.trim_end_matches('/'));
    let resp = state
        .http
        .post(&url)
        .json(&serde_json::json!({
            "model": state.embed_model,
            "input": [format!("search_query: {}", sorgu)],
        }))
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("embed erisilemiyor: {}", e)))?;
    if !resp.status().is_success() {
        return Err((StatusCode::BAD_GATEWAY, format!("embed http {}", resp.status())));
    }
    let j: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("embed yanit: {}", e)))?;
    let mut v: Vec<f32> = j
        .get("data")
        .and_then(|d| d.as_array())
        .and_then(|a| a.first())
        .and_then(|d| d.get("embedding"))
        .and_then(|e| e.as_array())
        .map(|a| a.iter().map(|x| x.as_f64().unwrap_or(0.0) as f32).collect())
        .ok_or((StatusCode::BAD_GATEWAY, "embed yanitinda vektor yok".to_string()))?;
    if v.len() != ARA_BOYUT {
        return Err((StatusCode::BAD_GATEWAY, format!("embed boyutu {} (768 beklenir)", v.len())));
    }
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
    Ok(v)
}

async fn ara_calistir(
    state: &Arc<AppState>,
    sorgu: &str,
    k_istek: Option<usize>,
) -> Result<AraResp, (StatusCode, String)> {
    let basla = std::time::Instant::now();
    let s = sorgu.trim();
    if s.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "sorgu bos".to_string()));
    }
    if s.chars().count() > 500 {
        return Err((StatusCode::BAD_REQUEST, "sorgu cok uzun (500 karakter)".to_string()));
    }
    let k = k_istek.unwrap_or(ARA_VARSAYILAN_K).clamp(1, ARA_ENBUYUK_K);

    let q = embed_sorgu(state, s).await?;

    // Skorlar: normalize oldugu icin nokta-carpim = kosinus.
    let (ids, skorlar, n) = {
        let h = state.matris.read().await;
        let n = h.n;
        if n == 0 {
            return Err((StatusCode::SERVICE_UNAVAILABLE, "havuz bos (henuz kanit yok)".to_string()));
        }
        let mut skorlar = vec![0f32; n];
        for i in 0..n {
            let taban = i * ARA_BOYUT;
            let mut t = 0f32;
            for j in 0..ARA_BOYUT {
                t += h.duz[taban + j] * q[j];
            }
            skorlar[i] = t;
        }
        (h.ids.clone(), skorlar, n)
    };

    let kk = k.min(n);
    let mut sira: Vec<usize> = (0..n).collect();
    sira.select_nth_unstable_by(kk, |&a, &b| {
        skorlar[b].partial_cmp(&skorlar[a]).unwrap_or(std::cmp::Ordering::Equal)
    });
    sira.truncate(kk);
    sira.sort_by(|&a, &b| skorlar[b].partial_cmp(&skorlar[a]).unwrap_or(std::cmp::Ordering::Equal));

    let mut sonuclar = Vec::with_capacity(kk);
    for i in sira {
        let mid = ids[i];
        let row = sqlx::query("SELECT baslik, ozet FROM madde WHERE id = ?")
            .bind(mid)
            .fetch_optional(&state.wiki)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let (baslik, ozet) = match row {
            Some(r) => {
                let b: String = r.get("baslik");
                let o: String = r.get::<String, _>("ozet").chars().take(600).collect();
                (b, o)
            }
            None => ("(silinmis madde)".to_string(), String::new()),
        };
        sonuclar.push(AraSonuc { madde_id: mid, baslik, skor: skorlar[i], ozet });
    }

    Ok(AraResp {
        sorgu: s.to_string(),
        sonuclar,
        k: kk,
        toplam_vektor: n,
        sure_ms: basla.elapsed().as_millis() as u64,
    })
}

/// POST /api/ara {"sorgu": "...", "k": 5} — herkese acik demo.
async fn ara(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AraReq>,
) -> Result<Json<AraResp>, (StatusCode, String)> {
    Ok(Json(ara_calistir(&state, &req.sorgu, req.k).await?))
}

/// GET /api/ara?q=...&k=5 — tarayici kolayligi.
async fn ara_get(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<AraResp>, (StatusCode, String)> {
    let sorgu = q.get("q").cloned().unwrap_or_default();
    let k = q.get("k").and_then(|s| s.parse::<usize>().ok());
    Ok(Json(ara_calistir(&state, &sorgu, k).await?))
}

/// Shard ilanini kaydet: imza + kayit + TOFU pubkey bagi + cursor'a
/// sabitleme + cakisma kontrolu. Donen: (baslangic, bitis).
async fn kaydet_ilan(state: &Arc<AppState>, ilan: &ShardIlan, kaynak: &str) -> Result<(i64, i64), String> {
    ilan.dogrula().map_err(|e| format!("ilan gecersiz: {}", e))?;
    let pool = &state.pool;
    // Sabitleme kritik bolumde: kontrol-et + yaz tek sira (cift sabitleme yarisini bitirir).
    let _kilit = state.dagitim_kilidi.lock().await;

    // Miner kayitli mi?
    let mrow = sqlx::query("SELECT pubkey_b64 FROM miners WHERE miner_id = ?")
        .bind(&ilan.miner_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
    let mrow = mrow.ok_or_else(|| format!("kayitsiz miner: {}", ilan.miner_id))?;

    // TOFU: ilk ilanda pubkey kilitlenir, sonra eslesmek zorunda.
    match mrow.get::<Option<String>, _>("pubkey_b64") {
        None => {
            sqlx::query("UPDATE miners SET pubkey_b64 = ? WHERE miner_id = ?")
                .bind(&ilan.pubkey_b64)
                .bind(&ilan.miner_id)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            info!("pubkey TOFU baglandi: {} <- {:.16}...", ilan.miner_id, ilan.pubkey_b64);
        }
        Some(k) if k == ilan.pubkey_b64 => {}
        Some(_) => return Err("pubkey kayittakiyle uyusmuyor".to_string()),
    }

    // Cursor'a sabitle: baslangic = son dagitilan + 1.
    let crow = sqlx::query("SELECT son_id FROM gorev_cursor WHERE corpus = ?")
        .bind(&ilan.corpus)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
    let son_id: i64 = crow.map(|r| r.get("son_id")).unwrap_or(0);
    let baslangic = son_id + 1;
    let bitis = baslangic + ilan.adet - 1;
    let now = current_epoch();

    // Cakisma: baska aktif + taze claim ile kesisme varsa reddet.
    let cakisma = sqlx::query(
        "SELECT miner_id, baslangic, bitis FROM shard_ilanlari WHERE corpus = ? AND durum = 'aktif' AND ts > ? AND NOT (bitis < ? OR baslangic > ?) LIMIT 1"
    )
    .bind(&ilan.corpus)
    .bind(now - SHARD_SURE_SN)
    .bind(baslangic)
    .bind(bitis)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    if let Some(c) = cakisma {
        let kim: String = c.get("miner_id");
        return Err(format!("cakisma: [{},{}] dolu (sahibi {})", baslangic, bitis, kim));
    }

    sqlx::query(
        "INSERT INTO shard_ilanlari (miner_id, corpus, baslangic, bitis, adet, pubkey_b64, ts_ms, kaynak, durum, ts) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'aktif', ?) ON CONFLICT(miner_id, ts_ms) DO NOTHING"
    )
    .bind(&ilan.miner_id)
    .bind(&ilan.corpus)
    .bind(baslangic)
    .bind(bitis)
    .bind(ilan.adet)
    .bind(&ilan.pubkey_b64)
    .bind(ilan.ts_ms)
    .bind(kaynak)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok((baslangic, bitis))
}

/// Kosinus benzerligi (f64 birikimle, deterministik).
fn kosinus(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let (mut dot, mut na, mut nb) = (0.0f64, 0.0f64, 0.0f64);
    for (x, y) in a.iter().zip(b.iter()) {
        let (x, y) = (*x as f64, *y as f64);
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    if na <= 0.0 || nb <= 0.0 {
        return 0.0;
    }
    (dot / (na.sqrt() * nb.sqrt())) as f32
}

// --- Handlers ---

async fn health() -> Json<HealthResp> {
    Json(HealthResp {
        ok: true,
        ts: Utc::now().format("%m-%d %H:%M:%S").to_string(),
    })
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterReq>,
) -> Result<Json<RegisterResp>, (StatusCode, String)> {
    let token = Uuid::new_v4().to_string().replace("-", "");
    let miner_id = format!("miner-{}", &token[..8]);
    let now = current_epoch();

    sqlx::query(
        "INSERT INTO miners (miner_id, token, cuzdan, makine_id, created_at, coin_mikro) VALUES (?, ?, ?, ?, ?, 0)"
    )
    .bind(&miner_id)
    .bind(&token)
    .bind(&req.cuzdan)
    .bind(&req.makine_id)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query(
        "INSERT INTO ledger (miner_id, delta_mikro, neden, epoch, ts) VALUES (?, 0, 'init', ?, ?)"
    )
    .bind(&miner_id)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    info!("Yeni madenci kaydedildi: {} ({})", miner_id, req.cuzdan);
    Ok(Json(RegisterResp { token, miner_id }))
}

async fn heartbeat(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let miner_id = token_dogrula(&headers, &state.pool).await?;
    let now = current_epoch();
    sqlx::query("UPDATE miners SET last_seen = ? WHERE miner_id = ?")
        .bind(now)
        .bind(&miner_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({"ok": true, "ts": Utc::now().format("%m-%d %H:%M:%S").to_string()})))
}

async fn gorev(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let miner_id = token_dogrula(&headers, &state.pool).await?;

    // 3 strike yiyenin gorev akisi kesilir (affet komutuyla acilir).
    let strike: i64 = sqlx::query_scalar("SELECT strike FROM miners WHERE miner_id = ?")
        .bind(&miner_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if strike >= STRIKE_LIMIT {
        warn!("Yasakli miner gorev istedi: {} (strike={})", miner_id, strike);
        return Err((StatusCode::FORBIDDEN, "3 strike - gorev akisi kesildi (operator affeti gerekli)".to_string()));
    }

    // Kuyruk biriktiyse ~5 gorevde 1 denetim dagit (es-dogrulama).
    if rand::thread_rng().gen_range(0..5) == 0 {
        if let Some(d) = dagit_denetim(&state, &miner_id).await? {
            return Ok(Json(d).into_response());
        }
    }

    // Kritik bolum: cursor oku -> dagit -> cursor yaz tek sira.
    let _kilit = state.dagitim_kilidi.lock().await;

    let row = sqlx::query("SELECT son_id FROM gorev_cursor WHERE corpus = ?")
        .bind(&state.corpus)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let son_id: i64 = row.map(|r| r.get("son_id")).unwrap_or(0);
    let now_ts0 = current_epoch();

    // Cursor arkasinda kalan claim'leri kapat.
    sqlx::query("UPDATE shard_ilanlari SET durum = 'tukendi' WHERE corpus = ? AND durum = 'aktif' AND bitis <= ?")
        .bind(&state.corpus)
        .bind(son_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Baskasinin aktif + taze claim'inin ustunden ATLA (shard sahipligi).
    let mut start = son_id;
    for _ in 0..8 {
        let hit = sqlx::query(
            "SELECT baslangic, bitis, miner_id FROM shard_ilanlari WHERE corpus = ? AND durum = 'aktif' AND ts > ? AND miner_id != ? AND baslangic <= ? AND ? <= bitis ORDER BY baslangic LIMIT 1"
        )
        .bind(&state.corpus)
        .bind(now_ts0 - SHARD_SURE_SN)
        .bind(&miner_id)
        .bind(start + 1)
        .bind(start + 1)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        match hit {
            Some(h) => {
                let e: i64 = h.get("bitis");
                let m: String = h.get("miner_id");
                info!("shard skip: {} {} araligi {}'in ({} -> {})", state.corpus, start + 1, m, start, e);
                start = e;
            }
            None => break,
        }
    }

    // Firsatci supurme: %10 orneklemede cursor gerisindeki bosluklari once dagit.
    if son_id % 200 < 20 {
        if let Some((offset, metinler, madde_idler)) = supurme_dene(&state, son_id).await? {
            let g = gorev_kaydet(&state, &miner_id, offset, metinler, madde_idler, "supuruldu").await?;
            return Ok(Json(g).into_response());
        }
    }

    let wiki_path = format!("/srv/beyin/wiki/wiki_{}.db", state.corpus);
    let wiki_db = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite:{}?mode=ro", wiki_path))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("wiki db: {}", e)))?;

    let rows = sqlx::query("SELECT id, ozet FROM madde WHERE id > ? AND length(ozet) > 20 ORDER BY id LIMIT ?")
        .bind(start)
        .bind(TASK_BATCH)
        .fetch_all(&wiki_db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if rows.is_empty() {
        // Cursor wiki sonuna dayandi: supurme turu baslasin, o da bossa beklemede.
        if let Some((offset, metinler, madde_idler)) = supurme_dene(&state, son_id).await? {
            let g = gorev_kaydet(&state, &miner_id, offset, metinler, madde_idler, "supuruldu-son").await?;
            return Ok(Json(g).into_response());
        }
        return Ok(StatusCode::NO_CONTENT.into_response());
    }

    let madde_idler: Vec<i64> = rows.iter().map(|r| r.get("id")).collect();
    let metinler: Vec<String> = rows.iter().map(|r| r.get::<String, _>("ozet").chars().take(4500).collect()).collect();
    let offset = madde_idler[0];
    let yeni_son = *madde_idler.last().unwrap();
    // Cursor monoton: asla geri gitmez.
    let ileri = yeni_son.max(son_id);

    sqlx::query(
        "INSERT INTO gorev_cursor (corpus, son_id) VALUES (?, ?) ON CONFLICT(corpus) DO UPDATE SET son_id = excluded.son_id"
    )
    .bind(&state.corpus)
    .bind(ileri)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let g = gorev_kaydet(&state, &miner_id, offset, metinler, madde_idler, "dagitildi").await?;
    Ok(Json(g).into_response())
}

/// Embed gorevi kur + batch takibine isle. Donen GorevResp dogrudan JSON'lanir.
async fn gorev_kaydet(
    state: &Arc<AppState>,
    miner_id: &str,
    offset: i64,
    metinler: Vec<String>,
    madde_idler: Vec<i64>,
    etiket: &str,
) -> Result<GorevResp, (StatusCode, String)> {
    let n = madde_idler.len();
    let gid = format!("{}:{}:{}:{}", state.corpus, offset, n, Uuid::new_v4().simple());
    let now_ts = current_epoch();
    // Dagitim izi: supurme, taze dagitilmis id'lere dokunmaz (TOCTOU korumasi).
    for chunk in madde_idler.chunks(500) {
        let mut q = String::from("INSERT INTO dagitilan_madde (madde_id, ts) VALUES ");
        q.push_str(&chunk.iter().map(|_| "(?, ?)").collect::<Vec<_>>().join(","));
        q.push_str(" ON CONFLICT(madde_id) DO UPDATE SET ts = excluded.ts");
        let mut qq = sqlx::query(&q);
        for m in chunk {
            qq = qq.bind(m).bind(now_ts);
        }
        qq.execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    sqlx::query(
        "INSERT OR IGNORE INTO gorevler (gorev_id, dagitilan_miner, corpus, offset, beklenen, alinan, durum, odul_mikro, ts) VALUES (?, ?, ?, ?, ?, 0, 'acik', 0, ?)"
    )
    .bind(&gid)
    .bind(miner_id)
    .bind(&state.corpus)
    .bind(offset)
    .bind(n as i64)
    .bind(now_ts)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    info!("Görev {}: {} ({} madde) -> {}", etiket, gid, n, miner_id);
    Ok(GorevResp {
        id: gid,
        tip: "embed".to_string(),
        corpus: state.corpus.clone(),
        shard: 0,
        offset,
        limit: n as i64,
        deadline: current_epoch() + TASK_DEADLINE_SEC,
        payload: GorevPayload { metinler, madde_idler, denetim: None },
    })
}

/// Supurme: cursor gerisinde kalmis, kaniti olmayan madde id'lerini bul.
/// supurme cursor'u `gorev_cursor.supurme:<corpus>` satirinda tutulur;
/// yakalayinca basa sarar (olumcul batch'ler bir sonraki turda tekrar denenir).
/// Donen: (offset, metinler, madde_idler). Yoksa None.
async fn supurme_dene(
    state: &Arc<AppState>,
    son_id: i64,
) -> Result<Option<(i64, Vec<String>, Vec<i64>)>, (StatusCode, String)> {
    if son_id <= 0 {
        return Ok(None);
    }
    let sup_key = format!("supurme:{}", state.corpus);
    let srow = sqlx::query("SELECT son_id FROM gorev_cursor WHERE corpus = ?")
        .bind(&sup_key)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mut sup: i64 = srow.map(|r| r.get("son_id")).unwrap_or(0);
    if sup >= son_id {
        sup = 0; // yakaladi, yeni tur
    }

    // Eski izleri temizle (1 saatten yasli dagitimlar supurulebilir).
    let now0 = current_epoch();
    sqlx::query("DELETE FROM dagitilan_madde WHERE ts < ?")
        .bind(now0 - 3600)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let wiki_path = format!("/srv/beyin/wiki/wiki_{}.db", state.corpus);
    let wiki_esc = wiki_path.replace('\'', "''");
    let mut conn = state.pool
        .acquire()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    sqlx::query(&format!("ATTACH DATABASE '{}' AS wiki", wiki_esc))
        .execute(&mut *conn)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("attach: {}", e)))?;
    let sorgu = sqlx::query(
        "SELECT w.id, w.ozet FROM wiki.madde w WHERE w.id > ? AND w.id <= ? AND length(w.ozet) > 20 AND NOT EXISTS (SELECT 1 FROM main.kanitlar k WHERE k.madde_id = w.id) AND NOT EXISTS (SELECT 1 FROM main.dagitilan_madde d WHERE d.madde_id = w.id AND d.ts > ?) ORDER BY w.id LIMIT ?"
    )
    .bind(sup)
    .bind(son_id)
    .bind(now0 - 300)
    .bind(TASK_BATCH)
    .fetch_all(&mut *conn)
    .await;
    // Havuz kirlenmesin: her halukarda DETACH.
    let _ = sqlx::query("DETACH DATABASE wiki").execute(&mut *conn).await;
    let rows = sorgu.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if rows.is_empty() {
        // Bosluk yok: sup'u cursor'a esitle, normal akisa don.
        sqlx::query(
            "INSERT INTO gorev_cursor (corpus, son_id) VALUES (?, ?) ON CONFLICT(corpus) DO UPDATE SET son_id = excluded.son_id"
        )
        .bind(&sup_key)
        .bind(son_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        return Ok(None);
    }

    let madde_idler: Vec<i64> = rows.iter().map(|r| r.get("id")).collect();
    let metinler: Vec<String> = rows.iter().map(|r| r.get::<String, _>("ozet").chars().take(4500).collect()).collect();
    let offset = madde_idler[0];
    let yeni_sup = *madde_idler.last().unwrap();
    sqlx::query(
        "INSERT INTO gorev_cursor (corpus, son_id) VALUES (?, ?) ON CONFLICT(corpus) DO UPDATE SET son_id = excluded.son_id"
    )
    .bind(&sup_key)
    .bind(yeni_sup)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Some((offset, metinler, madde_idler)))
}

/// Bekleyen spot-check kuyrugundan denetim gorevi kur.
/// Once baskasinin kanitlari (oz-denetim Sybil'ine karsi), yetmezse kendi.
/// Kuyruk birikmeden (<3) dagitmaz.
async fn dagit_denetim(state: &Arc<AppState>, auditor: &str) -> Result<Option<GorevResp>, (StatusCode, String)> {
    let bekleyen: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM kanitlar WHERE spot_check = 1 AND dogrulama IS NULL"
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if bekleyen < 3 {
        return Ok(None);
    }

    // Ayni denetciye iki kez gitmesin (son_denetci haric tutulur).
    let mut secilen = sqlx::query(
        "SELECT gorev_id, madde_id, miner_id FROM kanitlar WHERE spot_check = 1 AND dogrulama IS NULL AND miner_id != ? AND (son_denetci IS NULL OR son_denetci != ?) ORDER BY ts ASC LIMIT ?"
    )
    .bind(auditor)
    .bind(auditor)
    .bind(DENETIM_BATCH)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if (secilen.len() as i64) < DENETIM_BATCH {
        let kalan = DENETIM_BATCH - secilen.len() as i64;
        let mut kendi = sqlx::query(
            "SELECT gorev_id, madde_id, miner_id FROM kanitlar WHERE spot_check = 1 AND dogrulama IS NULL AND miner_id = ? AND (son_denetci IS NULL OR son_denetci != ?) ORDER BY ts ASC LIMIT ?"
        )
        .bind(auditor)
        .bind(auditor)
        .bind(kalan)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        secilen.append(&mut kendi);
    }
    if secilen.is_empty() {
        return Ok(None);
    }

    // Wiki havuzlari (corpus -> pool), pratikte tek corpus olur.
    let mut havuzlar: HashMap<String, SqlitePool> = HashMap::new();
    let mut metinler: Vec<String> = Vec::with_capacity(secilen.len());
    let mut madde_idler: Vec<i64> = Vec::with_capacity(secilen.len());
    let mut refs: Vec<DenetimRef> = Vec::with_capacity(secilen.len());
    let mut ilk_corpus = state.corpus.clone();

    for (i, r) in secilen.iter().enumerate() {
        let gid: String = r.get("gorev_id");
        let mid: i64 = r.get("madde_id");
        let corpus = gid.split(':').next().unwrap_or(&state.corpus).to_string();
        if i == 0 {
            ilk_corpus = corpus.clone();
        }
        if !havuzlar.contains_key(&corpus) {
            let wiki_path = format!("/srv/beyin/wiki/wiki_{}.db", corpus);
            let db = SqlitePoolOptions::new()
                .max_connections(1)
                .connect(&format!("sqlite:{}?mode=ro", wiki_path))
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("wiki db: {}", e)))?;
            havuzlar.insert(corpus.clone(), db);
        }
        let db = havuzlar.get(&corpus).unwrap();
        let srow = sqlx::query("SELECT ozet FROM madde WHERE id = ?")
            .bind(mid)
            .fetch_optional(db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let ozet: String = match srow {
            Some(x) => x.get("ozet"),
            None => continue, // madde silinmisse bu kaydi atla
        };
        metinler.push(ozet.chars().take(4500).collect());
        madde_idler.push(mid);
        refs.push(DenetimRef { gorev_id: gid.clone(), madde_id: mid });
        sqlx::query("UPDATE kanitlar SET denetim_sayisi = denetim_sayisi + 1 WHERE gorev_id = ? AND madde_id = ?")
            .bind(&gid)
            .bind(mid)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    if refs.is_empty() {
        return Ok(None);
    }

    let now = current_epoch();
    let did = format!("denetim:{}:{}", now, &Uuid::new_v4().simple().to_string()[..12]);
    let n = refs.len() as i64;
    sqlx::query(
        "INSERT OR IGNORE INTO gorevler (gorev_id, dagitilan_miner, corpus, offset, beklenen, alinan, durum, odul_mikro, ts) VALUES (?, ?, ?, 0, ?, 0, 'denetim', 0, ?)"
    )
    .bind(&did)
    .bind(auditor)
    .bind(&ilk_corpus)
    .bind(n)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    info!("Denetim gorevi: {} ({} kayit) -> {}", did, n, auditor);

    Ok(Some(GorevResp {
        id: did,
        tip: "denetim".to_string(),
        corpus: ilk_corpus,
        shard: 0,
        offset: 0,
        limit: n,
        deadline: now + TASK_DEADLINE_SEC,
        payload: GorevPayload { metinler, madde_idler, denetim: Some(refs) },
    }))
}

async fn kanit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<KanitReq>,
) -> Result<Json<KanitResp>, (StatusCode, String)> {
    let miner_id = token_dogrula(&headers, &state.pool).await?;

    // Denetim gorevlerine kanit gonderilmez (onlar denetim/sonuc'a gider).
    if req.gorev_id.starts_with("denetim:") {
        return Err((StatusCode::BAD_REQUEST, "Denetim gorevlerine kanit gonderilmez".to_string()));
    }

    let exists = sqlx::query("SELECT 1 FROM kanitlar WHERE gorev_id = ? AND madde_id = ?")
        .bind(&req.gorev_id)
        .bind(req.madde_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if exists.is_some() {
        return Err((StatusCode::CONFLICT, "Already processed".to_string()));
    }

    if let Some(sig) = &req.imza {
        let payload = serde_json::json!({
            "gorev_id": &req.gorev_id,
            "madde_id": req.madde_id,
            "v_int8_b64": &req.v_int8_b64,
            "v_min": req.v_min,
            "v_max": req.v_max
        }).to_string();
        if !verify_signed_command(payload.as_bytes(), sig, &state.master_pubkey_b64) {
            return Err((StatusCode::FORBIDDEN, "Invalid signature".to_string()));
        }
    }

    let v_bytes = base64::engine::general_purpose::STANDARD.decode(&req.v_int8_b64)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid base64".to_string()))?;
    if v_bytes.len() != 768 {
        return Err((StatusCode::BAD_REQUEST, "Vector must be 768 bytes".to_string()));
    }

    let now = current_epoch();
    let spot = spot_check_gerekli(&req.gorev_id, req.madde_id);

    // Kaniti kaydet (anlik odul 0; odul batch tamamlaninca dagitilir).
    sqlx::query(
        "INSERT INTO kanitlar (gorev_id, madde_id, miner_id, v_int8_b64, v_min, v_max, odul_mikro, spot_check, ts) VALUES (?, ?, ?, ?, ?, ?, 0, ?, ?)"
    )
    .bind(&req.gorev_id)
    .bind(req.madde_id)
    .bind(&miner_id)
    .bind(&req.v_int8_b64)
    .bind(req.v_min)
    .bind(req.v_max)
    .bind(if spot { 1 } else { 0 })
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // pay her kabulde +1 (emek sayaci); coin sadece batch bitince.
    sqlx::query("UPDATE miners SET pay = pay + 1 WHERE miner_id = ?")
        .bind(&miner_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if spot {
        info!("Spot-check bayragi: miner={} gorev={} madde={}", miner_id, req.gorev_id, req.madde_id);
    }

    // Batch tamamlandi mi? Bekleneni bul (yoksa gorev_id'den parse et).
    let grow: Option<sqlx::sqlite::SqliteRow> = sqlx::query(
        "SELECT beklenen, durum FROM gorevler WHERE gorev_id = ?"
    )
    .bind(&req.gorev_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (beklenen, zaten_tamam): (i64, bool) = match grow {
        Some(r) => {
            let b: i64 = r.get("beklenen");
            let d: String = r.get("durum");
            (b, d == "tamam")
        }
        None => {
            // Eski dagitimdan kalan gorev (002 oncesi): limit'i id'den cikar.
            let parts: Vec<&str> = req.gorev_id.split(':').collect();
            let lim = parts.get(2).and_then(|s| s.parse::<i64>().ok()).unwrap_or(TASK_BATCH);
            sqlx::query(
                "INSERT OR IGNORE INTO gorevler (gorev_id, dagitilan_miner, corpus, offset, beklenen, alinan, durum, odul_mikro, ts) VALUES (?, ?, ?, 0, ?, 0, 'acik', 0, ?)"
            )
            .bind(&req.gorev_id)
            .bind(&miner_id)
            .bind(&state.corpus)
            .bind(lim)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            (lim, false)
        }
    };

    let sayi: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kanitlar WHERE gorev_id = ?")
        .bind(&req.gorev_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query("UPDATE gorevler SET alinan = ? WHERE gorev_id = ?")
        .bind(sayi)
        .bind(&req.gorev_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Batch henuz bitmemisse: odul yok, sadece kabul + spot bayragi.
    if zaten_tamam || sayi < beklenen {
        return Ok(Json(KanitResp {
            kabul: true,
            odul_mikro: 0,
            odul_coin: 0.0,
            batch_tamam: false,
            batch_odul_mikro: 0,
            batch_odul_coin: 0.0,
            spot_check: spot,
            miner_id,
        }));
    }

    // Batch tamamlandi: odulu katki oranina gore dagit.
    let tamamlanan: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM gorevler WHERE durum = 'tamam'")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let batch_odul = current_batch_reward_micro(tamamlanan);

    let katki: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
        "SELECT miner_id, COUNT(*) as c FROM kanitlar WHERE gorev_id = ? GROUP BY miner_id ORDER BY c DESC"
    )
    .bind(&req.gorev_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut dagitilan: i64 = 0;
    let mut benim_payim: i64 = 0;
    let n_katki = katki.len() as i64;
    for (i, r) in katki.iter().enumerate() {
        let mid: String = r.get("miner_id");
        let c: i64 = r.get("c");
        // Son katkiya kalan bakiyeyi ver (kusurat kaybi olmasin).
        let pay_i = if i as i64 == n_katki - 1 {
            batch_odul - dagitilan
        } else {
            (batch_odul * c) / beklenen
        };
        if pay_i > 0 {
            sqlx::query("UPDATE miners SET coin_mikro = coin_mikro + ? WHERE miner_id = ?")
                .bind(pay_i)
                .bind(&mid)
                .execute(&state.pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query(
                "INSERT INTO ledger (miner_id, delta_mikro, neden, epoch, ts) VALUES (?, ?, 'batch', ?, ?)"
            )
            .bind(&mid)
            .bind(pay_i)
            .bind(now)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
        dagitilan += pay_i;
        if mid == miner_id {
            benim_payim = pay_i;
        }
    }

    sqlx::query("UPDATE gorevler SET durum = 'tamam', alinan = ?, odul_mikro = ? WHERE gorev_id = ?")
        .bind(sayi)
        .bind(batch_odul)
        .bind(&req.gorev_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    info!("Batch tamam: gorev={} beklenen={} odul={} mikro ({:.6} NEMES) bitiren={}",
          req.gorev_id, beklenen, batch_odul, batch_odul as f64 / COIN_UNIT as f64, miner_id);

    Ok(Json(KanitResp {
        kabul: true,
        odul_mikro: benim_payim,
        odul_coin: benim_payim as f64 / COIN_UNIT as f64,
        batch_tamam: true,
        batch_odul_mikro: benim_payim,
        batch_odul_coin: benim_payim as f64 / COIN_UNIT as f64,
        spot_check: spot,
        miner_id,
    }))
}

async fn status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<StatusResp>, (StatusCode, String)> {
    let miner_id = token_dogrula(&headers, &state.pool).await?;
    let row = sqlx::query("SELECT pay, coin_mikro, strike, itibar FROM miners WHERE miner_id = ?")
        .bind(&miner_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let pay: i64 = row.get("pay");
    let coin_mikro: i64 = row.get("coin_mikro");
    let strike: i64 = row.get("strike");
    let itibar: i64 = row.get("itibar");
    Ok(Json(StatusResp {
        miner_id,
        pay,
        coin_mikro,
        coin: coin_mikro as f64 / COIN_UNIT as f64,
        strike,
        itibar,
    }))
}

async fn bakiye(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<BakiyeResp>, (StatusCode, String)> {
    let miner_id = token_dogrula(&headers, &state.pool).await?;
    let row = sqlx::query("SELECT pay, coin_mikro, strike, itibar FROM miners WHERE miner_id = ?")
        .bind(&miner_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let pay: i64 = row.get("pay");
    let coin_mikro: i64 = row.get("coin_mikro");
    let strike: i64 = row.get("strike");
    let itibar: i64 = row.get("itibar");

    let defter: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(delta_mikro), 0) FROM ledger WHERE miner_id = ?")
        .bind(&miner_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(BakiyeResp {
        miner_id,
        pay,
        coin_mikro,
        coin: coin_mikro as f64 / COIN_UNIT as f64,
        defter_toplam_mikro: defter,
        tutarli: coin_mikro == defter,
        strike,
        itibar,
    }))
}

async fn arz(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let toplam_mikro: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(coin_mikro), 0) FROM miners")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    let toplam_kanit: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kanitlar")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    let tamamlanan: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM gorevler WHERE durum = 'tamam'")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    let kademe = (tamamlanan / HALVING_BATCH) as u32;
    let batch_odul = current_batch_reward_micro(tamamlanan);
    let vektor_sayisi: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kanitlar")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    // Legacy alanlar korunur: odul_mikro = batch odulu (birim: batch).
    Json(serde_json::json!({
        "toplam_mikro": toplam_mikro,
        "toplam_coin": toplam_mikro as f64 / COIN_UNIT as f64,
        "kademe": kademe,
        "odul_mikro": batch_odul,
        "odul_coin": batch_odul as f64 / COIN_UNIT as f64,
        "odul_birim": "batch",
        "odul_batch_mikro": batch_odul,
        "odul_batch_coin": batch_odul as f64 / COIN_UNIT as f64,
        "odul_kanit_mikro": batch_odul / TASK_BATCH,
        "tamamlanan_batch": tamamlanan,
        "toplam_kanit": toplam_kanit,
        "vektor_sayisi": vektor_sayisi,
        "yarilanma_adimi": HALVING_BATCH,
        "yarilanma_birim": "batch",
        "birim": COIN_UNIT,
    }))
}

async fn denetim_liste(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<DenetimResp>, (StatusCode, String)> {
    let _denetci = token_dogrula(&headers, &state.pool).await?;
    let limit: i64 = q.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20).clamp(1, 100);
    let rows = sqlx::query_as::<_, DenetimKaydi>(
        "SELECT gorev_id, madde_id, miner_id, v_int8_b64, v_min, v_max, ts FROM kanitlar WHERE spot_check = 1 AND dogrulama IS NULL ORDER BY ts ASC LIMIT ?"
    )
    .bind(limit)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let sayi = rows.len();
    let toplam_bekleyen: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM kanitlar WHERE spot_check = 1 AND dogrulama IS NULL"
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(DenetimResp { kayitlar: rows, sayi, toplam_bekleyen }))
}

async fn denetim_sonuc(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<DenetimSonuc>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let denetci = token_dogrula(&headers, &state.pool).await?;
    let now = current_epoch();

    // Orijinal kaniti bul (denetim bayrakli olmali).
    let orow = sqlx::query(
        "SELECT miner_id, v_int8_b64, v_min, v_max, dogrulama, ret FROM kanitlar WHERE gorev_id = ? AND madde_id = ?"
    )
    .bind(&req.gorev_id)
    .bind(req.madde_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let orow = orow.ok_or((StatusCode::NOT_FOUND, "Denetim kaydi bulunamadi".to_string()))?;
    if orow.get::<Option<f32>, _>("dogrulama").is_some() {
        return Err((StatusCode::CONFLICT, "Bu kayit zaten denetlenmis".to_string()));
    }
    let ret_onceki: i64 = orow.get("ret");
    let ureten: String = orow.get("miner_id");
    if ureten == denetci {
        warn!("Oz-denetim (testnet toleransli): denetci={} kendi kanitini denetliyor gorev={} madde={}", denetci, req.gorev_id, req.madde_id);
    }

    // Skor: taze vektor varsa sunucu hesaplar (guvenilmez istemciye birakilmaz).
    let (cos, gecerli) = match (&req.v_int8_b64, req.v_min, req.v_max) {
        (Some(b64), Some(vmin), Some(vmax)) => {
            let taze = dequantize_int8(b64, vmin, vmax)
                .ok_or((StatusCode::BAD_REQUEST, "Denetim vektoru 768 bayt olmali".to_string()))?;
            let o_b64: String = orow.get("v_int8_b64");
            let o_min: f32 = orow.get("v_min");
            let o_max: f32 = orow.get("v_max");
            let orig = dequantize_int8(&o_b64, o_min, o_max)
                .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Orijinal vektor cozule medi".to_string()))?;
            let c = kosinus(&taze, &orig);
            (c, c >= DENETIM_ESIK)
        }
        _ => {
            // Legacy: istemci-hesapli skor.
            let d = req.dogrulama.ok_or((StatusCode::BAD_REQUEST, "dogrulama veya taze vektor gerekli".to_string()))?;
            let g = req.gecerli.unwrap_or(d >= DENETIM_ESIK);
            (d, g)
        }
    };

    // Denetci ucreti (is basina, gecti/kaldi fark etmez — emek odendi).
    sqlx::query("UPDATE miners SET coin_mikro = coin_mikro + ? WHERE miner_id = ?")
        .bind(DENETIM_ODUL_MIKRO)
        .bind(&denetci)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    sqlx::query(
        "INSERT INTO ledger (miner_id, delta_mikro, neden, epoch, ts) VALUES (?, ?, 'denetim', ?, ?)"
    )
    .bind(&denetci)
    .bind(DENETIM_ODUL_MIKRO)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if gecerli {
        // Gecti: skoru kilitle. Supheden donduyse suphe giderildi.
        let r = sqlx::query(
            "UPDATE kanitlar SET dogrulama = ? WHERE gorev_id = ? AND madde_id = ? AND dogrulama IS NULL"
        )
        .bind(cos)
        .bind(&req.gorev_id)
        .bind(req.madde_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        if r.rows_affected() == 0 {
            return Err((StatusCode::CONFLICT, "Bu kayit araya denetlenmis".to_string()));
        }
        sqlx::query("UPDATE miners SET itibar = CASE WHEN itibar + ? > 100 THEN 100 ELSE itibar + ? END WHERE miner_id = ?")
            .bind(ITIBAR_ODUL)
            .bind(ITIBAR_ODUL)
            .bind(&ureten)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        if ret_onceki >= 1 {
            info!("Suphe GIDERILDI: denetci={} gorev={} madde={} cos={:.4} (onceki ret sayisi={})", denetci, req.gorev_id, req.madde_id, cos, ret_onceki);
        } else {
            info!("Denetim gecti: denetci={} gorev={} madde={} cos={:.4}", denetci, req.gorev_id, req.madde_id, cos);
        }
        return Ok(Json(serde_json::json!({"ok": true, "gecerli": true, "dogrulama": cos, "supheli": false})));
    }

    // Kaldi: iki asamali challenge.
    if ret_onceki >= 1 {
        // Ikinci bagimsiz ret: hile kesinlesti -> SLASH.
        sqlx::query(
            "UPDATE kanitlar SET dogrulama = ? WHERE gorev_id = ? AND madde_id = ? AND dogrulama IS NULL"
        )
        .bind(cos)
        .bind(&req.gorev_id)
        .bind(req.madde_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let bak: i64 = sqlx::query_scalar("SELECT coin_mikro FROM miners WHERE miner_id = ?")
            .bind(&ureten)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let kesinti = SLASH_MIKRO.min(bak).max(0);
        if kesinti > 0 {
            sqlx::query("UPDATE miners SET coin_mikro = coin_mikro - ? WHERE miner_id = ?")
                .bind(kesinti)
                .bind(&ureten)
                .execute(&state.pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query(
                "INSERT INTO ledger (miner_id, delta_mikro, neden, epoch, ts) VALUES (?, ?, 'slash', ?, ?)"
            )
            .bind(&ureten)
            .bind(-kesinti)
            .bind(now)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
        sqlx::query("UPDATE miners SET strike = strike + 1, itibar = CASE WHEN itibar - ? < 0 THEN 0 ELSE itibar - ? END WHERE miner_id = ?")
            .bind(ITIBAR_CEZA)
            .bind(ITIBAR_CEZA)
            .bind(&ureten)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let strike: i64 = sqlx::query_scalar("SELECT strike FROM miners WHERE miner_id = ?")
            .bind(&ureten)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        warn!("SLASH: ureten={} gorev={} madde={} cos={:.4} kesinti={} strike={} (denetciler: onceki+{})", ureten, req.gorev_id, req.madde_id, cos, kesinti, strike, denetci);
        return Ok(Json(serde_json::json!({"ok": true, "gecerli": false, "dogrulama": cos, "supheli": false, "slash": kesinti, "strike": strike})));
    }

    // Ilk ret: supheli, kuyruga iade (ikinci bagimsiz denetciye gider).
    sqlx::query(
        "UPDATE kanitlar SET ret = ret + 1, son_denetci = ? WHERE gorev_id = ? AND madde_id = ? AND dogrulama IS NULL"
    )
    .bind(&denetci)
    .bind(&req.gorev_id)
    .bind(req.madde_id)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    warn!("SUPHELI (1. ret, slash yok): denetci={} ureten={} gorev={} madde={} cos={:.4} -> ikinci denetciye iade", denetci, ureten, req.gorev_id, req.madde_id, cos);
    Ok(Json(serde_json::json!({"ok": true, "gecerli": false, "dogrulama": cos, "supheli": true})))
}

async fn ledger(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<LedgerResp>, (StatusCode, String)> {
    let miner_id = token_dogrula(&headers, &state.pool).await?;
    let rows = sqlx::query_as::<_, LedgerEntry>(
        "SELECT id, miner_id, delta_mikro, neden, epoch, ts FROM ledger WHERE miner_id = ? ORDER BY id DESC LIMIT 100"
    )
    .bind(&miner_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(LedgerResp { entries: rows }))
}

/// Shard ilani al (HTTP kontrol kanali; birincil yol gossip'tur).
async fn shard_ilan(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(ilan): Json<ShardIlan>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let miner_id = token_dogrula(&headers, &state.pool).await?;
    if ilan.miner_id != miner_id {
        return Err((StatusCode::FORBIDDEN, "ilan miner_id token ile uyusmuyor".to_string()));
    }
    match kaydet_ilan(&state, &ilan, "http").await {
        Ok((b, e)) => {
            info!("shard ilan http: {} [{},{}] <- {}", ilan.corpus, b, e, miner_id);
            Ok(Json(serde_json::json!({"ok": true, "baslangic": b, "bitis": e})))
        }
        Err(m) => Err((StatusCode::CONFLICT, m)),
    }
}

/// Aktif shard ilanlari (kayit defteri gorunumu).
async fn shard_liste(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<ShardListeResp>, (StatusCode, String)> {
    let _ = token_dogrula(&headers, &state.pool).await?;
    let rows = sqlx::query_as::<_, ShardKaydi>(
        "SELECT miner_id, corpus, baslangic, bitis, adet, kaynak, durum, ts FROM shard_ilanlari WHERE durum = 'aktif' ORDER BY ts DESC LIMIT 100"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let sayi = rows.len();
    Ok(Json(ShardListeResp { kayitlar: rows, sayi }))
}

async fn komut(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(cmd): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let epoch = cmd.get("epoch").and_then(|v| v.as_i64()).ok_or((StatusCode::BAD_REQUEST, "epoch missing".to_string()))?;
    let expires = cmd.get("expires").and_then(|v| v.as_i64()).ok_or((StatusCode::BAD_REQUEST, "expires missing".to_string()))?;
    let payload = cmd.get("payload_json").and_then(|v| v.as_str()).ok_or((StatusCode::BAD_REQUEST, "payload_json missing".to_string()))?;
    let sig_b64 = cmd.get("signature_b64").and_then(|v| v.as_str()).ok_or((StatusCode::BAD_REQUEST, "signature_b64 missing".to_string()))?;
    let pubkey_b64 = cmd.get("pubkey_b64").and_then(|v| v.as_str()).ok_or((StatusCode::BAD_REQUEST, "pubkey_b64 missing".to_string()))?;

    if epoch > current_epoch() + 300 {
        return Err((StatusCode::BAD_REQUEST, "Epoch too far in future".to_string()));
    }
    if expires < current_epoch() {
        return Err((StatusCode::BAD_REQUEST, "Command expired".to_string()));
    }

    if pubkey_b64 != state.master_pubkey_b64 {
        return Err((StatusCode::FORBIDDEN, "Unknown pubkey".to_string()));
    }
    let msg = komut_mesaj(epoch, expires, payload);
    if !verify_signed_command(&msg, sig_b64, pubkey_b64) {
        return Err((StatusCode::FORBIDDEN, "Invalid signature".to_string()));
    }

    let payload_json: serde_json::Value = serde_json::from_str(payload)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid payload JSON".to_string()))?;

    if let Some(tip) = payload_json.get("tip").and_then(|v| v.as_str()) {
        match tip {
            "dur" => {
                info!("DUR komutu alındı - graceful shutdown başlatılıyor");
            }
            "yeniden_baslat" => {
                info!("YENİDEN BAŞLAT komutu alındı");
            }
            "gorev_degistir" => {
                if let Some(yeni_corpus) = payload_json.get("corpus").and_then(|v| v.as_str()) {
                    info!("Görev corpus değiştirme isteği: {} (şu an: {})", yeni_corpus, state.corpus);
                }
            }
            "affet" => {
                // Operator affi: strike sifirla, itibari yuzle.
                let mid = payload_json.get("miner_id").and_then(|v| v.as_str()).ok_or((StatusCode::BAD_REQUEST, "miner_id gerekli".to_string()))?;
                sqlx::query("UPDATE miners SET strike = 0, itibar = 100 WHERE miner_id = ?")
                    .bind(mid)
                    .execute(&state.pool)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                info!("AFFET komutu: {} strike sifirlandi, itibar 100", mid);
            }
            _ => {
                info!("Bilinmeyen komut tipi: {}", tip);
            }
        }
    }

    Ok(Json(serde_json::json!({"ok": true, "epoch": epoch})))
}

// --- Main ---

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "/tmp/komuta.db".to_string());
    let master_pubkey = std::env::var("MASTER_PUBKEY_B64")
        .unwrap_or_else(|_| "Sdc/yVS0SXAiqclXKJ0iHggjvZZMgPfGYSOlG1mqZQs=".to_string());
    let corpus = std::env::var("GOREV_CORPUS").unwrap_or_else(|_| "tr".to_string());
    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "8787".to_string()).parse()?;
    let p2p_port: u16 = std::env::var("P2P_PORT").unwrap_or_else(|_| "4003".to_string()).parse()?;
    let shard_sub = std::env::var("P2P_SHARD_SUB").unwrap_or_else(|_| "1".to_string()) != "0";
    let embed_api = std::env::var("EMBED_API").unwrap_or_else(|_| "http://127.0.0.1:1241".to_string());
    let embed_model = std::env::var("EMBED_MODEL")
        .unwrap_or_else(|_| "text-embedding-nomic-embed-text-v1.5".to_string());

    let db_url = if db_path.starts_with('/') {
        format!("sqlite:{}", db_path)
    } else {
        format!("sqlite:{}", db_path)
    };
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    // Wiki havuzu (salt-okunur, arama metinleri icin).
    let wiki_path = format!("/srv/beyin/wiki/wiki_{}.db", corpus);
    let wiki = SqlitePoolOptions::new()
        .max_connections(2)
        .connect(&format!("sqlite:{}?mode=ro", wiki_path))
        .await?;
    info!("Wiki baglandi: {}", wiki_path);

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;

    // Vektor havuzu: acilista yukle, arka planda periyodik tazele.
    let matris = std::sync::Arc::new(tokio::sync::RwLock::new(VektorHavuzu::default()));
    match yukle_havuz(&pool, &corpus).await {
        Ok(h) => {
            info!("Vektor havuzu yuklendi: {} vektor", h.n);
            *matris.write().await = h;
        }
        Err(e) => warn!("ilk havuz yuklemesi basarisiz (bos devam): {}", e),
    }
    {
        let pool_r = pool.clone();
        let matris_r = matris.clone();
        let corpus_r = corpus.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(HAVUZ_YENILE_SN)).await;
                match yukle_havuz(&pool_r, &corpus_r).await {
                    Ok(h) => {
                        info!("Vektor havuzu tazelendi: {} vektor", h.n);
                        *matris_r.write().await = h;
                    }
                    Err(e) => warn!("havuz tazeleme hatasi: {}", e),
                }
            }
        });
    }

    let state = Arc::new(AppState {
        pool,
        wiki,
        http,
        matris,
        dagitim_kilidi: tokio::sync::Mutex::new(()),
        master_pubkey_b64: master_pubkey,
        corpus,
        embed_api,
        embed_model,
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/kayit", post(register))
        .route("/api/heartbeat", post(heartbeat))
        .route("/api/gorev", get(gorev))
        .route("/api/kanit", post(kanit))
        .route("/api/status", get(status))
        .route("/api/bakiye", get(bakiye))
        .route("/api/arz", get(arz))
        .route("/api/ledger", get(ledger))
        .route("/api/komut", post(komut))
        .route("/api/denetim", get(denetim_liste))
        .route("/api/denetim/sonuc", post(denetim_sonuc))
        .route("/api/shard/ilan", post(shard_ilan))
        .route("/api/shard", get(shard_liste))
        .route("/api/ara", post(ara))
        .route("/api/ara", get(ara_get))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    // Gossip abonesi: nemes/shard ilanlarini dinler, deftere isler.
    // HTTP API'dan bagimsiz gorevde kosar; duserse API etkilenmez.
    if shard_sub {
        let state2 = state.clone();
        tokio::spawn(async move {
            let mut node = match P2PNode::new(P2PConfig { port: p2p_port, enable_mdns: true }).await {
                Ok(n) => n,
                Err(e) => {
                    warn!("shard abonesi acilamadi (port {}): {} — ilanlar sadece HTTP ile alinacak", p2p_port, e);
                    return;
                }
            };
            let mut rx = match node.take_event_receiver() {
                Some(r) => r,
                None => return,
            };
            info!("shard abonesi dinliyor: nemes/shard (port {})", p2p_port);
            loop {
                if let Err(e) = node.run_for(5).await {
                    warn!("shard abone dongu hatasi: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
                while let Ok(ev) = rx.try_recv() {
                    if let NetworkEvent::MessageReceived { from, topic, data } = ev {
                        if topic != nemes_core::shard::SHARD_TOPIC {
                            continue;
                        }
                        match ShardIlan::json_coz(&data) {
                            Ok(ilan) => match kaydet_ilan(&state2, &ilan, "gossip").await {
                                Ok((b, e)) => info!("shard ilan gossip: {} [{},{}] <- {} ({})", ilan.corpus, b, e, ilan.miner_id, from),
                                Err(m) => warn!("shard ilan red ({}): {}", from, m),
                            },
                            Err(e) => warn!("bozuk shard ilani ({}): {}", from, e),
                        }
                    }
                }
            }
        });
    } else {
        info!("shard abonesi kapali (P2P_SHARD_SUB=0) — ilanlar sadece HTTP ile");
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Komuta API başlatılıyor: http://{}", addr);
    info!("Master pubkey: {}", &state.master_pubkey_b64[..20]);
    info!("Görev corpus: {}", state.corpus);

    axum::serve(
        tokio::net::TcpListener::bind(addr).await?,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        signal::ctrl_c().await.ok();
        info!("Shutdown sinyali alındı");
    })
    .await?;

    Ok(())
}