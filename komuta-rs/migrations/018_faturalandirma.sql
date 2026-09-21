-- 018_faturalandirma.sql
-- PARA-1: ödenebilir API için kullanım ölçümü. Mevcut auth'a dokunmaz.
-- /api/ara + /api/arastirma/sonuc her çağrıda satır yazar; fatura toplamada okunur.

CREATE TABLE IF NOT EXISTS api_kullanim (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    anahtar TEXT NOT NULL DEFAULT '',
    miner_id TEXT NOT NULL DEFAULT '',
    uc TEXT NOT NULL,
    adet INTEGER NOT NULL DEFAULT 1,
    ts INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_api_kullanim_anahtar ON api_kullanim(anahtar, ts);
CREATE INDEX IF NOT EXISTS idx_api_kullanim_uc ON api_kullanim(uc, ts);

-- Basit kota: anahtar başına günlük limit (0 = sınırsız). Fatura dışı fren.
CREATE TABLE IF NOT EXISTS api_kota (
    anahtar TEXT PRIMARY KEY,
    gunluk_limit INTEGER NOT NULL DEFAULT 0,
    ts INTEGER NOT NULL
);
