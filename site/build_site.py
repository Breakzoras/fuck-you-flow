# -*- coding: utf-8 -*-
"""Build the two language pages of fuckyouflow.app from the chosen design.

Source of truth for layout and CSS:  site/variants/v4-coral.html
Output:
    site/index.html      English, the root, the page search engines get
    site/el/index.html   Greek, same page, same design

Nothing here touches the live server. Run from anywhere:
    "C:/Python314/python.exe" "C:/Claude Projects/lalia/site/build_site.py"
"""
import datetime
import io
import os
import re
import sys

import llms_source
import site_nav
from site_data import BASE, VERSION, DL, SIZE, SIZE_BYTES, RELEASE_DATE, CONTENT_DATE, release_tokens

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, "variants", "v4-coral.html")
OUT_EN = os.path.join(HERE, "index.html")
OUT_EL = os.path.join(HERE, "el", "index.html")

OUT_LLMS = os.path.join(HERE, "llms.txt")
OUT_LLMS_FULL = os.path.join(HERE, "llms-full.txt")
OUT_SITEMAP = os.path.join(HERE, "sitemap.xml")

# ---------------------------------------------------------------- head blocks

COMMON_LINKS = """<link rel="icon" href="/assets/favicon.ico" sizes="48x48">
<link rel="icon" href="/assets/icon-64.png" type="image/png" sizes="64x64">
<link rel="apple-touch-icon" href="/assets/icon-180.png">
<link rel="alternate" hreflang="en" href="{base}/">
<link rel="alternate" hreflang="el" href="{base}/el/">
<link rel="alternate" hreflang="x-default" href="{base}/">
<meta name="theme-color" content="#000000">
<meta name="msvalidate.01" content="A79AFD627767F90B3097E8DF8FE5CAD5">
<meta property="og:type" content="website">
<meta property="og:site_name" content="Fuck You Flow">
<meta property="og:image" content="{base}/assets/og.png">
<meta property="og:image:width" content="1200">
<meta property="og:image:height" content="630">
<meta property="og:image:alt" content="A matte black sculptural hand lit with acid green, beside the words Fuck You Flow, free dictation for Windows.">
<meta name="twitter:card" content="summary_large_image">""".format(base=BASE)

HEAD_EN = """<title>Free Offline Dictation for Windows | FU Flow</title>
<meta name="description" content="Free, open-source voice typing for Windows 10 and 11. Dictate in Greek and English with local Whisper, no subscription and no word limit. Download FU Flow.">
<link rel="canonical" href="{base}/">
{common}
<meta property="og:title" content="Fuck You Flow: free dictation for Windows">
<meta property="og:description" content="Free Windows dictation in Greek and English. Local speech processing by default, with no subscription, account or word limit for the local engine.">
<meta property="og:url" content="{base}/">
<meta property="og:locale" content="en_US">
<meta property="og:locale:alternate" content="el_GR">""".format(base=BASE, common=COMMON_LINKS)

HEAD_EL = """<title>Δωρεάν υπαγόρευση στα ελληνικά για Windows | FU Flow</title>
<meta name="description" content="Γράψτε με τη φωνή σας στα ελληνικά και αγγλικά σε Windows 10 και 11. Τοπική αναγνώριση Whisper, χωρίς συνδρομή ή όριο λέξεων. Δωρεάν FU Flow από τη Luram.">
<link rel="canonical" href="{base}/el/">
{common}
<meta property="og:title" content="Fuck You Flow: δωρεάν υπαγόρευση για Windows">
<meta property="og:description" content="Δωρεάν υπαγόρευση στα ελληνικά και αγγλικά για Windows. Τοπική αναγνώριση ομιλίας από προεπιλογή, χωρίς συνδρομή, λογαριασμό ή όριο λέξεων για την τοπική μηχανή.">
<meta property="og:url" content="{base}/el/">
<meta property="og:locale" content="el_GR">
<meta property="og:locale:alternate" content="en_US">""".format(base=BASE, common=COMMON_LINKS)

# ---------------------------------------------------------------- structured data

