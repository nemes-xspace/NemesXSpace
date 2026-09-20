# FedAvg-G1 Tasarımı — 2-Madenci Adaptör-Değişimi (19 Eyl 2026)

> Girdi: G0-repro (tek-kart QLoRA çalışıyor) + DiLoCo-sim (koşuyor).
> Çıktı-hedef: ilk federe-adaptör (kayıp-geçitli).

## Protokol (tur)

1. KOMUTA TUR-AÇAR: `tur_id`, baz-adaptör-hash'i, veri-dilimi (madenci-başı
   disjunkt shard), kayıp-eşiği, son-tarih. Duyuru: gossip `nemes/egitim`
   (yeni topic) + HTTP `/api/egitim/tur` (imzalı-duyuru patterni).
2. MADENCİ EĞİTİR: yerel QLoRA (sabit hiper-parametre: r16/a32/lr2e-4,
   tohum-tur_id). Çıktı: adaptör-delta (safetensors, ~35-120MB).
3. MADENCİ GÖNDERİR: delta + eğitim-logu (adım, kayıp-eğrisi) + imza.
   Aktarım: komuta-relay (mevcut parca-relay mime-uyumlu) — mesh-DHT Faz-2.
4. KOMUTA BİRLEŞTİRİR: FedAvg (örnek-ağırlıklı ortalama) → aday-adaptör.
5. GEÇİT: aday, tutulan-değerlendirme-setinde baz'dan KÖTÜ DEĞİLSE
   (`kayıp_aday <= kayıp_baz + tolerans`) KABUL; değilse RED + stake-slash
   (zehirli-gönderim). Değerlendirme-seti komutada gizli tutulur.
6. DAĞITIM: kabul-adaptör yeni-baz olur; madenciler çeker.

## Bizans-dayanım (G2'ye köprü)

- v1: kayıp-geçidi (kaba-filtre) + stake (ekonomik caydırıcılık).
- v2: trimmed-mean toplama + çoklu-değerlendirme-seti + itibar-ağırlığı.

## İletişim-bütçesi (dağıtık-ölçek)

- Tur-başı/madenci: ~50-120MB up + ~50-120MB down (adaptör).
- Ev-upload 10-50Mbps → tur-katılımı dakikalar-mertebesi; kademeli-turlar
  (straggler-dostu, FEDERE-EGITIM-TASARIM §topoloji ile tutarlı).

## Ölçüm-sonuçları (19-20 Eyl, mikro-ölçek)

| Varyant | Kayıp | Hüküm |
|---|---|---|
| A / B / C (tekil) | 2.36 / 2.40 / 2.48 | taban (dürüstler uzlaşır) |
| saf-ortalama (A,B) | 2.53 | ebeveynden KÖTÜ (mikro-ölçekte FedAvg faydasız) |
| trimmed-3 (A,B,Z) | 2.66 | YETERSİZ |
| trimmed-4 (A,B,C,Z) | 2.70 | YETERSİZ |
| zehir (karıştırılmış) | 2.83 | kayıp-geçidi sinyali +0.295 (yön-doğru) |

## Ders (tasarım-revizyonu)

1. Karıştırma-zehiri marjinal-dağılımı korur → medyan onu SEÇER (dağılım-koruyan
   zehir, medyana-dayanıklıdır). Medyan tek-başına savunma DEĞİL.
2. Doğru-sıra: ÖNCE kayıp-geçidi (zehiri ele), SONRA yaşayanların ortalaması.
   Geçit-eşiği kalibre edilecek (0.300 ıskaladı, 0.250 yakalardı — veri-boyutu
   büyüyünce yeniden ölçülecek).
3. Mikro-ölçekte (50-adım) toplama-faydası YOK; fayda ölçekte beklenir
   (daha-uzun-yerel-eğitim + daha-fazla-katılımcı). G2'de test edilecek.

## Açık sorular (uygulama-öncesi, güncellendi)

- Değerlendirme-seti boyutu/rotasyonu (sızma vs temsil).
- Tolerans-değeri (gürültü-payında sahte-red olmamalı).
- Adaptör-sürüm-çatışması (eşzamanlı-turlar).
