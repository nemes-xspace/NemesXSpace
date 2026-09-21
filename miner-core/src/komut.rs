// Copyright 2026 NEMES-X. SPDX-License-Identifier: Apache-2.0.
//! PARANOID-MINER EPIC-05 (F1): miner tarafı komut dinleyici iskeleti.
//!
//! F1 kapsamı: sadece BEYAZ LİSTE + DUR algısı. Miner her heartbeat öncesi
//! `/tmp/nemes-kill` dosyasına bakar (kill-switch v0 ile aynı mekanizma).
//! Faz C'de buraya imzalı `/api/komut` poll + mesh `nemes/komut` abonesi gelecek.
//! Üretim mining döngüsüne DOKUNMAZ, sadece erken-çıkış sinyali üretir.

use std::path::Path;

/// F1 beyaz liste (komuta-rs KOMUT_BEYAZ_LISTE ile birebir).
pub const KOMUT_BEYAZ_LISTE: &[&str] = &[
    "dur", "yeniden_baslat", "gorev_degistir", "affet", "imha",
    "karantina", "karantina_kaldir", "duraklat", "devam", "arastir",
];

/// Kill-switch / DUR bayrağı var mı? (ops/kill-switch.sh `touch /tmp/nemes-kill`)
pub fn dur_bayragi_var_mi() -> bool {
    Path::new("/tmp/nemes-kill").exists()
}

/// Beyaz liste kontrolü: komuta'dan gelen `tip` uygulanabilir mi?
pub fn tip_izinli_mi(tip: &str) -> bool {
    KOMUT_BEYAZ_LISTE.contains(&tip)
}

/// Mining döngüsünün her tur başında çağırması gereken kontrol.
/// true dönerse döngü temiz çıkış yapmalı (mevcut kanıt korunur).
pub fn dur_kontrol() -> bool {
    if dur_bayragi_var_mi() {
        eprintln!("DUR bayrağı görüldü (/tmp/nemes-kill) - temiz çıkış");
        return true;
    }
    false
}

#[cfg(test)]
mod testler {
    use super::*;
    #[test]
    fn beyaz_liste_dur_var_arastir_var() {
        assert!(tip_izinli_mi("dur"));
        assert!(tip_izinli_mi("arastir"));
        assert!(!tip_izinli_mi("yok-boyle-tip"));
        assert!(!tip_izinli_mi("rm -rf /"));
    }
}
