# Uçta Üretim Döngüsü — TASARIM v1 + FİZİBİLİTE (B22, 17 Eyl 2026)

> Hedef (§6 KENDİ KENDİNE ÖĞRENME): bilgiden bilgi üreten mesh.
> v1 durumu: MEKANİK İSPATLANDI, kalite ağırlık bekliyor.

## İspatlanan (17 Eyl, bu makine)

- Qwen3-0.6B-Q8_0 (639MB) indirildi (`/srv/beyin/model-uretim/`), demet
  ikilisiyle :1258'de servis edildi (deney portu, sonra kapatıldı; canlı
  embed 1241/1242/1247), wiki bağlamıyla RAG→üretim çalıştı
  (60 token/328ms, 191 tok/s GPU). Döngü mekaniği ispatlı.
- Kalite: base model geveze/tekrarlı (beklenen). Üretim kalitesi için
  Instruct-ağırlık gerekir (indirilecek) veya base+adapter birleştirme
  (adapter-v01 mevcut, 35MB LoRA — birleştirme hattı yok).

## Hedef mimari (Faz 2)

```
merak/teklif (B21) -> baglam topla (RAG: FAISS + wiki, daginik)
  -> GPU madencide URET (kucuk model, teacher: buyuk acik model periyodik damitim)
  -> 2-of-3 tanik mutabakati (mesh, stake'li)
  -> bilgi defteri (yeni vektorler + kaynak zinciri)
  -> periyodik damitim (ogretmen -> ogrenci, QLoRA ucta)
```

## Kurallar (B15/B21 deseni tasinir)

1. Üretim iddiası stake'li; 2-of-3 tanık olmadan deftere girmez.
2. Kaynak zinciri zorunlu (hangi bağlamlardan üretildi).
3. Tanık yalanı = slash (mevcut slash mekanizması genişler).
4. Kurucu epoch'u üretim konusunu sinirlayabilir (imha/affet deseni).
5. GPU'suz madenci uretime katilmaz (embed/denetim/depolama rollerinde kalir).

## B22b kalite merdiveni (18 Eyl, ölçüldü)

| Model | Davranış | Hüküm |
|---|---|---|
| 0.6B base | tekrar döngüsü ("veya bir içecek içmek" ×6) | mekanik ispat |
| 1.7B base Q8 | tutarlı Türkçe, sınırsız | ölçek iyileştirir |
| **1.7B Instruct Q4** | **sınırlı, gerekçeli, dış bilgiyi etiketli** | **tanık döngüsüne uygun** |

Modeller `/srv/beyin/model-uretim/` altında (0.6B-Q8, 1.7B-Q8-base, 1.7B-Instruct-Q4).
Sıradaki: tanık protokolü + üretim ekonomisi (yukarıdaki faz planı).
