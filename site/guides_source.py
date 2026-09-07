# -*- coding: utf-8 -*-
"""The guides: one page per subject, in English and Greek, one list feeding both.

Kept apart from build_site.py so the landing page and the guides can be edited
without stepping on each other. build_site.py imports GUIDES from here and
writes /guides/ and /el/guides/ with the same head, stylesheet and header the
landing page uses.

A block is a tuple whose first item names its kind:

    ("p",     "one paragraph")
    ("h2",    "a heading")
    ("ul",    ["a point", "another point"])
    ("img",   "/assets/7-card-full-el.png", "what a reader who cannot see it needs")
    ("table", ["Head", "Head"], [["cell", "cell"], ["cell", "cell"]])
    ("note",  "a line set apart, for a warning or a measurement")

Every number in here was measured on this project's own hardware on
7 September 2026 and the text says so. Nothing is estimated silently.
"""

# Card and model figures, measured on an NVIDIA RTX 3070 with 8 GB on
# 7 September 2026: each model was started with the flags the app passes and the
# card was read while it ran. Accuracy and median time come from the project's
# own benchmark, eval/bench-summary.md, over the same Greek clips.
MODEL_TABLE = {
    "en": (
        ["Model", "Wants on the card", "Words wrong out of 100", "Median time"],
        [
            ["Whisper large-v3 (q5_0)", "1960 MB", "18", "783 ms"],
            ["Whisper medium (q5_0)", "927 MB", "22", "560 ms"],
            ["Whisper large-v3-turbo (q5_0)", "860 MB", "25", "304 ms"],
        ],
    ),
    "el": (
        ["Μοντέλο", "Θέλει στην κάρτα", "Λάθη στις 100 λέξεις", "Διάμεσος χρόνος"],
        [
            ["Whisper large-v3 (q5_0)", "1960 MB", "18", "783 ms"],
            ["Whisper medium (q5_0)", "927 MB", "22", "560 ms"],
            ["Whisper large-v3-turbo (q5_0)", "860 MB", "25", "304 ms"],
        ],
    ),
}


