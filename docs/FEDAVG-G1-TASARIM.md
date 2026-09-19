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

## Açık sorular (uygulama-öncesi)

- Değerlendirme-seti boyutu/rotasyonu (sızma vs temsil).
- Tolerans-değeri (gürültü-payında sahte-red olmamalı).
- Adaptör-sürüm-çatışması (eşzamanlı-turlar).
