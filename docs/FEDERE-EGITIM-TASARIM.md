# Federe Eğitim — 3-Yol Eleme Protokolü (19 Eyl 2026)

> Karar: tek-yol bahsi YOK; geçemeyen elenir. Ölçüt: otonomi
> (insansız-uçtan-uca) + yakınsama + iletişim-maliyeti + Bizans-dayanım.

## Geçiş kapıları (hepsi ölçülü)

| Kapı | FedAvg-LoRA | DiLoCo-tarzı | Swarm |
|---|---|---|---|
| G0 tek-kart repro | KOŞUYOR (19 Eyl, 1.7B, ~6sn/adım, GPU %94) | aynı repro + 2-parça-sim | kurulum-fizibilite |
| G1 2-düğüm | adaptör-gönder→ortalama→dağıt→kayıp-geçidi | seyrek-senkron simülasyonu | DHT-keşif + 1-eş eğitim |
| G2 Bizans | zehirli-adaptör red + slash | aynı | aynı |
| G3 otonomi | insansız-tur (kayıt-kanıtlı) | aynı | aynı |

## Elenen/ertelenen

- Yok (henüz). DiLoCo/Swarm G0'a girmedi — sıra onlarda.

## Altyapı notları (ölçülü)

- Eğitim-yığını çalışıyor (torch 2.14+cu126, peft/TRL).
- Veri: 189K talimat-çifti (`model-v01/train-v01.jsonl`).
- Üretim-çatışması: eğitim sırasında gömme-gecikmesi artar
  (bekçi+retry kapsar; uzun-eğitimler gece-penceresine).
