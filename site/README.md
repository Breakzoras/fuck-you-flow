# fuckyouflow.app

The landing page. Three pages, no framework, no CDN, no build tooling beyond one Python script.

## What is where

| Path | What it is |
|---|---|
| `index.html` | English, the root, the page search engines index |
| `el/index.html` | Greek |
| `wispr-flow-alternative/index.html` | The comparison page, written by hand |
| `404.html`, `robots.txt`, `sitemap.xml`, `llms.txt`, `llms-full.txt` | The usual furniture |
| `assets/` | Icons, the social card, app screenshots and the four brand images |
| `site.caddy` | The Caddy block the server imports. It lives at `/srv/fuckyouflow/`, one level ABOVE the web root |
| `googlefde419ece67880c4.html`, `BingSiteAuth.xml` | Search engine ownership. Deleting either loses the verification |

**`index.html` and `el/index.html` are generated. Editing them by hand is wasted work.**
Both come from `build_site.py`, which reads the locked design at `variants/v4-coral.html`
and applies the translation table. Change the design file or the table, then run the build.

## Scripts, none of which ship

- `build_site.py` builds the two language pages. It fails loudly if any English survives on
  the Greek page, and it writes the visible questions and the FAQ structured data from one
  shared list so the two cannot drift apart.
- `make_art.py` turns the four original renders into the lean WebP copies and the social card.
- `shoot_pages.py` photographs every page in dark, light and phone, and reports console errors.
- `deploy.sh` publishes.

## Deploy

```
bash site/deploy.sh --dry-run    # see exactly what would ship
bash site/deploy.sh              # build, back up, upload, reload, verify
```

It ships an explicit list of files. **Never copy this folder wholesale to the server**, because
the scripts and any working notes would go public with it. The script backs up the live site
first and records the timestamp in `/srv/fuckyouflow-last-backup.txt`, validates the Caddy
config before reloading, and finishes by checking the live URLs.

## How the two languages are served

English is the root. Greek is `/el/`. A visitor whose browser asks for Greek is sent from `/`
to `/el/` with a temporary redirect, and only from `/`. Crawlers send no language header, so
they always receive the English root. The English link on the Greek page carries `?lang=en`,
which switches the redirect off for that visit; the English page then removes the query from
the address bar. The old `/en/` path redirects permanently to `/`.

Only two lines of this live in the OneClickClaw repository: an `import` in
`deploy/webdock/Caddyfile` and a read-only mount in `deploy/webdock/docker-compose.prod.yml`.
They were merged on 6 September 2026 and do not change again. **Changing `site.caddy` needs no
pull request anywhere.**
