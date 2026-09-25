# S4 Dış Miner Provası — Ara Rapor (25 Eyl 2026)

> Durum: KISMEN YEŞİL. Kriterlerin 3/5'i kapanıyor; kalıcı otonomi ve 2. düğüm eksik.

## Kurulum

- Düğüm: `miner-fa8edb50` (WIN-TEST-DRILL, win10-test VM, libvirt NAT)
- Bağlantı: `https://komuta.nemes-x.space` (herkese açık hat, CF arkası)
- Gömme: `192.168.122.1:1247` (köprü ağı, izole embed sunucusu)
- Kimlik: TOFU pubkey, token auth, 0 strike

## Ölçümler (25 Eyl)

| Kriter | Hedef | Ölçülen | Durum |
|---|---|---|---|
| Bağımsız yazma | Komuta API ile kanıt | 2.638.340 kanıt, 271,7 coin | ✅ |
| Denetim katılımı | Mesh/HTTP denetim | 24saatte 41.980 başarı, 108k doğrulanmış | ✅ |
| İtibar disiplini | strike=0, itibar=100 | 0 / 100 | ✅ |
| Kesintisiz otonomi | Reboot sonrası kendiliğinden | 22 Eyl: el ile başlatıldı (W_N_STAR v1 bekliyor) | ❌ |
| 2. bağımsız düğüm | Farklı ağ/operatör | 0 (tek dış düğüm) | ❌ |

## Kapanış koşulu

W_N_STAR v1 kurulumu + 7 gün kesintisiz nabız + 2. düğüm (farklı operatör). Tarih hedefi: 31 Eki 2026 (test kapanışı).
