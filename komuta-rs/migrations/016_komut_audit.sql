-- 016_komut_audit.sql
-- PARANOID-MINER EPIC-05/EPIC-10 (F1): imzalı komut audit zinciri + karantina bayrağı.
-- Garanti kuralı: sadece CREATE (ALTER/DROP yok). Üretim DB'ye güvenli uygulanır.
-- Her /api/komut + gossip komutu buraya yazılır: tip, epoch, pubkey, kaynak, sonuç.

CREATE TABLE IF NOT EXISTS komut_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    epoch INTEGER NOT NULL,
    tip TEXT NOT NULL,
    payload_json TEXT NOT NULL DEFAULT '{}',
    pubkey_b64 TEXT NOT NULL DEFAULT '',
    kaynak TEXT NOT NULL DEFAULT 'http',
    sonuc TEXT NOT NULL DEFAULT 'ok',
    ts INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_komut_log_tip_ts ON komut_log(tip, ts);
CREATE INDEX IF NOT EXISTS idx_komut_log_ts ON komut_log(ts);

-- Global bayraklar (kill-switch + karantina + duraklat). Tek satırlık K/V.
-- k='karantina' v='1' => yeni görev dağıtımı durur (komut_uygula okur).
-- k='dagitim_duraklat' v='1' => aynı etki, operator el freni.
CREATE TABLE IF NOT EXISTS komut_durum (
    k TEXT PRIMARY KEY,
    v TEXT NOT NULL DEFAULT '',
    ts INTEGER NOT NULL
);
