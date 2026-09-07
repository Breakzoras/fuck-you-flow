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
  assets
  updates                       # latest.json: how an installed app learns a new version is out.
  changelog                     # what changed in every version, the English page. The Greek one rides along inside el/.
)
# Served from one level above the web root.
CONFIG=site.caddy

cd "$HERE"
echo "== build =="
PYTHONIOENCODING=utf-8 "$PY" build_site.py

echo "== what ships =="
for f in "${SHIP[@]}"; do
  [ -e "$f" ] || { echo "MISSING: $f"; exit 1; }
  echo "   $f"
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
chk(){ printf "   %-42s %s\n" "$1" "$(curl -s -o /dev/null -w '%{http_code}' "${@:2}")"; }
chk "Greek browser at /   want 302" -H "Accept-Language: el-GR,el;q=0.9" https://fuckyouflow.app/
chk "English at /         want 200" -H "Accept-Language: en-US" https://fuckyouflow.app/
chk "/el/                 want 200" https://fuckyouflow.app/el/
chk "/wispr-flow-alternative/ want 200" https://fuckyouflow.app/wispr-flow-alternative/
chk "/en/                 want 301" https://fuckyouflow.app/en/
chk "Google ownership     want 200" https://fuckyouflow.app/googlefde419ece67880c4.html
chk "Bing ownership       want 200" https://fuckyouflow.app/BingSiteAuth.xml
chk "build script         want 404" https://fuckyouflow.app/build_site.py
echo "done"
