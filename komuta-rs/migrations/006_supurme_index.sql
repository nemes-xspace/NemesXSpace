-- Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
-- 006_supurme_index.sql
-- Supurme taramasi: kanitlanmamis madde id'lerini hizli bulmak icin.
CREATE INDEX IF NOT EXISTS idx_kanitlar_madde ON kanitlar(madde_id);
