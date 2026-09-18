// Copyright 2026 NEMES-X. SPDX-License-Identifier: Apache-2.0.
//! Depolama katmani kabulu (C4): 100GB baraji + disk sinavi + taahhut dosyasi.
//!
//! Kural: sozle degil, sinavla. Miner `--storage` ile kaydolurken komuta
//! (sonraki fazda) su ucunu ister: kimlik pubkey'i + taahhut dosyasi +
//! sinav hash'i. Sinav dosyasi is bitince silinir, disk bos kalir.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Depolama katmani minimum kota: 100 GiB.
pub const MIN_KOTA_BYTE: u64 = 100 * 1024 * 1024 * 1024;
/// Varsayilan sinav boyutu: 1 GiB yaz + geri oku + dogrula.
pub const SINAV_VARSAYILAN_MB: u64 = 1024;
/// Taahhut dosyasi adi (depolama dizini icinde).
pub const TAAHHUT_DOSYASI: &str = "miner-depolama.json";

/// Varsayilan depolama dizini: ~/.nemes/depolama
pub fn varsayilan_depolama_yolu() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".nemes")
        .join("depolama")
}

/// Taahhut dosyasi icerigi (gizli degil — sadece beyan).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Taahhut {
    pub yol: String,
    pub kota_byte: u64,
    pub pubkey_b64: String,
    pub sinav_hash: String,
    pub ts_ms: i64,
}

/// Verilen yolun bulundugu dosya sistemindeki BOS alani dondurur.
/// En uzun eslesen baglama noktasi secilir.
pub fn bos_alan_byte(yol: &Path) -> anyhow::Result<u64> {
    use sysinfo::Disks;
    let hedef = yol
        .canonicalize()
        .unwrap_or_else(|_| yol.to_path_buf());
    let hedef_s = hedef.to_string_lossy().to_string();
    let disks = Disks::new_with_refreshed_list();
    let mut en_iyi: Option<(usize, u64)> = None;
    for d in disks.list() {
        let bag = d.mount_point().to_string_lossy().to_string();
        if hedef_s == bag || hedef_s.starts_with(&format!("{}/", bag.trim_end_matches('/'))) || bag == "/" {
            let skor = bag.len();
            let bos = d.available_space();
            match en_iyi {
                Some((s, _)) if s >= skor => {}
                _ => en_iyi = Some((skor, bos)),
            }
        }
    }
    en_iyi
        .map(|(_, b)| b)
        .ok_or_else(|| anyhow::anyhow!("disk bulunamadi: {}", yol.display()))
}

/// Disk sinavi: `boyut_mb` rastgele bayt yaz, geri oku, blake3 karsilastir.
/// Basariliysa dosya silinir, hex hash dondurulur.
pub fn sinav(yol: &Path, boyut_mb: u64) -> anyhow::Result<String> {
    use std::io::{Read, Write};
    if boyut_mb == 0 || boyut_mb > 10240 {
        anyhow::bail!("sinav boyutu 1-10240 MB olmali: {}", boyut_mb);
    }
    std::fs::create_dir_all(yol)?;
    let dosya = yol.join(".nemes-sinav.tmp");
    let toplam = boyut_mb * 1024 * 1024;
    let mut yazici = blake3::Hasher::new();
    {
        let mut f = std::fs::File::create(&dosya)?;
        let mut kalan = toplam;
        let mut blok = vec![0u8; 1024 * 1024];
        while kalan > 0 {
            let n = kalan.min(blok.len() as u64) as usize;
            getrandom::getrandom(&mut blok[..n])?;
            f.write_all(&blok[..n])?;
            yazici.update(&blok[..n]);
            kalan -= n as u64;
        }
        f.sync_all()?;
    }
    let yazilan = yazici.finalize();
    // Geri oku + dogrula.
    let mut okuyucu = blake3::Hasher::new();
    {
        let mut f = std::fs::File::open(&dosya)?;
        let mut blok = vec![0u8; 1024 * 1024];
        loop {
            let n = f.read(&mut blok)?;
            if n == 0 {
                break;
            }
            okuyucu.update(&blok[..n]);
        }
    }
    let okunan = okuyucu.finalize();
    std::fs::remove_file(&dosya)?;
    if yazilan != okunan {
        anyhow::bail!("sinav basarisiz: yazilan != okunan (disk bozuk olabilir)");
    }
    Ok(yazilan.to_hex().to_string())
}

/// Taahhut dosyasini yazar, yolunu dondurur.
pub fn taahhut_yaz(depolama_yolu: &Path, kota_byte: u64, pubkey_b64: &str, sinav_hash: &str, ts_ms: i64) -> anyhow::Result<PathBuf> {
    let t = Taahhut {
        yol: depolama_yolu.to_string_lossy().to_string(),
        kota_byte,
        pubkey_b64: pubkey_b64.to_string(),
        sinav_hash: sinav_hash.to_string(),
        ts_ms,
    };
    let yol = depolama_yolu.join(TAAHHUT_DOSYASI);
    std::fs::write(&yol, serde_json::to_string_pretty(&t)?)?;
    Ok(yol)
}

