#!/bin/bash
# Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
LOG="/home/d3str0y1ng/NemesXSpace/izle.log"
echo "=== NEMES-X IZLEYICI v2 BASLADI $(date '+%Y-%m-%d %H:%M:%S') ===" | tee -a "$LOG"
while true; do
    TS=$(date '+%m-%d %H:%M:%S')
    # hizli sayim - tek python cagrisi
    STATS=$(timeout 8 python3 << 'PY' 2>&1
import sqlite3, glob
def count_emb(pattern):
    total=0
    for p in glob.glob(pattern):
        try:
            c=sqlite3.connect(f'file:{p}?mode=ro', uri=True, timeout=1)
            total+=c.execute('SELECT COUNT(*) FROM vektor').fetchone()[0]
            c.close()
        except: pass
    return total
nc=count_emb('/srv/beyin/wiki/wiki_newscrawl_tr_emb_*.db')
try:
    tot=sqlite3.connect('file:/srv/beyin/wiki/wiki_newscrawl_tr.db?mode=ro', uri=True, timeout=1).execute('SELECT COUNT(*) FROM madde').fetchone()[0]
except: tot=0
try:
    gut=count_emb('/srv/beyin/wiki/wiki_gut_en_emb_*.db')
    gtot=sqlite3.connect('file:/srv/beyin/wiki/wiki_gut_en.db?mode=ro', uri=True, timeout=1).execute('SELECT COUNT(*) FROM madde').fetchone()[0]
except:
    gut=0; gtot=0
print(f"{nc} {tot} {gut} {gtot}")
PY
)
    echo "STATS raw: $STATS" >> "$LOG"
    NC_COUNT=$(echo $STATS | awk '{print $1}')
    NC_TOTAL=$(echo $STATS | awk '{print $2}')
    GUT_COUNT=$(echo $STATS | awk '{print $3}')
    GUT_TOTAL=$(echo $STATS | awk '{print $4}')
    if [ -n "$NC_COUNT" ] && [ "$NC_TOTAL" -gt 0 ] 2>/dev/null; then
        NC_PCT=$(python3 -c "print(f'{( $NC_COUNT / $NC_TOTAL * 100):.2f}')" 2>/dev/null)
        KALAN=$((NC_TOTAL - NC_COUNT))
    else
        NC_PCT="?" ; KALAN="?"
    fi
    GUT_PCT=$(python3 -c "print(f'{( $GUT_COUNT / $GUT_TOTAL * 100):.2f}')" 2>/dev/null || echo "?")
    DISK=$(df -h /srv/beyin 2>/dev/null | awk 'NR==2{print $4" boş ("$5")"}')
    TCTL=$(sensors 2>/dev/null | grep Tctl | awk '{print $2}')
    LINE="[$TS] NC:${NC_COUNT}/${NC_TOTAL} (${NC_PCT}%) KALAN:${KALAN} | GUT:${GUT_COUNT}/${GUT_TOTAL} (${GUT_PCT}%) | DISK:${DISK} | Tctl:${TCTL}"
    echo "$LINE" | tee -a "$LOG"
    sleep 60
done
