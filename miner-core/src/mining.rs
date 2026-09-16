// Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
use serde::{Deserialize, Serialize};
use std::time::Duration;
use base64::{engine::general_purpose::STANDARD as B64_STANDARD, Engine as _};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerState {
    pub id: usize,
    pub gpu: usize,
    pub vectors_per_sec: f32,
    pub pay_per_hour: u32,
    pub api_hakki: u64,
    pub gorev: String,
    pub progress_pct: u8,
    pub hiz_mb: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningStats {
    pub workers: Vec<WorkerState>,
    pub toplam_vectors_per_sec: f32,
    pub toplam_pay: u64,
    pub toplam_api_hakki: u64,
}

/// Görev yapısı - komuta API'den gelen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gorev {
    pub id: String,
    pub tip: String,           // "embed" | "rag"
    pub corpus: String,        // "newscrawl_tr" | "gut_en" | "wet_en"
    pub shard: usize,
    pub offset: usize,
    pub limit: usize,
    pub deadline: u64,         // unix timestamp
    pub payload: Option<serde_json::Value>,  // metinler / sorgu
}

/// Kanıt yapısı - komuta API'ye gönderilecek
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kanit {
    pub gorev_id: String,
    pub madde_id: usize,
    pub v_int8_b64: String,    // base64 encoded int8 vektör
    pub v_min: f32,
    pub v_max: f32,
    pub imza: Option<String>,  // Ed25519 imza (Faz2)
}

/// Denetim sonucu yanıtı - komuta kosinüsü kendisi hesaplar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenetimSonucResp {
    pub ok: bool,
    pub gecerli: bool,
    pub dogrulama: f32,
}

/// Embedding istemcisi - llama-server HTTP API
#[derive(Clone)]
pub struct EmbedClient {
    pub base_url: String,
    pub model: String,
    pub prefix: String,        // "search_document: " veya "search_query: "
    client: reqwest::Client,
    timeout: Duration,
}

impl EmbedClient {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        let base_url: String = base_url.into();
        let model: String = model.into();
        let client = reqwest::Client::builder()
            .user_agent(MINER_USER_AGENT)
            .timeout(Duration::from_secs(120))
            .build()
            .expect("reqwest client oluşturulamadı");
        Self {
            base_url,
            model,
            prefix: "search_document: ".to_string(),
            client,
            timeout: Duration::from_secs(120),
        }
    }

    pub fn with_query_prefix(mut self) -> Self {
        self.prefix = "search_query: ".to_string();
        self
    }

    /// Tek batch embedding - nomic-embed-text-v1.5 formatında
    pub async fn embed_batch(&self, metinler: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
        let url = format!("{}/v1/embeddings", self.base_url);
        let input: Vec<String> = metinler.iter().map(|m| format!("{}{}", self.prefix, m)).collect();
        let body = serde_json::json!({
            "model": self.model,
            "input": input
        });

        let resp = self.client
            .post(&url)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Embedding HTTP {}: {}", status, err_text));
        }

        let json: serde_json::Value = resp.json().await?;
        // BUG-1 duzeltmesi: siraya guvenme, `index` alanina yerlestir.
        // Eksik/hatali index = veri hizalanma bozulmasi demek, sessizce gecme.
        let mut sonuclar: Vec<Vec<f32>> = vec![Vec::new(); metinler.len()];
        for d in json["data"].as_array().cloned().unwrap_or_default() {
            let idx = d.get("index").and_then(|v| v.as_u64()).unwrap_or(usize::MAX as u64) as usize;
            if idx >= sonuclar.len() {
                return Err(anyhow::anyhow!("embedding index aralik disi: {} (beklenen < {})", idx, sonuclar.len()));
            }
            let embedding: Vec<f32> = d.get("embedding")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("embedding array yok (index {})", idx))?
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect();
            if embedding.is_empty() {
                return Err(anyhow::anyhow!("bos embedding (index {})", idx));
            }
            sonuclar[idx] = embedding;
        }
        if sonuclar.iter().any(|v| v.is_empty()) {
            return Err(anyhow::anyhow!("eksik embedding: {}/{} geldi", sonuclar.iter().filter(|v| !v.is_empty()).count(), sonuclar.len()));
        }
        Ok(sonuclar)
    }
}

