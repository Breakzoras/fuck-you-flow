# fuckyouflow.app

The landing page. Static files, no build step.

- `index.html` Greek, `en/index.html` English, `404.html`
- `style.css`, `theme.js` (dark or light, remembered per browser)
- `llms.txt`, `llms-full.txt`, `robots.txt`, `sitemap.xml`
- `assets/` favicon, icons, social card and four app screenshots; `make_assets.py` regenerates the first three from `design/icon-fu-dark.png`
- `site.caddy` the Caddy block the server imports

Deploy: copy everything except `site.caddy`, `make_assets.py` and this file to `/srv/fuckyouflow/site/` on the server that runs Caddy. The Caddy block and the compose mount live in the OneClickClaw deploy config.
