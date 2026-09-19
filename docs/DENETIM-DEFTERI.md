# Denetim Defteri — Tüzük ve Kayıt (19 Eyl 2026)

> Kural: kayıt yoksa olay yok sayılır. Bu dosya + `denetim/kayitlar.jsonl`
> (blake3-zincirli, `dogrula.py` ile doğrulanır) birlikte defterdir.

## BÖLÜM 1 — Tüzük (gerçekleşen)

1. Değişmezlik: git history (push'lu remote dahil). Yeniden-yazım (rebase)
   denetim kapsamındaki dallarda YASAK.
2. Zaman damgası: UTC (kayıtlarda ISO 8601; commit saatleri çapraz çapa).
3. Hash zinciri: `kayitlar.jsonl` — her kayıt `onceki` hash'i taşır;
   `dogrula.py` ile doğrulanır (şu an: SAGLAM, 42 kayıt).
4. İmza: `BEKLİYOR-kart-toreni` — sahte imza YOK, eksik açıkça yazılı.
   Tören sonrası RR-2026-002 ile kapatılacak.
5. Silme yok: düzeltme = yeni kayıt (iptal/düzeltme türüyle).
6. Yedek: git remote (GitHub) + `yedek/` DB zinciri; defter dosyasının
   ayrı-coğrafi kopyası YOK (eksik — BÖLÜM 6'da).
7. Bütünlük ritmi: her eklemede `dogrula.py` çalışır (CI'ya bağlanacak).

## BÖLÜM 2 — Kayıt türleri (kullanılan)

DR (karar) · CR (değişiklik) · FR (bulgu) · AR (onay) · IR (olay) ·
AL (erişim) · RR (gözden geçirme) · GEN (başlangıç).
Şablon: `no | tur | utc | baslik | detay | kanit | durum | sahip |
onceki | imza | hash` (kur.py üretir, elle yazılmaz).

## BÖLÜM 3 — Zincirleme

`kur.py` (üretir) + `dogrula.py` (doğrular). Algoritma: blake3
(`sort_keys` JSON). blake3 yoksa `sha256:` önekli geri-dönüş (başlıkta
işaretlenir — şu an blake3 aktif).

## BÖLÜM 4 — İzlenebilirlik sorgu matrisi (özet)

| Kayıt | Kim | Ne zaman (UTC) | Neyi | Neden | Kanıt |
|---|---|---|---|---|---|
| DR-2026-001 | hak-sahibi | 18 Eyl | Apache-2.0 | site vaadi | LICENSE:191, site commit |
| DR-2026-005 | denetci+sahip | 18 Eyl | deploy erteleme | systemd D-Bus | IR-2026-002 |
| CR-2026-001..013 | denetci | 18-19 Eyl | 13 değişiklik | bulgu-kapatma | commit hash'leri |
| FR-2026-001..015 | denetci | 18-19 Eyl | 15 bulgu | taramalar | kod:satır + canlı prob |
| AR-2026-002 | hak-sahibi | 18 Eyl | sohbet onayları | — | EKSİK (imzasız) |
| IR-2026-001..003 | denetci | 18-19 Eyl | 3 olay | — | PID/log/test |
| AL-2026-001 | — | 19 Eyl | YOKLUK (log yok) | zafiyet kaydı | — |
| RR-2026-001 | denetci | 19 Eyl | kuruluş | — | bu dosya |

(Tam liste: `kayitlar.jsonl` — 42 kayıt.)

## BÖLÜM 5 — Denetim takvimi

- Her commit: `dogrula.py` (elle; CI'ya bağlanacak).
- Haftalık: açık FR'lar + bekleyen AR'lar.
- Aylık: zincir + yedek-restore provası + yetki gözden geçirme.
- İmza töreni sonrası: RR-2026-002 (tüm `BEKLİYOR` kapanır).

## BÖLÜM 6 — Defterin kendi denetimi

1. Eksiksiz mi? HAYIR — erişim logu yok (AL-2026-001), sohbet onayları imzasız (AR-2026-002).
2. Tutarlı mı? EVET (42/42 zincir SAGLAM).
3. Zamanında mı? KISMEN (kayıtlar olaydan saatler sonra yazıldı; commit saatleri çapa).
4. Doğru mu? İddia edilen kanıtlar dosyada mevcut (spot-kontrol edilmedi — ikinci göz yok).
5. Erişilebilir mi? EVET (repo + remote).
6. Korunuyor mu? KISMEN (git değişmez; imza yok).
7. Yedekli mi? KISMEN (remote var; ayrı-coğrafi imza-kasa yok).
8. Okunabilir mi? EVET (JSONL + bu dosya).
9. Yasal uygunluk? [DOĞRULANMADI — çerçeve seçilmedi (KVKK/GDPR/ISO)].
10. Sızıntı riski? DÜŞÜK (defterde secret yok — token/hash yazılmadı).

## BÖLÜM 7 — Açık sorular

1. Yasal çerçeve hangisi? (seçilmedi)
2. Saklama süresi kaç yıl? (tanımsız)
3. İmza töreni ne zaman? (kart operatörde)
4. Ayrı-coğrafi defter kopyası nereye? (yok)
5. CI zincir-doğrulaması ne zaman bağlanacak? (yapılmadı)
