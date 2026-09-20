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
 ('RR', 'RR-2026-003', '2026-09-19T08:00:00Z', 'Sureklilik olcumu (bus/3-2-1/tek-kaynak)', 'bus=1, yedek 9-kopya/1-lokasyon/0-uzak, crates.io %100; fork-prova clone+fetch OK', 'SUREKLILIK.md', 'sonraki: eylemler kapaninca', 'denetci'),
 ('RR', 'RR-2026-004', '2026-09-19T08:30:00Z', 'Karsi-istihbarat supurmesi', 'Site/zip/git sizintisiz; DNS CF-arkasi; 401-korluk (olcum yaniltici cikti); opencode.db kirpik-token (kullanilamaz, mudahale yok)', 'KARSI-ISTIHBARAT.md', 'sonraki: aylik OSINT', 'denetci'),
 ('RR', 'RR-2026-005', '2026-09-19T09:00:00Z', 'Zincir kapanisi (P1-P7)', 'Yonetici ozeti + sahipli dongu takvimi + nihai kural politika (§12.4); dongu 2. tura hazir', 'ZINCIR-OZETI.md', 'sonraki: dongu periyoduna gore', 'denetci'),
 ('CR', 'CR-2026-014', '2026-09-19T10:00:00Z', 'Dalga-1-deploy (R-01..R-08+R-11)', 'systemd duzeldi; tum binaryler yenilendi (.bak4); canli-dogrulama: CORS-kilit, ara-401, http-red-log, uretim', 'journal + curl cikti (bu oturum)', 'CANLI', 'denetci'),
 ('CR', 'CR-2026-015', '2026-09-19T10:00:00Z', 'R-02 indirici-silme + R-08 atomik-claim + R-06 IP-middleware', 'URL olu cikti (404); claim rows_affected; middleware 4xx/5xx-only', '27/27 test + canli prob', 'CANLI', 'denetci'),
 ('IR', 'IR-2026-006', '2026-09-19T10:00:00Z', 'AIDE 22sa tarama = D-Bus bogulma koku', 'Mudahale: 90_nemes_data istisnasi + takilmis tarama olduruldu; dogrulama bu-gece-timer', 'R-11 izlemede', 'denetci'),
 ('CR', 'CR-2026-016', '2026-09-19T10:30:00Z', 'Prompt9: imza+kanarya+sandbox+SBOM+LICENSE+IR', 'SHA256SUMS.sig (Good signature); kanarya-credential (uyuyor); 3 sandbox drop-in (canli, hatasiz); SBOM 395+300; LICENSE 2 repoya; IR-PLAN', 'journal+sig-dogrulama+test', 'CANLI', 'denetci'),
 ('CR', 'CR-2026-017', '2026-09-19T11:00:00Z', 'Prompt10: derin-saglik+kaos-olcum+ADR-004', 'db_ms/havuz_boyut canli (0); restart-uretim 26sn/2dk; graceful-degradation kanitli; yapilmadilari gerekceli', 'curl+journal+27/27', 'CANLI', 'denetci'),
 ('CR', 'CR-2026-018', '2026-09-19T12:00:00Z', 'Prompt11: odeme_yaz atomik (8 cift) + VERI-ENVANTERI', '395-mikro tarihsel kayma arastirildi (dokunulmadi); 27/27; LUKS-suzluk olculdu (acik)', 'test+sql-olcum+lsblk', 'kod-hazir (deploy bekler)', 'denetci'),
 ('CR', 'CR-2026-019', '2026-09-19T13:30:00Z', 'Prompt12: limit+sandbox-canli+journal-cap+ops-surum+deploy-proseduru+disk-yuzde', 'MemoryMax 20G/2G, sandbox 3 serviste hatasiz, journald 2G, birimler gitte, erken-uyari %90', 'systemd-verify+journal+izleme', 'CANLI-kismen (limitler restartta)', 'denetci'),
 ('CR', 'CR-2026-020', '2026-09-19T16:15:00Z', 'Prompt13: nabiz+hiyerarsi+vasiyet-sablonu', 'otorite-nabiz canli (0 alarm); HSM/FIDO2/m-of-n YOK-kaydi; mektup YAZILMADI (sahipte)', 'izleme-ciktisi+grep', 'CANLI-kismen', 'denetci'),
 ('CR', 'CR-2026-021', '2026-09-19T17:00:00Z', 'Prompt14: talep-proseduru+S7-brief (avukat-degilim notuyla)', 'Mütalaa YOK; iskelet+sorular yazildi; privacy.html madenci-kapsamli (kismi-olumlu)', 'dokuman', 'taslak (hukuk-onayi bekler)', 'denetci'),
 ('CR', 'CR-2026-022', '2026-09-19T18:00:00Z', 'Prompt15-revize: AI-sinir politikasi + uyum-olcumu', '7 yasak olculdu (hepsi uyumlu); nabiz-dokunulmazlik kurali; otomatik-kilit N/A-gerekceli', 'remote/push-akisi + oturum-gecmisi', 'yururlukte', 'denetci'),
 ('CR', 'CR-2026-023', '2026-09-19T18:30:00Z', 'Prompt16-revize: switch-ENV + git-ifsasi-kabulu', 'Esik koddan cikti; commit-ritmi ifsasi yazili-kabul (6-ay gozden-gecirme)', 'izleme-ciktisi+git-log', 'yururlukte', 'denetci'),
 ('CR', 'CR-2026-024', '2026-09-19T19:00:00Z', 'Prompt17: regresyon-batarya + R-REG-01 + R-08-etkinlik', '27x6, audit, CI, SHA, zincir, canli-prob; README-hash regresyonu yakalandi-duzeltildi; 409 %12.8->%0; bagimsizlik-yapisal-eksik (sahip-gozu bekler)', 'test-ciktilari+olcumler', 'dogrulandi-kismen', 'denetci'),
 ('CR', 'CR-2026-025', '2026-09-19T19:30:00Z', 'Prompt18: dongu otomasyonu (Tur-1)', 'dongu-gunluk.py + timer (gunluk); Tur-1: 0 bulgu; .venv-denetim kalici', 'betik-ciktisi+timer-listesi', 'CANLI', 'denetci'),
 ('RR', 'RR-2026-006', '2026-09-19T20:00:00Z', 'Zincir kapanis-dogrulamasi (P1-P18)', '20/20 cikti-dosyasi mevcut; eksik 0; dongu 2. tura hazir (sahip-karari bekler)', 'dosya-varlik-taramasi (bu oturum)', 'kapandi', 'denetci'),
 ('FR', 'FR-2026-016', '2026-09-19T20:30:00Z', 'FAISS-birlesme duruk (8 Eyl), canli-uretim DBde birikiyor', 'ORTA: cikti aranamiyor; birlestirme el-isi ve kosulmadi', 'URETIM-ANATOMISI.md §5', 'acik (sahipte)', 'denetci'),
 ('CR', 'CR-2026-026', '2026-09-19T21:00:00Z', 'Federe-egitim G0: tek-kart QLoRA-repro gecti', '1.7B/100-adim/673sn, kayip~2.0, adaptor 123MB; uretim etkilenmedi (7654/15dk); GPU %94->%0', 'repro.log + adaptor-dizini + DB-sayimi', 'kapandi', 'denetci'),
 ('CR', 'CR-2026-027', '2026-09-19T21:30:00Z', 'Faz-A: uretim-FAISS otomasyonu (FR-2026-016 kapandi)', 'wiki_maden_tr.faiss 382.687 vektor (ntotal-dogrulamali); sadakat: brute self-rank=1, recall@10=2-3/10 (PQ32-siniri, kayitli); haftalik-timer', 'faiss-olcumler+timer', 'CANLI', 'denetci'),
 ('DR', 'DR-2026-007', '2026-09-19T22:00:00Z', 'Vasiyet-mektubu REDDI + dagitik-300 varsayimi', 'Mektup yazilmayacak (sahip); kalici-yokluk prosedursuz, risk tasinir, 6-ay-gozden-gecirme. 300 GPU dagitik: senkron/boru-hatti OLU, asenkron-FedAvg/DiLoCo yasar', 'sahip-beyani (bu oturum)', 'yururlukte', 'hak-sahibi'),
 ('CR', 'CR-2026-028', '2026-09-19T22:30:00Z', 'Dalga-tamamlama: B35/B36/B39/R-08/R-25/vendor/rehber + master-Rev2', 'libp2p-0.57, sqlx-0.8.6, HMAC-pepper+test, atomik-claim, PQ64-negatif, 884M-vendor, cüzdan-rehberi; hepsi test-yesil', 'test/audit/olcum', 'kod-hazir (deploy bekler)', 'denetci'),
 ('IR', 'IR-2026-007', '2026-09-19T21:45:00Z', 'Restore-yakin-kacirma: D-Bus-takim + `;` zinciri', 'Servisler durmadi, dosya tasindi; ayni-inode sayesinde sifir-kayip (5.29M dogrulandi). Kural: yikici-islemde `&&` + durum-kontrolu; tam-yol-restore D-Bus-istikrarsizken YASAK', 'journal+DB-sayimi', 'kapandi-dersli', 'denetci'),
 ('RR', 'RR-2026-007', '2026-09-19T22:30:00Z', 'Derin-tarama duzeltmesi-2: REPLIKA_HEDEF gercek', 'Sabit komuta-rs:67de + onarim-dongusu calisiyor (60 tamam); onceki yok-hukmu YANLIS (p2pde aranmis); REPLIKASYON.md yazildi', 'kod-satiri+DB-olcum', 'kapandi-duzeltmeli', 'denetci'),
 ('IR', 'IR-2026-008', '2026-09-19T23:00:00Z', 'Kutu-supurme: 4-ajan + HF-token-canli + backlog-75sa', 'HF-token dosyasi 700e cekildi (rotasyon sahipte); backlog alarm calisiyor (4.9x inflow); 350MB+ /tmp temizlendi; Tauri/K1-B8/kasa-log kayda gecti', 'supurme-raporlari', 'kismen (alarm+secim-sahipte)', 'denetci'),
 ('CR', 'CR-2026-029', '2026-09-19T23:30:00Z', 'Zehir-deneyi: gecit-yonu-dogru ama YETERSIZ', 'A2.36/B2.40/ort2.53/zehir2.83; fark +0.295 < esik +0.300 (ISSKAL); saf-ortalama ebeveynden kotu; trimmed-mean + tolerans-kalibrasyonu sirada', 'zehir-sonuc.log', 'deney-kapandi, tasarim-revize', 'denetci'),
 ('CR', 'CR-2026-030', '2026-09-20T00:00:00Z', 'Gomme-sunucu yenileme: uretim 20x', '1241/1242/1247 taze (2.4-gunluk eskimi); gecikme 20sn->0.3sn; ~9/sn->~177/sn; bekci-uyku-koku (R-30)', 'latency+DB+nvidia', 'CANLI', 'denetci'),
 ('FR', 'FR-2026-017', '2026-09-20T00:00:00Z', 'Bekci rolling-restart pipeline-bagimli (uyur)', 'wiki_embed_par yoksa restart yok; 2.4-gunluk sunucu 20-60x yavasladi; duzeltme-spec: hat-14 gate-kaldirma', 'embed_bekci.sh:11-14', 'acik (R-30)', 'denetci'),
 ('CR', 'CR-2026-031', '2026-09-20T12:00:00Z', 'Dalga-deploy-2 + trimmed-3-olumsuz + yedek-5li', 'B35/B36/B39/R-08 canli (HMAC seffaf, 0-hata); median n=3 YETERSIZ (2.658); C-egitimi basladi; rotasyon 7->5', 'journal+DB+olcum', 'CANLI/kismi', 'denetci'),
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
