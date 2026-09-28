#!/usr/bin/env bash
# Puts the report receiver on the fuckyouflow.app server, or updates it.
#
#   COPY_OCC_KEY=1 bash server/report-receiver/deploy.sh   (or RESEND_API_KEY=re_...)
#
# The key is needed only the first time (it is kept in /srv/fyf-reports/env,
# readable by root only). The site's Caddy route to it ships with site.caddy
# through site/deploy.sh; run this first so the route has somewhere to go.
set -euo pipefail
cd "$(dirname "$0")"
SSH_OPTS=(-o IdentitiesOnly=yes -i "$HOME/.ssh/occ_site_deploy")
HOST="occadmin@193.181.216.120"
FROM="${REPORT_FROM:-FU Flow reports <reports@oneclickclaw.io>}"
TO="${REPORT_TO:-info@luram.gr}"

"${PY:-python}" -m unittest test_receiver.py
scp "${SSH_OPTS[@]}" receiver.py "$HOST":~/fyf-receiver.py >/dev/null
if [ -n "${RESEND_API_KEY:-}" ]; then
  printf 'RESEND_API_KEY=%s\nREPORT_TO=%s\nREPORT_FROM=%s\nREPORT_DIR=/data/reports\n' "$RESEND_API_KEY" "$TO" "$FROM" |
    ssh "${SSH_OPTS[@]}" "$HOST" 'umask 077; cat > ~/fyf-reports.env'
fi
ssh "${SSH_OPTS[@]}" "$HOST" 'set -e
  sudo -n mkdir -p /srv/fyf-reports/app /srv/fyf-reports/data
  sudo -n mv ~/fyf-receiver.py /srv/fyf-reports/app/receiver.py
  if [ -f ~/fyf-reports.env ]; then sudo -n mv ~/fyf-reports.env /srv/fyf-reports/env; sudo -n chmod 600 /srv/fyf-reports/env; sudo -n chown root:root /srv/fyf-reports/env; fi
  # COPY_OCC_KEY=1: reuse the OneClickClaw send-only Resend key (oneclickclaw.io
  # is the verified sender). It is copied on the server and never leaves it.
  if [ ! -f /srv/fyf-reports/env ] && [ "'"${COPY_OCC_KEY:-0}"'" = 1 ]; then
    K=$(sudo -n docker exec webdock-backend-1 printenv RESEND_API_KEY | tr -d "\r")
    [ -n "$K" ] || { echo "NO OCC KEY"; exit 1; }
    printf "RESEND_API_KEY=%s\nREPORT_TO=%s\nREPORT_FROM=%s\nREPORT_DIR=/data/reports\n" "$K" "'"$TO"'" "'"$FROM"'" | sudo -n tee /srv/fyf-reports/env >/dev/null
    sudo -n chmod 600 /srv/fyf-reports/env; sudo -n chown root:root /srv/fyf-reports/env
  fi
  [ -f /srv/fyf-reports/env ] || { echo "NO KEY: run once with RESEND_API_KEY set"; exit 1; }
  sudo -n docker rm -f fyf-reports >/dev/null 2>&1 || true
  sudo -n docker run -d --name fyf-reports --restart unless-stopped --network webdock_web \
    --read-only --tmpfs /tmp --memory 96m --cpus 0.5 \
    -v /srv/fyf-reports/app:/app:ro -v /srv/fyf-reports/data:/data \
    --env-file /srv/fyf-reports/env python:3.12-alpine python -u /app/receiver.py >/dev/null
  sleep 3
  sudo -n docker exec fyf-reports python -c "import urllib.request;print(urllib.request.urlopen(\"http://127.0.0.1:8787/api/report/health\").read().decode())"'
echo "receiver up; now ship site.caddy with site/deploy.sh"
