#!/bin/bash
# Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
LOG="/home/d3str0y1ng/NemesXSpace/izle.log"
echo "=== NEMES-X IZLEYICI SIMPLE $(date) ===" | tee -a "$LOG"
while true; do
  TS=$(date '+%m-%d %H:%M:%S')
  NC_SIZE=$(du -ch /srv/beyin/wiki/wiki_newscrawl_tr_emb_*.db 2>/dev/null | tail -1 | awk '{print $1}')
  NC_FILES=$(ls /srv/beyin/wiki/wiki_newscrawl_tr_emb_*.db 2>/dev/null | wc -l)
  GUT_SIZE=$(du -ch /srv/beyin/wiki/wiki_gut_en_emb_*.db 2>/dev/null | tail -1 | awk '{print $1}')
  DISK=$(df -h /srv/beyin 2>/dev/null | awk 'NR==2{print $4" boş ("$5" dolu)"}')
  TCTL=$(sensors 2>/dev/null | grep Tctl | awk '{print $2}')
  WORKERS=$(pgrep -f "wiki_embed_par.py newscrawl" 2>/dev/null | wc -l)
  SIRA=$(tail -1 /srv/beyin/kaynaklar/sira.log 2>/dev/null | cut -c1-80)
  LINE="[$TS] NC: ${NC_SIZE} (${NC_FILES} shard) | GUT: ${GUT_SIZE} | DISK: ${DISK} | Tctl:${TCTL} | workers:${WORKERS}"
  echo "$LINE" | tee -a "$LOG"
  echo "  sira: $SIRA" | tee -a "$LOG"
  sleep 60
done
