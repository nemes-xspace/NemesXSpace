# Deploy Prosedürü — v1 (19 Eyl 2026, Prompt 12)

> Pencere: systemd sağlıklı + üretim-düşük an. Pencere-dışı: P0 + kayıt.
> Kaynak-gerçek: `ops/systemd/` (birimler + drop-in'ler).

## Adımlar (sıralı, her adımda doğrula)

1. Derle: `cargo build --release -p komuta-rs -p nemes-miner` (uyarı-sayımı not et).
2. Test: `cargo test -p komuta-rs` (27/27) + `cargo audit` (exit 0).
3. Yedekle: `.bak<N>-<tarih>` (mevcut binary'ler) — rollback hazır.
4. Durdur (sıralı): miner-a → miner-b → komuta. Her birinde `is-active`
   + `ps` ile doğrula (D-Bus takılırsa 1 kez tekrarla, olmazsa BIRAK — IR aç).
5. Değiştir: yeni binary'leri `bin/` altına kopyala.
6. Başlat (ters-sıra): komuta → `/health` (60-90sn havuz-yükleme) →
   miner-a/b → journal üretim-satırı.
7. Doğrula (10 dk): `/health`, `/api/arz`, kanıt-akışı (DB sayımı),
   `http-red` anomalisi, RSS.
8. Kaydet: ALTYAPI-KAYIT §4 + defter CR + bu dosyaya tarih.

## Geri-dönüş (tek-komut dizisi)

`systemctl stop X; cp bin/<ad>.bak<N> bin/<ad>; systemctl start X;
curl /health` — RTO ~2dk (ölçülü). DB-şeması değiştiyse runbook'taki
DB+binary eşleşme kuralı geçerli.

## Yasaklar

- Pencere-dışı deploy (P0 hariç).
- `.bak` almadan swap.
- Doğrulamasız "oldu" (her adım çıktılı).
