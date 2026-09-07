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

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, "variants", "v4-coral.html")
OUT_EN = os.path.join(HERE, "index.html")
OUT_EL = os.path.join(HERE, "el", "index.html")

BASE = "https://fuckyouflow.app"
VERSION = "0.9.2"
DL = ("https://github.com/Breakzoras/fuck-you-flow/releases/download/"
      "v0.9.2/Fuck.You.Flow.Setup.0.9.2.exe")
SIZE = "1.7 GB"        # 1,729,386,366 bytes, built on 8 September 2026. Larger than 0.9.1
                       # on purpose: the payload is no longer compressed, so the install
                       # writes the files straight out instead of unpacking them.
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
<meta name="google-site-verification" content="googlefde419ece67880c4.html">
<meta property="og:type" content="website">
<meta property="og:site_name" content="Fuck You Flow">
<meta property="og:image" content="{base}/assets/og.png">
<meta property="og:image:width" content="1200">
<meta property="og:image:height" content="630">
<meta property="og:image:alt" content="A matte black sculptural hand lit with acid green, beside the words Fuck You Flow, free dictation for Windows.">
<meta name="twitter:card" content="summary_large_image">""".format(base=BASE)

HEAD_EN = """<title>Fuck You Flow: the free Wispr Flow alternative for Windows</title>
<meta name="description" content="A free Wispr Flow alternative for Windows. Press one key, speak, press it again, and the text lands in any program. Greek and English, running on your own PC, with no word limit, no subscription and no account. Open source, MIT.">
<link rel="canonical" href="{base}/">
{common}
<meta property="og:title" content="Fuck You Flow: free dictation for Windows">
<meta property="og:description" content="The free Wispr Flow alternative. One key, any program, Greek and English, all of it on your own PC. No word limit, no subscription, no account.">
<meta property="og:url" content="{base}/">
<meta property="og:locale" content="en_US">
<meta property="og:locale:alternate" content="el_GR">""".format(base=BASE, common=COMMON_LINKS)

HEAD_EL = """<title>Fuck You Flow: δωρεάν υπαγόρευση για Windows, η εναλλακτική του Wispr Flow</title>
<meta name="description" content="Δωρεάν εναλλακτική του Wispr Flow για Windows. Πατάτε ένα πλήκτρο, μιλάτε, το ξαναπατάτε και το κείμενο προσγειώνεται σε όποιο πρόγραμμα θέλετε. Ελληνικά και αγγλικά, όλα στον δικό σας υπολογιστή, χωρίς όριο λέξεων, χωρίς συνδρομή, χωρίς λογαριασμό. Ανοιχτός κώδικας, MIT.">
<link rel="canonical" href="{base}/el/">
{common}
<meta property="og:title" content="Fuck You Flow: δωρεάν υπαγόρευση για Windows">
<meta property="og:description" content="Η δωρεάν εναλλακτική του Wispr Flow. Ένα πλήκτρο, κάθε πρόγραμμα, ελληνικά και αγγλικά, όλα στον υπολογιστή σας. Χωρίς όριο λέξεων, χωρίς συνδρομή, χωρίς λογαριασμό.">
<meta property="og:url" content="{base}/el/">
<meta property="og:locale" content="el_GR">
<meta property="og:locale:alternate" content="en_US">""".format(base=BASE, common=COMMON_LINKS)

# ---------------------------------------------------------------- structured data

def jsonld(lang):
    url = BASE + ("/el/" if lang == "el" else "/")
    if lang == "el":
        desc = ("Δωρεάν υπαγόρευση για Windows με ελληνικά και αγγλικά. Τρέχει εξ ολοκλήρου "
                "στον υπολογιστή του χρήστη. Εναλλακτική του Wispr Flow χωρίς συνδρομή "
                "και χωρίς όριο λέξεων.")
        faq = FAQ_EL
        pub = "Luram AI Agency"
    else:
        desc = ("Free dictation for Windows in Greek and English. Runs entirely on the user's "
                "own PC. A Wispr Flow alternative with no subscription and no word limit.")
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
      "fileSize": "1464MB",
      "license": "https://opensource.org/licenses/MIT",
      "isAccessibleForFree": true,
      "offers": {"@type": "Offer", "price": "0", "priceCurrency": "EUR",
                 "availability": "https://schema.org/InStock"},
      "softwareRequirements": "Windows 10 or 11, 64-bit. A graphics card from NVIDIA, AMD or Intel makes it faster; without one it runs on the processor.",
      "memoryRequirements": "8 GB RAM",
      "storageRequirements": "3 GB",
      "featureList": [
        "One hotkey dictation into any Windows program, using the right Alt key",
        "Speech recognition on the user's own machine, with no server and no account",
        "English and Greek, mixed inside a single sentence",
        "No word limit and no subscription",
        "A personal dictionary that learns from corrections",
        "Snippets: one spoken phrase expands into a block of text",
        "Local history and statistics, kept in SQLite on the user's disk",
        "Works offline after installation",
        "Password fields are detected and left empty",
        "Open source under the MIT license"
      ],
      "keywords": "free dictation Windows, Wispr Flow alternative, offline speech to text, local Whisper, Greek dictation, open source voice typing",
      "datePublished": "2026-09-06",
      "dateModified": "%(today)s",
      "author": {"@id": "https://luram.gr/#org"},
      "publisher": {"@id": "https://luram.gr/#org"},
      "maintainer": {"@id": "https://luram.gr/#org"},
      "codeRepository": "https://github.com/Breakzoras/fuck-you-flow",
      "programmingLanguage": ["Rust", "TypeScript", "C++"],
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
                "today": datetime.date.today().isoformat(),
                "howto_name": jstr(howto_name), "steps": steps}


def jstr(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


HOWTO = {
    "en": ("How to dictate text into any Windows program with Fuck You Flow", [
        ("Press the right Alt key",
         "A small window with a waveform appears and the app starts listening."),
        ("Speak",
         "English, Greek, or both in the same sentence. Each finished phrase is transcribed "
         "while you keep talking."),
        ("Press the right Alt key again",
         "The text lands where your cursor was, with punctuation. Escape cancels instead."),
    ]),
    "el": ("Πώς να υπαγορεύσετε κείμενο σε κάθε πρόγραμμα των Windows με το Fuck You Flow", [
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


FAQ_EN = [
    ("Is it a free alternative to Wispr Flow?",
     "Yes. Same job, one-key dictation into any Windows program, no subscription, no account, "
     "and no weekly word limit. Open source under MIT on GitHub."),
    ("Where does my voice go?",
     "Nowhere. Recognition runs on your own machine with the Whisper model. After setup no "
     "internet connection is needed. Password fields are detected and left empty."),
    ("Does it need an NVIDIA card?",
     "No. NVIDIA, AMD and Intel through Vulkan. Without a graphics card it runs on the "
     "processor, more slowly."),
    ("Why does Windows show a blue warning during install?",
     "The installer carries no purchased code-signing certificate. Click More info, then Run "
     "anyway. The SHA256 checksum is on GitHub."),
    ("What does beta mean?",
     "It runs daily on our machines and is now going out to yours. Some programs may refuse "
     "the paste, some cards are untested. Whatever breaks, we want it written down."),
    ("Why the name?",
     "A dictation company was valued at 2 billion dollars for something open models have done "
     "for free since September 2022. The name is the reply. On social media we write FU Flow."),
]

FAQ_EL = [
    ("Είναι δωρεάν εναλλακτική του Wispr Flow;",
     "Ναι. Ίδια δουλειά, υπαγόρευση με ένα πλήκτρο σε κάθε πρόγραμμα των Windows, χωρίς "
     "συνδρομή, χωρίς λογαριασμό, χωρίς εβδομαδιαίο όριο λέξεων. Ανοιχτός κώδικας με άδεια "
     "MIT στο GitHub."),
    ("Πού πάει η φωνή μου;",
     "Πουθενά. Η αναγνώριση τρέχει στο δικό σας μηχάνημα με το μοντέλο Whisper. Μετά την "
     "εγκατάσταση καμία σύνδεση στο ίντερνετ δεν χρειάζεται. Τα πεδία κωδικών αναγνωρίζονται "
     "και μένουν άδεια."),
    ("Χρειάζεται κάρτα NVIDIA;",
     "Όχι. NVIDIA, AMD και Intel μέσα από το Vulkan. Χωρίς κάρτα γραφικών τρέχει στον "
     "επεξεργαστή, πιο αργά."),
    ("Γιατί βγάζουν τα Windows μπλε προειδοποίηση στην εγκατάσταση;",
     "Ο εγκαταστάτης δεν έχει αγορασμένο πιστοποιητικό υπογραφής κώδικα. Πατήστε Περισσότερες "
     "πληροφορίες, μετά Εκτέλεση ούτως ή άλλως. Το άθροισμα ελέγχου SHA256 βρίσκεται στο GitHub."),
    ("Τι σημαίνει beta;",
     "Τρέχει καθημερινά στα δικά μας μηχανήματα και τώρα βγαίνει και στα δικά σας. Κάποια "
     "προγράμματα μπορεί να αρνηθούν την επικόλληση, κάποιες κάρτες μένουν αδοκίμαστες. Ό,τι "
     "χαλάσει, θέλουμε να το μάθουμε."),
    ("Γιατί αυτό το όνομα;",
     "Μια εταιρεία υπαγόρευσης αποτιμήθηκε στα 2 δισεκατομμύρια δολάρια για κάτι που τα "
     "ανοιχτά μοντέλα κάνουν δωρεάν από τον Σεπτέμβριο του 2022. Το όνομα είναι η απάντηση. "
     "Στα κοινωνικά δίκτυα γράφουμε FU Flow."),
]

# ---------------------------------------------------------------- translations
# Ordered. Longer strings first where one contains another.
TR = [
    # header and navigation
    ("Skip to content", "Στο περιεχόμενο"),
    (">Features<", ">Τι κάνει<"),
    (">Speed</a>", ">Ταχύτητα</a>"),
    (">The bill</a>", ">Ο λογαριασμός</a>"),
    (">Questions</a>", ">Ερωτήσεις</a>"),
    (">Thanks</a>", ">Ευχαριστίες</a>"),

    # hero
    ("Windows 10 and 11", "Windows 10 και 11"),
    ("Free dictation that lives on your PC.",
     "Δωρεάν υπαγόρευση που ζει στον υπολογιστή σας."),
    ("Wispr Flow raised 361 million dollars and hands you 2,000 words a week before the meter "
     "starts at 15 dollars a month. We put the same thing behind one key and left the meter "
     "off. Greek and English, everything on your PC, nothing in the cloud.",
     "Το Wispr Flow σήκωσε 361 εκατομμύρια δολάρια και σας δίνει 2.000 λέξεις την εβδομάδα "
     "πριν αρχίσει ο μετρητής στα 15 δολάρια τον μήνα. Εμείς βάλαμε το ίδιο πράγμα πίσω από "
     "ένα πλήκτρο και αφήσαμε τον μετρητή κλειστό. Ελληνικά και αγγλικά, όλα στον υπολογιστή "
     "σας, τίποτα στο σύννεφο."),
    ("Download for Windows", "Κατεβάστε το για Windows"),
    (">Read the code</a>", ">Δείτε τον κώδικα</a>"),
    ("Version 0.9.2 beta. 1.7 GB with every model inside. No internet connection after setup. "
     "MIT license. The SHA256 checksum is on the ",
     "Έκδοση 0.9.2 beta. 1,7 GB με όλα τα μοντέλα μέσα. Καμία σύνδεση στο ίντερνετ μετά την "
     "εγκατάσταση. Άδεια MIT. Το άθροισμα ελέγχου SHA256 βρίσκεται στη "),
    # The changelog sits under the download on both pages, each in its own language.
    (". What changed in each version is on the <a href=\"/changelog/\">changelog</a>.",
     ". Τι άλλαξε σε κάθε έκδοση βρίσκεται στο <a href=\"/el/changelog/\">ιστορικό αλλαγών</a>."),
    (">release page</a>", ">σελίδα της έκδοσης</a>"),
    ("The listening window: a coral dot, Listening 4.2 s, and a green waveform.",
     "Το παράθυρο ακρόασης: μια κοραλί τελεία, Ακούει 4,2 δ και μια πράσινη κυματομορφή."),
    ("Listening 4.2 s", "Ακούει 4,2 δ"),

    # the reason the thing exists
    ("Open model. Public code. Somebody charged you for it anyway.",
     "Ανοιχτό μοντέλο. Δημόσιος κώδικας. Κάποιος σας χρέωσε ούτως ή άλλως."),
    (">Why free</a>", ">Γιατί δωρεάν</a>"),
    (">Why this is free</h2>", ">Γιατί είναι δωρεάν</h2>"),
    ("The speech model was already open.", "Το μοντέλο ομιλίας ήταν ήδη ανοιχτό."),
    ("OpenAI published Whisper on 21 September 2022. Its own repository says it in one line: ",
     "Η OpenAI δημοσίευσε το Whisper στις 21 Σεπτεμβρίου 2022. Το ίδιο της το αποθετήριο "
     "το γράφει σε μία γραμμή: "),
    (" Public weights, public code, running on a graphics card you already paid for. That has "
     "been true for four years.",
     " Δημόσια βάρη, δημόσιος κώδικας, μια κάρτα γραφικών που έχετε ήδη πληρώσει. Αυτό ισχύει "
     "εδώ και τέσσερα χρόνια."),
    ("What the companies added on top was a login, a monthly fee and a counter that stops at "
     "2,000 words a week. Wispr Flow raised 361 million dollars doing it and reached a valuation "
     "of 2 billion. The recognition underneath is the same open model, and it was already yours.",
     "Αυτό που πρόσθεσαν από πάνω οι εταιρείες ήταν ένας λογαριασμός, μια μηνιαία συνδρομή και "
     "ένας μετρητής που σταματάει στις 2.000 λέξεις την εβδομάδα. Το Wispr Flow σήκωσε 361 "
     "εκατομμύρια δολάρια κάνοντάς το και έφτασε σε αποτίμηση 2 δισεκατομμυρίων. Η αναγνώριση "
     "από κάτω είναι το ίδιο ανοιχτό μοντέλο. Ήταν ήδη δικό σας."),
    ("So we wrapped it in one key and handed it back. It costs zero because it cost us close to "
     "zero. It runs on your machine because that is where it could always run. It asks for no "
     "account because we want to know nothing about you.",
     "Εμείς το τυλίξαμε σε ένα πλήκτρο και σας το δώσαμε πίσω. Κοστίζει μηδέν επειδή μας κόστισε "
     "σχεδόν μηδέν. Τρέχει στο μηχάνημά σας επειδή εκεί μπορούσε πάντα να τρέξει. Δεν ζητάει "
     "λογαριασμό επειδή δεν θέλουμε να ξέρουμε τίποτα για εσάς."),
    ("Take it. Read the code. Fork it. The MIT licence even lets you sell it.",
     "Πάρτε το. Διαβάστε τον κώδικα. Κάντε το δικό σας. Η άδεια MIT σας επιτρέπει ακόμα και να "
     "το πουλήσετε."),

    # what is inside
    ("What is inside", "Τι έχει μέσα"),
    ("Six things the app does on its own, on your machine, without asking anyone.",
     "Έξι πράγματα που κάνει μόνη της η εφαρμογή, στο μηχάνημά σας, χωρίς να ρωτήσει κανέναν."),
    ("Greek that holds up", "Ελληνικά που στέκουν"),
    ("Question marks come from grammar rules or from the rise of your voice. Spoken "
     "self-corrections are applied before the text lands.",
     "Τα ερωτηματικά βγαίνουν από κανόνες γραμματικής ή από το ανέβασμα της φωνής σας. Οι "
     "προφορικές αυτοδιορθώσεις εφαρμόζονται πριν προσγειωθεί το κείμενο."),
    (">Snippets<", ">Έτοιμα κομμάτια<"),
    ("Say a phrase, get a whole block of text. Signatures, addresses, the replies you send "
     "every day.",
     "Πείτε μια φράση, πάρτε ολόκληρο κομμάτι κειμένου. Υπογραφές, διευθύνσεις, οι απαντήσεις "
     "που στέλνετε κάθε μέρα."),
    ("Nothing leaves", "Τίποτα δεν φεύγει"),
    ("No server, no analytics, no account. Password fields are detected and stay empty.",
     "Κανένας διακομιστής, καμία μέτρηση, κανένας λογαριασμός. Τα πεδία κωδικών αναγνωρίζονται "
     "και μένουν άδεια."),

    ("Correct it once. It learns.", "Διορθώστε το μία φορά. Το μαθαίνει."),
    ("Every dictation is kept in History, one file on your disk. Fix a word there and the fix "
     "becomes a dictionary rule. The next time you say it, it lands the way you wrote it.",
     "Κάθε υπαγόρευση φυλάγεται στο Ιστορικό, ένα αρχείο στον δίσκο σας. Διορθώστε εκεί μια "
     "λέξη και η διόρθωση γίνεται κανόνας λεξικού. Την επόμενη φορά που θα την πείτε, "
     "προσγειώνεται όπως τη γράψατε."),
    (">History<", ">Ιστορικό<"),
    ("Your words, your rules.", "Οι λέξεις σας, οι κανόνες σας."),
    ("Names, brands, the terms of your trade. Add them to the dictionary and the engine spells "
     "them your way, every time. Rules made in History show up here too, ready to edit.",
     "Ονόματα, μάρκες, οι όροι της δουλειάς σας. Βάλτε τα στο λεξικό και η μηχανή τα γράφει με "
     "τον δικό σας τρόπο, κάθε φορά. Οι κανόνες που φτιάχνονται στο Ιστορικό εμφανίζονται κι "
     "εδώ, έτοιμοι για αλλαγή."),
    (">Dictionary<", ">Λεξικό<"),
    ("How much you said, how much you saved.", "Πόσα είπατε, πόσα κερδίσατε."),
    ("Words per day, median wait, minutes saved against typing. What you said yesterday is one "
     "click away. All of it stays in one file on your disk and goes nowhere else.",
     "Λέξεις τη μέρα, μέση αναμονή, λεπτά που γλιτώσατε από την πληκτρολόγηση. Ό,τι είπατε χθες "
     "απέχει ένα κλικ. Όλα μένουν σε ένα αρχείο στον δίσκο σας και δεν πάνε πουθενά αλλού."),
    (">Statistics<", ">Στατιστικά<"),
    ("Every graphics card. It picks the model itself.",
     "Κάθε κάρτα γραφικών. Διαλέγει μόνη της το μοντέλο."),
    ("NVIDIA, AMD and Intel all run through Vulkan. The first start reads your machine and "
     "picks the model that fits it. Change the key, the microphone or the model here whenever "
     "you like.",
     "NVIDIA, AMD και Intel περνούν όλες από το Vulkan. Το πρώτο ξεκίνημα διαβάζει το μηχάνημά "
     "σας και διαλέγει το μοντέλο που του ταιριάζει. Αλλάξτε εδώ το πλήκτρο, το μικρόφωνο ή το "
     "μοντέλο όποτε θέλετε."),
    (">Settings<", ">Ρυθμίσεις<"),

    # one key
    (">the right one<", ">το δεξί<"),
    ("One key.", "Ένα πλήκτρο."),
    ("Press right Alt.</h3><p>A small window with a waveform appears. It listens.",
     "Πατήστε δεξί Alt.</h3><p>Εμφανίζεται ένα μικρό παράθυρο με κυματομορφή. Ακούει."),
    ("Speak.</h3><p>Greek, English or both. Each finished phrase is transcribed while you keep "
     "talking.",
     "Μιλήστε.</h3><p>Ελληνικά, αγγλικά ή και τα δύο. Κάθε φράση που τελειώνει μεταγράφεται "
     "όσο εσείς συνεχίζετε να μιλάτε."),
    ("Press right Alt again.</h3><p>The text lands where your cursor was. With periods, commas "
     "and question marks.",
     "Πατήστε ξανά δεξί Alt.</h3><p>Το κείμενο προσγειώνεται εκεί που ήταν ο δρομέας σας. Με "
     "τελείες, κόμματα και ερωτηματικά."),
    ("Works in any program: Word, Chrome, Slack, Outlook, even a terminal.",
     "Δουλεύει σε κάθε πρόγραμμα: Word, Chrome, Slack, Outlook, ακόμα και σε τερματικό."),

    # speed
    (">Speed</h2>", ">Ταχύτητα</h2>"),
    ("Four real dictations on an RTX 3070 with the large model. The coral bar is how long the "
     "person talked. The green bar is how long they then waited for the text. Both bars are "
     "drawn on the same scale.",
     "Τέσσερις πραγματικές υπαγορεύσεις σε RTX 3070 με το μεγάλο μοντέλο. Η κοραλί μπάρα είναι "
     "πόση ώρα μίλησε ο άνθρωπος. Η πράσινη είναι πόση ώρα περίμενε μετά για το κείμενο. Οι "
     "δύο μπάρες είναι στην ίδια κλίμακα."),
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
    ("The wait depends only on the last phrase, because everything before it was transcribed "
     "while you were still speaking. AMD and Intel cards run through Vulkan at the same speed. "
     "Without a graphics card it runs on the processor, with a wait of a few seconds.",
     "Η αναμονή εξαρτάται μόνο από την τελευταία φράση, γιατί ό,τι προηγήθηκε μεταγράφηκε όσο "
     "ακόμα μιλούσατε. Οι κάρτες AMD και Intel περνούν από το Vulkan με την ίδια ταχύτητα. "
     "Χωρίς κάρτα γραφικών τρέχει στον επεξεργαστή, με αναμονή λίγων δευτερολέπτων."),

    # the bill
    (">The bill</h2>", ">Ο λογαριασμός</h2>"),
    ('<span class="price-tag">12 to 15 dollars a month</span>',
     '<span class="price-tag">12 ως 15 δολάρια τον μήνα</span>'),
    ('<span class="price-tag">0 dollars, forever</span>',
     '<span class="price-tag">0 δολάρια, για πάντα</span>'),
    ("<td>Free to 2,000 words a week, then the meter starts</td>",
     "<td>Δωρεάν ως 2.000 λέξεις την εβδομάδα, μετά ξεκινά ο μετρητής</td>"),
    ("<td>Zero, and it stays zero</td>", "<td>Μηδέν, και μηδέν μένει</td>"),
    ("Every row verified. The valuation and the funding are from TechCrunch, 17 August 2026. "
     "The prices and the weekly word count are from the Wispr Flow pricing page, read on "
     "6 September 2026.",
     "Κάθε γραμμή επαληθευμένη. Η αποτίμηση και η χρηματοδότηση είναι από το TechCrunch, "
     "17 Αυγούστου 2026. Οι τιμές και οι εβδομαδιαίες λέξεις είναι από τη σελίδα τιμών του "
     "Wispr Flow, όπως διαβάστηκε στις 6 Σεπτεμβρίου 2026."),
    ("<td>Price</td>", "<td>Τιμή</td>"),
    ("<td>Words per week</td>", "<td>Λέξεις την εβδομάδα</td>"),
    ("<td>2,000 on the free plan</td>", "<td>2.000 στο δωρεάν πακέτο</td>"),
    ("<td>As many as you can say</td>", "<td>Όσες προλαβαίνετε να πείτε</td>"),
    ("<td>Company valuation</td>", "<td>Αποτίμηση εταιρείας</td>"),
    ("<td>2 billion dollars (August 2026)</td>",
     "<td>2 δισεκατομμύρια δολάρια (Αύγουστος 2026)</td>"),
    ("<td>One domain name</td>", "<td>Ένα όνομα χώρου</td>"),
    ("<td>Investor money</td>", "<td>Χρήματα επενδυτών</td>"),
    ("<td>361 million dollars</td>", "<td>361 εκατομμύρια δολάρια</td>"),
    ("<td>One weekend</td>", "<td>Ένα σαββατοκύριακο</td>"),
    ("<td>Where your voice goes</td>", "<td>Πού πάει η φωνή σας</td>"),
    ("<td>Their servers</td>", "<td>Στους διακομιστές τους</td>"),
    ("<td>Into the text. Then nowhere.</td>", "<td>Στο κείμενο. Μετά πουθενά.</td>"),
    ("<td>Account</td>", "<td>Λογαριασμός</td>"),
    ("<td>Required</td>", "<td>Απαιτείται</td>"),
    ("<td>None</td>", "<td>Κανένας</td>"),
    ("<td>Internet</td>", "<td>Ίντερνετ</td>"),
    ("<td>Always</td>", "<td>Πάντα</td>"),
    ("<td>Only to download it</td>", "<td>Μόνο για να το κατεβάσετε</td>"),
    ("<td>Greek</td>", "<td>Ελληνικά</td>"),
    ("<td>One of 100+ languages</td>", "<td>Μία από 100+ γλώσσες</td>"),
    ("<td>First language, mixed with English in the same sentence</td>",
     "<td>Πρώτη γλώσσα, ανακατεμένη με αγγλικά στην ίδια πρόταση</td>"),
    ("<td>Source code</td>", "<td>Πηγαίος κώδικας</td>"),
    ("<td>Secret</td>", "<td>Μυστικός</td>"),
    ("<td>Open, MIT, on GitHub</td>", "<td>Ανοιχτός, MIT, στο GitHub</td>"),
    ("The models that do Greek well have been open and free for four years. Someone had to put "
     "them behind a key. We did.",
     "Τα μοντέλα που τα πάνε καλά με τα ελληνικά είναι ανοιχτά και δωρεάν εδώ και δύο χρόνια. "
     "Κάποιος έπρεπε να τα βάλει πίσω από ένα πλήκτρο. Το κάναμε."),
    ("Luram AI Agency, Thessaloniki", "Luram AI Agency, Θεσσαλονίκη"),

    # questions
    ("The whole comparison, row by row, with the sources: ",
     "Ολόκληρη η σύγκριση, γραμμή γραμμή, με τις πηγές: "),
    (">the free Wispr Flow alternative for Windows</a>.",
     '>η δωρεάν εναλλακτική του Wispr Flow για Windows</a>, στα αγγλικά.'),
    (">Questions</h2>", ">Ερωτήσεις</h2>"),
    ("Tell us the program, the graphics card and what you saw. The Diagnostics tab has Recent "
     "problems ready to copy.",
     "Πείτε μας το πρόγραμμα, την κάρτα γραφικών και τι είδατε. Η καρτέλα Διαγνωστικά έχει τα "
     "Πρόσφατα προβλήματα έτοιμα για αντιγραφή."),
    (">Tell us</h2>", ">Πείτε μας</h2>"),
    ("Three doors. Whatever broke, whatever you want, whoever you are.",
     "Τρεις πόρτες. Ό,τι χάλασε, ό,τι θέλετε, όποιος κι αν είστε."),
    ("Something broke", "Κάτι χάλασε"),
    ("Report a bug on GitHub", "Αναφέρετε σφάλμα στο GitHub"),
    ("Something you want", "Κάτι που θέλετε"),
    ("A feature, a language quirk, a program it should paste into. Say how you would use it "
     "and we read it.",
     "Μια δυνατότητα, μια ιδιοτροπία της γλώσσας, ένα πρόγραμμα όπου πρέπει να επικολλά. Πείτε "
     "πώς θα το χρησιμοποιούσατε και το διαβάζουμε."),
    ("Open an idea on GitHub", "Ανοίξτε μια ιδέα στο GitHub"),
    ("No GitHub account", "Χωρίς λογαριασμό GitHub"),
    ("An email is enough. Humans answer.", "Ένα email αρκεί. Απαντούν άνθρωποι."),
    ("Write to info@luram.gr", "Γράψτε στο info@luram.gr"),

    # honourable mentions
    (">Honourable mentions</h2>", ">Ευχαριστίες</h2>"),
    ("People who put their own machine and their own hours into this, for nothing.",
     "Άνθρωποι που έβαλαν το δικό τους μηχάνημα και τις δικές τους ώρες σε αυτό, χωρίς αντάλλαγμα."),
    (">AMD graphics cards<", ">Κάρτες γραφικών AMD<"),
    (">Tasos Minas<", ">Τάσος Μηνάς<"),
    ("Ran the app on AMD hardware and reported back what happened, which is how the Vulkan path "
     "stopped being a guess.",
     "Έτρεξε την εφαρμογή σε μηχάνημα με AMD και μας είπε τι έγινε, και έτσι ο δρόμος του "
     "Vulkan έπαψε να είναι εικασία."),

    # the agency block in the footer
    (">A gift from</div>", ">Ευγενική χορηγία από</div>"),
    ("Luram AI Agency, AI transformation partner",
     "Luram AI Agency, συνεργάτης μετασχηματισμού με τεχνητή νοημοσύνη"),
    ("Fuck You Flow is paid for and given away by ",
     "Το Fuck You Flow το πληρώνει και το χαρίζει η "),
    (" in Thessaloniki. We build tools and automations for Greek businesses. This one we built "
     "for everybody.",
     " στη Θεσσαλονίκη. Φτιάχνουμε εργαλεία και αυτοματισμούς για ελληνικές επιχειρήσεις. "
     "Αυτό εδώ το φτιάξαμε για όλους."),
    ("Fuck You Flow, version 0.9.2 beta. Free for everybody, and it stays free.",
     "Fuck You Flow, έκδοση 0.9.2 beta. Δωρεάν για όλους, και δωρεάν μένει."),

    # footer
    ("Code on GitHub", "Κώδικας στο GitHub"),
    ("All releases", "Όλες οι εκδόσεις"),
    ("Known limits", "Γνωστά όρια"),
    ("MIT license. The whisper.cpp engine and the models carry their own licenses. Wispr Flow "
     "is a trademark of Wispr AI, Inc. We have no relationship with them. The key is yours.",
     "Άδεια MIT. Η μηχανή whisper.cpp και τα μοντέλα έχουν τις δικές τους άδειες. Το Wispr "
     "Flow είναι σήμα κατατεθέν της Wispr AI, Inc. Δεν έχουμε καμία σχέση μαζί τους. Το "
     "πλήκτρο είναι δικό σας."),

    # alt text
    ("The Fuck You Flow home screen: the large-v3 engine on the GPU, the right Alt key for "
     "start and stop, words dictated today and the last five transcripts.",
     "Η αρχική οθόνη του Fuck You Flow: η μηχανή large-v3 στην κάρτα γραφικών, το δεξί Alt για "
     "ξεκίνημα και σταμάτημα, οι λέξεις που υπαγορεύτηκαν σήμερα και οι πέντε τελευταίες "
     "μεταγραφές σε ελληνικά και αγγλικά."),
    ("The History screen: a list of past dictations with the time, word count and wait, and an "
     "edit field for corrections.",
     "Η οθόνη Ιστορικό: μια λίστα με παλιές υπαγορεύσεις, την ώρα, τον αριθμό λέξεων και την "
     "αναμονή και ένα πεδίο για διορθώσεις."),
    ("The Dictionary screen: a table of spoken forms and the written forms they should become.",
     "Η οθόνη Λεξικό: ένας πίνακας με τις προφορικές μορφές και τις γραπτές μορφές που πρέπει "
     "να γίνουν."),
    ("The Statistics screen: words dictated per day, median wait and time saved, shown as "
     "numbers and bars.",
     "Η οθόνη Στατιστικά: λέξεις που υπαγορεύτηκαν ανά μέρα, μέση αναμονή και χρόνος που "
     "γλιτώθηκε, σε νούμερα και μπάρες."),
    ("The Settings screen: the speech model, the graphics card in use, the microphone and the "
     "hotkeys.",
     "Η οθόνη Ρυθμίσεις: το μοντέλο ομιλίας, η κάρτα γραφικών που χρησιμοποιείται, το "
     "μικρόφωνο και τα πλήκτρα συντόμευσης."),
    ("A black keyboard in the dark. The Alt key immediately to the right of the spacebar is "
     "lit from inside in acid green.",
     "Ένα μαύρο πληκτρολόγιο στο σκοτάδι. Το πλήκτρο Alt αμέσως δεξιά από το πλήκτρο διαστήματος "
     "είναι φωτισμένο από μέσα σε έντονο πράσινο."),
    ("A green sound wave on the left breaks into drifting particles that settle into rows of "
     "writing on the right.",
     "Ένα πράσινο ηχητικό κύμα στα αριστερά διαλύεται σε σωματίδια που κατακάθονται σε σειρές "
     "γραφής στα δεξιά."),
    ("A printed receipt on black: a year of Wispr Flow adding up to 180 dollars, the same job "
     "on Fuck You Flow at zero, stamped paid, zero dollars.",
     "Μια τυπωμένη απόδειξη σε μαύρο φόντο: ένας χρόνος Wispr Flow που βγάζει 180 δολάρια, η "
     "ίδια δουλειά με το Fuck You Flow στο μηδέν, με σφραγίδα πληρωμένο, μηδέν δολάρια."),

    # the theme button and the language switch
    ('?"Dark":"Light"', '?"Σκοτεινό":"Φωτεινό"'),
    ('<a href="/el/" hreflang="el" lang="el">ΕΛ</a>',
     '<a href="/?lang=en" hreflang="en" lang="en">EN</a>'),
    ('<a href="/el/" lang="el" hreflang="el">Ελληνικά</a>',
     '<a href="/?lang=en" lang="en" hreflang="en">English</a>'),
]


def rebuild_faq(html, faq):
    """Rewrite the visible questions from the same list the structured data uses.

    Google drops FAQ rich results when the JSON-LD and the visible text differ, and by
    hand they always drift. Here one list feeds both, so they cannot.
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
    write(OUT_EL, el)

    check_greek(el)
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
        "today": datetime.date.today().strftime("%-d %B %Y")
        if os.name != "nt" else datetime.date.today().strftime("%d %B %Y").lstrip("0"),
    }
    write(OUT_LLMS, llms_source.LLMS.format(**fields))
    write(OUT_LLMS_FULL, llms_source.LLMS_FULL.format(**fields))
    write_sitemap()
    # the old files carried a dead download link, so the check is worth keeping
    for path in (OUT_LLMS, OUT_LLMS_FULL):
        body = io.open(path, encoding="utf-8").read()
        stale = re.findall(r"v?0\.9\.1|Setup\.0\.9\.1", body)
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
    """The sitemap is generated too, so lastmod cannot be left behind by an edit."""
    today = datetime.date.today().isoformat()
    log_alt = ('    <xhtml:link rel="alternate" hreflang="en" href="{b}/changelog/"/>\n'
               '    <xhtml:link rel="alternate" hreflang="el" href="{b}/el/changelog/"/>\n'
               '    <xhtml:link rel="alternate" hreflang="x-default" href="{b}/changelog/"/>\n').format(b=BASE)
    urls = [(BASE + "/", ALT), (BASE + "/el/", ALT),
            (BASE + "/wispr-flow-alternative/", ""),
            (BASE + "/changelog/", log_alt), (BASE + "/el/changelog/", log_alt)]
    body = "".join(
        "  <url>\n    <loc>%s</loc>\n    <lastmod>%s</lastmod>\n%s  </url>\n" % (u, today, a)
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
