#!/usr/bin/env python3
# B27 gölge tier hesaplayıcı (salt-görüntü, ödemeye dokunmaz).
# Formül: ham = V30n*0.5 + STAKEn*0.3 + ITIBARn*0.1 + SUREn*0.1 ; tier = 1+floor(ham*4)
import sqlite3, time
DB = "/home/d3str0y1ng/nemes-testnet/komuta.db"
PENCERE = 30 * 86400
simdi = int(time.time())
con = sqlite3.connect(DB)
con.execute("""CREATE TABLE IF NOT EXISTS miner_tier (miner_id TEXT PRIMARY KEY, v30 INTEGER DEFAULT 0,
stake_mikro INTEGER DEFAULT 0, itibar INTEGER DEFAULT 100, sure_gun REAL DEFAULT 0, ham REAL DEFAULT 0, tier INTEGER DEFAULT 1, ts INTEGER)""")
minerlar = list(con.execute("SELECT miner_id, itibar, created_at, last_seen FROM miners WHERE miner_id!='miner-kanarya-00'"))
veriler = []
for mid, itibar, olustu, goruldu in minerlar:
    v30 = con.execute("SELECT count(*) FROM kanitlar WHERE miner_id=? AND dogrulama IS NOT NULL AND ts>=?",
                      (mid, simdi - PENCERE)).fetchone()[0]
    stake = con.execute("SELECT coalesce(sum(stake_mikro),0) FROM teklifler WHERE miner_id=? AND durum='acik'",
                        (mid,)).fetchone()[0] or 0
    sure = max(0.0, (min(goruldu, simdi) - olustu) / 86400.0)
    veriler.append([mid, v30, stake, itibar or 100, sure])
def norm(vals):
    med = sorted(vals)[len(vals) // 2] if vals else 1
    med = med or 1
    return [min(1.0, v / med) for v in vals]
V = norm([v[1] for v in veriler]); S = norm([v[2] for v in veriler])
I = [(v[3] / 100.0) for v in veriler]; D = norm([v[4] for v in veriler])
for i, v in enumerate(veriler):
    ham = V[i] * 0.5 + S[i] * 0.3 + I[i] * 0.1 + D[i] * 0.1
    tier = 1 + min(4, int(ham * 4))
    con.execute("INSERT OR REPLACE INTO miner_tier VALUES (?,?,?,?,?,?,?,?)",
                (v[0], v[1], v[2], v[3], round(v[4], 1), round(ham, 3), tier, simdi))
    print(f"{v[0]}: V30={v[1]} stake={v[2]} itibar={v[3]} sure={v[4]:.0f}g ham={ham:.3f} -> T{tier}")
con.commit()
print("yazıldı:", con.execute("SELECT count(*) FROM miner_tier").fetchone()[0])
con.close()
