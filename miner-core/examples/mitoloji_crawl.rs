// Mitoloji kapalı-devre crawl (Faz C-2): 10 izinli sayfa + provenance.
// Çalıştır: cargo run -p miner-core --example mitoloji_crawl
// Çıktı: /tmp/mitoloji-paket/*.json (metin + provenance). Üretime yazmaz.
use miner_core::{kuralli_fetch, ZAMAN_ASIMI_SN};
use std::time::Duration;

const URLS: &[&str] = &[
    "https://tr.wikipedia.org/wiki/Türk_mitolojisi",
    "https://tr.wikipedia.org/wiki/Yunan_mitolojisi",
    "https://tr.wikipedia.org/wiki/Mısır_mitolojisi",
    "https://tr.wikipedia.org/wiki/İskandinav_mitolojisi",
    "https://tr.wikipedia.org/wiki/Mezopotamya_mitolojisi",
    "https://tr.wikipedia.org/wiki/Tengri",
    "https://tr.wikipedia.org/wiki/Ergenekon",
    "https://tr.wikipedia.org/wiki/Gılgamış",
    "https://tr.wikipedia.org/wiki/Zeus",
    "https://tr.wikipedia.org/wiki/Odin",
];

fn main() -> anyhow::Result<()> {
    std::fs::create_dir_all("/tmp/mitoloji-paket")?;
    let istemci = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(ZAMAN_ASIMI_SN))
        .build()?;
    // robots.txt tek sefer (tr.wikipedia.org)
    let robots = istemci
        .get("https://tr.wikipedia.org/robots.txt")
        .timeout(Duration::from_secs(ZAMAN_ASIMI_SN))
        .send()
        .ok()
        .and_then(|r| r.text().ok());
    println!("robots: {} bayt", robots.as_ref().map(|s| s.len()).unwrap_or(0));
    let mut ok = 0;
    for (i, url) in URLS.iter().enumerate() {
        match kuralli_fetch(&istemci, url, robots.as_deref()) {
            Ok((metin, prov)) => {
                let kayit = serde_json::json!({
                    "sira": i,
                    "url": url,
                    "karakter": metin.chars().count(),
                    "metin": &metin[..metin.len().min(20_000)],
                    "provenance": prov,
                });
                std::fs::write(
                    format!("/tmp/mitoloji-paket/{:02}.json", i),
                    serde_json::to_string_pretty(&kayit)?,
                )?;
                println!("[{}/10] OK {} ({} karakter)", i + 1, url, metin.chars().count());
                ok += 1;
            }
            Err(e) => println!("[{}/10] RED {}: {}", i + 1, url, e),
        }
    }
    println!("paket: {ok}/10 -> /tmp/mitoloji-paket/");
    if ok < 8 {
        anyhow::bail!("paket eksik: {}/10 (eşik 8)", ok);
    }
    Ok(())
}
