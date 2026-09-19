#!/usr/bin/env python3
"""Denetim defteri kurucu: kayitlari hash-zincirine baglar (blake3).
Imza alani kart torenine kadar BEKLİYOR kalir (sahte imza YOK).
Calistir: python3 kur.py  -> kayitlar.jsonl (uzerine yazar, deterministik)
Dogrula: python3 dogrula.py
"""
import hashlib
import json
import os

try:
    from blake3 import blake3 as _b3

    def ozet(b: bytes) -> str:
        return _b3(b).hexdigest()
except ImportError:  # blake3 yoksa: zincir SHA256 ile kurulur (baslikta isaretlenir)
    def ozet(b: bytes) -> str:
        return 'sha256:' + hashlib.sha256(b).hexdigest()

BURADA = os.path.dirname(os.path.abspath(__file__))
CIKTI = os.path.join(BURADA, 'kayitlar.jsonl')

# no, tur, utc, baslik, tur-detay, kanit, durum, sahip
HAM = [
 ('GEN', 'GEN-000', '2026-09-18T14:00:00Z', 'Defter baslangici', 'Tur-3/4 + surum-kilidi donemi kapsar', 'git log (4fc7e7e..79e649b)', 'arsiv', 'denetci'),
 # --- KARARLAR ---
 ('DR', 'DR-2026-001', '2026-09-18T16:00:00Z', 'LICENSE Apache-2.0', 'Site 3 noktada vaat ediyordu; 14 baslik SPDX; marka korunur; geri-donus: revert', 'site commit 2215609, LICENSE:191', 'yururlukte', 'hak-sahibi'),
 ('DR', 'DR-2026-002', '2026-09-18T16:30:00Z', 'STRICT_DENETIM testnet 0 kalir', 'Ayni-host 3 madenci toleransi sart; mainnet acilisinda 1', 'MAINNET-GECIS.md §A, kod:3674', 'yururlukte', 'hak-sahibi'),
 ('DR', 'DR-2026-003', '2026-09-18T16:30:00Z', 'Era1: testnet-2000, mainnet-800+50B', 'TOKENOMI §kilitli plan; celiski degil takvim', 'TOKENOMI.md:31-33', 'yururlukte', 'hak-sahibi'),
 ('DR', 'DR-2026-004', '2026-09-19T00:00:00Z', 'audit.toml gerekceli istisna (7 advisory)', 'Erisilemezlik gerekceli; kaldirma B35/B36 bagli', '.cargo/audit.toml, cargo audit exit 0', 'yururlukte', 'denetci'),
 ('DR', 'DR-2026-005', '2026-09-18T17:00:00Z', 'Deploy erteleme (systemd D-Bus)', 'Saglikli uretim indirilemez; binary hazir, .bak tam', 'ALTYAPI-KAYIT, IR-2026-002', 'yururlukte', 'denetci+hak-sahibi'),
 ('DR', 'DR-2026-006', '2026-09-19T00:30:00Z', 'Win token rotasyonu erteleme (VM-oturumu)', 'VM yazı yolu yok; cevirmek madenci oldurur; dosya 600 git-disi', 'ss taramasi (tum portlar kapali)', 'yururlukte', 'denetci'),
 # --- DEGISIKLIKLER ---
 ('CR', 'CR-2026-001', '2026-09-18T15:44:00Z', 'Kuyruk tabani kodda max(5,...)', '2000>>n olumu kaldirildi; test taban-kilidi', 'ac4aa0a, test_halving', 'yayinda-degil (deploy bekler)', 'denetci'),
 ('CR', 'CR-2026-002', '2026-09-18T15:50:00Z', 'miner-api silindi (6 dosya)', 'Derlenemez, sabit-JWT, joker-CORS; 08 Eyl dondurulmustu', 'ac4aa0a (git rm)', 'yayinda-degil', 'denetci'),
 ('CR', 'CR-2026-003', '2026-09-18T15:30:00Z', 'P2P: taze-dogrulama+dial+validate', 'Replay kapandi; mDNS warn+continue; bail', 'p2p c02bc3a, build+test yesil', 'yayinda-degil', 'denetci'),
 ('CR', 'CR-2026-004', '2026-09-18T15:10:00Z', 'Izleme FIFO-alarm (sayi->yas)', '88K birikim normal; 72sa esigi', 'izleme.py, canli cikti (alarm yok)', 'CANLI', 'denetci'),
 ('CR', 'CR-2026-005', '2026-09-19T00:27:00Z', 'API kilitleri (kova/CORS/limit/401/ara)', '10/sa/IP, sik CORS, 512KB, 401-once, ara tokenli', '79e649b, 27/27 test', 'yayinda-degil', 'denetci'),
 ('CR', 'CR-2026-006', '2026-09-19T00:22:00Z', 'Yedek sertlestirme (pages=-1,atomik,dogrulama,rotasyon)', '2.7sa takilma kokten; 8.6G dogrulamali alindi', 'backup-db.py, yedek/komuta-2026-09-19-0321.db', 'CANLI (timer)', 'denetci'),
 ('CR', 'CR-2026-007', '2026-09-19T00:28:00Z', 'Zip tazelendi (reqwest-0.12 binary)', '17.6M, KURULUM esit, SHA 3/3', '93fbc48, sha256sum -c OK', 'yayinda (site push bekler)', 'denetci'),
 ('CR', 'CR-2026-008', '2026-09-18T16:06:00Z', 'Site Tur-4 (odeme/video/KURULUM/SHA/CI)', '8 dil, CI baz=en, og duzeltme', 'site 878e78e, CI TEMIZ', 'yayinda (push bekler)', 'denetci'),
 ('CR', 'CR-2026-009', '2026-09-18T17:30:00Z', 'Gossip-komut kablolandi (komut_uygula)', 'HTTP ile es kontroller; test eklendi', '8b6a56a, test_komut_uygula_gossip', 'yayinda-degil', 'denetci'),
 ('CR', 'CR-2026-010', '2026-09-18T17:40:00Z', 'Flake koku: test tohumu 60->200', '%25->15/15 + paralel 5/5', '8b6a56a', 'yayinda-degil', 'denetci'),
 ('CR', 'CR-2026-011', '2026-09-18T18:00:00Z', 'reqwest 0.12 + sqlx sadelestirme', 'rustls-0.23 advisory oldu; mysql derlenmiyor', '8b6a56a, cargo audit', 'yayinda-degil', 'denetci'),
 ('CR', 'CR-2026-012', '2026-09-19T00:25:00Z', 'KURULUM yeniden (LM-Studio birincil)', 'Bundle URL-siz; port acik; ornek hedefli', '93fbc48, CLI help capraz', 'yayinda (push bekler)', 'denetci'),
 ('CR', 'CR-2026-013', '2026-09-19T00:20:00Z', 'Runbook komut/boyut duzeltmesi', '8.1GiB, birimli komutlar', 'RUNBOOK-ROLLBACK.md:12-32', 'yayinda-degil', 'denetci'),
 # --- BULGULAR ---
 ('FR', 'FR-2026-001', '2026-09-18T15:00:00Z', 'Gossip komut vadesiz (replay)', 'KRITIK', 'protocol.rs:16, CLI:96-101', 'kapatildi (CR-2026-003)', 'denetci'),
 ('FR', 'FR-2026-002', '2026-09-18T15:00:00Z', 'mDNS dial tum donguyu devirir', 'YUKSEK', 'p2p lib.rs:229', 'kapatildi (CR-2026-003)', 'denetci'),
 ('FR', 'FR-2026-003', '2026-09-18T15:00:00Z', 'validate_block Ok(true) sahte guvence', 'YUKSEK', 'consensus.rs:17', 'kapatildi (CR-2026-003)', 'denetci'),
 ('FR', 'FR-2026-004', '2026-09-18T15:40:00Z', 'Kuyruk tabani kodda yok (olum)', 'YUKSEK', 'TOKENOMI:34 vs kod >>n', 'kapatildi (CR-2026-001)', 'denetci'),
 ('FR', 'FR-2026-005', '2026-09-18T22:00:00Z', 'CORS permissive', 'YUKSEK', 'main.rs:3613 + canli prob *', 'kapatildi-kod (deploy bekler)', 'denetci'),
 ('FR', 'FR-2026-006', '2026-09-18T22:00:00Z', 'Toplu-kanit authsuz 200', 'ORTA', 'main.rs:2163 + canli prob', 'kapatildi-kod (deploy bekler)', 'denetci'),
 ('FR', 'FR-2026-007', '2026-09-18T22:00:00Z', 'Acik sinirsiz kayit (Sybil)', 'YUKSEK', 'main.rs:867,928', 'kapatildi-kod (deploy bekler)', 'denetci'),
 ('FR', 'FR-2026-008', '2026-09-18T22:00:00Z', 'Anonim /api/ara 14.4sn yakma', 'KRITIK', 'canli prob 200/14.4s', 'kapatildi-kod (deploy bekler)', 'denetci'),
 ('FR', 'FR-2026-009', '2026-09-19T00:00:00Z', 'Yedek 2.7sa takilma + kismi dosya', 'YUKSEK', 'PID 1397850 %99.5 CPU, 2.37G/8.6G', 'kapatildi (CR-2026-006)', 'denetci'),
 ('FR', 'FR-2026-010', '2026-09-19T01:00:00Z', 'Hashsiz GitHub indirici (RCE-sartli)', 'KRITIK-sartli', 'llama.rs:238-262, cagiran yok', 'acik (B40)', 'denetci'),
 ('FR', 'FR-2026-011', '2026-09-19T01:00:00Z', 'Tokenlar DB duz-metin', 'YUKSEK', 'miners.token', 'acik (B39-asiri)', 'denetci'),
 ('FR', 'FR-2026-012', '2026-09-19T01:00:00Z', 'SPOF tek-host', 'YUKSEK', 'servis listesi, operator 0', 'ertelendi (Tohum disi)', 'hak-sahibi'),
 ('FR', 'FR-2026-013', '2026-09-18T23:00:00Z', 'Disk ~30 gun projeksiyonu', 'YUKSEK', 'df + 3 yedek buyume', 'kismen (rotasyon)', 'denetci'),
 ('FR', 'FR-2026-014', '2026-09-18T17:52:00Z', 'Cift-denetim %12.8 GPU israfi', 'ORTA', '80/626 journal sayimi', 'acik (B38)', 'denetci'),
 ('FR', 'FR-2026-015', '2026-09-18T16:00:00Z', 'Ajan fantom-bulgulari (11+8)', 'ORTA-surec', 'grep ile dusuruldu, ornekli', 'kapatildi (§12.4 kural)', 'denetci'),
 # --- ONAY / OLAY / ERISIM / GOZDEN ---
 ('AR', 'AR-2026-001', '2026-09-01T00:00:00Z', 'Mutlak otorite berati (hak sahibi)', 'Onay: docs/MUTLAK-OTORITE.md; imza: kart-toreni [DOGRULANMADI-bu-oturumda]', 'durum: gecerli', 'hak-sahibi'),
 ('AR', 'AR-2026-002', '2026-09-18T14:00:00Z', 'Sohbet-ici operator onaylari (kalanlar, lisans, repo-acilis)', 'EKSIK: yazili/imzali degil, sohbet kaydi disinda bagimsiz kanit yok', 'durum: eksik-kayit', 'hak-sahibi'),
 ('IR', 'IR-2026-001', '2026-09-19T00:38:00Z', 'Yedek timer 2.7sa takilma', 'Tespit: denetim; neden: pages=100+sleep; mudahale: kill+temizlik+pages=-1; sonuc: 8.6G dogrulamali', 'ders: buyuk-DB sayfa-kademesi yasak', 'denetci'),
 ('IR', 'IR-2026-002', '2026-09-18T19:10:00Z', 'systemd D-Bus stop/kill zaman-asimi', 'Tespit: deploy denemesi; neden: BILINMIYOR (dailyaidecheck supheli); mudahale: PID1e dokunulmadı, erteleme', 'ders: dagitim-kilit kontrolu deploy-oncesi', 'denetci'),
 ('IR', 'IR-2026-003', '2026-09-18T17:00:00Z', 'Test flake %25 (denetim turu)', 'Neden: 60-maddelik tohum tukenmesi; mudahale: 200 madde; sonuc: 15/15', 'ders: zar-kapasite eslesmesi testi', 'denetci'),
 ('AL', 'AL-2026-001', '2026-09-19T07:15:00Z', 'YOKLUK-KAYDI: erisim logu tutulmuyor', 'Kim neye eristi sorusu cevaplanamaz (auditd yok, IP log yok); zafiyet olarak isli', 'durum: acik-zafiyet', 'denetci'),
 ('RR', 'RR-2026-001', '2026-09-19T07:15:00Z', 'Defter kurulus gozden-gecirmesi', 'Kapsam: Prompt1-3 + bu oturum; katilan: denetci(otomatik)+hak-sahibi; karar: defter gecerli, imza toreni bekleniyor', 'sonraki: imza sonrasi RR-2026-002', 'denetci'),
 ('IR', 'IR-2026-004', '2026-09-19T07:25:00Z', 'Felaket provasi: restore (kopya)', 'Kopya 12sn + integrity ok (4.76M) + DB_PATH/PORT ile canli acilis ~2dk; kopya silindi, uretim etkilenmedi', 'tatbikat logu (bu oturum)', 'kapandi-basarili', 'denetci'),
 ('IR', 'IR-2026-005', '2026-09-19T07:20:00Z', 'Felaket provasi: bozuk-kopya tespiti', 'Sifirlanmis baslik aninda yakalandi (not-a-database 26); backup-db.py integrity ayni yolu kullanir', 'tatbikat logu (bu oturum)', 'kapandi-basarili', 'denetci'),
 ('RR', 'RR-2026-002', '2026-09-19T07:30:00Z', 'Felaket provasi degerlendirmesi', 'Restore KANITLANDI (RTO~3dk, RPO~24sa); .bak duman testi OK; /tmp tmpfs dersi (8G kopya doldurur)', 'FELAKET-PROVASI.md', 'sonraki: 3 ay sonra tekrar', 'denetci'),
]

with open(CIKTI, 'w', encoding='utf-8') as f:
    onceki = '0' * 64
    for kayit in HAM:
        if len(kayit) == 7:  # AR: kanit alani yok -> isaretle
            tur, no, utc, baslik, detay, durum, sahip = kayit
            kanit = '(AR turu: kanit detaya gomulu)'
        else:
            tur, no, utc, baslik, detay, kanit, durum, sahip = kayit
        govde = {'no': no, 'tur': tur, 'utc': utc, 'baslik': baslik,
                 'detay': detay, 'kanit': kanit, 'durum': durum,
                 'sahip': sahip, 'onceki': onceki, 'imza': 'BEKLIYOR-kart-toreni'}
        ham = json.dumps(govde, ensure_ascii=False, sort_keys=True)
        h = ozet(ham.encode())
        govde['hash'] = h
        f.write(json.dumps(govde, ensure_ascii=False) + '\n')
        onceki = h
print(f'{len(HAM)} kayit yazildi -> {CIKTI}')
