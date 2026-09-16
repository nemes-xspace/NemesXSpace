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

## Mainnet (800 mikro, 50B halving)

| Madenci | Coin/gün | Era-1 ömrü (40M) | Hüküm |
|---|---|---|---|
| 100 | ~17 | ~6,3 yıl | sağlıklı |
| 1.000 | ~170 | ~230 gün | hızlı ama yaşanır |
| 10.000 | ~1.700 | ~23 gün | era buharlaşır |
| 30.000 | ~5.200 | ~8 gün | tavan 210M ~1 yılda biter |

## Altyapı yükü (madenci sayısına göre)

| Madenci | HTTP istek/sn | DB yazma/sn | Hüküm |
|---|---|---|---|
| 3 | ~1 | ~50 | boşta |
| 100 | ~520 | ~1.500 | SQLite WAL rahat |
| 1.000 | ~5.200 | ~15.000 | **SQLite tavan bölgesi** → federasyon şart |
| 10.000 | ~52.000 | ~150.000 | tek komuta imkansız (P2P + Postgres + toplu-kanıt) |

## Sonuçlar (kilitli kararlara etkisi YOK, eşiklere etkisi VAR)
1. Mainnet sabitleri (800/50B) 100-1.000 madencide sağlıklı; 10K+ senaryoda
   era-1 ~1 aydan kısa sürer → 10K eşiğinde **ara-halving gözden geçirmesi**
   takvime yazıldı (1.1 provasında karar).
2. Tek komuta ~1.000 madenciye kadar taşır; üstü federasyon ister (Faz 2).
3. Testnet parametreleri (2000/5M) bilinçli cömert — mainnet provasında
   karıştırılmaması için 1.1 checklist'ine "testnet sabitleri öldü" maddesi.
