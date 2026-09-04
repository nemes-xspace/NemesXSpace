#!/bin/bash
# Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
# NEMES-X ilerleme izleyici - 60sn'de bir newscrawl/gut_en/disk/ısı loglar
LOG="/home/d3str0y1ng/NemesXSpace/izle.log"
DB_NC="/srv/beyin/wiki/wiki_newscrawl_tr.db"
DB_GUT="/srv/beyin/wiki/wiki_gut_en.db"

echo "=== NEMES-X IZLEYICI BASLADI $(date '+%Y-%m-%d %H:%M:%S') ===" | tee -a "$LOG"

while true; do
    TS=$(date '+%m-%d %H:%M:%S')
    
    # newscrawl sayımı (vektor tablosu)
    NC_COUNT=$(python3 -c "
import sqlite3, glob
total=0
for p in glob.glob('/srv/beyin/wiki/wiki_newscrawl_tr_emb_*.db'):
    try:
        c=sqlite3.connect(f'file:{p}?mode=ro', uri=True, timeout=1)
        cur=c.cursor()
        cur.execute('SELECT COUNT(*) FROM vektor')
        total+=cur.fetchone()[0]
        c.close()
    except: pass
print(total)
" 2>/dev/null)
    
    NC_TOTAL=$(python3 -c "import sqlite3; c=sqlite3.connect('file:/srv/beyin/wiki/wiki_newscrawl_tr.db?mode=ro', uri=True, timeout=1); print(c.execute('SELECT COUNT(*) FROM madde').fetchone()[0])" 2>/dev/null)
    if [ -n "$NC_COUNT" ] && [ -n "$NC_TOTAL" ] && [ "$NC_TOTAL" -gt 0 ]; then
        NC_PCT=$(python3 -c "print(f'{( $NC_COUNT / $NC_TOTAL * 100):.2f}')" 2>/dev/null)
        KALAN=$((NC_TOTAL - NC_COUNT))
    else
        NC_PCT="?"
        KALAN="?"
    fi
    
    # gut_en
    GUT_COUNT=$(python3 -c "
import sqlite3, glob
total=0
for p in glob.glob('/srv/beyin/wiki/wiki_gut_en_emb_*.db'):
    try:
        c=sqlite3.connect(f'file:{p}?mode=ro', uri=True, timeout=1)
        cur=c.cursor()
        cur.execute('SELECT COUNT(*) FROM vektor')
        total+=cur.fetchone()[0]
        c.close()
    except: pass
print(total)
" 2>/dev/null)
    GUT_TOTAL=$(python3 -c "import sqlite3; c=sqlite3.connect('file:/srv/beyin/wiki/wiki_gut_en.db?mode=ro', uri=True, timeout=1); print(c.execute('SELECT COUNT(*) FROM madde').fetchone()[0])" 2>/dev/null)
    if [ -n "$GUT_COUNT" ] && [ -n "$GUT_TOTAL" ] && [ "$GUT_TOTAL" -gt 0 ]; then
        GUT_PCT=$(python3 -c "print(f'{( $GUT_COUNT / $GUT_TOTAL * 100):.2f}')" 2>/dev/null)
    else
        GUT_PCT="?"
    fi
    
    DISK=$(df -h /srv/beyin | awk 'NR==2{print $4" boş ("$5" dolu)"}')
    TCTL=$(sensors 2>/dev/null | grep Tctl | awk '{print $2}' || echo "?")
    WORKERS=$(pgrep -f "wiki_embed_par.py newscrawl" | wc -l)
    EMBED_PS=$(ps aux | grep "wiki_embed_par.py newscrawl" | grep -v grep | wc -l)
    
    # hız hesabı (son log farkı)
    LINE="[$TS] NC:${NC_COUNT}/${NC_TOTAL} (${NC_PCT}%) KALAN:${KALAN} | GUT:${GUT_COUNT}/${GUT_TOTAL} (${GUT_PCT}%) | DISK:${DISK} | Tctl:${TCTL} | workers:${WORKERS}"
    echo "$LINE" | tee -a "$LOG"
    
    # Kritik uyarılar
    DISK_GB=$(df -P /srv/beyin | awk 'NR==2{print $4/1048576}' | cut -d. -f1)
    if [ "$DISK_GB" -lt 40 ]; then
        echo "[$TS] UYARI: Disk ${DISK_GB}GB <40GB KRITIK!" | tee -a "$LOG"
    fi
    
    # gut_en resume kontrolü - newscrawl bitti mi?
    if [ -n "$NC_COUNT" ] && [ -n "$NC_TOTAL" ] && [ "$NC_COUNT" -ge "$NC_TOTAL" ] && [ "$NC_TOTAL" -gt 0 ]; then
        echo "[$TS] ✅ newscrawl_tr TAMAM! gut_en resume bekleniyor (beyin_sirasi.py)" | tee -a "$LOG"
    fi
    
    sleep 60
done
