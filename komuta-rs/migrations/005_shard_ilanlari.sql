-- Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
-- 005_shard_ilanlari.sql
-- P2P gossip (nemes/shard) ile shard sahipligi.
-- Ilan GORELI'dir: komuta ulasma anindaki cursor'a sabirler.

-- Miner kimligi <-> pubkey bagi (ilk ilanda kilitlenir: TOFU).
ALTER TABLE miners ADD COLUMN pubkey_b64 TEXT NULL;

CREATE TABLE IF NOT EXISTS shard_ilanlari (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    miner_id TEXT NOT NULL,
    corpus TEXT NOT NULL,
    baslangic INTEGER NOT NULL,
    bitis INTEGER NOT NULL,
    adet INTEGER NOT NULL,
    pubkey_b64 TEXT NOT NULL,
    ts_ms INTEGER NOT NULL,
    kaynak TEXT DEFAULT 'gossip',
    durum TEXT DEFAULT 'aktif',
    ts INTEGER NOT NULL,
    UNIQUE(miner_id, ts_ms)
);

CREATE INDEX IF NOT EXISTS idx_shard_corpus ON shard_ilanlari(corpus, durum);
CREATE INDEX IF NOT EXISTS idx_shard_miner ON shard_ilanlari(miner_id, durum);
