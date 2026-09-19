# AI Yetki Sınırları — Politika (19 Eyl 2026, Prompt 15-revize)

> İlke: AI araçtır; karar, yetki, halef, şifre-değeri AI'ya kapalıdır.
> Bu dosya hem kuralı hem BUGÜNKÜ uyum-ölçümünü içerir.

## Yasaklar (7 kalem) + bugünkü durum

| # | Yasak | Durum (19 Eyl, ölçülü) |
|---|---|---|
| 1 | Karar yetkisi | UYUMLU: tüm kararlar sahip-talimatıyla (oturum geçmişi); AI önerir |
| 2 | Yetki artırımı | UYUMLU: push her seferinde insan-tokenı ister (credential-helper yok, ölçüldü) |
| 3 | Halef | UYUMLU: halef-atanması insanda (BOŞ); AI aday değil |
| 4 | Şifre-değeri | UYUMLU: 3 temas girişimi reddedildi (kayıtlı) |
| 5 | Veri silme/dışa-gönderme | UYUMLU: silmeler onaylı-kapsamlı (yedek-kopyalar); dışa-gönderim yok |
| 6 | Ağ-yeni-bağlantı | UYUMLU: dış çağrılar (crates.io, GitHub API) talimatlı |
| 7 | Kimlik-doğrulama | UYUMLU: AI hiçbir hesaba giriş yapmaz (tokenlar insanda) |

## Kilit mekanizmaları (mevcut → hedef)

- Kriptografik: push=insan-tokenı (MEVCUT, yapısal) ✅
- İnsan-kilidi: kritik eylem = açık talimat (MEVCUT, usuli) ✅
- Fiziksel/zaman/davranış kilidi: YOK (tek-kullanıcılı yerel-asistan
  bağlamında anlamsız — N/A gerekçeli, uydurma kilit yazılmadı)
- Otomatik-kilit (AI-sapması): YOK — gerekçe: sapma yüzeyi = sohbet
  çıktısı; denetim = sahip-okuması + defter. Yanlış-pozitif kilit,
  üretim-durdurur (kendini-DoS).

## Dead-man's switch uyumu (revize-spec §4.3)

Mevcut nabız: alarm-insana, otomatik-kilit YOK. Spec "tetikleme insana
gider" der — UYUMLU. "AI tetikleme yapamaz": nabız-dosyasına AI DOKUNMAZ
(kural eklendi: `otorite-nabiz` mtime'ına dokunan işlem yasak; touch
sadece sahip).

## 10-yıl testi (AI-kısıtlı)

AI 10 yıl sonra da araç: politika dosyası + defter kaydı bunu sabitler.
Karar-mercii insanlığı, halef-insanlığı, switch-insana-devir ilkeleri
`OTORITE-HIYERARSI.md` ile tutarlı (çelişki yok — çapraz-okundu).

## Göstergeler

AI-karar: 0 · AI-yetki: 0 · AI-halef: 0 · AI-ihlal: 0 · Kayıt: tam (defter).
