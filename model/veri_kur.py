#!/usr/bin/env python3
# Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
"""Community-v0.1 veri kurulumu: wiki -> filtre -> sablon -> train/eval.
Kullanim: veri_kur.py [--cikti /srv/beyin/model-v01] [--limit N]
- Filtre: length(ozet) >= 100, denetimde kalmis madde'ler haric.
- 6 sablon, deterministik bolme: id % 500 == 0 -> eval havuzu.
- Cevaplar 1200 karakterde kirpilir (1B baglam dostu).
"""
import argparse
import json
import os
import sqlite3
import sys

WIKI = "file:/srv/beyin/wiki/wiki_tr.db?mode=ro"
KOMUTA_DB = "/home/d3str0y1ng/nemes-testnet/komuta.db"
MIN_OZET = 100
MAX_CEVAP = 1200
EVAL_MOD = 500  # id % 500 == 0 -> eval

SABLONLAR = [
    lambda b, k, o: ("%s hakkında bilgi ver." % b, o),
    lambda b, k, o: ("%s nedir? Kısaca özetle." % b, o),
    lambda b, k, o: ("%s konusunu açıkla." % b, o),
    lambda b, k, o: ("Bana %s konusundan bahset." % b, o),
    lambda b, k, o: ("%s ile ilgili temel bilgiler nelerdir?" % b, o),
    lambda b, k, o: ("%s hangi alanda yer alır? Açıkla." % b,
                     ("%s, %s alanına girer. %s" % (b, k, o) if k else o)),
]


def kirp(s, n=MAX_CEVAP):
    s = (s or "").strip()
    return s if len(s) <= n else s[:n].rsplit(" ", 1)[0]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cikti", default="/srv/beyin/model-v01")
    ap.add_argument("--limit", type=int, default=0)
    a = ap.parse_args()
    os.makedirs(a.cikti, exist_ok=True)

    # Denetimde kalmis (supheli/hatali) madde id'leri egitim disi.
    bozuk = set()
    try:
        k = sqlite3.connect("file:%s?mode=ro" % KOMUTA_DB, uri=True)
        bozuk = {r[0] for r in k.execute(
            "SELECT DISTINCT madde_id FROM kanitlar WHERE dogrulama IS NOT NULL AND dogrulama < 0.98")}
        k.close()
    except Exception as e:
        print("uyari: komuta DB okunamadi (%s), eleme yok" % e, file=sys.stderr)
    print("elenen supheli madde: %d" % len(bozuk), flush=True)

    w = sqlite3.connect(WIKI, uri=True)
    cur = w.execute(
        "SELECT id, baslik, ozet, kategori FROM madde WHERE length(ozet) >= ? ORDER BY id",
        (MIN_OZET,))
    ft = open(os.path.join(a.cikti, "train.jsonl"), "w")
    fe = open(os.path.join(a.cikti, "eval.jsonl"), "w")
    n_madde = n_train = n_eval = 0
    while True:
        grup = cur.fetchmany(2000)
        if not grup:
            break
        for mid, baslik, ozet, kat in grup:
            if mid in bozuk:
                continue
            if a.limit and n_madde >= a.limit:
                break
            b = (baslik or "").strip()
            o = kirp(ozet)
            if not b or len(o) < MIN_OZET:
                continue
            k = (kat or "").strip()
            hedef = fe if (mid % EVAL_MOD == 0) else ft
            for i, fn in enumerate(SABLONLAR):
                t, c = fn(b, k, o)
                hedef.write(json.dumps({"talimat": t, "cevap": c, "kaynak": mid,
                                        "sablon": i}, ensure_ascii=False) + "\n")
            n_madde += 1
            if mid % EVAL_MOD == 0:
                n_eval += 1
            else:
                n_train += 1
        if a.limit and n_madde >= a.limit:
            break
        if n_madde % 50000 == 0:
            print("... %d madde" % n_madde, flush=True)
    ft.close()
    fe.close()
    w.close()
    rapor = {"madde": n_madde, "train_cift": n_train * len(SABLONLAR),
             "eval_cift": n_eval * len(SABLONLAR), "elenen": len(bozuk)}
    open(os.path.join(a.cikti, "veri_rapor.json"), "w").write(
        json.dumps(rapor, indent=2, ensure_ascii=False))
    print(json.dumps(rapor, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
