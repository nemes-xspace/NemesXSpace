#!/usr/bin/env python3
# Mitoloji graf çıkarıcı v0 (Faz C graf-1): kural tabanlı, deterministik.
# Girdi: /tmp/mitoloji-paket/*.json | Çıktı: /tmp/mitoloji.db
# Tablo: varlik(ad, tur, frekans), iliski(kaynak, hedef, tip, kanit), kaynak(dosya, url, hash)
import json, glob, re, sqlite3, os

PAKET = "/tmp/mitoloji-paket"
DB = "/tmp/mitoloji.db"

# Bilinen çekirdek adlar (tohum liste — çıkarıcı bunları kaçırmaz)
TOHUM = ["Tengri", "Zeus", "Odin", "Gılgamış", "Ergenekon", "Thor", "Hera",
         "Osiris", "İsıs", "Horus", "Odin", "Frigg", "Enki", "Enlil",
         "Marduk", "İştar", "Umay", "Erlik", "Kayra", "Ülgen", "Aten",
         "Ra", "Anubis", "Thor", "Loki", "Freyja", "Bozkurt", "Demirdağ"]

# İlişki kalıpları: X, Y'nin <rol> | X ile Y | X ve Y
ROL = r"(oğlu|kızı|eşi|kardeşi|babası|annesi|düşmanı|karşılığı|denk|şeklidir|olarak bilinir)"
P1 = re.compile(r"([A-ZÇĞİÖŞÜ][a-zçğıöşü]+(?:\s+[A-ZÇĞİÖŞÜ][a-zçğıöşü]+)?),?\s+([A-ZÇĞİÖŞÜ][a-zçğıöşü]+(?:\s+[A-ZÇĞİÖŞÜ][a-zçğıöşü]+)?)['’](?:nin|nın|nun|nün|in|ın|un|ün)\s+" + ROL)
P2 = re.compile(r"([A-ZÇĞİÖŞÜ][a-zçğıöşü]+)\s+ile\s+([A-ZÇĞİÖŞÜ][a-zçğıöşü]+)")
P3 = re.compile(r"\b([A-ZÇĞİÖŞÜ][a-zçğıöşü]{2,}(?:\s+[A-ZÇĞİÖŞÜ][a-zçğıöşü]{2,})?)\b")

STOP = set("""Ve Bir Bu İçin Gibi Daha Çok Tüm Her Şu Olarak İle Veya Ya Da De Ki Ne Nasıl Neden Hangi Hangi
Türk Yunan Mısır Vikipedi İçeriğe İçindekiler Görünüm Kişisel Ara Anasayfa Katılım Dolaşım Yardım
Mitolojisi Mitoloji Tarih Tanrı Tanrısı Tanrılar Kültür Dönem Yüzyıl Yıl Krallık İmparatorluk
Wayback Machine Ara Hesap Oturum Değiştir Kaynağı Geçmişi Press Diğer Erişim Orijinal Nisan
Giriş Bağlantıları Madde Tartışma Türkçe Oku Bekleyen Değişiklikler Son Özel Rastgele Seçkin Yakınımdakiler
Deneme Köy Dosya Topluluk Dükkân Bağış Oluştur Sayfa Ana Geçmiş Görüntüle Kaynak Geçmişi Değiştir
Erişim Tarihi Arşivlendi Orijinalinden Alındı Dil Konu Ekle Kenar Gizle Taşı Site Araçlar Eylemler
Katkılar Hesap Oturum Bağış Yapın Hesap Oluştur Katılım Deneme Görünüm Bağış Madde Tartışma Türkçe
Press Erişim Diğer Orijinal Nisan Sayfa Ara""".split())
PHRASE_STOP = {"Wayback Machine", "Ara Ara", "Değiştir Kaynağı", "Katılım Deneme",
    "Görünüm Bağış", "Madde Tartışma", "Türkçe Oku", "Erişim Tarihi", "Oturum Aç",
    "Hesap Oluştur", "Bağış Yapın", "Ana Menü", "Son Değişiklikler", "Rastgele Madde",
    "Seçkin İçerik", "Topluluk Portali", "Dosya Yükle", "Köy Çeşmesi", "Deneme Tahtası"}

if os.path.exists(DB):
    os.remove(DB)
con = sqlite3.connect(DB)
con.executescript("""
CREATE TABLE varlik (ad TEXT PRIMARY KEY, tur TEXT DEFAULT 'bilinmiyor', frekans INTEGER DEFAULT 0);
CREATE TABLE iliski (kaynak TEXT, hedef TEXT, tip TEXT, kanit TEXT, PRIMARY KEY (kaynak, hedef, tip));
CREATE TABLE kaynak (dosya TEXT PRIMARY KEY, url TEXT, hash TEXT, karakter INTEGER);
""")

for f in sorted(glob.glob(f"{PAKET}/*.json")):
    d = json.load(open(f, encoding="utf-8"))
    url, metin = d["url"], d["metin"]
    h = d["provenance"]["icerik_hash"]
    con.execute("INSERT OR REPLACE INTO kaynak VALUES (?,?,?,?)",
                (os.path.basename(f), url, h, d["karakter"]))
    frek = {}
    for m in P3.finditer(metin):
        ad = m.group(1).strip()
        if ad in STOP or ad in PHRASE_STOP or len(ad) < 3:
            continue
        frek[ad] = frek.get(ad, 0) + 1
    for t in TOHUM:
        if t in metin:
            frek[t] = frek.get(t, 0) + metin.count(t)
    for ad, s in frek.items():
        tur = "tanrı" if ad in TOHUM else "kavram"
        con.execute("""INSERT INTO varlik (ad, tur, frekans) VALUES (?,?,?)
                       ON CONFLICT(ad) DO UPDATE SET frekans=frekans+excluded.frekans""",
                    (ad, tur, s))
    for m in P1.finditer(metin):
        k, hdf, rol = m.group(1).strip(), m.group(2).strip(), m.group(3).strip()
        if k in STOP or hdf in STOP or k in PHRASE_STOP or hdf in PHRASE_STOP:
            continue
        kanit = m.group(0)[:200]
        con.execute("INSERT OR IGNORE INTO iliski VALUES (?,?,?,?)", (k, hdf, rol, kanit))
    for m in P2.finditer(metin[:20000]):
        k, hdf = m.group(1).strip(), m.group(2).strip()
        if k in STOP or hdf in STOP or k in PHRASE_STOP or hdf in PHRASE_STOP:
            continue
        con.execute("INSERT OR IGNORE INTO iliski VALUES (?,?,?,?)", (k, hdf, "ile-bagli", m.group(0)[:200]))

con.commit()
print("varlik:", con.execute("SELECT count(*) FROM varlik").fetchone()[0])
print("iliski:", con.execute("SELECT count(*) FROM iliski").fetchone()[0])
print("kaynak:", con.execute("SELECT count(*) FROM kaynak").fetchone()[0])
print("---EN SIK 15---")
for r in con.execute("SELECT ad, tur, frekans FROM varlik ORDER BY frekans DESC LIMIT 15"):
    print(r)
print("---ILISKI ORNEK 15---")
for r in con.execute("SELECT kaynak, hedef, tip FROM iliski LIMIT 15"):
    print(r)
con.close()
print("DB:", DB)
