#!/usr/bin/env bash
# NEMES dış operatör tek-komut kurulum (testnet, v0.2.0).
# Yapar: miner indir + kayıt + ilk görev denemesi. SÜRÜCÜ ve GÖMME ucu sende.
# Gereksinim: Linux x86_64, 8GB RAM, 50GB boş SSD, 5 Mbps.
# Gömme ucu ŞART (LM Studio nomic-embed-text-v1.5 veya uyumlu OpenAI-API).
set -euo pipefail
echo "== NEMES operatör kurulum =="
[ $# -ge 2 ] || { echo "Kullanim: $0 <TRC20_CUZDAN> <MAKINE_ID> [EMBED_API]"; exit 1; }
CUZDAN="$1"; MAKINE="$2"; EMBED="${3:-http://127.0.0.1:1234}"
mkdir -p ~/nemes-operator && cd ~/nemes-operator
[ -f nemes-miner-v0.2.0-linux.zip ] || curl -sSL -o nemes-miner-v0.2.0-linux.zip https://nemes-x.space/miner/nemes-miner-v0.2.0-linux.zip
unzip -o -q nemes-miner-v0.2.0-linux.zip
echo "--- kayit (komuta) ---"
curl -s -X POST https://komuta.nemes-x.space/api/kayit -H 'Content-Type: application/json' \
  -d "{\"cuzdan\":\"$CUZDAN\",\"makine_id\":\"$MAKINE\"}" | tee kayit.json; echo
TOKEN=$(python3 -c "import json;print(json.load(open('kayit.json'))['token'])")
echo "$TOKEN" > miner.token && chmod 600 miner.token
echo "--- gömme ucu kontrol ---"
curl -s -m 10 "$EMBED/v1/models" >/dev/null && echo "embed OK: $EMBED" || { echo "UYARI: gömme ucuna erişilemiyor ($EMBED). LM Studio'yu başlatıp tekrar dene."; exit 1; }
echo "--- ilk görev denemesi ---"
./nemes-miner mine --simple --komuta https://komuta.nemes-x.space --token "$TOKEN" \
  --embed-api "$EMBED" --kira --kira-adet 200 &&
echo "OK: ilk batch tamam. Sürekli için systemd/tmux ile çalıştır (KURULUM.txt adım 3)."
