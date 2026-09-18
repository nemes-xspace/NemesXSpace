# Piyasaya Çıkış Denetimi — 16 Eyl 2026 (canlı ölçümlü)

> Soru: bugün dışarıdan biri gelse madencilik yapabilir mi? **Cevap: hayır.**
> Aşağıdaki liste kapanmadan lansman yapılmaz.

## P0 — Lansman engelleri (biri bile açıkken çıkış yok)

| # | Engel | Kanıt | Kapanış |
|---|---|---|---|
| P0-1 | İndirilen miner çalışmıyor | `miner/nemes-x-miner-v0.1.0.zip` = 1.1K, içi README + "Early Access" echo scripti; madencilik kodu YOK | Gerçek Linux binary + kurulum provası (0.5) |
| P0-2 | Dağıtılacak görev yok | ~~Corpus tr tükenik~~ ÇÖZÜLDÜ (18 Eyl): newscrawl_tr üretimde, 4.3M kanıt | Kapandı |
| P0-3 | Dış API bot duvarı | CF 1010 çıplak istemcileri kesiyor (16 Eyl kanıtlı) | Miner UA ✅ devrede + CF WAF-skip ✅ CANLI (16 Eyl) |
| P0-4 | Windows istemcisi yok | Ev kullanıcılarının çoğu Windows; `.exe` yok, imza yok (1.3) | Natif .exe + EV sertifika |
| P0-5 | Ağ tek makinede | 3 miner kurucu hostta; bağımsız operatör 0; P2P mesh DONE (B12), Tohum-0 CANLI | 1 bağımsız düğüm (0.4) |
| P0-6 | Mainnet sabitleri uygulanmadı | Canlıda testnet: 2000 mikro + 5M halving | 1.1 commit + 1 hafta proofsuz izleme |

## P1 — Lansman öncesi şart (güvensiz çıkış olur)

| # | İş | Durum |
|---|---|---|
| P1-1 | Avukat görüşmesi (token sınıflandırma) | Yapılmadı (0.9) |
| P1-2 | Genesis + multi-sig + kilit adresleri | Tasarım var, tören yok |
| P1-3 | Wallet v1 (gönder/al + soğuk imza) | Yok (1.12) |
| P1-4 | Ödeme rayı (USDT-TRC20 manuel akış + TX kaydı) | Yok; H=$0 |
| P1-5 | Kurucu anahtarı K1 prosedürü | Taslak var, prova yok |
| P1-6 | Bağımsız denetim + bug bounty | Yok (2.10/1.10) |
| P1-7 | Site senkronu (sayılar + Y-ürün dili) | Y1-Y5 listesi hazır, uygulanmadı |

## P2 — Büyüme fazı (çıkış sonrası)
DEX/CEX, reputation v1, anti-fraud, governance forum, telemetri panosu, docs sitesi,
topluluk kanalları, mobil izleme, 2./3. corpus dalgaları, damıtma (kilitli-erte).

## Önerilen çıkış sırası (3 dalga)
- **Dalga A (kapalı beta, 2-3 hafta):** P0-2 (caselaw) + P0-3 (WAF) + gerçek Linux
  binary + davetli 3-5 miner → ilk dış pay dağıtılır → 0.3 DONE.
- **Dalga B (açık testnet, 1-2 ay):** P0-4 + P1-1/2/3/4 + site + topluluk →
  100+ madenci, H>$0.
- **Dalga C (mainnet):** 1.1 + genesis + 1.10/2.10 + P1-5/6 → Faz 1.

## Not
Teknik hat (komuta/miner/P2P/FAISS) bugün Dalga A'yı taşıyacak durumda;
eksikler ürünleştirme + hukuk + dağıtım alanlarında. Kod değil, cephe genişliği.
