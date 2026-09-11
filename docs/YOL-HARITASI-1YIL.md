# NEMES-X 1 Yıllık Yol Haritası (Eyl 2026 → Eyl 2027) — v1.1

**Durum:** taslak v1.1 — 10 Eyl 2026. Başlangıç fotoğrafı: testnet canlı (2 madenci, 99 NEMES, 976K kanıt), FAISS 23 index 3,4GB, cc100 embed sürüyor, site 8 dilde yayında, tokenomik + güvenlik + tüm mimari kararlar kilitli.

**İlke:** merkezi sunucu yok (Garanti Kuralı 6) — her faz dağıtımı artırır.
**Ölçü birimi her fazda aynıdır:** düğüm sayısı, vektör sayısı, haftalık batch, havuz büyüklüğü (H), bağımsız operatör sayısı.
**Efor birimi:** adam-gün (1 kişi × 1 tam iş günü ≈ 8 saat). **Bütçe:** nakit harcama (USD); token/treasury kalemleri ayrıca belirtilir.
**Not:** Kurucu geliştirici tek kişi olduğundan paralel yürümeyen işler bağımlılık haritasına göre sıralanır; "sen" tarafındaki işler kritik yol üzerindedir.

---

## FAZ 0 — Testnet'i Kapatma (Eyl – Eki 2026, 6-8 hafta)

**Hedef:** eksiksiz beyin + herkesin indirebildiği miner + ilk tohum ağı.

### Aylık Kırılım

