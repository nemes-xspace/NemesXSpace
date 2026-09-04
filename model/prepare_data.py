#!/usr/bin/env python3
# Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
"""
prepare_data.py — BEYİN'den damıtma verisi hazırlar
Girdi: /srv/beyin/wiki (40M vektör) + Sovereign sentetik 5M
Çıktı: ~/NemesXSpace/model/data/nano_train.jsonl (HF datasets format)
Durum: Sovereign gut_en checkpoint bekleniyor — şimdilik iskelet, BEYİN'den örnek çekme hazır
"""
import os, json, sys
from pathlib import Path

OUT = Path.home() / "NemesXSpace/model/data"
OUT.mkdir(parents=True, exist_ok=True)

def dummy():
    # Sovereign hazır olana kadar 100 örnek sentetik veri (test)
    out = OUT / "nano_train.jsonl"
    with open(out, "w") as f:
        for i in range(100):
            f.write(json.dumps({
                "prompt": f"Örnek soru {i}: NEMES-X nedir?",
                "completion": f"Örnek cevap {i}: NEMES-X, bilgiyi madencileyen egemen bir ağdır.",
                "source": "dummy"
            }, ensure_ascii=False) + "\n")
    print(f"dummy veri yazildi: {out} (100 satir)")

if __name__ == "__main__":
    print(f"BEYIN kaynagi: /srv/beyin/wiki (40M vektor)")
    print(f"Cikti: {OUT}/nano_train.jsonl")
    print("Sovereign checkpoint bekleniyor — simdilik dummy veri olusturuluyor...")
    dummy()
    print("Hazir. Gercek veri icin: BEYIN wiki_cc100 + gut_en DB'lerinden cekilecek.")
