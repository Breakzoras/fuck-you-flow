"""Notify Bing/IndexNow about verified, changed canonical pages after deployment.

Default mode is a local preview. --submit checks the public ownership file and
exact deployed page bytes before notifying. A 200/202 response is receipt, not
indexing. State and the ownership key stay out of Git and public audit logs.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import secrets
import sys
import urllib.error
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET

from site_data import BASE

HERE = Path(__file__).resolve().parent
LOCAL = HERE / '.indexnow'
CONFIG = LOCAL / 'config.json'
STATE = LOCAL / 'state.json'
ENDPOINT = 'https://www.bing.com/indexnow'
NS = {'s': 'http://www.sitemaps.org/schemas/sitemap/0.9'}


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix('.tmp')
    temporary.write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')
    temporary.replace(path)


def config():
    data = json.loads(CONFIG.read_text(encoding='utf-8'))
    key = data['key']
    if not re.fullmatch(r'[a-f0-9]{32}', key):
        raise ValueError('Invalid local IndexNow configuration')
    filename = 'indexnow-' + key + '.txt'
    if (HERE / filename).read_text(encoding='utf-8').strip() != key:
        raise ValueError('Local IndexNow ownership file is missing or mismatched')
    return key, filename


def page_hashes():
    result = {}
    for entry in ET.parse(HERE / 'sitemap.xml').findall('s:url', NS):
        url = entry.findtext('s:loc', namespaces=NS)
        parsed = urllib.parse.urlsplit(url)
        if (parsed.scheme + '://' + parsed.netloc != BASE or parsed.query
                or parsed.fragment or not parsed.path.endswith('/')):
            raise ValueError('IndexNow only accepts this site\'s canonical URLs')
        path = (HERE / urllib.parse.unquote(parsed.path).lstrip('/') / 'index.html').resolve()
        if not path.is_relative_to(HERE):
            raise ValueError('Page path leaves the site')
        result[url] = hashlib.sha256(path.read_bytes()).hexdigest()
    return result


def fetch(url):
    req = urllib.request.Request(url, headers={'User-Agent': 'FUFlow-IndexNow/1.0'})
    with urllib.request.urlopen(req, timeout=25) as response:
        if response.status != 200 or response.url != url:
            raise ValueError('Public URL is unavailable or redirects')
        return response.read()


def run(args):
    if args.init:
        if not CONFIG.exists():
            key = secrets.token_hex(16)
            (HERE / ('indexnow-' + key + '.txt')).write_text(key, encoding='utf-8')
            write_json(CONFIG, {'key': key})
        config()
        print('IndexNow ownership configured. No URLs submitted.')
        return
    if args.ship_filename:
        if CONFIG.exists():
            print(config()[1])
        return
    hashes = page_hashes()
    previous = json.loads(STATE.read_text(encoding='utf-8')) if STATE.exists() else {}
    changed = [url for url, digest in hashes.items() if previous.get('hashes', {}).get(url) != digest]
    report = {'checked_at': datetime.now(timezone.utc).isoformat(),
              'mode': 'submit' if args.submit else 'preview', 'endpoint': ENDPOINT,
              'changed_count': len(changed), 'urls': changed, 'status': 'unchanged' if not changed else 'preview'}
    if changed and args.submit:
        key, filename = config()
        location = BASE + '/' + filename
        if fetch(location).decode('utf-8').strip() != key:
            raise ValueError('Public IndexNow ownership verification failed')
        for url in changed:
            if hashlib.sha256(fetch(url)).hexdigest() != hashes[url]:
                raise ValueError('Public page differs from local build: ' + url)
        payload = {'host': urllib.parse.urlsplit(BASE).netloc, 'key': key,
                   'keyLocation': location, 'urlList': changed}
        request = urllib.request.Request(ENDPOINT, data=json.dumps(payload).encode('utf-8'),
                                         headers={'Content-Type': 'application/json; charset=utf-8',
                                                  'User-Agent': 'FUFlow-IndexNow/1.0'}, method='POST')
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                code = response.status
        except urllib.error.HTTPError as error:
            code = error.code
        report['http_status'] = code
        report['status'] = {200: 'received', 202: 'received_key_validation_pending'}.get(code, 'rejected')
        if code in (200, 202):
            write_json(STATE, {'hashes': hashes, 'last_receipt': report})
        if args.output:
            write_json(args.output, report)
        print(json.dumps(report, indent=2))
        if code not in (200, 202):
            raise RuntimeError('IndexNow rejected submission. No automatic retry or state advance.')
        return
    if args.output:
        write_json(args.output, report)
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--init', action='store_true')
    mode.add_argument('--ship-filename', action='store_true')
    mode.add_argument('--submit', action='store_true')
    parser.add_argument('--output', type=Path)
    try:
        run(parser.parse_args())
    except (OSError, ValueError, RuntimeError) as error:
        # Avoid echoing URLs from networking exceptions: ownership paths contain the key.
        print('IndexNow failed: ' + (str(error) if isinstance(error, (ValueError, RuntimeError)) else type(error).__name__), file=sys.stderr)
        sys.exit(1)