**Eylül 2026 (kalan haftalar):**
- 0.1 cc100_tr embed → merge → FAISS (#24) başlangıcı (embed zaten akıyor, merge + index bu ayın ikinci yarısı)
- 0.3 Tohum-0 DNS + TLS + modem kurulumu (sen)
- 0.7 NemesXSpace diff temizliği başlangıcı
- 0.9 (eklendi) Yasal ön hazırlık — avukat görüşmesi randevusu ve brief

**Ekim 2026:**
- 0.2 caselaw embed (Illinois vol 1) — cc100 merge sonrası
- 0.3 tohum-0 dış erişim testi + prova
- 0.4 Operatör kiti v1 yayını
- 0.5 Miner indirme sayfası doğrulaması
- 0.6 6-dil gövde çevirileri (insan rötuşu)
- 0.7 diff temizliği bitiş + `testnet-kapanis` etiketi
- 0.8 Bant optimizasyonu (mikro-chunk)
- 0.10 (eklendi) Topluluk kanalları (Matrix/Discord + moderasyon kuralları)
- 0.11 (eklendi) Testnet regresyon test paketi
- 0.12 (eklendi) Monitoring/telemetri paneli (merkezi olmayan — düğüm log endpoint'leri)
- 0.13 (eklendi) Dokümantasyon sitesi (docs.nemes-x.*)

### İş Tablosu

| # | İş | Bitiş kriteri | Efor (adam-gün) | Bütçe (USD) | Sahip |
|---|---|---|---|---|---|
| 0.1 | cc100_tr embed → merge → FAISS (#24) | 24. index `wiki_cc100_tr.faiss` + örnek sorgu yeşil | 4 | 0 (mevcut donanım) | ben (akıyor) |
| 0.2 | caselaw embed (Illinois vol 1 hazır) | `wiki_caselaw.faiss` + pipeline turu TAMAM | 4 | 0 | ben (cc100 sonrası) |
| 0.3 | Tohum-0 canlı (ev makinesi) | `komuta.` DNS + TLS + dışarıdan kayıt→görev→kanıt turu yeşil | 3 | 30 (alan adı yıllık) | sen (DNS+modem) + ben (prova) |
| 0.4 | Operatör kiti v1 yayında | docs + 1 gönüllüde 2. düğüm ayakta | 5 | 0 | ben |
| 0.5 | Miner indirme sayfası doğrulama | Linux zip + WSL kılavuzu + SHA256, 3. parti makinede kurulum provası | 3 | 0 | ben |
| 0.6 | 6-dil gövde çevirileri (miner+manifesto) | topluluk + AI destekli, TR/EN önce, diğerleri kademeli | 6 | 0 (gönüllü) | sen + topluluk |
| 0.7 | NemesXSpace diff temizliği | C4-C8+P2P+güvenlik commit + etiket `testnet-kapanis` | 3 | 0 | sen (onay) |
| 0.8 | Bant optimizasyonu (mikro-chunk) | Ev upload dostu küçük parça + akıllı dağıtım (Gemini önerisi) | 5 | 0 | ben |
| 0.9 | Yasal ön hazırlık: avukat brief + ilk görüşme | yazılı görüş taslağı + risk listesi (eklendi) | 4 | 500 (ön danışma) | sen |
| 0.10 | Topluluk kanalları + moderasyon | Matrix/Discord + kurallar + ilk 20 üye (eklendi) | 3 | 0 | ben |
| 0.11 | Testnet regresyon test paketi | CI'da 20+ senaryo, her commit'te yeşil (eklendi) | 4 | 0 | ben |
| 0.12 | Telemetri/monitoring (dağıtık) | Düğüm sağlığı görünür, merkez yok, site 5 dk gecikmeli (eklendi) | 4 | 0 | ben |
| 0.13 | Dokümantasyon sitesi (docs.*) | Operatör + madenci + güvenlik bölümleri yayında (eklendi) | 5 | 0 | ben |

**Faz 0 toplam efor:** ~53 adam-gün · **Toplam nakit bütçe:** ~$30 (alan adı) — sıfır bütçe revizyonu (Eyl 2026)

**Metrikler (Eki sonu):** düğüm ≥5 (≥3 bağımsız operatör), vektör 60M+, FAISS 24-25, site 8/8 güncel.
**Çıkış kapısı:** 5 düğüm 7 gün kesintisiz + dışarıdan katılan 1 madenci ilk payını alırsa Faz 1'e geçilir.

---

## FAZ 1 — Mainnet Açılışı (Kas 2026 – Oca 2027, ~10 hafta)

**Hedef:** gerçek coin, gerçek havuz, gerçek kullanıcı.

### Aylık Kırılım

**Kasım 2026:**
- 1.1 Mainnet sabitleri commit'i (BATCH_ODUL, HALVING_BATCH, tavan kontrolü) — test ağında 1 hafta proofsuz
- 1.2 Genesis töreni hazırlığı (Treasury 100M + ekip kilidi 20M + likidite 10M adresleri, 3 imzalı duyuru taslağı)
- 1.8 (eklendi) Treasury multi-sig kurulumu (3/5 imza)
- 1.13 (eklendi) Hukuk görüşü nihai — token menkul değil teyidi

**Aralık 2026:**
- 1.2 Genesis töreni canlı (site+repo+mail duyuru)
- 1.3 Natif Windows .exe (imzalı installer, SmartScreen)
- 1.9 (eklendi) Blok explorer (community sürümü, salt-okunur)
- 1.12 (eklendi) Wallet v1 (CLI + basit GUI, soğuk imza)
- 1.11 (eklendi) Sybil direnci katmanı (IP/anahtar çeşitliliği katsayıları)

**Ocak 2027:**
- 1.4 HF model kartları + benchmark (adapter-v01 GGUF Q4)
- 1.10 (eklendi) Bug bounty programı açılışı
- 1.14 (eklendi) Mainnet provası (fork/salınım tatbikatı)
- 1.5/1.6/1.7 eşik arkası (bu ay sadece izleme)

### İş Tablosu

| # | İş | Bitiş kriteri | Efor (adam-gün) | Bütçe (USD) | Sahip |
|---|---|---|---|---|---|
| 1.1 | Mainnet sabitleri commit'i | `BATCH_ODUL 80.000 mikro` + `HALVING_BATCH 500M` + tavan kontrolü, test ağında 1 hafta proofsuz hata yok | 4 | 0 | ben |
| 1.2 | Genesis töreni | Treasury 100M + ekip kilidi 20M + likidite 10M, 3 imzalı duyuru (site+repo+mail) | 3 | 0 | sen (tören) + ben (teknik) |
| 1.3 | Natif Windows .exe | Standart imza (Let's Encrypt zinciri yeterli) + SmartScreen itibar birikimi + siteye ekleme | 10 | 0 | ben (CI kurulumu) |
| 1.4 | HF model kartları + benchmark | adapter-v01 GGUF (Q4) + 10 soruluk skor + kart yayında | 5 | 0 | ben |
| 1.5 | `/api/sohbet` demo (RAG) | EŞİK ARKASI (10 Eyl ilkesi): 10K madenci veya dev kapasite dolmadan yazılmaz | — | — | beklemede |
| 1.6 | Enterprise sayfası + fiyatlar | EŞİK ARKASI (fiyatlar kilitli, sayfa/kod eşikte) | — | — | beklemede |
| 1.7 | İlk Enterprise müşteri | EŞİK ARKASI | — | — | sen |
| 1.8 | Treasury multi-sig (3/5) + kilit sözleşmesi | Zincir üstü doğrulanabilir adresler, imza sahipleri açık (eklendi) | 4 | 0 | ben |
| 1.9 | Blok explorer (salt-okunur, community) | Mainnet blok/tx/pool görünür, ev makinesinden servis (eklendi) | 6 | 0 | ben + topluluk |
| 1.10 | Bug bounty programı (nakitsiz) | Kapsam yayında; ödül = itibar tablosu + erken madenci bonusu, nakit yok (eklendi) | 3 | 0 | ben |
| 1.11 | Sybil direnci katmanı | Anahtar/IP/coğrafya çeşitliliği katsayısı canlı (eklendi) | 6 | 0 | ben |
| 1.12 | Wallet v1 (CLI + basit GUI) | Gönder/al + soğuk imza + çoklu hesap (eklendi) | 8 | 0 | ben |
| 1.13 | Hukuk görüşü (kapsam daraltıldı) | Mainnet öncesi tek konuluk sabit ücretli görüş (sadece token sınıflandırması); tam paket gelire ertelendi (eklendi) | — | 500 (hedef tavan, pazarlık) | sen |
| 1.14 | Mainnet provası (fork/salınım) | Simüle fork + rollback + iletişim planı tatbikatı (eklendi) | 5 | 0 | ben |

**Faz 1 toplam efor:** ~54 adam-gün · **Toplam nakit bütçe:** ~$500 (hukuk tavanı; pazarlıkla 0 hedeflenir)

**Metrikler (Oca sonu):** düğüm ≥25 (≥10 bağımsız), madenci ≥100, H > $0 (ilk dolar!), site trafiği 10K/ay.
**Çıkış kapısı:** H ≥ $500/hafta × 4 hafta = Faz 2 otomatik (TOKENOMI §6).

---

## FAZ 2 — Büyüme (Şub – May 2027, ~16 hafta)

**Hedef:** ağ kendi kendini taşır; ilk nakit payout.

### Aylık Kırılım

**Şubat 2027:**
- 2.1 İlk nakit payout töreni hazırlığı (muhasebe + TX imzalama akışı)
- 2.3 P2P sertleştirme (NAT delme, varsayılan açık dinleme)
- 2.7 (eklendi) CEX başvuru dosyaları (opsiyon)

**Mart 2027:**
- 2.1 İlk nakit payout töreni canlı (TX ID'ler sitede)
- 2.4 İtiraz penceresi + auditor slotu (24s soğuma)
- 2.11 (eklendi) Reputation sistemi v1 (operatör skorları)
- 2.9 (eklendi) Governance forum (RFC süreci)

**Nisan 2027:**
- 2.2 DEX likidite havuzu (10M likidite bütçesi)
- 2.5 2. corpus dalgası başlangıcı (paracrawl mirror + wet_en)
- 2.8 (eklendi) Marketing kampanyası #1
- 2.12 (eklendi) Anti-fraud araçları (grafik analiz, anomali)

**Mayıs 2027:**
- 2.5 2. corpus dalgası bitişi (+20M vektör)
- 2.6 Mobil izleme uygulaması (karar verilirse)
- 2.10 (eklendi) Bağımsız denetim (akıllı sözleşme + ağ güvenliği)

### İş Tablosu

| # | İş | Bitiş kriteri | Efor (adam-gün) | Bütçe (USD) | Sahip |
|---|---|---|---|---|---|
| 2.1 | İlk nakit payout töreni | İlk H dağıtımı, TX ID'ler sitede | 4 | 0 | ben + sen (duyuru) |
| 2.2 | DEX likidite havuzu | 10M likidite bütçesiyle ilk havuz + fiyat ekranı | 7 | 0 nakit (10M NEMES treasury) | sen (onay) + ben (teknik) |
| 2.3 | P2P sertleştirme | `--p2p-dinle` varsayılan açık, NAT delme rehberi, 50+ düğümde mesh testi | 8 | 0 | ben |
| 2.4 | İtiraz penceresi + auditor slotu | 24s soğuma + slot kısıtı canlıda | 5 | 0 | ben |
| 2.5 | 2. corpus dalgası (paracrawl mirror + wet_en parçalı) | +20M vektör, FAISS genişler | 10 | 0 | ben |
| 2.6 | Mobil izleme uygulaması (opsiyonel) | Pay/havuz takibi (madencilik yok, sadece izleme) | 12 | 0 (kişisel zaman) | karar (sen) |
| 2.7 | CEX başvuru dosyası (opsiyon) | 2 borsa ile ön görüşme + KYC paketi (eklendi) | 6 | 0 | sen |
| 2.8 | Organik tanıtım #1 | X/Reddit/Discord içerik takvimi, 0 bütçe, emek bazlı (eklendi) | 8 | 0 | sen + ben |
| 2.9 | Governance forum + RFC süreci | İlk 5 RFC açıldı, 2'si oylamaya gitti (eklendi) | 5 | 0 | ben |
| 2.10 | Bağımsız denetim | ERTELETLENDİ (gelir sonrasına): yerine topluluk kod incelemesi + otomatik testler (eklendi) | — | 0 | ben |
| 2.11 | Reputation sistemi v1 | Operatör skoru (uptime×kanıt×yaş) canlı (eklendi) | 8 | 0 | ben |
| 2.12 | Anti-fraud araçları (grafik analiz) | Wash mining + Sybil küme tespiti (eklendi) | 6 | 0 | ben |

**Faz 2 toplam efor:** ~79 adam-gün · **Toplam nakit bütçe:** ~$0 (denetim ertelendi, tanıtım organik)

**Metrikler (May sonu):** düğüm ≥100, madenci ≥1.000, H ≥ $2K/hafta, uptime %99.
**Risk kapısı:** H büyümezse Faz 2 uzar — Treasury dilimleri devrede (tasarım gereği, panik yok).

---

## FAZ 3 — Ölçek (Haz – Eyl 2027, ~14 hafta)

**Hedef:** öğretmen düğümler + damıtma + 70B hattı.

### Aylık Kırılım

**Haziran 2027:**
- 3.1 K5 öğretmen programı başlangıcı (donanım/partner kararı — en uzun kurşun, erken başla)
- 3.5 (eklendi) 2. teknik el işe alım süreci
- 3.7 (eklendi) Governance oylama mekanizması v1

**Temmuz 2027:**
- 3.1 3+ sunucu düğümde V3/K2 çalışıyor, damıtma verisi üretiyor
- 3.6 (eklendi) v2.0 yol haritası taslağı
- 3.9 (eklendi) 3. corpus dalgası başlangıcı

**Ağustos 2027:**
- 3.2 NEMES-Free 7B v2 (damıtılmış) eğitim + benchmark
- 3.3 On-prem referans kurulum (1 şirkette kapalı devre)
- 3.8 (eklendi) Felaket kurtarma tatbikatı
- 3.10 (eklendi) Partnerlik anlaşmaları (2 partner hedefi)

**Eylül 2027:**
- 3.2 HF yayını
- 3.4 1. yıl raporu + v2.0 yol haritası resmi yayın
- 3.9 bitiş (+30M vektör)

### İş Tablosu

| # | İş | Bitiş kriteri | Efor (adam-gün) | Bütçe (USD) | Sahip |
|---|---|---|---|---|---|
| 3.1 | K5 öğretmen programı | Partner donanımı (üniversite/topluluk) veya kiralanmış spot GPU; satın alma yok | 12 | 0 (spot ~500 hedef tavan, onayla) | sen (donanım/partner) + ben (entegrasyon) |
| 3.2 | NEMES-Free 7B v2 (damıtılmış) | MMLU/GSM8K/HumanEval/TR-MMLU skorları + HF yayını | 20 | 0 (mevcut donanım / partner GPU) | ben |
| 3.3 | On-prem referans kurulum | 1 şirkette kapalı devre kurulum belgesi | 10 | 0 | sen |
| 3.4 | 1. yıl raporu | Sayılarla rapor (düğüm, vektör, H, payout toplamı) + v2.0 yol haritası | 5 | 0 | ben + sen |
| 3.5 | 2. teknik el (gönüllü modeli) | Açık kaynak katkıcı + pay bazlı teşvik, maaş yok (eklendi) | 8 | 0 | sen |
| 3.6 | v2.0 yol haritası taslağı (eklendi) | 2. yıl hedefleri + 70B hattı + ölçek planı | 4 | 0 | ben |
| 3.7 | Governance oylama mekanizması v1 (eklendi) | Zincir üstü/off-chain oylama, H sahiplerine ağırlık | 10 | 0 | ben |
| 3.8 | Felaket kurtarma tatbikatı (eklendi) | Tohum-0 düşüşü + treasury kaybı senaryosu prova | 4 | 0 | ben + sen |
| 3.9 | 3. corpus dalgası (eklendi) | +30M vektör, yeni diller (FR/DE/ES) | 10 | 0 | ben |
| 3.10 | Partnerlik anlaşmaları (eklendi) | 2 partner (üniversite/şirket) MOU imzalı | 6 | 0 | sen |

**Faz 3 toplam efor:** ~89 adam-gün · **Toplam nakit bütçe:** ~$500 (spot tavan, onayla)

**Metrikler (Eyl 2027, 1. yıl):** düğüm ≥300, madenci ≥5.000, H ≥ $10K/hafta, vektör 150M+, bağımsız operatör ≥50.

---

## Bağımlılık Haritası

```
0.3 tohum-0 ──→ 0.4 operatör ağı ──→ 1.x mainnet ──→ 2.1 payout ──→ 2.2 DEX
0.1/0.2 beyin ──→ EŞİK (10K madenci/dev kapasite) ──→ 1.5 sohbet ──→ 1.6/1.7 enterprise ──→ H büyümesi ──→ Faz 2
1.4 HF modeller ──→ 3.2 damıtma
3.1 öğretmenler (donanım/partner — en uzun kurşun, erken başla)
0.9 yasal ──→ 1.13 hukuk görüşü ──→ 2.7 CEX ──→ 2.2 DEX
1.8 multi-sig ──→ 2.1 payout ──→ 3.8 felaket tatbikatı
1.11 sybil ──→ 2.11 reputation ──→ 2.12 anti-fraud
0.12 telemetri ──→ 2.10 denetim ──→ 3.7 governance
```

## Karar Noktaları (takvimli)

| Tarih | Karar | Kim |
|---|---|---|
| Eki 2026 | Faz 0→1 geçiş (5 düğüm kriteri tuttu mu?) | sen |
| Kas 2026 | Mainnet sabitleri son gözden geçirme | sen |
| Ara 2026 | Hukuk görüşü nihai (token menkul mü?) — 1.2 öncesi şart | sen + avukat |
| Oca 2027 | Windows .exe imza sertifikası bütçesi (EV mi, standart mı?) | sen |
| Şub 2027 | Bug bounty ödül kademeleri | sen + ben |
| Mar 2027 | DEX zamanlaması (H yeterli mi?) | sen + veri |
| Nis 2027 | Bağımsız denetim firması seçimi | sen |
| Haz 2027 | K5 donanım yatırımı/partnerliği | sen |
| Tem 2027 | 2. teknik el işe alım onayı | sen |
| Ağu 2027 | v2.0 kapsamı (70B hattı gerçekçi mi?) | sen + ben |

## Riskler + Panzehir (Genişletilmiş)

| Risk | Olasılık | Etki | Panzehir (tasarımda var) |
|---|---|---|---|
| Tohum-0 ev kesintisi | yüksek | orta | Operatör ağı (0.4); ceza yok, kayma olur |
| H büyümez (müşteri yok) | orta | yüksek | Treasury dilimleri 2 yıl; Faz 1 uzar, ağ ölmez |
| Borsa listelememe | orta | orta | DEX havuzu bizde; CEX opsiyon |
| Hukuk (token menkul mü?) | orta | yüksek | Lansman öncesi hukuk görüşü (1.2 öncesi şart) |
| Ana geliştirici tek kişi | yüksek | yüksek | Docs + kayıt disiplini (bu dosyalar); 1. yılda 2. teknik el hedefi (3.5) |
| GPU/elektrik fiyat şoku | düşük | orta | Verimlilik katmanları (K katsayısı ayarlanabilir) |
| Akıllı sözleşme açığı | orta | yüksek | Bağımsız denetim (2.10) + bug bounty (1.10) (eklendi) |
| Sybil/wash mining saldırısı | yüksek | orta | Sybil direnci (1.11) + reputation (2.11) + anti-fraud (2.12) (eklendi) |
| P2P ağ bölünmesi (fork) | orta | yüksek | Mainnet provası (1.14) + felaket tatbikatı (3.8) (eklendi) |
| Windows/macOS imza sorunu | orta | düşük | EV sertifika + SmartScreen itibar birikimi (1.3) (eklendi) |
| Çeviri kalitesi düşük | orta | düşük | İnsan rötuşu (0.6) + ana dil konuşurlarından geri bildirim (eklendi) |
| DEX likidite kaçışı (rug) | düşük | yüksek | Multi-sig + kilit sözleşmesi (1.8); likidite kilidi (eklendi) |
| Regülasyon şoku (yerel yasak) | düşük | yüksek | Merkezi sunucu yok → kapatılacak ofis yok; açık kaynak + dağıtık (eklendi) |
| Öğretmen düğüm donanım arızası | orta | orta | Partner çeşitliliği (3+ düğüm) + yedek donanım bütçesi (3.1) (eklendi) |
| Topluluk yönetim krizi (mod) | düşük | orta | Net kurallar (0.10) + şeffaf moderasyon logu (eklendi) |
| Efor/bitiş tarihi kayması | yüksek | orta | Aylık kırılım + karar noktaları; her faz kapanışında revizyon (eklendi) |

## Bütçe Özeti (Nakit)

| Faz | Efor (adam-gün) | Nakit bütçe (USD) | Token/treasury kalemi |
|---|---|---|---|
| Faz 0 | ~53 | ~2.330 | — |
| Faz 1 | ~54 | ~3.400 | Genesis likidite 10M NEMES |
| Faz 2 | ~79 | ~13.000 | DEX likidite 10M NEMES |
| Faz 3 | ~89 | ~18.000 | — |
| **Toplam** | **~275 adam-gün** | **~$1.000 tavan (hedef $30)** | 20M NEMES (likidite — token, nakit değil) |

*Not (sıfır bütçe revizyonu Eyl 2026): nakit harcama hedefi $0, tavan ~$1.000 (alan adı + pazarlıklı hukuk + opsiyonel spot). Eski ~$36K tahmin geçersizdir. Paralı kalemlerin tamamı topluluk/emek modeline çevrildi; denetim ve işe alım gelire ertelendi.*

---

*Son güncelleme: 10 Eyl 2026 · v1.1. Her faz kapanışında bu belge + DURUM birlikte güncellenir. Aylık kırılım, efor ve bütçe tahminleri ilk kez bu sürümde eklenmiştir; kilitli kararlara dokunulmamıştır.*
