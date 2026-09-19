# İyileştirme Ustalık Planı — P0→P4 (19 Eyl 2026, Rev-2: eski-kayıtlar hükmen-kapandı, yeni-kayıt disiplini)

> Tüzük: sahipsiz bulgu kapatılamaz; kanıtsız kapatma geçersiz; kabul
> yazılı-gerekçeli-süreli; her kapatma deftere işlenir.
> Rev-2 notu (sahip-emri): önceki tablo satırları TEK TEK hükme bağlandı;
> aşağıda güncel-hüküm vardır. Defter (ledger) ayrıca durur, silinmedi.
> Kaynak: 1 kişi, nakit ~$1K, pencere: systemd-sağlıklı an.

## BÖLÜM 2 — Bulgu havuzu (güncel-hüküm, 19 Eyl 21:00)

| ID | Başlık | Kat. | Önem | Sahip | Hüküm |
|---|---|---|---|---|---|
| R-01 | API kilitleri | güvenlik | P0 | denetçi | ✅ KAPANDI (canlı) |
| R-02 | İndirici-silme | güvenlik | P0 | denetçi | ✅ KAPANDI |
| R-03 | Token-hash | güvenlik | P1 | denetçi | ✅ KAPANDI-kod (HMAC-pepper + test; deploy bekler) |
| R-04 | libp2p 0.57 | güvenlik | P1 | denetçi | ✅ KAPANDI-kod (tek-satır API; test yeşil; deploy bekler) |
| R-05 | sqlx 0.8.6 | güvenlik | P1 | denetçi | ✅ KAPANDI-kod (27/27×3; deploy bekler) |
| R-06 | IP-logging | gizlilik | P1 | denetçi | ✅ KAPANDI (canlı) |
| R-07 | SPOF tek-host | mimari | P1 | sahip | AÇIK (Tohum-dışı) |
| R-08 | Atomik-claim | mimari | P1 | denetçi | ✅ KAPANDI-kod (409 %0 ölçüldü; deploy bekler) |
| R-09 | B27 parametreleri | mimari | P2 | sahip | AÇIK (karar-sende) |
| R-10 | Disk ~30 gün | operasyon | P1 | sahip | İZLEMEDE |
| R-11 | AIDE-kökü | operasyon | P1 | denetçi | ✅ KÖK+istisna; doğrulama bu-gece-timer |
| R-12 | Restore | veri | P2 | denetçi | kopya ✅; tam-yol DENENDİ-VAZGEÇİLDİ (aşağıda) |
| R-13 | Retention/GDPR | veri/yasal | P2 | sahip | taslak (S7'ye bağlı) |
| R-14 | Bus/vasiyet/kart | insan | P0 | sahip | PARÇALI: vasiyet REDDEDİLDİ ✅(hüküm) · halef-tatbikatı KAPANDI (emir) · yedek-kart AÇIK |
| R-15 | Kurtarma prosedürü | insan | P1 | sahip | AÇIK (taslak-yok) |
| R-16 | Win token | güvenlik | P2 | sahip | ERTELEDİ-gerekçeli (VM-oturumu) |
| R-17 | S7 avukat | yasal | P1 | sahip | AÇIK (brief hazır) |
| R-18 | Uzak-yedek | süreklilik | P1 | sahip | ENGELLİ (hedef-kararı yok) |
| R-19 | Vendor + cüzdan-rehberi | süreklilik | P2 | denetçi | ✅ KAPANDI (884M ayna + rehber) |
| R-20 | E-posta alarmı | operasyon | P2 | denetçi | ENGELLİ (SMTP-kimliği yok) |
| R-21 | Era1-800 | mimari | P2 | sahip | AÇIK (karar-sende) |
| R-22 | Gossip-sayacı | mimari | P3 | denetçi | AÇIK (sıralı) |
| R-23 | Journal-cap | operasyon | P3 | denetçi | ✅ KAPANDI (2G) |
| R-24 | Sandbox | operasyon | P3 | denetçi | ✅ KAPANDI (canlı, 0 hata) |
| R-25 | FAISS-PQ64 | mimari | P3 | denetçi | ✅ KAPANDI-negatif (PQ32'den kötü: 0.7 vs 2.7) |
| R-26 | DiLoCo-sim | mimari | P2 | denetçi | KOŞUYOR (A-egitimi) |
| R-27 | Swarm-fizibilite | mimari | P3 | denetçi | ✅ DEĞERLENDİRİLDİ (kurulabilir, Faz-2+ iş) |
| R-28 | FedAvg-G1-tasarım | mimari | P2 | denetçi | ✅ YAZILDI (uygulama-sonraki) |
| R-29 | Restore tam-yol | veri | P2 | denetçi | VAZGEÇİLDİ-gerekçeli (aşağıda) |

Kabul (yazılı-süreli): audit-istisnaları (B35/B36 düşünce revize) ·
STRICT-0 (mainnet'e-kadar) · Win-token (VM-oturumu) · git-ritmi-ifşası (6-ay).
Reddedilen: vasiyet-mektubu (DR-2026-007) · tam-yol-restore (R-29).

## R-29 gerekçesi (tam-yol-restore neden yapılmadı)

Denendi: servis-durdurma D-Bus-takıldı, `;` zinciri dosyayı taşıdı
(komuta açık-FD ile çalışmaya devam etti). Kurtarma: geri-taşıma,
sıfır-kayıp (5.29M kanıt doğrulandı). Hüküm: tam-yol-değişimi, D-Bus
istikrarsızken YASAK (kural); kopya-boot kanıtı yeterli sayıldı.
Kural-ihlali kayda geçti (IR): yıkıcı-işlem öncesi `&&` + durum-kontrolü.

## BÖLÜM 4 — Kayıtlar

Her bulgu için tam şablon `denetim/kayitlar.jsonl`'de FR/DR/IR olarak
kayıtlı; kapanış-kanıtı ilgili commit/test/prob çıktısı. Yeni kayıtlar
(R-02..R-25) bu dosyanın eki olarak RR-2026-006 ile deftere işlendi.

## BÖLÜM 5 — Ustalık planı (dalgalar, Rev-2 durumu)

- Dalga 1: R-01 ✅, R-02 ✅, R-03 ✅-kod, R-06 ✅, R-08 ✅-kod, R-11 ✅,
  R-24 ✅, R-25 ✅, PQ64 ✅-negatif, DiLoCo ⏳, Swarm ✅-değerlendirme,
  FedAvg-G1 ✅-tasarım. Bekleyen-deploy: R-03/R-04/R-05/R-08 (tek-pencere).
- Dalga 2: R-07, R-09, R-10, R-13, R-15, R-17, R-21, R-22, R-23.
- Engelli: R-16, R-18, R-20 (kimlik/hedef-kararı yok).
- Kapatıldı-emir: halef-tatbikatı, vasiyet (reddedildi).
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
