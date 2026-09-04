// Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
//! models.rs — HF GGUF pull (Ollama benzeri)
//! Önerilen 4 model + serbest HF arama, resume’li indirme

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, OnceLock,
};
static PULL_LOCK: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
static PULL_CANCEL: OnceLock<Mutex<HashMap<String, Arc<AtomicBool>>>> = OnceLock::new();
struct PullGuard(String);
impl Drop for PullGuard {
    fn drop(&mut self) {
        if let Some(lock) = PULL_LOCK.get() {
            if let Ok(mut set) = lock.try_lock() {
                set.remove(&self.0);
            }
        }
        if let Some(map) = PULL_CANCEL.get() {
            if let Ok(mut m) = map.try_lock() {
                m.remove(&self.0);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelInfo {
    pub id: String,           // bartowski/Qwen2.5-3B-Instruct-GGUF
    pub dosya: String,        // qwen2.5-3b-instruct-q4_k_m.gguf
    pub boyut: String,        // ~2.0GB
    pub vram: String,         // 4GB+
    pub aciklama: String,
    pub hf_url: String,
    pub yerel_yol: Option<String>,
    pub indirilmis: bool,
}

/// Önerilen 4 model — plan v2 onaylı
pub fn onerilen_modeller() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "bartowski/Qwen2.5-3B-Instruct-GGUF".into(),
            dosya: "Qwen2.5-3B-Instruct-Q4_K_M.gguf".into(),
            boyut: "~1.93GB".into(),
            vram: "4GB+".into(),
            aciklama: "Önerilen — Türkçe en iyi küçük model".into(),
            hf_url: "https://huggingface.co/bartowski/Qwen2.5-3B-Instruct-GGUF".into(),
            yerel_yol: None,
            indirilmis: false,
        },
        ModelInfo {
            id: "bartowski/gemma-2-2b-it-GGUF".into(),
            dosya: "gemma-2-2b-it-Q4_K_M.gguf".into(),
            boyut: "~1.6GB".into(),
            vram: "3.5GB+".into(),
            aciklama: "Ultra hafif — 4GB GPU’lar".into(),
            hf_url: "https://huggingface.co/bartowski/gemma-2-2b-it-GGUF".into(),
            yerel_yol: None,
            indirilmis: false,
        },
        ModelInfo {
            id: "bartowski/Qwen2.5-7B-Instruct-GGUF".into(),
            dosya: "Qwen2.5-7B-Instruct-Q4_K_M.gguf".into(),
            boyut: "~4.7GB".into(),
            vram: "6GB+".into(),
            aciklama: "Dengeli".into(),
            hf_url: "https://huggingface.co/bartowski/Qwen2.5-7B-Instruct-GGUF".into(),
            yerel_yol: None,
            indirilmis: false,
        },
        ModelInfo {
            id: "bartowski/Qwen2.5-14B-Instruct-GGUF".into(),
            dosya: "Qwen2.5-14B-Instruct-Q4_K_M.gguf".into(),
            boyut: "~8.5GB".into(),
            vram: "10GB+".into(),
            aciklama: "Güçlü — mini öğretmen".into(),
            hf_url: "https://huggingface.co/bartowski/Qwen2.5-14B-Instruct-GGUF".into(),
            yerel_yol: None,
            indirilmis: false,
        },
    ]
}

#[tauri::command]
pub fn list_models() -> Vec<ModelInfo> {
    let mut mods = onerilen_modeller();
    // yerel kontrol: ~/.nemes/models/*.gguf
    if let Some(home) = dirs::home_dir() {
        let base = home.join(".nemes").join("models");
        for m in &mut mods {
            let yol = base.join(&m.dosya);
            if yol.exists() {
                m.indirilmis = true;
                m.yerel_yol = Some(yol.to_string_lossy().into());
            }
        }
    }
    mods
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PullProgress {
    pub model_id: String,
    pub indirilen: u64,
    pub toplam: u64,
    pub hiz: String,
}

/// HF’den GGUF indir (resume’li, stream) — TAM SÜRÜM
#[tauri::command]
pub async fn pull_model(window: tauri::Window, model_id: String) -> Result<String, String> {
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let mods = onerilen_modeller();
    let m = mods
        .iter()
        .find(|x| x.id == model_id)
        .ok_or_else(|| format!("model bulunamadı: {} (önerilen 4 + HF arama kullanın)", model_id))?
        .clone();

    let hf_file_url = format!(
        "https://huggingface.co/{}/resolve/main/{}",
        m.id, m.dosya
    );
    let home = dirs::home_dir().ok_or("home bulunamadı")?;
    let hedef = home.join(".nemes").join("models").join(&m.dosya);
    let part = hedef.with_extension("part");
    std::fs::create_dir_all(hedef.parent().unwrap()).map_err(|e| e.to_string())?;

    // Resume: mevcut .part boyutu
    let mevcut: u64 = std::fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
    if hedef.exists() {
        let _ = window.emit(
            "pull-progress",
            PullProgress {
                model_id: model_id.clone(),
                indirilen: 0,
                toplam: 0,
                hiz: "zaten indirildi".into(),
            },
        );
        return Ok(format!("{} zaten indirildi: {}", model_id, hedef.display()));
    }

    // Concurrent pull guard — aynı model iki kez indirilmesin
    {
        let lock = PULL_LOCK.get_or_init(|| Mutex::new(HashSet::new()));
        let mut set = lock.lock().map_err(|_| "lock zehirlendi")?;
        if !set.insert(model_id.clone()) {
            return Err(format!("{} zaten indiriliyor", model_id));
        }
    }
    let _guard = PullGuard(model_id.clone());
    // Cancel token — UI'dan iptal/duraklat için
    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let map = PULL_CANCEL.get_or_init(|| Mutex::new(HashMap::new()));
        let mut m = map.lock().map_err(|_| "lock zehirlendi")?;
        m.insert(model_id.clone(), cancel_flag.clone());
    }

    let client = reqwest::Client::builder()
        .user_agent("NEMES-Miner/0.2.0")
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| e.to_string())?;