GUIDES = [
    {
        "slug": "card-memory",
        "featured": True,
        "en": {
            "title": "Why dictation slows down, and how to keep the card fast",
            "lead": ("Dictation on Fuck You Flow went from about one second to over thirty on the "
                     "afternoon of 7 September 2026, on a machine where nothing about the app had "
                     "changed. The graphics card had filled up with other programs, and Windows had "
                     "moved most of the speech model off it. This page explains how to see that "
                     "coming and what to do about it."),
            "blocks": [
                ("h2", "What happens"),
                ("p", "The speech model sits on the graphics card while the app runs. On the machine "
                      "measured here it holds 1960 MB there. The card has 8017 MB of usable memory, "
                      "so at eight in the morning, with little else open, it had room to spare."),
                ("p", "Through the day other programs took that memory: a pixel art editor holding "
                      "4025 MB, an Android emulator holding 1419 MB, a game engine and browsers "
                      "holding a few hundred more. By the afternoon 6160 MB of the card was taken."),
                ("p", "Windows does not refuse a program that no longer fits. It accepts it and "
                      "moves most of its memory out to ordinary system RAM. Reading the card at "
                      "16:51 that day showed 133 MB of the model still on it and 1825 MB sitting in "
                      "system RAM."),
                ("note", "Memory in system RAM reaches the card over the PCIe bus at roughly 16 GB "
                         "per second. Memory on the card is read at roughly 448 GB per second. The "
                         "same work therefore takes about twenty eight times longer, which matches "
                         "the measured drop from 1.0 to between 20 and 37 seconds."),
                ("h2", "How to see it"),
                ("p", "The rail down the right hand side of the app answers the question directly. "
                      "It reads the card every three seconds, and again after every dictation."),
                ("img", "/assets/1-home.png",
                        "The app with the memory rail on the right. The word at the top reads Room "
                        "to spare in green. The card is 26 per cent full and the model bar reads 100 "
                        "per cent on the card."),
                ("p", "Four bars, each one a reading rather than a calculation:"),
                ("ul", ["<b>Full</b> is how much of the card every program together is holding.",
                        "<b>Model on the card</b> is how much of the speech model is still there. "
                        "This is the bar that matters. A model at 100 per cent is running at full "
                        "speed whatever the first bar says.",
                        "<b>Accuracy</b> and <b>Speed</b> describe the model in use, from the "
                        "project's own measurements."]),
                ("p", "When the second bar falls, the word at the top turns red and the rail says "
                      "how many MB were pushed out."),
                ("img", "/assets/7-card-full.png",
                        "The same app with the card full. The word at the top reads Out of room in "
                        "red, the card is 77 per cent full, and the model bar reads 7 per cent with "
                        "a line saying 1825 MB pushed out to system RAM. A button underneath offers "
                        "to drop to a smaller model."),
                ("h2", "What to do about it"),
                ("p", "There are two ways back, and the first one is free."),
                ("h2", "1. Give the card its memory back"),
                ("p", "Close whatever is holding the most. Image editors, game engines, phone "
                      "emulators and 3D tools hold gigabytes. Browsers hold a few hundred MB each "
                      "and are rarely the cause on their own. Windows moves the speech model back "
                      "onto the card by itself once there is room, so the next dictation is already "
                      "faster. Nothing needs restarting."),
                ("h2", "2. Use a model that fits in what is left"),
                ("p", "When the card has to stay busy, a smaller model on the card beats a large one "
                      "in system RAM by a wide margin. The button under the rail switches to the "
                      "most accurate model already installed that fits."),
                ("table", *MODEL_TABLE["en"]),
                ("p", "Medium is the middle choice: it holds less than half of what large-v3 holds "
                      "and gets four more words wrong in a hundred. Turbo is the fastest and the "
                      "least accurate of the three."),
                ("note", "Running on the processor instead is slower than either. The same clip "
                         "that takes 783 ms on the card took 14.5 seconds on a sixteen thread "
                         "processor in this project's own benchmark on 6 September 2026."),
                ("h2", "Two things that do not help"),
                ("p", "Asking Windows to give this program priority does not work. The speech "
                      "engine can be built to ask for the highest memory priority the graphics "
                      "driver offers, and it was tested that way on 7 September 2026: with the "
                      "priority on, the model still ended up with 50 MB on the card and 1582 MB "
                      "outside it, and the timings were unchanged. That setting orders one "
                      "program's own allocations. It takes nothing back from other programs."),
                ("p", "Adding system RAM does not help either. The model is already in system RAM "
                      "when this happens. The bus between there and the card is the limit."),
            ],
        },
        "el": {
            "title": "Γιατί αργεί η υπαγόρευση, και πώς κρατάτε την κάρτα γρήγορη",
            "lead": ("Η υπαγόρευση στο Fuck You Flow πήγε από περίπου ένα δευτερόλεπτο σε πάνω από "
                     "τριάντα, το απόγευμα της 7ης Σεπτεμβρίου 2026, σε υπολογιστή όπου τίποτα μέσα "
                     "στην εφαρμογή δεν είχε αλλάξει. Η κάρτα γραφικών είχε γεμίσει από άλλα "
                     "προγράμματα και τα Windows είχαν μετακινήσει το μεγαλύτερο μέρος του "
                     "μοντέλου φωνής έξω από αυτήν. Εδώ εξηγείται πώς το βλέπετε να έρχεται και τι "
                     "κάνετε."),
            "blocks": [
                ("h2", "Τι συμβαίνει"),
                ("p", "Το μοντέλο φωνής κάθεται πάνω στην κάρτα γραφικών όσο τρέχει η εφαρμογή. Στο "
                      "μηχάνημα που μετρήθηκε πιάνει εκεί 1960 MB. Η κάρτα έχει 8017 MB διαθέσιμης "
                      "μνήμης, οπότε στις οκτώ το πρωί, με λίγα ανοιχτά, υπήρχε άπλετος χώρος."),
                ("p", "Μέσα στη μέρα άλλα προγράμματα πήραν αυτή τη μνήμη: ένας επεξεργαστής "
                      "εικόνας που κρατούσε 4025 MB, ένας εξομοιωτής Android με 1419 MB, μια "
                      "μηχανή παιχνιδιών και φυλλομετρητές με μερικές εκατοντάδες ακόμα. Το "
                      "απόγευμα ήταν πιασμένα 6160 MB της κάρτας."),
                ("p", "Τα Windows δεν αρνούνται σε ένα πρόγραμμα που πια δεν χωράει. Το δέχονται και "
                      "μεταφέρουν το μεγαλύτερο μέρος της μνήμης του στην κοινή μνήμη του "
                      "υπολογιστή. Η μέτρηση της κάρτας στις 16:51 εκείνη τη μέρα έδειξε 133 MB του "
                      "μοντέλου ακόμα πάνω της και 1825 MB στη μνήμη του υπολογιστή."),
                ("note", "Η μνήμη του υπολογιστή φτάνει στην κάρτα με περίπου 16 GB το δευτερόλεπτο. "
                         "Η μνήμη πάνω στην κάρτα διαβάζεται με περίπου 448 GB το δευτερόλεπτο. Η "
                         "ίδια δουλειά παίρνει έτσι περίπου είκοσι οκτώ φορές περισσότερο, που "
                         "συμφωνεί με τη μετρημένη πτώση από 1,0 σε 20 ως 37 δευτερόλεπτα."),
                ("h2", "Πώς το βλέπετε"),
                ("p", "Η στήλη στη δεξιά πλευρά της εφαρμογής απαντά κατευθείαν. Διαβάζει την κάρτα "
                      "κάθε τρία δευτερόλεπτα, και ξανά μετά από κάθε υπαγόρευση."),
                ("img", "/assets/1-home-el.png",
                        "Η εφαρμογή με τη στήλη μνήμης στα δεξιά. Η λέξη στην κορυφή γράφει Άνετα σε "
                        "πράσινο. Η κάρτα είναι γεμάτη κατά 26 τοις εκατό και η μπάρα του μοντέλου "
                        "δείχνει 100 τοις εκατό πάνω στην κάρτα."),
                ("p", "Τέσσερις μπάρες, η καθεμία μέτρηση:"),
                ("ul", ["<b>Γεμάτη</b> είναι πόσο από την κάρτα κρατάνε όλα τα προγράμματα μαζί.",
                        "<b>Το μοντέλο μέσα</b> είναι πόσο από το μοντέλο φωνής βρίσκεται ακόμα "
                        "εκεί. Αυτή είναι η μπάρα που μετράει. Ένα μοντέλο στο 100 τοις εκατό "
                        "τρέχει με πλήρη ταχύτητα, ό,τι κι αν λέει η πρώτη μπάρα.",
                        "<b>Ακρίβεια</b> και <b>Ταχύτητα</b> περιγράφουν το μοντέλο που τρέχει, από "
                        "τις μετρήσεις του ίδιου του έργου."]),
                ("p", "Όταν πέσει η δεύτερη μπάρα, η λέξη στην κορυφή κοκκινίζει και η στήλη λέει "
                      "πόσα MB βγήκαν έξω."),
                ("img", "/assets/7-card-full-el.png",
                        "Η ίδια εφαρμογή με την κάρτα γεμάτη. Η λέξη στην κορυφή γράφει Δεν χωράει "
                        "σε κόκκινο, η κάρτα είναι γεμάτη κατά 77 τοις εκατό και η μπάρα του "
                        "μοντέλου δείχνει 7 τοις εκατό, με γραμμή που λέει 1825 MB βγήκαν έξω. Από "
                        "κάτω ένα κουμπί προσφέρει αλλαγή σε μικρότερο μοντέλο."),
                ("h2", "Τι κάνετε"),
                ("p", "Υπάρχουν δύο δρόμοι πίσω, και ο πρώτος είναι δωρεάν."),
                ("h2", "1. Δώστε στην κάρτα τη μνήμη της πίσω"),
                ("p", "Κλείστε ό,τι κρατάει τα περισσότερα. Επεξεργαστές εικόνας, μηχανές "
                      "παιχνιδιών, εξομοιωτές κινητών και εργαλεία τριών διαστάσεων κρατάνε "
                      "γιγαμπάιτ. Οι φυλλομετρητές κρατάνε μερικές εκατοντάδες MB ο καθένας και "
                      "σπάνια ευθύνονται μόνοι τους. Τα Windows επιστρέφουν το μοντέλο στην κάρτα "
                      "μόνα τους μόλις υπάρξει χώρος, οπότε η επόμενη υπαγόρευση είναι ήδη πιο "
                      "γρήγορη. Καμία επανεκκίνηση δεν χρειάζεται."),
                ("h2", "2. Βάλτε μοντέλο που χωράει σε ό,τι έμεινε"),
                ("p", "Όταν η κάρτα πρέπει να μείνει απασχολημένη, ένα μικρό μοντέλο πάνω στην κάρτα "
                      "κερδίζει ένα μεγάλο στη μνήμη του υπολογιστή με μεγάλη διαφορά. Το κουμπί "
                      "κάτω από τη στήλη αλλάζει στο πιο ακριβές μοντέλο που είναι ήδη "
                      "εγκατεστημένο και χωράει."),
                ("table", *MODEL_TABLE["el"]),
                ("p", "Το medium είναι η μεσαία επιλογή: πιάνει λιγότερο από το μισό απ' ό,τι το "
                      "large-v3 και κάνει τέσσερα λάθη παραπάνω στις εκατό λέξεις. Το turbo είναι "
                      "το πιο γρήγορο και το λιγότερο ακριβές από τα τρία."),
                ("note", "Το τρέξιμο στον επεξεργαστή είναι πιο αργό και από τα δύο. Το ίδιο "
                         "απόσπασμα που παίρνει 783 ms στην κάρτα πήρε 14,5 δευτερόλεπτα σε "
                         "επεξεργαστή δεκαέξι νημάτων, στη μέτρηση του έργου στις 6 Σεπτεμβρίου "
                         "2026."),
                ("h2", "Δύο πράγματα που δεν βοηθάνε"),
                ("p", "Το να ζητήσετε από τα Windows προτεραιότητα για αυτό το πρόγραμμα δεν "
                      "δουλεύει. Η μηχανή φωνής μπορεί να χτιστεί ώστε να ζητάει την ανώτατη "
                      "προτεραιότητα μνήμης που προσφέρει ο οδηγός της κάρτας, και δοκιμάστηκε έτσι "
                      "στις 7 Σεπτεμβρίου 2026: με την προτεραιότητα ανοιχτή, το μοντέλο κατέληξε "
                      "πάλι με 50 MB πάνω στην κάρτα και 1582 MB έξω, και οι χρόνοι έμειναν ίδιοι. "
                      "Η ρύθμιση αυτή ταξινομεί τα κομμάτια ενός προγράμματος μεταξύ τους. Δεν "
                      "παίρνει τίποτα πίσω από άλλα προγράμματα."),
                ("p", "Ούτε η προσθήκη μνήμης στον υπολογιστή βοηθάει. Το μοντέλο βρίσκεται ήδη στη "
                      "μνήμη του υπολογιστή όταν συμβαίνει αυτό. Ο δίαυλος ανάμεσα σε εκείνη και "
                      "στην κάρτα είναι το όριο."),
            ],
        },
    },
    {
        "slug": "the-key",
        "en": {
            "title": "The key, and what each press does",
            "lead": "One key starts the dictation, the same key ends it, and Escape throws it away.",
            "blocks": [
                ("p", "The key is the Alt on the right of the spacebar, the one marked Alt Gr on "
                      "some keyboards. Press it and the app starts listening. Press it again and "
                      "the text lands in whatever window had the cursor. Escape while it is "
                      "listening throws the recording away and inserts nothing."),
                ("p", "Nothing is sent anywhere in between. The recording, the model and the text "
                      "all stay on the machine."),
                ("h2", "If the key does nothing"),
                ("p", "Open Diagnostics from the menu on the left. It lists every key the app has "
                      "seen. If pressing the key writes nothing there, another program has claimed "
                      "it first, and the usual suspects are gaming keyboard software and remote "
                      "desktop tools. Settings lets you pick a different key."),
            ],
        },
        "el": {
            "title": "Το πλήκτρο, και τι κάνει κάθε πάτημα",
            "lead": "Ένα πλήκτρο ξεκινάει την υπαγόρευση, το ίδιο πλήκτρο την τελειώνει και το "
                    "Escape την πετάει.",
            "blocks": [
                ("p", "Το πλήκτρο είναι το Alt στα δεξιά του πλήκτρου διαστήματος, αυτό που σε "
                      "μερικά πληκτρολόγια γράφει Alt Gr. Το πατάτε και η εφαρμογή αρχίζει να "
                      "ακούει. Το ξαναπατάτε και το κείμενο προσγειώνεται στο παράθυρο που είχε τον "
                      "κέρσορα. Το Escape όσο ακούει πετάει την ηχογράφηση και δεν γράφει τίποτα."),
                ("p", "Στο ενδιάμεσο τίποτα δεν φεύγει προς τα έξω. Η ηχογράφηση, το μοντέλο και το "
                      "κείμενο μένουν στο μηχάνημα."),
                ("h2", "Αν το πλήκτρο δεν κάνει τίποτα"),
                ("p", "Ανοίξτε τα Διαγνωστικά από το μενού αριστερά. Καταγράφουν κάθε πλήκτρο που "
                      "είδε η εφαρμογή. Αν το πάτημα δεν γράφει τίποτα εκεί, κάποιο άλλο πρόγραμμα "
                      "το έχει πάρει πρώτο, και οι συνηθισμένοι ύποπτοι είναι τα προγράμματα "
                      "πληκτρολογίων για παιχνίδια και τα εργαλεία απομακρυσμένης σύνδεσης. Οι "
                      "Ρυθμίσεις σας αφήνουν να διαλέξετε άλλο πλήκτρο."),
            ],
        },
    },
    {
        "slug": "words-it-gets-wrong",
        "en": {
            "title": "Teaching it a word it keeps getting wrong",
            "lead": "Names, places and trade terms are what dictation misses most, and the "
                    "Dictionary fixes them once.",
            "blocks": [
                ("p", "Open Dictionary from the menu and add what you hear next to what you want "
                      "written. Every transcript from then on carries the correction."),
                ("p", "The app also watches your own edits. Correct the same word in History twice "
                      "and it appears under Suggestions, ready to become a rule with one click. "
                      "Nothing is added behind your back."),
                ("h2", "Snippets"),
                ("p", "Snippets are the other direction: a short spoken phrase that expands into "
                      "something long you type often, such as an address or a sign off."),
            ],
        },
        "el": {
            "title": "Μαθαίνοντάς του μια λέξη που την πιάνει λάθος",
            "lead": "Ονόματα, τοπωνύμια και όροι της δουλειάς είναι αυτά που χάνει πιο συχνά η "
                    "υπαγόρευση, και το Λεξικό τα λύνει μια φορά.",
            "blocks": [
                ("p", "Ανοίξτε το Λεξικό από το μενού και γράψτε τι ακούγεται δίπλα σε αυτό που "
                      "θέλετε να γράφεται. Από εκεί και πέρα κάθε κείμενο βγαίνει διορθωμένο."),
                ("p", "Η εφαρμογή παρακολουθεί επίσης τις δικές σας διορθώσεις. Διορθώστε την ίδια "
                      "λέξη δύο φορές στο Ιστορικό και εμφανίζεται στις Προτάσεις μάθησης, έτοιμη "
                      "να γίνει κανόνας με ένα κλικ. Τίποτα δεν μπαίνει πίσω από την πλάτη σας."),
                ("h2", "Αποσπάσματα"),
                ("p", "Τα Αποσπάσματα είναι η άλλη κατεύθυνση: μια σύντομη φράση που λέτε και "
                      "ανοίγει σε κάτι μεγάλο που γράφετε συχνά, όπως μια διεύθυνση ή μια "
                      "υπογραφή."),
            ],
        },
    },
    {
        "slug": "when-the-paste-refuses",
        "en": {
            "title": "When a program refuses the text",
            "lead": "A few windows turn the paste down. The text is kept on the clipboard so "
                    "nothing is lost.",
            "blocks": [
                ("p", "The app types into the window that had the cursor by pasting. Some windows "
                      "reject a paste they did not expect: parts of remote desktop sessions, some "
                      "terminals, and windows running with higher privileges than the app."),
                ("p", "When that happens the app says so and leaves the text on the clipboard. "
                      "Press Ctrl and V yourself and it goes in. The transcript is also in History, "
                      "with a Copy button."),
                ("h2", "The privileges case"),
                ("p", "A window opened as an administrator will not take input from a program that "
                      "was not. Starting Fuck You Flow as an administrator too puts them on the "
                      "same footing."),
            ],
        },
        "el": {
            "title": "Όταν ένα πρόγραμμα αρνείται το κείμενο",
            "lead": "Μερικά παράθυρα απορρίπτουν την επικόλληση. Το κείμενο μένει στο πρόχειρο, "
                    "οπότε τίποτα δεν χάνεται.",
            "blocks": [
                ("p", "Η εφαρμογή γράφει στο παράθυρο που είχε τον κέρσορα κάνοντας επικόλληση. "
                      "Κάποια παράθυρα απορρίπτουν μια επικόλληση που δεν περίμεναν: κομμάτια "
                      "συνεδριών απομακρυσμένης σύνδεσης, μερικά τερματικά, και παράθυρα που "
                      "τρέχουν με περισσότερα δικαιώματα από την εφαρμογή."),
                ("p", "Όταν συμβεί, η εφαρμογή σας το λέει και αφήνει το κείμενο στο πρόχειρο. "
                      "Πατάτε εσείς Ctrl και V και μπαίνει. Το κείμενο βρίσκεται επίσης στο "
                      "Ιστορικό, με κουμπί αντιγραφής."),
                ("h2", "Η περίπτωση των δικαιωμάτων"),
                ("p", "Ένα παράθυρο που άνοιξε ως διαχειριστής δεν δέχεται πληκτρολόγηση από "
                      "πρόγραμμα που δεν άνοιξε έτσι. Ξεκινώντας και το Fuck You Flow ως "
                      "διαχειριστής, τα δύο βρίσκονται στο ίδιο επίπεδο."),
            ],
        },
    },
    {
        "slug": "greek-and-english",
        "en": {
            "title": "Greek, English, and switching between them",
            "lead": "Naming the language you are about to speak is measurably faster than letting "
                    "the model work it out.",
            "blocks": [
                ("p", "Settings holds the dictation language. Set to a single language, the model is "
                      "told what to expect. Left on automatic, it spends an extra pass on every "
                      "request deciding, which this project measured at 220 ms per request on "
                      "7 September 2026."),
                ("p", "Set it to the language you speak most and change it on the days you do not. "
                      "The interface language is a separate setting, so an English interface can "
                      "take Greek dictation."),
                ("h2", "Mixed sentences"),
                ("p", "Technical words in English inside a Greek sentence come through with the "
                      "language fixed to Greek. A whole paragraph in the other language is the case "
                      "that needs the switch."),
            ],
        },
        "el": {
            "title": "Ελληνικά, αγγλικά, και η εναλλαγή τους",
            "lead": "Το να δηλώσετε τη γλώσσα που πρόκειται να μιλήσετε είναι μετρήσιμα πιο γρήγορο "
                    "από το να την ψάχνει το μοντέλο.",
            "blocks": [
                ("p", "Οι Ρυθμίσεις κρατάνε τη γλώσσα υπαγόρευσης. Όταν είναι σε μία γλώσσα, το "
                      "μοντέλο ξέρει τι να περιμένει. Όταν μένει στο αυτόματο, ξοδεύει ένα επιπλέον "
                      "πέρασμα σε κάθε αίτημα για να αποφασίσει, που το έργο μέτρησε στα 220 ms ανά "
                      "αίτημα στις 7 Σεπτεμβρίου 2026."),
                ("p", "Βάλτε τη γλώσσα που μιλάτε τις περισσότερες φορές και αλλάξτε την τις μέρες "
                      "που κάνετε κάτι άλλο. Η γλώσσα του περιβάλλοντος είναι ξεχωριστή ρύθμιση, "
                      "οπότε ένα αγγλικό περιβάλλον δέχεται ελληνική υπαγόρευση."),
                ("h2", "Ανάμεικτες προτάσεις"),
                ("p", "Τεχνικές λέξεις στα αγγλικά μέσα σε ελληνική πρόταση περνάνε κανονικά με τη "
                      "γλώσσα κλειδωμένη στα ελληνικά. Ολόκληρη παράγραφος στην άλλη γλώσσα είναι η "
                      "περίπτωση που θέλει την εναλλαγή."),
            ],
        },
    },
]

INDEX_TEXT = {
    "en": {
        "title": "Guides",
        "lead": "Short pages on getting the most out of Fuck You Flow.",
    },
    "el": {
        "title": "Οδηγοί",
        "lead": "Σύντομες σελίδες για να βγάλετε τα περισσότερα από το Fuck You Flow.",
    },
}