/// Komuta-yonu HTTP istemcilerin User-Agent'i (CF bot korumasindan gecis + taninirlik).
/// NOT: kalici cozum CF tarafinda /api/* WAF-muafiyetidir (operator); bu baslik
/// ikinci savunmadir. Localhost embed trafiginde etkisi yok.
pub const MINER_USER_AGENT: &str = "NEMES-Miner/0.2 (testnet; +https://nemes-x.space)";

/// BUG-2 duzeltmesi: gecici hatalarda ustel beklemeli retry (max 4 deneme).
/// 500 (ctx-asimi) retry ile duzelmez -> hemen doner (caller atlanan'a isler
/// veya gorevi atlar). Donus Err ise caller `?` ile OLMEZ, gorevi atlar.
pub async fn embed_retry(client: &EmbedClient, metinler: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
    let mut bekle = Duration::from_secs(2);
    for deneme in 0..4 {
        match client.embed_batch(metinler).await {
            Ok(v) => return Ok(v),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("HTTP 500") {
                    return Err(e); // ctx-asimi kalici, retry faydasiz
                }
                if deneme == 3 {
                    return Err(e);
                }
                eprintln!("[embed] hata (deneme {}/4): {} - {:?} sonra tekrar", deneme + 1, msg, bekle);
                tokio::time::sleep(bekle).await;
                bekle *= 2;
            }
        }
    }
    unreachable!("retry dongusu dustu")
}

/// Görev alıcı - komuta API'den görev çeker
#[derive(Clone)]
pub struct GorevAlici {
    pub base_url: String,
    pub token: String,
    client: reqwest::Client,
}

impl GorevAlici {
    pub fn new(base_url: impl Into<String>, token: impl Into<String>) -> Self {
        let base_url: String = base_url.into();
        let client = reqwest::Client::builder()
            .user_agent(MINER_USER_AGENT)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest client");
        Self { base_url, token: token.into(), client }
    }

    /// Sıradaki görevi al
    pub async fn gorev_al(&self) -> anyhow::Result<Option<Gorev>> {
        let url = format!("{}/api/gorev", self.base_url.trim_end_matches('/'));
        let resp = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        // 204 bos govdelidir (is bitmis korpus) — json parse'tan ONCE yakala.
        // (204, reqwest'te is_success() sayilir; onceki kod bosa json cozmeye
        // calisip "EOF while parsing" hatasiyla boguluyordu.)
        if resp.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(None); // görev yok, sakin bekle
        }
        if !resp.status().is_success() {
            if resp.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(None); // görev yok
            }
            return Err(anyhow::anyhow!("gorev http {}", resp.status()));
        }

        let gorev: Gorev = resp.json().await?;
        Ok(Some(gorev))
    }
}

/// Kanıt gönderici
#[derive(Clone)]
pub struct KanitGonderici {
    pub base_url: String,
    pub token: String,
    client: reqwest::Client,
}

impl KanitGonderici {
    pub fn new(base_url: impl Into<String>, token: impl Into<String>) -> Self {
        let base_url: String = base_url.into();
        let client = reqwest::Client::builder()
            .user_agent(MINER_USER_AGENT)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest client");
        Self { base_url, token: token.into(), client }
    }

    pub async fn kanit_gonder(&self, kanit: &Kanit) -> anyhow::Result<()> {
        let url = format!("{}/api/kanit", self.base_url.trim_end_matches('/'));
        let resp = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(kanit)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("kanit http {}: {}", status, err));
        }
        Ok(())
    }

    /// Parça indir (C7 onarım): relay'den ham baytlar. Boyut doğrulanmaz burada.
    pub async fn parca_indir(&self, hash: &str) -> anyhow::Result<Vec<u8>> {
        let url = format!("{}/api/parca/indir/{}", self.base_url.trim_end_matches('/'), hash.trim());
        let resp = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("parca http {}: {}", status, err));
        }
        Ok(resp.bytes().await?.to_vec())
    }

    /// Denetim sonucu gönder - taze embedding ile (kosinüsü komuta hesaplar)
    pub async fn denetim_gonder(
        &self,
        gorev_id: &str,
        madde_id: i64,
        v_int8_b64: String,
        v_min: f32,
        v_max: f32,
    ) -> anyhow::Result<DenetimSonucResp> {
        let url = format!("{}/api/denetim/sonuc", self.base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "gorev_id": gorev_id,
            "madde_id": madde_id,
            "v_int8_b64": v_int8_b64,
            "v_min": v_min,
            "v_max": v_max,
        });
        let resp = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("denetim http {}: {}", status, err));
        }
        let out: DenetimSonucResp = resp.json().await?;
        Ok(out)
    }
}

