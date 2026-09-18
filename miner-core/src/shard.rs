// Copyright 2026 NEMES-X. SPDX-License-Identifier: Apache-2.0.
//! Shard sahipligi (P2P gossip `nemes/shard`).
//!
//! Fikir: madenci, kazacagi id araligini mesh'e ILAN eder, komuta bu
//! ilanlari dinleyip dagitimi yonlendirir (cursor + atlama).
//! Ilan GORELI'dir: "sonraki K id'yi istiyorum" der, komuta ulasma
//! anindaki cursor'a sabirler. Gecikmeli gossip'e dayaniklidir.
//!
//! Format v0 (imzali):
//!   kanonik = "SHARDILAN|v0|{corpus}|{adet}|{miner_id}|{ts_ms}"
//!   sig = Ed25519(kanonik, miner_tohumu)

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::Signer;
use std::path::{Path, PathBuf};

// Wire formati tek evde: nemes-core::shard (komuta-rs ile ayni struct).
pub use ed25519_dalek::SigningKey;
pub use nemes_core::shard::{ShardIlan, SHARD_FORMAT, SHARD_MAX_ADET, SHARD_TOPIC};

/// Yeni ilan kur + imzala (miner tarafi yardimcisi).
pub fn ilan_imzala(
    sk: &SigningKey,
    corpus: impl Into<String>,
    adet: i64,
    miner_id: impl Into<String>,
    ts_ms: i64,
) -> ShardIlan {
    let corpus = corpus.into();
    let miner_id = miner_id.into();
    let msg = ShardIlan::kanonik(&corpus, adet, &miner_id, ts_ms);
    let sig = sk.sign(&msg);
    let vk = sk.verifying_key();
    ShardIlan {
        v: SHARD_FORMAT.to_string(),
        corpus,
        adet,
        miner_id,
        ts_ms,
        pubkey_b64: B64.encode(vk.to_bytes()),
        sig_b64: B64.encode(sig.to_bytes()),
    }
}

/// Miner kimligi: ~/.nemes/miner-ed25519.key (hex tohum, 0600).
pub fn anahtar_yolu() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".nemes")
        .join("miner-ed25519.key")
}

/// Yoksa uret, varsa oku. Donen imza anahtari.
pub fn anahtar_yukle_veya_uret(yol: &Path) -> anyhow::Result<SigningKey> {
    if yol.exists() {
        let hexs = std::fs::read_to_string(yol)?;
        let raw = hex::decode(hexs.trim())?;
        let arr: [u8; 32] = raw
            .try_into()
            .map_err(|_| anyhow::anyhow!("anahtar dosyasi 32 bayt olmali: {}", yol.display()))?;
        return Ok(SigningKey::from_bytes(&arr));
    }
    // Windows'ta Unix izin biti yok; dosya ACL ile korunur (kullanici dizini).
    // Paranoyak not: paylasimli makinede anahtar dizinini elle kilitleyin.
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;
    if let Some(parent) = yol.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // OS entropisiyle uret.
    let mut tohum = [0u8; 32];
    getrandom::getrandom(&mut tohum)?;
    let sk = SigningKey::from_bytes(&tohum);
    let mut opt = std::fs::OpenOptions::new();
    opt.write(true).create_new(true);
    #[cfg(unix)]
    opt.mode(0o600);
    use std::io::Write;
    let mut f = opt.open(yol)?;
    writeln!(f, "{}", hex::encode(tohum))?;
    // Bellekteki tohumu temizle (best-effort).
    tohum.iter_mut().for_each(|b| *b = 0);
    Ok(sk)
}

pub fn pubkey_b64(sk: &SigningKey) -> String {
    B64.encode(sk.verifying_key().to_bytes())
}

/// Simdiki zaman (milisaniye).
pub fn simdi_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Uzun omurlu P2P relay: mesh'e bagli kalir, ilanlari yayinlar.
/// Tek gorev sahibi vardir (publish + run_for ayni dongude).
pub struct ShardRelay {
    tx: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
}

impl ShardRelay {
    pub fn baslat(port: u16) -> (Self, tokio::task::JoinHandle<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
        let h = tokio::spawn(async move {
            let mut node = match nemes_p2p::P2PNode::new(nemes_p2p::P2PConfig {
                port,
                enable_mdns: true,
                ..Default::default()
            })
            .await
            {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("[relay] P2P node acilamadi: {} (madencilik suruyor, ilan yok)", e);
                    return;
                }
            };
            let mut kuyruk: std::collections::VecDeque<Vec<u8>> = std::collections::VecDeque::new();
            loop {
                while let Ok(d) = rx.try_recv() {
                    if kuyruk.len() >= 16 {
                        kuyruk.pop_front();
                    }
                    kuyruk.push_back(d);
                }
                // Mesh henuz yoksa InsufficientPeers verir; kuyrukta tut, sonra tekrar dene.
                let mut bekleyen = std::mem::take(&mut kuyruk);
                // VecDeque yerine Vec ile dolas (sirali).
                let sirali: Vec<Vec<u8>> = bekleyen.drain(..).collect();
                for d in sirali {
                    if let Err(e) = node.publish(SHARD_TOPIC, d.clone()) {
                        eprintln!("[relay] publish ertelendi: {} (mesh bekleniyor)", e);
                        if kuyruk.len() < 16 {
                            kuyruk.push_back(d);
                        }
                    } else {
                        eprintln!("[relay] shard ilani yayinlandi ({} bayt)", d.len());
                    }
                }
                if node.run_for(5).await.is_err() {
                    break;
                }
            }
        });
        (Self { tx }, h)
    }

    pub fn ilan_gonder(&self, ilan: &ShardIlan) -> anyhow::Result<()> {
        let data = ilan.json_bayt()?;
        self.tx.send(data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ilan_imza_dongusu() {
        let sk = SigningKey::from_bytes(&[7u8; 32]);
        let ilan = ilan_imzala(&sk, "tr", 2000, "miner-test", 1234567890);
        assert!(ilan.dogrula().is_ok());
        let bytes = ilan.json_bayt().unwrap();
        let geri = ShardIlan::json_coz(&bytes).unwrap();
        assert_eq!(geri.miner_id, "miner-test");
    }

    #[test]
    fn bozuk_imza_reddedilir() {
        let sk = SigningKey::from_bytes(&[7u8; 32]);
        let mut ilan = ilan_imzala(&sk, "tr", 2000, "miner-test", 1234567890);
        ilan.adet = 9999; // imzadan sonra degistir
        assert!(ilan.dogrula().is_err());
    }
}
