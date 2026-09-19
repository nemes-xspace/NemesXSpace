# Parça Replikasyonu — Mevcut Durum (19 Eyl 2026, derin-tarama sonucu)

> Not: bu mekanizma kodda vardı ama dokümante değildi (19 Eyl taramasında
> bulundu). Tur-4 raporundaki dosya-atfı (mesh_audit) yanlıştı; sabit
> komuta-rs:67'dedir. İkisi de kayda geçti.

## Kural (kod-kanıtlı)

- `REPLIKA_HEDEF = 3` (komuta-rs:67): parça-başı canlı-kopya hedefi.
- Uygulama: onarım-planlayıcı (`HAVING canli < 3`) → `onarimlar` satırı →
  madenci relay'den çeker → hash-doğrular → kapatır + ödül alır.
- Canlılık: `last_seen` + `OLU_ESIK_SN` üzerinden (ölü-madenci kopyası sayılmaz).

## Canlı ölçü (19 Eyl)

- 30 parça, parça-başı 2 kopya (hedefin 1 altında).
- `onarimlar`: 60 tamam, 0 açık (mekanizma çalışıyor; hedefe ulaşamayan
  parçalar için yeni-tur planlaması izlemede).

## Kapsam-sınırı (dürüst)

- Bu sistem YEDEK-PARÇALARI kapsar (relay/depolama parçaları).
- Bilgi-index'inin (vektörler, FAISS) çok-makineli yedeği YOKTUR —
  o federasyon işidir (Faz B/C).
