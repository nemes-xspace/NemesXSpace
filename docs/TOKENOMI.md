# NEMES-X Tokenomik — Coin Ekonomisi (v1.0-KİLİTLİ)
**Durum:** KİLİTLİ — 09 Eyl 2026 onayı. Sayılar mainnet sabitidir.
**Karar:** R1 (08 Eyl 2026) — coin + halving devam, API-hakkı elendi.
**Önceki belge:** `odul-stratejisi.md` (Faz 1 API-hakkı ifadesi bu belgeyle
yürürlükten kalkmıştır; faz mantığı korunmuştur).

---

## 1. Arz (sabit, değiştirilemez)

| Kalem | Miktar | Oran | Not |
|---|---|---|---|
| **MAX_SUPPLY** | 210.000.000 NEMES | %100 | Kod + beyaz kağıt tavanı |
| Genesis Treasury | 100.000.000 | %47,6 | Kilitli; §4 dilimleriyle havuza |
| Mining tail (tüm era'lar) | ~80.000.000 | %38,1 | §2 halving eğrisi |
| Ekip (2 yıl vesting) | 20.000.000 | %9,5 | Kilitli kontrat, aylık lineer |
| Likidite + ekosistem | 10.000.000 | %4,8 | Borsa havuzu, partner, airdrop |

- 1 NEMES = 1.000.000 mikro (`COIN_UNIT`, kodda). Defter mikro ile tutulur.
- Tavan kodlanır (arz endpoint'i tavanı reddeder); artırım = hard fork + duyuru.

## 2. Emisyon (halving — Bitcoin modeli, KİLİTLİ 09 Eyl)

- Batch = 20 doğrulanmış kanıt. Era = **500.000.000 batch** (`HALVING_BATCH`
  mainnet değeri; kodtaki 5M testnet değeridir).
- Era ödülü (batch başına): era1 **0,08 NEMES** (80.000 mikro) → era2 0,04 →
  era3 0,02 → … (yarılanma).
  - Era1 toplamı: 500M × 0,08 = 40M. Kuyruk toplamı ≈ 80M (tavan içi).
  - Era süresi hedefi ~3 yıl (10 bin aktif madenci temposunda; Bitcoin'in
    4 yıl ritmine denk).
  - Kodtaki `BATCH_ODUL_TABAN_MIKRO = 2.000` **testnet değeridir**;
    mainnet açılışında 80.000 mikro + `HALVING_BATCH` 500M olarak
    güncellenecek (tek commit, ikisi birlikte).
- **Kuyruk tabanı (Bitcoin kuralı):** era ödülü **0,0005 NEMES/batch altına inmez.**
  Bitcoin'de blok ödülü bitince madenci fee ile yaşar; bizde de kuyruk bitince
  madenci H havuzuyla (ücret piyasası) yaşar. Ödül asla sıfırlanmaz — tıpkı
  Bitcoin/Ethereum'da bitmediği gibi.
- **Tavan koruması:** kuyruk + hazine + ekip + likidite toplamı 210M tavanı
  aşamaz; tavana 1M kala kuyruk orantısal kısılır (kod kuralı, mainnet'te).
  Pratikte tavana ulaşmak 40+ yıl sürer — o gün fee piyasası (H) ana gelir olur.
- Era süresi batch ile tanımlıdır (zamana değil). Tahmini (10.000 aktif madenci
  × ~50 batch/gün ≈ 500K batch/gün): era1 ≈ **5-10 yıl değil, ~10 gün** —
  bu yüzden mainnet öncesi `HALVING_BATCH` **ağ büyüklüğüne göre revize edilir**
  (öneri: 500M batch/era, ≈ 3 yıl). Revizyon mainnet lansman kararı ile kilitlenir.

## 3. Pay → Ödeme

- Pay: `S = V × K` (V = doğrulanmış vektör, K = katman: 0,5 / 1 / 2 / 5 / 10).
- Haftalık havuz: `H = max(Enterprise_geliri × %50, Treasury_dilimi)`.
- Pay değeri: `P = H / toplam_pay`. Ödeme: `pay × P`. Minimum yok, toz birikir.
- Kesinti: %2 operasyon (doğrulama + gas). Min payout: $5 eşdeğeri.

## 4. Treasury Dilimleri (Faz 1 yakıtı)

- 100M Treasury, 104 haftada (2 yıl) lineer erir: **~961.500 NEMES/hafta** —
  AMA yalnızca `Enterprise_geliri × %50` dilimin altındaysa tamamlayıcı olarak
  (`max()` formülü). Gelir dilimi geçerse Treasury harcanmaz (korunur).
- 2 yıl sonunda Treasury'nin kalanı yakılır (tavan düşer) veya DAO kararıyla
  uzatılır. Varsayılan: **yakma**.

## 5. Enterprise Fiyatları (gelir tarafı — ÖNERİ)

| Paket | Fiyat | Hak |
|---|---|---|
| Starter | $49/ay | 1M token/ay Cloud API |
| Pro | $199/ay | 10M token/ay + RAG |
| On-Prem | $10.000/yıl | Kendi DC'sinde lisans + destek |

- `H` bu gelirlerin %50'sidir. Fiyatlar lansmanda kilitlenir.

## 6. Faz Geçişi (ölçülebilir kriter)

- **Faz 1 → Faz 2:** `H ≥ $500/hafta` üst üste **4 hafta** sürerse otomatik geçiş:
  nakit payout başlar, Treasury tamamlayıcı moda iner.
- Geçiş komuta log'una + DURUM.md'ye işlenir (şeffaflık).

## 6b. API Eşiği (10 Eyl 2026 ilkesi — KİLİTLİ)

- Herkese açık API (sohbet/RAG/Enterprise) **şu eşikten ÖNCE açılmaz:**
  **10.000 aktif madenci** VEYA **çok büyük şirkete AI sağlayacak kapasite**.
- Gerekçe: API erken açılırsa 3 kişi kullanır, destek yükü ağı yavaşlatır;
  ölçek önce, vitrin sonra. Demo/sohbet kodu bu eşiğe kadar bekler.
- Eşik tutunca: `/api/sohbet` + kota + Enterprise kodu sırayla yazılır.

## 7. Riskler (dürüstlük bölümü)

- Coin'in borsa değeri bugün **sıfırdır**; likidite likidite havuzu (10M) ile
  DEX'te kurulur, fiyat piyasaya aittir. Garanti yok.
- Erken madenci avantajı bilinçlidir (halving); geç gelenin payı küçüktür —
  ağ büyümesi için kabul edilen bedel.
- Hukuk: menkul kıymet değerlendirmesi + KYC/AML + vergi için lansman öncesi
  hukuk görüşü şart. Bu belge vaat değil tasarıdır.

---
*09 Eyl 2026 onayı ile kilitlendi. Testnet `0.002` ile devam eder; mainnet
açılışında sabitler bu belgeye geçirilir (tek commit).*
*Düzeltme 11 Eyl 2026: kuyruk tabanı 0,5 → 0,0005 (yazım hatasıydı; 0,5 başlangıç
0,08'in üstünde olduğu için kural uygulanamazdı).*
