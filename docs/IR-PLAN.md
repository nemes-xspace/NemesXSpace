# Olay Müdahale Planı (IR) — v1 (19 Eyl 2026)

> Ekip: 1 kişi (operatör = sahip = denetçi). Roller çakışır; prosedür
> bunu varsayar (çift-onay yerine checklist + defter-kaydı).

## 1. Sınıflandırma + ilk 15 dakika

| Seviye | Örnek | İlk eylem |
|---|---|---|
| S1 | Üretim durdu, DB şüpheli, token-dizini sızdı | İzole et (servis durdur) + deftere IR aç |
| S2 | Tek miner down, alarm tekil, site statik-bozuk | İzle + 1 saat içinde müdahale |
| S3 | Uyarı, drift,期限 yaklaşan | Haftalık bakıma yaz |

## 2. Müdahale akışı (S1)

1. TESPİT: izleme.py alarmı / kullanıcı bildirimi / journal.
2. İZOLE: `systemctl stop <birim>` (yayılmayı kes; DB'ye yazma devam ediyorsa komuta önce).
3. KORU: `yedek/` + `bin/.bak` + journal dışa aktar (`journalctl -u X --since ... > /tmp/opencode/ir-$$.log`).
4. ADLİ KOPYA: şüpheli dosyayı hash'le (`sha256sum`), kopyala, zincir-kaydı (kim/ne-zaman).
5. TEMİZLE: nedene göre (runbook rollback / token-iptal (kara_liste) / kova-sıkılaştırma).
6. KURTAR: RTO 30dk hedefiyle başlat + `/health` + üretim-metriği.
7. RAPOR: 24 saat içinde IR kaydı (defter) + ders + eylem.

## 3. İletişim

- İç: yok (tek kişi) — kararlar deftere.
- Madenciler: kanal YOK (eksik — MAINNET öncesi açılacak; şimdilik site duyurusu).
- Yasal: S7 avukat tanımlanınca eşikler yazılacak (şu an tanımsız).

## 4. Kanarya tetiklenmesi

`KANARYA OYANDI` alarmı = S1 (token sızıntısı şüphesi): kanarya satırı
İNCELE (hangi tablo değişti), ilgili tokenı kara-listeye al, kapsamı
araştır (aynı IP'den başka istek var mı — http-red logları).

## 5. Tatbikat

Yılda 2 masa-başı + 1 saha (Felaket-provası ile birleşik). Sonraki:
Aralık 2026 (kanarya-tetikleme simülasyonu — kopya-DB'de).
