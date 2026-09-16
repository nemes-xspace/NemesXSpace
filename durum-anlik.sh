#!/usr/bin/env bash
# durum-anlik.sh - 10 saniyelik canli durum ozeti (00-OTURUM-DEVAM.md §7 adim 2).
# Agir DB sayimi YAPMAZ; sadece hizli kontroller.
cd /home/d3str0y1ng/NemesXSpace || exit 1
echo "=== $(date '+%F %T') ==="
echo "--- GIT ---"; git log --oneline -3; git status -sb | head -n 2
echo "--- SERVIS ---"; systemctl is-active beyin_bekci nemes-komuta nemes-miner-a nemes-miner-b caddy cloudflared 2>&1 | tr '\n' ' '; echo
echo "--- LLAMA ---"; for p in 1241 1242 1243 1244 1245 1246; do printf "$p:"; curl -s -m 3 http://127.0.0.1:$p/health 2>/dev/null | head -c 12; echo; done
echo "--- KOMUTA ---"; curl -s -m 5 http://127.0.0.1:8787/health 2>&1 | head -c 60; echo
echo "--- DISK ---"; df -h / /srv/beyin 2>/dev/null | tail -n 2
echo "--- FAISS ---"; ls /srv/beyin/wiki/*.faiss 2>/dev/null | wc -l
echo "--- UPTIME ---"; uptime
