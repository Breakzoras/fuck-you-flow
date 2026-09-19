# -*- coding: utf-8 -*-
"""The one header menu, shared by every page of the site.

Before this existed there were three menus. The home page carried seven links
to its own sections, every guide and comparison carried Home / Guides /
Privacy, and the changelog carried Home / Download / GitHub. Clicking a link
therefore changed the whole menu under the reader, the guides could not be
reached from the home page at all, and the changelog could not be reached from
anywhere except the body text. Measured on 2026-09-10.

Below 1240 pixels the menu used to disappear with nothing in its place, so a
phone, a tablet and a 1366 wide laptop had no navigation whatsoever. The button
here fills that gap.

The section links point at the home page rather than at a bare fragment, so the
same menu works from every page. Greek labels are the wide ones: this set needs
63 characters against the 74 the old home menu used, so it fits where the old
one already fitted.
"""

# label, href without the language prefix
ITEMS_EN = [
    ("Features", "/#features"),
    ("Why free", "/#why"),
    ("Speed", "/#speed"),
    ("Guides", "/guides/"),
    ("vs Wispr Flow", "/wispr-flow-alternative/"),
    ("Changelog", "/changelog/"),
    ("Questions", "/#faq"),
]

ITEMS_EL = [
    ("Τι κάνει", "/el/#features"),
    ("Γιατί δωρεάν", "/el/#why"),
    ("Ταχύτητα", "/el/#speed"),
    ("Οδηγοί", "/el/guides/"),
    ("vs Wispr Flow", "/el/wispr-flow-alternative/"),
    ("Αλλαγές", "/el/changelog/"),
    ("Ερωτήσεις", "/el/#faq"),
]

LABEL = {"el": "Μενού", "en": "Menu"}
ARIA = {"el": "Περιήγηση", "en": "Navigation"}


def items(lang):
    return ITEMS_EL if lang == "el" else ITEMS_EN


def nav_html(lang, current=None):
    """The button and the list, ready to drop into the header.

    `current` is a path such as "/el/guides/". The matching link is marked so a
    reader knows where they are.
    """
    out = [
        '<button class="nav-btn" id="nav-btn" type="button" aria-expanded="false" '
        f'aria-controls="site-nav">{LABEL.get(lang, LABEL["en"])}</button>',
        f'<nav class="nav" id="site-nav" aria-label="{ARIA.get(lang, ARIA["en"])}">',
    ]
    for label, href in items(lang):
        here = ' aria-current="page"' if current and href == current else ""
        out.append(f'<a href="{href}"{here}>{label}</a>')
    out.append("</nav>")
    return "".join(out)


CSS = """
.nav-btn{display:none}
.nav a[aria-current="page"]{color:var(--fg)}
@media (max-width:1240px){
.top{position:sticky}
.top-in{position:relative}
.nav-btn{display:inline-block;font:inherit;font-size:16px;line-height:1;color:var(--fg);
background:none;border:1px solid var(--line);padding:9px 12px;cursor:pointer}
.nav-btn:hover{border-color:var(--coral);color:var(--coral-text)}
.nav{display:none}
.nav.open{display:flex;flex-direction:column;gap:18px;position:absolute;left:0;right:0;top:calc(100% + 10px);
background:var(--bg);border:1px solid var(--line);padding:18px 20px;z-index:60}
.nav.open a{font-size:17px}
}
"""

JS = (
    # This script is placed in the head on the guides, the comparisons and the
    # changelog, where it would run before the button exists, and at the end of
    # the body on the home page. Waiting for the document covers both. Measured
    # on 2026-09-10: without the wait the button opened nothing on 24 of the 26
    # pages, and only the two home pages worked.
    "(function(){function go(){"
    "var b=document.getElementById('nav-btn'),n=document.getElementById('site-nav');"
    "if(!b||!n)return;"
    "b.addEventListener('click',function(){"
    "var o=b.getAttribute('aria-expanded')==='true';"
    "b.setAttribute('aria-expanded',String(!o));n.classList.toggle('open',!o);});"
    "n.addEventListener('click',function(e){if(e.target.tagName==='A'){"
    "b.setAttribute('aria-expanded','false');n.classList.remove('open');}});}"
    "if(document.readyState==='loading'){"
    "document.addEventListener('DOMContentLoaded',go);}else{go();}})();"
)
