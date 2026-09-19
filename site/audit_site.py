"""Check generated pages and, with --live, actual deployment responses.

Validates page structure, language pairs, internal links/assets/fragments,
release consistency and JSON-LD. It does not measure rankings or certify CWV.
"""
import argparse
from collections import Counter
from html.parser import HTMLParser
from pathlib import Path
import json
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET

from site_data import BASE, DL, VERSION, SIZE_BYTES

HERE = Path(__file__).resolve().parent
NS = {'s': 'http://www.sitemaps.org/schemas/sitemap/0.9', 'x': 'http://www.w3.org/1999/xhtml'}


class Page(HTMLParser):
    def __init__(self, body):
        super().__init__(convert_charrefs=True)
        self.lang = ''
        self.titles, self.h1s, self.canonicals, self.descriptions = [], [], [], []
        self.links, self.assets, self.ids, self.alternates = [], [], set(), {}
        self.schemas, self.errors = [], []
        self.robots, self.meta = '', {}
        self.capture, self.buffer = None, ''
        self.feed(body)

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if a.get('id'):
            if a['id'] in self.ids:
                self.errors.append('Duplicate id: ' + a['id'])
            self.ids.add(a['id'])
        if tag == 'html':
            self.lang = a.get('lang', '')
        if tag == 'meta':
            key = a.get('name', a.get('property', ''))
            self.meta[key] = a.get('content', '')
            if key == 'description': self.descriptions.append(a.get('content', ''))
            if key == 'robots': self.robots = a.get('content', '')
        if tag == 'link':
            rel = a.get('rel', '')
            if rel == 'canonical': self.canonicals.append(a.get('href', ''))
            if rel == 'alternate' and 'hreflang' in a: self.alternates[a['hreflang']] = a.get('href', '')
            if rel in ('stylesheet', 'icon', 'apple-touch-icon'): self.assets.append(a.get('href', ''))
        if tag == 'a' and 'href' in a: self.links.append(a['href'])
        if tag in ('img', 'script') and 'src' in a: self.assets.append(a['src'])
        if tag == 'img' and 'alt' not in a: self.errors.append('Image lacks alt: ' + a.get('src', ''))
        if tag in ('title', 'h1') or tag == 'script' and a.get('type') == 'application/ld+json':
            self.capture = tag
            self.buffer = ''

    def handle_data(self, data):
        if self.capture: self.buffer += data

    def handle_endtag(self, tag):
        if self.capture != tag: return
        if tag == 'title': self.titles.append(self.buffer.strip())
        if tag == 'h1': self.h1s.append(self.buffer.strip())
        if tag == 'script':
            try: self.schemas.append(json.loads(self.buffer))
            except ValueError: self.errors.append('Invalid JSON-LD')
        self.capture = None


def site_file(url):
    path = urllib.parse.unquote(urllib.parse.urlsplit(url).path).lstrip('/')
    result = (HERE / path).resolve()
    if not result.is_relative_to(HERE): raise ValueError('Internal path leaves site: ' + url)
    return result / 'index.html' if result.is_dir() else result


def request(url, method='GET', headers=None):
    started = time.monotonic()
    req = urllib.request.Request(url, method=method, headers={'User-Agent': 'FUFlow-site-audit/1.0', **(headers or {})})
    try:
        response = urllib.request.urlopen(req, timeout=25)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        return response.status, response.url, dict(response.headers), response.read(), round(time.monotonic() - started, 3)


