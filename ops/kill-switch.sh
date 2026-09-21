#!/usr/bin/env bash
# NEMES-X kill-switch v0 (PARANOID-MINER EPIC-05/EPIC-10)
# Amaç: tek komutla tüm madencilik + embed hattını <1sn hedefiyle durdur.
# Üretim DB'ye DOKUNMAZ, sadece proses + servis durdurur. Geri alma: servisleri restart et.
# Kullanım: sudo ./ops/kill-switch.sh [--kuru] [--neden "..."]
#   --kuru : sadece ne yapacağını gösterir, durdurmaz
set -euo pipefail
KURU=0
NEDEN="operator-kill-switch"
for a in "$@"; do
  case "$a" in
    --kuru) KURU=1 ;;
    --neden) shift || true ;;
    --neden=*) NEDEN="${a#--neden=}" ;;
  esac
done
LOG="/home/d3str0y1ng/nemes-testnet/kill-switch.log"
ms_now() {
  # ns çözünürlük, locale-bağımsız. %N yoksa saniyeye düş.
  local v
  v=$(LC_ALL=C date +%s%N 2>/dev/null || LC_ALL=C date +%s000000000)
  # sayısal değilse (BSD date) saniyeye düş
  case "$v" in
    ''|*[!0-9]*) v=$(( $(LC_ALL=C date +%s) * 1000000000 )) ;;
  esac
  echo $(( v / 1000000 ))
}
T0=$(ms_now)
kaydet() {
  if [ "$KURU" = "1" ]; then echo "[kuru-log] $1 neden=$NEDEN"; return 0; fi
  echo "$(date '+%F %T') KILL $1 neden=$NEDEN" | tee -a "$LOG"
}
calistir() {
  if [ "$KURU" = "1" ]; then echo "[kuru] $*"; else eval "$*"; fi
}
kaydet "basladi"
# 1. systemd madenciler (en hızlı yük kesici)
for s in nemes-miner-a nemes-miner-b; do
  if systemctl is-active --quiet "$s" 2>/dev/null; then
    kaydet "durdur: $s"
    calistir "sudo systemctl stop $s"
  else
    echo "atlandi (zaten duruk): $s"
  fi
done
# 2. llama embed sunucuları (GPU yükü)
for p in 1241 1242 1247; do
  if ss -ltn 2>/dev/null | grep -q ":$p "; then
    kaydet "fuser-k: $p/tcp"
    calistir "sudo fuser -k $p/tcp >/dev/null 2>&1 || true"
  fi
done
# 3. komuta dağıtımı duraklat (gossip DUR yerine dosyayla kilit - komuta-rs F1'de okur)
# Not: komuta servisi ÇALIŞIR kalır (kanıt/ledger korunur), sadece yeni görev dağıtımı durur.
calistir "touch /tmp/nemes-kill"
calistir "echo '$NEDEN $(date -Is)' | sudo tee /tmp/nemes-kill >/dev/null"
# 4. kalan miner prosesleri (güvence)
calistir "pkill -f '[n]emes-miner mine' || true"
T1=$(ms_now)
SURE=$((T1 - T0))
kaydet "bitti sure_ms=$SURE"
echo "kill-switch sure: ${SURE}ms (hedef <1000ms)"
echo "geri-alma: sudo systemctl restart nemes-miner-a nemes-miner-b; sudo rm -f /tmp/nemes-kill"