    // HEAD ile toplam boyut al (opsiyonel)
    let toplam: u64 = client
        .head(&hf_file_url)
        .send()
        .await
        .ok()
        .and_then(|r| r.headers().get(reqwest::header::CONTENT_LENGTH)?.to_str().ok()?.parse().ok())
        .unwrap_or(0);

    // Disk limiti — 50GB üstü reddet (DoS koruması)
    if toplam > 50 * 1024 * 1024 * 1024 {
        return Err("dosya cok buyuk (>50GB), reddedildi".into());
    }

    // GET — resume destekli
    let mut req = client.get(&hf_file_url);
    if mevcut > 0 {
        req = req.header(reqwest::header::RANGE, format!("bytes={}-", mevcut));
    }
    let resp = req.send().await.map_err(|e| format!("HF bağlantı hatası: {}", e))?;
    // Resume kontrolü: Range gönderip 200 gelirse server resume desteklemiyor → truncate
    let mut indirilen_baslangic = mevcut;
    let mut file: tokio::fs::File;
    if mevcut > 0 && resp.status().as_u16() == 200 {
        // Server Range’i yok saydı, sıfırdan başla
        file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&part)
            .await
            .map_err(|e| e.to_string())?;
        indirilen_baslangic = 0;
        let _ = window.emit(
            "pull-progress",
            PullProgress {
                model_id: model_id.clone(),
                indirilen: 0,
                toplam,
                hiz: "resume desteklenmiyor, yeniden başlıyor".into(),
            },
        );
    } else if !resp.status().is_success() && resp.status().as_u16() != 206 {
        return Err(format!("HF http {}: {}", resp.status(), hf_file_url));
    } else {
        file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&part)
            .await
            .map_err(|e| e.to_string())?;
    }
    let toplam_gercek = resp
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok()?.parse::<u64>().ok())
        .map(|c| c + indirilen_baslangic)
        .unwrap_or(toplam);
    if toplam_gercek > 50 * 1024 * 1024 * 1024 {
        return Err("toplam boyut cok buyuk (>50GB)".into());
    }

    // Başlangıç emit
    let _ = window.emit(
        "pull-progress",
        PullProgress {
            model_id: model_id.clone(),
            indirilen: indirilen_baslangic,
            toplam: toplam_gercek,
            hiz: if indirilen_baslangic > 0 {
                format!("resume {}MB", indirilen_baslangic / 1024 / 1024)
            } else {
                "başlatılıyor".into()
            },
        },
    );

    let mut stream = resp.bytes_stream();
    let mut indirilen = indirilen_baslangic;
    let mut son_emit = std::time::Instant::now();
    let baslangic = std::time::Instant::now();
    let mut hiz_str = String::new();

    while let Some(chunk) = stream.next().await {
        // Duraklat/İptal kontrolü
        if cancel_flag.load(Ordering::Relaxed) {
            let _ = window.emit(
                "pull-progress",
                PullProgress {
                    model_id: model_id.clone(),
                    indirilen,
                    toplam: toplam_gercek,
                    hiz: "duraklatıldı".into(),
                },
            );
            // part dosyası kalsın — resume edilebilir
            return Err("indirme duraklatıldı — tekrar İndir’e basın resume eder".into());
        }
        let chunk = match chunk {
            Ok(c) => c,
            Err(e) => {
                let msg = e.to_string();
                // Timeout ise resume edilebilir hata olarak dön
                if msg.contains("timed out") || msg.contains("timeout") || msg.contains("operation timed out") {
                    let _ = window.emit(
                        "pull-progress",
                        PullProgress {
                            model_id: model_id.clone(),
                            indirilen,
                            toplam: toplam_gercek,
                            hiz: "bağlantı kesildi — resume edilebilir".into(),
                        },
                    );
                    return Err(format!(
                        "bağlantı kesildi (timeout): {:.1}MB indirildi — tekrar İndir’e basın resume eder (part korundu)",
                        indirilen as f64 / 1024.0 / 1024.0
                    ));
                }
                return Err(format!("indirme hatası: {}", msg));
            }
        };
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        indirilen += chunk.len() as u64;

        // 200ms’de bir emit (throttle)
        if son_emit.elapsed().as_millis() > 200 {
            let gecen = baslangic.elapsed().as_secs_f64().max(1.0);
            let hiz_mb = (indirilen - indirilen_baslangic) as f64 / gecen / 1024.0 / 1024.0;
            hiz_str = format!("{:.1} MB/s", hiz_mb);
            let _ = window.emit(
                "pull-progress",
                PullProgress {
                    model_id: model_id.clone(),
                    indirilen,
                    toplam: toplam_gercek,
                    hiz: hiz_str.clone(),
                },
            );
            son_emit = std::time::Instant::now();
        }
    }
    file.flush().await.map_err(|e| e.to_string())?;
    drop(file);

    // Boyut doğrulama (HF bazen Content-Length vermez, o yüzden sadece varsa kontrol)
    if toplam_gercek > 0 && indirilen != toplam_gercek {
        // Kısa indirme — part kalsın, resume edilebilir
        return Err(format!(
            "eksik indirme: {}/{} byte (resume ile tekrar deneyin)",
            indirilen, toplam_gercek
        ));
    }

    tokio::fs::rename(&part, &hedef)
        .await
        .map_err(|e| format!("rename hata: {}", e))?;

    let _ = window.emit(
        "pull-progress",
        PullProgress {
            model_id: model_id.clone(),
            indirilen,
            toplam: toplam_gercek,
            hiz: "tamamlandı".into(),
        },
    );
    Ok(format!("{} indirildi: {} ({} byte)", model_id, hedef.display(), indirilen))
}