def audit(live=False):
    errors, warnings, records = [], [], []
    sitemap = ET.parse(HERE / 'sitemap.xml')
    entries = sitemap.findall('s:url', NS)
    urls = [entry.findtext('s:loc', namespaces=NS) for entry in entries]
    # A rendered page must not silently escape validation by being omitted from the sitemap.
    for candidate in HERE.rglob('index.html'):
        if any(part.startswith('.') or part in ('variants', 'node_modules') for part in candidate.relative_to(HERE).parts):
            continue
        candidate_page = Page(candidate.read_text(encoding='utf-8'))
        for canonical in candidate_page.canonicals:
            if canonical.startswith(BASE + '/') and canonical not in urls:
                errors.append('Generated canonical omitted from sitemap: ' + canonical)
    if len(urls) != len(set(urls)): errors.append('Duplicate sitemap URLs')
    if any(not u.startswith(BASE + '/') or not u.endswith('/') for u in urls): errors.append('Non-canonical sitemap path')
    pages = {}
    assets = set()
    for entry, url in zip(entries, urls):
        path = site_file(url)
        if not path.is_file(): errors.append('Missing page: ' + url); continue
        body = path.read_text(encoding='utf-8')
        p = Page(body)
        pages[url] = p
        errors.extend(url + ': ' + e for e in p.errors)
        if len(p.titles) != 1 or not p.titles[0]: errors.append(url + ': expected one nonempty title')
        if len(p.h1s) != 1 or not p.h1s[0]: errors.append(url + ': expected one nonempty H1')
        if p.canonicals != [url]: errors.append(url + ': wrong canonical')
        if len(p.descriptions) != 1 or not p.descriptions[0]: errors.append(url + ': description missing/duplicated')
        if p.lang not in ('en', 'el'): errors.append(url + ': unsupported/missing language')
        if 'noindex' in p.robots.lower() or 'nofollow' in p.robots.lower(): errors.append(url + ': indexing blocked')
        if '{{' in body or '}}' in re.sub(r'<script.*?</script>|<style.*?</style>', '', body, flags=re.S): errors.append(url + ': unexpanded content token')
        if chr(0x2014) in body: errors.append(url + ': prohibited punctuation')
        if p.meta.get('og:url') != url: errors.append(url + ': incorrect social URL')
        for name in ('og:title', 'og:description', 'og:image', 'twitter:card'):
            if not p.meta.get(name): errors.append(url + ': missing ' + name)
        if p.meta.get('og:image'): assets.add(p.meta['og:image'])
        for target in p.links + p.assets:
            absolute = urllib.parse.urljoin(url, target)
            parts = urllib.parse.urlsplit(absolute)
            if '/releases/download/' in absolute and '.exe' in absolute and absolute != DL:
                errors.append(url + ': stale installer: ' + absolute)
            if parts.netloc == urllib.parse.urlsplit(BASE).netloc:
                file = site_file(absolute)
                if not file.is_file(): errors.append(url + ': broken internal resource: ' + target)
                if target in p.assets or parts.path.endswith('.json'): assets.add(urllib.parse.urldefrag(absolute).url)
        expected = {a.attrib['hreflang']: a.attrib['href'] for a in entry.findall('x:link', NS)}
        if p.alternates != expected: errors.append(url + ': HTML and sitemap language pairs differ')
        for schema in p.schemas:
            for node in schema.get('@graph', [schema]):
                if node.get('@type') == 'SoftwareApplication' and 'softwareVersion' in node:
                    if node.get('softwareVersion') != VERSION or node.get('downloadUrl') != DL: errors.append(url + ': stale application schema')
                    if node.get('fileSize') != f'{SIZE_BYTES} B': errors.append(url + ': incorrect installer size')
        records.append({'url': url, 'title': p.titles, 'description_characters': [len(d) for d in p.descriptions], 'html_bytes': path.stat().st_size, 'language': p.lang, 'links': len(p.links), 'schemas': len(p.schemas)})
    for url, page in pages.items():
        for lang, alternate in page.alternates.items():
            if alternate not in pages or pages[alternate].alternates != page.alternates: errors.append(url + ': non-reciprocal hreflang')
        for target in page.links:
            absolute, fragment = urllib.parse.urldefrag(urllib.parse.urljoin(url, target))
            absolute = absolute.split('?')[0]
            if fragment and absolute in pages and fragment not in pages[absolute].ids: errors.append(url + ': broken fragment: ' + target)
    for title, count in Counter(p.titles[0] for p in pages.values() if p.titles).items():
        if count > 1: errors.append('Duplicate title: ' + title)
    # Every discoverable canonical page must be reachable through navigation from home.
    reachable, queue = set(), [BASE + '/']
    while queue:
        current = queue.pop()
        if current in reachable or current not in pages: continue
        reachable.add(current)
        queue.extend(urllib.parse.urldefrag(urllib.parse.urljoin(current, u)).url.split('?')[0] for u in pages[current].links)
    errors.extend('Orphan page: ' + url for url in pages if url not in reachable)
    robots = (HERE / 'robots.txt').read_text(encoding='utf-8')
    if 'Sitemap: ' + BASE + '/sitemap.xml' not in robots: errors.append('Robots sitemap declaration missing')
    for name in ('llms.txt', 'llms-full.txt'):
        body = (HERE / name).read_text(encoding='utf-8')
        if DL not in body or VERSION not in body: errors.append(name + ': stale release facts')
    network = []
    if live:
        for url in urls + [BASE + '/robots.txt', BASE + '/sitemap.xml', BASE + '/llms.txt', BASE + '/llms-full.txt', BASE + '/googlefde419ece67880c4.html', BASE + '/BingSiteAuth.xml']:
            status, final, headers, data, seconds = request(url)
            network.append({'url': url, 'status': status, 'final': final, 'seconds': seconds, 'bytes': len(data)})
            if status != 200 or final != url: errors.append('Live canonical does not return 200 directly: ' + url)
            if 'noindex' in headers.get('X-Robots-Tag', '').lower(): errors.append('Live noindex header: ' + url)
            if data != site_file(url).read_bytes(): errors.append('Live bytes differ from generated file: ' + url)
        for url in sorted(assets):
            status, final, headers, data, seconds = request(url, 'HEAD')
            if status != 200: errors.append('Live asset unavailable: ' + url)
        for name in ('missing-fuflow-audit-page', 'build_site.py', 'build_content.py', 'comparisons_source.py', 'comparison_greek_source.py', 'workflows_source.py', 'site_data.py', 'audit_site.py', 'site.caddy'):
            status, *rest = request(BASE + '/' + name)
            if status != 404: errors.append('Expected 404: ' + name)
        for agent in ('Googlebot', 'bingbot', 'GPTBot', 'OAI-SearchBot', 'ClaudeBot'):
            status, final, headers, data, seconds = request(BASE + '/', headers={'User-Agent': agent})
            if status != 200 or data != site_file(BASE + '/').read_bytes(): errors.append('Crawler response differs: ' + agent)
        status, final, headers, data, seconds = request(BASE + '/', headers={'Accept-Language': 'el-GR,el;q=0.9'})
        if status != 200 or final != BASE + '/': errors.append('Forced language redirect remains')
        for old, target in [(BASE + '/en/', BASE + '/'), ('https://www.fuckyouflow.app/', BASE + '/'), ('http://fuckyouflow.app/', BASE + '/')]:
            status, final, headers, data, seconds = request(old)
            if final != target or status != 200: errors.append('Legacy/host redirect failed: ' + old)
        status, final, headers, data, seconds = request(DL, 'HEAD')
        if status != 200 or int(headers.get('Content-Length', '0')) != SIZE_BYTES: errors.append('Current download/size verification failed')
    return {'mode': 'live' if live else 'local', 'page_count': len(pages), 'reachable_pages': len(reachable), 'asset_count': len(assets), 'errors': errors, 'warnings': warnings, 'pages': records, 'network': network}


if __name__ == '__main__':
    sys.stdout.reconfigure(encoding='utf-8')
    parser = argparse.ArgumentParser()
    parser.add_argument('--live', action='store_true')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    report = audit(args.live)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding='utf-8')
    print(json.dumps({k: v for k, v in report.items() if k not in ('pages', 'network')}, ensure_ascii=False, indent=2))
    sys.exit(bool(report['errors']))