def jsonld(lang):
    url = BASE + ("/el/" if lang == "el" else "/")
    if lang == "el":
        desc = ("Δωρεάν υπαγόρευση για Windows με ελληνικά και αγγλικά. Η αναγνώριση ομιλίας "
                "γίνεται τοπικά από προεπιλογή. Εναλλακτική του Wispr Flow χωρίς συνδρομή "
                "ή όριο λέξεων για την τοπική μηχανή.")
        faq = FAQ_EL
        pub = "Luram AI Agency"
    else:
        desc = ("Free dictation for Windows in Greek and English. Speech recognition runs locally "
                "by default. A Wispr Flow alternative with no subscription or word limit for the local engine.")
        faq = FAQ_EN
        pub = "Luram AI Agency"
    q = []
    for question, answer in faq:
        q.append('{"@type":"Question","name":%s,"acceptedAnswer":{"@type":"Answer","text":%s}}'
                 % (jstr(question), jstr(answer)))
    howto_name, howto_steps = HOWTO[lang]
    steps = ",".join(
        '{"@type":"HowToStep","position":%d,"name":%s,"text":%s}' % (i + 1, jstr(n), jstr(t))
        for i, (n, t) in enumerate(howto_steps))
    return """<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@graph": [
    {
      "@type": "SoftwareApplication",
      "@id": "%(base)s/#app",
      "name": "Fuck You Flow",
      "alternateName": "FU Flow",
      "applicationCategory": "UtilitiesApplication",
      "applicationSubCategory": "Speech to text",
      "operatingSystem": "Windows 10, Windows 11",
      "softwareVersion": "%(ver)s",
      "inLanguage": ["en", "el"],
      "description": %(desc)s,
      "url": "%(url)s",
      "downloadUrl": "%(dl)s",
      "installUrl": "https://github.com/Breakzoras/fuck-you-flow/releases",
      "fileSize": "%(bytes)s B",
      "image": "%(base)s/assets/og.png",
      "license": "https://opensource.org/licenses/MIT",
      "isAccessibleForFree": true,
      "offers": {"@type": "Offer", "price": "0", "priceCurrency": "EUR",
                 "availability": "https://schema.org/InStock"},
      "softwareRequirements": "Windows 10 or 11, 64-bit. A graphics card from NVIDIA, AMD or Intel makes it faster; without one it runs on the processor.",
      "featureList": [
        "Hotkey dictation into compatible Windows text fields, using the right Alt key",
        "Local speech recognition by default, without an account",
        "English and Greek, mixed inside a single sentence",
        "No word limit or subscription for local dictation",
        "A personal dictionary that learns from corrections",
        "Snippets: one spoken phrase expands into a block of text",
        "Local history and statistics, kept in SQLite on the user's disk",
        "Local speech recognition works offline after installation",
        "Password fields are detected and left empty",
        "Open source under the MIT license"
      ],
      "keywords": "free dictation Windows, Wispr Flow alternative, offline speech to text, local Whisper, Greek dictation, open source voice typing",
      "datePublished": "2026-09-06",
      "dateModified": "%(today)s",
      "author": {"@id": "https://luram.gr/#org"},
      "publisher": {"@id": "https://luram.gr/#org"},
      "maintainer": {"@id": "https://luram.gr/#org"},
      "isBasedOn": {
        "@type": "SoftwareApplication",
        "name": "Whisper",
        "author": {"@type": "Organization", "name": "OpenAI"},
        "license": "https://opensource.org/licenses/MIT",
        "url": "https://github.com/openai/whisper"
      }
    },
    {
      "@type": "Organization",
      "@id": "https://luram.gr/#org",
      "name": "%(pub)s",
      "url": "https://luram.gr/",
      "email": "info@luram.gr",
      "description": "AI tools, agents and automation for Greek businesses.",
      "address": {"@type": "PostalAddress", "addressLocality": "Thessaloniki",
                  "addressCountry": "GR"},
      "sameAs": ["https://github.com/Breakzoras"]
    },
    {
      "@type": "HowTo",
      "@id": "%(url)s#howto",
      "inLanguage": "%(lang)s",
      "name": %(howto_name)s,
      "totalTime": "PT10S",
      "estimatedCost": {"@type": "MonetaryAmount", "currency": "EUR", "value": "0"},
      "step": [%(steps)s]
    },
    {
      "@type": "FAQPage",
      "@id": "%(url)s#faq",
      "inLanguage": "%(lang)s",
      "mainEntity": [%(q)s]
    },
    {
      "@type": "WebSite",
      "@id": "%(base)s/#site",
      "url": "%(base)s/",
      "name": "Fuck You Flow",
      "inLanguage": "%(lang)s"
    }
  ]
}
</script>""" % {"base": BASE, "ver": VERSION, "desc": jstr(desc), "url": url, "dl": DL,
                "pub": pub, "lang": lang, "q": ",".join(q),
                "today": RELEASE_DATE, "bytes": SIZE_BYTES,
                "howto_name": jstr(howto_name), "steps": steps}


