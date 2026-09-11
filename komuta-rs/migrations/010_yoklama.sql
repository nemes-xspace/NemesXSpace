-- 010_yoklama.sql
-- C8: rastgele parca yoklamasi (proof-of-storage-lite) + dusurme karari izi.
-- dusurme sayaci tabloya yazilmaz: son 2 sonuc pes pese fail ise kota sifirlanir.

CREATE TABLE IF NOT EXISTS yoklamalar (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    miner_id TEXT NOT NULL,
    parca_hash TEXT NOT NULL,
    offset INTEGER NOT NULL,
    uzunluk INTEGER NOT NULL,
    beklenen_hash TEXT NOT NULL,
    durum TEXT DEFAULT 'acik',
    ts INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_yoklama_miner ON yoklamalar(miner_id, durum);