/// Serbest HF arama (kullanıcı istediği GGUF’u arar) — URL encode fix
#[tauri::command]
pub async fn search_hf(query: String) -> Result<Vec<String>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }
    if query.len() > 100 {
        return Err("sorgu cok uzun (max 100)".into());
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("NEMES-Miner/0.2.0")
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get("https://huggingface.co/api/models")
        .query(&[
            ("search", query.as_str()),
            ("filter", "gguf"),
            ("limit", "10"),
            ("sort", "downloads"),
            ("direction", "-1"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HF api http {}", resp.status()));
    }
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let ids = json
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.get("id")?.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    Ok(ids)
}

#[tauri::command]
pub fn remove_model(dosya: String) -> Result<String, String> {
    use std::path::Path;
    // Sadece dosya adı kabul et, path traversal engelle
    let file_name = Path::new(&dosya)
        .file_name()
        .ok_or("gecersiz dosya adi")?
        .to_string_lossy()
        .to_string();
    if file_name != dosya || dosya.contains('/') || dosya.contains('\\') || dosya.contains("..") {
        return Err("gecersiz dosya adi: path traversal algilandi".into());
    }
    // Sadece .gguf ve .part uzantılarına izin ver
    if !file_name.ends_with(".gguf") && !file_name.ends_with(".part") && !file_name.ends_with(".gguf.part") {
        return Err("gecersiz uzanti: sadece .gguf".into());
    }
    let home = dirs::home_dir().ok_or("home yok")?;
    let base = home.join(".nemes").join("models");
    let yol = base.join(&file_name);
    // Canonicalize ile base altında olduğunu doğrula
    let canonical_base = base.canonicalize().unwrap_or(base.clone());
    if let Ok(canonical_yol) = yol.canonicalize() {
        if !canonical_yol.starts_with(&canonical_base) {
            return Err("gecersiz yol: base disinda".into());
        }
    } else {
        // Dosya yoksa, yolun parent’ı base mi kontrol et
        if let Some(parent) = yol.parent() {
            if parent != base && !parent.starts_with(&canonical_base) {
                return Err("gecersiz yol".into());
            }
        }
    }
    let mut silinen = Vec::new();
    if yol.exists() {
        std::fs::remove_file(&yol).map_err(|e| e.to_string())?;
        silinen.push(yol.display().to_string());
    }
    // Yarım .part dosyasını da sil
    let part = yol.with_extension("gguf.part");
    // Alternatif: dosya zaten .gguf ise .part uzantılı companion
    let part2 = base.join(format!("{}.part", file_name));
    for p in [part, part2] {
        if p.exists() && p != yol {
            let _ = std::fs::remove_file(&p);
            silinen.push(p.display().to_string());
        }
    }
    if silinen.is_empty() {
        return Err(format!("dosya bulunamadi: {}", file_name));
    }
    Ok(format!("silindi: {}", silinen.join(", ")))
}

#[tauri::command]
pub fn cancel_pull(model_id: String) -> Result<String, String> {
    if let Some(map) = PULL_CANCEL.get() {
        if let Ok(m) = map.lock() {
            if let Some(flag) = m.get(&model_id) {
                flag.store(true, Ordering::Relaxed);
                return Ok(format!("{} için iptal/duraklat istendi — part korundu, resume edilebilir", model_id));
            }
        }
    }
    Err("aktif indirme yok veya zaten bitti".into())
}

#[tauri::command]
pub fn pause_pull(model_id: String) -> Result<String, String> {
    cancel_pull(model_id)
}
