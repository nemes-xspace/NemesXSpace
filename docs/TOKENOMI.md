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

## 2. Emisyon (halving — Bitcoin modeli, KİLİTLİ 09 Eyl + ETH-ölçeği 11 Eyl)

- Batch = 20 doğrulanmış kanıt. Era = **50.000.000.000 batch** (`HALVING_BATCH`
  mainnet değeri; kodtaki 5M testnet değeridir).
- Era ödülü (batch başına): era1 **0,0008 NEMES** (800 mikro) → era2 0,0004 →
  era3 0,0002 → … (yarılanma).
  - Era1 toplamı: 50B × 0,0008 = 40M. Kuyruk toplamı ≈ 80M (tavan içi).
  - Era süresi hedefi ~2,7 yıl (1 milyon madenci × ~50 batch/gün = 50M batch/gün
    temposunda; Bitcoin'in 4 yıl ritmine denk).
  - Kodtaki `BATCH_ODUL_TABAN_MIKRO = 2.000` **testnet değeridir**;
    mainnet açılışında 800 mikro + `HALVING_BATCH` 50B olarak
    güncellenecek (tek commit, ikisi birlikte).
- **Kuyruk tabanı (Bitcoin kuralı):** era ödülü **0,000005 NEMES/batch altına inmez.**
  Bitcoin'de blok ödülü bitince madenci fee ile yaşar; bizde de kuyruk bitince
  madenci H havuzuyla (ücret piyasası) yaşar. Ödül asla sıfırlanmaz — tıpkı
  Bitcoin/Ethereum'da bitmediği gibi.
- **Tavan koruması:** kuyruk + hazine + ekip + likidite toplamı 210M tavanı
  aşamaz; tavana 1M kala kuyruk orantısal kısılır (kod kuralı, mainnet'te).
  Pratikte tavana ulaşmak 40+ yıl sürer — o gün fee piyasası (H) ana gelir olur.
- Era süresi batch ile tanımlıdır (zamana değil). Kural: `HALVING_BATCH ≈
  aktif madenci × 50 × 1000` (1000 günlük era hedefi). 1M madencide 50B ≈ 2,7 yıl;
  ağ 100K'da kalırsa era ~27 yıla şişer, 10M'a koşarsa ~100 güne iner —
  bu yüzden değer mainnet lansmanında gerçek sayıma göre kilitlenir.

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
*Revizyon 11 Eyl 2026 (ETH-ölçeği): era 500M → 50B batch, era1 ödül 0,08 →
0,0008 (800 mikro), taban 0,0005 → 0,000005. Gerekçe: ETH-2021 ölçeği (~1M
madenci) baz alındı; era1 toplamı yine 40M, kuyruk yine ≈80M, taban-era yine
250K — tüm tavan matematiği korundu, sadece süre 100 kat uzadı.*
