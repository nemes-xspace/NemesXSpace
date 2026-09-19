#!/usr/bin/env python3
"""Denetim zinciri dogrulayici: kayitlar.jsonl hash-bagliligini kontrol eder.
Calistir: python3 dogrula.py  (blake3 paketi gerekir; yoksa sha256-satirlari atlar)
Cikis: SAGLAM + kayit sayisi, ya da ilk kirik halka.
"""
import json
import os
import sys

BURADA = os.path.dirname(os.path.abspath(__file__))
DOSYA = os.path.join(BURADA, 'kayitlar.jsonl')

try:
    from blake3 import blake3 as _b3

    def ozet(b: bytes) -> str:
        return _b3(b).hexdigest()

    ALGO = 'blake3'
except ImportError:
    import hashlib

    def ozet(b: bytes) -> str:
        return 'sha256:' + hashlib.sha256(b).hexdigest()

    ALGO = 'sha256-geri-donus'


def main() -> int:
    onceki = '0' * 64
    sayi = 0
    with open(DOSYA, encoding='utf-8') as f:
        for i, satir in enumerate(f, 1):
            r = json.loads(satir)
            if r.get('onceki') != onceki:
                print(f'KIRIK HALKA satir {i} ({r.get("no")}): onceki uyusmuyor')
                return 1
            beklenen = r.pop('hash')
            ham = json.dumps(r, ensure_ascii=False, sort_keys=True)
            if ozet(ham.encode()) != beklenen:
                print(f'KIRIK IMZA satir {i} ({r.get("no")}): hash uyusmuyor')
                return 1
            onceki = beklenen
            sayi += 1
    print(f'SAGLAM ({ALGO}): {sayi} kayit, zincir butun')
    return 0


if __name__ == '__main__':
    sys.exit(main())
