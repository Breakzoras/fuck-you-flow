"""Generate the comparison, guide library and technical privacy pages."""
from pathlib import Path
from html import escape
import json
import re

from build_changelog import head_bits
import site_nav
from guides_source import GUIDES
from site_data import BASE, REPO, VERSION, SIZE, SIZE_EL, DL, CONTENT_DATE, OG_IMAGE
from workflows_source import WORKFLOWS, PRIVACY
from comparisons_source import COMPARISONS
from comparison_greek_source import WISPR_EL
from voice_source import VOICE_TO_TEXT

HERE = Path(__file__).resolve().parent
# The plain-words voice to text guide leads the library: it answers the queries
# with by far the most searches (see voice_source.py).
ALL_GUIDES = [VOICE_TO_TEXT] + GUIDES + WORKFLOWS


def paired_paths():
    return ['/wispr-flow-alternative/', '/guides/', '/privacy/'] + [f'/guides/{g["slug"]}/' for g in ALL_GUIDES]


def comparison_paths():
    return ['/wispr-flow-alternative/'] + [c['path'] for c in COMPARISONS]


def text(value):
    # The existing guide data uses only bold inline markup.
    return escape(value).replace('&lt;b&gt;', '<b>').replace('&lt;/b&gt;', '</b>')


def blocks(items):
    out = []
    for block in items:
        kind, *v = block
        if kind in ('p', 'h2'):
            out.append(f'<{kind}>{text(v[0])}</{kind}>')
        elif kind == 'note':
            out.append(f'<p class="note">{text(v[0])}</p>')
        elif kind in ('ul', 'ol'):
            out.append(f'<{kind}>' + ''.join(f'<li>{text(s)}</li>' for s in v[0]) + f'</{kind}>')
        elif kind == 'img':
            out.append(f'<figure><img src="{escape(v[0], quote=True)}" alt="{escape(v[1], quote=True)}" loading="lazy" width="1280" height="820"></figure>')
        elif kind == 'table':
            out.append('<div class="tbl"><table class="bill"><thead><tr>' + ''.join(f'<th scope="col">{text(s)}</th>' for s in v[0]) + '</tr></thead><tbody>' + ''.join('<tr>' + ''.join(f'<td>{text(s)}</td>' for s in row) + '</tr>' for row in v[1]) + '</tbody></table></div>')
        elif kind == 'links':
            out.append('<ul>' + ''.join(f'<li><a href="{escape(url, quote=True)}">{escape(label)}</a></li>' for url, label in v[0]) + '</ul>')
        else:
            raise ValueError(f'Unknown guide block: {kind}')
    return '\n'.join(out)


def faq_html(faq, lang):
    """Visible questions and answers. The same list feeds the FAQPage data, so the
    two cannot drift apart. This does not promise a rich result."""
    if not faq:
        return ''
    head = 'Συχνές ερωτήσεις' if lang == 'el' else 'Frequently asked questions'
    return f'<h2 id="faq">{head}</h2>' + ''.join(f'<h3>{escape(q)}</h3><p>{escape(a)}</p>' for q, a in faq)


