# ÇİFTLİK REHBERİ — 100+ GPU ile NEMES-X (B25 paketi, 18 Eyl 2026)

> İlke: dürüst dev kazanır. Kimlik bedava, iş ispatlı; birim fiyat herkese
> eşit, hacim serbest. Kısıtlama YOK (tavan/kota/çarpan yok); koruma:
> spot-denetim + emanet + slash + anormallik sigortası.

## Toplu kayıt (tek çağrı, 200'e kadar)

```
POST /api/kayit/toplu  {"cuzdan":"TRC20","adet":100,"makine_onek":"raf"}
-> {"adet":100,"madenciler":[{"miner_id","token","makine_id"}...]}
```

Her token ayrı madenci (ayrı itibar/strike/pay defteri). Token'ları
env-dosyasına koyun (600), komut satırına/log'a yazmayın.

## Verimli üretim (ölçek ekonomisi protokolde)

- `--kira --kira-adet 2000`: 100 batch tek çağrı (dağıtıcı dostu).
- Toplu kanıt varsayılan (20 HTTP yerine 1).
- `--p2p-dinle`: boşta uyanma (poll gecikmesi yok).
- heartbeat 60sn + yetenek ilanı otomatik (filoda görünürlük).

## Filo izleme

```
GET /api/filo?cuzdan=TRC20   # filonun pay/coin/itibar/gpu listesi
```

## Ödeme matematiği (testnet bugünü)

- Batch (20 kanıt): 2000 mikro (0.002 NEMES); era1 mainnet: 800 mikro.
- Bayraklı kanıt payı emanette bekler (denetim geçince çözülür).
- Kötü kanıt: slash 2000 mikro + strike (3 = hat dışı).
- Kaba kural: GPU başına ~5 kanıt/sn → 100 GPU ≈ 43K kanıt/gün ≈
  ~43 NEMES/gün (testnet; mainnet era1 ~17/gün).

## Yasaklar (ağ sağlığı)

- Aynı işi çok kimlikle satmak = denetimde yakalanır (slash+strike).
- Anormal hızlanma = devre kesici duraklatır (inceleme sonrası açılır).
- Sorun: komuta logu + `/health` (`kesik` alanı) izlenir.
