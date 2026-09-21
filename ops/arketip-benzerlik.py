#!/usr/bin/env python3
# Arketip benzerlik v0 (Faz C-3): tanrı bağlam cümlelerini 1241'e embedlet,
# ortalama vektörler arası kosinüsle aynı-arketip öner. Deterministik, eşikli.
# Girdi: /tmp/mitoloji-paket/*.json + /tmp/mitoloji.db | Çıktı: iliski(tip='arketip-benzer')
import json, glob, sqlite3, urllib.request, math

API = "http://127.0.0.1:1241/v1/embeddings"
MODEL = "text-embedding-nomic-embed-text-v1.5"
ESIK = 0.75  # nomic-tr bağlamlarında gözlenen üst dilim (v0, raporda dağılımla birlikte)
TANRILAR = ["Tengri", "Zeus", "Odin", "Ra", "Enlil", "Ergenekon", "Gılgamış",
            "Loki", "Thor", "Ülgen", "Erlik", "Enki", "Hera", "Horus", "Osiris", "Umay", "Marduk"]

def embed_batch(metinler):
    data = json.dumps({"model": MODEL, "input": metinler}).encode()
    req = urllib.request.Request(API, data=data, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=120) as r:
        j = json.loads(r.read())
    out = [None] * len(j["data"])
    for d in j["data"]:
        out[d["index"]] = d["embedding"]
    return out

def cos(a, b):
    dot = sum(x * y for x, y in zip(a, b))
    na = math.sqrt(sum(x * x for x in a))
    nb = math.sqrt(sum(x * x for x in b))
    return dot / (na * nb) if na and nb else 0.0

# her tanrı için bağlam cümleleri topla (adı geçen cümleler, en fazla 6)
baglam = {}
for f in sorted(glob.glob("/tmp/mitoloji-paket/*.json")):
    metin = json.load(open(f, encoding="utf-8"))["metin"]
    cumleler = [c.strip() for c in metin.replace("!", ".").replace("?", ".").split(".")]
    for t in TANRILAR:
        for c in cumleler:
            if t in c and 40 < len(c) < 600 and len(baglam.get(t, [])) < 6:
                baglam.setdefault(t, []).append("search_document: " + c)

print("bağlam:", {k: len(v) for k, v in baglam.items()})
aktif = {k: v for k, v in baglam.items() if len(v) >= 2}
print(f"aktif tanrı: {len(aktif)}/{len(TANRILAR)} (en az 2 cümle)")

# embed (toplu)
tum, sahipler = [], []
for t, lst in aktif.items():
    for c in lst:
        tum.append(c)
        sahipler.append(t)
veks = embed_batch(tum)
ort = {}
for t in aktif:
    idx = [i for i, s in enumerate(sahipler) if s == t]
    n = len(veks[0])
    m = [sum(veks[i][j] for i in idx) / len(idx) for j in range(n)]
    ort[t] = m

# eşleşmeler
eslesme = []
adlar = sorted(ort)
for i in range(len(adlar)):
    for j in range(i + 1, len(adlar)):
        c = cos(ort[adlar[i]], ort[adlar[j]])
        eslesme.append((c, adlar[i], adlar[j]))
eslesme.sort(reverse=True)
print("---EN YÜKSEK 15---")
for c, a, b in eslesme[:15]:
    print(f"{c:.4f} {a} - {b}")

con = sqlite3.connect("/tmp/mitoloji.db")
yazilan = 0
for c, a, b in eslesme:
    if c >= ESIK:
        kanit = f"bağlam-ort cos={c:.4f} (nomic-embed-tr, {len(aktif[a])}+{len(aktif[b])} cümle)"
        con.execute("INSERT OR IGNORE INTO iliski VALUES (?,?,?,?)", (a, b, "arketip-benzer", kanit))
        yazilan += 1
con.commit()
print(f"ilişki yazıldı: {yazilan} (eşik {ESIK})")
print("toplam ilişki:", con.execute("SELECT count(*) FROM iliski").fetchone()[0])
con.close()