def page(path, lang, title, desc, lead, body, paired=True, article=False, faq=None):
    prefix = '/el' if lang == 'el' else ''
    home = prefix + '/'
    url = BASE + prefix + path
    script, css = head_bits()
    if lang == 'el':
        script = script.replace('?"Dark":"Light"', '?"Σκοτεινό":"Φωτεινό"')
    css = css.replace('</style>', '''
.content{max-width:900px;margin:0 auto;padding:38px 24px 64px}
.content h1{max-width:25ch;margin:0 0 20px;font-size:clamp(30px,5vw,48px)}
.content h2{margin:36px 0 14px;font-size:clamp(25px,3vw,32px)}
.content h3{margin:24px 0 10px}.content p{margin:14px 0;max-width:75ch}
.content ul,.content ol{padding-left:25px;margin:16px 0}.content li{margin:10px 0}
.content figure{margin:24px 0}.content img{max-width:100%;height:auto}
.content .lead{margin:0 0 22px}.content .cta{justify-content:flex-start;margin:24px 0}
.content .byline,.content .crumbs{font-size:15px;color:var(--muted)}
.content .note{padding:16px;border-left:3px solid var(--coral)}
.content a:not(.btn){text-underline-offset:3px}.content .tbl{margin:24px 0}
.content td,.content th{min-width:140px}.content .guide-list{list-style:none;padding:0}
.content .guide-list li{padding:18px 0;border-top:1px solid var(--line)}
</style>''')
    other_lang = 'en' if lang == 'el' else 'el'
    other_path = path if lang == 'el' else '/el' + path
    alternatives = ''
    if paired:
        alternatives = '\n'.join(f'<link rel="alternate" hreflang="{l}" href="{BASE}{p}">' for l, p in [('en', path), ('el', '/el' + path), ('x-default', path)])
    else:
        other_path = '/el/'
    words = ('Αρχική', 'Οδηγοί', 'Ιδιωτικότητα', 'Κατεβάστε το για Windows', 'Στο περιεχόμενο', 'Ενημερώθηκε', 'Περιήγηση') if lang == 'el' else ('Home', 'Guides', 'Privacy', 'Download for Windows', 'Skip to content', 'Updated', 'Navigation')
    schema = {'@context': 'https://schema.org', '@graph': [
        {'@type': 'Article' if article else 'WebPage', '@id': url + '#page', 'url': url,
         'name': title, 'headline': title, 'description': desc, 'inLanguage': lang,
         'dateModified': CONTENT_DATE, 'image': OG_IMAGE,
         'author': {'@type': 'Organization', 'name': 'Luram AI Agency', 'url': 'https://luram.gr/'},
         'about': {'@type': 'SoftwareApplication', '@id': BASE + '/#app', 'name': 'Fuck You Flow'}},
        {'@type': 'BreadcrumbList', 'itemListElement': [
            {'@type': 'ListItem', 'position': 1, 'name': words[0], 'item': BASE + home},
            *([{'@type': 'ListItem', 'position': 2, 'name': words[1], 'item': BASE + prefix + '/guides/'}] if article else []),
            {'@type': 'ListItem', 'position': 3 if article else 2, 'name': title, 'item': url}]}]}
    if faq:
        schema['@graph'].append({'@type': 'FAQPage', '@id': url + '#faq', 'inLanguage': lang,
                                 'mainEntity': [{'@type': 'Question', 'name': q,
                                                 'acceptedAnswer': {'@type': 'Answer', 'text': a}} for q, a in faq]})
    body = body + faq_html(faq, lang)
    breadcrumbs = f'<a href="{home}">{words[0]}</a> / ' + (f'<a href="{prefix}/guides/">{words[1]}</a> / ' if article else '') + escape(title)
    return f'''<!doctype html>
<html lang="{lang}" data-theme="dark"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{escape(title)} | FU Flow</title><meta name="description" content="{escape(desc, quote=True)}">
<link rel="canonical" href="{url}">{alternatives}
<link rel="icon" href="/assets/favicon.ico" sizes="48x48"><link rel="apple-touch-icon" href="/assets/icon-180.png">
<meta name="theme-color" content="#000000"><meta property="og:type" content="{'article' if article else 'website'}">
<meta property="og:site_name" content="Fuck You Flow"><meta property="og:url" content="{url}">
<meta property="og:locale" content="{'el_GR' if lang == 'el' else 'en_US'}">
<meta property="og:title" content="{escape(title, quote=True)} | FU Flow"><meta property="og:description" content="{escape(desc, quote=True)}">
<meta property="og:image" content="{OG_IMAGE}"><meta property="og:image:width" content="1200"><meta property="og:image:height" content="630">
<meta property="og:image:alt" content="FU Flow"><meta name="twitter:card" content="summary_large_image">
<script type="application/ld+json">{json.dumps(schema, ensure_ascii=False).replace('</', '<\\/')}</script>{script}{css}
</head><body><a class="skip" href="#main">{words[4]}</a>
<header class="top"><div class="wrap top-in"><a class="brand" href="{home}"><img src="/assets/icon-64.png" alt="" width="32" height="32">Fuck You Flow</a>
{site_nav.nav_html(lang, url.replace(BASE, '') or '/')}
<div class="tools"><a href="{other_path}" lang="{other_lang}">{other_lang.upper()}</a><button id="theme-toggle" type="button">{'Φωτεινό' if lang == 'el' else 'Light'}</button></div></div></header>
<main id="main" class="content"><p class="crumbs">{breadcrumbs}</p><h1>{escape(title)}</h1><p class="lead">{escape(lead)}</p>
<p class="byline">Luram AI Agency · {words[5]} <time datetime="{CONTENT_DATE}">{CONTENT_DATE}</time></p>
{body}<div class="cta"><a class="btn btn-acid" href="{DL}">{words[3]}</a></div><p class="fine">FU Flow {VERSION} beta · {SIZE_EL if lang == 'el' else SIZE} · Windows 10 / 11 · MIT</p></main>
<footer class="foot"><div class="wrap"><nav class="foot-links" aria-label="{words[6]}"><a href="{home}">{words[0]}</a><a href="{prefix}/guides/">{words[1]}</a><a href="{prefix}/privacy/">{words[2]}</a><a href="{prefix}/wispr-flow-alternative/">FU Flow vs Wispr Flow</a><a href="{REPO}">GitHub</a></nav><p>Luram AI Agency · {'Θεσσαλονίκη' if lang == 'el' else 'Thessaloniki'} · <a href="https://luram.gr/">luram.gr</a></p></div></footer></body></html>'''


