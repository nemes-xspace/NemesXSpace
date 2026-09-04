-- Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
-- 004_slash_itibar.sql
-- Iki-asamali challenge + slash + itibar.
-- fail#1: supheli (ret=1, dogrulama NULL birakilir, ikinci denetciye gider).
-- fail#2: slash (geri alim + strike + itibar kesintisi).

ALTER TABLE miners ADD COLUMN strike INTEGER DEFAULT 0;
ALTER TABLE miners ADD COLUMN itibar INTEGER DEFAULT 100;

ALTER TABLE kanitlar ADD COLUMN ret INTEGER DEFAULT 0;
ALTER TABLE kanitlar ADD COLUMN son_denetci TEXT NULL;
