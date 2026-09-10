# -*- coding: utf-8 -*-
"""What changed in every released version, in both languages.

One entry per release, newest first. Adding a release means adding a dict here
and running build_changelog.py; nothing else knows the history.

Each line is a pair: the English sentence and the Greek one. Keep them saying
the same thing. House rules apply to both: no em-dash, no emoji, and in Greek
no comma before the word "και".
"""

# The three kinds of line, and how each is titled in each language.
KINDS = {
    "added": ("New", "Νέα"),
    "fixed": ("Fixed", "Διορθώθηκαν"),
    "changed": ("Changed", "Άλλαξαν"),
}

RELEASES = [
    {
        "version": "0.9.4",
        "date": "2026-09-10",
        "summary": (
            "A key check for the days the hotkey seems dead, and the startup entry stays "
            "with the installed copy.",
            "Ένας έλεγχος πλήκτρου για τις μέρες που το πλήκτρο μοιάζει νεκρό και η αυτόματη "
            "εκκίνηση που μένει στο εγκατεστημένο αντίγραφο.",
        ),
        "lines": [
            ("added",
             "Diagnostics, Key check: press your hotkey and see live what Windows delivered. "
             "When the left Alt arrives while hands-free sits on the right one, it says so.",
             "Διαγνωστικά, Έλεγχος πλήκτρου: πατάτε το πλήκτρο σας και βλέπετε ζωντανά τι "
             "παρέδωσαν τα Windows. Όταν φτάνει το αριστερό Alt ενώ τα ελεύθερα χέρια είναι "
             "στο δεξί, το λέει."),
            ("fixed",
             "A copy run from the build folder no longer takes over the Windows startup entry. "
             "Only the installed copy does.",
             "Ένα αντίγραφο που τρέχει από τον φάκελο χτισίματος δεν παίρνει πια την αυτόματη "
             "εκκίνηση των Windows. Μόνο το εγκατεστημένο αντίγραφο την παίρνει."),
            ("fixed",
             "A dictation that ended with the tray icon in front is now refused out loud, the "
             "way the desktop and the taskbar are, with a message that names the tray icon.",
             "Μια υπαγόρευση που τελείωσε με το εικονίδιο της γραμμής εργασιών μπροστά "
             "απορρίπτεται πλέον φωναχτά, όπως η επιφάνεια εργασίας και η γραμμή εργασιών, "
             "με μήνυμα που κατονομάζει το εικονίδιο."),
        ],
    },
    {
        "version": "0.9.3",
        "date": "2026-09-08",
        "summary": (
            "Two small repairs on the update itself, found the same night 0.9.2 went out.",
            "Δύο μικρές διορθώσεις πάνω στην ίδια την ενημέρωση, που βρέθηκαν το ίδιο βράδυ "
            "που βγήκε η 0.9.2.",
        ),
        "lines": [
            ("fixed",
             "The bar that announces a new version said the word version twice.",
             "Η μπάρα που ανακοινώνει νέα έκδοση έλεγε τη λέξη έκδοση δύο φορές."),
            ("fixed",
             "The check that runs on its own fifteen seconds after a window opens could land "
             "in the middle of a download and swap the version being installed. It now waits "
             "for the install to finish.",
             "Ο έλεγχος που τρέχει μόνος του δεκαπέντε δευτερόλεπτα αφού ανοίξει ένα παράθυρο "
             "μπορούσε να πέσει μέσα σε ένα κατέβασμα και να αλλάξει την έκδοση που "
             "εγκαθίσταται. Τώρα περιμένει να τελειώσει η εγκατάσταση."),
        ],
    },
    {
        "version": "0.9.2",
        "date": "2026-09-08",
        "summary": (
            "Updates from inside the app, an existing recording turned into text, and the "
            "crash that left nothing behind.",
            "Ενημερώσεις μέσα από την εφαρμογή, ηχογράφηση που γίνεται κείμενο και το "
            "σφάλμα που έριχνε το πρόγραμμα χωρίς να αφήνει ίχνος.",
        ),
        "lines": [
            ("added",
             "The app asks fuckyouflow.app once at every start and shows a bar when a newer "
             "version is out. Nothing is downloaded until you press the button on it.",
             "Η εφαρμογή ρωτάει το fuckyouflow.app μία φορά σε κάθε άνοιγμα και βγάζει μια "
             "μπάρα όταν υπάρχει νεότερη έκδοση. Δεν κατεβαίνει τίποτα μέχρι να πατήσετε το κουμπί."),
            ("added",
             "A recording you already have can be turned into text: Home, Sound file to text. "
             "Up to 90 minutes, cut into pieces at the quiet parts.",
             "Μια ηχογράφηση που έχετε ήδη γίνεται κείμενο: Αρχική, Αρχείο ήχου σε κείμενο. "
             "Έως 90 λεπτά, κομμένη στα σημεία που υπάρχει σιωπή."),
            ("added",
             "When a dictation cannot reach the program you were in, a small window opens with "
             "the words, so nothing is lost. It has a switch at the bottom to turn it off, and "
             "the same switch lives in Settings, Insertion.",
             "Όταν μια υπαγόρευση δεν καταφέρνει να μπει στο πρόγραμμα που ήσασταν, ανοίγει ένα "
             "μικρό παράθυρο με τα λόγια, ώστε να μη χαθεί τίποτα. Έχει διακόπτη στο κάτω μέρος "
             "για να κλείνει και ο ίδιος διακόπτης βρίσκεται στις Ρυθμίσεις, Εισαγωγή κειμένου."),
            ("added",
             "A graphics card that was left switched off is switched on by itself when the app "
             "finds one with a working Vulkan runtime.",
             "Μια κάρτα γραφικών που είχε μείνει κλειστή ανοίγει μόνη της, όταν η εφαρμογή βρει "
             "κάρτα με Vulkan που δουλεύει."),
            ("fixed",
             "The app could die on the spot while reading the clipboard, leaving no message "
             "anywhere. It looked for the end of the text as far as 50 million characters ahead, "
             "so text placed by another program without a closing marker sent the read past the "
             "end of the memory block. It now reads only as far as the block goes.",
             "Η εφαρμογή μπορούσε να πεθάνει ακαριαία όσο διάβαζε το πρόχειρο, χωρίς να αφήσει "
             "μήνυμα πουθενά. Έψαχνε το τέλος του κειμένου έως και 50 εκατομμύρια χαρακτήρες "
             "μπροστά, οπότε κείμενο που είχε βάλει άλλο πρόγραμμα χωρίς σημάδι τέλους έστελνε "
             "το διάβασμα έξω από το κομμάτι μνήμης. Τώρα διαβάζει μόνο όσο φτάνει το κομμάτι."),
            ("fixed",
             "The Windows startup entry named whatever copy of the program was running the last "
             "time the settings were saved. It is now written at every start.",
             "Η καταχώρηση εκκίνησης των Windows έδειχνε όποιο αντίγραφο του προγράμματος έτρεχε "
             "την τελευταία φορά που αποθηκεύτηκαν οι ρυθμίσεις. Τώρα γράφεται σε κάθε άνοιγμα."),
            ("fixed",
             "The small window with the words had no permission to talk to the app, so its "
             "buttons did nothing.",
             "Το μικρό παράθυρο με τα λόγια δεν είχε άδεια να μιλήσει με την εφαρμογή, οπότε τα "
             "κουμπιά του δεν έκαναν τίποτα."),
            ("fixed",
             "That same window was painted dark whatever theme you had chosen.",
             "Το ίδιο παράθυρο ήταν σκούρο όποιο θέμα και αν είχατε διαλέξει."),
            ("fixed",
             "The Settings page threw away what you were typing if the app changed a setting at "
             "the same moment.",
             "Η σελίδα Ρυθμίσεις πετούσε ό,τι γράφατε, αν η εφαρμογή άλλαζε μια ρύθμιση την ίδια "
             "στιγμή."),
            ("fixed",
             "A crash used to leave nothing behind. Panics are written straight to "
             "logs\\panic.log, and a normal exit writes its own line, so the two can be told "
             "apart afterwards.",
             "Ένα κρασάρισμα δεν άφηνε τίποτα πίσω του. Τα σφάλματα γράφονται κατευθείαν στο "
             "logs\\panic.log και ένα κανονικό κλείσιμο γράφει τη δική του γραμμή, ώστε να "
             "ξεχωρίζουν τα δύο μετά."),
            ("changed",
             "The installer carries its files uncompressed. The download is 1.7 GB and the "
             "install writes the files straight out.",
             "Ο installer κουβαλάει τα αρχεία του ασυμπίεστα. Το κατέβασμα είναι 1,7 GB και η "
             "εγκατάσταση γράφει τα αρχεία κατευθείαν."),
            ("changed",
             "The speech models move to your own data folder the first time the app runs. That is "
             "what makes a 70 MB update possible, and they stay there when you uninstall.",
             "Τα μοντέλα ομιλίας μετακομίζουν στον δικό σας φάκελο δεδομένων την πρώτη φορά που "
             "ανοίγει η εφαρμογή. Αυτό κάνει εφικτή μια ενημέρωση των 70 MB και μένουν εκεί όταν "
             "απεγκαταστήσετε."),
        ],
    },
    {
        "version": "0.9.1",
        "date": "2026-09-06",
        "summary": (
            "A paste that goes nowhere is reported as one, and the interface stops being Greek "
            "on the inside.",
            "Μια επικόλληση που δεν πάει πουθενά αναφέρεται ως τέτοια και το περιβάλλον σταματάει "
            "να είναι ελληνικό από μέσα.",
        ),
        "lines": [
            ("fixed",
             "The desktop and the taskbar read the clipboard when they receive Ctrl+V, so a "
             "transcript sent there looked like a success in every signal the app had while you "
             "saw nothing arrive. The app now stops before the keystroke, leaves the text on the "
             "clipboard and says which window was in front.",
             "Η επιφάνεια εργασίας και η γραμμή εργασιών διαβάζουν το πρόχειρο όταν δεχτούν "
             "Ctrl+V, οπότε ένα κείμενο που πήγαινε εκεί έμοιαζε επιτυχία σε κάθε ένδειξη που "
             "είχε η εφαρμογή, ενώ εσείς δεν βλέπατε τίποτα. Τώρα σταματάει πριν το πάτημα, "
             "αφήνει το κείμενο στο πρόχειρο και λέει ποιο παράθυρο ήταν μπροστά."),
            ("fixed",
             "The model descriptions were written in Greek inside the code, so an English "
             "interface still showed Greek text, and an unread settings file fell back to Greek.",
             "Οι περιγραφές των μοντέλων ήταν γραμμένες στα ελληνικά μέσα στον κώδικα, οπότε ένα "
             "αγγλικό περιβάλλον έδειχνε ελληνικά και ένα αρχείο ρυθμίσεων που δεν διαβαζόταν "
             "γύριζε στα ελληνικά."),
            ("added",
             "A fresh install reads the language Windows is set to, once, before there are any "
             "settings to overwrite. Both the interface language and the dictation language stay "
             "changeable under Settings, Language.",
             "Μια καθαρή εγκατάσταση διαβάζει τη γλώσσα των Windows, μία φορά, πριν υπάρξουν "
             "ρυθμίσεις να χαλάσει. Και η γλώσσα του περιβάλλοντος και η γλώσσα υπαγόρευσης "
             "αλλάζουν από τις Ρυθμίσεις, Γλώσσα."),
            ("added",
             "Settings, Speech models says which device the engine is running on right now and "
             "splits its settings into what works the graphics card and what works the processor.",
             "Οι Ρυθμίσεις, Μοντέλα ομιλίας λένε σε ποια συσκευή τρέχει η μηχανή αυτή τη στιγμή "
             "και χωρίζουν τις ρυθμίσεις σε όσες δουλεύουν την κάρτα γραφικών και όσες δουλεύουν "
             "τον επεξεργαστή."),
            ("added",
             "An event journal is written next to the logs, one line per event, seven days kept. "
             "It records what happened and what failed, and never the text you dictate.",
             "Ένα ημερολόγιο συμβάντων γράφεται δίπλα στα αρχεία καταγραφής, μία γραμμή ανά "
             "συμβάν, με επτά μέρες ιστορικό. Καταγράφει τι έγινε και τι απέτυχε και ποτέ το "
             "κείμενο που υπαγορεύετε."),
        ],
    },
    {
        "version": "0.9.0",
        "date": "2026-09-06",
        "summary": (
            "The first public release: one key, both languages, everything on your own machine.",
            "Η πρώτη δημόσια έκδοση: ένα πλήκτρο, δύο γλώσσες και τα πάντα στο δικό σας μηχάνημα.",
        ),
        "lines": [
            ("added",
             "Hold the right Alt, speak, press it again. The text lands where your cursor is, in "
             "any program. Greek and English, mixed in the same sentence.",
             "Κρατάτε το δεξί Alt, μιλάτε, το πατάτε ξανά. Το κείμενο προσγειώνεται εκεί που "
             "είναι ο κέρσορας, σε οποιοδήποτε πρόγραμμα. Ελληνικά και αγγλικά, ανακατεμένα στην "
             "ίδια πρόταση."),
            ("added",
             "The installer carries everything: the speech engine built with the Vulkan backend, "
             "which accelerates on AMD, Intel and NVIDIA cards alike, both speech models and the "
             "voice activity detector. Nothing is downloaded later.",
             "Ο installer τα κουβαλάει όλα: τη μηχανή ομιλίας με Vulkan, που επιταχύνει σε κάρτες "
             "AMD, Intel και NVIDIA το ίδιο. Μαζί και τα δύο μοντέλα ομιλίας και ο ανιχνευτής φωνής. "
             "Τίποτα δεν κατεβαίνει αργότερα."),
            ("added",
             "The first start reads the machine and picks the backend, the model that fits the "
             "graphics memory and the thread count that fits the processor.",
             "Το πρώτο άνοιγμα διαβάζει το μηχάνημα και διαλέγει τη μηχανή, το μοντέλο που χωράει "
             "στη μνήμη της κάρτας και τον αριθμό των πυρήνων που ταιριάζει στον επεξεργαστή."),
        ],
    },
]