/// R4/P2P: bosta beklerken uyandi sinyaline de kulak ver (yoksa eski sabit uyku).
/// Sinyal (veya kanal kapanmasi) gelirse sure dolmadan doner; dongu basi
/// hemen `gorev_al` yapar. Timeout da normaldir (yeni poll turu).
async fn bosta_bekle(sure: Duration, uyan_rx: &mut Option<tokio::sync::watch::Receiver<u64>>) {
    match uyan_rx {
        Some(rx) => {
            let _ = tokio::time::timeout(sure, rx.changed()).await;
        }
        None => tokio::time::sleep(sure).await,
    }
}

/// Ana mining döngüsü - görev al -> embed et -> kanıt gönder
///
/// `uyan_rx`: R4/P2P uyandirma sayaci (komuta `nemes/gorev` duyurusu).
/// None ise eski davranis (sabit 5/10sn uyku). Some ise bosta beklerken
/// duyuru gelirse hemen uyanip HTTP'den gorevi ceker.
pub async fn mining_loop(
    gorev_alici: GorevAlici,
    embed_client: EmbedClient,
    kanit_gonderici: KanitGonderici,
    worker_id: usize,
    gpu_index: usize,
    durum_tx: tokio::sync::mpsc::Sender<WorkerState>,
    durdur_rx: tokio::sync::watch::Receiver<bool>,
    depolama_dir: Option<std::path::PathBuf>,
    mut uyan_rx: Option<tokio::sync::watch::Receiver<u64>>,
) -> anyhow::Result<()> {
    let mut islenen = 0u64;
    let baslangic = std::time::Instant::now();

    loop {
        // Durdur sinyali kontrolü
        if *durdur_rx.borrow() {
            break;
        }

        // 1. Görev al
        let gorev = match gorev_alici.gorev_al().await {
            Ok(Some(g)) => g,
            Ok(None) => {
                bosta_bekle(Duration::from_secs(5), &mut uyan_rx).await;
                continue;
            }
            Err(e) => {
                eprintln!("[worker-{}] görev alma hatası: {}", worker_id, e);
                bosta_bekle(Duration::from_secs(10), &mut uyan_rx).await;
                continue;
            }
        };

        // Görev tipine göre işle.
        // Donen: (vektorler, madde_idler, denetim_refs, yedek_refs).
        let (vektorler, madde_idler, denetim_refs, yedek_refs) = match gorev.tip.as_str() {
            "embed" => {
                // Payload'u klonla ki birden fazla erişim yapabilelim
                let payload = gorev.payload.clone();

                // Metinleri payload'dan al
                let metinler: Vec<String> = payload.as_ref()
                    .and_then(|p| p.get("metinler"))
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();

                if metinler.is_empty() {
                    eprintln!("[worker-{}] boş metin listesi, atlanıyor", worker_id);
                    continue;
                }

                // Embed et (retry'li; kalici hata -> gorevi atla, OLME)
                let embeddings = match embed_retry(&embed_client, &metinler).await {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("[worker-{}] embed basarisiz, gorev atlaniyor: {}", worker_id, e);
                        continue;
                    }
                };

                // Madde ID'leri payload'dan al veya index olarak kullan
                let madde_idler: Vec<usize> = payload.as_ref()
                    .and_then(|p| p.get("madde_idler"))
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_u64().map(|u| u as usize)).collect())
                    .unwrap_or_else(|| (0..metinler.len()).collect());

                (embeddings, madde_idler, None, None)
            }
            "denetim" => {
                // Es-dogrulama: ayni metinleri yeniden embed et, taze vektorle raporla.
                // Kosinusu komuta hesaplar (denetciye guvenilmez).
                let payload = gorev.payload.clone().ok_or_else(|| anyhow::anyhow!("denetim payload eksik"))?;
                let metinler: Vec<String> = payload.get("metinler")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                let madde_idler: Vec<usize> = payload.get("madde_idler")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_u64().map(|u| u as usize)).collect())
                    .unwrap_or_default();
                let refs: Vec<(String, i64)> = payload.get("denetim")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|o| {
                        let g = o.get("gorev_id")?.as_str()?.to_string();
                        let m = o.get("madde_id")?.as_i64()?;
                        Some((g, m))
                    }).collect())
                    .unwrap_or_default();

                if metinler.is_empty() || refs.is_empty() || metinler.len() != refs.len() {
                    eprintln!("[worker-{}] bozuk denetim gorevi (metin={}, ref={}), atlanıyor", worker_id, metinler.len(), refs.len());
                    continue;
                }

                let embeddings = match embed_retry(&embed_client, &metinler).await {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("[worker-{}] denetim embed basarisiz, atlaniyor: {}", worker_id, e);
                        continue;
                    }
                };
                (embeddings, madde_idler, Some(refs), None)
            }
            "rag" => {
                let payload = gorev.payload.as_ref().ok_or_else(|| anyhow::anyhow!("RAG payload eksik"))?;
                let query = payload.get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("RAG sorgusu eksik"))?;
                
                let embeddings = match embed_retry(
                    &embed_client.clone().with_query_prefix(),
                    &[query.to_string()],
                )
                .await
                {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("[worker-{}] RAG embed basarisiz, atlaniyor: {}", worker_id, e);
                        continue;
                    }
                };
                
                let madde_idler = vec![0]; // RAG için tek sonuç
                (embeddings, madde_idler, None, None)
            }
            "yedekle" => {
                // C7 onarim: relay'den parca indir, depolama dizinine hash adiyla yaz.
                // Dogrulama komutada (sonraki heartbeat raporlar, hash eslesirse onarim kapanir).
                let payload = gorev.payload.clone().ok_or_else(|| anyhow::anyhow!("yedek payload eksik"))?;
                let refs: Vec<(String, i64, i64)> = payload.get("yedek")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|o| {
                        let h = o.get("parca_hash")?.as_str()?.to_string();
                        if h.len() != 64 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
                            return None;
                        }
                        let b = o.get("boyut")?.as_i64()?;
                        let oid = o.get("onarim_id")?.as_i64()?;
                        Some((h, b, oid))
                    }).collect())
                    .unwrap_or_default();
                if refs.is_empty() {
                    eprintln!("[worker-{}] bos yedek gorevi, atlanıyor", worker_id);
                    continue;
                }
                (Vec::new(), Vec::new(), None, Some(refs))
            }
            _ => {
                eprintln!("[worker-{}] bilinmeyen görev tipi: {}", worker_id, gorev.tip);
                continue;
            }
        };

        // Yedek kolu: indir -> depolama dizinine yaz (ucret heartbeat kapanisinda).
        if let Some(yrefs) = yedek_refs {
            match depolama_dir.as_ref() {
                None => {
                    eprintln!("[worker-{}] yedek gorevi geldi ama depolama dizini yok (--depolama ver)", worker_id);
                }
                Some(ddir) => {
                    if let Err(e) = std::fs::create_dir_all(ddir) {
                        eprintln!("[worker-{}] depolama dizini acilamadi: {}", worker_id, e);
                    } else {
                        for (h, boyut, oid) in yrefs {
                            match kanit_gonderici.parca_indir(&h).await {
                                Ok(veri) => {
                                    if veri.len() as i64 != boyut {
                                        eprintln!("[worker-{}] parca boyut uyusmadi (onarim {}): beklenen {} gelen {}", worker_id, oid, boyut, veri.len());
                                        continue;
                                    }
                                    let yol = ddir.join(&h);
                                    match std::fs::write(&yol, &veri) {
                                        Ok(()) => {
                                            islenen += 1;
                                            eprintln!("[worker-{}] parca indi: {} ({} bayt, onarim {})", worker_id, &h[..12], veri.len(), oid);
                                        }
                                        Err(e) => eprintln!("[worker-{}] parca yazilamadi: {}", worker_id, e),
                                    }
                                }
                                Err(e) => eprintln!("[worker-{}] parca indirme hatasi: {}", worker_id, e),
                            }
                        }
                    }
                }
            }
            // Durum gonderimi icin asagidaki ortak bloga dus.
        } else
        // Denetim kolu: taze vektorle sonuc raporla (kosinus komutada)
        if let Some(refs) = denetim_refs {
            for ((vektor, _madde_id), (ref_gorev, ref_madde)) in vektorler.into_iter().zip(madde_idler.into_iter()).zip(refs.into_iter()) {
                if vektor.is_empty() {
                    eprintln!("[worker-{}] denetimde boş vektör atlanıyor", worker_id);
                    continue;
                }
                let (v_int8, v_min, v_max) = crate::int8_nicele(&vektor);
                let v_int8_b64 = B64_STANDARD.encode(&v_int8);
                match kanit_gonderici.denetim_gonder(&ref_gorev, ref_madde, v_int8_b64, v_min, v_max).await {
                    Ok(r) => {
                        islenen += 1;
                        eprintln!("[worker-{}] denetim {} madde={} gecerli={} cos={:.4}", worker_id, ref_gorev, ref_madde, r.gecerli, r.dogrulama);
                    }
                    Err(e) => {
                        eprintln!("[worker-{}] denetim gönderme hatası: {}", worker_id, e);
                    }
                }
            }
            // Durum gonderimi icin asagidaki ortak bloga dus (kanit dongusunu atla)
        } else {
        // Her vektör için kanıt gönder
        for (i, (vektor, madde_id)) in vektorler.into_iter().zip(madde_idler.into_iter()).enumerate() {
            if vektor.is_empty() {
                eprintln!("[worker-{}] boş vektör atlanıyor", worker_id);
                continue;
            }

            // int8 nicele
            let (v_int8, v_min, v_max) = crate::int8_nicele(&vektor);
            let v_int8_b64 = B64_STANDARD.encode(&v_int8);

            // Kanıt oluştur
            let kanit = Kanit {
                gorev_id: gorev.id.clone(),
                madde_id,
                v_int8_b64,
                v_min,
                v_max,
                imza: None, // Faz2'de Ed25519 eklenecek
            };

            // Kanıt gönder
            if let Err(e) = kanit_gonderici.kanit_gonder(&Kanit {
                gorev_id: gorev.id.clone(),
                madde_id,
                v_int8_b64: kanit.v_int8_b64.clone(),
                v_min: kanit.v_min,
                v_max: kanit.v_max,
                imza: None,
            }).await {
                eprintln!("[worker-{}] kanıt gönderme hatası: {}", worker_id, e);
            } else {
                islenen += 1;
            }
        }
        } // else (embed/rag kanit kolu) sonu

        // Durum güncelle ve gönder
        let gecen = baslangic.elapsed().as_secs_f32().max(1.0);
        let vectors_per_sec = islenen as f32 / gecen.max(1.0);

        let state = crate::mining::WorkerState {
            id: worker_id,
            gpu: gpu_index,
            vectors_per_sec,
            pay_per_hour: (3600.0 * vectors_per_sec) as u32,
            api_hakki: 0, // API'den güncellenecek
            gorev: format!("{} shard {}", gorev.corpus, gorev.shard),
            progress_pct: 0, // ilerleme % hesaplanacak
            hiz_mb: 0.0,
        };

        let _ = durum_tx.send(state).await;

        // Rate limit - CPU'yu yormamak için
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}

/// wiki_embed_par.py:44 ile birebir aynı niceleme (clamp'li)
pub fn int8_nicele(vektor: &[f32]) -> (Vec<u8>, f32, f32) {
    if vektor.is_empty() {
        return (vec![], 0.0, 0.0);
    }
    let lo = vektor.iter().cloned().fold(f32::INFINITY, f32::min);
    let hi = vektor.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let hi = if (hi - lo).abs() < 1e-9 { lo + 1e-9 } else { hi };
    let skala = 255.0 / (hi - lo);
    let q = vektor
        .iter()
        .map(|v| ((v - lo) * skala).round().clamp(0.0, 255.0) as u8)
        .collect();
    (q, lo, hi)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_int8_roundtrip() {
        let v = vec![0.0, 0.5, 1.0];
        let (q, lo, hi) = int8_nicele(&v);
        assert_eq!(q.len(), 3);
        assert!(lo <= 0.0);
        assert!(hi >= 1.0);
        assert_eq!(q[0], 0);
        assert_eq!(q[2], 255);
    }
    #[test]
    fn test_int8_empty() {
        let (q, lo, hi) = int8_nicele(&[]);
        assert!(q.is_empty());
        assert_eq!(lo, 0.0);
    }
}
