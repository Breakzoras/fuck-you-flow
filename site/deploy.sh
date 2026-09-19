#!/usr/bin/env bash
# Publish fuckyouflow.app.
#
# Ships an EXPLICIT list of files. Never copy this folder wholesale: it also holds the
# build scripts, and a stray copy would publish them.
#
#   bash site/deploy.sh            build, back up, upload, reload, verify
#   bash site/deploy.sh --dry-run  show what would ship and stop
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SSH_OPTS=(-o IdentitiesOnly=yes -i "$HOME/.ssh/occ_site_deploy")
HOST="occadmin@193.181.216.120"
PY="C:/Python314/python.exe"

# Everything that is allowed to reach the public server, and nothing else.
SHIP=(
  index.html
  404.html
  robots.txt
  sitemap.xml
  llms.txt
  llms-full.txt
  style.css
  theme.js
  googlefde419ece67880c4.html   # Google Search Console ownership. Removing it loses it.
  BingSiteAuth.xml              # Bing Webmaster Tools ownership. Same.
  el
  wispr-flow-alternative
  openwhispr-alternative
  free-offline-dictation-alternatives
  assets
  updates                       # latest.json: how an installed app learns a new version is out.
  changelog                     # what changed in every version, the English page. The Greek one rides along inside el/.
  guides
  privacy
)
# Served from one level above the web root.
CONFIG=site.caddy

cd "$HERE"
echo "== build =="
PYTHONIOENCODING=utf-8 "$PY" build_site.py
PYTHONIOENCODING=utf-8 "$PY" audit_site.py
INDEXNOW_FILE=$("$PY" notify_indexnow.py --ship-filename)
if [ -n "$INDEXNOW_FILE" ]; then SHIP+=("$INDEXNOW_FILE"); fi

echo "== what ships =="
for f in "${SHIP[@]}"; do
  [ -e "$f" ] || { echo "MISSING: $f"; exit 1; }
  if [ "$f" = "$INDEXNOW_FILE" ]; then echo "   IndexNow ownership file"; else echo "   $f"; fi
done
echo "   $CONFIG  (goes to /srv/fuckyouflow/, not into the web root)"

if [ "${1:-}" = "--dry-run" ]; then echo "dry run, nothing sent"; exit 0; fi

echo "== back up what is live =="
TS=$(ssh "${SSH_OPTS[@]}" "$HOST" 'TS=$(date +%Y%m%d-%H%M%S); sudo -n cp -a /srv/fuckyouflow /srv/fuckyouflow-backup-$TS; echo "$TS" | sudo -n tee /srv/fuckyouflow-last-backup.txt >/dev/null; echo $TS')
echo "   /srv/fuckyouflow-backup-$TS"

echo "== upload =="
tar -czf /tmp/fyf-deploy.tgz "${SHIP[@]}" "$CONFIG"
scp "${SSH_OPTS[@]}" /tmp/fyf-deploy.tgz "$HOST":~/ >/dev/null
ssh "${SSH_OPTS[@]}" "$HOST" 'set -e
  rm -rf ~/fyf-in && mkdir ~/fyf-in && tar -xzf ~/fyf-deploy.tgz -C ~/fyf-in
  sudo -n mv ~/fyf-in/site.caddy /srv/fuckyouflow/site.caddy
  sudo -n cp -a ~/fyf-in/. /srv/fuckyouflow/site/
  sudo -n chown -R occadmin:sudo /srv/fuckyouflow
  rm -rf ~/fyf-in ~/fyf-deploy.tgz'

echo "== validate, then reload =="
ssh "${SSH_OPTS[@]}" "$HOST" 'set -e
  C=$(sudo -n docker ps --filter name=caddy -q | head -1)
  sudo -n docker exec $C caddy validate --config /etc/caddy/Caddyfile >/dev/null 2>&1 || { echo "CONFIG INVALID, nothing reloaded"; exit 1; }
  sudo -n docker exec $C caddy reload --config /etc/caddy/Caddyfile >/dev/null 2>&1
  echo "   reloaded"'

echo "== verify on the live site =="
chk(){ local want="$1"; shift; local label="$1"; shift; local got; got=$(curl -s -o /dev/null -w '%{http_code}' "$@"); printf "   %-42s %s\n" "$label" "$got"; [ "$got" = "$want" ] || { echo "unexpected status: wanted $want"; exit 1; }; }
chk 200 "Greek browser at /" -H "Accept-Language: el-GR,el;q=0.9" https://fuckyouflow.app/
chk 200 "English at /" -H "Accept-Language: en-US" https://fuckyouflow.app/
chk 200 "/el/" https://fuckyouflow.app/el/
chk 200 "/wispr-flow-alternative/" https://fuckyouflow.app/wispr-flow-alternative/
chk 301 "/en/" https://fuckyouflow.app/en/
chk 200 "Google ownership" https://fuckyouflow.app/googlefde419ece67880c4.html
chk 200 "Bing ownership" https://fuckyouflow.app/BingSiteAuth.xml
chk 404 "build script" https://fuckyouflow.app/build_site.py
PYTHONIOENCODING=utf-8 "$PY" audit_site.py --live
if [ -n "$INDEXNOW_FILE" ]; then
  echo "== notify changed URLs through IndexNow =="
  PYTHONIOENCODING=utf-8 "$PY" notify_indexnow.py --submit --output .indexnow/last-submission.json
fi
echo "done"
