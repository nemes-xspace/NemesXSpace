# Defter Federasyonu — TASARIM v1 (B16, 18 Eyl 2026)

> Tetikleyici (ölçüldü 17 Eyl): dağıtıcı ~370 görev/sn tavan (~460 madenci);
> kira ile ~20-30 kira/sn (~7-11K madenci). Üstü federasyon ister.
> Kod YOK (tasarım kilidi); tetiklenince Faz 2'de icra edilir.

## Hedef

Tek komuta defteri yerine: **bölge dağıtıcıları** (her biri bugünkü komuta
kodu + kendi DB'si) + **kök mutabakat** (epoch özetleri, kurucu imzalı).
Madenci sayısından bağımsız ölçek: bölge ekle, kapasite ekle.

## Bölgeleme

- Görev uzayı corpus-shard'larına bölünür (örn. newscrawl 0-10M: bölge-A,
  10-20M: bölge-B). Shard sınırları epoch başında ilan edilir (imzalı).
- Madenci kaydı bölge-yereldir; epoch içinde TEK bölgede çalışır (çift
  kayıt = strike; `miners` bölge etiketi taşır).
- Denetim BÖLGE-İÇİ kalır (kanıt + denetçi aynı bölgede; gecikme düşük).
  Bölgeler-arası denetim YOK (v1) — güven bölge sınırında netleşir.

## Kök mutabakat (epoch)

- Epoch = 24 saat. Her bölge epoch sonunda özet yayınlar:
  `{bolge, epoch, dagitilan_batch, odenen_mikro, emanet_mikro, slash_sayisi,
  defter_koku (blake3 merkle), onceki_ozet_hash}` — gossip `nemes/epoch`.
- Kurucu epoch imzacısı özetleri toplar, kök karar defterine işler
  (tören anahtarıyla imzalı epoch belgesi; tohum-0'da saklanır).
- Anlaşmazlık: bölge özeti gelmezse/uyumsuzsa o bölge "şüpheli" işaretlenir,
  madencileri komşu bölgeye göçer (yeniden kayıt, stake taşınır).

## Coin korunumu (kritik değişmez)

- Basım YALNIZCA batch kapanışında, bölge DB'sinde (mevcut matematik aynen).
- Bölge-ötesi transfer YOK v1'de (cüzdan bölge-etiketli bakiye görür;
  birleşme cüzdan katmanında Faz 3).
- Halving sayacı GLOBAL tutulur (kök): bölge batch'leri köke raporlanır,
  `tamamlanan_batch` toplamı halving'i tetikler (mevcut `HALVING_BATCH`
  semantiği korunur; bölge başına değil, ağ toplamına bakılır).

## Faz planı

1. **Hazırlık (şimdi, kodsuz):** `gorevler`/`kanitlar` şemasına `bolge` sütunu
   tasarımı (migration taslağı, uygulanmaz); epoch gossip konusu ayrılır.
2. **Tek-makine provası:** aynı hostta 2 komuta (farklı port+DB, corpus yarıda)
   + epoch özet değişimi (kopya DB'de, canlıya dokunulmaz).
3. **Canlı pilot:** 2. bölge yalnızca YENİ corpus'ta (risk izole).
4. **Kesme:** shard sınırları epoch belgesine girer, madenciler göçer.

## Açık sorular (1.1 provasında karar)

- Bölge sayısı formülü (madenci/10K başına 1?).
- Epoch süresi (24s vs 6s) + özet boyutu.
- Kök defter teknolojisi (SQLite zincir dosya mı, Postgres mi?).
