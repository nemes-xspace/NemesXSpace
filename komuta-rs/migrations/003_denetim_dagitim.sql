-- Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
-- 003_denetim_dagitim.sql
-- Es-dogrulama: bayrakli kanitlarin kac kez denetciye dagitildigini say.
ALTER TABLE kanitlar ADD COLUMN denetim_sayisi INTEGER DEFAULT 0;