WISPR_FAQ_EN = [
    ('Is there a free Wispr Flow alternative for Windows?',
     'Yes. FU Flow is a free, open-source dictation app for Windows 10 and 11 under MIT. Its local engine has no subscription, no account and no weekly word limit.'),
    ('How many words does Wispr Flow give for free?',
     'Wispr Flow lists 2,000 words a week on desktop, 1,000 on iPhone and unlimited dictation on Android in its free plan, checked on 8 September 2026. Pro is US$15 a month, or US$12 a month billed annually.'),
    ('Is there an offline Wispr Flow alternative?',
     'Yes. Wispr states that its transcription happens in the cloud. FU Flow runs Whisper models on your PC by default, so voice to text keeps working without internet after installation.'),
    ('Is FU Flow free forever?',
     'The current app is free, has no paid local tier and is released under MIT. That license lets you keep and modify the released source. Third-party cloud providers, if you choose one, set their own fees.'),
    ('Is this the same as OpenAI Whisper?',
     "Whisper is the open speech-recognition model used by FU Flow. Wispr Flow is a separate product and company. FU Flow is independent of Wispr AI and does not claim to use Wispr's code, models or infrastructure."),
    ('Which languages does FU Flow dictate?',
     'You choose your own language from thirty-three in Settings, and the default mode accepts English words inside your sentences. Greek and English are the most tested.'),
    ('Can I use FU Flow and Wispr Flow side by side?',
     'Yes. Give each app a different dictation key in its settings, then compare them in the programs you use every day before changing your subscription.'),
]


