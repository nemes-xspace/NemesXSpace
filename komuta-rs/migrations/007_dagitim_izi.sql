-- Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
-- 007_dagitim_izi.sql
-- Dagitilmis-ama-kaniti-henuz-gelmemis id'leri supurmeden muaf tut (TOCTOU).
-- Supurme SADECE izi eski (>300sn) veya hic izi olmayan bosluklari dagitir.
CREATE TABLE IF NOT EXISTS dagitilan_madde (
    madde_id INTEGER PRIMARY KEY,
    ts INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_dagitilan_ts ON dagitilan_madde(ts);