def jstr(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


HOWTO = {
    "en": ("How to dictate text into compatible Windows apps with Fuck You Flow", [
        ("Press the right Alt key",
         "A small window with a waveform appears and the app starts listening."),
        ("Speak",
         "English, Greek, or both in the same sentence. Each finished phrase is transcribed "
         "while you keep talking."),
        ("Press the right Alt key again",
         "The text lands where your cursor was, with punctuation. Escape cancels instead."),
    ]),
    "el": ("Πώς να υπαγορεύσετε κείμενο σε συμβατές εφαρμογές Windows με το Fuck You Flow", [
        ("Πατήστε το δεξί Alt",
         "Εμφανίζεται ένα μικρό παράθυρο με κυματομορφή και η εφαρμογή αρχίζει να ακούει."),
        ("Μιλήστε",
         "Ελληνικά, αγγλικά, ή και τα δύο στην ίδια πρόταση. Κάθε φράση που τελειώνει "
         "μεταγράφεται όσο εσείς συνεχίζετε."),
        ("Ξαναπατήστε το δεξί Alt",
         "Το κείμενο προσγειώνεται εκεί που ήταν ο κέρσορας, με τα σημεία στίξης. Το Escape "
         "ακυρώνει."),
    ]),
}


FAQ_EN = [('Is it a free alternative to Wispr Flow?',
  'Yes, for local dictation on Windows 10 and 11. FU Flow is free under MIT, with no subscription, account '
  'or weekly word cap for the local engine. It is a beta and does not offer every Wispr Flow feature.'),
 ('Where does my voice go?',
  'The default Whisper engine processes speech on your PC and works offline after setup. The app checks '
  'online for updates. If you choose an OpenAI-compatible remote speech provider in Settings, audio goes to '
  'that provider and its fees and privacy terms apply.'),
 ('Does it need an NVIDIA card?',
  'No. Supported NVIDIA, AMD and Intel cards can use Vulkan. The CPU fallback also works. Speed depends on '
  'the model, hardware, drivers and available memory.'),
 ('Why does Windows show a warning during install?',
  'The beta installer has no purchased code-signing certificate. Download only from the linked project '
  'release and compare the published SHA256 checksum. Source code is available for inspection.'),
 ('What does beta mean?',
  'Some applications may refuse automatic paste and some graphics cards remain untested. Check your first '
  'dictation in a plain text editor. You can copy transcripts from History if insertion fails.'),
 ('Why the name FU Flow?',
  'The name expresses our preference for free local dictation without a recurring software subscription. FU '
  'Flow is the short name used on social media. The project is independent of Wispr Flow.')]

FAQ_EL = [('Είναι δωρεάν εναλλακτική του Wispr Flow;',
  'Ναι, για τοπική υπαγόρευση σε Windows 10 και 11. Το FU Flow είναι δωρεάν με άδεια MIT, χωρίς συνδρομή, '
  'λογαριασμό ή εβδομαδιαίο όριο λέξεων για την τοπική μηχανή. Είναι beta και δεν προσφέρει όλα τα '
  'χαρακτηριστικά του Wispr Flow.'),
 ('Πού πάει η φωνή μου;',
  'Η προεπιλεγμένη μηχανή Whisper επεξεργάζεται την ομιλία στον υπολογιστή σας και λειτουργεί χωρίς σύνδεση '
  'μετά την εγκατάσταση. Η εφαρμογή ελέγχει online για ενημερώσεις. Αν επιλέξετε απομακρυσμένο πάροχο '
  'συμβατό με OpenAI στις Ρυθμίσεις, ο ήχος αποστέλλεται εκεί και ισχύουν οι χρεώσεις και οι όροι '
  'ιδιωτικότητάς του.'),
 ('Χρειάζεται κάρτα NVIDIA;',
  'Όχι. Υποστηριζόμενες κάρτες NVIDIA, AMD και Intel μπορούν να χρησιμοποιήσουν Vulkan. Υπάρχει και '
  'λειτουργία στον επεξεργαστή. Η ταχύτητα εξαρτάται από το μοντέλο, τον υπολογιστή, τους οδηγούς και τη '
  'διαθέσιμη μνήμη.'),
 ('Γιατί εμφανίζεται προειδοποίηση των Windows στην εγκατάσταση;',
  'Ο εγκαταστάτης beta δεν έχει αγορασμένο πιστοποιητικό υπογραφής κώδικα. Κατεβάστε τον μόνο από τη '
  'συνδεδεμένη σελίδα έκδοσης του έργου και συγκρίνετε το δημοσιευμένο άθροισμα SHA256. Ο πηγαίος κώδικας '
  'είναι διαθέσιμος για έλεγχο.'),
 ('Τι σημαίνει beta;',
  'Κάποιες εφαρμογές μπορεί να απορρίψουν την αυτόματη επικόλληση και κάποιες κάρτες παραμένουν αδοκίμαστες. '
  'Ελέγξτε την πρώτη υπαγόρευση σε έναν απλό επεξεργαστή κειμένου. Μπορείτε να αντιγράψετε το κείμενο από το '
  'Ιστορικό αν αποτύχει η εισαγωγή.'),
 ('Γιατί το όνομα FU Flow;',
  'Το όνομα εκφράζει την προτίμησή μας για δωρεάν τοπική υπαγόρευση χωρίς μηνιαία συνδρομή λογισμικού. Το FU '
  'Flow είναι η σύντομη ονομασία στα κοινωνικά δίκτυα. Το έργο είναι ανεξάρτητο από το Wispr Flow.')]

# ---------------------------------------------------------------- translations
# Ordered. Longer strings first where one contains another.
TR = [('Skip to content', 'Στο περιεχόμενο'),
 ('>Feature<', '>Χαρακτηριστικό<'),
 ('Windows 10 and 11', 'Windows 10 και 11'),
 ('Free offline dictation for Windows.', 'Δωρεάν υπαγόρευση στα ελληνικά για Windows.'),
 ('Speak instead of typing. FU Flow turns Greek and English speech into text in Windows apps using local '
  'Whisper. No subscription, no weekly word cap and no account for local dictation. Built by Luram AI Agency '
  'in Thessaloniki.',
  'Γράψτε με τη φωνή σας. Το FU Flow μετατρέπει ελληνική και αγγλική ομιλία σε κείμενο στις εφαρμογές των '
  'Windows με τοπικό Whisper. Χωρίς συνδρομή, εβδομαδιαίο όριο λέξεων ή λογαριασμό για τοπική υπαγόρευση. '
  'Από τη Luram AI Agency στη Θεσσαλονίκη.'),
 ('Download for Windows', 'Κατεβάστε το για Windows'),
 ('<small class="btn-v">Version {{VERSION}}</small>', '<small class="btn-v">Έκδοση {{VERSION}}</small>'),
 ('>Read the code</a>', '>Δείτε τον κώδικα</a>'),
 ('Version {{VERSION}} beta. {{SIZE}} with the speech models included. Local dictation works offline after '
  'setup. MIT license. The SHA256 checksum is on the ',
  'Έκδοση {{VERSION}} beta. {{SIZE}} με τα μοντέλα ομιλίας μέσα. Η τοπική υπαγόρευση λειτουργεί χωρίς '
  'ίντερνετ μετά την εγκατάσταση. Άδεια MIT. Το άθροισμα ελέγχου SHA256 βρίσκεται στη '),
 ('. What changed in each version is on the <a href="/changelog/">changelog</a>.',
  '. Τι άλλαξε σε κάθε έκδοση βρίσκεται στο <a href="/el/changelog/">ιστορικό αλλαγών</a>.'),
 ('>release page</a>', '>σελίδα της έκδοσης</a>'),
 ('The listening window: a coral dot, Listening 4.2 s, and a green waveform.',
  'Το παράθυρο ακρόασης: μια κοραλί τελεία, Ακούει 4,2 δ και μια πράσινη κυματομορφή.'),
 ('Listening 4.2 s', 'Ακούει 4,2 δ'),
 ('Your voice. Your computer. No subscription.', 'Η φωνή σας. Ο υπολογιστής σας. Χωρίς συνδρομή.'),
 ('>Why this is free</h2>', '>Γιατί είναι δωρεάν</h2>'),
 ('The speech model was already open.', 'Το μοντέλο ομιλίας ήταν ήδη ανοιχτό.'),
 ('OpenAI published Whisper on 21 September 2022. Its own repository says it in one line: ',
  'Η OpenAI δημοσίευσε το Whisper στις 21 Σεπτεμβρίου 2022. Το ίδιο της το αποθετήριο το γράφει σε μία '
  'γραμμή: '),
 (' Public weights, public code, local speech recognition on hardware you already own.',
  ' Δημόσια βάρη, δημόσιος κώδικας, τοπική αναγνώριση ομιλίας στον υπολογιστή που ήδη έχετε.'),
 ('We use Whisper through whisper.cpp to offer local voice typing without a recurring fee. FU Flow is an '
  'independent app, not a modified copy of Wispr Flow. We do not claim the two products use the same '
  'recognition system or offer identical features.',
  'Χρησιμοποιούμε το Whisper μέσα από το whisper.cpp για τοπική υπαγόρευση χωρίς μηνιαία χρέωση. Το FU Flow '
  'είναι ανεξάρτητη εφαρμογή. Δεν είναι τροποποιημένο αντίγραφο του Wispr Flow και δεν ισχυριζόμαστε ότι τα '
  'δύο προϊόντα έχουν την ίδια μηχανή αναγνώρισης ή τα ίδια χαρακτηριστικά.'),
 ('We built the hotkey, the local workflow and the Greek text handling around that open engine. Luram funds '
  'the project and releases the code under MIT. Local processing uses hardware you already own.',
  'Πάνω σε αυτή την ανοιχτή μηχανή φτιάξαμε το πλήκτρο, την τοπική ροή και την επεξεργασία ελληνικού '
  'κειμένου. Η Luram χρηματοδοτεί το έργο και διαθέτει τον κώδικα με άδεια MIT. Η τοπική επεξεργασία '
  'χρησιμοποιεί τον υπολογιστή που ήδη έχετε.'),
 ('Take it. Read the code. Fork it. The MIT licence even lets you sell it.',
  'Πάρτε το. Διαβάστε τον κώδικα. Κάντε το δικό σας. Η άδεια MIT σας επιτρέπει ακόμα και να το πουλήσετε.'),
 ('What is inside', 'Τι έχει μέσα'),
 ('Six things the app does on its own, on your machine, without asking anyone.',
  'Έξι πράγματα που κάνει μόνη της η εφαρμογή, στο μηχάνημά σας, χωρίς να ρωτήσει κανέναν.'),
 ('Greek that holds up', 'Ελληνικά που στέκουν'),
 ('Question marks come from grammar rules or from the rise of your voice. Spoken self-corrections are '
  'applied before the text lands.',
  'Τα ερωτηματικά βγαίνουν από κανόνες γραμματικής ή από το ανέβασμα της φωνής σας. Οι προφορικές '
  'αυτοδιορθώσεις εφαρμόζονται πριν προσγειωθεί το κείμενο.'),
 ('>Snippets<', '>Έτοιμα κομμάτια<'),
 ('Say a phrase, get a whole block of text. Signatures, addresses, the replies you send every day.',
  'Πείτε μια φράση, πάρτε ολόκληρο κομμάτι κειμένου. Υπογραφές, διευθύνσεις, οι απαντήσεις που στέλνετε κάθε '
  'μέρα.'),
 ('Local by default', 'Τοπικά από προεπιλογή'),
 ('The default local engine keeps speech on your PC. Update checks use the internet. Choosing a remote '
  'speech provider sends audio to that provider.',
  'Η προεπιλεγμένη τοπική μηχανή κρατά την ομιλία στον υπολογιστή σας. Ο έλεγχος ενημερώσεων χρησιμοποιεί το '
  'ίντερνετ. Αν επιλέξετε απομακρυσμένο πάροχο ομιλίας, ο ήχος αποστέλλεται σε εκείνον.'),
 ('Correct it once. It learns.', 'Διορθώστε το μία φορά. Το μαθαίνει.'),
 ('Every dictation is kept in History, one file on your disk. Fix a word there and the fix becomes a '
  'dictionary rule. The next time you say it, it lands the way you wrote it.',
  'Κάθε υπαγόρευση φυλάγεται στο Ιστορικό, ένα αρχείο στον δίσκο σας. Διορθώστε εκεί μια λέξη και η διόρθωση '
  'γίνεται κανόνας λεξικού. Την επόμενη φορά που θα την πείτε, προσγειώνεται όπως τη γράψατε.'),
 ('>History<', '>Ιστορικό<'),
 ('Your words, your rules.', 'Οι λέξεις σας, οι κανόνες σας.'),
 ('Names, brands, the terms of your trade. Add them to the dictionary and the engine spells them your way, '
  'every time. Rules made in History show up here too, ready to edit.',
  'Ονόματα, μάρκες, οι όροι της δουλειάς σας. Βάλτε τα στο λεξικό και η μηχανή τα γράφει με τον δικό σας '
  'τρόπο, κάθε φορά. Οι κανόνες που φτιάχνονται στο Ιστορικό εμφανίζονται κι εδώ, έτοιμοι για αλλαγή.'),
 ('>Dictionary<', '>Λεξικό<'),
 ('How much you said, how much you saved.', 'Πόσα είπατε, πόσα κερδίσατε.'),
 ('Words per day, median wait, minutes saved against typing. What you said yesterday is one click away. All '
  'of it stays in one file on your disk and goes nowhere else.',
  'Λέξεις τη μέρα, μέση αναμονή, λεπτά που γλιτώσατε από την πληκτρολόγηση. Ό,τι είπατε χθες απέχει ένα '
  'κλικ. Όλα μένουν σε ένα αρχείο στον δίσκο σας και δεν πάνε πουθενά αλλού.'),
 ('>Statistics<', '>Στατιστικά<'),
 ('GPU acceleration, with a CPU fallback.', 'Επιτάχυνση στην κάρτα, εναλλακτικά στον επεξεργαστή.'),
 ('NVIDIA, AMD and Intel all run through Vulkan. The first start reads your machine and picks the model that '
  'fits it. Change the key, the microphone or the model here whenever you like.',
  'NVIDIA, AMD και Intel περνούν όλες από το Vulkan. Το πρώτο ξεκίνημα διαβάζει το μηχάνημά σας και διαλέγει '
  'το μοντέλο που του ταιριάζει. Αλλάξτε εδώ το πλήκτρο, το μικρόφωνο ή το μοντέλο όποτε θέλετε.'),
 ('>Settings<', '>Ρυθμίσεις<'),
 ('>the right one<', '>το δεξί<'),
 ('One key.', 'Ένα πλήκτρο.'),
 ('Press right Alt.</h3><p>A small window with a waveform appears. It listens.',
  'Πατήστε δεξί Alt.</h3><p>Εμφανίζεται ένα μικρό παράθυρο με κυματομορφή. Ακούει.'),
 ('Speak.</h3><p>Greek, English or both. Each finished phrase is transcribed while you keep talking.',
  'Μιλήστε.</h3><p>Ελληνικά, αγγλικά ή και τα δύο. Κάθε φράση που τελειώνει μεταγράφεται όσο εσείς '
  'συνεχίζετε να μιλάτε.'),
 ('Press right Alt again.</h3><p>The text lands where your cursor was. With periods, commas and question '
  'marks.',
  'Πατήστε ξανά δεξί Alt.</h3><p>Το κείμενο προσγειώνεται εκεί που ήταν ο δρομέας σας. Με τελείες, κόμματα '
  'και ερωτηματικά.'),
 ('Dictate into compatible text fields in Word, Chrome, Slack, Outlook and other Windows apps. '
  '<a href="/guides/when-the-paste-refuses/">If automatic paste fails</a>, copy the text from History.',
  'Υπαγορεύστε σε συμβατά πεδία κειμένου στο Word, το Chrome, το Slack, το Outlook και άλλες εφαρμογές '
  'Windows. <a href="/el/guides/when-the-paste-refuses/">Αν αποτύχει η αυτόματη επικόλληση</a>, '
  'αντιγράψτε το κείμενο από το Ιστορικό.'),
 ('>Speed</h2>', '>Ταχύτητα</h2>'),
 ('Four real dictations on an RTX 3070 with the large model. The coral bar is how long the person talked. '
  'The green bar is how long they then waited for the text. Both bars are drawn on the same scale.',
  'Τέσσερις πραγματικές υπαγορεύσεις σε RTX 3070 με το μεγάλο μοντέλο. Η κοραλί μπάρα είναι πόση ώρα μίλησε '
  'ο άνθρωπος. Η πράσινη είναι πόση ώρα περίμενε μετά για το κείμενο. Οι δύο μπάρες είναι στην ίδια '
  'κλίμακα.'),
 ('<span class="m-u">words</span>', '<span class="m-u">λέξεις</span>'),
 ('<span class="m-k">You spoke for</span>', '<span class="m-k">Μιλήσατε</span>'),
 ('<span class="m-k">You waited</span>', '<span class="m-k">Περιμένατε</span>'),
 ('>17.5 s<', '>17,5 δ<'),
 ('>0.59 s<', '>0,59 δ<'),
 ('>82 s<', '>82 δ<'),
 ('>0.17 s<', '>0,17 δ<'),
 ('>77 s<', '>77 δ<'),
 ('>0.53 s<', '>0,53 δ<'),
 ('>152 s<', '>152 δ<'),
 ('>0.88 s<', '>0,88 δ<'),
 ('Finished phrases are transcribed while you speak, reducing the work left at the end. These four '
  'observations are from our RTX 3070, not a comparison with Wispr Flow. Results vary with the model, '
  'recording length, drivers and free GPU memory. AMD, Intel and CPU performance can differ.',
  'Οι ολοκληρωμένες φράσεις μεταγράφονται όσο μιλάτε, μειώνοντας τη δουλειά που μένει στο τέλος. Οι τέσσερις '
  'μετρήσεις έγιναν στη δική μας RTX 3070 και δεν είναι σύγκριση με το Wispr Flow. Ο χρόνος αλλάζει ανάλογα '
  'με το μοντέλο, τη διάρκεια, τους οδηγούς και την ελεύθερη μνήμη της κάρτας. Η απόδοση σε AMD, Intel ή '
  'επεξεργαστή μπορεί να διαφέρει.'),
 ('>The bill</h2>', '>Ο λογαριασμός</h2>'),
 ('<span class="price-tag">Pro: 12 to 15 dollars a month</span>',
  '<span class="price-tag">Pro: 12 ως 15 δολάρια τον μήνα</span>'),
 ('<span class="price-tag">Free local dictation</span>', '<span class="price-tag">Δωρεάν τοπική υπαγόρευση</span>'),
 ('<td>Free desktop plan; paid Pro for unlimited dictation</td>',
  '<td>Δωρεάν πακέτο υπολογιστή, Pro για απεριόριστη υπαγόρευση</td>'),
 ('<td>Current local app: free under MIT</td>', '<td>Τρέχουσα τοπική εφαρμογή: δωρεάν με MIT</td>'),
 ('Desktop dictation pricing checked on 8 September 2026. Wispr Flow has a free desktop plan with 2,000 '
  'words per week. Pro is US$15 monthly or US$12 per month billed annually. Plans and regional prices can '
  'change.',
  'Οι τιμές υπαγόρευσης σε υπολογιστή ελέγχθηκαν στις 8 Σεπτεμβρίου 2026. Το Wispr Flow έχει δωρεάν πακέτο '
  'υπολογιστή με 2.000 λέξεις την εβδομάδα. Το Pro κοστίζει 15 δολάρια τον μήνα ή 12 με ετήσια χρέωση. '
  'Πακέτα και τοπικές τιμές μπορεί να αλλάξουν.'),
 ('<td>Price</td>', '<td>Τιμή</td>'),
 ('<td>Words per week</td>', '<td>Λέξεις την εβδομάδα</td>'),
 ('<td>2,000 on the free desktop plan</td>', '<td>2.000 στο δωρεάν πακέτο υπολογιστή</td>'),
 ('<td>As many as you can say</td>', '<td>Όσες προλαβαίνετε να πείτε</td>'),
 ('<td>Where your voice goes</td>', '<td>Πού πάει η φωνή σας</td>'),
 ('<td>Their servers</td>', '<td>Στους διακομιστές τους</td>'),
 ('<td>On your PC with the local engine</td>', '<td>Στον υπολογιστή σας με την τοπική μηχανή</td>'),
 ('<td>Account</td>', '<td>Λογαριασμός</td>'),
 ('<td>Required</td>', '<td>Απαιτείται</td>'),
 ('<td>None</td>', '<td>Κανένας</td>'),
 ('<td>Internet</td>', '<td>Ίντερνετ</td>'),
 ('<td>Always</td>', '<td>Πάντα</td>'),
 ('<td>Downloads and updates; local dictation works offline</td>',
  '<td>Λήψεις και ενημερώσεις, η τοπική υπαγόρευση λειτουργεί χωρίς σύνδεση</td>'),
 ('<td>Greek</td>', '<td>Ελληνικά</td>'),
 ('<td>One of 100+ languages</td>', '<td>Μία από 100+ γλώσσες</td>'),
 ('<td>First language, mixed with English in the same sentence</td>',
  '<td>Πρώτη γλώσσα, ανακατεμένη με αγγλικά στην ίδια πρόταση</td>'),
 ('<td>Source code</td>', '<td>Πηγαίος κώδικας</td>'),
 ('<td>Proprietary</td>', '<td>Ιδιόκτητος</td>'),
 ('<td>Open, MIT, on GitHub</td>', '<td>Ανοιχτός, MIT, στο GitHub</td>'),
 ('Open speech models. Greek and English behind one key. We built a Windows app around them and released it under MIT.',
  'Ανοιχτά μοντέλα ομιλίας. Ελληνικά και αγγλικά πίσω από ένα πλήκτρο. Χτίσαμε γύρω τους μία εφαρμογή '
  'Windows και τη διαθέσαμε με άδεια MIT.'),
 ('Luram AI Agency, Thessaloniki', 'Luram AI Agency, Θεσσαλονίκη'),
 ('The whole comparison, row by row, with the sources: ',
  'Ολόκληρη η σύγκριση, γραμμή γραμμή, με τις πηγές: '),
 ('>the free Wispr Flow alternative for Windows</a>.',
  '>η δωρεάν εναλλακτική του Wispr Flow για Windows</a>.'),
 ('Also compare: <a href="/openwhispr-alternative/">OpenWhispr alternative</a> · '
  '<a href="/free-offline-dictation-alternatives/">Free offline dictation alternatives</a>.',
  'Δείτε επίσης: <a href="/openwhispr-alternative/">Εναλλακτική του OpenWhispr (στα αγγλικά)</a> · '
  '<a href="/free-offline-dictation-alternatives/">Δωρεάν εναλλακτικές για τοπική υπαγόρευση (στα αγγλικά)</a>.'),
 ('Illustrative receipt: 180 US dollars is twelve monthly Wispr Pro payments of 15 dollars. Annual '
  'billing costs less, and a free desktop plan exists. FU Flow\'s current local app is free under MIT; '
  'the products have different features.',
  'Ενδεικτική απόδειξη: τα 180 δολάρια είναι δώδεκα μηνιαίες πληρωμές Wispr Pro των 15 δολαρίων. '
  'Η ετήσια χρέωση κοστίζει λιγότερο και υπάρχει δωρεάν πακέτο υπολογιστή. Η τρέχουσα τοπική εφαρμογή '
  'FU Flow διατίθεται δωρεάν με MIT. Τα προϊόντα έχουν διαφορετικά χαρακτηριστικά.'),
 ('>Questions</h2>', '>Ερωτήσεις</h2>'),
 ('Tell us the program, the graphics card and what you saw. The Diagnostics tab has Recent problems ready to '
  'copy.',
  'Πείτε μας το πρόγραμμα, την κάρτα γραφικών και τι είδατε. Η καρτέλα Διαγνωστικά έχει τα Πρόσφατα '
  'προβλήματα έτοιμα για αντιγραφή.'),
 ('>Tell us</h2>', '>Πείτε μας</h2>'),
 ('Three doors. Whatever broke, whatever you want, whoever you are.',
  'Τρεις πόρτες. Ό,τι χάλασε, ό,τι θέλετε, όποιος κι αν είστε.'),
 ('Something broke', 'Κάτι χάλασε'),
 ('Report a bug on GitHub', 'Αναφέρετε σφάλμα στο GitHub'),
 ('Something you want', 'Κάτι που θέλετε'),
 ('A feature, a language quirk, a program it should paste into. Say how you would use it and we read it.',
  'Μια δυνατότητα, μια ιδιοτροπία της γλώσσας, ένα πρόγραμμα όπου πρέπει να επικολλά. Πείτε πώς θα το '
  'χρησιμοποιούσατε και το διαβάζουμε.'),
 ('Open an idea on GitHub', 'Ανοίξτε μια ιδέα στο GitHub'),
 ('No GitHub account', 'Χωρίς λογαριασμό GitHub'),
 ('An email is enough. Humans answer.', 'Ένα email αρκεί. Απαντούν άνθρωποι.'),
 ('Write to info@luram.gr', 'Γράψτε στο info@luram.gr'),
 ('>Honourable mentions</h2>', '>Ευχαριστίες</h2>'),
 ('People who put their own machine and their own hours into this, for nothing.',
  'Άνθρωποι που έβαλαν το δικό τους μηχάνημα και τις δικές τους ώρες σε αυτό, χωρίς αντάλλαγμα.'),
 ('>AMD graphics cards<', '>Κάρτες γραφικών AMD<'),
 ('>Tasos Minas<', '>Τάσος Μηνάς<'),
 ('Ran the app on AMD hardware and reported back what happened, which is how the Vulkan path stopped being a '
  'guess.',
  'Έτρεξε την εφαρμογή σε μηχάνημα με AMD και μας είπε τι έγινε, και έτσι ο δρόμος του Vulkan έπαψε να είναι '
  'εικασία.'),
 ('>A gift from</div>', '>Ευγενική χορηγία από</div>'),
 ('Luram AI Agency, AI transformation partner',
  'Luram AI Agency, συνεργάτης μετασχηματισμού με τεχνητή νοημοσύνη'),
 ('Fuck You Flow is paid for and given away by ', 'Το Fuck You Flow το πληρώνει και το χαρίζει η '),
 (' in Thessaloniki. We build tools and automations for Greek businesses. This one we built for everybody.',
  ' στη Θεσσαλονίκη. Φτιάχνουμε εργαλεία και αυτοματισμούς για ελληνικές επιχειρήσεις. Αυτό εδώ το φτιάξαμε '
  'για όλους.'),
 ('Fuck You Flow, version {{VERSION}} beta. Free local dictation under MIT.',
  'Fuck You Flow, έκδοση {{VERSION}} beta. Δωρεάν τοπική υπαγόρευση με MIT.'),
 ('Code on GitHub', 'Κώδικας στο GitHub'),
 ('All releases', 'Όλες οι εκδόσεις'),
 ('Known limits', 'Γνωστά όρια'),
 ('MIT license. The whisper.cpp engine and the models carry their own licenses. Wispr Flow is a trademark of '
  'Wispr AI, Inc. We have no relationship with them. The key is yours.',
  'Άδεια MIT. Η μηχανή whisper.cpp και τα μοντέλα έχουν τις δικές τους άδειες. Το Wispr Flow είναι σήμα '
  'κατατεθέν της Wispr AI, Inc. Δεν έχουμε καμία σχέση μαζί τους. Το πλήκτρο είναι δικό σας.'),
 ('The Fuck You Flow home screen: the large-v3 engine on the GPU, the right Alt key for start and stop, '
  'words dictated today and the last five transcripts.',
  'Η αρχική οθόνη του Fuck You Flow: η μηχανή large-v3 στην κάρτα γραφικών, το δεξί Alt για ξεκίνημα και '
  'σταμάτημα, οι λέξεις που υπαγορεύτηκαν σήμερα και οι πέντε τελευταίες μεταγραφές σε ελληνικά και '
  'αγγλικά.'),
 ('The History screen: a list of past dictations with the time, word count and wait, and an edit field for '
  'corrections.',
  'Η οθόνη Ιστορικό: μια λίστα με παλιές υπαγορεύσεις, την ώρα, τον αριθμό λέξεων και την αναμονή και ένα '
  'πεδίο για διορθώσεις.'),
 ('The Dictionary screen: a table of spoken forms and the written forms they should become.',
  'Η οθόνη Λεξικό: ένας πίνακας με τις προφορικές μορφές και τις γραπτές μορφές που πρέπει να γίνουν.'),
 ('The Statistics screen: words dictated per day, median wait and time saved, shown as numbers and bars.',
  'Η οθόνη Στατιστικά: λέξεις που υπαγορεύτηκαν ανά μέρα, μέση αναμονή και χρόνος που γλιτώθηκε, σε νούμερα '
  'και μπάρες.'),
 ('The Settings screen: the speech model, the graphics card in use, the microphone and the hotkeys.',
  'Η οθόνη Ρυθμίσεις: το μοντέλο ομιλίας, η κάρτα γραφικών που χρησιμοποιείται, το μικρόφωνο και τα πλήκτρα '
  'συντόμευσης.'),
 ('A black keyboard in the dark. The Alt key immediately to the right of the spacebar is lit from inside in '
  'acid green.',
  'Ένα μαύρο πληκτρολόγιο στο σκοτάδι. Το πλήκτρο Alt αμέσως δεξιά από το πλήκτρο διαστήματος είναι '
  'φωτισμένο από μέσα σε έντονο πράσινο.'),
 ('A green sound wave on the left breaks into drifting particles that settle into rows of writing on the '
  'right.',
  'Ένα πράσινο ηχητικό κύμα στα αριστερά διαλύεται σε σωματίδια που κατακάθονται σε σειρές γραφής στα '
  'δεξιά.'),
 ('Campaign receipt on black showing 180 dollars for Wispr Flow and zero dollars for FU Flow.',
  'Απόδειξη της καμπάνιας σε μαύρο φόντο που δείχνει 180 δολάρια για το Wispr Flow και μηδέν για το FU Flow.'),
 ('?"Dark":"Light"', '?"Σκοτεινό":"Φωτεινό"'),
 ('<a href="/el/" hreflang="el" lang="el">ΕΛ</a>', '<a href="/?lang=en" hreflang="en" lang="en">EN</a>'),
 ('<a href="/el/" lang="el" hreflang="el">Ελληνικά</a>',
  '<a href="/?lang=en" lang="en" hreflang="en">English</a>'),
 ('Make voice typing part of your work', 'Βάλτε την υπαγόρευση στην καθημερινή σας δουλειά'),
 ('Practical guides for writing, prompting and fixing dictation problems.',
  'Πρακτικοί οδηγοί για κείμενα, εντολές προς AI και προβλήματα υπαγόρευσης.'),
 ('Write documents by voice', 'Γράψτε έγγραφα με τη φωνή'),
 ('Set up Greek and English dictation in Word, then check names and punctuation before sharing.',
  'Ρυθμίστε ελληνική και αγγλική υπαγόρευση στο Word και ελέγξτε ονόματα και στίξη πριν μοιραστείτε το '
  'κείμενο.'),
 ('>Dictation in Word<', '>Υπαγόρευση στο Word<'),
 ('Speak your AI prompts', 'Πείτε τις εντολές σας προς το AI'),
 ('Dictate a brief into ChatGPT or Claude, review the text, then send it yourself.',
  'Υπαγορεύστε το αίτημά σας στο ChatGPT ή στο Claude, ελέγξτε το κείμενο και στείλτε το εσείς.'),
 ('Voice typing for AI prompts', 'Φωνητική πληκτρολόγηση για εντολές προς AI'),
 ('Keep dictation local', 'Κρατήστε την υπαγόρευση τοπική'),
 ('Understand speech processing, saved history and update checks.',
  'Δείτε πώς λειτουργούν η επεξεργασία ομιλίας, το αποθηκευμένο ιστορικό και ο έλεγχος ενημερώσεων.'),
 ('Privacy and offline operation', 'Ιδιωτικότητα και λειτουργία χωρίς σύνδεση'),
 ('All dictation guides', 'Όλοι οι οδηγοί υπαγόρευσης'),
 ('href="/guides/', 'href="/el/guides/'),
 ('href="/wispr-flow-alternative/"', 'href="/el/wispr-flow-alternative/"'),
 ('href="/privacy/"', 'href="/el/privacy/"')]


def put_nav(html, lang):
    """Swap the template's menu for the one this language should carry.

    The whole block is replaced rather than translated word by word, so the
    menu has a single source (site_nav.py) and cannot drift between the two
    languages or between the home page and the rest of the site.
    """
    m = re.search(r'<button class="nav-btn".*?</nav>', html, re.S)
    if not m:
        print("MENU: the nav block was not found in the template")
        sys.exit(1)
    return html[:m.start()] + site_nav.nav_html(lang) + html[m.end():]


def rebuild_faq(html, faq):
    """Rewrite the visible questions from the same list the structured data uses.

    Keep visible answers and machine-readable answers consistent. This does not
    promise FAQ rich results or a ranking benefit.
    """
    items = []
    for i, (q, a) in enumerate(faq):
        items.append('    <details%s>\n      <summary>%s</summary>\n      <p>%s</p>\n    </details>'
                     % (" open" if i == 0 else "", esc(q), esc(a)))
    block = "\n".join(items)
    start = html.index('<section id="faq"')
    first = html.index("<details", start)
    last = html.index("</details>", start)
    while True:
        nxt = html.find("</details>", last + 1)
        if nxt == -1 or nxt > html.index("</section>", start):
            break
        last = nxt
    last += len("</details>")
    return html[:first] + block.lstrip() + html[last:]


def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def build():
    src = io.open(SRC, encoding="utf-8").read()

    # the design file points at ../assets, both published pages use absolute paths
    src = src.replace('"../assets/', '"/assets/')
    # the design file is kept out of the index while it is a draft
    src = src.replace('<meta name="robots" content="noindex">\n', "")
    # the H1 carries the word people actually search for
    src = src.replace("Dictation that lives on your PC.",
                      "Free dictation that lives on your PC.")

    # the language redirect uses ?lang=en as its escape hatch. Once the page is open
    # the query has done its work, so drop it from the address bar and keep the URL
    # the canonical one.
    src = src.replace("</body>", """<script>
(function(){try{if(location.search.indexOf("lang=en")!==-1){
history.replaceState(null,"",location.pathname+location.hash)}}catch(e){}})();
</script>
</body>""")

    head_start = src.index("<title>")
    head_end = src.index('<meta name="theme-color" content="#000000">') + len(
        '<meta name="theme-color" content="#000000">')
    before, after = src[:head_start], src[head_end:]

    # ---- English
    en = before + HEAD_EN + "\n" + jsonld("en") + after
    en = rebuild_faq(en, FAQ_EN)
    en = put_nav(en, "en")
    en = release_tokens(en)
    write(OUT_EN, en)

    # ---- Greek
    el = before + HEAD_EL + "\n" + jsonld("el") + after
    el = el.replace('<html lang="en"', '<html lang="el"')
    # The app pictures come in two sets. The English page shows English sample
    # dictations and an English sample dictionary; the Greek page shows the Greek
    # ones, which are what proves the Greek actually works. Both sets are built by
    # press/make_shots.py.
    for stem in ("1-home", "2-history", "3-dictionary", "5-statistics", "6-settings"):
        el = el.replace("/assets/%s.png" % stem, "/assets/%s-el.png" % stem)
    el = rebuild_faq(el, FAQ_EL)
    missing = []
    for a, b in TR:
        if a not in el:
            missing.append(a[:60])
            continue
        el = el.replace(a, b)
    if missing:
        print("MISSING %d source strings, translation incomplete:" % len(missing))
        for m in missing:
            print("   ", m)
        sys.exit(1)
    el = put_nav(el, "el")
    el = release_tokens(el, "el")
    write(OUT_EL, el)

    check_greek(el)
    import build_content
    import build_changelog
    build_content.build()
    build_changelog.main()
    write_llms()


def write_llms():
    """Write llms.txt and llms-full.txt from the same constants the pages use.

    Both files used to be edited by hand and both went stale: on 7 September 2026
    they still announced 0.9.0 and linked an installer that returned 404.
    """
    fields = {
        "ver": VERSION,
        "dl": DL,
        "base": BASE,
        "size": SIZE,
        "today": CONTENT_DATE,
    }
    import build_content
    fields["guides"] = '\n'.join('- [%s](%s/guides/%s/)' % (g['en']['title'], BASE, g['slug']) for g in build_content.ALL_GUIDES)
    write(OUT_LLMS, llms_source.LLMS.format(**fields))
    write(OUT_LLMS_FULL, llms_source.LLMS_FULL.format(**fields))
    write_sitemap()
    # the old files carried a dead download link, so the check is worth keeping
    for path in (OUT_LLMS, OUT_LLMS_FULL):
        body = io.open(path, encoding="utf-8").read()
        stale = re.findall(r"v?0\.9\.2|Setup\.0\.9\.2", body)
        if stale:
            print("STALE VERSION in %s: %s" % (os.path.basename(path), set(stale)))
            sys.exit(1)


# words that are meant to stay in Latin script on the Greek page
KEEP = set("""fuck you flow fu wispr github mit vulkan nvidia amd intel windows word chrome
slack outlook whisper large v3 alt beta gb sha256 luram ai agency inc api llms txt en el html
info luram gr techcrunch macos ios android rtx png webp""".split())


def check_greek(el):
    """Fail loudly if any English sentence survived in the Greek page.

    The first version of this check listed a handful of phrases by hand and missed the
    entire questions section. This one reads every visible string instead.
    """
    body = el.split("<body", 1)[1]
    body = re.sub(r"<script.*?</script>", "", body, flags=re.S)
    body = re.sub(r"<style.*?</style>", "", body, flags=re.S)
    # a <q> is a verbatim quotation and stays in the language it was said in
    body = re.sub(r"<q>.*?</q>", "", body, flags=re.S)
    texts = re.findall(r">([^<>]+)<", body)
    texts += re.findall(r'(?:alt|aria-label|title|content)="([^"]+)"', body)
    bad = []
    for t in texts:
        words = re.findall(r"[A-Za-z][A-Za-z'.]{1,}", t)
        unknown = [w for w in words if w.lower().strip(".") not in KEEP]
        if len(unknown) >= 4:
            bad.append(" ".join(t.split())[:90])
    if bad:
        print("ENGLISH LEFT IN THE GREEK PAGE, %d place(s):" % len(bad))
        for b in bad:
            print("   ", b)
        sys.exit(1)
    print("clean: every visible string on the Greek page is Greek")



ALT = ('    <xhtml:link rel="alternate" hreflang="en" href="{b}/"/>\n'
       '    <xhtml:link rel="alternate" hreflang="el" href="{b}/el/"/>\n'
       '    <xhtml:link rel="alternate" hreflang="x-default" href="{b}/"/>\n').format(b=BASE)


def write_sitemap():
    """Publish canonical URLs. Omit lastmod rather than invent a date on every build."""
    log_alt = ('    <xhtml:link rel="alternate" hreflang="en" href="{b}/changelog/"/>\n'
               '    <xhtml:link rel="alternate" hreflang="el" href="{b}/el/changelog/"/>\n'
               '    <xhtml:link rel="alternate" hreflang="x-default" href="{b}/changelog/"/>\n').format(b=BASE)
    urls = [(BASE + "/", ALT), (BASE + "/el/", ALT),
            (BASE + "/changelog/", log_alt), (BASE + "/el/changelog/", log_alt)]
    import build_content
    urls.extend((BASE + path, "") for path in build_content.comparison_paths() if path not in build_content.paired_paths())
    for path in build_content.paired_paths():
        en, el = BASE + path, BASE + "/el" + path
        alternate = ''.join('    <xhtml:link rel="alternate" hreflang="%s" href="%s"/>\n' % (lang, u)
                            for lang, u in (("en", en), ("el", el), ("x-default", en)))
        urls.extend(((en, alternate), (el, alternate)))
    body = "".join(
        "  <url>\n    <loc>%s</loc>\n%s  </url>\n" % (u, a)
        for u, a in urls)
    write(OUT_SITEMAP,
          '<?xml version="1.0" encoding="UTF-8"?>\n'
          '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" '
          'xmlns:xhtml="http://www.w3.org/1999/xhtml">\n' + body + "</urlset>\n")


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    io.open(path, "w", encoding="utf-8", newline="\n").write(text)
    print("%-40s %s KB" % (os.path.relpath(path, HERE), round(len(text.encode("utf-8")) / 1024)))


if __name__ == "__main__":
    build()
