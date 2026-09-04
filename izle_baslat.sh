#!/bin/bash
# Copyright (c) 2026 NEMES-X. All Rights Reserved. Unauthorized use prohibited.
# izleyiciyi tmux veya nohup ile başlat
TMUX_NAME="nemes-izle"
if tmux has-session -t $TMUX_NAME 2>/dev/null; then
    echo "tmux $TMUX_NAME zaten var, yeniden başlatılıyor"
    tmux kill-session -t $TMUX_NAME
fi
tmux new-session -d -s $TMUX_NAME "/home/d3str0y1ng/NemesXSpace/izle_progress.sh"
echo "tmux session $TMUX_NAME başlatıldı"
sleep 2
tmux capture-pane -t $TMUX_NAME -p | tail -n 20
echo "---"
echo "Log: /home/d3str0y1ng/NemesXSpace/izle.log"
echo "İzle: tmux attach -t $TMUX_NAME  (çıkış: Ctrl+b d)"
echo "Durdur: tmux kill-session -t $TMUX_NAME"