def comparison():
    body = f'<div class="cta"><a class="btn btn-acid" href="{DL}">Download free for Windows</a><a class="btn btn-coral" href="{REPO}">Read the source</a></div><p class="fine">Version {VERSION} beta · {SIZE} · Models included</p>'
    body += '''<h2>The short answer</h2>
<p>FU Flow is a free Wispr Flow alternative for Windows 10 and 11 and Ubuntu Linux 22.04 or newer. It types what you say into any application with a Whisper speech model running on your own PC, with no weekly word limit, no account and no subscription. A cleanup step, on by default, removes hesitation sounds such as “um” and “uh”, applies spoken self-corrections and fixes punctuation before the text lands. The installer carries the speech models, and the app is open source under the MIT licence, with the code on GitHub.</p>
<p>FU Flow is an independent beta for Windows and Ubuntu built by Luram AI Agency. The comparison below covers dictation, and the two products differ in scope. Product details were checked on 8 September 2026.</p>
<h2>FU Flow compared with Wispr Flow</h2>
<div class="tbl"><table class="bill"><thead><tr><th scope="col">Dictation feature</th><th scope="col">Wispr Flow</th><th scope="col">FU Flow</th></tr></thead><tbody>
<tr><td>Windows price</td><td>Free desktop plan; paid Pro for unlimited use</td><td>Free local dictation, MIT licensed</td></tr>
<tr><td>Weekly words on Windows</td><td>2,000 free; unlimited on Pro</td><td>No word cap</td></tr>
<tr><td>Speech processing</td><td>Cloud processing</td><td>On your PC by default</td></tr>
<tr><td>Offline dictation</td><td>Internet required for cloud processing</td><td>Works with the bundled local models</td></tr>
<tr><td>Platforms</td><td>Windows, macOS, iPhone and Android</td><td>Windows 10 and 11, Ubuntu Linux 22.04 or newer</td></tr>
<tr><td>Languages</td><td>100+ languages</td><td>99 languages, Greek and English the most tested</td></tr>
<tr><td>Account for local dictation</td><td>Wispr account</td><td>No account or API key</td></tr>
<tr><td>Product scope</td><td>Dictation, team features and a Mac notetaker</td><td>Dictation, local history, dictionary and snippets</td></tr>
</tbody></table></div>
<h2>What the free plans cost you</h2>
<p>Wispr Flow has a free plan: 2,000 words a week on desktop, 1,000 on iPhone and unlimited dictation on Android. Pro offers unlimited dictation at US$15 per month, or US$12 per month billed annually. Regional prices and offers can differ. <a href="https://wisprflow.ai/pricing">Check current Wispr Flow pricing</a>.</p>
<p>FU Flow costs nothing on Windows, counts no words and asks for no card. A week of heavy dictation passes the 2,000 word mark in a couple of days, which is the point where a free tier stops being free.</p>
<h2>What offline and private mean here</h2>
<p>FU Flow uses a local Whisper engine by default. The installed models can transcribe without an internet connection. The app also checks online for updates. An optional remote speech provider sends audio to the configured provider and may charge for usage. The destination app receives any text you insert into it. <a href="/privacy/">Read the exact data flow</a>.</p>
<p>Wispr states that transcription happens in the cloud and offers data and retention controls. Cloud processing does not by itself mean audio is sold or retained indefinitely. <a href="https://wisprflow.ai/privacy">Read Wispr Flow's privacy information</a>.</p>
<h2>Speed depends on your computer</h2>
<p>Our homepage reports four RTX 3070 observations with waits of 0.17 to 0.88 seconds after finishing dictation. These are our own measurements on our own machine. An independent benchmark or a head-to-head Wispr Flow run would be a separate exercise. Models, speech, drivers and available GPU memory affect results. <a href="/guides/card-memory/">See model and memory troubleshooting</a>.</p>
<h2>Who should consider switching?</h2>
<p>FU Flow fits Windows and Ubuntu users who want local Greek or English dictation without a subscription or weekly cap and are comfortable testing a beta. Wispr Flow remains an option for people needing its phone apps, broader language range, team administration or Mac meeting notes. FU Flow has a CPU fallback, but performance on a low-power PC may be slower.</p>
<h2>How to try FU Flow alongside Wispr Flow</h2>
<ol><li>Download the current Windows installer below. The speech models are included. Check the release notes and published checksum.</li><li>Open FU Flow, select your microphone and keep the local speech engine. Set the language you use most.</li><li>Put the cursor in a text editor. Press right Alt, speak a short sentence, then press right Alt again.</li><li>Review the result, test your everyday apps and add difficult names to the dictionary. If both dictation apps use the same hotkey, change one of them in Settings.</li></ol>
<p>You can test before changing your existing subscription. There is no automatic account migration or import of a Wispr dictionary. <a href="/guides/the-key/">Hotkey guide</a> · <a href="/guides/words-it-gets-wrong/">Dictionary guide</a>.</p>
'''
    body += '<h2>Compare other local dictation options</h2><p>OpenWhispr and Handy also offer free local speech recognition. Compare the setup, platforms and cloud options in our <a href="/free-offline-dictation-alternatives/">free and offline dictation alternatives guide</a>, or read the detailed <a href="/openwhispr-alternative/">FU Flow vs OpenWhispr comparison</a>.</p>'
    return page('/wispr-flow-alternative/', 'en', 'Free Wispr Flow Alternative for Windows',
                'FU Flow is a free, open-source Wispr Flow alternative for Windows and Ubuntu: offline Whisper dictation into any app, with no word limit or subscription.',
                'Say goodbye to the dictation subscription. Try FU Flow for free local voice typing in Greek and English, with no weekly word limit.', body,
                faq=WISPR_FAQ_EN)


