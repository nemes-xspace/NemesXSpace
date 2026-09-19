# Definition of Done — DURUM (18 Eyl 2026, kanıtlı)

> Kaynak: `docs/MUTLAK-OTORITE.md` §7. Her madde: DURUM + KANIT + EKSİK.
> Lejant: ✅ kanıtlı | 🟡 kısmi | 🔴 eksik.

| # | Madde | Durum | Kanıt / Eksik |
|---|---|---|---|
| 1 | Acceptance criteria | 🟡 | §5 listesi var; formal AC dokümanı YOK → yazılacak |
| 2 | Testler geçti, kritik yol kapsamlı, flaky yok | ✅ | 27/27 (paralel 3/3 + tekli). Flake kökü öldürüldü (test tohumu 60→200). Kova/401/gossip-komut testleri eklendi |
| 3 | Güvenlik taraması temiz, SBOM+lisans | 🟡 | cargo-audit KURULDU+YEŞİL (9→0: rustls/reqwest/sqlx-sade yükseltme + gerekçeli audit.toml). LICENSE Apache-2.0 yayında. Eksik: cargo-deny + SBOM dosyası + B35/B36 yükseltmeleri |
| 4 | Performans hedefleri + yük testi | 🟡 | rampa ölçüldü (370/sn tavan); hedef bildirgesi YOK → yazılacak |
| 5 | Erişilebilirlik AA | 🟡 | hiyerarsi + odaksal duzeltildi (footer h2, stil korundu); kontrast/ekran-okuyucu denetimi YOK |
| 6 | Dokümantasyon tam+güncel | 🟡 | Tur-4'te 30+ drift düzeltildi (sayılar+WAF+kapanışlar); tarihsel sürümler banner'lı. API referansı + runbook YOK |
| 7 | CI/CD + rollback testi | 🟡 | CI workflow'lar eklendi (rust test + site i18n/HTML; ilk yesil push'u bekliyor). Rollback MEKANIGI ispatli. Canli downgrade YOK (dogru). |
| 8 | Backup/restore + DR | 🟡 | yedek günlük + restore provası GEÇTİ (16 Eyl); yedekler 600'e kilitlendi, çift-script temizlendi. DR planı YOK → yazılacak |
| 9 | Review + ADR | 🟡 | ADR dizini acildi (000-003); review yok (tek kisi) |
| 10 | Risk/borç/eksik listesi | 🟡 | §5 + §11 var; formal risk register YOK |
| 11 | Son paranoid tarama | 🟡 | süpürmeler yapıldı; 500-tur sayacı YOK → bu dosya sayar: **tur 4** (Tur-1 repo, Tur-2 DB, Tur-3 site/doc, Tur-4 kapsamlı: 5 ajan, 117 dosya, 30+ fix, 11 fantom) |
| 12 | 3+ iyileştirme | ✅ | OOM havuzu, kanarya filtresi, toplu-kanıt, kira, mesh-denetim (kanıtlı, canlıda) |
| 13 | Şifre sahibi onayı | ⏳ | bekleniyor |

## Tur sayacı

- Tur 1 (18 Eyl): secret taraması + lisans envanteri + bu dosya.
