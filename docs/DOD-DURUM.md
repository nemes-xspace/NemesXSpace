# Definition of Done — DURUM (18 Eyl 2026, kanıtlı)

> Kaynak: `docs/MUTLAK-OTORITE.md` §7. Her madde: DURUM + KANIT + EKSİK.
> Lejant: ✅ kanıtlı | 🟡 kısmi | 🔴 eksik.

| # | Madde | Durum | Kanıt / Eksik |
|---|---|---|---|
| 1 | Acceptance criteria | 🟡 | §5 listesi var; formal AC dokümanı YOK → yazılacak |
| 2 | Testler geçti, kritik yol kapsamlı, flaky yok | 🟡 | 20+12+10+2 yeşil; 1 flake gözlendi (denetim turu, 1/9, kökü açık) |
| 3 | Güvenlik taraması temiz, SBOM+lisans | 🟡 | secret: 4 repo TEMİZ (Tur-1). Lisans: 533 dep, GPL/AGPL yok; 6 crate tescilli hizalandı; ring-0.16 belirsiz (transitive, düşük risk). Eksik: cargo-audit/cargo-deny + SBOM dosyası |
| 4 | Performans hedefleri + yük testi | 🟡 | rampa ölçüldü (370/sn tavan); hedef bildirgesi YOK → yazılacak |
| 5 | Erişilebilirlik AA | 🟡 | hiyerarsi + odaksal duzeltildi (footer h2, stil korundu); kontrast/ekran-okuyucu denetimi YOK |
| 6 | Dokümantasyon tam+güncel | 🟡 | 20+ doküman; API referansı + runbook YOK |
| 7 | CI/CD + rollback testi | 🟡 | CI yok (yerel cargo); rollback MEKANIGI ispatli (eski binary acilmaz → yedek+eslesen binary; yedek saglam 3.75M). Canli downgrade provasi YOK (veri kaybi riski, dogru kararla atlandi) |
| 8 | Backup/restore + DR | 🟡 | yedek günlük + restore provası GEÇTİ (16 Eyl); DR planı YOK → yazılacak |
| 9 | Review + ADR | 🟡 | ADR dizini acildi (000-003); review yok (tek kisi) |
| 10 | Risk/borç/eksik listesi | 🟡 | §5 + §11 var; formal risk register YOK |
| 11 | Son paranoid tarama | 🟡 | süpürmeler yapıldı; 500-tur sayacı YOK → bu dosya sayar: **tur 1** |
| 12 | 3+ iyileştirme | ✅ | OOM havuzu, kanarya filtresi, toplu-kanıt, kira, mesh-denetim (kanıtlı, canlıda) |
| 13 | Şifre sahibi onayı | ⏳ | bekleniyor |

## Tur sayacı

- Tur 1 (18 Eyl): secret taraması + lisans envanteri + bu dosya.
