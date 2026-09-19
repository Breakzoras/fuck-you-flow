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
    ("links", [("/path/", "descriptive label")])

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
            "title": "Fix Slow Offline Dictation: Whisper GPU Memory",
            "description": "Troubleshoot slow local dictation in FU Flow. Check GPU memory, compare bundled Whisper models and understand the limits of the RTX 3070 measurements.",
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
                ("note", "The project observed waits rising from about 1 second to 20 to 37 seconds "
                         "while most of the model was outside GPU memory. This is one hardware "
                         "case, not a universal slowdown factor. Memory bandwidth alone does not "
                         "predict end-to-end recognition time."),
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
                        "At 100 per cent, the model fits in GPU memory. That avoids this memory "
                        "spillover problem, but does not guarantee a particular processing speed.",
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
                      "and can also contribute. In this case, Windows moved the speech model back "
                      "onto the card once there was room, without restarting. Check the model bar "
                      "and time another dictation to confirm the result on your machine."),
                ("h2", "2. Use a model that fits in what is left"),
                ("p", "When the card has to stay busy, a smaller model on the card beats a large one "
                      "in system RAM by a wide margin. The button under the rail switches to the "
                      "most accurate model already installed that fits."),
                ("table", *MODEL_TABLE["en"]),
                ("p", "In the project's Greek test clips, medium used less memory than large-v3 "
                      "with a higher word error rate. Turbo had the lowest median processing time "
                      "and highest error rate of these three models on that corpus. These rounded "
                      "results do not predict accuracy for every speaker or document."),
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
                ("p", "Adding system RAM does not increase dedicated GPU memory. In this observed "
                      "case, freeing GPU memory or selecting a smaller model addressed the issue. "
                      "Other causes of slow dictation need separate diagnosis."),
            ],
        },
        "el": {
            "title": "Αργή υπαγόρευση: μνήμη κάρτας και μοντέλα Whisper",
            "description": "Δείτε γιατί αργεί η τοπική υπαγόρευση στο FU Flow, πώς ελέγχετε τη μνήμη της κάρτας και πότε βοηθά ένα μικρότερο μοντέλο Whisper.",
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
                ("note", "Στη δοκιμή του έργου η αναμονή ανέβηκε από περίπου 1 σε 20 έως 37 "
                         "δευτερόλεπτα όσο το μεγαλύτερο μέρος του μοντέλου βρισκόταν έξω από "
                         "τη μνήμη της κάρτας. Είναι συγκεκριμένη περίπτωση υπολογιστή. Το εύρος "
                         "ζώνης της μνήμης μόνο του δεν προβλέπει τον συνολικό χρόνο αναγνώρισης."),
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
                        "εκεί. Στο 100 τοις εκατό το μοντέλο χωράει στη μνήμη της κάρτας. Αυτό "
                        "αποφεύγει το συγκεκριμένο πρόβλημα μεταφοράς στη RAM, αλλά δεν εγγυάται "
                        "συγκεκριμένη ταχύτητα επεξεργασίας.",
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
                      "μπορούν επίσης να συμβάλουν. Στη συγκεκριμένη δοκιμή τα Windows επέστρεψαν "
                      "το μοντέλο στην κάρτα μόλις υπήρξε χώρος, χωρίς επανεκκίνηση. Ελέγξτε την "
                      "μπάρα του μοντέλου και χρονομετρήστε νέα υπαγόρευση στον δικό σας υπολογιστή."),
                ("h2", "2. Βάλτε μοντέλο που χωράει σε ό,τι έμεινε"),
                ("p", "Όταν η κάρτα πρέπει να μείνει απασχολημένη, ένα μικρό μοντέλο πάνω στην κάρτα "
                      "κερδίζει ένα μεγάλο στη μνήμη του υπολογιστή με μεγάλη διαφορά. Το κουμπί "
                      "κάτω από τη στήλη αλλάζει στο πιο ακριβές μοντέλο που είναι ήδη "
                      "εγκατεστημένο και χωράει."),
                ("table", *MODEL_TABLE["el"]),
                ("p", "Στα ελληνικά αποσπάσματα του έργου, το medium χρησιμοποίησε λιγότερη "
                      "μνήμη από το large-v3 με μεγαλύτερο ποσοστό λαθών. Το turbo είχε τον "
                      "χαμηλότερο διάμεσο χρόνο και το υψηλότερο ποσοστό λαθών από τα τρία "
                      "μοντέλα σε αυτό το δείγμα. Αυτά τα στρογγυλοποιημένα αποτελέσματα δεν "
                      "προβλέπουν την ακρίβεια για κάθε ομιλητή ή κείμενο."),
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
                ("p", "Η προσθήκη RAM δεν αυξάνει την αποκλειστική μνήμη της κάρτας γραφικών. "
                      "Στη συγκεκριμένη περίπτωση βοήθησε η απελευθέρωση μνήμης στην κάρτα ή "
                      "η επιλογή μικρότερου μοντέλου. Άλλες αιτίες αργής υπαγόρευσης χρειάζονται "
                      "ξεχωριστή διάγνωση."),
            ],
        },
    },
    {
        "slug": "the-key",
        "en": {
            "title": "Windows Dictation Hotkey: Start, Stop and Cancel",
            "description": "Use right Alt to start and stop FU Flow dictation on Windows. Cancel with Escape and troubleshoot a hotkey that does not respond.",
            "lead": "One key starts the dictation, the same key ends it, and Escape throws it away.",
            "blocks": [
                ("p", "The key is the Alt on the right of the spacebar, the one marked Alt Gr on "
                      "some keyboards. Press it and the app starts listening. Press it again and "
                      "the text lands in whatever window had the cursor. Escape while it is "
                      "listening throws the recording away and inserts nothing."),
                ("p", "With the default local speech provider, recognition stays on your PC. "
                      "An optional remote provider changes that behavior. The destination "
                      "application receives the text when you insert it."),
                ("h2", "If the key does nothing"),
                ("p", "Open Diagnostics from the menu on the left. It lists every key the app has "
                      "seen. If pressing the key writes nothing there, another program has claimed "
                      "it first, and the usual suspects are gaming keyboard software and remote "
                      "desktop tools. Settings lets you pick a different key."),
            ],
        },
        "el": {
            "title": "Πλήκτρο υπαγόρευσης: έναρξη, τέλος και ακύρωση",
            "description": "Ξεκινήστε και σταματήστε την υπαγόρευση στο FU Flow με το δεξί Alt. Ακύρωση με Escape και λύσεις όταν το πλήκτρο δεν ανταποκρίνεται.",
            "lead": "Ένα πλήκτρο ξεκινάει την υπαγόρευση, το ίδιο πλήκτρο την τελειώνει και το "
                    "Escape την πετάει.",
            "blocks": [
                ("p", "Το πλήκτρο είναι το Alt στα δεξιά του πλήκτρου διαστήματος, αυτό που σε "
                      "μερικά πληκτρολόγια γράφει Alt Gr. Το πατάτε και η εφαρμογή αρχίζει να "
                      "ακούει. Το ξαναπατάτε και το κείμενο προσγειώνεται στο παράθυρο που είχε τον "
                      "κέρσορα. Το Escape όσο ακούει πετάει την ηχογράφηση και δεν γράφει τίποτα."),
                ("p", "Με την προεπιλεγμένη τοπική μηχανή, η αναγνώριση μένει στον υπολογιστή "
                      "σας. Η επιλογή απομακρυσμένου παρόχου αλλάζει αυτή τη λειτουργία. Η "
                      "εφαρμογή προορισμού λαμβάνει το κείμενο όταν το εισάγετε."),
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
            "title": "Fix Dictation Spelling with a Custom Dictionary",
            "description": "Correct names and recurring speech recognition mistakes in FU Flow. Use Dictionary, History suggestions and snippets for Greek and English dictation.",
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
            "title": "Διορθώσεις υπαγόρευσης με προσωπικό λεξικό",
            "description": "Διορθώστε ονόματα και επαναλαμβανόμενα λάθη αναγνώρισης στο FU Flow με το Λεξικό, τις προτάσεις του Ιστορικού και τα Αποσπάσματα.",
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
            "title": "Dictation Text Not Appearing? Fix Windows Paste",
            "description": "Find missing dictation text in FU Flow History, paste manually and check focus or application permissions when Windows insertion fails.",
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
            "title": "Δεν εμφανίζεται το κείμενο της υπαγόρευσης;",
            "description": "Βρείτε τη μεταγραφή στο Ιστορικό του FU Flow, δοκιμάστε χειροκίνητη επικόλληση και ελέγξτε εστίαση και δικαιώματα εφαρμογών στα Windows.",
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
            "title": "Greek and English Dictation: Language Settings",
            "description": "Choose the speech language in FU Flow, dictate Greek text with English terms and understand when to switch the local Whisper language setting.",
            "lead": "Choose a language, try a mixed Greek and English sentence, and check names and punctuation before sharing the text.",
            "blocks": [
                ("p", "In Settings, choose Greek for mainly Greek speech or English for mainly English speech. Automatic detection is available when you prefer the model to infer the language. The tradeoff depends on the speech and model; a fixed language does not guarantee a more accurate transcript."),
                ("p", "Set it to the language you speak most and change it on the days you do not. "
                      "The interface language is a separate setting, so an English interface can "
                      "take Greek dictation."),
                ("h2", "Mixed sentences"),
                ("p", "An English name inside a Greek sentence can work with Greek selected, but recognition may change its spelling or translate it. If you switch to a full English paragraph, select English and compare the result."),
                ("h2", "Try your real vocabulary"),
                ("ol", ["Put the cursor in a blank Word document or another editable text field. Select your microphone and the local speech engine.",
                        "Select Greek and say: Στείλε το brief στο Slack για review. This is an example to try, not a promised recognition result.",
                        "Press right Alt to stop, then check the Greek words and the spelling of brief, Slack and review. Add a Dictionary correction for a recurring mistake.",
                        "Select English for a full English paragraph. Keep a reference sentence so you can count corrections instead of judging from memory."]),
                ("h2", "Greek question marks and spoken punctuation"),
                ("p", "FU Flow applies text cleanup after speech recognition. These examples are existing source-test cases, not recordings or accuracy measurements. They depend on the recognizer first producing the words shown."),
                ("table", ["Recognized text before cleanup", "Result from the text rule"], [
                    ["Είσαι σίγουρος ερωτηματικό Πάμε.", "Είσαι σίγουρος; Πάμε."],
                    ["Are you sure question mark", "Are you sure?"],
                    ["Τέλεια θαυμαστικό", "Τέλεια!"],
                    ["Το ερωτηματικό είναι σημείο στίξης.", "Το ερωτηματικό είναι σημείο στίξης."],
                    ["Μου είπε τι ώρα είναι.", "Μου είπε τι ώρα είναι."]]),
                ("p", "The punctuation word can remain when used as a noun, as in the fourth row. An indirect question can remain a statement, as in the fifth. The optional intonation heuristic is separate from these text rules and can misfire. Review punctuation when it changes the meaning."),
                ("links", [("https://github.com/Breakzoras/fuck-you-flow/blob/main/src-tauri/src/cleanup/questions.rs", "Read the punctuation rules and existing test cases")]),
                ("h2", "What our language-setting measurement actually showed"),
                ("p", "A recorded project experiment from 7 September 2026 used the same 12 Greek clips, an RTX 3070, large-v3-q5_0 and beam size 5. Mean request processing time was 1,129.5 ms with automatic language detection and 909.2 ms with Greek fixed. Mean word error rates were 22.18% and 22.88%, respectively. This sample showed a time saving, not an accuracy improvement."),
                ("note", "These are maker observations, recalculated from the saved results on 8 September, not a new microphone test or a competitor benchmark. Request processing time does not include the complete hotkey-to-paste workflow. Other models, hardware, languages and recordings can behave differently."),
                ("links", [("/assets/language-settings-observations.json", "Download the measurement settings and per-clip numbers")]),
            ],
        },
        "el": {
            "title": "Ελληνική και αγγλική υπαγόρευση: ρυθμίσεις γλώσσας",
            "description": "Ρυθμίστε τη γλώσσα υπαγόρευσης στο FU Flow για ελληνικά, αγγλικά και ανάμεικτους όρους. Η γλώσσα της εφαρμογής επιλέγεται ξεχωριστά.",
            "lead": "Επιλέξτε γλώσσα, δοκιμάστε μια πρόταση με ελληνικά και αγγλικά και ελέγξτε ονόματα και στίξη πριν μοιραστείτε το κείμενο.",
            "blocks": [
                ("p", "Στις Ρυθμίσεις, επιλέξτε Ελληνικά όταν μιλάτε κυρίως ελληνικά ή Αγγλικά όταν μιλάτε κυρίως αγγλικά. Υπάρχει και αυτόματη ανίχνευση, αν προτιμάτε να αποφασίζει το μοντέλο. Η σταθερή γλώσσα δεν εγγυάται μεγαλύτερη ακρίβεια: το αποτέλεσμα εξαρτάται από την ομιλία και το μοντέλο."),
                ("p", "Βάλτε τη γλώσσα που μιλάτε τις περισσότερες φορές και αλλάξτε την τις μέρες "
                      "που κάνετε κάτι άλλο. Η γλώσσα του περιβάλλοντος είναι ξεχωριστή ρύθμιση, "
                      "οπότε ένα αγγλικό περιβάλλον δέχεται ελληνική υπαγόρευση."),
                ("h2", "Ανάμεικτες προτάσεις"),
                ("p", "Ένα αγγλικό όνομα μέσα σε ελληνική πρόταση μπορεί να αποδοθεί με επιλεγμένα τα Ελληνικά, αλλά η αναγνώριση μπορεί να αλλάξει την ορθογραφία του ή να το μεταφράσει. Για ολόκληρη αγγλική παράγραφο, επιλέξτε Αγγλικά και συγκρίνετε το αποτέλεσμα."),
                ("h2", "Δοκιμάστε τις λέξεις που χρησιμοποιείτε"),
                ("ol", ["Βάλτε τον δρομέα σε κενό έγγραφο Word ή άλλο επεξεργάσιμο πεδίο. Επιλέξτε μικρόφωνο και την τοπική μηχανή ομιλίας.",
                        "Επιλέξτε Ελληνικά και πείτε: Στείλε το brief στο Slack για review. Είναι παράδειγμα για δοκιμή, όχι εγγυημένο αποτέλεσμα αναγνώρισης.",
                        "Πατήστε δεξί Alt για να σταματήσετε και ελέγξτε τις ελληνικές λέξεις και τα brief, Slack, review. Προσθέστε διόρθωση στο Λεξικό για λάθη που επαναλαμβάνονται.",
                        "Για ολόκληρη αγγλική παράγραφο, επιλέξτε Αγγλικά. Κρατήστε γραμμένο το αρχικό κείμενο ώστε να μετράτε τις διορθώσεις αντί να βασίζεστε στη μνήμη."]),
                ("h2", "Ελληνικό ερωτηματικό και προφορική στίξη"),
                ("p", "Το FU Flow καθαρίζει το κείμενο μετά την αναγνώριση ομιλίας. Τα παρακάτω είναι υπάρχοντα παραδείγματα ελέγχου του κώδικα. Δεν είναι ηχογραφήσεις ή μετρήσεις ακρίβειας. Προϋποθέτουν ότι η αναγνώριση έχει ήδη δώσει τις λέξεις που φαίνονται."),
                ("table", ["Αναγνωρισμένο κείμενο πριν τον καθαρισμό", "Αποτέλεσμα του κανόνα"], [
                    ["Είσαι σίγουρος ερωτηματικό Πάμε.", "Είσαι σίγουρος; Πάμε."],
                    ["Are you sure question mark", "Are you sure?"],
                    ["Τέλεια θαυμαστικό", "Τέλεια!"],
                    ["Το ερωτηματικό είναι σημείο στίξης.", "Το ερωτηματικό είναι σημείο στίξης."],
                    ["Μου είπε τι ώρα είναι.", "Μου είπε τι ώρα είναι."]]),
                ("p", "Η λέξη ερωτηματικό μπορεί να παραμείνει όταν χρησιμοποιείται ως ουσιαστικό, όπως στην τέταρτη γραμμή. Μια πλάγια ερώτηση μπορεί να μείνει κατάφαση, όπως στην πέμπτη. Η προαιρετική εκτίμηση από τον τόνο της φωνής είναι ξεχωριστή και μπορεί να κάνει λάθος. Ελέγχετε τη στίξη όταν αλλάζει το νόημα."),
                ("links", [("https://github.com/Breakzoras/fuck-you-flow/blob/main/src-tauri/src/cleanup/questions.rs", "Δείτε τους κανόνες στίξης και τα παραδείγματα ελέγχου στον κώδικα")]),
                ("h2", "Τι έδειξε η δική μας μέτρηση γλώσσας"),
                ("p", "Το καταγεγραμμένο πείραμα του έργου στις 7 Σεπτεμβρίου 2026 χρησιμοποίησε τα ίδια 12 ελληνικά δείγματα, RTX 3070, large-v3-q5_0 και beam size 5. Ο μέσος χρόνος επεξεργασίας αιτήματος ήταν 1.129,5 ms με αυτόματη ανίχνευση και 909,2 ms με σταθερά Ελληνικά. Τα μέσα ποσοστά σφάλματος λέξεων ήταν αντίστοιχα 22,18% και 22,88%. Το δείγμα έδειξε εξοικονόμηση χρόνου, όχι βελτίωση ακρίβειας."),
                ("note", "Πρόκειται για μετρήσεις των δημιουργών, που επανυπολογίστηκαν από τα αποθηκευμένα αποτελέσματα στις 8 Σεπτεμβρίου. Δεν είναι νέα δοκιμή μικροφώνου ή σύγκριση ανταγωνιστών. Ο χρόνος αιτήματος δεν περιλαμβάνει όλη τη διαδικασία από το πλήκτρο μέχρι την επικόλληση. Άλλα μοντέλα, μηχανήματα, γλώσσες και ηχογραφήσεις μπορούν να δώσουν διαφορετικά αποτελέσματα."),
                ("links", [("/assets/language-settings-observations.json", "Κατεβάστε τις ρυθμίσεις και τους αριθμούς ανά δείγμα")]),
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
