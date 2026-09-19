# İyileştirme Ustalık Planı — P0→P4 (19 Eyl 2026)

> Tüzük: sahipsiz bulgu kapatılamaz; kanıtsız kapatma geçersiz; kabul
> yazılı-gerekçeli-süreli; her kapatma deftere işlenir.
> Kaynak: 1 kişi (operatör+sahip aynı), nakit ~$1K tavan, değişiklik
> penceresi: systemd sağlıklı + madenciler boşta anı.

## BÖLÜM 2 — Bulgu havuzu (tekilleştirilmiş)

| ID | Başlık | Kat. | Önem | Sahip | Son tarih | Durum |
|---|---|---|---|---|---|---|
| R-01 | Canlı API kilitleri (kova/CORS/limit/401/ara) | güvenlik | P0 | denetçi | 19 Eyl | ✅ KAPANDI (canlı-doğrulandı) |
| R-02 | B40 hash'siz indirici | güvenlik | P0 | denetçi | 26 Eyl | ✅ KAPANDI (fonksiyon silindi — URL'ler ölüydü, kaynak yok) |
| R-03 | B39 token-hash | güvenlik | P1 | denetçi | 26 Eyl | açık |
| R-04 | B35 libp2p yükseltme | güvenlik | P1 | denetçi | 31 Eki | açık |
| R-05 | B36 sqlx-0.8 | güvenlik | P1 | denetçi | 31 Eki | açık |
| R-06 | IP-logging canlı | gizlilik | P1 | denetçi | 26 Eyl | ✅ KAPANDI (canlı: `http-red 401 ip=`) |
| R-07 | SPOF tek-host | mimari | P1 | sahip | Faz 0 | açık (Tohum-dışı) |
| R-08 | B38 çift-denetim %12.8 | mimari | P1 | denetçi | 26 Eyl | ✅ KAPANDI-kod (atomik claim, canlıda; oran izlemede) |
| R-09 | B27 tier parametreleri | mimari | P2 | sahip | 31 Eki | tasarım-taslak |
| R-10 | Disk ~30 gün | operasyon | P1 | sahip | 19 Eki | izlemede (rotasyon aktif) |
| R-11 | systemd D-Bus kökü | operasyon | P1 | sahip | 26 Eyl | ✅ KÖK BULUNDU (aide 22sa tarama) + istisna yazıldı; doğrulama: bu gece timer |
| R-12 | Restore canlı-provası | veri | P2 | denetçi | 31 Eki | kopya-prova ✅, canlı-prova yok |
| R-13 | Retention/GDPR prosedürü | veri/yasal | P2 | sahip | 30 Kas | taslak-var |
| R-14 | Bus-1 + vasiyet + yedek-kart | insan | P0 | sahip | 26 Eyl | açık |
| R-15 | Kurtarma prosedürü (cüzdan-imzalı) | insan | P1 | sahip | 15 Eki | açık |
| R-16 | Win token rotasyonu | insan/güvenlik | P2 | sahip | VM-oturumu | ertelendi-gerekçeli |
| R-17 | S7 avukat (corpus-lisans) | yasal | P1 | sahip | 30 Kas | açık |
| R-18 | Uzak-şifreli-yedek | süreklilik | P1 | sahip | 3 Eki | açık |
| R-19 | Vendor aynası + bare-klon | süreklilik | P2 | denetçi | 3 Eki | açık |
| R-20 | Cüzdan-rotasyon rehberi | gizlilik | P2 | denetçi | 31 Eki | açık |
| R-21 | E-posta alarmı (gece) | operasyon | P2 | denetçi | 31 Eki | açık |
| R-22 | Era1-800 takvimi | mimari | P2 | sahip | mainnet-kararı | planlı |
| R-23 | Gossip send-kayıpları sayacı | mimari | P3 | denetçi | 30 Kas | açık |
| R-24 | Journal-retention cap | operasyon | P3 | denetçi | 30 Kas | açık |
| R-25 | Servis sandbox (ProtectSystem) | operasyon | P3 | denetçi | 30 Kas | açık |

Kabul-edilen (yazılı, süreli): audit.toml 7 istisna (gözden-geçirme: B35/B36 kapanınca);
testnet-0-STRICT (mainnet açılışına kadar); Win-token (VM-oturumuna kadar).
Reddedilen: YOK.

## BÖLÜM 3 — Bağımlılık haritası

- R-03 → R-15'i kolaylaştırır (hash sonrası kurtarma prosedürü değişir).
- R-07 → R-18'i kapsar (2. lokasyon = uzak-yedek hedefi olabilir).
- R-04 → h2/hickory/ring/webpki istisnalarını düşürür.
- R-05 → rsa/sqlx istisnalarını düşürür.
- Döngü yok.

## BÖLÜM 4 — Kayıtlar

Her bulgu için tam şablon `denetim/kayitlar.jsonl`'de FR/DR/IR olarak
kayıtlı; kapanış-kanıtı ilgili commit/test/prob çıktısı. Yeni kayıtlar
(R-02..R-25) bu dosyanın eki olarak RR-2026-006 ile deftere işlendi.

## BÖLÜM 5 — Ustalık planı (dalgalar)

- Dalga 1 (19-26 Eyl): R-01 ✅, R-02, R-03, R-06, R-08, R-11, R-14.
  Çakışma: R-02+R-03+R-06+R-08 aynı binary'de → tek deploy (Dalga-1-deploy).
- Dalga 2 (26 Eyl-31 Eki): R-04, R-05, R-07, R-09, R-10, R-12, R-13, R-15, R-17, R-18, R-19, R-20, R-21.
- Dalga 3 (3-6 ay): R-23, R-24, R-25 + tekrar-taramalar.
- Kaynak: dar-boğaz = tek kişi; deploy pencereleri gece-düşük-üretim.
- İletişim: bu dosya + defter (dış paydaş yok).

## BÖLÜM 6 — Doğrulama protokolü (bulgu-bazlı)

- R-02: `download_llama_server` SHA-pini + pin-yanlışlığında red testi.
- R-03: DB'de düz-token kalmadığı (`SELECT token` deseni) + login-akış 27/27.
- R-08: 20-dk 409-oranı <%2 (canlı journal sayımı).
- R-12: kopya-DB canlı-boot (T1 tekrarı, RTO<10dk).
- Tümü: bağımsız = ikinci-göz yoksa kırmızı-mini-tatbikat (P3 kuralı).

## BÖLÜM 7 — İzleme ve süreklilik

Kapananlar 30/90/180-gün regresyonunda (izleme.py + tatbikat).
Yeni bulgu akışı: P3/P5/P7 → FR → bu plan (çeyrek-güncelleme).

## BÖLÜM 8 — Göstergeler (bugün)

Kapanma: Dalga-0'da 30+/40 (önceki turlar). P0 açık: R-02, R-14.
Kabul: 3 (yazılı-süreli). Red: 0. Artık-risk: B35/B36 zinciri + bus-1.

## BÖLÜM 9 — Yönetici özeti

- En kritik 5: R-02, R-14, R-07, R-04/R-05, R-10.
- En hızlı kapanabilir 3: R-06 (deploy), R-08 (tasarım-hazır), R-02 (pin).
- En riskli 3 kabul: audit-istisnaları, STRICT-0, Win-token.
- En büyük belirsizlik: systemd-kökü (R-11) + gelir-takvimi.
