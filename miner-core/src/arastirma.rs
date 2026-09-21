// Copyright 2026 NEMES-X. SPDX-License-Identifier: Apache-2.0.
//! PARANOID-MINER EPIC-04 (Faz C): güvenli araştırma iskeleti.
//!
//! Faz C-2: egress kurallı fetch AÇIK (allowlist + robots + rate + cap).
//! Üretim mining döngüsüne DOKUNMAZ (ayrı modül, ayrı test).

use std::collections::HashSet;
use std::time::Duration;

/// Kurallar (STORY-04.2 + 04.6): istek arası bekleme, zaman aşımı, gövde tavanı.
pub const ISTEK_ARASI_SN: u64 = 2;
pub const ZAMAN_ASIMI_SN: u64 = 15;
pub const GOVDE_TAVAN_BAYT: usize = 2_000_000;

/// İzinli kök alanlar (STORY-04.1). Dışındakiler red.
pub fn izinli_kokler() -> HashSet<String> {
    [
        "wikipedia.org",
        "wikimedia.org",
        "archive.org",
        "arxiv.org",
        "edu.tr",
        "gov.tr",
        "ac.uk",
        "edu",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// URL izinli mi? (alt alan adı dahil: tr.wikipedia.org ok)
pub fn url_izinli_mi(url: &str) -> bool {
    let alt = url.to_lowercase();
    // şema + host çıkar (kaba ama yeterli, Faz C-2'de url crate gelecek)
    let host = alt
        .split("://")
        .nth(1)
        .unwrap_or(&alt)
        .split('/')
        .next()
        .unwrap_or("")
        .trim_start_matches("www.");
    izinli_kokler().iter().any(|kok| host == kok || host.ends_with(&format!(".{}", kok)))
}

/// Provenance kaydı (STORY-02.5 + 04.5): her çıktıda zorunlu.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Provenance {
    pub url: String,
    pub erisim_ts: i64,
    pub lisans: String,
    pub icerik_hash: String,
    pub guven: f32, // 0.0-1.0 kaynak güvenilirlik skoru (stub: allowlist=0.8)
}

pub fn provenance_uret(url: &str, icerik: &[u8]) -> Provenance {
    let hash = blake3::hash(icerik);
    Provenance {
        url: url.to_string(),
        erisim_ts: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
        lisans: "bilinmiyor-toplandı".to_string(),
        icerik_hash: hex::encode(hash.as_bytes()),
        guven: if url_izinli_mi(url) { 0.8 } else { 0.0 },
    }
}

/// robots.txt ham metninde bu path'e izin var mı? (kaba uyum: User-agent:* + Disallow öneki)
/// robots çekilemezse fail-secure: false (çekme).
pub fn robots_izinli_mi(robots_txt: &str, path: &str) -> bool {
    let mut genel_disallow: Vec<String> = Vec::new();
    let mut ilgili_bolum = false;
    let mut ilk_bolum = true;
    for satir in robots_txt.lines() {
        let s = satir.trim();
        if s.is_empty() || s.starts_with('#') {
            continue;
        }
        let kucuk = s.to_lowercase();
        if kucuk.starts_with("user-agent:") {
            let ajan = s[11..].trim();
            // yeni bölüm başlar; sadece * veya NEMES bölümü ilgili
            ilgili_bolum = ajan == "*" || ajan.eq_ignore_ascii_case("NEMES") || ajan.eq_ignore_ascii_case("Argus");
            ilk_bolum = false;
            continue;
        }
        // ilk User-agent öncesi Disallow satırları genel sayılır
        if ilk_bolum && kucuk.starts_with("disallow:") {
            let yol = s[9..].trim().to_string();
            if !yol.is_empty() {
                genel_disallow.push(yol);
            }
            continue;
        }
        if ilgili_bolum && kucuk.starts_with("disallow:") {
            let yol = s[9..].trim();
            if yol.is_empty() {
                continue; // Disallow: (boş) = tam izin
            }
            if path.starts_with(yol) {
                return false;
            }
        }
    }
    for d in &genel_disallow {
        if path.starts_with(d) {
            return false;
        }
    }
    true
}

/// HTML'den kaba metin çıkar (script/style atılır, etiketler silinir).
/// Not: nav/header/footer span olarak silinmez (iç içe yapı + ana içeriği
/// yeme riski, 21 Eyl ölçümü: header silme makaleyi 473 karaktere düşürdü).
/// Faz C-3'te readability benimsenir; şimdilik deterministik stub.
pub fn html_metin_cikar(html: &str) -> String {
    let mut s = html.to_string();
    for etiket in ["script", "style"] {
        loop {
            let ac = format!("<{}", etiket);
            let kapa = format!("</{}>", etiket);
            let Some(bas) = s.to_lowercase().find(&ac) else { break };
            let Some(bit) = s.to_lowercase()[bas..].find(&kapa) else { break };
            let son = bas + bit + kapa.len();
            s.replace_range(bas..son.min(s.len()), " ");
            if s.len() > GOVDE_TAVAN_BAYT * 2 {
                break;
            }
        }
    }
    // etiket sil
    let mut cikti = String::with_capacity(s.len().min(200_000));
    let mut icerde = false;
    for ch in s.chars() {
        match ch {
            '<' => icerde = true,
            '>' => icerde = false,
            _ if !icerde => cikti.push(ch),
            _ => {}
        }
    }
    // boşluk normalize
    cikti
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(50_000)
        .collect()
}

/// Miner istemcisi: açık araştırma görevlerini çek (token'lı).
/// Dönüş: [(gorev_id, hedef, parametre_json)]
pub fn kuyruk_cek(
    istemci: &reqwest::blocking::Client,
    komuta: &str,
    token: &str,
    limit: u32,
) -> anyhow::Result<Vec<(i64, String, serde_json::Value)>> {
    let url = format!("{}/api/arastirma/kuyruk?limit={}", komuta.trim_end_matches('/'), limit);
    let resp = istemci
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .timeout(std::time::Duration::from_secs(ZAMAN_ASIMI_SN))
        .send()?;
    if !resp.status().is_success() {
        anyhow::bail!("kuyruk http={}", resp.status());
    }
    let j: serde_json::Value = resp.json()?;
    let mut out = Vec::new();
    if let Some(arr) = j.get("gorevler").and_then(|v| v.as_array()) {
        for g in arr {
            let id = g.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let hedef = g.get("hedef").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let par = g.get("parametre").cloned().unwrap_or(serde_json::json!({}));
            if id > 0 && !hedef.is_empty() {
                out.push((id, hedef, par));
            }
        }
    }
    Ok(out)
}

/// Miner istemcisi: bulguyu provenance ile gönder.
pub fn sonuc_gonder(
    istemci: &reqwest::blocking::Client,
    komuta: &str,
    token: &str,
    gorev_id: i64,
    prov: &Provenance,
    karakter: usize,
) -> anyhow::Result<()> {
    let url = format!("{}/api/arastirma/sonuc", komuta.trim_end_matches('/'));
    let govde = serde_json::json!({
        "gorev_id": gorev_id,
        "url": prov.url,
        "karakter": karakter as i64,
        "icerik_hash": prov.icerik_hash,
        "guven": prov.guven,
    });
    let resp = istemci
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&govde)
        .timeout(std::time::Duration::from_secs(ZAMAN_ASIMI_SN))
        .send()?;
    if !resp.status().is_success() {
        anyhow::bail!("sonuc http={}", resp.status());
    }
    Ok(())
}
/// Kurallı fetch: allowlist -> robots -> rate -> get(cap) -> provenance.
/// Hata mesajı neden içerir (red-allowlist / red-robots / ag-hatasi).
pub fn kuralli_fetch(
    istemci: &reqwest::blocking::Client,
    url: &str,
    robots_txt: Option<&str>,
) -> anyhow::Result<(String, Provenance)> {
    if !url_izinli_mi(url) {
        anyhow::bail!("red-allowlist: {}", url);
    }
    let path = url.splitn(4, '/').nth(3).map(|p| format!("/{}", p)).unwrap_or("/".to_string());
    if let Some(rt) = robots_txt {
        if !robots_izinli_mi(rt, &path) {
            anyhow::bail!("red-robots: {} path={}", url, path);
        }
    }
    std::thread::sleep(Duration::from_secs(ISTEK_ARASI_SN));
    let resp = istemci
        .get(url)
        .header("User-Agent", "NEMES-Argus/0.1 (+arastirma; izinli-kaynak)")
        .timeout(Duration::from_secs(ZAMAN_ASIMI_SN))
        .send()?;
    let kod = resp.status();
    if !kod.is_success() {
        anyhow::bail!("ag-hatasi http={} url={}", kod, url);
    }
    let bayt = resp.bytes()?;
    let kesik = &bayt[..bayt.len().min(GOVDE_TAVAN_BAYT)];
    let metin = html_metin_cikar(&String::from_utf8_lossy(kesik));
    if metin.chars().count() < 200 {
        anyhow::bail!("icerik-kisa ({} karakter): {}", metin.chars().count(), url);
    }
    let prov = provenance_uret(url, metin.as_bytes());
    Ok((metin, prov))
}

