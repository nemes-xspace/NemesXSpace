# Tokenomik Stres Tablosu — 16 Eyl 2026 (hesap, kod yok)

> Varsayımlar (ölçülü): madenci başı 5 kanıt/sn; batch 20 kanıt;
> testnet ödül 2000 mikro/batch + 5M halving; mainnet 800 mikro + 50B halving;
> tavan 210M (genesis 130M + era toplamı 80M).
> Formüller: batch/sn = N/4; coin/sn = batch/sn × ödül; halving süresi =
> HALVING_BATCH / (batch/sn).

## Testnet (değer: oyun parası, amaç: mekanik prova)

| Madenci | Kanıt/sn | Batch/sn | Coin/gün | Halving aralığı | Hüküm |
|---|---|---|---|---|---|
| 3 (bugün) | 15 | 0.75 | ~130 | ~77 gün | sakin |
| 100 | 500 | 25 | ~4.300 | ~2,3 gün | halving komedisi başlar |
| 1.000 | 5.000 | 250 | ~43.000 | ~5,5 saat | DB de ölür (bkz. aşağısı) |

## Mainnet (800 mikro, 50B halving) + §2b freni — DÜZELTME 17 Eyl

> Önceki sürümde 100 kat çarpım hatası vardı (17 Eyl akşam denetiminde
> yakalandı). Aşağıdaki tablo frenli gerçektir. Frensiz ham formül:
> madenci başı 17.28 coin/gün (0.25 batch/sn × 800 mikro × 86400).
> Fren: günlük dağıtım ≤ 27.360 coin (HALVING_BATCH/1460 kuralı) → era-1
> 40M/27.360 ≈ 4 yıl (madenci sayısından bağımsız). Fren ~1.600 madencide
> devreye girer; üstünde madenci-başı verim 1/N seyrelir.

| Madenci | Coin/gün | Era-1 | Madenci-başı/gün | Hüküm |
|---|---|---|---|---|
| 100 | ~1.728 | ~63 yıl (frensiz) | ~17 | sağlıklı |
| 1.000 | ~17.280 | ~6,3 yıl (frensiz) | ~17 | sağlıklı |
| 10.000 | ~27.360 (tavan) | 4 yıl (fren) | ~2,7 | fren devrede |
| 30.000 | ~27.360 (tavan) | 4 yıl (fren) | ~0,9 | fren + dağıtıcı federasyonu ister |
| 1.000.000 | ~27.360 (tavan) | 4 yıl (fren) | ~0,027 | verim sembolik; mesh + federasyon şart |

## Altyapı yükü (madenci sayısına göre) — DÜZELTME 17 Eyl

> Ölçüm bazlı (17 Eyl rampa testi: dağıtıcı ~370 gorev/sn tavan, p99 370ms;
> canlı miner 0.76 istek/sn; B14 sonrası kanıt POST'u 20'de 1). Eski tablonun
> HTTP sayıları toplu-öncesiydi.

| Madenci | HTTP istek/sn | DB yazma/sn | Hüküm |
|---|---|---|---|
| 3 | ~2 | ~350 | boşta |
| 100 | ~80 | ~6.600 | SQLite WAL rahat |
| 460 | ~370 | ~30.000 | **dağıtıcı tavanı (ölçüldü)** → üstü kuyruk/federasyon |
| 1.000 | ~800 | ~66.000 | tek komuta imkansız (claim-kirası veya federasyon) |
| 10.000 | ~8.000 | ~660.000 | federasyon + P2P şart |
| 30.000 | ~23.000 | ~2M | mesh + federasyon şart (claim-kirasıyla dağıtıcı ~230/sn'ye iner) |

## Sonuçlar (kilitli kararlara etkisi YOK, eşiklere etkisi VAR)
1. Mainnet sabitleri (800/50B) + §2b freni era'yı korur (4 yıl); 10K+ senaryoda
   sorun era değil **madenci-başı verim** (~2,7→0,027 coin/gün). 1.1 provasında
   karar: verim tabanı (anti-toz eşiği) + fren parametreleri.
2. Tek komuta ~460 madenciye kadar taşır (ölçüldü 17 Eyl); üstü claim-kirası
   (B13 devamı) veya federasyon ister (Faz 2).
3. Testnet parametreleri (2000/5M) bilinçli cömert — mainnet provasında
   karıştırılmaması için 1.1 checklist'ine "testnet sabitleri öldü" maddesi.
