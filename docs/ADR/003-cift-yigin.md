# ADR-003: Çift-Yığın Dağıtım (Eski Yol Yaşar)

- Tarih: 17 Eyl 2026 (B14/B15/B20 örüntüsü). Statü: kabul.
- Bağlam: win `.exe` ve eski miner'lar sahada; protokol değişimi onları
  öldürmemeli (S8, dış makine güveni).
- Karar: her yeni yol EK uç/flag olarak gelir (`/api/kanit/toplu`,
  `--kira`, mesh-denetim); eski yol aynen çalışır, düşme yedeğidir.
  Kaldırma ancak 2 epoch eski + duyurulu olur (henüz uygulanmadı).
- Alternatifler: (a) sert geçiş — reddedildi (saha riski); (b) sürüm
  pazarlığı — ertelendi (B23b yetenek bayrağı temel olacak).
- Sonuçlar: B14/B15/B20 sıfır kesintiyle canlıya girdi.
- Geri-dönüş: flag kapatma = eski davranış (drop-in sil + restart).
