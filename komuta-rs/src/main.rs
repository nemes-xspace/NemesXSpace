// Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use chrono::Utc;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};
use sqlx::sqlite::SqlitePoolOptions;
use nemes_core::shard::ShardIlan;
use nemes_p2p::{NetworkEvent, P2PConfig, P2PNode, GOREV_TOPIC};
use nemes_core::mesh_audit::{DENETIM_TOPIC, DENETIM_SAYISI, DenetimDuyuru, KanitAtama, denetciler};
use rand::Rng;
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::signal;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};
use uuid::Uuid;
use base64::{engine::general_purpose::STANDARD, Engine as _};

// --- Config ---
const COIN_UNIT: i64 = 1_000_000;
// Uretim: batch bazli odul. 20 kanit = 1 batch = 1 odul birimi.
const BATCH_ODUL_TABAN_MIKRO: i64 = 2_000; // batch basina 0.002 NEMES (kademe 0)
const KUYRUK_TABAN_MIKRO: i64 = 5; // kuyruk tabani 0,000005 (TOKENOMI kilitli; altina inmez)
const HALVING_BATCH: i64 = 5_000_000; // her 5M tamamlanan batch'te odul yariya iner (=100M kanit)
const SPOT_CHECK_YUZDE: u8 = 10; // kanitlarin %10'u rastgele denetime duser (site ile uyumlu; B-1)
// B25 bagisiklik: asagidaki atomikler calisma aninda guncellenir (arka-plan
// gorevi, son 1 saatin ret oranina gore). Tabandan baslar, tavana kadar cikar.
static SPOT_YUZDE_DINAMIK: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(SPOT_CHECK_YUZDE as u64);
static KANARYA_BPBIN_DINAMIK: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(400); // 400bp = %4

/// Bagisiklik eslestirmesi (saf, testli): ret orani -> (spot %, kanarya bp).
/// Taban (10, 400) -> kaldi %5'te (50, 2000) tavanina dogrusal tirmanir.
fn bagisiklik_eslestir(ret_orani: f64) -> (u64, u64) {
    let r = ret_orani.clamp(0.0, 1.0);
    let spot = (10.0 + r * 800.0).round() as u64;
    let kanarya = (400.0 + r * 32000.0).round() as u64;
    (spot.min(50), kanarya.min(2000))
}

/// Kanarya olasiligi (dagitim aninda okunur).
fn kanarya_orani() -> f64 {
    KANARYA_BPBIN_DINAMIK.load(std::sync::atomic::Ordering::Relaxed) as f64 / 10000.0
}
const DENETIM_BATCH: i64 = 20; // bir denetim gorevinde en fazla kac kayit (sel tasmamasi icin 5->20, 16 Eyl)
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
// --- C7 onarım/çoğaltma sabitleri ---
const PARCA_BOYUT: u64 = 64 * 1024 * 1024; // 64 MiB parça
const REPLIKA_HEDEF: i64 = 3; // parça başına canlı kopya hedefi
const YEDEKLE_BATCH: i64 = 3; // görev başına en fazla parça
const ONARIM_ODUL_MIKRO: i64 = 100; // parça başına hedef ücreti
const OLU_ESIK_SN: i64 = 300; // nabız bu süredir yoksa ölü
// --- C8 yoklama sabitleri ---
const YOKLAMA_SURE_SN: i64 = 1800; // cevap penceresi (30 dk)
const YOKLAMA_ARALIK_SN: u64 = 300; // uretim dongusu (5 dk)
const YOKLAMA_BOYUT: u64 = 1024 * 1024; // orneklem 1 MiB
const YOKLAMA_ACIK_LIMIT: i64 = 2; // miner basina acik ust sinir
const YOKLAMA_TUR_LIMIT: i64 = 3; // dongu basina uretim ust sinir

/// RAM'deki vektor havuzu: normalize edilmis duz dizi (n x 768).
/// B10 tavan (18 Eyl): /api/ara 24 saatte hic cagrilmadi; havuz sinirsiz
/// buyuyup swap'i sisiriyordu. HAVUZ_MAX_VEKTOR (default 1M, 0 = sinirsiz)
/// asilinca en yeniler tutulur (arama tazeligi korunur).
const HAVUZ_MAX_VARSAYILAN: usize = 1_000_000;

#[derive(Default)]
struct VektorHavuzu {
    ids: Vec<i64>,
    duz: Vec<f32>,
    n: usize,
}

impl VektorHavuzu {
    /// Tavani uygula: fazlalik en eskiden kirpilir (ids+duz eszamanli).
    fn tavan_uygula(&mut self, tavan: usize) {
        if tavan == 0 || self.n <= tavan {
            return;
        }
        let fazla = self.n - tavan;
        self.ids.drain(..fazla);
        self.duz.drain(..fazla * ARA_BOYUT);
        self.n = self.ids.len();
    }
}

struct AppState {
    pool: SqlitePool,
    wiki: SqlitePool,
    http: reqwest::Client,
    matris: std::sync::Arc<tokio::sync::RwLock<VektorHavuzu>>,
    /// Vektor havuz tavani (B10): 0 = sinirsiz. (Durum gorunurlugu icin
    /// saklanir; yukleme sirasinda yerel kopya kullanilir.)
    #[allow(dead_code)]
    havuz_tavan: usize,
    /// Dagitim kritik bolumu kilidi: cursor + supurme + claim sabitleme
    /// ayni anda tek gorevde (cift dagitim yarisini bitirir).
    dagitim_kilidi: tokio::sync::Mutex<()>,
    master_pubkey_b64: String,
    corpus: String,
    embed_api: String,
    embed_model: String,
    /// Parça relay dizini (tohum parçalar + C7b'ye kadar transfer aktarması).
    relay_dir: String,
    /// Yedek dizini (tohum kaynağı; dışına çıkılmaz).
    yedek_dir: String,
    /// P2P gorev duyuru kuyrugu (None = yayin kapali). gorev_kaydet basarili
    /// dagitimi buraya atar, shard-abone gorevi mesh'e yayinlar (R4/P2P).
    /// HTTP dagitim yolu bundan etkilenmez (hata yoksayilir).
    gorev_yayin_tx: Option<tokio::sync::mpsc::UnboundedSender<(String, Vec<u8>)>>,
    /// Oz-denetim engeli (mainnet oncesi acilir): denetci kendi kanitini
    /// denetleyemez. Testnet toleransi icin default kapali (sadece warn).
    strict_denetim: bool,
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
    /// B25 sigorta durumu (true = dagitim duraklatildi).
    kesik: bool,
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
    // tip="yedekle" ise doldurulur.
    #[serde(skip_serializing_if = "Option::is_none")]
    yedek: Option<Vec<YedekParca>>,
}

#[derive(Deserialize, Clone)]
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
    // TOKENOMI §kuyruk-taban: era odulu 5 mikro (0,000005) altina INMEZ.
    // Kayma 2000>>n hic 5 uretmez (...,7,3,1,0) — tabanla kilitle, olumu kaldir.
    (BATCH_ODUL_TABAN_MIKRO >> halvings.min(31)).max(KUYRUK_TABAN_MIKRO)
}

/// Guvenlik Md.1: corpus adi tele cikmaz; 8-hex kod gonderilir.
/// Cozum sozlugu yalnizca komuta icindedir (miner icin opak etiket).
fn corpus_kodu(corpus: &str) -> String {
    let h = blake3::hash(corpus.as_bytes());
    hex::encode(&h.as_bytes()[..4])
}

/// B-4: salt'li spot-check. blake3(gorev_id + madde_id + gunluk_salt).
/// Salt gunluk get-or-create ile DB'de tutulur: gun ici deterministik
/// (tekrarlanabilir denetim), gunler arasi ongorulemez (seckinci durustluk engeli).
async fn spot_check_gerekli(pool: &SqlitePool, gorev_id: &str, madde_id: i64) -> bool {
    let now = current_epoch();
    let gun = now / 86400;
    let salt: Vec<u8> = match sqlx::query_scalar("SELECT salt FROM spot_salt WHERE gun = ?")
        .bind(gun)
        .fetch_optional(pool)
        .await
    {
        Ok(Some(s)) => s,
        _ => {
            let mut s = vec![0u8; 32];
            rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut s);
            let _ = sqlx::query("INSERT OR IGNORE INTO spot_salt (gun, salt, ts) VALUES (?, ?, ?)")
                .bind(gun)
                .bind(&s)
                .bind(now)
                .execute(pool)
                .await;
            sqlx::query_scalar("SELECT salt FROM spot_salt WHERE gun = ?")
                .bind(gun)
                .fetch_optional(pool)
                .await
                .unwrap_or(None)
                .unwrap_or(s)
        }
    };
    let mut h = blake3::Hasher::new();
    h.update(gorev_id.as_bytes());
    h.update(&madde_id.to_le_bytes());
    h.update(&salt);
    let digest = h.finalize();
    let yuzde = SPOT_YUZDE_DINAMIK.load(std::sync::atomic::Ordering::Relaxed) as u8;
    (digest.as_bytes()[0] % 100) < yuzde.min(100)
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

/// Kanit defterinden RAM havuzunu kur/genislet: (id, madde, b64, min, max)
/// -> normalize vektorler. Sadece bu komutanin corpus'u alinir.
///
/// 17 Eyl OOM dersi: eski surum `SELECT ... FROM kanitlar` ile TUM satirlari
/// tek `fetch_all` ile RAM'e yigip Rust tarafinda corpus süzüyordu (2.4M satir
/// -> 17-19G anon RSS, kernel OOM-kill dongusu). Yeni surum:
///  1. corpus filtresi SQL'de (`LIKE 'corpus:%'`, idx_kanitlar_gorev kullanir),
///  2. parcali okuma (50K'lik dilimler, sinirli gecici bellek),
///  3. id-artimli yukleme (`after_id`): kanitlar'a silme YOK, o yuzden artimli
///     ekleme tam yuklemeyle birebir esdeger; tazelemede sadece yeni satirlar
///     okunur. Boot `after_id=0` ile tam corpus yukler.
async fn yukle_havuz(
    pool: &SqlitePool,
    corpus: &str,
    after_id: i64,
) -> anyhow::Result<(VektorHavuzu, i64)> {
    const PARCA: i64 = 50_000;
    let onek = format!("{}:%", corpus);
    let mut ids = Vec::new();
    let mut duz = Vec::new();
    let mut son = after_id;
    loop {
        let rows = sqlx::query(
            "SELECT id, madde_id, v_int8_b64, v_min, v_max FROM kanitlar \
             WHERE gorev_id LIKE ? AND id > ? ORDER BY id LIMIT ?",
        )
        .bind(&onek)
        .bind(son)
        .bind(PARCA)
        .fetch_all(pool)
        .await?;
        if rows.is_empty() {
            break;
        }
        let parca_sayisi = rows.len();
        for r in &rows {
            let id: i64 = r.get("id");
            if id > son {
                son = id;
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
        if (parca_sayisi as i64) < PARCA {
            break;
        }
    }
    let n = ids.len();
    Ok((VektorHavuzu { ids, duz, n }, son))
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

/// P2P dugum kimlik tohumu (B18): dosyada 32B saklanir, yoksa uretilir (0600).
/// Ayni tohum = ayni PeerId (tohum listeleri curumez).
fn p2p_anahtar_yukle_veya_uret(yol: &str) -> anyhow::Result<[u8; 32]> {
    use std::os::unix::fs::OpenOptionsExt;
    let p = std::path::Path::new(yol);
    if p.exists() {
        let hexs = std::fs::read_to_string(p)?;
        let raw = hex::decode(hexs.trim())?;
        let arr: [u8; 32] = raw
            .try_into()
            .map_err(|_| anyhow::anyhow!("p2p anahtar dosyasi 32 bayt olmali"))?;
        return Ok(arr);
    }
    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let mut tohum = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut tohum);
    let mut opt = std::fs::OpenOptions::new();
    opt.write(true).create_new(true).mode(0o600);
    use std::io::Write;
    opt.open(p)?.write_all(hex::encode(tohum).as_bytes())?;
    Ok(tohum)
}

/// Anormallik sigortasi (B25 revize Faz B): olagan-disi hizlanma gorurse
/// dagitimi duraklatir (503 + alarm), operator incelemesine birakir.
/// Normal akista HIC devreye girmez: son 5dk kapanisi, 60dk ortalamasinin
/// 10 katini VE mutlak tabani (1000/dk) asmadikca kapalidir. 30dk sonra
/// otomatik yari-acik (saldirm devam ediyorsa yeniden atar).
/// Durustluk notu: kotalama degil, sigorta (durust ciftlik hissetmez).
struct DevreKesici {
    kova: [(i64, i64); 60], // (dakika-no, sayim); eski kova yaslanarak duser
    kesik_son: i64,         // epoch-sn; 0 = kapali degil
}

impl DevreKesici {
    const fn yeni() -> Self {
        Self { kova: [(0, 0); 60], kesik_son: 0 }
    }

    fn slot(simdi: i64) -> usize {
        ((simdi / 60) % 60) as usize
    }

    /// Batch kapanisini isle; esik asilirsa sigortayi atirir.
    fn kapanis_kaydet(&mut self, simdi: i64) {
        let dk = simdi / 60;
        let s = Self::slot(simdi);
        if self.kova[s].0 == dk {
            self.kova[s].1 = self.kova[s].1.saturating_add(1);
        } else {
            self.kova[s] = (dk, 1);
        }
        let _ = self.degerlendir(simdi);
    }

    /// Degerlendirme: son 5 TAM dakika toplami, onceki 55 dakikanin
    /// ortalamasinin 10 kati VE mutlak tabani asarsa true (sigorta atar).
    fn degerlendir(&mut self, simdi: i64) -> bool {
        let dk = simdi / 60;
        let mut son5 = 0i64;
        let mut onceki = 0i64;
        for (d, sayi) in self.kova.iter() {
            if *d == 0 {
                continue; // hic yazilmamis
            }
            let yas = dk.saturating_sub(*d);
            if yas >= 1 && yas <= 5 {
                son5 += *sayi;
            } else if yas > 5 && yas <= 60 {
                onceki += *sayi;
            }
        }
        // Taban: 55dk pencerede is yoksa oran anlamsiz (ilk saatler / bos ag).
        if onceki < 100 {
            return false;
        }
        let esik_oran = onceki / 55 * 10 * 5; // 55dk ortalamasinin 10 kati, 5dk'lik
        const MUTLAK_TABAN_5DK: i64 = 5000; // 1000/dk
        if son5 > esik_oran.max(MUTLAK_TABAN_5DK) {
            if self.kesik_son <= simdi {
                self.kesik_son = simdi + 1800;
            }
            return true;
        }
        false
    }

    /// Dagitim acik mi? Sure dolduysa otomatik yari-acik (sifirla).
    fn dagitim_acik(&mut self, simdi: i64) -> bool {
        if self.kesik_son == 0 || simdi >= self.kesik_son {
            self.kesik_son = 0;
            return true;
        }
        false
    }

    fn kesik_mi(&self, simdi: i64) -> bool {
        self.kesik_son != 0 && simdi < self.kesik_son
    }
}

static KESICI: std::sync::LazyLock<std::sync::Mutex<DevreKesici>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(DevreKesici::yeni()));
/// bekleme + sayım. 500ms üstü tekil warn, her 1000 kilitte ortalama info.
/// Eşik aşımı = federasyon ihtiyacı sinyali (yol haritası Faz 2).
static KILIT_TOPLAM_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static KILIT_SAYI: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

async fn kilit_al(kilit: &tokio::sync::Mutex<()>) -> tokio::sync::MutexGuard<'_, ()> {
    let t0 = std::time::Instant::now();
    let g = kilit.lock().await;
    let ms = t0.elapsed().as_millis() as u64;
    let n = KILIT_SAYI.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    let top = KILIT_TOPLAM_MS.fetch_add(ms, std::sync::atomic::Ordering::Relaxed) + ms;
    if ms > 500 {
        warn!("kilit-bekleme yuksek: {}ms ({}-nci kilit)", ms, n);
    } else if n % 1000 == 0 {
        info!("kilit-istatistik: {} kilit, ortalama {}ms", n, top / n.max(1));
    }
    g
}

