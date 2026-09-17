# Teklif Protokolü — TASARIM v1 (B21, 17 Eyl 2026)

> Amaç (§6 KENDİ KENDİNE ÖĞRENME): öğrenme hedefini merkez değil ağ belirler.
> v1 kapsam: **boşluk teklifleri** (kapsama eksiği, deterministik doğrulanabilir).
> Dış korpus teklifleri v2 (içerik yargısı gerektirir).

## Aktörler ve akış

```
madenci (bosluk bulur) --POST /api/teklif--> komuta (dogrular: kapsama <%50?)
   | Stake kilitlenir (ledger 'teklif-kilit')          | red: zaten kapaliysa
   v                                                  v
teklifler tablosu (durum='acik')               204-esdegeri red (ucretsiz)
   | supurme_dene ONCELIKLE teklif araliklarini dagitir
   v
aralik kapsama >= %80 -> stake iade + bulucu payi (%1, kapanan batch'lerden)
aralik 7 gun doldu -> stake iade (cezasiz), teklif kapanir
```

## Kurallar (kilitli)

1. **Doğrulanabilirlik:** teklif anında aralık kapsama oranı ölçülür
   (`kanitlar` yoğunluğu, sınırlı pencere sayımı). ≥%50 kapalıysa RED (stake alınmaz).
2. **Stake:** sabit 500 mikro (`TEKLIF_STAKE_MIKRO`), bakiyeden kilitlenir
   (ledger `teklif-kilit`, coin'den DÜŞER — çift harcama yok).
3. **Bulucu payı:** teklifteki batch kapanışlarında ödülün %1'i bulucuya
   (ledger `teklif-odul`). Basım YOK — mevcut ödülden yönlendirme.
4. **Kapanış:** kapsama ≥%80 → `tamam` + stake iade (ledger `teklif-iade`).
5. **Vade:** 7 gün (`TEKLIF_VADE_SN`) → `suresi-doldu` + stake iade. Slash YOK
   (v1 yumuşak; sahte teklifin maliyeti kilitli stake'in fırsat maliyeti).
6. **Öncelik:** supurme_dene önce açık teklif aralıklarını dener (tarih sırası),
   sonra normal süpürmeye düşer. Normal dağıtım etkilenmez.
7. **Şema:** migration 014, CREATE TABLE `teklifler` (Garanti-3 uyumlu).
8. **Geriye uyum:** yeni uçlar ek; eski akışlar aynen. Win .exe etkilenmez.

## Uçlar

- `POST /api/teklif` {corpus, baslangic, bitis, } → {teklif_id, durum} /
  400 (aralık geçersiz) / 409 (zaten kapalı) / 402 (bakiye yetmez).
  Limitler: 0 < bitis-baslangic ≤ 100.000; corpus = canlı corpuslar.
- `GET /api/bosluklar` → ilk 10 geri kalmış 10K-pencere + açık teklifler.
  Kaynaklar (ucuz sorgular): cursor−supurme makası + kapanmamış eski batch'ler.
- `GET /api/tekliflerim` → madencinin teklifleri (durum+paylaşım).

## Madenci tarafı (v1)

- `nemes-miner teklif --corpus --baslangic --bitis` (manuel komut).
- Otomatik merak v1b: 204 pencereleri + cursor sıçramalarını izleyip önerir
  (bu dosyada değil, B21b işi).

## Test planı

- Red (kapalı aralık), kabul+stake-kilidi, öncelikli süpürme, kapanış+iade+pay,
  vade-dolumu, çift-teklif (aynı aralık ikinci kez → mevcut açık teklife yönlendirme).
- Canlı prova: gerçek boşluk teklif edilir, süpürme önceliği journal'dan izlenir.

## v2 (kapsam dışı)

Dış korpus teklifleri (URL/hash + örneklem denetimi), slash'li sahtekarlık,
teklif piyasası (açık artırma), merak otomatı.
