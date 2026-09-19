# Karşı İstihbarat — İz Envanteri, Tespit, Küçültme (19 Eyl 2026)

> Tüzük: görünmezlik değil, kontrollü görünürlük. Zorunlu iz korunur,
> gereksiz iz kaldırılır, yanıltıcı iz YOK (karar: bu aşamada üretilmedi).

## BÖLÜM 1 — Tüzük (uygulanan)

Sızıntı taraması icra edildi (site + zip + git geçmişi), OSINT yüzey
ölçüldü (DNS + headers + canlı dosyalar), anomali bakışı yapıldı
(süreç + log). Üçüncü tarafa dokunulmadı.

## BÖLÜM 2 — İz envanteri (ölçülü)

| Tür | İz | Kime görünür | Süre | Kontrol |
|---|---|---|---|---|
| Ağ | CF arkası gerçek IP (CF-Connecting-IP) | CF + komuta logu (yok, loglanmıyor) | — | loglanmadığı için ne görünür ne denetlenir (AL-2026-001) |
| Ağ | DNS nemes-x.space → 172.67.201.161 (CF) | herkes | sürekli | origin gizli ✅ |
| Kimlik | TRC20 cüzdan (3 madenci) | zincir + DB | kalıcı | rotasyon rehberi yok |
| Kimlik | Bearer token (düz-metin DB) | DB okuyucu | kalıcı | B39-aşırı |
| Uygulama | Miner UA `NEMES-Miner/0.2` | CF logu | istek-başı | WAF ikincil kural |
| Uygulama | `/health`+`/api/arz` herkese açık | internet | sürekli | tasarım (salt-okunur) |
| Veri | 8.6G DB + 9 yedek (tek lokasyon) | host erişimi | kalıcı | 600 izin ✅, şifre yok |
| Bulut | GitHub (3 repo: 2 public + 1 private) | public kısım herkes | kalıcı | secret taraması temiz ✅ |
| Cihaz | llama-server ×3 (CPU %100+%96) | host | süreç | normal (üretim) |
| Sosyal | X @NemesXSpace, tek e-posta | herkes | kalıcı | tek-ağız ✅ |
| Zaman | Timer ritmi (00:38 yedek, 15dk izleme) | davranışsal | — | öngörülebilirlik (düşük risk) |
| Paranoid | Commit mesajları (Türkçe, saatli) | public repo | kalıcı | çalışma-ritmi ifşa eder (kabul) |

## BÖLÜM 3 — İzleme tespiti (bu oturum)

1. Ağ: beklenmedik gelen YOK (ölçülmedi — CF paneli görülmedi) [DOĞRULANMADI].
2. Sistem: aide AKTİF (host IDS mevcut ✅); beklenmedik süreç YOK (llama/opencode/aide normal).
3. Kimlik: 401 patlaması YOK — ama ölçüm YANILTICIydı (eşleşmeler `2974401` gorev_id'leriydi, auth değil). Sonuç: brute-force görünürlüğü SIFIR (log yok) — kör nokta kayda geçti.
4. Fiziksel/sosyal: kapsam-dışı (gözlem yok).

## BÖLÜM 4 — İz küçültme (uygulanan + plan)

| İz | Yöntem | Etki | Maliyet | Sınır |
|---|---|---|---|---|
| Site dahili-yol | grep taraması (19 Eyl) | sızıntı yok ✅ | 0 | — |
| Git-token | geçmiş taraması (`-S ghp_`) | değer yok, 1 kelime-eşleşme ✅ | 0 | opencode.db harici iz (temizlenecek) |
| Zip secret | strings taraması | temiz ✅ | 0 | her sürümde tekrar |
| Auth-körlük | IP loglama (1 satır) | S3-tespiti açılır | ~0 | deploy bekler |
| Cüzdan-doxx | rotasyon rehberi | zincir-izi kırılır | düşük | yazılmadı (görev) |

## BÖLÜM 5 — Yanıltıcı iz politikası

ÜRETİLMEDİ. Gerekçe kaydı: mevcut tehdit profili (fırsatçı tarayıcı +
teorik içeriden) yanıltmayı gerektirmez; maliyet (güvenilirlik) >
fayda. Karar gözden-geçirme: 3 ayda bir.

## BÖLÜM 6 — Kendi-izleri denetimi (dış-göz simülasyonu)

Saldırgan 10 dakikada öğrenir: site (teknoloji yığını: Pages+CF),
miner zip (sürüm+binary), API rotaları (açık probing: /health, /arz,
/ara-14sn), token modeli (kayıt açık), ekip-büyüklüğü (commit ritmi:
tek kişi). Öğrenemez: gerçek IP, token değerleri, DB içeriği,
kart-PIN, iç topoloji (192.168 sızmamış ✅).

## BÖLÜM 7 — Döngü ve takvim

Günlük: süreç-anomali bakışı (izleme.py yanına 1 satır: top-CPU).
Haftalık: sızıntı-grep (site+zip+git). Aylık: OSINT-yüzey (DNS,
headers, canlı dosyalar). Yıllık: sosyal-mühendislik masa-başı.

## BÖLÜM 8 — Yönetici özeti

- En kritik 5 iz: düz-metin token, TRC20 cüzdan, tek-lokasyon DB,
  auth-körlük, commit-ritmi.
- En acil 3 tespit: IP-logging (kod hazır, deploy bekler),
  opencode.db token-izi temizliği, cüzdan-rotasyon rehberi.
- En büyük belirsizlik: CF-öncesi doğrudan erişim + WAF detayları
  (panel görülmedi — içeriden bakılmalı).
