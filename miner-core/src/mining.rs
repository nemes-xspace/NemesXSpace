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
        let mut sonuclar = Vec::with_capacity(metinler.len());
        for d in json["data"].as_array().unwrap_or(&vec![]) {
            let idx = d["index"].as_u64().unwrap_or(0) as usize;
            let embedding = d["embedding"].as_array()
                .ok_or_else(|| anyhow::anyhow!("embedding array yok"))?
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect();
            sonuclar.push(embedding);
        }
        // Sıralamayı koru
        sonuclar.sort_by_key(|_| 0); // zaten sıralı gelir ama garantisi için
        Ok(sonuclar)
    }
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

        if !resp.status().is_success() {
            if resp.status() == reqwest::StatusCode::NOT_FOUND || resp.status() == reqwest::StatusCode::NO_CONTENT {
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

/// Ana mining döngüsü - görev al -> embed et -> kanıt gönder
pub async fn mining_loop(
    gorev_alici: GorevAlici,
    embed_client: EmbedClient,
    kanit_gonderici: KanitGonderici,
    worker_id: usize,
    gpu_index: usize,
    durum_tx: tokio::sync::mpsc::Sender<WorkerState>,
    durdur_rx: tokio::sync::watch::Receiver<bool>,
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
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
            Err(e) => {
                eprintln!("[worker-{}] görev alma hatası: {}", worker_id, e);
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                continue;
            }
        };

        // Görev tipine göre işle
        let (vektorler, madde_idler, denetim_refs) = match gorev.tip.as_str() {
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

                // Embed et
                let embeddings = embed_client.embed_batch(&metinler).await?;

                // Madde ID'leri payload'dan al veya index olarak kullan
                let madde_idler: Vec<usize> = payload.as_ref()
                    .and_then(|p| p.get("madde_idler"))
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_u64().map(|u| u as usize)).collect())
                    .unwrap_or_else(|| (0..metinler.len()).collect());

                (embeddings, madde_idler, None)
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

                let embeddings = embed_client.embed_batch(&metinler).await?;
                (embeddings, madde_idler, Some(refs))
            }
            "rag" => {
                let payload = gorev.payload.as_ref().ok_or_else(|| anyhow::anyhow!("RAG payload eksik"))?;
                let query = payload.get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("RAG sorgusu eksik"))?;
                
                let embeddings = embed_client
                    .clone()
                    .with_query_prefix()
                    .embed_batch(&[query.to_string()])
                    .await?;
                
                let madde_idler = vec![0]; // RAG için tek sonuç
                (embeddings, madde_idler, None)
            }
            _ => {
                eprintln!("[worker-{}] bilinmeyen görev tipi: {}", worker_id, gorev.tip);
                continue;
            }
        };

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