/// Taahhut dosyasini okur (yoksa None).
pub fn taahhut_oku(depolama_yolu: &Path) -> Option<Taahhut> {
    let ham = std::fs::read_to_string(depolama_yolu.join(TAAHHUT_DOSYASI)).ok()?;
    serde_json::from_str(&ham).ok()
}

pub fn gb(byte: u64) -> f64 {
    byte as f64 / 1024.0 / 1024.0 / 1024.0
}

/// Parca dosyasinin [offset, offset+uzunluk) araliginin blake3 hex'i (C8 yoklama cevabi).
/// Dosya adi 64-hex olmali, aralik dosya icinde olmali.
pub fn parca_hash_aralik(depolama_yolu: &Path, hash: &str, offset: u64, uzunluk: u64) -> anyhow::Result<String> {
    use std::io::{Read, Seek, SeekFrom};
    let h = hash.trim();
    if h.len() != 64 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
        anyhow::bail!("gecersiz parca hash");
    }
    if uzunluk == 0 || uzunluk > 64 * 1024 * 1024 {
        anyhow::bail!("gecersiz aralik uzunlugu: {}", uzunluk);
    }
    let yol = depolama_yolu.join(h);
    let mut f = std::fs::File::open(&yol)?;
    let boyut = f.metadata()?.len();
    if offset.checked_add(uzunluk).is_none() || offset + uzunluk > boyut {
        anyhow::bail!("aralik dosya disinda (boyut {})", boyut);
    }
    f.seek(SeekFrom::Start(offset))?;
    let mut hasher = blake3::Hasher::new();
    let mut kalan = uzunluk;
    let mut blok = vec![0u8; 1024 * 1024];
    while kalan > 0 {
        let n = kalan.min(blok.len() as u64) as usize;
        f.read_exact(&mut blok[..n])?;
        hasher.update(&blok[..n]);
        kalan -= n as u64;
    }
    Ok(hasher.finalize().to_hex().to_string())
}

/// Depolama dizinindeki parca dosyalarini listele.
/// Kural (C7 ile ortak): dosya adi = 64 karakter hex parca_hash.
/// Donen: (parca_hash, boyut_byte).
pub fn parca_listele(depolama_yolu: &Path) -> Vec<(String, i64)> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(depolama_yolu) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for en in entries.flatten() {
        let ad = en.file_name().to_string_lossy().to_string();
        if ad.len() != 64 || !ad.chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }
        let boyut = en.metadata().map(|m| m.len() as i64).unwrap_or(0);
        if boyut > 0 {
            out.push((ad, boyut));
        }
        if out.len() >= 4096 {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sinav_kucuk_boyut() {
        let dir = std::env::temp_dir().join(format!("nemes-sinav-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let h = sinav(&dir, 1).expect("1MB sinav gecmeli");
        assert_eq!(h.len(), 64);
        assert!(!dir.join(".nemes-sinav.tmp").exists(), "sinav dosyasi silinmeli");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sinav_gecersiz_boyut() {
        let dir = std::env::temp_dir();
        assert!(sinav(&dir, 0).is_err());
        assert!(sinav(&dir, 10241).is_err());
    }

    #[test]
    fn taahhut_yaz_oku() {
        let dir = std::env::temp_dir().join(format!("nemes-taahhut-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        taahhut_yaz(&dir, MIN_KOTA_BYTE, "AAA=", "abc123", 1).unwrap();
        let t = taahhut_oku(&dir).expect("taahhut okunmali");
        assert_eq!(t.kota_byte, MIN_KOTA_BYTE);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn bos_alan_okunur() {
        let b = bos_alan_byte(&std::env::temp_dir()).expect("bos alan okunmali");
        assert!(b > 0);
    }

    #[test]
    fn aralik_hash_dogrulugu() {
        use std::io::Write;
        let dir = std::env::temp_dir().join(format!("nemes-aralik-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let h64 = "b".repeat(64);
        let mut f = std::fs::File::create(dir.join(&h64)).unwrap();
        f.write_all(&[7u8; 3000]).unwrap();
        drop(f);
        let h = parca_hash_aralik(&dir, &h64, 100, 500).unwrap();
        let mut bek = blake3::Hasher::new();
        bek.update(&[7u8; 500]);
        assert_eq!(h, bek.finalize().to_hex().to_string());
        assert!(parca_hash_aralik(&dir, &h64, 2900, 500).is_err(), "tasmalı aralik reddedilmeli");
        assert!(parca_hash_aralik(&dir, "zzz", 0, 10).is_err(), "bozuk hash reddedilmeli");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parca_listele_filtreler() {
        use std::io::Write;
        let dir = std::env::temp_dir().join(format!("nemes-parca-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let h64 = "a".repeat(64);
        let mut f = std::fs::File::create(dir.join(&h64)).unwrap();
        f.write_all(b"veri").unwrap();
        std::fs::write(dir.join("miner-depolama.json"), "{}").unwrap();
        std::fs::write(dir.join(".nemes-sinav.tmp"), "x").unwrap();
        let l = parca_listele(&dir);
        assert_eq!(l.len(), 1);
        assert_eq!(l[0], (h64, 4));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
