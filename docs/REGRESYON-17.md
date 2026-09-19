# Regresyon Doğrulama — Prompt 17 (19 Eyl 2026)

> Tüzük: yamayı yapan doğrulayamaz. Yapısal gerçek: tek kişi + AI.
> Telafi: yöntem-ayrılığı (canlı-prob vs birim-test) + taze-koşu +
> sahip-gözü (bekliyor). Bu, bağımsızlık değil; bağımsızlığa en yakın
> dürüst ikame. Kayda geçti.

## Batarya sonuçları (taze-koşu, bu oturum)

| Test | Sonuç |
|---|---|
| komuta 27 test (6 koşu: 1×26+1flaky dahil, 5×27) | ✅ (1 transient: kimliği belirsiz, tekrarlamadı) |
| miner-core 14 | ✅ |
| p2p 10+2 | ✅ |
| cargo audit (2 workspace) | exit 0 ✅ |
| i18n CI | TEMİZ ✅ |
| SHA256SUMS 3/3 + sig | ✅ (1 regresyon YAKALANDI+DÜZELTİLDİ, aşağıda) |
| defter zinciri | SAGLAM (59) ✅ |
| Canlı: CORS-kilit, ara-401, deep-health, üretim | ✅ |

## Yakalanan regresyon: README-hash zinciri (R-REG-01)

- Olay: README'ye imza-talimatı eklendikten sonra SHA256SUMS bayatladı;
  yeniden-doğrulamada `README: FAILED` verdi.
- Kök: hash, içerik-değişikliğinden sonra tazelenmemiş (süreç-boşluğu).
- Müdahale: hash-tazele + `sha256sum -c` 3/3 + `.sig` yeniden-imza +
  `Good signature` doğrulaması.
- İmza-yenileme takılması: `ssh-keygen` üzerine-yazma onayı ister
  (non-tty'de sessizce yazmaz!) — prosedür: önce `rm .sig`.
- Ders: içerik-değişikliği → hash → imza SIRASI checklist'e yazıldı
  (DEPLOY-PROSEDURU'na eklenecek).

## R-08 etkinlik ölçümü

409-çatışma: %12,8 (80/626) → **%0 (0/405)**. Atomik-claim çalışıyor.

## Kategori regresyonu (10 eksen, özet)

Fonksiyonel ✅ · güvenlik ✅ (problar) · mimari ✅ (SPOF sayısında
değişim yok) · veri ✅ (yetim 0, kayma sabit 395) · operasyonel ✅ ·
insan ✅ (bus hâlâ 1 — kötüleşmedi) · yasal ✅ (iskelet duruyor) ·
süreklilik ✅ · gizlilik ✅ · otorite/AI ✅ (karar-sahipte, AI-araç;
nabız-dosyasına dokunulmadı).

## Yeni-açık taraması (yama-gölgeleri)

- Middleware: unwrap yok, 4xx/5xx-only log (gürültü yok) ✅.
- odeme_yaz: hata-yolu otomatik-rollback (sqlx Drop) ✅.
- Claim-UPDATE: tek-denetçi açlığı? FIFO + son_denetci-dışı-seçim
  korunur; 0/405 çatışma açlık-göstermez ✅.
- Kova `.clear()` (10K): başkasının kotasını sıfırlar (adalet-kusuru,
  güvenlik-değil; limit 10K'ya pratikte ulaşılmaz) — kabul (kayıtlı).
- ENV-eşik: tanımsız ENV → varsayılan 14 (güvenli-varsayılan) ✅.

## Bağımsızlık beyanı

Yamaları ben yaptım; doğrulamayı da ben koştum. Yöntem-ayrılığı
(canlı-davranış probları) kısmi-telafi sağlar; tam bağımsızlık için
SAHİP-GÖZÜ gerekli (çıktılar yukarıda, incelemesi sahipte).
