# Mimari Dayanıklılık Notu — Prompt 10 (19 Eyl 2026)

> Tüzük: her karar ödünleşimli; karmaşıklık son çare; geri-dönüş hazır.

## Uygulanan (kanıtlı)

1. **Derin-sağlık:** `/health` artık DB-ping + havuz-boyutu döner
   (`db_ms`, `havuz_boyut`). Canlı: `db_ms:0`.
2. **Kaos-ölçümleri:** miner restart→üretim 26sn; komuta ~2dk RTO.
   Madenciler kesintiyi atlatır (783/3dk) — graceful-degradation kanıtlı.
3. **Zaman-aşımı envanteri:** miner 120sn/30sn+retry(4); komuta 60sn.
4. **ADR-004:** ödünleşimli kayıtlı.

## Bilinçli-YAPILMADI (gerekçeli)

- Devre-kesici, komuta-retry, sharding, multi-region, blue-green,
  Redis, tracing: ölçek haklı çıkarmaz. Eşik: 10+ bağımsız düğüm.
- Mavi-yeşil eşdeğeri: `.bak` + sıralı-restart (RTO ~2dk).

## Bağlaşım

- miner→komuta: HTTP+JSON (gevşek). komuta→p2p: path (`../../`) SIKI.
- Döngüsel: YOK.

## Göstergeler

SPOF-kritik: 1. Sözleşme-ihlali: 0. MTTR: 26sn/2dk. Geri-dönüş: `.bak5`.
