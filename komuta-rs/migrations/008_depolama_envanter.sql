-- 008_depolama_envanter.sql
-- Otonom depolama katmani: parca envanteri + yer bilgisi + kota.
-- Kural: sadece CREATE TABLE + ADD COLUMN (mevcut tabloya dokunulmaz).

-- Icerik-adresli parcalar (blake3 hash = kimlik).
CREATE TABLE IF NOT EXISTS parcalar (
    parca_hash TEXT PRIMARY KEY,
    boyut INTEGER NOT NULL,
    ts INTEGER NOT NULL
);

-- Hangi parca hangi miner'da (kopya takibi).
CREATE TABLE IF NOT EXISTS parca_yerleri (
    parca_hash TEXT NOT NULL,
    miner_id TEXT NOT NULL,
    ts INTEGER NOT NULL,
    PRIMARY KEY (parca_hash, miner_id)
);

CREATE INDEX IF NOT EXISTS idx_yer_miner ON parca_yerleri(miner_id);
CREATE INDEX IF NOT EXISTS idx_yer_parca ON parca_yerleri(parca_hash);

-- Miner depolama kotasi (bayt). 0 = compute-only katman.
ALTER TABLE miners ADD COLUMN depolama_kota INTEGER DEFAULT 0;
