-- 015_miner_yetenek.sql
-- B23: kabiliyet ilani (heterojen mesh'in kapisi). Madenci nabizda donanim
-- ve rollerini bildirir; komuta gorev sinifini kabiliyete gore eslestirir
-- (v1: kayit + gorunurluk; eslestirme ilk metin-disi gorevde).
-- Garanti kurali 3 uyumu: sadece CREATE (ALTER/DROP yok).

CREATE TABLE IF NOT EXISTS miner_yetenek (
    miner_id TEXT PRIMARY KEY,
    gpu_ad TEXT NOT NULL DEFAULT '',
    vram_mb INTEGER NOT NULL DEFAULT 0,
    roller TEXT NOT NULL DEFAULT 'embed,denetim',
    ts INTEGER NOT NULL
);
