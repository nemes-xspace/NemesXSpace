# NEMES Mainnet Yol Haritası — v1.0 (22 Eyl 2026)

> Kaynak: `MAINNET-GECIS.md` kontrol listesinin zamanlı, sahipli hali.
> İlke: tek-yön kapı — kutusuz açılış YOK. Tarihler iş-günü değil, bağımlılık sırasıdır.

## Faz M0 — Kilitler (1 hafta, teknik)

| # | İş | Sahip | Durum |
|---|----|-------|-------|
| M0.1 | `BATCH_ODUL_TABAN_MIKRO` 2000→800, `HALVING_BATCH` 5M→50B, `STRICT_DENETIM`→1 (tek commit) | Baş mimar | Açık |
| M0.2 | Tavan-kota kuralı koda göm (tavana 1M kala kısılma, TOKENOMI §2b) | Baş mimar | Açık (tasarım var) |
| M0.3 | `cargo-audit` + `cargo-deny` temiz, SBOM dosyası | Güvenlik | Açık |

## Faz M1 — Ağ bağımsızlığı (3-4 hafta, SRE)

| # | İş | Sahip | Durum |
|---|----|-------|-------|
| M1.1 | 3+ bağımsız operatör canlı (şu an 0 — en uzun kutup) | SRE | Açık |
| M1.2 | Dış miner provası S4 yeşil | Test | Açık |
| M1.3 | Tohum-0 dışı düğüm mesh'te + `nemes/komut` gossip provası | SRE | Açık (kod testte yeşil) |

## Faz M2 — Ekonomi (2-3 hafta, ML/ekonomi)

| # | İş | Sahip | Durum |
|---|----|-------|-------|
| M2.1 | B27 tier kararı (gölge-skor) | ML lideri | Açık |
| M2.2 | Hazine + Enterprise %50 payout mekaniği | Baş mimar | Açık |
| M2.3 | 1 hafta proofsuz izleme (P0-6) | SRE | Açık |

## Faz M3 — Hukuk + site (2-4 hafta, paralel)

| # | İş | Sahip | Durum |
|---|----|-------|-------|
| M3.1 | S7 avukat onayı (token niteliği, KVKK, vergi notu) | Hukuk | Açık |
| M3.2 | Terms/privacy mainnet cümleleri | Hukuk | Açık |
| M3.3 | Master anahtar töreni (kartta, B24) | Proje sahibi | Açık |

## Faz M4 — Lansman (1 hafta)

- Anlık görüntü (testnet bakiyeleri mainnet'e taşınMAZ — sıfırdan başlanır; karar metni yayınlanır).
- Tohum düğümler + izleme + kill-switch provası.
- Duyuru + borsa/likidite teması (ayrı iş kolu, 1-3 ay).

## Kritik yol

M1.1 (bağımsız operatörler) → M2.3 → M3.1 → M4. Gerçekçi toplam: **3-5 ay**, en büyük risk operatör bulma ve hukuk. Testnet bu sürede çalışmaya ve rapor satmaya devam eder.