#[cfg(test)]
mod testler {
    use super::*;
    #[test]
    fn allowlist_dogrulugu() {
        assert!(url_izinli_mi("https://tr.wikipedia.org/wiki/Mitoloji"));
        assert!(url_izinli_mi("https://arxiv.org/abs/1234"));
        assert!(!url_izinli_mi("https://ornek-kotu-site.com/gizli"));
        assert!(!url_izinli_mi("http://192.168.1.1/admin"));
    }
    #[test]
    fn provenance_hashli() {
        let p = provenance_uret("https://tr.wikipedia.org/wiki/X", b"deneme");
        assert_eq!(p.icerik_hash.len(), 64);
        assert!((p.guven - 0.8).abs() < 0.001);
    }
    #[test]
    fn robots_kaba_uyum() {
        let rt = "User-agent: *\nDisallow: /özel/\nDisallow: /api/\n";
        assert!(robots_izinli_mi(rt, "/wiki/Mitoloji"));
        assert!(!robots_izinli_mi(rt, "/özel/gizli"));
        let bos = "User-agent: *\nDisallow:\n";
        assert!(robots_izinli_mi(bos, "/herhangi"));
    }
    #[test]
    fn html_temizlenir() {
        let h = "<html><head><style>a{}</style></head><body><script>x</script><p>Merhaba <b>dünya</b></p></body></html>";
        let m = html_metin_cikar(h);
        assert!(m.contains("Merhaba"));
        assert!(m.contains("dünya"));
        assert!(!m.contains("script"));
    }
}