/// Shard ilanini kaydet: imza + kayit + TOFU pubkey bagi + cursor'a
/// sabitleme + cakisma kontrolu. Donen: (baslangic, bitis).
async fn kaydet_ilan(state: &Arc<AppState>, ilan: &ShardIlan, kaynak: &str) -> Result<(i64, i64), String> {
    ilan.dogrula().map_err(|e| format!("ilan gecersiz: {}", e))?;
    let pool = &state.pool;
    // Sabitleme kritik bolumde: kontrol-et + yaz tek sira (cift sabitleme yarisini bitirir).
    let _kilit = kilit_al(&state.dagitim_kilidi).await;

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
    let kesik = KESICI
        .lock()
        .map(|k| k.kesik_son > current_epoch())
        .unwrap_or(false);
    Json(HealthResp {
        ok: true,
        ts: Utc::now().format("%m-%d %H:%M:%S").to_string(),
        kesik,
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

/// Heartbeat govdesi (tamami opsiyonel — eski istemciler bos gonderir).
/// parcalar: miner'in tuttugu parca hash'leri + boyutlari.
/// depolama_kota: taahhut edilen kota (bayt), varsa guncellenir.
#[derive(Debug, Deserialize)]
struct HeartbeatParca {
    parca_hash: String,
    boyut: i64,
}

#[derive(Debug, Deserialize)]
struct HeartbeatReq {
    parcalar: Option<Vec<HeartbeatParca>>,
    depolama_kota: Option<i64>,
    /// B23 kabiliyet ilani (heterojen mesh): donanim + roller.
    yetenek: Option<HeartbeatYetenek>,
}

#[derive(Debug, Deserialize)]
struct HeartbeatYetenek {
    gpu_ad: Option<String>,
    vram_mb: Option<i64>,
    roller: Option<Vec<String>>,
}

/// Toplu kayit (B25 ciftlik paketi): bir cuzdana N madenci (cap 200).
/// Kimlikler ucuzdur (guvenlik ispatta, kimlikte degil); asil deger
/// ciftligin tek cagrida filosunu acmasidir. Her kalem bagimsiz satirdir.
const TOPLU_KAYIT_EN_FAZLA: i64 = 200;

#[derive(Deserialize)]
struct TopluKayitIstek {
    cuzdan: String,
    adet: Option<i64>,
    makine_onek: Option<String>,
}

async fn register_toplu(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TopluKayitIstek>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let cuzdan = req.cuzdan.trim().to_string();
    if cuzdan.is_empty() || cuzdan.len() > 128 {
        return Err((StatusCode::BAD_REQUEST, "gecersiz cuzdan".to_string()));
    }
    let adet = req.adet.unwrap_or(1).clamp(1, TOPLU_KAYIT_EN_FAZLA);
    let onek: String = req
        .makine_onek
        .unwrap_or_else(|| "ciftlik".to_string())
        .chars()
        .take(40)
        .collect();
    let now = current_epoch();
    let mut liste = Vec::with_capacity(adet as usize);
    for i in 0..adet {
        let token = Uuid::new_v4().to_string().replace("-", "");
        let miner_id = format!("miner-{}", &token[..8]);
        let makine = format!("{}-{:04}", onek, i + 1);
        sqlx::query(
            "INSERT INTO miners (miner_id, token, cuzdan, makine_id, created_at, coin_mikro) VALUES (?, ?, ?, ?, ?, 0)"
        )
        .bind(&miner_id)
        .bind(&token)
        .bind(&cuzdan)
        .bind(&makine)
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
        liste.push(serde_json::json!({ "miner_id": miner_id, "token": token, "makine_id": makine }));
    }
    info!("Toplu kayit: {} madenci <- {} ({})", adet, cuzdan, onek);
    Ok(Json(serde_json::json!({ "adet": adet, "madenciler": liste })))
}

async fn heartbeat(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Option<Json<HeartbeatReq>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let miner_id = token_dogrula(&headers, &state.pool).await?;
    let now = current_epoch();
    sqlx::query("UPDATE miners SET last_seen = ? WHERE miner_id = ?")
        .bind(now)
        .bind(&miner_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut bildirilen: usize = 0;
    if let Some(Json(req)) = body {
        // Kota beyanı (taahhut dosyasındaki değer; C5'teki sütuna işlenir).
        if let Some(kota) = req.depolama_kota {
            if kota >= 0 {
                sqlx::query("UPDATE miners SET depolama_kota = ? WHERE miner_id = ?")
                    .bind(kota)
                    .bind(&miner_id)
                    .execute(&state.pool)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            }
        }
        // B23 kabiliyet ilani: donanim + roller kaydi (upsert; sinirli uzunluk).
        // Gondermeyen eski miner'lar etkilenmez (satir acilmaz).
        if let Some(y) = req.yetenek {
            let gpu: String = y.gpu_ad.unwrap_or_default().chars().take(120).collect();
            let vram = y.vram_mb.unwrap_or(0).clamp(0, 1 << 30);
            let mut roller: Vec<String> = y
                .roller
                .unwrap_or_default()
                .into_iter()
                .map(|r| r.chars().take(24).collect::<String>())
                .filter(|r| !r.is_empty())
                .take(8)
                .collect();
            roller.sort();
            roller.dedup();
            if roller.is_empty() {
                roller.push("embed".to_string());
            }
            let rol_str = roller.join(",");
            sqlx::query(
                "INSERT INTO miner_yetenek (miner_id, gpu_ad, vram_mb, roller, ts) VALUES (?, ?, ?, ?, ?) ON CONFLICT(miner_id) DO UPDATE SET gpu_ad = excluded.gpu_ad, vram_mb = excluded.vram_mb, roller = excluded.roller, ts = excluded.ts"
            )
            .bind(&miner_id)
            .bind(&gpu)
            .bind(vram)
            .bind(&rol_str)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
        // Parça canlılık raporu: yer bilgisini tazele (ölüm ilanı C7'de okur).
        if let Some(parcalar) = req.parcalar {
            for p in parcalar.iter().take(4096) {
                if p.parca_hash.len() != 64 || p.boyut <= 0 {
                    continue;
                }
                sqlx::query(
                    "INSERT INTO parcalar (parca_hash, boyut, ts) VALUES (?, ?, ?) ON CONFLICT(parca_hash) DO NOTHING"
                )
                .bind(&p.parca_hash)
                .bind(p.boyut)
                .bind(now)
                .execute(&state.pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                sqlx::query(
                    "INSERT INTO parca_yerleri (parca_hash, miner_id, ts) VALUES (?, ?, ?) ON CONFLICT(parca_hash, miner_id) DO UPDATE SET ts = excluded.ts"
                )
                .bind(&p.parca_hash)
                .bind(&miner_id)
                .bind(now)
                .execute(&state.pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                bildirilen += 1;
                // Açık onarım kapanışı + hedef ücreti (C7).
                let kapanan = sqlx::query(
                    "UPDATE onarimlar SET durum = 'tamam' WHERE parca_hash = ? AND hedef_miner = ? AND durum = 'acik'"
                )
                .bind(&p.parca_hash)
                .bind(&miner_id)
                .execute(&state.pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                .rows_affected();
                if kapanan > 0 {
                    sqlx::query("UPDATE miners SET coin_mikro = coin_mikro + ? WHERE miner_id = ?")
                        .bind(ONARIM_ODUL_MIKRO * kapanan as i64)
                        .bind(&miner_id)
                        .execute(&state.pool)
                        .await
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                    sqlx::query(
                        "INSERT INTO ledger (miner_id, delta_mikro, neden, epoch, ts) VALUES (?, ?, 'onarim', ?, ?)"
                    )
                    .bind(&miner_id)
                    .bind(ONARIM_ODUL_MIKRO * kapanan as i64)
                    .bind(now)
                    .bind(now)
                    .execute(&state.pool)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                    info!("onarim kapandi: {} parca -> {} (+{} mikro)", p.parca_hash.chars().take(12).collect::<String>(), miner_id, ONARIM_ODUL_MIKRO * kapanan as i64);
                }
            }
        }
    }

    Ok(Json(serde_json::json!({
        "ok": true,
        "ts": Utc::now().format("%m-%d %H:%M:%S").to_string(),
        "parca_bildirilen": bildirilen,
    })))
}

/// Kira girisi (B20): madenci aralik kiralar, ~100 batch sormadan calisir.
/// Kapilar tekil yolla ayni (kara + strike). Bos aralik -> 204 (madenci eski
/// yola duser). Yanit: alt-gorev listesi (her biri bagimsiz batch).
async fn kira_al(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(istek): Json<KiraIstek>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let miner_id = token_dogrula(&headers, &state.pool).await?;

    let kara: Option<String> = sqlx::query_scalar("SELECT neden FROM kara_liste WHERE miner_id = ?")
        .bind(&miner_id).fetch_optional(&state.pool).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if let Some(neden) = kara {
        return Err((StatusCode::FORBIDDEN, format!("kara liste ({})", neden)));
    }

    let strike: i64 = sqlx::query_scalar("SELECT strike FROM miners WHERE miner_id = ?")
        .bind(&miner_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if strike >= STRIKE_LIMIT {
        return Err((StatusCode::FORBIDDEN, "3 strike - gorev akisi kesildi".to_string()));
    }

    // B25 sigorta: kesikse kira dagitimi durur.
    {
        let now_k = current_epoch();
        if let Ok(mut kesici) = KESICI.lock() {
            if !kesici.dagitim_acik(now_k) {
                return Err((StatusCode::SERVICE_UNAVAILABLE, "devre kesik - inceleme suruyor".to_string()));
            }
        }
    }

    let adet = istek.adet.unwrap_or(KIRA_VARSAYILAN);
    let liste = kira_kur(&state, &miner_id, adet).await?;
    // Bos liste = aralik yok -> madenci eski yola duser (HTTP 200 + bos dizi).
    Ok(Json(serde_json::json!({ "gorevler": liste })))
}

/// Teklif sabitleri (B21 kendi-kendine ogrenme): stake, vade, esikler.
const TEKLIF_STAKE_MIKRO: i64 = 500;
const TEKLIF_VADE_SN: i64 = 7 * 86400;
const TEKLIF_KAPSAMA_RED: f64 = 0.5; // bu oranda kapaliysa teklif RED (ucretsiz)
const TEKLIF_KAPSAMA_BITIS: f64 = 0.8; // bu orana ulasirsa tamam + iade
const TEKLIF_MAX_ARALIK: i64 = 100000;

/// Aralik kapsama orani (B21): kanitlanmis farkli madde / genislik.
async fn aralik_kapsama(
    pool: &SqlitePool,
    baslangic: i64,
    bitis: i64,
) -> Result<(i64, i64), (StatusCode, String)> {
    let genislik = (bitis - baslangic + 1).max(1);
    let kapali: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT madde_id) FROM kanitlar WHERE madde_id >= ? AND madde_id <= ?",
    )
    .bind(baslangic)
    .bind(bitis)
    .fetch_one(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((kapali, genislik))
}

#[derive(Deserialize)]
struct TeklifIstek {
    corpus: String,
    baslangic: i64,
    bitis: i64,
}

/// Bosluk teklifi ver (B21): stake kilitlenir, aralik supurme onceligi kazanir.
async fn teklif_ver(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(istek): Json<TeklifIstek>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let miner_id = token_dogrula(&headers, &state.pool).await?;
    let corpus = istek.corpus.trim().to_string();
    if corpus.is_empty() || istek.bitis <= istek.baslangic {
        return Err((StatusCode::BAD_REQUEST, "gecersiz aralik".to_string()));
    }
    if istek.bitis - istek.baslangic + 1 > TEKLIF_MAX_ARALIK {
        return Err((StatusCode::BAD_REQUEST, "aralik cok genis".to_string()));
    }
    if corpus != state.corpus {
        return Err((StatusCode::BAD_REQUEST, "su anki corpus degil".to_string()));
    }
    // Ayni madencinin acik cakisan teklifi varsa yenisini acma, mevcudu dondur.
    if let Some(row) = sqlx::query(
        "SELECT id FROM teklifler WHERE durum = 'acik' AND miner_id = ? AND corpus = ? AND baslangic <= ? AND bitis >= ? ORDER BY ts ASC LIMIT 1",
    )
    .bind(&miner_id)
    .bind(&corpus)
    .bind(istek.bitis)
    .bind(istek.baslangic)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        let id: i64 = row.get("id");
        return Ok(Json(serde_json::json!({ "teklif_id": id, "durum": "mevcut" })));
    }
    // Kapsama redde: zaten kapali araliga teklif ucretsiz RED.
    let (kapali, genislik) = aralik_kapsama(&state.pool, istek.baslangic, istek.bitis).await?;
    if kapali as f64 / genislik as f64 >= TEKLIF_KAPSAMA_RED {
        return Err((StatusCode::CONFLICT, "aralik zaten kapali".to_string()));
    }
    // Bakiye + stake kilidi (slash deseni: coin dus, ledger negatif).
    let bakiye: i64 = sqlx::query_scalar("SELECT coin_mikro FROM miners WHERE miner_id = ?")
        .bind(&miner_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if bakiye < TEKLIF_STAKE_MIKRO {
        return Err((StatusCode::PAYMENT_REQUIRED, "stake bakiyesi yetmez".to_string()));
    }
    let now = current_epoch();
    sqlx::query("UPDATE miners SET coin_mikro = coin_mikro - ? WHERE miner_id = ?")
        .bind(TEKLIF_STAKE_MIKRO)
        .bind(&miner_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    sqlx::query(
        "INSERT INTO ledger (miner_id, delta_mikro, neden, epoch, ts) VALUES (?, ?, 'teklif-kilit', ?, ?)",
    )
    .bind(&miner_id)
    .bind(-TEKLIF_STAKE_MIKRO)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let r = sqlx::query(
        "INSERT INTO teklifler (miner_id, corpus, baslangic, bitis, stake_mikro, durum, ts) VALUES (?, ?, ?, ?, ?, 'acik', ?)",
    )
    .bind(&miner_id)
    .bind(&corpus)
    .bind(istek.baslangic)
    .bind(istek.bitis)
    .bind(TEKLIF_STAKE_MIKRO)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let id = r.last_insert_rowid();
    info!("teklif acildi: #{} {} [{},{}] <- {}", id, corpus, istek.baslangic, istek.bitis, miner_id);
    Ok(Json(serde_json::json!({ "teklif_id": id, "durum": "acik" })))
}

/// Bosluk yayincisi (B21): geri kalmis pencereler + acik teklifler (ucuz sorgular).
async fn bosluklar(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let _miner_id = token_dogrula(&headers, &state.pool).await?;
    let now = current_epoch();
    // Cursor-supurme makasi (bilinen geri kalmislik).
    let son: i64 = sqlx::query_scalar("SELECT son_id FROM gorev_cursor WHERE corpus = ?")
        .bind(&state.corpus)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .unwrap_or(0);
    let sup_key = format!("supurme:{}", state.corpus);
    let sup: i64 = sqlx::query_scalar("SELECT son_id FROM gorev_cursor WHERE corpus = ?")
        .bind(&sup_key)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .unwrap_or(0);
    // Kapanmamis eski batch'ler (olu gorev supurme adaylari).
    let oluler = sqlx::query(
        "SELECT gorev_id, offset FROM gorevler WHERE durum = 'acik' AND ts < ? ORDER BY ts ASC LIMIT 10",
    )
    .bind(now - 3600)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let olu_liste: Vec<serde_json::Value> = oluler
        .iter()
        .map(|r| {
            let g: String = r.get("gorev_id");
            let o: i64 = r.get("offset");
            serde_json::json!({ "gorev_id": g, "offset": o })
        })
        .collect();
    let acik = sqlx::query(
        "SELECT id, miner_id, corpus, baslangic, bitis, ts FROM teklifler WHERE durum = 'acik' ORDER BY ts ASC LIMIT 20",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let acik_liste: Vec<serde_json::Value> = acik
        .iter()
        .map(|r| {
            let id: i64 = r.get("id");
            let m: String = r.get("miner_id");
            let c: String = r.get("corpus");
            let b0: i64 = r.get("baslangic");
            let b1: i64 = r.get("bitis");
            serde_json::json!({ "id": id, "miner": m, "corpus": c, "baslangic": b0, "bitis": b1 })
        })
        .collect();
    Ok(Json(serde_json::json!({
        "cursor": son,
        "supurme": sup,
        "makas": (son - sup).max(0),
        "olu_gorevler": olu_liste,
        "acik_teklifler": acik_liste,
    })))
}

/// Madencinin teklifleri (B21): durum + kazanilan pay ozeti.
async fn tekliflerim(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let miner_id = token_dogrula(&headers, &state.pool).await?;
    let rows = sqlx::query(
        "SELECT id, corpus, baslangic, bitis, stake_mikro, durum, ts FROM teklifler WHERE miner_id = ? ORDER BY ts DESC LIMIT 50",
    )
    .bind(&miner_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let kazanc: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(delta_mikro),0) FROM ledger WHERE miner_id = ? AND neden IN ('teklif-odul','teklif-iade')",
    )
    .bind(&miner_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let liste: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let id: i64 = r.get("id");
            let c: String = r.get("corpus");
            let b0: i64 = r.get("baslangic");
            let b1: i64 = r.get("bitis");
            let d: String = r.get("durum");
            serde_json::json!({ "id": id, "corpus": c, "baslangic": b0, "bitis": b1, "durum": d })
        })
        .collect();
    Ok(Json(serde_json::json!({ "teklifler": liste, "kazanc_mikro": kazanc })))
}

async fn gorev(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let miner_id = token_dogrula(&headers, &state.pool).await?;

    // Kara liste (Guvenlik Md.4): imha edilen gorev de alamaz, kanit da veremez.
    let kara: Option<String> = sqlx::query_scalar("SELECT neden FROM kara_liste WHERE miner_id = ?")
        .bind(&miner_id).fetch_optional(&state.pool).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if let Some(neden) = kara {
        warn!("Kara listedeki miner gorev istedi: {} ({})", miner_id, neden);
        return Err((StatusCode::FORBIDDEN, "kara liste (operator affeti gerekli)".to_string()));
    }

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

    // B25 sigorta: kesikse dagitim durur (miner retry ile bekler).
    {
        let now_k = current_epoch();
        if let Ok(mut kesici) = KESICI.lock() {
            if !kesici.dagitim_acik(now_k) {
                return Err((StatusCode::SERVICE_UNAVAILABLE, "devre kesik - inceleme suruyor".to_string()));
            }
        }
    }

    // Kuyruk biriktiyse ~5 gorevde 1 denetim dagit (es-dogrulama).
    if rand::thread_rng().gen_range(0..5) == 0 {
        if let Some(d) = dagit_denetim(&state, &miner_id).await? {
            return Ok(Json(d).into_response());
        }
    }

    // Kritik bolum: cursor oku -> dagit -> cursor yaz tek sira.
    let _kilit = kilit_al(&state.dagitim_kilidi).await;
    let now_ts0b = current_epoch();

    // Onarım önceliği: kotası olan miner'a eksik kopyalı parçaları ver (C7).
    // gorevler tablosuna 'yedekle' durumuyla işlenir (batch sayacını kirletmez).
    {
        let plan = onarim_planla(&state, &miner_id, now_ts0b).await?;
        if !plan.is_empty() {
            let gid = format!("yedekle:{}:{}", now_ts0b, &Uuid::new_v4().simple().to_string()[..12]);
            let yedek: Vec<YedekParca> = plan.iter().map(|(h, b, oid)| YedekParca {
                parca_hash: h.clone(), boyut: *b, onarim_id: *oid,
            }).collect();
            let n = yedek.len() as i64;
            sqlx::query(
                "INSERT OR IGNORE INTO gorevler (gorev_id, dagitilan_miner, corpus, offset, beklenen, alinan, durum, odul_mikro, ts) VALUES (?, ?, ?, 0, ?, 0, 'yedekle', 0, ?)"
            )
            .bind(&gid).bind(&miner_id).bind(&state.corpus).bind(n).bind(now_ts0b)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            info!("Yedek gorevi: {} ({} parca) -> {}", gid, n, miner_id);
            return Ok(Json(GorevResp {
                id: gid,
                tip: "yedekle".to_string(),
                corpus: state.corpus.clone(),
                shard: 0,
                offset: 0,
                limit: n,
                deadline: now_ts0b + TASK_DEADLINE_SEC,
                payload: GorevPayload { metinler: Vec::new(), madde_idler: Vec::new(), denetim: None, yedek: Some(yedek) },
            }).into_response());
        }
    }

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

/// Kira sinirlari (B20 olcek): tek cagrida en fazla madde.
const KIRA_EN_FAZLA: i64 = 20000;
const KIRA_VARSAYILAN: i64 = 2000;

#[derive(Deserialize)]
struct KiraIstek {
    adet: Option<i64>,
}

/// Kira ile aralik kur (B20): TEK kilit, TEK wiki okuma, toplu kayitlar.
/// `/api/gorev` tekil yoluna DOKUNULMAZ. Kira = N alt-gorev (20'lik dilimler);
/// madenci kirayi bir kez alir, ~100 batch gorev sormadan calisir.
/// Bitmeyen araliklar supurme ile dolar (mevcut mekanizma, yeni kod yok).
async fn kira_kur(
    state: &Arc<AppState>,
    miner_id: &str,
    adet_istek: i64,
) -> Result<Vec<GorevResp>, (StatusCode, String)> {
    let adet = adet_istek.clamp(TASK_BATCH, KIRA_EN_FAZLA);
    let _kilit = kilit_al(&state.dagitim_kilidi).await;
    let now_ts = current_epoch();

    let row = sqlx::query("SELECT son_id FROM gorev_cursor WHERE corpus = ?")
        .bind(&state.corpus)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let son_id: i64 = row.map(|r| r.get("son_id")).unwrap_or(0);

    // Cursor arkasinda kalan claim'leri kapat (tekil yolla ayni).
    sqlx::query("UPDATE shard_ilanlari SET durum = 'tukendi' WHERE corpus = ? AND durum = 'aktif' AND bitis <= ?")
        .bind(&state.corpus)
        .bind(son_id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Baskasinin aktif + taze claim'inin ustunden ATLA (tekil yolla ayni).
    let mut start = son_id;
    for _ in 0..8 {
        let hit = sqlx::query(
            "SELECT baslangic, bitis FROM shard_ilanlari WHERE corpus = ? AND durum = 'aktif' AND ts > ? AND miner_id != ? AND baslangic <= ? AND ? <= bitis ORDER BY baslangic LIMIT 1"
        )
        .bind(&state.corpus)
        .bind(now_ts - SHARD_SURE_SN)
        .bind(&miner_id)
        .bind(start + 1)
        .bind(start + 1)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        match hit {
            Some(h) => {
                let e: i64 = h.get("bitis");
                start = e;
            }
            None => break,
        }
    }

    // TEK wiki aralik okuma (state.wiki havuzundan; cagri basi baglanti yok).
    // Uretimde wiki_{corpus}.db (salt-okunur), testte :memory: (tohumlanir).
    let rows = sqlx::query("SELECT id, ozet FROM madde WHERE id > ? AND length(ozet) > 20 ORDER BY id LIMIT ?")
        .bind(start)
        .bind(adet)
        .fetch_all(&state.wiki)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let kira_id = format!("kira:{}:{}", now_ts, &Uuid::new_v4().simple().to_string()[..12]);
    // Kira sirri: kor ID'ler blake3(sir+dilim+sira)'dan turetilir (tahmin
    // edilemez; tek toplu cakisma kontrolu yeterli — tekil yoldaki RNG+SELECT
    // dongusu yerine gecer).
    let mut sir = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut sir);
    let kor_uret = |dilim_no: i64, i: u64| -> i64 {
        let mut h = blake3::Hasher::new();
        h.update(&sir);
        h.update(&dilim_no.to_le_bytes());
        h.update(&i.to_le_bytes());
        (u64::from_le_bytes(h.finalize().as_bytes()[..8].try_into().unwrap_or([0; 8])) >> 1) as i64
    };
    let mut tum_kor: Vec<(i64, i64, String)> = Vec::new(); // (kor, madde, gid)
    let mut tum_madde: Vec<i64> = Vec::new();
    let mut sonuclar: Vec<GorevResp> = Vec::new();
    let mut dilim_no = 0i64;
    for dilim in rows.chunks(TASK_BATCH as usize) {
        let madde_idler: Vec<i64> = dilim.iter().map(|r| r.get("id")).collect();
        let mut metinler: Vec<String> = dilim.iter().map(|r| r.get::<String, _>("ozet").chars().take(4500).collect()).collect();
        let offset = madde_idler[0];
        let gid = format!("{}:{}:{}:k{}-{}", state.corpus, offset, madde_idler.len(), kira_id, dilim_no);
        // KANARYA (tekil yolla ayni, %4): sentetik cumle + negatif isaret.
        let mut kanarya_kid = 0i64;
        {
            let roll: f64 = rand::thread_rng().gen_range(0.0..1.0);
            if roll < kanarya_orani() {  // B25 bagisiklik (dinamik)
                if let Ok(krow) = sqlx::query("SELECT id, metin FROM kanaryalar WHERE aktif = 1 ORDER BY RANDOM() LIMIT 1")
                    .fetch_optional(&state.pool).await
                {
                    if let Some(k) = krow {
                        let kid: i64 = k.get("id");
                        let kmetin: String = k.get("metin");
                        metinler.push(kmetin);
                        kanarya_kid = kid;
                        let _ = sqlx::query("INSERT INTO kanarya_dagitim (kanarya_id, gorev_id, miner_id, ts) VALUES (?, ?, ?, ?)")
                            .bind(kid).bind(&gid).bind(miner_id).bind(now_ts)
                            .execute(&state.pool).await;
                    }
                }
            }
        }
        // Kor listesi (tekil yolun madde_idler'i = kor_idler): gercekler + kanarya.
        let mut kors: Vec<i64> = Vec::with_capacity(metinler.len());
        for (i, m) in madde_idler.iter().enumerate() {
            let kor = kor_uret(dilim_no, i as u64);
            kors.push(kor);
            tum_kor.push((kor, *m, gid.clone()));
        }
        if kanarya_kid != 0 {
            let kor = kor_uret(dilim_no, madde_idler.len() as u64);
            kors.push(kor);
            tum_kor.push((kor, -kanarya_kid, gid.clone()));
        }
        sqlx::query(
            "INSERT OR IGNORE INTO gorevler (gorev_id, dagitilan_miner, corpus, offset, beklenen, alinan, durum, odul_mikro, ts) VALUES (?, ?, ?, ?, ?, 0, 'acik', 0, ?)"
        )
        .bind(&gid)
        .bind(miner_id)
        .bind(&state.corpus)
        .bind(offset)
        .bind(metinler.len() as i64)
        .bind(now_ts)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        tum_madde.extend(madde_idler.iter());
        sonuclar.push(GorevResp {
            id: gid,
            tip: "embed".to_string(),
            corpus: corpus_kodu(&state.corpus),
            shard: 0,
            offset,
            limit: metinler.len() as i64,
            deadline: now_ts + TASK_DEADLINE_SEC,
            payload: GorevPayload { metinler, madde_idler: kors, denetim: None, yedek: None },
        });
        dilim_no += 1;
    }
    // TEK toplu cakisma kontrolu (63-bit alanda carpisma pratikte imkansiz).
    if !tum_kor.is_empty() {
        let yerler = tum_kor.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let q = format!("SELECT kor_id FROM kor_esleme WHERE kor_id IN ({})", yerler);
        let mut qq = sqlx::query_scalar::<_, i64>(&q);
        for (k, _, _) in &tum_kor {
            qq = qq.bind(k);
        }
        let carpisan: Vec<i64> = qq.fetch_all(&state.pool).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        if !carpisan.is_empty() {
            return Err((StatusCode::CONFLICT, "kor cakismasi (tekrar deneyin)".to_string()));
        }
        // TEK toplu kor eslemesi.
        let degerler = tum_kor.iter().map(|_| "(?, ?, ?, ?)").collect::<Vec<_>>().join(",");
        let qi = format!("INSERT OR IGNORE INTO kor_esleme (kor_id, madde_id, gorev_id, ts) VALUES {}", degerler);
        let mut qqi = sqlx::query(&qi);
        for (k, m, g) in &tum_kor {
            qqi = qqi.bind(k).bind(m).bind(g).bind(now_ts);
        }
        qqi.execute(&state.pool).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        // TEK toplu dagitim izi (500'luk parcalar; SQLite degisken siniri icin).
        for chunk in tum_madde.chunks(500) {
            let qd = format!("INSERT INTO dagitilan_madde (madde_id, ts) VALUES {} ON CONFLICT(madde_id) DO UPDATE SET ts = excluded.ts",
                chunk.iter().map(|_| "(?, ?)").collect::<Vec<_>>().join(","));
            let mut qdq = sqlx::query(&qd);
            for m in chunk {
                qdq = qdq.bind(m).bind(now_ts);
            }
            qdq.execute(&state.pool).await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
    }
    // Cursor tek yazma (monoton).
    if let Some(son) = tum_madde.iter().max() {
        let ileri = (*son).max(son_id);
        sqlx::query(
            "INSERT INTO gorev_cursor (corpus, son_id) VALUES (?, ?) ON CONFLICT(corpus) DO UPDATE SET son_id = excluded.son_id"
        )
        .bind(&state.corpus)
        .bind(ileri)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    // TEK duyuru (100 yerine 1 gossip).
    if !sonuclar.is_empty() {
        let ilk = &sonuclar[0];
        let duyuru = serde_json::json!({"kira_id": kira_id, "gorev_id": ilk.id, "corpus": corpus_kodu(&state.corpus), "madde": tum_madde.len(), "ts": now_ts});
        if let Ok(raw) = serde_json::to_vec(&duyuru) {
            if let Some(tx) = state.gorev_yayin_tx.as_ref() {
                let _ = tx.send((GOREV_TOPIC.to_string(), raw));
            }
        }
        info!("kira kuruldu: {} ({} alt-gorev, {} madde) -> {}", kira_id, sonuclar.len(), tum_madde.len(), miner_id);
    }
    Ok(sonuclar)
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
    let mut metinler = metinler;
    let mut madde_idler = madde_idler;
    let n0 = madde_idler.len();
    let gid = format!("{}:{}:{}:{}", state.corpus, offset, n0, Uuid::new_v4().simple());
    let now_ts = current_epoch();
    // KANARYA (Guvenlik Md.3): %4 olasilikla sentetik filigranli cumle serpistir.
    // madde_id = -kanarya_id (negatif = sentetik isaret, gercek id ile cakismaz).
    // Dagitim kanarya_dagitim'a yazilir; disarida gorulurse kaynak bellidir.
    {
        let roll: f64 = rand::thread_rng().gen_range(0.0..1.0);
        if roll < kanarya_orani() {  // B25 bagisiklik (dinamik)
            if let Ok(krow) = sqlx::query("SELECT id, metin FROM kanaryalar WHERE aktif = 1 ORDER BY RANDOM() LIMIT 1")
                .fetch_optional(&state.pool).await
            {
                if let Some(k) = krow {
                    let kid: i64 = k.get("id");
                    let kmetin: String = k.get("metin");
                    metinler.push(kmetin);
                    madde_idler.push(-kid);
                    let _ = sqlx::query("INSERT INTO kanarya_dagitim (kanarya_id, gorev_id, miner_id, ts) VALUES (?, ?, ?, ?)")
                        .bind(kid).bind(&gid).bind(miner_id).bind(now_ts)
                        .execute(&state.pool).await;
                }
            }
        }
    }
    let n = madde_idler.len();
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
    // KOR ID (Guvenlik Md.1): miner'a gercek madde_id gitmez. Dagitim basina
    // rastgele 63-bit kor ID uretilir, esleme DB'de tutulur, kanit/denetim
    // donusunde cozulur. dagitilan_madde/supurme ic izlerde GERCEK id kalir.
    // NOT: RNG gecici tutulur (await oncesi duser) ki future Send kalsin.
    let mut kor_idler: Vec<i64> = Vec::with_capacity(n);
    {
        let mut q = String::from("INSERT OR IGNORE INTO kor_esleme (kor_id, madde_id, gorev_id, ts) VALUES ");
        q.push_str(&madde_idler.iter().map(|_| "(?, ?, ?, ?)").collect::<Vec<_>>().join(","));
        let mut qq = sqlx::query(&q);
        for m in &madde_idler {
            // Cakisma olursa (OR IGNORE) yeniden cek.
            let kor: i64 = loop {
                let k: i64 = rand::thread_rng().gen_range(1..i64::MAX);
                let var: Option<i64> = sqlx::query_scalar("SELECT kor_id FROM kor_esleme WHERE kor_id = ?")
                    .bind(k).fetch_optional(&state.pool).await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                if var.is_none() { break k; }
            };
            kor_idler.push(kor);
            qq = qq.bind(kor).bind(m).bind(&gid).bind(now_ts);
        }
        qq.execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    info!("Görev {}: {} ({} madde) -> {}", etiket, gid, n, miner_id);
    // R4/P2P: hafif duyuru (gorev_id + corpus hash). Gercek yuk HTTP ile
    // alinir, gossip'e hassas veri konmaz. Kuyruk dolu/kapaliysa atlanir.
    if let Some(tx) = state.gorev_yayin_tx.as_ref() {
        let duyuru = serde_json::json!({"gorev_id": gid, "corpus": corpus_kodu(&state.corpus), "ts": now_ts});
        if let Ok(raw) = serde_json::to_vec(&duyuru) {
            let _ = tx.send((GOREV_TOPIC.to_string(), raw));
        }
    }
    Ok(GorevResp {
        id: gid,
        tip: "embed".to_string(),
        corpus: corpus_kodu(&state.corpus),
        shard: 0,
        offset,
        limit: n as i64,
        deadline: current_epoch() + TASK_DEADLINE_SEC,
        payload: GorevPayload { metinler, madde_idler: kor_idler, denetim: None, yedek: None },
    })
}

/// Supurme: cursor gerisinde kalmis, kaniti olmayan madde id'lerini bul.
/// supurme cursor'u `gorev_cursor.supurme:<corpus>` satirinda tutulur;
/// yakalayinca basa sarar (olumcul batch'ler bir sonraki turda tekrar denenir).
/// Donen: (offset, metinler, madde_idler). Yoksa None.
/// Teklif oncelikli supurme (B21): en eski acik teklifin kapsanmamis ilk 20'si.
/// Teklif yoksa/kapandiysa None (normal supurgeye dusulur).
/// ATTACH yerine 3 sinirli sorgu (testte :memory: wiki ile calisir).
async fn teklif_supurme_dene(
    state: &Arc<AppState>,
) -> Result<Option<(i64, Vec<String>, Vec<i64>)>, (StatusCode, String)> {
    let acik: Vec<(i64, String, i64, i64)> = sqlx::query_as(
        "SELECT id, corpus, baslangic, bitis FROM teklifler WHERE durum = 'acik' ORDER BY ts ASC LIMIT 5",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let now0 = current_epoch();
    for (tid, tcorpus, b0, b1) in acik {
        if tcorpus != state.corpus {
            continue;
        }
        // Aday metinler (wiki havuzundan; uretimde corpus DB, testte tohumlu).
        let adaylar: Vec<(i64, String)> = sqlx::query_as(
            "SELECT id, ozet FROM madde WHERE id >= ? AND id <= ? AND length(ozet) > 20 ORDER BY id",
        )
        .bind(b0)
        .bind(b1)
        .fetch_all(&state.wiki)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        if adaylar.is_empty() {
            continue;
        }
        // Kanitlanmis + taze dagitilmis kumeler (aralik sinirli).
        let kanitli: std::collections::HashSet<i64> =
            sqlx::query_scalar("SELECT DISTINCT madde_id FROM kanitlar WHERE madde_id >= ? AND madde_id <= ?")
                .bind(b0)
                .bind(b1)
                .fetch_all(&state.pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                .into_iter()
                .collect();
        let dagitilmis: std::collections::HashSet<i64> =
            sqlx::query_scalar("SELECT madde_id FROM dagitilan_madde WHERE madde_id >= ? AND madde_id <= ? AND ts > ?")
                .bind(b0)
                .bind(b1)
                .bind(now0 - 300)
                .fetch_all(&state.pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                .into_iter()
                .collect();
        let mut madde_idler = Vec::new();
        let mut metinler = Vec::new();
        for (id, ozet) in adaylar {
            if kanitli.contains(&id) || dagitilmis.contains(&id) {
                continue;
            }
            madde_idler.push(id);
            metinler.push(ozet.chars().take(4500).collect());
            if madde_idler.len() >= TASK_BATCH as usize {
                break;
            }
        }
        if madde_idler.is_empty() {
            continue;
        }
        let offset = madde_idler[0];
        info!("teklif supurme: #{} [{},{}] -> {} madde", tid, b0, b1, madde_idler.len());
        return Ok(Some((offset, metinler, madde_idler)));
    }
    Ok(None)
}

async fn supurme_dene(
    state: &Arc<AppState>,
    son_id: i64,
) -> Result<Option<(i64, Vec<String>, Vec<i64>)>, (StatusCode, String)> {
    if son_id <= 0 {
        return Ok(None);
    }
    // B21 teklif onceligi: acik teklif araliklarindaki ilk kapsanmamis 20'lik.
    // Teklif yoksa asagidaki normal supurgeye dusulur (davranis aynen).
    if let Some(t) = teklif_supurme_dene(state).await? {
        return Ok(Some(t));
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
    // Tarama penceresi sinirli: her supurme en fazla 5000 id tarar.
    // Sinirsiz tarama buyuk DB'de dagitimi kilitler (2026-09-06 olayi).
    let ust = (sup + 5000).min(son_id);

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
    .bind(ust)
    .bind(now0 - 300)
    .bind(TASK_BATCH)
    .fetch_all(&mut *conn)
    .await;
    // Havuz kirlenmesin: her halukarda DETACH.
    let _ = sqlx::query("DETACH DATABASE wiki").execute(&mut *conn).await;
    let rows = sorgu.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if rows.is_empty() {
        // Bu pencerede bosluk yok: sup'u pencere sonuna ilerlet, normal akisa don.
        // Pencere cursor'a dayandiysa bir sonraki cagri yeni tur baslatir.
        sqlx::query(
            "INSERT INTO gorev_cursor (corpus, son_id) VALUES (?, ?) ON CONFLICT(corpus) DO UPDATE SET son_id = excluded.son_id"
        )
        .bind(&sup_key)
        .bind(ust)
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
    // 17 Eyl B5 dersi: kanarya sentetikleri (madde_id<0) wiki'de ozet'e sahip
    // degildir; secime girerse tum dagitim bosa duser (refs bos -> Ok(None))
    // ve kuyruk basi zehirlenir (16 Eyl 20:33 sonrasi 17 saat denetim OLDU).
    // Sentetikler sizinti izidir, kalite denetimine tabi degildir.
    let mut secilen = sqlx::query(
        "SELECT gorev_id, madde_id, miner_id FROM kanitlar WHERE spot_check = 1 AND dogrulama IS NULL AND madde_id >= 0 AND miner_id != ? AND (son_denetci IS NULL OR son_denetci != ?) ORDER BY ts ASC LIMIT ?"
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
            "SELECT gorev_id, madde_id, miner_id FROM kanitlar WHERE spot_check = 1 AND dogrulama IS NULL AND madde_id >= 0 AND miner_id = ? AND (son_denetci IS NULL OR son_denetci != ?) ORDER BY ts ASC LIMIT ?"
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
            // Ayni corpus state havuzundaysa yeni baglanti acma (B3: testte
            // :memory: wiki calisir; uretimde baglanti curufesi azalir).
            if corpus == state.corpus {
                havuzlar.insert(corpus.clone(), state.wiki.clone());
            } else {
                let wiki_path = format!("/srv/beyin/wiki/wiki_{}.db", corpus);
                let db = SqlitePoolOptions::new()
                    .max_connections(1)
                    .connect(&format!("sqlite:{}?mode=ro", wiki_path))
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("wiki db: {}", e)))?;
                havuzlar.insert(corpus.clone(), db);
            }
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
        // KOR ID (Guvenlik Md.1): denetciye de gercek id gitmez. Bu dagitim
        // basina kor uret, eslemeyi did altina yaz.
        let kor: i64 = loop {
            let k: i64 = rand::thread_rng().gen_range(1..i64::MAX);
            let var: Option<i64> = sqlx::query_scalar("SELECT kor_id FROM kor_esleme WHERE kor_id = ?")
                .bind(k).fetch_optional(&state.pool).await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            if var.is_none() { break k; }
        };
        sqlx::query("INSERT OR IGNORE INTO kor_esleme (kor_id, madde_id, gorev_id, ts) VALUES (?, ?, ?, ?)")
            .bind(kor).bind(mid).bind(&gid)
            .bind(current_epoch()).execute(&state.pool).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        madde_idler.push(kor);
        refs.push(DenetimRef { gorev_id: gid.clone(), madde_id: kor });
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
        corpus: corpus_kodu(&ilk_corpus),
        shard: 0,
        offset: 0,
        limit: n,
        deadline: now + TASK_DEADLINE_SEC,
        payload: GorevPayload { metinler, madde_idler, denetim: Some(refs), yedek: None },
    }))
}

/// Toplu kanit girisi (B14 olcek): bir gorevin kanitlari TEK HTTP istegiyle.
/// Geriye uyumlu EK yol; `/api/kanit` tekil akis aynen durur (eski miner'lar,
/// win .exe). Her kalem mevcut `kanit()` mantigiyla islenir (auth/kara/kor,
/// odul/emanet/kapanis birebir ayni kod): sonuc dizisi doner, kalem hatasi
/// batch'i durdurmaz. Cap: TOPLU_EN_FAZLA (DoS freni).
const TOPLU_EN_FAZLA: usize = 100;

#[derive(Serialize)]
struct TopluKalemResp {
    madde_id: i64,
    kabul: bool,
    hata: Option<String>,
    spot: bool,
    batch_tamam: bool,
    odul_mikro: i64,
}

async fn kanit_toplu(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(istekler): Json<Vec<KanitReq>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if istekler.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "bos toplu kanit".to_string()));
    }
    if istekler.len() > TOPLU_EN_FAZLA {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("toplu kanit cok buyuk ({} > {})", istekler.len(), TOPLU_EN_FAZLA),
        ));
    }
    let mut sonuclar = Vec::with_capacity(istekler.len());
    for req in &istekler {
        let mid = req.madde_id;
        match kanit(State(state.clone()), headers.clone(), Json(req.clone())).await {
            Ok(Json(r)) => sonuclar.push(TopluKalemResp {
                madde_id: mid,
                kabul: true,
                hata: None,
                spot: r.spot_check,
                batch_tamam: r.batch_tamam,
                odul_mikro: r.odul_mikro,
            }),
            Err((kod, mesaj)) => sonuclar.push(TopluKalemResp {
                madde_id: mid,
                kabul: false,
                hata: Some(format!("{}: {}", kod.as_u16(), mesaj)),
                spot: false,
                batch_tamam: false,
                odul_mikro: 0,
            }),
        }
    }
    Ok(Json(serde_json::json!({ "sonuclar": sonuclar })))
}

/// Mesh denetim duyurusu kur (B15): kapanan batch'teki bayrakli kanitlar icin
/// dagitim kor eslemesi + deterministik denetci atamasi. Dagitim YOK (sadece
/// duyuru metni); tasma isini mesh denetciler + hakem (`denetim/sonuc`) gorur.
/// Eski `dagit_denetim` yolu aynen durur (mesh-disi miner'lar, win .exe).
/// Donen None = duyurulacak bayrakli kanit yok.
async fn mesh_duyuru_kur(
    state: &Arc<AppState>,
    gorev_id: &str,
    salt_gun: i64,
    simdi: i64,
) -> anyhow::Result<Option<DenetimDuyuru>> {
    // Bayrakli (emanetteki) kanitlar + dagitim kor eslemesi (tekil: MIN kor).
    let satirlar = sqlx::query(
        "SELECT k.miner_id, k.madde_id, MIN(e.kor_id) AS kor FROM kanitlar k JOIN kor_esleme e ON e.gorev_id = k.gorev_id AND e.madde_id = k.madde_id WHERE k.gorev_id = ? AND k.spot_check = 1 AND k.dogrulama IS NULL AND k.madde_id >= 0 GROUP BY k.miner_id, k.madde_id"
    )
    .bind(gorev_id)
    .fetch_all(&state.pool)
    .await?;
    if satirlar.is_empty() {
        return Ok(None);
    }
    // Aday havuzu: kalp atisi taze madenciler (olu esik alti).
    let adaylar: Vec<String> = sqlx::query_scalar(
        "SELECT miner_id FROM miners WHERE last_seen > ?"
    )
    .bind(simdi - OLU_ESIK_SN)
    .fetch_all(&state.pool)
    .await?;
    let mut atamalar = Vec::new();
    for r in &satirlar {
        let prover: String = r.get("miner_id");
        let kor: i64 = r.get("kor");
        let sec = denetciler(gorev_id, salt_gun, &adaylar, &prover, DENETIM_SAYISI);
        if sec.is_empty() {
            continue;
        }
        atamalar.push(KanitAtama { kor, gorev: gorev_id.to_string(), denetciler: sec });
    }
    if atamalar.is_empty() {
        return Ok(None);
    }
    Ok(Some(DenetimDuyuru { gorev_id: gorev_id.to_string(), salt_gun, atamalar }))
}

/// Mesh denetci metin kapisi (B15): kor ID ile asil metni ver (salt-okunur).
/// Sadece kayitli miner (token). Eski dagitim yolu etkilenmez.
async fn metin(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(kor): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let _miner_id = token_dogrula(&headers, &state.pool).await?;
    let row = sqlx::query("SELECT gorev_id, madde_id FROM kor_esleme WHERE kor_id = ?")
        .bind(kor)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let (gid, mid): (String, i64) = match row {
        Some(r) => (r.get("gorev_id"), r.get("madde_id")),
        None => return Err((StatusCode::NOT_FOUND, "bilinmeyen kor ID".to_string())),
    };
    let corpus = gid.split(':').next().unwrap_or(&state.corpus).to_string();
    let wiki_path = format!("/srv/beyin/wiki/wiki_{}.db", corpus);
    let wiki_db = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite:{}?mode=ro", wiki_path))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("wiki db: {}", e)))?;
    let srow = sqlx::query("SELECT ozet FROM madde WHERE id = ?")
        .bind(mid)
        .fetch_optional(&wiki_db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let ozet: String = match srow {
        Some(x) => x.get("ozet"),
        None => return Err((StatusCode::NOT_FOUND, "madde yok".to_string())),
    };
    let metin: String = ozet.chars().take(4500).collect();
    Ok(Json(serde_json::json!({ "gorev_id": gid, "madde_id": mid, "metin": metin })))
}

async fn kanit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<KanitReq>,
) -> Result<Json<KanitResp>, (StatusCode, String)> {
    let miner_id = token_dogrula(&headers, &state.pool).await?;

    // Kara liste (Guvenlik Md.4): imha edilen kanit da veremez.
    let kara: Option<String> = sqlx::query_scalar("SELECT neden FROM kara_liste WHERE miner_id = ?")
        .bind(&miner_id).fetch_optional(&state.pool).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if kara.is_some() {
        return Err((StatusCode::FORBIDDEN, "kara liste (operator affeti gerekli)".to_string()));
    }

    // Denetim gorevlerine kanit gonderilmez (onlar denetim/sonuc'a gider).
    if req.gorev_id.starts_with("denetim:") {
        return Err((StatusCode::BAD_REQUEST, "Denetim gorevlerine kanit gonderilmez".to_string()));
    }

    // KOR ID cozumu (Guvenlik Md.1): miner kor ID gonderir, gercek ID
    // eslemeden cozulur. Esleme yoksa dagitilmamis/harici ID'dir -> red.
    // Asagidaki tum akis GERCEK id ile surer (kanitlar/supurme ic izler).
    let madde_gercek: i64 = sqlx::query_scalar(
        "SELECT madde_id FROM kor_esleme WHERE kor_id = ? AND gorev_id = ?"
    )
    .bind(req.madde_id)
    .bind(&req.gorev_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::BAD_REQUEST, "Bilinmeyen kor ID (dagitilmamis gorev)".to_string()))?;

    let exists = sqlx::query("SELECT 1 FROM kanitlar WHERE gorev_id = ? AND madde_id = ?")
        .bind(&req.gorev_id)
        .bind(madde_gercek)
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
    // 17 Eyl B5: kanarya sentetikleri (madde_id<0) spot kuyruguna girmez.
    // Gerekce: wiki'de ozetleri yoktur, denetlenemezler; secime girip kuyruk
    // basini zehirliyorlardi. Odeme/kanit akisi aynen surer, sadece bayrak yok.
    let spot = madde_gercek >= 0 && spot_check_gerekli(&state.pool, &req.gorev_id, madde_gercek).await;

    // Kaniti kaydet (anlik odul 0; odul batch tamamlaninca dagitilir).
    // Ic izlerde GERCEK id (supurme/denetim tutarliligi icin).
    sqlx::query(
        "INSERT INTO kanitlar (gorev_id, madde_id, miner_id, v_int8_b64, v_min, v_max, odul_mikro, spot_check, ts) VALUES (?, ?, ?, ?, ?, ?, 0, ?, ?)"
    )
    .bind(&req.gorev_id)
    .bind(madde_gercek)
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
        info!("Spot-check bayragi: miner={} gorev={} kor={}", miner_id, req.gorev_id, req.madde_id);
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

    // B21 bulucu payi: bu gorev acik teklifteyse (en eski) %1'i bastan ayir.
    // Basim yok: batch odulunden yonlendirme (korunum: odenen+emanet+ucret=odul).
    let mut teklif_bilgi: Option<(i64, String, i64, i64)> = None;
    let mut batch_odul = batch_odul;
    {
        if let Some(grow2) = sqlx::query("SELECT corpus, offset FROM gorevler WHERE gorev_id = ?")
            .bind(&req.gorev_id).fetch_optional(&state.pool).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        {
            let gcorpus: String = grow2.get("corpus");
            let goffset: i64 = grow2.get("offset");
            if let Some(trow) = sqlx::query("SELECT id, miner_id, baslangic, bitis FROM teklifler WHERE durum = 'acik' AND corpus = ? AND baslangic <= ? AND bitis >= ? ORDER BY ts ASC LIMIT 1")
                .bind(&gcorpus).bind(goffset).bind(goffset).fetch_optional(&state.pool).await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            {
                let tid: i64 = trow.get("id");
                let bulucu: String = trow.get("miner_id");
                let tb0: i64 = trow.get("baslangic");
                let tb1: i64 = trow.get("bitis");
                let ucret = batch_odul / 100;
                if ucret > 0 {
                    sqlx::query("UPDATE miners SET coin_mikro = coin_mikro + ? WHERE miner_id = ?")
                        .bind(ucret).bind(&bulucu).execute(&state.pool).await
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                    sqlx::query("INSERT INTO ledger (miner_id, delta_mikro, neden, epoch, ts) VALUES (?, ?, 'teklif-odul', ?, ?)")
                        .bind(&bulucu).bind(ucret).bind(now).bind(now).execute(&state.pool).await
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                    batch_odul -= ucret;
                    info!("teklif odulu: #{} <- {} mikro ({})", tid, ucret, bulucu);
                }
                teklif_bilgi = Some((tid, bulucu, tb0, tb1));
            }
        }
    }

    // B-1: dagitimda KALAN (dogrulanmamis-basarisiz) kanitlar pay almaz;
    // supheli (spot+denetimsiz) kanitlarin payi emanete (escrow) yazilir.
    // Kapanis sayaci (sayi) aynen tum kanitlari sayar (liveness korunur).
    let satirlar: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
        "SELECT miner_id, madde_id, (spot_check = 1 AND dogrulama IS NULL) AS emanet FROM kanitlar WHERE gorev_id = ? AND (dogrulama IS NULL OR dogrulama >= ?) ORDER BY miner_id, madde_id"
    )
    .bind(&req.gorev_id)
    .bind(DENETIM_ESIK)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut dagitilan: i64 = 0;
    let mut benim_payim: i64 = 0;
    let n_satir = satirlar.len() as i64;
    for (i, r) in satirlar.iter().enumerate() {
        let mid: String = r.get("miner_id");
        let mid_kanit: i64 = r.get("madde_id");
        let emanet: i64 = r.get("emanet");
        // Son satira kalan bakiyeyi ver (kusurat kaybi olmasin).
        let pay_i = if i as i64 == n_satir - 1 {
            batch_odul - dagitilan
        } else {
            batch_odul / beklenen
        };
        if pay_i <= 0 {
            continue;
        }
        if emanet == 1 {
            sqlx::query(
                "INSERT OR IGNORE INTO escrow (gorev_id, madde_id, miner_id, miktar_mikro, ts) VALUES (?, ?, ?, ?, ?)"
            )
            .bind(&req.gorev_id)
            .bind(mid_kanit)
            .bind(&mid)
            .bind(pay_i)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        } else {
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
        if mid == miner_id && emanet == 0 {
            benim_payim += pay_i;
        }
    }

    // B21 tamamlanma: kapsama bittiyse teklifi kapat + stake iade.
    if let Some((tid, bulucu, tb0, tb1)) = teklif_bilgi {
        if let Ok((kapali, genislik)) = aralik_kapsama(&state.pool, tb0, tb1).await {
            if kapali as f64 / genislik.max(1) as f64 >= TEKLIF_KAPSAMA_BITIS {
                sqlx::query("UPDATE teklifler SET durum = 'tamam', kapanma_ts = ? WHERE id = ? AND durum = 'acik'")
                    .bind(now).bind(tid).execute(&state.pool).await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                sqlx::query("UPDATE miners SET coin_mikro = coin_mikro + ? WHERE miner_id = ?")
                    .bind(TEKLIF_STAKE_MIKRO).bind(&bulucu).execute(&state.pool).await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                sqlx::query("INSERT INTO ledger (miner_id, delta_mikro, neden, epoch, ts) VALUES (?, ?, 'teklif-iade', ?, ?)")
                    .bind(&bulucu).bind(TEKLIF_STAKE_MIKRO).bind(now).bind(now).execute(&state.pool).await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                info!("teklif kapandi: #{} (iade {})", tid, bulucu);
            }
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

    // B25 sigorta besleme: kapanis hiz olcume girer (esikte TEK uyari).
    if let Ok(mut kesici) = KESICI.lock() {
        let once = kesici.kesik_son;
        kesici.kapanis_kaydet(now);
        if kesici.kesik_son > now && once <= now {
            warn!("DEVRE KESIK: anormal kapanis hizi (dagitim 30dk duraklatildi)");
        }
    }

    // B15 mesh duyurusu: bayrakli kanitlar icin atama yayinla.
    // Hata/eksik yoksayilir (eski dagit_denetim yolu aynen calisir).
    {
        let salt_gun = now / 86400;
        match mesh_duyuru_kur(&state, &req.gorev_id, salt_gun, now).await {
            Ok(Some(duyuru)) => {
                let n = duyuru.atamalar.len();
                if let Ok(raw) = serde_json::to_vec(&duyuru) {
                    if let Some(tx) = state.gorev_yayin_tx.as_ref() {
                        let _ = tx.send((DENETIM_TOPIC.to_string(), raw));
                    }
                    info!("mesh denetim duyurusu: {} ({} atama)", req.gorev_id, n);
                }
            }
            Ok(None) => {}
            Err(e) => warn!("mesh duyuru kurulamadi {}: {}", req.gorev_id, e),
        }
    }

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

/// Filo gorunumu (B23): madenciler + kabiliyet + canlilik (operator planlamasi).
/// `?cuzdan=` filtresi (B25 ciftlik paketi): ciftlik filosunu listeler.
async fn filo(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(filtre): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let _miner_id = token_dogrula(&headers, &state.pool).await?;
    let now = current_epoch();
    let rows = if let Some(cuzdan) = filtre.get("cuzdan") {
        sqlx::query(
            "SELECT m.miner_id, m.pay, m.coin_mikro, m.strike, m.itibar, m.last_seen, m.depolama_kota, y.gpu_ad, y.vram_mb, y.roller FROM miners m LEFT JOIN miner_yetenek y ON y.miner_id = m.miner_id WHERE m.cuzdan = ? ORDER BY m.pay DESC LIMIT 10000",
        )
        .bind(cuzdan)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        sqlx::query(
            "SELECT m.miner_id, m.pay, m.coin_mikro, m.strike, m.itibar, m.last_seen, m.depolama_kota, y.gpu_ad, y.vram_mb, y.roller FROM miners m LEFT JOIN miner_yetenek y ON y.miner_id = m.miner_id ORDER BY m.pay DESC LIMIT 10000",
        )
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };
    let liste: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let mid: String = r.get("miner_id");
            let pay: i64 = r.get("pay");
            let coin: i64 = r.get("coin_mikro");
            let strike: i64 = r.get("strike");
            let itibar: i64 = r.get("itibar");
            let goruldu: i64 = r.get("last_seen");
            let kota: i64 = r.get("depolama_kota");
            let gpu: Option<String> = r.get("gpu_ad");
            let vram: Option<i64> = r.get("vram_mb");
            let roller: Option<String> = r.get("roller");
            serde_json::json!({
                "miner_id": mid, "pay": pay, "coin_mikro": coin,
                "strike": strike, "itibar": itibar,
                "son_nabiz_sn_once": (now - goruldu).max(0),
                "depolama_kota": kota,
                "gpu": gpu.unwrap_or_default(), "vram_mb": vram.unwrap_or(0),
                "roller": roller.unwrap_or_default(),
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "madenci": liste, "sayi": liste.len() })))
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

/// Guvenlik Md.3: supheli metin kanarya mi? Operator destekli sizinti taramasi.
/// Token'li herkes sorabilir ama kimlik DÖNMEZ (dagitim gecmisi operatorde/DB'de).
/// Kullanim: disarida gorulen metin buraya yapistirilir, eslesme + sayi doner.
/// Eslesme + sayi >= 2 dagitim ve dis kaynak = imha delili (komut/imha ile).
async fn kanarya_kontrol(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let _miner_id = token_dogrula(&headers, &state.pool).await?;
    let metin = q.get("metin").map(|s| s.trim().to_string()).unwrap_or_default();
    if metin.len() < 20 {
        return Err((StatusCode::BAD_REQUEST, "metin cok kisa (min 20)".to_string()));
    }
    let kid: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM kanaryalar WHERE aktif = 1 AND instr(?, metin) > 0"
    )
    .bind(&metin).fetch_optional(&state.pool).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    // Ters yon de dene (supheli parca kanaryanin parcasiysa).
    let kid = match kid {
        Some(k) => Some(k),
        None => sqlx::query_scalar(
            "SELECT id FROM kanaryalar WHERE aktif = 1 AND instr(metin, ?) > 0"
        )
        .bind(&metin).fetch_optional(&state.pool).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
    };
    let dagitim_sayisi: i64 = match kid {
        Some(k) => sqlx::query_scalar("SELECT COUNT(*) FROM kanarya_dagitim WHERE kanarya_id = ?")
            .bind(k).fetch_one(&state.pool).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
        None => 0,
    };
    Ok(Json(serde_json::json!({"eslesti": kid.is_some(), "dagitim_sayisi": dagitim_sayisi})))
}

async fn denetim_sonuc(    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<DenetimSonuc>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let denetci = token_dogrula(&headers, &state.pool).await?;
    let now = current_epoch();

    // KOR ID cozumu (Guvenlik Md.1): denetci kor ID gonderir.
    let madde_gercek: i64 = sqlx::query_scalar(
        "SELECT madde_id FROM kor_esleme WHERE kor_id = ? AND gorev_id = ?"
    )
    .bind(req.madde_id)
    .bind(&req.gorev_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::BAD_REQUEST, "Bilinmeyen kor ID (dagitilmamis denetim)".to_string()))?;

    // Orijinal kaniti bul (denetim bayrakli olmali).
    let orow = sqlx::query(
        "SELECT miner_id, v_int8_b64, v_min, v_max, dogrulama, ret FROM kanitlar WHERE gorev_id = ? AND madde_id = ?"
    )
    .bind(&req.gorev_id)
    .bind(madde_gercek)
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
        if state.strict_denetim {
            return Err((StatusCode::FORBIDDEN, "Oz-denetim yasak (STRICT_DENETIM)".to_string()));
        }
        warn!("Oz-denetim (testnet toleransli): denetci={} kendi kanitini denetliyor gorev={} kor={}", denetci, req.gorev_id, req.madde_id);
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
            // B-3: istemci-hesapli skor kabul edilmez (emeksiz denetim).
            // Denetci taze vektor gondermek zorunda; kosinusu komuta hesaplar.
            return Err((StatusCode::BAD_REQUEST, "taze denetim vektoru gerekli (v_int8_b64+v_min+v_max)".to_string()));
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
        .bind(madde_gercek)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        if r.rows_affected() == 0 {
            return Err((StatusCode::CONFLICT, "Bu kayit araya denetlenmis".to_string()));
        }
        // B-1: bu kanitin emanetteki payi varsa serbest birak (batch kapandiysa yazilmistir).
        let emanetler: Vec<sqlx::sqlite::SqliteRow> = sqlx::query(
            "SELECT miner_id, miktar_mikro FROM escrow WHERE gorev_id = ? AND madde_id = ?"
        )
        .bind(&req.gorev_id)
        .bind(madde_gercek)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        for erow in emanetler {
            let emid: String = erow.get("miner_id");
            let emik: i64 = erow.get("miktar_mikro");
            sqlx::query("UPDATE miners SET coin_mikro = coin_mikro + ? WHERE miner_id = ?")
                .bind(emik)
                .bind(&emid)
                .execute(&state.pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query(
                "INSERT INTO ledger (miner_id, delta_mikro, neden, epoch, ts) VALUES (?, ?, 'escrow', ?, ?)"
            )
            .bind(&emid)
            .bind(emik)
            .bind(now)
            .bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            sqlx::query("DELETE FROM escrow WHERE gorev_id = ? AND madde_id = ?")
                .bind(&req.gorev_id)
                .bind(madde_gercek)
                .execute(&state.pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            info!("Escrow serbest: miner={} gorev={} madde={} miktar={}", emid, req.gorev_id, madde_gercek, emik);
        }
        sqlx::query("UPDATE miners SET itibar = CASE WHEN itibar + ? > 100 THEN 100 ELSE itibar + ? END WHERE miner_id = ?")
            .bind(ITIBAR_ODUL)
            .bind(ITIBAR_ODUL)
            .bind(&ureten)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        if ret_onceki >= 1 {
            info!("Suphe GIDERILDI: denetci={} gorev={} kor={} cos={:.4} (onceki ret sayisi={})", denetci, req.gorev_id, req.madde_id, cos, ret_onceki);
        } else {
            info!("Denetim gecti: denetci={} gorev={} kor={} cos={:.4}", denetci, req.gorev_id, req.madde_id, cos);
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
        .bind(madde_gercek)
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
        // B-1: hile kesinlesen kanitin emanetteki payi yanar (odenmemisti, ledger'e dokunulmaz).
        sqlx::query("DELETE FROM escrow WHERE gorev_id = ? AND madde_id = ?")
            .bind(&req.gorev_id)
            .bind(madde_gercek)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let strike: i64 = sqlx::query_scalar("SELECT strike FROM miners WHERE miner_id = ?")
            .bind(&ureten)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        warn!("SLASH: ureten={} gorev={} kor={} cos={:.4} kesinti={} strike={} (denetciler: onceki+{})", ureten, req.gorev_id, req.madde_id, cos, kesinti, strike, denetci);
        return Ok(Json(serde_json::json!({"ok": true, "gecerli": false, "dogrulama": cos, "supheli": false, "slash": kesinti, "strike": strike})));
    }

    // Ilk ret: supheli, kuyruga iade (ikinci bagimsiz denetciye gider).
    sqlx::query(
        "UPDATE kanitlar SET ret = ret + 1, son_denetci = ? WHERE gorev_id = ? AND madde_id = ? AND dogrulama IS NULL"
    )
    .bind(&denetci)
    .bind(&req.gorev_id)
    .bind(madde_gercek)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    warn!("SUPHELI (1. ret, slash yok): denetci={} ureten={} gorev={} kor={} cos={:.4} -> ikinci denetciye iade", denetci, ureten, req.gorev_id, req.madde_id, cos);
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

/// --- C7: onarım/çoğaltma ---
/// Relay dosya yolu (64-hex ad zorunlu, dizin dışına çıkılmaz).
fn relay_yolu(state: &Arc<AppState>, hash: &str) -> Option<std::path::PathBuf> {
    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(std::path::PathBuf::from(&state.relay_dir).join(hash))
}

#[derive(Serialize)]
struct YedekParca {
    parca_hash: String,
    boyut: i64,
    onarim_id: i64,
}

/// Kaynak dosyayı (SADECE yedek dizinindeki komuta-*.db) parçala,
/// relay dizinine yaz, kataloğa işle. Dönen: (hash, boyut) listesi.
async fn tohumla(state: &Arc<AppState>, dosya: &str) -> Result<Vec<(String, i64)>, String> {
    use tokio::io::AsyncReadExt;
    if !dosya.starts_with("komuta-") || !dosya.ends_with(".db") || dosya.contains('/') || dosya.contains("..") {
        return Err("yalnizca yedek dizinindeki komuta-*.db tohumlanabilir".to_string());
    }
    let kaynak = std::path::PathBuf::from(&state.yedek_dir).join(dosya);
    let meta = tokio::fs::metadata(&kaynak).await.map_err(|e| format!("kaynak okunamadi: {}", e))?;
    if !meta.is_file() || meta.len() == 0 {
        return Err("kaynak dosya gecersiz".to_string());
    }
    tokio::fs::create_dir_all(&state.relay_dir).await.map_err(|e| e.to_string())?;
    let mut f = tokio::fs::File::open(&kaynak).await.map_err(|e| e.to_string())?;
    let now = current_epoch();
    let mut cikti = Vec::new();
    let mut tampon = vec![0u8; 4 * 1024 * 1024];
    let mut aktif: Option<(blake3::Hasher, tokio::fs::File, u64)> = None;
    let mut parca_say = 0i64;
    loop {
        let n = f.read(&mut tampon).await.map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        let mut dilim = &tampon[..n];
        while !dilim.is_empty() {
            if aktif.is_none() {
                let gecici = format!("{}/.yaziliyor-{}", state.relay_dir, parca_say);
                let wf = tokio::fs::File::create(&gecici).await.map_err(|e| e.to_string())?;
                aktif = Some((blake3::Hasher::new(), wf, 0));
            }
            let (h, wf, yazilan) = aktif.as_mut().unwrap();
            let oda = (PARCA_BOYUT - *yazilan).min(dilim.len() as u64) as usize;
            use tokio::io::AsyncWriteExt;
            wf.write_all(&dilim[..oda]).await.map_err(|e| e.to_string())?;
            h.update(&dilim[..oda]);
            *yazilan += oda as u64;
            dilim = &dilim[oda..];
            if *yazilan >= PARCA_BOYUT {
                let (h, mut wf, boyut) = aktif.take().unwrap();
                wf.shutdown().await.map_err(|e| e.to_string())?;
                let hash = h.finalize().to_hex().to_string();
                let hedef = format!("{}/{}", state.relay_dir, hash);
                tokio::fs::rename(format!("{}/.yaziliyor-{}", state.relay_dir, parca_say), &hedef)
                    .await.map_err(|e| e.to_string())?;
                sqlx::query("INSERT OR IGNORE INTO parcalar (parca_hash, boyut, ts) VALUES (?, ?, ?)")
                    .bind(&hash).bind(boyut as i64).bind(now)
                    .execute(&state.pool).await.map_err(|e| e.to_string())?;
                cikti.push((hash, boyut as i64));
                parca_say += 1;
            }
        }
    }
    if let Some((h, mut wf, boyut)) = aktif.take() {
        if boyut > 0 {
            use tokio::io::AsyncWriteExt;
            wf.shutdown().await.map_err(|e| e.to_string())?;
            let hash = h.finalize().to_hex().to_string();
            let hedef = format!("{}/{}", state.relay_dir, hash);
            tokio::fs::rename(format!("{}/.yaziliyor-{}", state.relay_dir, parca_say), &hedef)
                .await.map_err(|e| e.to_string())?;
            sqlx::query("INSERT OR IGNORE INTO parcalar (parca_hash, boyut, ts) VALUES (?, ?, ?)")
                .bind(&hash).bind(boyut as i64).bind(now)
                .execute(&state.pool).await.map_err(|e| e.to_string())?;
            cikti.push((hash, boyut as i64));
        }
    }
    info!("tohumlandi: {} ({} parca)", dosya, cikti.len());
    Ok(cikti)
}

/// Onarım planla: canlı kopyası hedefin altındaki parçalardan hedefe atanmamışları seç.
/// Kritik bölüm kilidi ÇAĞIRANDA tutulur (dagitim_kilidi) — burada kilitlenmez!
async fn onarim_planla(
    state: &Arc<AppState>,
    hedef: &str,
    now: i64,
) -> Result<Vec<(String, i64, i64)>, (StatusCode, String)> {
    // Kota yoksa onarım görevi de yok.
    let kota: i64 = sqlx::query_scalar("SELECT COALESCE(depolama_kota, 0) FROM miners WHERE miner_id = ?")
        .bind(hedef)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if kota <= 0 {
        return Ok(Vec::new());
    }
    let satirlar = sqlx::query(
        "SELECT p.parca_hash, p.boyut, COUNT(DISTINCT CASE WHEN m.last_seen > ? THEN y.miner_id END) AS canli
         FROM parcalar p
         LEFT JOIN parca_yerleri y ON y.parca_hash = p.parca_hash
         LEFT JOIN miners m ON m.miner_id = y.miner_id
         WHERE NOT EXISTS (SELECT 1 FROM parca_yerleri h WHERE h.parca_hash = p.parca_hash AND h.miner_id = ?)
           AND NOT EXISTS (SELECT 1 FROM onarimlar o WHERE o.parca_hash = p.parca_hash AND o.hedef_miner = ? AND o.durum = 'acik')
         GROUP BY p.parca_hash HAVING canli < ?
         ORDER BY canli ASC, p.parca_hash ASC LIMIT ?"
    )
    .bind(now - OLU_ESIK_SN)
    .bind(hedef)
    .bind(hedef)
    .bind(REPLIKA_HEDEF)
    .bind(YEDEKLE_BATCH)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mut cikti = Vec::new();
    for r in satirlar {
        let h: String = r.get("parca_hash");
        let b: i64 = r.get("boyut");
        let res = sqlx::query("INSERT INTO onarimlar (parca_hash, hedef_miner, durum, ts) VALUES (?, ?, 'acik', ?)")
            .bind(&h).bind(hedef).bind(now)
            .execute(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        cikti.push((h, b, res.last_insert_rowid()));
    }
    if !cikti.is_empty() {
        info!("onarim planlandi: {} parca -> {} ", cikti.len(), hedef);
    }
    Ok(cikti)
}

/// POST /api/parca/tohum {"dosya": "komuta-2026-09-06-0018.db"}
async fn parca_tohum(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let miner_id = token_dogrula(&headers, &state.pool).await?;
    let dosya = req.get("dosya").and_then(|v| v.as_str()).ok_or((StatusCode::BAD_REQUEST, "dosya gerekli".to_string()))?;
    match tohumla(&state, dosya).await {
        Ok(parcalar) => {
            info!("tohum istendi: {} <- {} ({} parca)", dosya, miner_id, parcalar.len());
            let liste: Vec<serde_json::Value> = parcalar
                .iter()
                .map(|(h, b)| serde_json::json!({"parca_hash": h, "boyut": b}))
                .collect();
            let n = liste.len();
            Ok(Json(serde_json::json!({"ok": true, "parca_sayisi": n, "parcalar": liste})))
        }
        Err(m) => Err((StatusCode::BAD_REQUEST, m)),
    }
}

/// GET /api/parca/indir/:hash — relay dosyasını akıtır.
async fn parca_indir(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(hash): axum::extract::Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let _ = token_dogrula(&headers, &state.pool).await?;
    let yol = relay_yolu(&state, hash.trim()).ok_or((StatusCode::BAD_REQUEST, "gecersiz hash".to_string()))?;
    let veri = tokio::fs::read(&yol).await.map_err(|_| (StatusCode::NOT_FOUND, "parca relay'de yok".to_string()))?;
    use axum::response::Response;
    Ok(Response::builder()
        .header("content-type", "application/octet-stream")
        .header("content-length", veri.len().to_string())
        .body(axum::body::Body::from(veri))
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "yanit kurulamadi".to_string()))?)
}

/// --- C8: rastgele parca yoklamasi (proof-of-storage-lite) ---
/// Aday sec (kotasi olan, acik yoklamasi az olan tutucu) -> relay dosyasindan
/// rastgele 1 MiB ornekle -> hash'le -> yoklamalar'a yaz. Donen: yoklama id.
async fn yoklama_uret_bir(pool: &SqlitePool, relay_dir: &str, miner_filtre: Option<&str>) -> Result<Option<i64>, String> {
    let aday = if let Some(m) = miner_filtre {
        sqlx::query(
            "SELECT y.miner_id, y.parca_hash FROM parca_yerleri y JOIN miners m ON m.miner_id = y.miner_id
             WHERE y.miner_id = ? AND COALESCE(m.depolama_kota, 0) > 0
               AND (SELECT COUNT(*) FROM yoklamalar o WHERE o.miner_id = y.miner_id AND o.durum = 'acik') < ?
             ORDER BY RANDOM() LIMIT 1"
        )
        .bind(m)
        .bind(YOKLAMA_ACIK_LIMIT)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query(
            "SELECT y.miner_id, y.parca_hash FROM parca_yerleri y JOIN miners m ON m.miner_id = y.miner_id
             WHERE COALESCE(m.depolama_kota, 0) > 0
               AND (SELECT COUNT(*) FROM yoklamalar o WHERE o.miner_id = y.miner_id AND o.durum = 'acik') < ?
             ORDER BY RANDOM() LIMIT 1"
        )
        .bind(YOKLAMA_ACIK_LIMIT)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
    };
    let (mid, hash): (String, String) = match aday {
        Some(r) => (r.get("miner_id"), r.get("parca_hash")),
        None => return Ok(None),
    };
    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("katalogda bozuk hash".to_string());
    }
    let veri = tokio::fs::read(format!("{}/{}", relay_dir.trim_end_matches('/'), hash))
        .await
        .map_err(|_| "parca relay'de yok (tohum kayip?)".to_string())?;
    if veri.is_empty() {
        return Err("relay dosyasi bos".to_string());
    }
    let ornek = YOKLAMA_BOYUT.min(veri.len() as u64) as usize;
    let basla = if veri.len() > ornek {
        rand::thread_rng().gen_range(0..=(veri.len() - ornek))
    } else {
        0
    };
    let beklenen = blake3::hash(&veri[basla..basla + ornek]).to_hex().to_string();
    let now = current_epoch();
    let res = sqlx::query(
        "INSERT INTO yoklamalar (miner_id, parca_hash, offset, uzunluk, beklenen_hash, durum, ts) VALUES (?, ?, ?, ?, ?, 'acik', ?)"
    )
    .bind(&mid)
    .bind(&hash)
    .bind(basla as i64)
    .bind(ornek as i64)
    .bind(&beklenen)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    info!("yoklama uretildi: {} <- {} (offset {}, {} bayt)", &hash[..12], mid, basla, ornek);
    Ok(Some(res.last_insert_rowid()))
}

/// Son 2 sonucu pes pese fail olanin kotasini sifirla (dusurme).
/// Donen: dusuruldu mu?
async fn dusurme_kontrol(pool: &SqlitePool, miner_id: &str) -> Result<bool, String> {
    let sonuclar: Vec<String> = sqlx::query_scalar(
        "SELECT durum FROM yoklamalar WHERE miner_id = ? AND durum != 'acik' ORDER BY id DESC LIMIT 2"
    )
    .bind(miner_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    if sonuclar.len() == 2 && sonuclar.iter().all(|d| d == "kaldi" || d == "sure-doldu") {
        sqlx::query("UPDATE miners SET depolama_kota = 0 WHERE miner_id = ?")
            .bind(miner_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        warn!("DUSURME: {} depolama katmanindan cikarildi (2 pes pese fail)", miner_id);
        return Ok(true);
    }
    Ok(false)
}

#[derive(Serialize)]
struct YoklamaKaydi {
    id: i64,
    parca_hash: String,
    offset: i64,
    uzunluk: i64,
}

/// GET /api/depolama/yoklama — cagiranin en eski acik yoklamasi (yoksa 204).
async fn yoklama_al(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let miner_id = token_dogrula(&headers, &state.pool).await?;
    let now = current_epoch();
    let row = sqlx::query(
        "SELECT id, parca_hash, offset, uzunluk FROM yoklamalar WHERE miner_id = ? AND durum = 'acik' AND ts > ? ORDER BY ts ASC LIMIT 1"
    )
    .bind(&miner_id)
    .bind(now - YOKLAMA_SURE_SN)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    match row {
        None => Ok(StatusCode::NO_CONTENT.into_response()),
        Some(r) => Ok(Json(YoklamaKaydi {
            id: r.get("id"),
            parca_hash: r.get("parca_hash"),
            offset: r.get("offset"),
            uzunluk: r.get("uzunluk"),
        }).into_response()),
    }
}

#[derive(Deserialize)]
struct YoklamaSonuc {
    id: i64,
    hash: String,
}

/// POST /api/depolama/yoklama/sonuc {"id", "hash"} — karsilastir, kapat, gerekirse dusur.
async fn yoklama_sonuc(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<YoklamaSonuc>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let miner_id = token_dogrula(&headers, &state.pool).await?;
    let row = sqlx::query(
        "SELECT parca_hash, beklenen_hash FROM yoklamalar WHERE id = ? AND miner_id = ? AND durum = 'acik'"
    )
    .bind(req.id)
    .bind(&miner_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let row = row.ok_or((StatusCode::NOT_FOUND, "acik yoklama bulunamadi".to_string()))?;
    let beklenen: String = row.get("beklenen_hash");
    let gecerli = req.hash.trim().eq_ignore_ascii_case(beklenen.trim());
    let durum = if gecerli { "gecti" } else { "kaldi" };
    sqlx::query("UPDATE yoklamalar SET durum = ? WHERE id = ?")
        .bind(durum)
        .bind(req.id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if gecerli {
        info!("yoklama gecti: {} #{} ", miner_id, req.id);
        return Ok(Json(serde_json::json!({"ok": true, "gecerli": true})));
    }
    warn!("yoklama KALDI: {} #{} ", miner_id, req.id);
    let dusuruldu = dusurme_kontrol(&state.pool, &miner_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({"ok": true, "gecerli": false, "dusuruldu": dusuruldu})))
}

/// POST /api/depolama/yoklama/uret {"miner_id": "...", "parca_hash": "..."} (hepsi opsiyonel).
/// Test/ops tetikleyici + hedefli yeniden denetim.
async fn yoklama_uret_endpoint(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let _ = token_dogrula(&headers, &state.pool).await?;
    // Hedefli uretim: supheli parcayi ozellikle denetle.
    if let Some(hedef_hash) = req.get("parca_hash").and_then(|v| v.as_str()) {
        let h = hedef_hash.trim();
        if h.len() != 64 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err((StatusCode::BAD_REQUEST, "gecersiz parca_hash".to_string()));
        }
        let tutucu: Option<String> = sqlx::query_scalar(
            "SELECT y.miner_id FROM parca_yerleri y JOIN miners m ON m.miner_id = y.miner_id
             WHERE y.parca_hash = ? AND COALESCE(m.depolama_kota, 0) > 0
               AND (SELECT COUNT(*) FROM yoklamalar o WHERE o.miner_id = y.miner_id AND o.durum = 'acik') < ?
             ORDER BY RANDOM() LIMIT 1"
        )
        .bind(h)
        .bind(YOKLAMA_ACIK_LIMIT)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let mid = tutucu.ok_or((StatusCode::NOT_FOUND, "parcayi tutan uygun miner yok".to_string()))?;
        let veri = tokio::fs::read(format!("{}/{}", state.relay_dir.trim_end_matches('/'), h))
            .await
            .map_err(|_| (StatusCode::NOT_FOUND, "parca relay'de yok (tohum kayip?)".to_string()))?;
        if veri.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "relay dosyasi bos".to_string()));
        }
        let ornek = YOKLAMA_BOYUT.min(veri.len() as u64) as usize;
        let basla = if veri.len() > ornek {
            rand::thread_rng().gen_range(0..=(veri.len() - ornek))
        } else {
            0
        };
        let beklenen = blake3::hash(&veri[basla..basla + ornek]).to_hex().to_string();
        let now = current_epoch();
        let res = sqlx::query(
            "INSERT INTO yoklamalar (miner_id, parca_hash, offset, uzunluk, beklenen_hash, durum, ts) VALUES (?, ?, ?, ?, ?, 'acik', ?)"
        )
        .bind(&mid).bind(h).bind(basla as i64).bind(ornek as i64).bind(&beklenen).bind(now)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        info!("hedefli yoklama: {} <- {} (offset {})", &h[..12], mid, basla);
        return Ok(Json(serde_json::json!({"ok": true, "yoklama_id": res.last_insert_rowid()})));
    }
    let filtre = req.get("miner_id").and_then(|v| v.as_str());
    match yoklama_uret_bir(&state.pool, &state.relay_dir, filtre).await {
        Ok(Some(id)) => Ok(Json(serde_json::json!({"ok": true, "yoklama_id": id}))),
        Ok(None) => Ok(Json(serde_json::json!({"ok": true, "yoklama_id": null, "not": "uygun aday yok"}))),
        Err(m) => Err((StatusCode::BAD_REQUEST, m)),
    }
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
                // Operator affi: strike sifirla, itibari yuzle, kara listeden cikar.
                let mid = payload_json.get("miner_id").and_then(|v| v.as_str()).ok_or((StatusCode::BAD_REQUEST, "miner_id gerekli".to_string()))?;
                sqlx::query("UPDATE miners SET strike = 0, itibar = 100 WHERE miner_id = ?")
                    .bind(mid)
                    .execute(&state.pool)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                sqlx::query("DELETE FROM kara_liste WHERE miner_id = ?")
                    .bind(mid)
                    .execute(&state.pool)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                info!("AFFET komutu: {} strike sifirlandi, itibar 100, kara liste temiz", mid);
            }
            "imha" => {
                // Guvenlik Md.4: kanitli hainlikte imha — ban + pay/coin sifirlama.
                // Sadece master imzasiyla (bu handler'in giris kosulu). Fiş cekene degil.
                let mid = payload_json.get("miner_id").and_then(|v| v.as_str()).ok_or((StatusCode::BAD_REQUEST, "miner_id gerekli".to_string()))?;
                let neden = payload_json.get("neden").and_then(|v| v.as_str()).unwrap_or("kanitli sizinti");
                sqlx::query("INSERT OR REPLACE INTO kara_liste (miner_id, neden, ts) VALUES (?, ?, ?)")
                    .bind(mid).bind(neden).bind(current_epoch())
                    .execute(&state.pool)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                sqlx::query("UPDATE miners SET strike = 3, itibar = 0, pay = 0, coin_mikro = 0 WHERE miner_id = ?")
                    .bind(mid)
                    .execute(&state.pool)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                warn!("IMHA komutu: {} banlandi, pay sifirlandi (neden: {})", mid, neden);
            }
            _ => {
                info!("Bilinmeyen komut tipi: {}", tip);
            }
        }
    }

    Ok(Json(serde_json::json!({"ok": true, "epoch": epoch})))
}

// --- Main ---

/// HTTP yonlendirme tablosu (B3: entegrasyon testleri canliyla ayni tabloyu kullanir).
fn app_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/kayit", post(register))
        .route("/api/kayit/toplu", post(register_toplu))
        .route("/api/heartbeat", post(heartbeat))
        .route("/api/gorev", get(gorev))
        .route("/api/kira/al", post(kira_al))
        .route("/api/teklif", post(teklif_ver))
        .route("/api/bosluklar", get(bosluklar))
        .route("/api/tekliflerim", get(tekliflerim))
        .route("/api/kanit", post(kanit))
        .route("/api/kanit/toplu", post(kanit_toplu))
        .route("/api/status", get(status))
        .route("/api/filo", get(filo))
        .route("/api/bakiye", get(bakiye))
        .route("/api/arz", get(arz))
        .route("/api/ledger", get(ledger))
        .route("/api/komut", post(komut))
        .route("/api/denetim", get(denetim_liste))
        .route("/api/denetim/sonuc", post(denetim_sonuc))
        .route("/api/metin/:kor", get(metin))
        .route("/api/kanarya/kontrol", get(kanarya_kontrol))
        .route("/api/shard/ilan", post(shard_ilan))
        .route("/api/shard", get(shard_liste))
        .route("/api/ara", post(ara))
        .route("/api/ara", get(ara_get))
        .route("/api/parca/tohum", post(parca_tohum))
        .route("/api/parca/indir/:hash", get(parca_indir))
        .route("/api/depolama/yoklama", get(yoklama_al))
        .route("/api/depolama/yoklama/sonuc", post(yoklama_sonuc))
        .route("/api/depolama/yoklama/uret", post(yoklama_uret_endpoint))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

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
    // B10 havuz tavani: HAVUZ_MAX_VEKTOR (default 1M, 0 = sinirsiz).
    let havuz_tavan: usize = std::env::var("HAVUZ_MAX_VEKTOR")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(HAVUZ_MAX_VARSAYILAN);
    if havuz_tavan > 0 {
        info!("Vektor havuz tavani: {}", havuz_tavan);
    }
    // B18 WAN kesif: tohum adresleri (virgullu, `/ip4/.../tcp/.../p2p/...`).
    // Bos = yalnizca LAN (mDNS) kesfi (eski davranis).
    let p2p_bootstrap: Vec<String> = std::env::var("P2P_BOOTSTRAP")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    // B18 sabit dugum kimligi: dosya yoksa uretilir (0600). Dosya varsa ayni
    // PeerId ile acilir (tohum listeleri curumez).
    let p2p_key_path = std::env::var("P2P_KEY_PATH").unwrap_or_else(|_| {
        std::path::PathBuf::from(&db_path)
            .parent().map(|p| p.join("p2p-komuta.key"))
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp/p2p-komuta.key"))
            .to_string_lossy().to_string()
    });
    let p2p_seed: Option<[u8; 32]> = match p2p_anahtar_yukle_veya_uret(&p2p_key_path) {
        Ok(s) => Some(s),
        Err(e) => {
            tracing::warn!("p2p kimlik dosyasi acilamadi ({}): {} — gecici kimlik", p2p_key_path, e);
            None
        }
    };
    let embed_api = std::env::var("EMBED_API").unwrap_or_else(|_| "http://127.0.0.1:1241".to_string());
    let embed_model = std::env::var("EMBED_MODEL")
        .unwrap_or_else(|_| "text-embedding-nomic-embed-text-v1.5".to_string());
    // C7: relay dizini + yedek dizini (tohum kaynagi). Yedek dizini DB'nin
    // yanindaki `yedek/` klasorudur; tohum baska yola cikamaz.
    let relay_dir = std::env::var("PARCA_RELAY").unwrap_or_else(|_| {
        std::path::PathBuf::from(&db_path)
            .parent().map(|p| p.join("parca-relay"))
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp/parca-relay"))
            .to_string_lossy().to_string()
    });
    let yedek_dir = std::env::var("PARCA_YEDEK").unwrap_or_else(|_| {
        std::path::PathBuf::from(&db_path)
            .parent().map(|p| p.join("yedek"))
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .to_string_lossy().to_string()
    });
    // Oz-denetim engeli: STRICT_DENETIM=1 ise denetci kendi kanitini denetleyemez.
    // Testnet default 0 (toleransli, sadece warn); mainnet oncesi 1 yapilir.
    let strict_denetim = std::env::var("STRICT_DENETIM").unwrap_or_else(|_| "0".to_string()) != "0";

    let db_url = if db_path.starts_with('/') {
        format!("sqlite:{}", db_path)
    } else {
        format!("sqlite:{}", db_path)
    };
    // Ölçek ayarı (16 Eyl): WAL + NORMAL + 30sn busy-timeout. WAL okur-yazar
    // eszamanliligini artirir (tek-yazar kilidi surer ama okurlar bloklanmaz);
    // NORMAL WAL'da guvenlidir; yedekleme API'si WAL ile uyumludur.
    // NOT: ag diski/NFS'te WAL kullanma (yerel SSD varsayimi).
    let copts: sqlx::sqlite::SqliteConnectOptions = db_url
        .parse()
        .map_err(|e| anyhow::anyhow!("DB URL hatasi: {}", e))?;
    let copts = copts
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_secs(30));
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(copts)
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
    // Tazeleme artimlidir (son_id'den yeni satirlar eklenir); kanitlar'a silme
    // olmadigi icin sonuc tam yuklemeyle esdegerdir (17 Eyl OOM duzeltmesi).
    let matris = std::sync::Arc::new(tokio::sync::RwLock::new(VektorHavuzu::default()));
    let mut son_id: i64 = 0;
    match yukle_havuz(&pool, &corpus, 0).await {
        Ok((mut h, son)) => {
            h.tavan_uygula(havuz_tavan);
            info!("Vektor havuzu yuklendi: {} vektor", h.n);
            *matris.write().await = h;
            son_id = son;
        }
        Err(e) => warn!("ilk havuz yuklemesi basarisiz (bos devam): {}", e),
    }
    {
        let pool_r = pool.clone();
        let matris_r = matris.clone();
        let corpus_r = corpus.clone();
        let tavan_r = havuz_tavan;
        tokio::spawn(async move {
            let mut son = son_id;
            loop {
                tokio::time::sleep(Duration::from_secs(HAVUZ_YENILE_SN)).await;
                match yukle_havuz(&pool_r, &corpus_r, son).await {
                    Ok((ek, yeni_son)) => {
                        son = yeni_son;
                        if ek.n > 0 {
                            let mut m = matris_r.write().await;
                            m.ids.extend(ek.ids);
                            m.duz.extend(ek.duz);
                            m.n += ek.n;
                            let once = m.n;
                            m.tavan_uygula(tavan_r);
                            if m.n < once {
                                info!("havuz tavan kirpmasi: {} -> {}", once, m.n);
                            }
                            info!("Vektor havuzu tazelendi: {} vektor", m.n);
                        }
                    }
                    Err(e) => warn!("havuz tazeleme hatasi: {}", e),
                }
            }
        });
    }

    // R4/P2P gorev duyuru kanali: gorev_kaydet -> abone gorevi -> mesh.
    let (gorev_yayin_tx, gorev_yayin_rx) = tokio::sync::mpsc::unbounded_channel::<(String, Vec<u8>)>();
    let gorev_yayin_acik = std::env::var("P2P_GOREV_YAYIN").unwrap_or_else(|_| "1".to_string()) != "0";
    if !gorev_yayin_acik {
        info!("P2P gorev yayini kapali (P2P_GOREV_YAYIN=0)");
    }
    let mut gorev_yayin_rx: Option<tokio::sync::mpsc::UnboundedReceiver<(String, Vec<u8>)>> =
        if gorev_yayin_acik { Some(gorev_yayin_rx) } else { None };
    let state = Arc::new(AppState {
        pool,
        wiki,
        http,
        matris,
        havuz_tavan,
        dagitim_kilidi: tokio::sync::Mutex::new(()),
        master_pubkey_b64: master_pubkey,
        corpus,
        embed_api,
        embed_model,
        relay_dir: relay_dir.clone(),
        yedek_dir: yedek_dir.clone(),
        gorev_yayin_tx: if gorev_yayin_acik { Some(gorev_yayin_tx) } else { None },
        strict_denetim,
    });
    info!("Parca relay: {} | yedek: {}", relay_dir, yedek_dir);

    let app = app_router(state.clone());

    // Gossip abonesi: nemes/shard ilanlarini dinler, deftere isler.
    // HTTP API'dan bagimsiz gorevde kosar; duserse API etkilenmez.
    if shard_sub {
        let state2 = state.clone();
        let mut yayin_rx = gorev_yayin_rx.take();
        tokio::spawn(async move {
            let mut node = match P2PNode::new(P2PConfig { port: p2p_port, enable_mdns: true, bootstrap: p2p_bootstrap.clone(), key_seed: p2p_seed }).await {
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
                // R4/P2P: biriken gorev duyurularini mesh'e yayinla.
                // Bos mesh / hatsiz publish zararsizdir (hata yoksayilir).
                if let Some(rx) = yayin_rx.as_mut() {
                    while let Ok((konu, raw)) = rx.try_recv() {
                        match node.publish(&konu, raw) {
                            Ok(()) => info!("mesh duyurusu yayinlandi: {}", konu),
                            Err(e) => warn!("mesh duyuru yayin hatasi ({}): {}", konu, e),
                        }
                    }
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

    // C8 yoklama dongusu: sure-dolmuslari kapat + dusurme kontrolu + yeni uret.
    {
        let pool_y = state.pool.clone();
        let relay_y = state.relay_dir.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(YOKLAMA_ARALIK_SN)).await;
                let now = current_epoch();
                // 1. Suresi dolanlari kapat.
                let dolan: Vec<(i64, String)> = sqlx::query_as::<_, (i64, String)>(
                    "SELECT id, miner_id FROM yoklamalar WHERE durum = 'acik' AND ts < ?"
                )
                .bind(now - YOKLAMA_SURE_SN)
                .fetch_all(&pool_y)
                .await
                .unwrap_or_default();
                for (yid, mid) in dolan {
                    let _ = sqlx::query("UPDATE yoklamalar SET durum = 'sure-doldu' WHERE id = ? AND durum = 'acik'")
                        .bind(yid).execute(&pool_y).await;
                    match dusurme_kontrol(&pool_y, &mid).await {
                        Ok(true) => warn!("yoklama zaman asimi sonrasi DUSURME: {}", mid),
                        Ok(false) => warn!("yoklama zaman asimi: {} #{} (ilk fail)", mid, yid),
                        Err(e) => warn!("dusurme kontrol hatasi: {}", e),
                    }
                }
                // 2.5 B21 teklif vade dolumu: suresi dolan acik tekliflerin
                // stake'i iade edilir (cezasiz), teklif kapanir.
                let vade: Vec<(i64, String, i64)> = sqlx::query_as::<_, (i64, String, i64)>(
                    "SELECT id, miner_id, stake_mikro FROM teklifler WHERE durum = 'acik' AND ts < ?",
                )
                .bind(now - TEKLIF_VADE_SN)
                .fetch_all(&pool_y)
                .await
                .unwrap_or_default();
                for (tid, mid, stake) in vade {
                    let _ = sqlx::query("UPDATE teklifler SET durum = 'suresi-doldu', kapanma_ts = ? WHERE id = ? AND durum = 'acik'")
                        .bind(now).bind(tid).execute(&pool_y).await;
                    let _ = sqlx::query("UPDATE miners SET coin_mikro = coin_mikro + ? WHERE miner_id = ?")
                        .bind(stake).bind(&mid).execute(&pool_y).await;
                    let _ = sqlx::query("INSERT INTO ledger (miner_id, delta_mikro, neden, epoch, ts) VALUES (?, ?, 'teklif-iade', ?, ?)")
                        .bind(&mid).bind(stake).bind(now).bind(now).execute(&pool_y).await;
                    warn!("teklif vade dolumu: #{} (iade {} -> {})", tid, stake, mid);
                }
                // 1.5 B25 bagisiklik: son 1 saatin ret oranina gore spot+kanarya
                // siddetini ayarla (basarisiz denetimler artarsa ornekleme artar).
                {
                    let top: i64 = sqlx::query_scalar(
                        "SELECT COUNT(*) FROM kanitlar WHERE dogrulama IS NOT NULL AND ts > ?",
                    )
                    .bind(now - 3600)
                    .fetch_one(&pool_y)
                    .await
                    .unwrap_or(0);
                    if top > 0 {
                        let kotu: i64 = sqlx::query_scalar(
                            &format!("SELECT COUNT(*) FROM kanitlar WHERE dogrulama IS NOT NULL AND dogrulama < {} AND ts > ?", DENETIM_ESIK),
                        )
                        .bind(now - 3600)
                        .fetch_one(&pool_y)
                        .await
                        .unwrap_or(0);
                        let (spot, kanarya) =
                            bagisiklik_eslestir(kotu as f64 / top.max(1) as f64);
                        let once_s = SPOT_YUZDE_DINAMIK.swap(spot, std::sync::atomic::Ordering::Relaxed);
                        let once_k = KANARYA_BPBIN_DINAMIK.swap(kanarya, std::sync::atomic::Ordering::Relaxed);
                        if once_s != spot || once_k != kanarya {
                            info!("bagisiklik guncellendi: ret={}/{} spot=%{} kanarya=%{}bp", kotu, top, spot, kanarya);
                        }
                    }
                }
                // 2. Yeni yoklamalar uret (tur limiti).
                for _ in 0..YOKLAMA_TUR_LIMIT {
                    match yoklama_uret_bir(&pool_y, &relay_y, None).await {
                        Ok(Some(_)) => {}
                        Ok(None) => break,
                        Err(e) => {
                            warn!("yoklama uretim hatasi: {}", e);
                            break;
                        }
                    }
                }
            }
        });
    }

    // HTTP baglama adresi: BIND_ADDR=127.0.0.1 default (guvenlik).
    // Dis erisim Caddy (TLS) uzerinden; dogrudan LAN erisimi gerekmez.
    // P2P (4003) mesh icin ayri sokettedir, bundan etkilenmez.
    let bind_ip: std::net::IpAddr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1".to_string())
        .parse()
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
    let addr = SocketAddr::new(bind_ip, port);
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
#[cfg(test)]
mod tests {
    use super::*;

    fn b64_768(doldur: impl Fn(usize) -> u8) -> String {
        let raw: Vec<u8> = (0..768).map(doldur).collect();
        base64::engine::general_purpose::STANDARD.encode(&raw)
    }

    #[test]
    fn test_dequantize_roundtrip() {
        let b64 = b64_768(|i| (i % 256) as u8);
        let v = dequantize_int8(&b64, 0.0, 1.0).expect("cozulmeli");
        assert_eq!(v.len(), 768);
        assert!((v[0] - 0.0).abs() < 1e-6);
        assert!((v[255] - 1.0).abs() < 1e-6);
        // monotonluk: q buyudukce deger buyur
        assert!(v[255] > v[128] && v[128] > v[0]);
    }

    #[test]
    fn test_dequantize_red() {
        // yanlis boy (100 bayt)
        let kisa = base64::engine::general_purpose::STANDARD.encode(&vec![0u8; 100]);
        assert!(dequantize_int8(&kisa, 0.0, 1.0).is_none());
        // bozuk base64
        assert!(dequantize_int8("!!!degil!!!", 0.0, 1.0).is_none());
        // sifir aralik
        let b64 = b64_768(|_| 42);
        assert!(dequantize_int8(&b64, 1.0, 1.0).is_none());
        // NaN aralik
        assert!(dequantize_int8(&b64, 0.0, f32::NAN).is_none());
    }

    #[test]
    fn test_kosinus() {
        let a = vec![1.0f32, 0.0, 0.0];
        let b = vec![1.0f32, 0.0, 0.0];
        assert!((kosinus(&a, &b) - 1.0).abs() < 1e-5);
        let c = vec![0.0f32, 1.0, 0.0];
        assert!(kosinus(&a, &c).abs() < 1e-5);
        let d = vec![-1.0f32, 0.0, 0.0];
        assert!((kosinus(&a, &d) + 1.0).abs() < 1e-5);
        // olcekten bagimsiz (normalize kosinus)
        let e = vec![5.0f32, 0.0, 0.0];
        assert!((kosinus(&a, &e) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_corpus_kodu() {
        let k1 = corpus_kodu("tr");
        let k2 = corpus_kodu("tr");
        assert_eq!(k1, k2); // deterministik
        assert_eq!(k1.len(), 8); // 4 bayt hex
        assert!(k1.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(k1, corpus_kodu("en")); // corpus ayirimi
    }

    #[test]
    fn test_halving_matematigi() {
        assert_eq!(current_batch_reward_micro(0), BATCH_ODUL_TABAN_MIKRO);
        assert_eq!(current_batch_reward_micro(HALVING_BATCH - 1), BATCH_ODUL_TABAN_MIKRO);
        assert_eq!(current_batch_reward_micro(HALVING_BATCH), BATCH_ODUL_TABAN_MIKRO / 2);
        assert_eq!(current_batch_reward_micro(2 * HALVING_BATCH), BATCH_ODUL_TABAN_MIKRO / 4);
        // Kuyruk taban kilidi (TOKENOMI): 31+ halving de 5 mikroya iner, sifira DUSMEZ.
        assert_eq!(current_batch_reward_micro(31 * HALVING_BATCH), KUYRUK_TABAN_MIKRO);
        assert_eq!(current_batch_reward_micro(i64::MAX / 2), KUYRUK_TABAN_MIKRO);
        assert_eq!(current_batch_reward_micro(9 * HALVING_BATCH), 5); // 2000>>9=3 -> taban 5
    }

    #[test]
    fn test_esik_sabiti() {
        // Site 0.99 yazar, kod 0.98 uygular — bu test kodun gercegini kilitler.
        // Site duzeltilmeden esik DEGISMEZ (ekonomik davranis degisir).
        assert!((DENETIM_ESIK - 0.98).abs() < 1e-6);
        assert_eq!(SPOT_CHECK_YUZDE, 10);
        assert_eq!(TASK_BATCH, 20);
    }

    async fn test_havuzu() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE escrow (gorev_id TEXT NOT NULL, madde_id INTEGER NOT NULL, miner_id TEXT NOT NULL, miktar_mikro INTEGER NOT NULL, ts INTEGER NOT NULL, PRIMARY KEY (gorev_id, madde_id))"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE miners (miner_id TEXT PRIMARY KEY, coin_mikro INTEGER DEFAULT 0)"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE ledger (miner_id TEXT, delta_mikro INTEGER, neden TEXT, epoch INTEGER, ts INTEGER)"
        ).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO miners (miner_id, coin_mikro) VALUES ('m1', 1000)")
            .execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_escrow_serbest_birakma() {
        // Uretimdeki serbest-birakma SQL'lerinin aynisi: coin+ledger+delete.
        let pool = test_havuzu().await;
        sqlx::query("INSERT INTO escrow (gorev_id, madde_id, miner_id, miktar_mikro, ts) VALUES ('g1', 7, 'm1', 100, 1)")
            .execute(&pool).await.unwrap();
        sqlx::query("UPDATE miners SET coin_mikro = coin_mikro + 100 WHERE miner_id = 'm1'")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO ledger (miner_id, delta_mikro, neden, epoch, ts) VALUES ('m1', 100, 'escrow', 1, 1)")
            .execute(&pool).await.unwrap();
        sqlx::query("DELETE FROM escrow WHERE gorev_id = 'g1' AND madde_id = 7")
            .execute(&pool).await.unwrap();
        let coin: i64 = sqlx::query_scalar("SELECT coin_mikro FROM miners WHERE miner_id = 'm1'")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(coin, 1100);
        let defter: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(delta_mikro), 0) FROM ledger WHERE miner_id = 'm1'")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(defter, 100); // defter tutarliligi (bakiye kosulu)
        let kalan: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM escrow")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(kalan, 0);
    }

    #[tokio::test]
    async fn test_escrow_yanma() {
        // Slash'ta emanet silinir, coin ve deftere DOKUNULMAZ (odenmemisti).
        let pool = test_havuzu().await;
        sqlx::query("INSERT INTO escrow (gorev_id, madde_id, miner_id, miktar_mikro, ts) VALUES ('g2', 9, 'm1', 250, 1)")
            .execute(&pool).await.unwrap();
        sqlx::query("DELETE FROM escrow WHERE gorev_id = 'g2' AND madde_id = 9")
            .execute(&pool).await.unwrap();
        let coin: i64 = sqlx::query_scalar("SELECT coin_mikro FROM miners WHERE miner_id = 'm1'")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(coin, 1000);
        let defter: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(delta_mikro), 0) FROM ledger WHERE miner_id = 'm1'")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(defter, 0);
    }

    #[test]
    fn test_havuz_tavan() {
        // B10: fazlalik en eskiden kirpilir, ids+duz eszamanli.
        let mut h = VektorHavuzu { ids: vec![1, 2, 3, 4, 5], duz: vec![0.0; 5 * ARA_BOYUT], n: 5 };
        h.tavan_uygula(3);
        assert_eq!(h.n, 3);
        assert_eq!(h.ids, vec![3, 4, 5]);
        assert_eq!(h.duz.len(), 3 * ARA_BOYUT);
        h.tavan_uygula(0); // sinirsiz: degismez
        assert_eq!(h.n, 3);
        h.tavan_uygula(99); // tavan ustu: degismez
        assert_eq!(h.n, 3);
    }

    #[tokio::test]
    async fn test_kilit_metrik() {
        // Metrik sayaci her kilitte genau 1 artar (ölçek gözetiminin temeli).
        let m = tokio::sync::Mutex::new(());
        let n0 = KILIT_SAYI.load(std::sync::atomic::Ordering::Relaxed);
        { let _g = kilit_al(&m).await; }
        { let _g = kilit_al(&m).await; }
        let n1 = KILIT_SAYI.load(std::sync::atomic::Ordering::Relaxed);
        assert_eq!(n1, n0 + 2);
    }

    /// B3: tam batch yaşam döngüsü GERÇEK handler'larla (:memory: DB).
    /// kanit × 20 (kapanış) → escrow matematiği → denetim geçişi (serbest) →
    /// çöp kanıt → 2 ret (slash+yakma) → defter tutarlılığı.
    /// NOT: spot örneklemesi saltlı-rastgele; test bayrakları DB'den okuyup
    /// ona göre sürer (yapıyı değil, mekaniği kilitler).
    async fn test_state() -> std::sync::Arc<AppState> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let wiki = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        std::sync::Arc::new(AppState {
            pool,
            wiki,
            http: reqwest::Client::new(),
            matris: std::sync::Arc::new(tokio::sync::RwLock::new(VektorHavuzu::default())),
            dagitim_kilidi: tokio::sync::Mutex::new(()),
            master_pubkey_b64: String::new(),
            corpus: "test".to_string(),
            embed_api: String::new(),
            embed_model: String::new(),
            relay_dir: "/tmp".to_string(),
            yedek_dir: "/tmp".to_string(),
            gorev_yayin_tx: None,
            strict_denetim: false,
            havuz_tavan: 0,
        })
    }

    fn test_headers(token: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert("authorization", format!("Bearer {}", token).parse().unwrap());
        h
    }

    // Tohumlu sözde-rastgele 768B vektör: AYNI tohum AYNI yön (cos=1),
    // FARKLI tohum ILISKISIZ yön (cos≈0). Sabit-desen KULLANMA (normalize
    // olunca yönler çakışır, kosinüs hep 1.0 çıkar).
    fn test_vektor(seed: u64) -> String {
        let mut x = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        if x == 0 {
            x = 0x9E3779B97F4A7C15;
        }
        let raw: Vec<u8> = (0..768)
            .map(|_| {
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                (x >> 33) as u8
            })
            .collect();
        base64::engine::general_purpose::STANDARD.encode(&raw)
    }

    #[tokio::test]
    async fn test_batch_yasam_dongusu() {
        let state = test_state().await;
        // 2 madenci + 20'lik görev + kör eşleme.
        for (mid, tok) in [("m1", "t1"), ("m2", "t2")] {
            sqlx::query("INSERT INTO miners (miner_id, token, cuzdan, makine_id, created_at) VALUES (?, ?, 'c', 'mk', 1)")
                .bind(mid).bind(tok).execute(&state.pool).await.unwrap();
        }
        sqlx::query("INSERT INTO gorevler (gorev_id, dagitilan_miner, corpus, offset, beklenen, alinan, durum, odul_mikro, ts) VALUES ('test:1', 'm1', 'test', 0, 20, 0, 'acik', 0, 1)")
            .execute(&state.pool).await.unwrap();
        for i in 0..20i64 {
            sqlx::query("INSERT INTO kor_esleme (kor_id, madde_id, gorev_id, ts) VALUES (?, ?, 'test:1', 1)")
                .bind(5000 + i).bind(1 + i).execute(&state.pool).await.unwrap();
        }
        // 20 kanıt (10+10), aynı dürüst vektör.
        let vb = test_vektor(7);
        let mut son = None;
        for i in 0..20i64 {
            let tok = if i % 2 == 0 { "t1" } else { "t2" };
            let r = kanit(
                State(state.clone()),
                test_headers(tok),
                Json(KanitReq { gorev_id: "test:1".to_string(), madde_id: 5000 + i, v_int8_b64: vb.clone(), v_min: 0.0, v_max: 1.0, imza: None }),
            ).await.unwrap().0;
            son = Some(r);
        }
        let son = son.unwrap();
        assert!(son.kabul && son.batch_tamam, "batch kapanmali");
        // Koruma: dagitilan toplam == batch odulu (temiz + emanet).
        let odenen: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(coin_mikro),0) FROM miners")
            .fetch_one(&state.pool).await.unwrap();
        let emanet: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(miktar_mikro),0) FROM escrow")
            .fetch_one(&state.pool).await.unwrap();
        assert_eq!(odenen + emanet, 2000, "batch odulu korunmali (taban, kademe 0)");
        // Bayraklı satırlar emanette, bayraksızlar odenmis.
        let bayrakli: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kanitlar WHERE gorev_id='test:1' AND spot_check=1 AND dogrulama IS NULL")
            .fetch_one(&state.pool).await.unwrap();
        let emanet_satir: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM escrow WHERE gorev_id='test:1'")
            .fetch_one(&state.pool).await.unwrap();
        assert_eq!(bayrakli, emanet_satir, "her supheli emanette olmali");
        // Geçiş: bayraklı ilk kanıtı BAŞKA madenci aynı vektörle doğrular.
        if bayrakli > 0 {
            let (kor, gercek): (i64, i64) = sqlx::query_as(
                "SELECT k.madde_id, k.madde_id FROM kanitlar k WHERE k.gorev_id='test:1' AND k.spot_check=1 AND k.dogrulama IS NULL LIMIT 1")
                .fetch_one(&state.pool).await.unwrap();
            // kor_id'yi eslemeden bul (kanitlar GERCEK id tutar).
            let kor_id: i64 = sqlx::query_scalar("SELECT kor_id FROM kor_esleme WHERE gorev_id='test:1' AND madde_id=?")
                .bind(gercek).fetch_one(&state.pool).await.unwrap();
            let _ = kor;
            let r = denetim_sonuc(
                State(state.clone()),
                test_headers("t2"),
                Json(DenetimSonuc { gorev_id: "test:1".to_string(), madde_id: kor_id, v_int8_b64: Some(vb.clone()), v_min: Some(0.0), v_max: Some(1.0), dogrulama: None, gecerli: None }),
            ).await.unwrap();
            let v = r.0;
            assert_eq!(v.get("gecerli").and_then(|x| x.as_bool()), Some(true), "ayni vektor gecmeli");
            assert!((v.get("dogrulama").and_then(|x| x.as_f64()).unwrap_or(-1.0) - 1.0).abs() < 1e-5);
        }
        // Defter tutarlılığı: her madencide coin == ledger toplami.
        for mid in ["m1", "m2"] {
            let coin: i64 = sqlx::query_scalar("SELECT coin_mikro FROM miners WHERE miner_id=?")
                .bind(mid).fetch_one(&state.pool).await.unwrap();
            let defter: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(delta_mikro),0) FROM ledger WHERE miner_id=?")
                .bind(mid).fetch_one(&state.pool).await.unwrap();
            assert_eq!(coin, defter, "defter tutarli olmali: {}", mid);
        }
    }

    #[tokio::test]
    async fn test_toplu_yasam_dongusu() {
        // B14: 20 kanit TEK istekte; kapanis + odul korunumu tekil akisla esdeger.
        let state = test_state().await;
        for (mid, tok) in [("m1", "t1"), ("m2", "t2")] {
            sqlx::query("INSERT INTO miners (miner_id, token, cuzdan, makine_id, created_at) VALUES (?, ?, 'c', 'mk', 1)")
                .bind(mid).bind(tok).execute(&state.pool).await.unwrap();
        }
        sqlx::query("INSERT INTO gorevler (gorev_id, dagitilan_miner, corpus, offset, beklenen, alinan, durum, odul_mikro, ts) VALUES ('test:2', 'm1', 'test', 0, 20, 0, 'acik', 0, 1)")
            .execute(&state.pool).await.unwrap();
        for i in 0..20i64 {
            sqlx::query("INSERT INTO kor_esleme (kor_id, madde_id, gorev_id, ts) VALUES (?, ?, 'test:2', 1)")
                .bind(6000 + i).bind(101 + i).execute(&state.pool).await.unwrap();
        }
        let vb = test_vektor(7);
        let istekler: Vec<KanitReq> = (0..20i64)
            .map(|i| KanitReq {
                gorev_id: "test:2".to_string(),
                madde_id: 6000 + i,
                v_int8_b64: vb.clone(),
                v_min: 0.0,
                v_max: 1.0,
                imza: None,
            })
            .collect();
        let resp = kanit_toplu(State(state.clone()), test_headers("t1"), Json(istekler))
            .await
            .unwrap()
            .0;
        let sonuclar = resp.get("sonuclar").and_then(|v| v.as_array()).unwrap();
        assert_eq!(sonuclar.len(), 20, "20 kalem donmeli");
        assert!(sonuclar.iter().all(|s| s.get("kabul").and_then(|x| x.as_bool()) == Some(true)), "hepsi kabul");
        assert!(sonuclar.iter().any(|s| s.get("batch_tamam").and_then(|x| x.as_bool()) == Some(true)), "batch kapanmali");
        let odenen: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(coin_mikro),0) FROM miners")
            .fetch_one(&state.pool).await.unwrap();
        let emanet: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(miktar_mikro),0) FROM escrow")
            .fetch_one(&state.pool).await.unwrap();
        assert_eq!(odenen + emanet, 2000, "batch odulu korunmali");
        // Ayni batch tekrar: 20 kalem de CONFLICT ile reddedilir, odul degismez.
        let istekler2: Vec<KanitReq> = (0..20i64)
            .map(|i| KanitReq {
                gorev_id: "test:2".to_string(),
                madde_id: 6000 + i,
                v_int8_b64: vb.clone(),
                v_min: 0.0,
                v_max: 1.0,
                imza: None,
            })
            .collect();
        let resp2 = kanit_toplu(State(state.clone()), test_headers("t1"), Json(istekler2))
            .await
            .unwrap()
            .0;
        let son2 = resp2.get("sonuclar").and_then(|v| v.as_array()).unwrap();
        assert!(son2.iter().all(|s| s.get("kabul").and_then(|x| x.as_bool()) == Some(false)), "cift kayit reddedilmeli");
        let odenen2: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(coin_mikro),0) FROM miners")
            .fetch_one(&state.pool).await.unwrap();
        let emanet2: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(miktar_mikro),0) FROM escrow")
            .fetch_one(&state.pool).await.unwrap();
        assert_eq!(odenen2 + emanet2, 2000, "tekrar odul uretmemeli");
        // Bos + asiri batch reddedilir.
        assert!(kanit_toplu(State(state.clone()), test_headers("t1"), Json(vec![])).await.is_err());
        let asiri: Vec<KanitReq> = (0..101i64)
            .map(|i| KanitReq {
                gorev_id: "test:2".to_string(),
                madde_id: 6000 + i,
                v_int8_b64: vb.clone(),
                v_min: 0.0,
                v_max: 1.0,
                imza: None,
            })
            .collect();
        assert!(kanit_toplu(State(state.clone()), test_headers("t1"), Json(asiri)).await.is_err());
    }

    #[tokio::test]
    async fn test_mesh_duyuru_turu() {
        // B15: kapanmis batch'in bayraklilari duyurulur, atama dogrulanabilir.
        let state = test_state().await;
        for (mid, tok) in [("m1", "t1"), ("m2", "t2"), ("m3", "t3")] {
            sqlx::query("INSERT INTO miners (miner_id, token, cuzdan, makine_id, created_at, last_seen) VALUES (?, ?, 'c', 'mk', 1, 9999999999)")
                .bind(mid).bind(tok).execute(&state.pool).await.unwrap();
        }
        sqlx::query("INSERT INTO gorevler (gorev_id, dagitilan_miner, corpus, offset, beklenen, alinan, durum, odul_mikro, ts) VALUES ('test:4', 'm1', 'test', 0, 2, 0, 'acik', 0, 1)")
            .execute(&state.pool).await.unwrap();
        for (kor, mid) in [(7001i64, 201i64), (7002, 202)] {
            sqlx::query("INSERT INTO kor_esleme (kor_id, madde_id, gorev_id, ts) VALUES (?, ?, 'test:4', 1)")
                .bind(kor).bind(mid).execute(&state.pool).await.unwrap();
            sqlx::query("INSERT INTO kanitlar (gorev_id, madde_id, miner_id, v_int8_b64, v_min, v_max, odul_mikro, spot_check, ts) VALUES ('test:4', ?, 'm1', 'eA', 0.0, 1.0, 0, 1, 1)")
                .bind(mid).execute(&state.pool).await.unwrap();
        }
        let d = mesh_duyuru_kur(&state, "test:4", 42, 9999999999).await.unwrap();
        let d = d.expect("bayrakli varken duyuru kurulmali");
        assert_eq!(d.atamalar.len(), 2);
        // Yeniden hesapla-dogrula (paylasilan modulle ayni sonuc).
        let mut proverlar = std::collections::HashMap::new();
        proverlar.insert(7001i64, "m1".to_string());
        proverlar.insert(7002i64, "m1".to_string());
        let kayitli = vec!["m1".to_string(), "m2".to_string(), "m3".to_string()];
        assert!(nemes_core::mesh_audit::duyuru_dogrula(&d, &kayitli, &proverlar));
        // Denetciler m1 degil, 2 kisi.
        for a in &d.atamalar {
            assert_eq!(a.denetciler.len(), 2);
            assert!(!a.denetciler.contains(&"m1".to_string()));
        }
        // Bayrak yoksa None.
        sqlx::query("UPDATE kanitlar SET spot_check = 0 WHERE gorev_id = 'test:4'").execute(&state.pool).await.unwrap();
        assert!(mesh_duyuru_kur(&state, "test:4", 42, 9999999999).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_kira_yasam_dongusu() {
        // B20: kira ile 40 madde (2 alt-gorev); ilk alt-gorev kapanir, odul korunur.
        let state = test_state().await;
        sqlx::query("INSERT INTO miners (miner_id, token, cuzdan, makine_id, created_at, last_seen) VALUES ('m1', 't1', 'c', 'mk', 1, 9999999999)")
            .execute(&state.pool).await.unwrap();
        sqlx::query("CREATE TABLE madde (id INTEGER PRIMARY KEY, ozet TEXT)").execute(&state.wiki).await.unwrap();
        for i in 1..=40i64 {
            sqlx::query("INSERT INTO madde (id, ozet) VALUES (?, ?)")
                .bind(i)
                .bind(format!("kira test maddesi {} sufficiently long text for length filter", i))
                .execute(&state.wiki).await.unwrap();
        }
        let resp = kira_al(State(state.clone()), test_headers("t1"), Json(KiraIstek { adet: Some(40) }))
            .await
            .unwrap()
            .0;
        let gorevler = resp.get("gorevler").and_then(|v| v.as_array()).unwrap();
        assert_eq!(gorevler.len(), 2, "40 madde = 2 alt-gorev");
        // Cursor tek yazimla 40'a dayanmali.
        let cursor: i64 = sqlx::query_scalar("SELECT son_id FROM gorev_cursor WHERE corpus = 'test'")
            .fetch_one(&state.pool).await.unwrap();
        assert_eq!(cursor, 40);
        // Kor eslemesi 40 satir (dagitim korlari).
        let kor_say: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kor_esleme")
            .fetch_one(&state.pool).await.unwrap();
        assert_eq!(kor_say, 40);
        // Ilk alt-gorevin 20 kaniti (kor ID'ler payload'dan) -> kapanis.
        let ilk = &gorevler[0];
        let gid = ilk.get("id").and_then(|v| v.as_str()).unwrap();
        let korlar: Vec<i64> = ilk.get("payload").and_then(|p| p.get("madde_idler"))
            .and_then(|v| v.as_array()).unwrap()
            .iter().filter_map(|v| v.as_i64()).collect();
        assert_eq!(korlar.len(), 20);
        let vb = test_vektor(7);
        let mut son = None;
        for k in &korlar {
            let r = kanit(
                State(state.clone()),
                test_headers("t1"),
                Json(KanitReq { gorev_id: gid.to_string(), madde_id: *k, v_int8_b64: vb.clone(), v_min: 0.0, v_max: 1.0, imza: None }),
            ).await.unwrap().0;
            son = Some(r);
        }
        let son = son.unwrap();
        assert!(son.kabul && son.batch_tamam, "kira alt-gorevi kapanmali");
        let odenen: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(coin_mikro),0) FROM miners")
            .fetch_one(&state.pool).await.unwrap();
        let emanet: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(miktar_mikro),0) FROM escrow")
            .fetch_one(&state.pool).await.unwrap();
        assert_eq!(odenen + emanet, 2000, "kira odulu korunmali");
        // Bos wiki -> bos liste (madenci eski yola duser).
        let state2 = test_state().await;
        sqlx::query("INSERT INTO miners (miner_id, token, cuzdan, makine_id, created_at, last_seen) VALUES ('m1', 't1', 'c', 'mk', 1, 9999999999)")
            .execute(&state2.pool).await.unwrap();
        sqlx::query("CREATE TABLE madde (id INTEGER PRIMARY KEY, ozet TEXT)").execute(&state2.wiki).await.unwrap();
        let resp2 = kira_al(State(state2.clone()), test_headers("t1"), Json(KiraIstek { adet: Some(40) }))
            .await
            .unwrap()
            .0;
        assert!(resp2.get("gorevler").and_then(|v| v.as_array()).unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_teklif_yasam_dongusu() {
        // B21: teklif ver -> kilit -> oncelikli supurme -> kapanis ucret+iade.
        let state = test_state().await;
        for (mid, tok) in [("m1", "t1"), ("m2", "t2")] {
            sqlx::query("INSERT INTO miners (miner_id, token, cuzdan, makine_id, created_at, last_seen, coin_mikro) VALUES (?, ?, 'c', 'mk', 1, 9999999999, 100000)")
                .bind(mid).bind(tok).execute(&state.pool).await.unwrap();
        }
        sqlx::query("CREATE TABLE madde (id INTEGER PRIMARY KEY, ozet TEXT)").execute(&state.wiki).await.unwrap();
        for i in 1..=60i64 {
            sqlx::query("INSERT INTO madde (id, ozet) VALUES (?, ?)")
                .bind(i)
                .bind(format!("teklif test maddesi {} sufficiently long text here", i))
                .execute(&state.wiki).await.unwrap();
        }
        // m2 [1,40] araligini onerir (bos aralik).
        let r = teklif_ver(
            State(state.clone()),
            test_headers("t2"),
            Json(TeklifIstek { corpus: "test".to_string(), baslangic: 1, bitis: 40 }),
        ).await.unwrap().0;
        let tid = r.get("teklif_id").and_then(|v| v.as_i64()).unwrap();
        assert_eq!(r.get("durum").and_then(|v| v.as_str()), Some("acik"));
        // Stake kilitlendi (coin dustu + ledger negatif).
        let coin: i64 = sqlx::query_scalar("SELECT coin_mikro FROM miners WHERE miner_id='m2'")
            .fetch_one(&state.pool).await.unwrap();
        assert_eq!(coin, 100000 - 500);
        // Ayni aralik tekrar -> mevcut doner (yeni kilit yok).
        let r2 = teklif_ver(
            State(state.clone()),
            test_headers("t2"),
            Json(TeklifIstek { corpus: "test".to_string(), baslangic: 5, bitis: 30 }),
        ).await.unwrap().0;
        assert_eq!(r2.get("teklif_id").and_then(|v| v.as_i64()), Some(tid));
        assert_eq!(r2.get("durum").and_then(|v| v.as_str()), Some("mevcut"));
        let coin2: i64 = sqlx::query_scalar("SELECT coin_mikro FROM miners WHERE miner_id='m2'")
            .fetch_one(&state.pool).await.unwrap();
        assert_eq!(coin2, coin, "cift kilit olmamali");
        // Supurme onceligi teklifi secer (ilk 20 kapsanmamis).
        let sup = supurme_dene(&state, 1000).await.unwrap().expect("teklif supurmeli");
        assert_eq!(sup.0, 1, "teklif araligindan baslamali");
        assert_eq!(sup.1.len(), 20);
        // Teklif araligindaki 2 batch'i m1 kapatir (40 kanit, kor eslemeli).
        for (b, taban) in [(9001i64, 1i64), (9101, 21)] {
            let gid = format!("test:5:{}", b);
            sqlx::query("INSERT INTO gorevler (gorev_id, dagitilan_miner, corpus, offset, beklenen, alinan, durum, odul_mikro, ts) VALUES (?, 'm1', 'test', ?, 20, 0, 'acik', 0, 1)")
                .bind(&gid).bind(taban).execute(&state.pool).await.unwrap();
            for i in 0..20i64 {
                sqlx::query("INSERT INTO kor_esleme (kor_id, madde_id, gorev_id, ts) VALUES (?, ?, ?, 1)")
                    .bind(b + i).bind(taban + i).bind(&gid).execute(&state.pool).await.unwrap();
            }
            let vb = test_vektor(7);
            for i in 0..20i64 {
                kanit(
                    State(state.clone()),
                    test_headers("t1"),
                    Json(KanitReq { gorev_id: gid.clone(), madde_id: b + i, v_int8_b64: vb.clone(), v_min: 0.0, v_max: 1.0, imza: None }),
                ).await.unwrap();
            }
        }
        // 40/40 kapandi -> teklif tamam + iade + bulucu payi.
        let durum: String = sqlx::query_scalar("SELECT durum FROM teklifler WHERE id = ?")
            .bind(tid).fetch_one(&state.pool).await.unwrap();
        assert_eq!(durum, "tamam");
        let odul: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(delta_mikro),0) FROM ledger WHERE miner_id='m2' AND neden='teklif-odul'")
            .fetch_one(&state.pool).await.unwrap();
        assert!(odul > 0, "bulucu payi odenmeli");
        let iade: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(delta_mikro),0) FROM ledger WHERE miner_id='m2' AND neden='teklif-iade'")
            .fetch_one(&state.pool).await.unwrap();
        assert_eq!(iade, 500, "stake iade edilmeli");
        // Kapali araliga yeni teklif RED.
        let red = teklif_ver(
            State(state.clone()),
            test_headers("t1"),
            Json(TeklifIstek { corpus: "test".to_string(), baslangic: 1, bitis: 40 }),
        ).await;
        assert!(red.is_err(), "kapali aralik reddedilmeli");
    }

    /// B3 entegrasyon iskelesi: gercek HTTP yigini (ephemeral port).
    /// Donen: (taban_url, paylasilan_state).
    async fn test_sunucu() -> (String, std::sync::Arc<AppState>) {
        let state = test_state().await;
        sqlx::query("CREATE TABLE madde (id INTEGER PRIMARY KEY, ozet TEXT)")
            .execute(&state.wiki)
            .await
            .unwrap();
        for i in 1..=60i64 {
            sqlx::query("INSERT INTO madde (id, ozet) VALUES (?, ?)")
                .bind(i)
                .bind(format!("entegrasyon test maddesi {} sufficiently long text here", i))
                .execute(&state.wiki)
                .await
                .unwrap();
        }
        let app = app_router(state.clone());
        let dinleyici = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let adres = dinleyici.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(dinleyici, app).await.unwrap();
        });
        (format!("http://{}", adres), state)
    }

    fn test_istemci() -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap()
    }

    async fn test_kayit(
        istemci: &reqwest::Client,
        taban: &str,
        cuzdan: &str,
    ) -> String {
        let r = istemci
            .post(format!("{}/api/kayit", taban))
            .json(&serde_json::json!({ "cuzdan": cuzdan, "makine_id": "test-makine" }))
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 200);
        let j: serde_json::Value = r.json().await.unwrap();
        j.get("token").and_then(|v| v.as_str()).unwrap().to_string()
    }

    #[tokio::test]
    async fn test_http_iskelet() {
        // B3: kablo canli mi? /health + 404 + auth kapisi.
        let (taban, _state) = test_sunucu().await;
        let istemci = test_istemci();
        let r = istemci.get(format!("{}/health", taban)).send().await.unwrap();
        assert_eq!(r.status(), 200);
        let j: serde_json::Value = r.json().await.unwrap();
        assert_eq!(j.get("ok").and_then(|v| v.as_bool()), Some(true));
        // Bilinmeyen yol 404.
        let r = istemci.get(format!("{}/api/yok-boyle-uc", taban)).send().await.unwrap();
        assert_eq!(r.status(), 404);
        // Tokensiz status 401.
        let r = istemci.get(format!("{}/api/status", taban)).send().await.unwrap();
        assert_eq!(r.status(), 401);
        // Kayit acik (tokensiz) ve token uretir.
        let tok = test_kayit(&istemci, &taban, "cuzdan-a").await;
        assert!(!tok.is_empty());
        // Tokenli status 200 + alanlar.
        let r = istemci
            .get(format!("{}/api/status", taban))
            .header("Authorization", format!("Bearer {}", tok))
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 200);
        let j: serde_json::Value = r.json().await.unwrap();
        assert!(j.get("miner_id").and_then(|v| v.as_str()).is_some());
        assert!(j.get("coin_mikro").is_some());
    }

    #[tokio::test]
    async fn test_http_uctan_uca_kira() {
        // B3: kayit -> kira -> toplu kanit -> kapanis -> odul korunumu (HTTP).
        let (taban, state) = test_sunucu().await;
        let istemci = test_istemci();
        let tok = test_kayit(&istemci, &taban, "cuzdan-a").await;
        let auth = format!("Bearer {}", tok);
        // Kira: 40 madde = 2 alt-gorev.
        let r = istemci
            .post(format!("{}/api/kira/al", taban))
            .header("Authorization", auth.clone())
            .json(&serde_json::json!({ "adet": 40 }))
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 200);
        let j: serde_json::Value = r.json().await.unwrap();
        let gorevler = j.get("gorevler").and_then(|v| v.as_array()).unwrap();
        assert_eq!(gorevler.len(), 2);
        // Ilk alt-gorevin kor ID'leriyle toplu kanit.
        let ilk = &gorevler[0];
        let gid = ilk.get("id").and_then(|v| v.as_str()).unwrap();
        let korlar: Vec<i64> = ilk
            .get("payload")
            .and_then(|p| p.get("madde_idler"))
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .filter_map(|v| v.as_i64())
            .collect();
        assert_eq!(korlar.len(), 20);
        let vb = test_vektor(7);
        let kalemler: Vec<serde_json::Value> = korlar
            .iter()
            .map(|k| {
                serde_json::json!({ "gorev_id": gid, "madde_id": k, "v_int8_b64": vb, "v_min": 0.0, "v_max": 1.0, "imza": null })
            })
            .collect();
        let r = istemci
            .post(format!("{}/api/kanit/toplu", taban))
            .header("Authorization", auth.clone())
            .json(&kalemler)
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 200);
        let j: serde_json::Value = r.json().await.unwrap();
        let sonuclar = j.get("sonuclar").and_then(|v| v.as_array()).unwrap();
        assert_eq!(sonuclar.len(), 20);
        assert!(sonuclar.iter().all(|s| s.get("kabul").and_then(|x| x.as_bool()) == Some(true)));
        assert!(sonuclar.iter().any(|s| s.get("batch_tamam").and_then(|x| x.as_bool()) == Some(true)));
        // Odul korunumu DB'den dogrula.
        let odenen: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(coin_mikro),0) FROM miners")
            .fetch_one(&state.pool)
            .await
            .unwrap();
        let emanet: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(miktar_mikro),0) FROM escrow")
            .fetch_one(&state.pool)
            .await
            .unwrap();
        assert_eq!(odenen + emanet, 2000);
    }

    #[tokio::test]
    async fn test_http_denetim_turu() {
        // B3: bayrakli kanit -> denetim gorevi (HTTP poll) -> sonuc -> dogrulama.
        let (taban, state) = test_sunucu().await;
        let istemci = test_istemci();
        let t1 = test_kayit(&istemci, &taban, "cuzdan-a").await;
        let t2 = test_kayit(&istemci, &taban, "cuzdan-b").await;
        // m1 kira ile 20 kanit basar (icinden bayrakli cikar).
        let r = istemci
            .post(format!("{}/api/kira/al", taban))
            .header("Authorization", format!("Bearer {}", t1))
            .json(&serde_json::json!({ "adet": 20 }))
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 200);
        let j: serde_json::Value = r.json().await.unwrap();
        let gorevler = j.get("gorevler").and_then(|v| v.as_array()).unwrap();
        assert_eq!(gorevler.len(), 1);
        let gid = gorevler[0].get("id").and_then(|v| v.as_str()).unwrap().to_string();
        let korlar: Vec<i64> = gorevler[0]
            .get("payload")
            .and_then(|p| p.get("madde_idler"))
            .and_then(|v| v.as_array())
            .unwrap()
            .iter()
            .filter_map(|v| v.as_i64())
            .collect();
        let vb = test_vektor(7);
        // Bayrak birikene kadar kira+kanit turu (spot %10 zar: tur basi ~1/3
        // olasilikla >=3 bayrak; 6 turda kacirma olasiligi ihmal edilir).
        // Her turda farkli gorev (kor cakismasi yok).
        let mut tur = 0;
        let bayrakli: i64 = loop {
            tur += 1;
            assert!(tur <= 6, "bayrak birikmedi");
            let r = istemci
                .post(format!("{}/api/kira/al", taban))
                .header("Authorization", format!("Bearer {}", t1))
                .json(&serde_json::json!({ "adet": 20 }))
                .send()
                .await
                .unwrap();
            assert_eq!(r.status(), 200);
            let j: serde_json::Value = r.json().await.unwrap();
            let gs = j.get("gorevler").and_then(|v| v.as_array()).unwrap();
            assert_eq!(gs.len(), 1);
            let gid = gs[0].get("id").and_then(|v| v.as_str()).unwrap().to_string();
            let korlar: Vec<i64> = gs[0]
                .get("payload")
                .and_then(|p| p.get("madde_idler"))
                .and_then(|v| v.as_array())
                .unwrap()
                .iter()
                .filter_map(|v| v.as_i64())
                .collect();
            let kalemler: Vec<serde_json::Value> = korlar
                .iter()
                .map(|k| {
                    serde_json::json!({ "gorev_id": gid, "madde_id": k, "v_int8_b64": vb, "v_min": 0.0, "v_max": 1.0, "imza": null })
                })
                .collect();
            let r = istemci
                .post(format!("{}/api/kanit/toplu", taban))
                .header("Authorization", format!("Bearer {}", t1))
                .json(&kalemler)
                .send()
                .await
                .unwrap();
            assert_eq!(r.status(), 200);
            let n: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM kanitlar WHERE spot_check = 1 AND dogrulama IS NULL",
            )
            .fetch_one(&state.pool)
            .await
            .unwrap();
            if n >= 3 {
                break n;
            }
        };
        assert!(bayrakli >= 3);
        // m2 denetim gorevi yakalayana kadar yokla (1/5 sans, 80 deneme).
        let mut denetim: Option<serde_json::Value> = None;
        for _ in 0..80 {
            let r = istemci
                .get(format!("{}/api/gorev", taban))
                .header("Authorization", format!("Bearer {}", t2))
                .send()
                .await
                .unwrap();
            if r.status() == 200 {
                let g: serde_json::Value = r.json().await.unwrap();
                if g.get("tip").and_then(|v| v.as_str()) == Some("denetim") {
                    denetim = Some(g);
                    break;
                }
            }
        }
        let denetim = denetim.expect("80 yoklamada denetim gorevi gelmeli");
        let refs = denetim
            .get("payload")
            .and_then(|p| p.get("denetim"))
            .and_then(|v| v.as_array())
            .unwrap();
        assert!(!refs.is_empty());
        // Ayni vektorle sonuclari bas: hepsi gecmeli.
        let mut gecen = 0;
        for rf in refs {
            let rg = rf.get("gorev_id").and_then(|v| v.as_str()).unwrap();
            let rm = rf.get("madde_id").and_then(|v| v.as_i64()).unwrap();
            let r = istemci
                .post(format!("{}/api/denetim/sonuc", taban))
                .header("Authorization", format!("Bearer {}", t2))
                .json(&serde_json::json!({ "gorev_id": rg, "madde_id": rm, "v_int8_b64": vb, "v_min": 0.0, "v_max": 1.0 }))
                .send()
                .await
                .unwrap();
            assert_eq!(r.status(), 200);
            let j: serde_json::Value = r.json().await.unwrap();
            if j.get("gecerli").and_then(|v| v.as_bool()) == Some(true) {
                gecen += 1;
            }
        }
        assert_eq!(gecen, refs.len(), "denetimler gecmeli");
        // Dogrulama DB'ye islendi.
        let dogrulanan: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM kanitlar WHERE dogrulama IS NOT NULL")
                .fetch_one(&state.pool)
                .await
                .unwrap();
        assert!(dogrulanan > 0);
    }

    #[tokio::test]
    async fn test_yetenek_yasam_dongusu() {
        // B23: nabizla kabiliyet kaydi + guncelleme + filo gorunumu.
        let state = test_state().await;
        sqlx::query("INSERT INTO miners (miner_id, token, cuzdan, makine_id, created_at, last_seen) VALUES ('m1', 't1', 'c', 'mk', 1, 9999999999)")
            .execute(&state.pool).await.unwrap();
        // Yetenekli nabiz.
        let r = heartbeat(
            State(state.clone()),
            test_headers("t1"),
            Some(Json(serde_json::from_value(serde_json::json!({
                "parcalar": [], "depolama_kota": 100,
                "yetenek": { "gpu_ad": "Test GPU 11G", "vram_mb": 11264,
                    "roller": ["uretim", "embed", "uretim"] }
            })).unwrap())),
        ).await.unwrap();
        assert!(r.get("ok").and_then(|v| v.as_bool()) == Some(true));
        let row: (String, i64, String) = sqlx::query_as(
            "SELECT gpu_ad, vram_mb, roller FROM miner_yetenek WHERE miner_id = 'm1'",
        )
        .fetch_one(&state.pool).await.unwrap();
        assert_eq!(row.0, "Test GPU 11G");
        assert_eq!(row.1, 11264);
        assert_eq!(row.2, "embed,uretim", "sirali + tekrarsiz");
        // Eski nabiz (yeteneksiz) satiri bozmaz.
        heartbeat(State(state.clone()), test_headers("t1"), None).await.unwrap();
        let sayi: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM miner_yetenek WHERE miner_id = 'm1'")
            .fetch_one(&state.pool).await.unwrap();
        assert_eq!(sayi, 1);
        // Filo listeler.
        let f = filo(State(state.clone()), test_headers("t1"), Query(std::collections::HashMap::new())).await.unwrap().0;
        let liste = f.get("madenci").and_then(|v| v.as_array()).unwrap();
        assert_eq!(liste.len(), 1);
        assert_eq!(liste[0].get("gpu").and_then(|v| v.as_str()), Some("Test GPU 11G"));
        assert_eq!(liste[0].get("roller").and_then(|v| v.as_str()), Some("embed,uretim"));
    }

    #[test]
    fn test_sigorta_sessiz() {
        // B25: normal akis atmaz (60dk x 10/dk).
        let mut k = DevreKesici::yeni();
        let t0 = 1_000_000i64 - (1_000_000i64 % 60);
        for m in 0..60i64 {
            for _ in 0..10 {
                k.kapanis_kaydet(t0 + m * 60);
            }
        }
        assert!(k.dagitim_acik(t0 + 3600));
        assert!(!k.kesik_mi(t0 + 3600));
    }

    #[test]
    fn test_sigorta_atar_ve_doner() {
        // B25: 5dk'da 12000 kapanis (10x + mutlak taban ustu) -> kesik.
        let mut k = DevreKesici::yeni();
        let t0 = 2_000_000i64 - (2_000_000i64 % 60);
        for m in 0..60i64 {
            for _ in 0..10 {
                k.kapanis_kaydet(t0 + m * 60);
            }
        }
        for i in 0..12000i64 {
            k.kapanis_kaydet(t0 + 3600 + (i % 300));
        }
        assert!(!k.dagitim_acik(t0 + 3900));
        assert!(k.kesik_mi(t0 + 3900));
        // Sure dolunca otomatik yari-acik.
        assert!(k.dagitim_acik(t0 + 3900 + 1800));
        assert!(!k.kesik_mi(t0 + 3900 + 1800));
    }

    #[test]
    fn test_bagisiklik_eslestir() {
        // B25: taban, tavan, monotonluk.
        assert_eq!(bagisiklik_eslestir(0.0), (10, 400));
        assert_eq!(bagisiklik_eslestir(0.05), (50, 2000));
        assert_eq!(bagisiklik_eslestir(0.5), (50, 2000));
        assert_eq!(bagisiklik_eslestir(-1.0), (10, 400));
        let (a, _) = bagisiklik_eslestir(0.01);
        let (b, _) = bagisiklik_eslestir(0.02);
        assert!(b >= a && a >= 10);
    }

    #[tokio::test]
    async fn test_toplu_kayit_ve_filo() {
        // B25: toplu kayit (cap dahil) + cuzdan filtreli filo.
        let state = test_state().await;
        let r = register_toplu(
            State(state.clone()),
            Json(TopluKayitIstek { cuzdan: "ciftlik-c".to_string(), adet: Some(5), makine_onek: Some("raf".to_string()) }),
        ).await.unwrap().0;
        assert_eq!(r.get("adet").and_then(|v| v.as_i64()), Some(5));
        let madenciler = r.get("madenciler").and_then(|v| v.as_array()).unwrap();
        assert_eq!(madenciler.len(), 5);
        assert!(madenciler.iter().all(|m| m.get("token").and_then(|v| v.as_str()).map(|s| s.len()).unwrap_or(0) == 32));
        // Cap: 500 istense 200 doner.
        let r = register_toplu(
            State(state.clone()),
            Json(TopluKayitIstek { cuzdan: "ciftlik-c".to_string(), adet: Some(500), makine_onek: None }),
        ).await.unwrap().0;
        assert_eq!(r.get("adet").and_then(|v| v.as_i64()), Some(200));
        // Filo filtresi: cuzdan ciftlik-c -> 205 satir.
        let mut f = std::collections::HashMap::new();
        f.insert("cuzdan".to_string(), "ciftlik-c".to_string());
        // token: ilk madencinin tokeni.
        let tok: String = sqlx::query_scalar("SELECT token FROM miners LIMIT 1")
            .fetch_one(&state.pool).await.unwrap();
        let mut h = HeaderMap::new();
        h.insert("authorization", format!("Bearer {}", tok).parse().unwrap());
        let f = filo(State(state.clone()), h, Query(f)).await.unwrap().0;
        assert_eq!(f.get("sayi").and_then(|v| v.as_i64()), Some(205));
        // Bos cuzdan red.
        let red = register_toplu(
            State(state.clone()),
            Json(TopluKayitIstek { cuzdan: "  ".to_string(), adet: Some(2), makine_onek: None }),
        ).await;
        assert!(red.is_err());
    }

    #[tokio::test]
    async fn test_slash_yasam_dongusu() {
        // Çöp kanıt → 2 bağımsız ret → slash + emanet yanar.
        let state = test_state().await;
        for (mid, tok) in [("m1", "t1"), ("m2", "t2"), ("m3", "t3")] {
            sqlx::query("INSERT INTO miners (miner_id, token, cuzdan, makine_id, created_at) VALUES (?, ?, 'c', 'mk', 1)")
                .bind(mid).bind(tok).execute(&state.pool).await.unwrap();
        }
        sqlx::query("INSERT INTO gorevler (gorev_id, dagitilan_miner, corpus, offset, beklenen, alinan, durum, odul_mikro, ts) VALUES ('test:9', 'm1', 'test', 0, 3, 0, 'acik', 0, 1)")
            .execute(&state.pool).await.unwrap();
        sqlx::query("INSERT INTO kor_esleme (kor_id, madde_id, gorev_id, ts) VALUES (9001, 41, 'test:9', 1)")
            .execute(&state.pool).await.unwrap();
        // m1 çöp basar (3 kanıtla batch'i kapatır; hangisi bayraklı olursa test onu kullanır).
        let cop = test_vektor(200);
        for k in [9001i64, 9002, 9003] {
            sqlx::query("INSERT INTO kor_esleme (kor_id, madde_id, gorev_id, ts) VALUES (?, ?, 'test:9', 1)")
                .bind(k).bind(k).execute(&state.pool).await.unwrap_or_default();
        }
        for k in [9001i64, 9002, 9003] {
            kanit(
                State(state.clone()),
                test_headers("t1"),
                Json(KanitReq { gorev_id: "test:9".to_string(), madde_id: k, v_int8_b64: cop.clone(), v_min: 0.0, v_max: 1.0, imza: None }),
            ).await.unwrap();
        }
        // Bayraklı bir kanıt bul (yoksa zorla işaretle — örnekleyici ayrı testli).
        let hedef: Option<(i64, i64)> = sqlx::query_as(
            "SELECT kor_id, madde_id FROM (SELECT e.kor_id, e.madde_id FROM kor_esleme e JOIN kanitlar k ON k.gorev_id=e.gorev_id AND k.madde_id=e.madde_id WHERE e.gorev_id='test:9' AND k.spot_check=1 AND k.dogrulama IS NULL LIMIT 1)")
            .fetch_optional(&state.pool).await.unwrap();
        let (kor, _gercek) = match hedef {
            Some(t) => t,
            None => {
                sqlx::query("UPDATE kanitlar SET spot_check=1 WHERE gorev_id='test:9' AND madde_id=41")
                    .execute(&state.pool).await.unwrap();
                // emanet satırını da üret (kapanışta yazılırdı).
                sqlx::query("INSERT OR IGNORE INTO escrow (gorev_id, madde_id, miner_id, miktar_mikro, ts) VALUES ('test:9', 41, 'm1', 100, 1)")
                    .execute(&state.pool).await.unwrap();
                (9001i64, 41i64)
            }
        };
        // m1'e biraz coin ver ki slash kesecek bakiye bulsun.
        sqlx::query("UPDATE miners SET coin_mikro=5000 WHERE miner_id='m1'").execute(&state.pool).await.unwrap();
        // 2 bağımsız ret, farklı çöp vektörlerle.
        for (tok, desen) in [("t2", 11u64), ("t3", 77u64)] {
            let r = denetim_sonuc(
                State(state.clone()),
                test_headers(tok),
                Json(DenetimSonuc { gorev_id: "test:9".to_string(), madde_id: kor, v_int8_b64: Some(test_vektor(desen)), v_min: Some(0.0), v_max: Some(1.0), dogrulama: None, gecerli: None }),
            ).await.unwrap();
            let v = r.0;
            assert_eq!(v.get("gecerli").and_then(|x| x.as_bool()), Some(false));
        }
        let strike: i64 = sqlx::query_scalar("SELECT strike FROM miners WHERE miner_id='m1'")
            .fetch_one(&state.pool).await.unwrap();
        assert_eq!(strike, 1, "slash sonrasi strike 1 olmali");
        let slash_var: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ledger WHERE miner_id='m1' AND neden='slash'")
            .fetch_one(&state.pool).await.unwrap();
        assert_eq!(slash_var, 1, "slash deftere islenmeli");
        let emanet_kaldi: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM escrow WHERE gorev_id='test:9' AND madde_id=41")
            .fetch_one(&state.pool).await.unwrap();
        assert_eq!(emanet_kaldi, 0, "hile kesinlesen emanet yanmali");
    }
}
