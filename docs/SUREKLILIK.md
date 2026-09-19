# Süreklilik — Tüzük, Envanter, Stratejiler (19 Eyl 2026)

> Varsayım (kullanıcı vermedi): ufuk 1/5/10 yıl ayrı ele alındı;
> kabul eşiği öneri olarak yazıldı (onaylı değil).

## BÖLÜM 1 — Tüzük (uygulanan)

Bus-factor ölçümü yapıldı (1), taze-klon provası icra edildi (clone +
`cargo fetch` exit 0), 3-2-1 sayımı yapıldı, tek-kaynak oranı ölçüldü
(%100 crates.io). Kâğıt strateji yok — her madde ölçüm + eylem taşır.

## BÖLÜM 2 — Bağımlılık envanteri (ölçülü)

| Tür | Bağımlılık | Kritiklik | Kaybolma ol. | Alternatif | Durum |
|---|---|---|---|---|---|
| İnsan | Tek operatör (bus=1) | KRİTİK | DÜŞÜK | yok | AÇIK |
| İnsan | Master kart+PIN tek elde | KRİTİK | DÜŞÜK | yok | AÇIK |
| Yazılım | crates.io tek-kaynak (%100, 520/520) | YÜKSEK | DÜŞÜK | vendor/ yok | AÇIK |
| Yazılım | libp2p 0.53 (tek sürüm hattı) | YÜKSEK | DÜŞÜK | B35 yükseltme | planlı |
| Yazılım | CUDA 11 blob (tescilli) | ORTA | DÜŞÜK | CPU/ggml (yavaş) | kısmi |
| Veri | Corpus: kamusal (Wiki/newscrawl) | DÜŞÜK | DÜŞÜK | yeniden-çekilebilir | sağlam |
| Veri | Yedek: 9 kopya, 1 lokasyon, 0 uzak | YÜKSEK | ORTA (yangın/hırsızlık) | uzak-kopya yok | AÇIK |
| Altyapı | Tek host, tek hat, CF tünel | KRİTİK | ORTA | Tohum-dışı 0 | AÇIK |
| Altyapı | GitHub (kod+Pages) | YÜKSEK | DÜŞÜK | yerel git tam-klon | kısmi |
| Tedarikçi | NVIDIA (sürücü), Zoho (e-posta) | DÜŞÜK | DÜŞÜK | — | izlemede |
| Lisans | Apache-2.0 (kod) + CUDA-EULA + model-teyitsiz | ORTA | DÜŞÜK | fork hakkı var | kısmi |
| Yasal | Yargı/yetki tanımsız, S7 açık | YÜKSEK | ORTA | — | AÇIK |
| Finansal | Nakit ~$30 harcanmış, gelir 0, rezerv tanımsız | YÜKSEK | — | — | AÇIK |
| Bilgi | Kafada bilgi: CF panel, modem, kart-PIN, VM-SPICE | KRİTİK | — | kısmen dokümante | AÇIK |
| Coğrafi | Tek lokasyon/ülke/dil-çekirdeği | YÜKSEK | DÜŞÜK | site 8 dil (içerik), altyapı 0 | kısmi |

## BÖLÜM 3 — Strateji matrisi (eylemli)

| Bağımlılık | Strateji | Eylem | Sorumlu | Süre |
|---|---|---|---|---|
| Tek operatör | YEDEKLE+BELGELE | 2. operatör adayı + runbook provası | sahip | 1 ay |
| Kart+PIN | YEDEKLE | Yedek kart (farklı fiziksel yer — KART-PROTOKOLU §6 emreder, icra yok) | sahip | 1 hafta |
| crates.io | YEDEKLE | `cargo vendor/` aynası (repo-dışı disk) + kilit-dosya arşivi | denetçi | 1 hafta |
| Yedek-tek-lokasyon | YEDEKLE | Şifreli uzak-kopya (haftalık, ayrı kimlik) | sahip | 2 hafta |
| Tek host | AZALT | Tohum-dışı 1 düğüm (0.4 görevi zaten var) | sahip | Faz 0 |
| GitHub | YEDEKLE | Aylık bare-klon 2. diske (tarihli) | denetçi | bu hafta |
| Kafada bilgi | BELGELE | CF/modem/PIN/VM erişim defteri (kasada) | sahip | 1 hafta |
| Gelir/rezerv | AZALT | Rezerv hedefi 6 ay işletme (~$500) + Enterprise hattı | sahip | 5 yıl ufku |
| Yasal | SÖZLEŞ | S7 avukat + yargı-kararı | sahip | 1 ay |

## BÖLÜM 4 — Kritik planlar (özet; detay görevlerde)

- İnsan: bus=1 → 2. operatör + vasiyet-mektubu (BÖLÜM 7'nin 1. eylemi).
- Yazılım: vendor aynası + B35/B36 takvimi.
- Veri: 3-2-1'e tamamlama (uzak-kopya) — restore KANITLI, replikasyon yok.
- Altyapı: ikinci lokasyon = tohum-dışı düğümle başlar.
- Finansal: 1 yıl (mevcut hızda ~$30/yıl — sürdürülebilir), 5 yıl (gelir şart), 10 yıl (kurumsal yapı şart) — projeksiyon, vaat değil.

## BÖLÜM 5 — Yıllık tatbikat takvimi

1. Kilit-kişi-kaybı (masa-başı + runbook-icra). 2. Tedarikçi-kes (crates.io'suz `cargo build --offline` vendor ile). 3. Host-kaybı (yedekten 2. makineye kur — RTO ölç). 4. Lisans-iptal (CUDA'sız CPU-only boot). 5. Veri-taşı (FAISS→alternatif okuma). 6. Fork-derle (icra edildi 19 Eyl: clone+fetch ✅; tam-derleme sıradaki). 7. Rol-değişim. 8. Sıfırdan-kur (KURULUM ile kör-kurulum). 9. Mevzuat-değişim (masa-başı). 10. 10-yıl (donanım-ömrü + format-çürümesi denetimi).

## BÖLÜM 6 — Göstergeler (bugünkü ölçüm → hedef)

1. Bus factor: 1 → ≥2. 2. Tek-tedarikçi: %100 → <%20. 3. Dokümantasyon: ~%70 → %100. 4. Tatbikat başarısı: 3/3 (clone/fetch/restore) → sürdür. 5. Geçiş süresi: ölçülmedi → ilk tatbikatta ölç. 6. Rezerv: tanımsız → 6 ay. 7. Yedek-doğrulama: 19 Eyl ✅ → aylık. 8. Kapalı-kaynak kritik: 2 (CUDA, model-lisans) → 1. 9. Coğrafi: 1/1 → 2/1. 10. Bilgi tazeliği: 19 Eyl → 30-gün kuralı.

## BÖLÜM 7 — Yönetici özeti

- En kritik 5: tek-insan, tek-host, yedek-tek-lokasyon, kafada-bilgi, yasal-tanımsızlık.
- En zayıf 3 süreklilik: insan-yedeği (0), coğrafi (0), finansal-rezerv (tanımsız).
- En acil 3 eylem: (1) vasiyet-mektubu + yedek-kart (1 hafta), (2) şifreli uzak-yedek (2 hafta), (3) vendor aynası + aylık bare-klon (1 hafta).