def build():
    target = HERE / 'wispr-flow-alternative/index.html'
    target.write_text(comparison(), encoding='utf-8', newline='\n')
    greek_target = HERE / 'el/wispr-flow-alternative/index.html'
    greek_target.parent.mkdir(parents=True, exist_ok=True)
    greek_target.write_text(page('/wispr-flow-alternative/', 'el', WISPR_EL['title'], WISPR_EL['description'],
                                  WISPR_EL['lead'], WISPR_EL['body'], faq=WISPR_EL.get('faq')), encoding='utf-8', newline='\n')
    for data in COMPARISONS:
        dest = HERE / data['path'].lstrip('/') / 'index.html'
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(page(data['path'], data['lang'], data['title'], data['description'],
                             data['lead'], data['body'], paired=False), encoding='utf-8', newline='\n')
    for lang in ('en', 'el'):
        prefix = '/el' if lang == 'el' else ''
        for guide in ALL_GUIDES:
            data = guide[lang]
            path = f'/guides/{guide["slug"]}/'
            description = data.get('description', data['lead'])
            related = ('Περισσότεροι οδηγοί' if lang == 'el' else 'More dictation guides')
            body = blocks(data['blocks']) + f'<h2>{related}</h2><ul>' + ''.join(f'<li><a href="{prefix}/guides/{g["slug"]}/">{escape(g[lang]["title"])}</a></li>' for g in ALL_GUIDES if g['slug'] != guide['slug']) + '</ul>'
            dest = HERE / (prefix + path).lstrip('/') / 'index.html'
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_text(page(path, lang, data['title'], description, data['lead'], body, article=True, faq=data.get('faq')), encoding='utf-8', newline='\n')
        title = 'Οδηγοί φωνητικής πληκτρολόγησης για Windows' if lang == 'el' else 'Voice to Text and Dictation Guides for Windows'
        desc = 'Φωνητική πληκτρολόγηση στον υπολογιστή, υπαγόρευση κειμένου στο Word, ελληνικά και αγγλικά, εντολές για AI και λύσεις για ταχύτητα, λεξικό και επικόλληση.' if lang == 'el' else 'Voice to text on a Windows PC: talk to type in Word and AI prompts, Greek and English, hotkeys, dictionary corrections and slow offline dictation.'
        body = '<ul class="guide-list">' + ''.join(f'<li><h2><a href="{prefix}/guides/{g["slug"]}/">{escape(g[lang]["title"])}</a></h2><p>{escape(g[lang]["lead"])}</p></li>' for g in ALL_GUIDES) + '</ul>'
        (HERE / (prefix + '/guides/index.html').lstrip('/')).write_text(page('/guides/', lang, title, desc, desc, body), encoding='utf-8', newline='\n')
        data = PRIVACY[lang]
        dest = HERE / (prefix + '/privacy/index.html').lstrip('/')
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(page('/privacy/', lang, data['title'], data['description'], data['lead'], blocks(data['blocks'])), encoding='utf-8', newline='\n')
    print(f'Generated {len(paired_paths()) * 2} paired content pages and {len(COMPARISONS)} additional English comparisons')


if __name__ == '__main__':
    build()
