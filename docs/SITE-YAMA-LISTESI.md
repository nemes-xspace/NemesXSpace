# Site Yama Listesi — TASLAK (uygulama + yayın onayı bekliyor)

> Repo: `nemes-xspace.github.io` (ayrı repo). Aşağıdaki yamalar hazırlanır,
> operatör onaylarsa o repoya işlenip push'lanır.

## Y1. Sayaçlar (8 dil × i18n JSON + HTML fallback)
- `assets/i18n/{en,tr,de,fr,es,ru,ar,zh}.json` satır ~537 `"l2": "40M+ ..."`
  → `"109M+ ..."` (8 dosya, aynı anahtar).
- `index.html:373` fallback metni: "distilled from 40M vectors" → "109M vectors".

## Y2. Ürün iddiası (DİKKAT — 16 Eyl kapsam kilidi: KALDIR/ERTELE)
- `pre_free_1` (8 dil, satır ~382): "40M vektörden damıtılmış 7B sınıfı".
- Kapsam kilidi gereği model iddiası ŞİMDİ YAYINLANMAZ: bu satır ya kaldırılır
  ya da "yakında" diline çevrilir. Sayı güncellemesi YAPILMAZ (yanıltıcı olur).
  Damıtma başlayınca gerçek baza göre yeniden yazılır.

## Y3. FAISS/EPOCH bölümü
- `index.st_faiss_n` + roadmap `road_5` ("FAISS index completion"):
  25 index / ~7.4GB notu eklenir.

## Y4. Tokenomik özeti
- Coin+halving kilitli kararın kamuya açık 3 satırlık özeti (tavan 210M,
  era1, halving) — TOKENOMI.md'den sadeleştirilir. Vesting cümlesi 1.2 ile uyumlu.

## Y5. Güvenlik notu
- Kör-ID + kanarya + kara liste 3 maddelik kamu özeti (detaysız, güven verici).
- Eşik dili: kod 0.98 — site %99 yazıyorsa düzeltilir.

## Uygulama sırası
Y1 → Y3 → Y5 → Y4 → Y2 (Y2+Y4 kelime onayı gerektirir).
