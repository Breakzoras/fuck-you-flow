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
    "known": ("Known issues", "Γνωστά προβλήματα"),
    "added": ("New", "Νέα"),
    "fixed": ("Fixed", "Διορθώθηκαν"),
    "changed": ("Changed", "Άλλαξαν"),
}

RELEASES = [
{'version': '0.9.11',
 'date': '2026-09-25',
 'summary': ('A new look with depth called Carbon, and your words stay one Ctrl+V away after pasting into a '
             'browser, Claude or Slack.',
             'Νέα εμφάνιση με βάθος με το όνομα Carbon και οι λέξεις σου μένουν ένα Ctrl+V μακριά μετά την επικόλληση σε '
             'browser, Claude ή Slack.'),
 'lines': [('added',
            'Carbon, the new default look of the dark theme: panels with depth, teal edges and a dark red glow in '
            'the middle, made from still gradients and shadows so the graphics card does no extra work. The '
            'classic flat black stays one choice away in Settings, General.',
            'Carbon, η νέα προεπιλεγμένη εμφάνιση του σκούρου θέματος: πάνελ με βάθος, πρασινογάλαζα περιγράμματα '
            'και μια σκούρα κόκκινη λάμψη στη μέση, φτιαγμένα από σταθερά ντεγκραντέ και σκιές, ώστε η κάρτα '
            'γραφικών να μην κάνει καμία επιπλέον δουλειά. Η κλασική επίπεδη μαύρη εμφάνιση μένει μία επιλογή '
            'μακριά, στις Ρυθμίσεις, Γενικά.'),
           ('fixed',
            'In Chrome, Edge and programs built on them such as Claude and Slack, the words stay on the '
            'clipboard after pasting. If they do not show up where you were typing, one Ctrl+V brings them '
            'back. In these programs the earlier clipboard content is no longer put back, even with the '
            'restore setting on.',
            'Στο Chrome, στον Edge και σε προγράμματα χτισμένα πάνω τους, όπως το Claude και το Slack, οι '
            'λέξεις μένουν στο πρόχειρο μετά την επικόλληση. Αν δεν εμφανιστούν εκεί που έγραφες, ένα '
            'Ctrl+V τις φέρνει πίσω. Σε αυτά τα προγράμματα το προηγούμενο περιεχόμενο του προχείρου δεν '
            'επιστρέφει πια, ακόμα κι αν είναι ανοιχτή η σχετική ρύθμιση.')]},
{'version': '0.9.10',
 'date': '2026-09-25',
 'summary': ('Pasting that checks the words arrived, a guide that picks the right model for you, '
             'and the same recording now gives the same text every time.',
             'Επικόλληση που ελέγχει ότι οι λέξεις έφτασαν, οδηγός που διαλέγει το σωστό μοντέλο για εσένα '
             'και η ίδια ηχογράφηση δίνει πια κάθε φορά το ίδιο κείμενο.'),
 'lines': [('added',
            'Help me choose: two questions in Settings and on first launch (the language you speak, what '
            'matters most) plus your graphics card give one model marked Best for you. Use it downloads '
            'what is missing and switches to it.',
            'Βοήθησέ με να διαλέξω: δύο ερωτήσεις στις Ρυθμίσεις και στην πρώτη εκκίνηση (η γλώσσα που μιλάς '
            'και τι σε νοιάζει πιο πολύ) μαζί με την κάρτα γραφικών σου δίνουν ένα μοντέλο με την ένδειξη '
            'Ιδανικό για σένα. Το Βάλε το κατεβάζει ό,τι λείπει και το ενεργοποιεί.'),
           ('added',
            'Every model and engine now says in plain words who it is for, what you get and what it costs.',
            'Κάθε μοντέλο και κάθε μηχανή λέει πια με απλά λόγια για ποιον είναι, τι κερδίζεις και ποιο '
            'είναι το τίμημα.'),
           ('added',
            'The bar warns within six seconds when the microphone sends nothing, so you can unmute before '
            'the dictation is lost.',
            'Η μπάρα σε ειδοποιεί μέσα σε έξι δευτερόλεπτα όταν το μικρόφωνο δεν στέλνει τίποτα, ώστε να '
            'το ανοίξεις πριν χαθεί η υπαγόρευση.'),
           ('fixed',
            'Pasting now checks that the program you are typing in actually took the words. When nothing '
            'took them within a second, Ctrl+V is pressed once more, and if that fails too the words stay '
            'on the clipboard and the note says so.',
            'Η επικόλληση ελέγχει πια ότι το πρόγραμμα όπου γράφεις πήρε όντως τις λέξεις. Όταν κανείς δεν '
            'τις πήρε μέσα σε ένα δευτερόλεπτο, πατιέται ξανά Ctrl+V. Αν αποτύχει κι αυτό, οι λέξεις μένουν '
            'στο πρόχειρο και το μήνυμα το λέει καθαρά.'),
           ('fixed',
            'Browsers and Electron programs could move the focus to their menu before the paste, so the '
            'words landed nowhere.',
            'Οι browsers και τα προγράμματα Electron μετέφεραν μερικές φορές την εστίαση στο μενού τους πριν '
            'την επικόλληση και οι λέξεις δεν έφταναν πουθενά.'),
           ('fixed',
            'The same recording could come out as different text on each try. The speech engine now '
            'starts every recognition the same way, so the same audio gives the same words.',
            'Η ίδια ηχογράφηση μπορούσε να βγάλει διαφορετικό κείμενο σε κάθε δοκιμή. Η μηχανή ομιλίας '
            'ξεκινά πια κάθε αναγνώριση με τον ίδιο τρόπο, οπότε ο ίδιος ήχος δίνει τις ίδιες λέξεις.'),
           ('fixed',
            'A correction learned from one History edit could rewrite a common word everywhere, web '
            'addresses included. Learned corrections now match only the exact spelling you fixed and '
            'never touch addresses.',
            'Μια διόρθωση που μάθαινε από το Ιστορικό μπορούσε να αλλάζει μια συνηθισμένη λέξη παντού, '
            'ακόμα και μέσα σε διευθύνσεις. Οι μαθημένες διορθώσεις πιάνουν πια μόνο την ακριβή γραφή που '
            'διόρθωσες και αφήνουν τις διευθύνσεις ήσυχες.'),
           ('fixed',
            'When one History edit fixes words in two places, both are learned. Small grammar changes '
            'such as one letter in an ending stay out of the dictionary.',
            'Όταν μια διόρθωση στο Ιστορικό αλλάζει λέξεις σε δύο σημεία, μαθαίνονται και τα δύο. Μικρές '
            'αλλαγές γραμματικής, όπως ένα γράμμα στην κατάληξη, μένουν έξω από το λεξικό.'),
           ('fixed',
            'ό,τι stays one word, and subtitle credits that the engine sometimes adds after real speech are '
            'removed.',
            'Το ό,τι μένει μία λέξη. Οι τίτλοι υποτίτλων που προσθέτει καμιά φορά η μηχανή μετά από '
            'πραγματική ομιλία αφαιρούνται.'),
           ('known',
            'In the Claude desktop app about 2 in 60 dictations still did not arrive in testing. The words '
            'stay on the clipboard and Ctrl+V pastes them.',
            'Στην εφαρμογή Claude για υπολογιστή, περίπου 2 στις 60 υπαγορεύσεις δεν έφτασαν ακόμα στις '
            'δοκιμές. Οι λέξεις μένουν στο πρόχειρο και το Ctrl+V τις επικολλά.')]},
{'version': '0.9.9',
 'date': '2026-09-19',
 'summary': ('Better recovery when recognition leaves your chosen languages, plus fixes for startup, '
             'shortcuts and pasting.',
             'Καλύτερη ανάκτηση όταν η αναγνώριση ξεφεύγει από τις γλώσσες σου και διορθώσεις στην εκκίνηση, '
             'στις συντομεύσεις και στην επικόλληση.'),
 'lines': [('fixed',
            'Both recognition attempts are checked. If the retry fails or still returns a third language, '
            'the recording stays available to retry. The same check covers imported audio.',
            'Ελέγχονται και οι δύο προσπάθειες αναγνώρισης. Αν η επανάληψη αποτύχει ή επιστρέψει ξανά τρίτη '
            'γλώσσα, η ηχογράφηση κρατιέται για νέα προσπάθεια. Ο ίδιος έλεγχος καλύπτει τα εισαγόμενα '
            'ηχητικά.'),
           ('fixed',
            'Startup offers a retry button after about 15 seconds when settings cannot load.',
            'Η εκκίνηση εμφανίζει κουμπί επανάληψης μετά από περίπου 15 δευτερόλεπτα όταν καθυστερεί η '
            'φόρτωση των ρυθμίσεων.'),
           ('fixed',
            'Changing the cloud recognition address or model now updates the active connection.',
            'Η αλλαγή διεύθυνσης ή μοντέλου της εξωτερικής υπηρεσίας αναγνώρισης ενημερώνει πλέον την ενεργή '
            'σύνδεση.'),
           ('changed',
            'Imported source recordings are excluded from saved-audio cleanup lists, adding another layer of '
            'file protection.',
            'Τα αρχικά αρχεία που εισάγεις εξαιρούνται από τις λίστες καθαρισμού αποθηκευμένων ηχητικών, '
            'προσθέτοντας ακόμη ένα επίπεδο προστασίας.'),
           ('fixed',
            'The Alt key that stops recording keeps its release handling. The floating bar also ends a drag '
            'when a quick mouse release was missed.',
            'Το Alt που σταματά την εγγραφή διατηρεί τον σωστό χειρισμό κατά την απελευθέρωσή του. Η μπάρα '
            'σταματά επίσης να ακολουθεί το ποντίκι όταν χαθεί μια γρήγορη απελευθέρωση του κουμπιού.'),
           ('fixed',
            'Shortcut fields keep focus while typing. Snippets preserve literal dollar signs. Clipboard '
            'recovery sends its request before waiting for the result.',
            'Τα πεδία συντομεύσεων κρατούν την εστίαση όσο γράφεις. Τα αποσπάσματα διατηρούν το σύμβολο του '
            'δολαρίου. Η ανάκτηση του προχείρου στέλνει πρώτα το αίτημα και μετά περιμένει το αποτέλεσμα.'),
           ('fixed',
            'English statements such as “You are right.” retain their punctuation. Diagnostics selects the '
            'newest log and screen readers follow the interface language.',
            'Αγγλικές δηλώσεις όπως το «You are right.» διατηρούν τη στίξη τους. Τα διαγνωστικά επιλέγουν το '
            'νεότερο αρχείο καταγραφής και οι αναγνώστες οθόνης ακολουθούν τη γλώσσα της εφαρμογής.'),
           ('known',
            'The intermittent white Windows window above the listening indicator remains under '
            'investigation.',
            'Το περιστασιακό λευκό παράθυρο των Windows πάνω από την ένδειξη ακρόασης παραμένει υπό '
            'διερεύνηση.'),
           ('known',
            'Third-language text written in Latin letters may still pass if the engine labels it as English '
            'or omits its language. Everyday dictation accuracy still varies.',
            'Κείμενο τρίτης γλώσσας με λατινικά γράμματα μπορεί ακόμη να περάσει αν η μηχανή το χαρακτηρίσει '
            'αγγλικό ή παραλείψει τη γλώσσα. Η ακρίβεια της καθημερινής υπαγόρευσης εξακολουθεί να '
            'ποικίλλει.')]},
    {
        "version": "0.9.8",
        "date": "2026-09-17",
        "summary": (
            "The language lock from 0.9.7 now works. Every short piece of a dictation is "
            "checked on its own, and a piece that comes out in a third language is heard "
            "again in yours.",
            "Το κλείδωμα γλώσσας της 0.9.7 δουλεύει πλέον. Κάθε μικρό κομμάτι της υπαγόρευσης "
            "ελέγχεται χωριστά και όποιο βγει σε τρίτη γλώσσα ακούγεται ξανά στη δική σας.",
        ),
        "lines": [
            ("fixed",
             "The language lock added in 0.9.7 never fired. It waited for the engine to name "
             "the language it heard, and the app never asked the engine for that. The engine "
             "now names it for every piece.",
             "Το κλείδωμα γλώσσας της 0.9.7 δεν ενεργοποιούνταν ποτέ. Περίμενε από τη μηχανή να "
             "πει ποια γλώσσα άκουσε, αλλά η εφαρμογή δεν της το ζητούσε. Τώρα η μηχανή το λέει "
             "για κάθε κομμάτι."),
            ("fixed",
             "Stray words in another language inside a correct sentence, such as one Russian or "
             "Polish word in a Greek paragraph. While you speak, the app cuts the recording at "
             "every pause, and on a piece one second long the engine guesses the language badly. "
             "Each piece is now checked on its own and heard again in your language when it "
             "strays. One word in a foreign alphabet is enough.",
             "Σκόρπιες λέξεις σε άλλη γλώσσα μέσα σε σωστή πρόταση, για παράδειγμα μία ρωσική ή "
             "πολωνική λέξη σε ελληνική παράγραφο. Όσο μιλάτε, η εφαρμογή κόβει την ηχογράφηση σε "
             "κάθε παύση και σε κομμάτι ενός δευτερολέπτου η μηχανή μαντεύει άσχημα τη γλώσσα. "
             "Κάθε κομμάτι ελέγχεται πλέον χωριστά και ακούγεται ξανά στη γλώσσα σας όταν ξεφύγει. "
             "Μία λέξη σε ξένο αλφάβητο αρκεί."),
            ("changed",
             "When the last piece of a dictation has to be heard again, the text arrives half a "
             "second to a second and a half later. A single English word said on its own can "
             "come out in the letters of your language.",
             "Όταν το τελευταίο κομμάτι μιας υπαγόρευσης χρειάζεται δεύτερο άκουσμα, το κείμενο "
             "φτάνει μισό ως ενάμισι δευτερόλεπτο αργότερα. Μια αγγλική λέξη ειπωμένη μόνη της "
             "μπορεί να βγει με τα γράμματα της γλώσσας σας."),
        ],
    },
    {
        "version": "0.9.7",
        "date": "2026-09-16",
        "summary": (
            "The dictation language stays yours: when the engine guesses a third language, "
            "the recording is heard again in the one you chose. Any of thirty-three languages "
            "can be that one now. The bar can be dragged to any edge of the screen, the "
            "discreet style is a single dot, and a crash that killed the app in the first hour "
            "after every boot is gone.",
            "Η γλώσσα της υπαγόρευσης μένει η δική σας: όταν η μηχανή μαντέψει τρίτη γλώσσα, "
            "η ηχογράφηση ακούγεται ξανά στη γλώσσα που διαλέξατε. Δική σας μπορεί πλέον να "
            "είναι οποιαδήποτε από τριάντα τρεις γλώσσες. Η μπάρα σέρνεται σε όποια πλευρά της "
            "οθόνης θέλετε, η διακριτική εμφάνιση είναι μια τελεία και μια κατάρρευση που "
            "σκότωνε την εφαρμογή την πρώτη ώρα μετά από κάθε εκκίνηση των Windows χάθηκε.",
        ),
        "lines": [
            ("fixed",
             "In the mixed mode (your language and English) the engine was free to guess among "
             "all ninety-nine languages it knows, and a short Greek phrase could come out in "
             "Czech or Turkish. Now anything that is neither your language nor English is heard "
             "again with your language forced. It costs about a second, only on those occasions.",
             "Στη μεικτή λειτουργία (η γλώσσα σας και αγγλικά) η μηχανή ήταν ελεύθερη να μαντέψει "
             "ανάμεσα και στις ενενήντα εννιά γλώσσες που ξέρει και μια σύντομη ελληνική φράση "
             "μπορούσε να βγει τσέχικα ή τουρκικά. Τώρα ό,τι δεν είναι ούτε η γλώσσα σας ούτε "
             "αγγλικά ακούγεται ξανά με τη γλώσσα σας κλειδωμένη. Κοστίζει περίπου ένα "
             "δευτερόλεπτο, μόνο σε αυτές τις περιπτώσεις."),
            ("added",
             "Your language is a choice in Settings, Language, with thirty-three to pick from. "
             "The modes are named after it: your language and English mixed, your language "
             "only, English only, or auto-detect among everything. A fresh install picks the "
             "language of your Windows.",
             "Η γλώσσα σας είναι επιλογή στις Ρυθμίσεις, Γλώσσα, με τριάντα τρεις διαθέσιμες. "
             "Οι λειτουργίες παίρνουν το όνομά της: η γλώσσα σας και αγγλικά μαζί, μόνο η "
             "γλώσσα σας, μόνο αγγλικά ή αυτόματη ανίχνευση ανάμεσα σε όλες. Μια καινούργια "
             "εγκατάσταση διαλέγει τη γλώσσα των Windows σας."),
            ("added",
             "The bar can be dragged with the mouse while it is on screen. It sticks to the "
             "nearest edge, top, bottom, left or right, at the spot where you let go, and "
             "remembers the place. Settings, Overlay has a button that shows the bar for fifteen "
             "seconds so you can move it without dictating.",
             "Η μπάρα σέρνεται με το ποντίκι όσο είναι στην οθόνη. Κολλάει στην πιο κοντινή "
             "πλευρά, πάνω, κάτω, αριστερά ή δεξιά, στο σημείο που την αφήσατε και θυμάται τη "
             "θέση. Στις Ρυθμίσεις, Μπάρα υπάρχει κουμπί που τη δείχνει για δεκαπέντε δευτερόλεπτα "
             "για να τη μετακινήσετε χωρίς να υπαγορεύετε."),
            ("changed",
             "The discreet style is now a single dot and nothing else: red while listening, "
             "green when done, amber when the words landed in the clipboard and wait for you to paste them.",
             "Η διακριτική εμφάνιση είναι πλέον μια τελεία και τίποτα άλλο: κόκκινη όσο ακούει, "
             "πράσινη όταν τελειώσει, πορτοκαλί όταν οι λέξεις μπήκαν στο πρόχειρο και περιμένουν "
             "να τις επικολλήσετε."),
            ("fixed",
             "For the first hour after Windows started, the app could close without a word the "
             "moment the microphone stumbled. A clock that counts from boot was asked for a time "
             "before boot. Both the cause and a test that forbids it are in.",
             "Την πρώτη ώρα μετά την εκκίνηση των Windows η εφαρμογή μπορούσε να κλείσει χωρίς "
             "κουβέντα τη στιγμή που σκόνταφτε το μικρόφωνο. Ένα ρολόι που μετράει από την "
             "εκκίνηση ρωτήθηκε για μια στιγμή πριν από αυτήν. Μπήκε και η διόρθωση και ένα "
             "τεστ που το απαγορεύει."),
            ("fixed",
             "The Windows clipboard history (Win+V) could grab the text before the window you "
             "were dictating into, and the paste came out empty. The history is kept out of the "
             "way until the paste is over.",
             "Το ιστορικό προχείρου των Windows (Win+V) μπορούσε να αρπάξει το κείμενο πριν από "
             "το παράθυρο στο οποίο υπαγορεύατε και η επικόλληση έβγαινε άδεια. Το ιστορικό "
             "μένει στην άκρη μέχρι να τελειώσει η επικόλληση."),
        ],
    },
    {
        "version": "0.9.6",
        "date": "2026-09-11",
        "summary": (
            "Versions 0.9.4 and 0.9.5 were never published, so this release carries both: the "
            "right Alt key that stopped working and the twenty-three faults found while proving "
            "it was never the key, a rebuilt paste, shortcuts on the mouse, the program's own "
            "name, an update that now announces itself, and eleven fixes from a full audit.",
            "Οι εκδόσεις 0.9.4 και 0.9.5 δεν δημοσιεύτηκαν ποτέ, οπότε αυτή η έκδοση φέρνει και "
            "τις δύο: το δεξί Alt που σταμάτησε να δουλεύει και τα είκοσι τρία λάθη που βρέθηκαν "
            "όσο αποδεικνυόταν ότι δεν έφταιγε ποτέ το πλήκτρο, μια επικόλληση χτισμένη από την "
            "αρχή, συντομεύσεις στο ποντίκι, το δικό του όνομα για το πρόγραμμα, μια ενημέρωση "
            "που πλέον ανακοινώνεται μόνη της και έντεκα διορθώσεις από έναν πλήρη έλεγχο.",
        ),
        "lines": [
            ("added",
             "The app looks for a new version a few seconds after it opens. When there is one, "
             "its window comes to the front with a large Update now card in the middle, and if "
             "you are dictating at that moment it waits until you finish. When there is nothing "
             "newer, nothing appears. Check for updates is also in the tray menu.",
             "Η εφαρμογή ψάχνει για νέα έκδοση λίγα δευτερόλεπτα αφού ανοίξει. Όταν υπάρχει, το "
             "παράθυρό της έρχεται μπροστά με μια μεγάλη κάρτα Ενημέρωση τώρα στη μέση και αν "
             "εκείνη τη στιγμή υπαγορεύετε, περιμένει να τελειώσετε. Όταν δεν υπάρχει κάτι νεότερο, "
             "δεν εμφανίζεται τίποτα. Ο Έλεγχος για ενημερώσεις υπάρχει και στο μενού του εικονιδίου."),
            ("fixed",
             "Deleting a History entry that came from an audio file you transcribed also deleted "
             "your original audio file, wherever it was on the disk. Only the copies the app made "
             "itself are deleted now.",
             "Η διαγραφή μιας εγγραφής του Ιστορικού που προερχόταν από αρχείο ήχου που "
             "μεταγράψατε έσβηνε και το δικό σας αρχικό αρχείο, όπου κι αν βρισκόταν στον δίσκο. "
             "Τώρα σβήνονται μόνο τα αντίγραφα που έφτιαξε η ίδια η εφαρμογή."),
            ("fixed",
             "Two rare faults could close the app in the middle of a dictation without a word: "
             "unusual content on the clipboard, and a cleanup rule that ran too long on a long "
             "transcript. In both cases the text is now kept as it is and the dictation carries on.",
             "Δύο σπάνια λάθη μπορούσαν να κλείσουν την εφαρμογή στη μέση μιας υπαγόρευσης χωρίς "
             "κουβέντα: ασυνήθιστο περιεχόμενο στο πρόχειρο και ένας κανόνας καθαρισμού που "
             "κρατούσε πολύ σε μεγάλο κείμενο. Και στις δύο περιπτώσεις το κείμενο μένει πλέον "
             "όπως είναι και η υπαγόρευση συνεχίζει."),
            ("fixed",
             "Installing over an older version could leave the old program running. The new one "
             "then closed at once, and you stayed on the old version without being told. The "
             "installer now closes the old one, and when Windows will not let it, it asks you to "
             "quit it from the tray and try again.",
             "Η εγκατάσταση πάνω σε παλαιότερη έκδοση μπορούσε να αφήσει το παλιό πρόγραμμα να "
             "τρέχει. Το νέο έκλεινε αμέσως και μένατε στην παλιά έκδοση χωρίς να το ξέρετε. Ο "
             "installer κλείνει πλέον το παλιό και όταν τα Windows δεν τον αφήνουν, σας ζητάει να "
             "το κλείσετε από το εικονίδιο και να ξαναδοκιμάσετε."),
            ("fixed",
             "An old copy started after the new version had moved your data saw an empty history "
             "and began a second one of its own. The two are now joined the next time the app "
             "starts, and nothing is deleted.",
             "Ένα παλιό αντίγραφο που ξεκινούσε αφού η νέα έκδοση είχε μεταφέρει τα δεδομένα σας "
             "έβλεπε άδειο ιστορικό και ξεκινούσε δεύτερο δικό του. Τα δύο ενώνονται πλέον την "
             "επόμενη φορά που ξεκινάει η εφαρμογή και δεν σβήνεται τίποτα."),
            ("fixed",
             "During the move to the new folder names, a folder that held only recordings or "
             "logs could be deleted. It is now kept under another name.",
             "Κατά τη μετακόμιση στα νέα ονόματα φακέλων, ένας φάκελος με μόνο ηχογραφήσεις ή "
             "αρχεία καταγραφής μπορούσε να σβηστεί. Τώρα κρατιέται με άλλο όνομα."),
            ("fixed",
             "If the settings file could not be read for a moment, your choices were replaced by "
             "the defaults. They are now kept, and a file that cannot be read is saved aside.",
             "Αν το αρχείο ρυθμίσεων δεν διαβαζόταν για μια στιγμή, οι επιλογές σας "
             "αντικαθίσταντο από τις προεπιλογές. Τώρα κρατιούνται και ένα αρχείο που δεν "
             "διαβάζεται φυλάγεται στην άκρη."),
            ("fixed",
             "Delete all data left full copies of the history in the backup folder. They are "
             "deleted too now.",
             "Η Διαγραφή όλων των δεδομένων άφηνε ολόκληρα αντίγραφα του ιστορικού στον φάκελο "
             "αντιγράφων ασφαλείας. Τώρα σβήνονται κι αυτά."),
            ("fixed",
             "After an update the Windows startup entry could keep naming the old program file, "
             "which the installer had just removed. It now names the new one.",
             "Μετά από ενημέρωση η αυτόματη εκκίνηση των Windows μπορούσε να δείχνει ακόμα το "
             "παλιό αρχείο του προγράμματος, που ο installer είχε μόλις σβήσει. Τώρα δείχνει το νέο."),
            ("fixed",
             "Pressing the update button twice could start two downloads at once. A second "
             "press is now turned down while the first download runs.",
             "Δύο πατήματα στο κουμπί ενημέρωσης μπορούσαν να ξεκινήσουν δύο κατεβάσματα μαζί. "
             "Ένα δεύτερο πάτημα απορρίπτεται πλέον όσο τρέχει το πρώτο κατέβασμα."),
            ("fixed",
             "A damaged speech model of the right size could make the app throw away the good "
             "copy that came with the installer. The two files are now compared before either "
             "is removed.",
             "Ένα χαλασμένο μοντέλο φωνής με το σωστό μέγεθος μπορούσε να κάνει την εφαρμογή να "
             "πετάξει το καλό αντίγραφο που ήρθε με τον installer. Τα δύο αρχεία συγκρίνονται "
             "πλέον πριν σβηστεί οποιοδήποτε."),
            ("changed",
             "The part of the app that listens for keys now runs ahead of ordinary work, so a "
             "busy machine is less likely to make Windows drop it.",
             "Το κομμάτι της εφαρμογής που ακούει τα πλήκτρα τρέχει πλέον πριν από την υπόλοιπη "
             "δουλειά του υπολογιστή, οπότε ένα φορτωμένο μηχάνημα έχει λιγότερες πιθανότητες να "
             "κάνει τα Windows να το πετάξουν."),
            ("fixed",
             "The hotkey went dead for up to half a minute at a time. Windows removes the "
             "part of the app that listens for keys when one busy moment takes too long, "
             "and tells nobody. The app now puts it back every second, where it used to be every "
             "thirty. On a machine at full load, eleven key presses in a row reached "
             "Windows and none of them reached the app.",
             "Το πλήκτρο νέκρωνε ως και μισό λεπτό κάθε φορά. Τα Windows αφαιρούν το "
             "κομμάτι της εφαρμογής που ακούει τα πλήκτρα όταν μια στιγμή φόρτου κρατήσει "
             "πολύ, χωρίς να το πουν πουθενά. Τώρα ξαναμπαίνει κάθε δευτερόλεπτο, ενώ πριν "
             "ξαναέμπαινε κάθε τριάντα. Σε μηχάνημα στο φόρτο, έντεκα συνεχόμενα πατήματα έφτασαν στα "
             "Windows και κανένα στην εφαρμογή."),
            ("fixed",
             "Everything on the clipboard that was not plain text was destroyed by every "
             "dictation. A copied picture, a copied file or copied formatted text is now "
             "put back exactly as it was.",
             "Ό,τι βρισκόταν στο πρόχειρο και δεν ήταν σκέτο κείμενο καταστρεφόταν σε κάθε "
             "υπαγόρευση. Μια εικόνα, ένα αρχείο ή κείμενο με μορφοποίηση επιστρέφουν πλέον "
             "ακριβώς όπως ήταν."),
            ("fixed",
             "Recording a new shortcut read one press of the right Alt as two keys, because "
             "that is how Windows delivers it on a Greek keyboard, and the shortcut it saved "
             "never matched afterwards.",
             "Η καταγραφή νέας συντόμευσης διάβαζε ένα πάτημα του δεξιού Alt ως δύο πλήκτρα, "
             "επειδή έτσι το παραδίδουν τα Windows σε ελληνικό πληκτρολόγιο και η συντόμευση "
             "που αποθηκευόταν δεν ταίριαζε ποτέ μετά."),
            ("fixed",
             "If the speech model file disappeared while the app was running, the app tried "
             "to restart it every five seconds for as long as it stayed open. It now stops "
             "and says what is missing.",
             "Αν χανόταν το αρχείο του μοντέλου φωνής ενώ έτρεχε η εφαρμογή, εκείνη "
             "προσπαθούσε να το ξεκινήσει κάθε πέντε δευτερόλεπτα για όσο έμενε ανοιχτή. "
             "Τώρα σταματάει και λέει τι λείπει."),
            ("fixed",
             "After the app had crashed once, the Recent problems list showed only old crash "
             "lines and hid every warning of the current run.",
             "Αφού η εφαρμογή είχε καταρρεύσει μία φορά, η λίστα Πρόσφατα προβλήματα έδειχνε "
             "μόνο παλιές γραμμές κατάρρευσης και έκρυβε κάθε προειδοποίηση της τρέχουσας "
             "εκτέλεσης."),
            ("fixed",
             "A very short word said between two long pauses could disappear from the "
             "finished text.",
             "Μια πολύ σύντομη λέξη ανάμεσα σε δύο μεγάλες παύσεις μπορούσε να εξαφανιστεί "
             "από το τελικό κείμενο."),
            ("fixed",
             "The paste was built the wrong way round. The app used to put a note on the clipboard "
             "saying the words could be had on request, then wait to be asked. Anything else on the "
             "machine that watches the clipboard, a remote desktop session for one, could ask first "
             "and take the answer, leaving the window that mattered with nothing and the app with a "
             "question it could no longer answer. The words now go on the clipboard as they are, "
             "before the key is pressed, so there is nothing left to ask for and nothing to go wrong "
             "in between. The wait that used to cost most of a second is gone with it.",
             "Η επικόλληση ήταν χτισμένη ανάποδα. Η εφαρμογή άφηνε στο πρόχειρο ένα σημείωμα ότι το "
             "κείμενο δίνεται όποτε ζητηθεί και μετά περίμενε να της το ζητήσουν. Οτιδήποτε άλλο στο "
             "μηχάνημα παρακολουθεί το πρόχειρο, όπως μια σύνδεση απομακρυσμένου γραφείου, προλάβαινε "
             "να ρωτήσει πρώτο και έπαιρνε την απάντηση, αφήνοντας το παράθυρο που μας ενδιέφερε με "
             "τίποτα. Πλέον το κείμενο μπαίνει στο πρόχειρο όπως είναι, πριν πατηθεί το πλήκτρο, οπότε "
             "δεν μένει τίποτα να ζητηθεί και τίποτα να στραβώσει ενδιάμεσα. Μαζί του έφυγε και η "
             "αναμονή που έτρωγε σχεδόν ένα δευτερόλεπτο.",
             ),
            ("changed",
             "The dictated words stay in the Windows clipboard history on purpose, so Win+V brings "
             "them back if a window ever swallows a paste. They are still refused to the cloud "
             "clipboard, so nothing leaves the machine.",
             "Τα λόγια που υπαγορεύετε μένουν επίτηδες στο ιστορικό προχείρου των Windows, ώστε το "
             "Win+V να τα φέρνει πίσω αν κάποιο παράθυρο καταπιεί την επικόλληση. Εξακολουθούν να "
             "μην ανεβαίνουν στο πρόχειρο του σύννεφου, οπότε τίποτα δεν φεύγει από το μηχάνημα.",
             ),
            ("fixed",
             "On a German, French or Polish keyboard the right Alt is also the key that types the "
             "at sign, the euro sign and every accented letter, so every one of those started a "
             "dictation. A letter typed within the first moments of holding the key now means "
             "writing, and the recording is dropped without a word.",
             "Σε γερμανικό, γαλλικό ή πολωνικό πληκτρολόγιο το δεξί Alt είναι και το πλήκτρο που "
             "γράφει το παπάκι, το ευρώ και κάθε τονισμένο γράμμα, οπότε καθένα από αυτά ξεκινούσε "
             "υπαγόρευση. Ένα γράμμα που πατιέται μέσα στις πρώτες στιγμές του κρατήματος σημαίνει "
             "πλέον γράψιμο και η ηχογράφηση πετιέται χωρίς κουβέντα.",
             ),
            ("changed",
             "The program, its folders and its files carry the product name now. The old name "
             "showed up in Windows dialogs and on disk, where it had no business being. Your "
             "settings, your history and your models move themselves the first time the new "
             "version starts.",
             "Το πρόγραμμα, οι φάκελοί του και τα αρχεία του έχουν πλέον το όνομα του προϊόντος. Το "
             "παλιό όνομα εμφανιζόταν σε παράθυρα των Windows και στον δίσκο, εκεί που δεν είχε "
             "καμία δουλειά. Οι ρυθμίσεις σας, το ιστορικό σας και τα μοντέλα σας μετακομίζουν μόνα "
             "τους την πρώτη φορά που ξεκινάει η νέα έκδοση.",
             ),
            ("fixed",
             "The small window's message was cut off at 12 pixels on one line, so the half that said "
             "what to do was the half you could not see. It is now 16 pixels and wraps.",
             "Το μήνυμα στο μικρό παράθυρο κοβόταν στα 12 πίξελ σε μία γραμμή, οπότε το μισό που έλεγε τι "
             "να κάνετε ήταν το μισό που δεν βλέπατε. Τώρα είναι 16 πίξελ και αναδιπλώνεται."),
            ("fixed",
             "Every message in that window was in English. They now follow the language of the app.",
             "Κάθε μήνυμα σε αυτό το παράθυρο ήταν στα αγγλικά. Τώρα ακολουθούν τη γλώσσα της εφαρμογής."),
            ("added",
             "The side buttons of a mouse can hold a shortcut. They are called Mouse4 and Mouse5 and "
             "combine with Ctrl, Shift and Alt like any key.",
             "Τα πλαϊνά κουμπιά του ποντικιού μπορούν να γίνουν συντόμευση. Λέγονται Mouse4 και Mouse5 "
             "και συνδυάζονται με Ctrl, Shift και Alt όπως κάθε πλήκτρο."),
            ("fixed",
             "If the last piece of a long dictation failed to transcribe, the whole dictation was "
             "thrown away. What was already transcribed while you spoke is now kept.",
             "Αν το τελευταίο κομμάτι μιας μεγάλης υπαγόρευσης αποτύγχανε να μεταγραφεί, πετιόταν "
             "ολόκληρη. Ό,τι είχε ήδη μεταγραφεί όσο μιλούσατε κρατιέται πλέον."),
            ("fixed",
             "Saving an empty correction in History made the entry look blank for good, and Copy "
             "copied nothing. The original stays visible until a real correction replaces it.",
             "Η αποθήκευση άδειας διόρθωσης στο Ιστορικό έκανε την εγγραφή να φαίνεται κενή για πάντα "
             "και η Αντιγραφή δεν αντέγραφε τίποτα. Το αρχικό μένει ορατό μέχρι να το αντικαταστήσει "
             "αληθινή διόρθωση."),
            ("fixed",
             "Paste last and the Paste button in History ignored your correction and used the "
             "original words.",
             "Η επικόλληση του τελευταίου και το κουμπί Επικόλληση στο Ιστορικό αγνοούσαν τη διόρθωσή "
             "σας και έβαζαν τις αρχικές λέξεις."),
            ("fixed",
             "The Paste button in History pasted into the app's own window. It now puts the text on "
             "the clipboard and asks you to click the window you want.",
             "Το κουμπί Επικόλληση στο Ιστορικό επικολλούσε μέσα στο παράθυρο της ίδιας της εφαρμογής. "
             "Τώρα βάζει το κείμενο στο πρόχειρο και σας ζητάει να πατήσετε στο παράθυρο που θέλετε."),
            ("fixed",
             "A retention of 0 days was accepted and would erase the whole history with its "
             "recordings on the next start. The smallest value is one day.",
             "Διατήρηση 0 ημερών γινόταν δεκτή και θα έσβηνε όλο το ιστορικό με τις ηχογραφήσεις του "
             "στην επόμενη εκκίνηση. Η μικρότερη τιμή είναι μία μέρα."),
            ("fixed",
             "One invalid shortcut discarded every change on every settings tab. An empty shortcut "
             "now means off, and a bad one is named.",
             "Μία λανθασμένη συντόμευση πετούσε κάθε αλλαγή σε κάθε καρτέλα ρυθμίσεων. Άδεια συντόμευση "
             "σημαίνει πλέον απενεργοποίηση και η λανθασμένη κατονομάζεται."),
            ("fixed",
             "The Start with Windows switch could stay on when nothing had been registered. It now "
             "turns itself back off and says so.",
             "Ο διακόπτης Εκκίνηση με τα Windows μπορούσε να μείνει αναμμένος χωρίς να έχει καταχωρηθεί "
             "τίποτα. Τώρα σβήνει μόνος του και το λέει."),
            ("fixed",
             "The Save button in Settings could stay lit after a successful save.",
             "Το κουμπί Αποθήκευση στις Ρυθμίσεις μπορούσε να μείνει αναμμένο μετά από επιτυχή αποθήκευση."),
            ("fixed",
             "The small window could stay on Failed, or keep spinning, after pasting with nothing to "
             "paste or when the microphone would not open.",
             "Το μικρό παράθυρο μπορούσε να μείνει στο Απέτυχε ή να γυρίζει ασταμάτητα, μετά από "
             "επικόλληση χωρίς κείμενο ή όταν δεν άνοιγε το μικρόφωνο."),
            ("fixed",
             "When a paste was refused and the clipboard refused it too, the app still said the text "
             "was on the clipboard. It now says to open History.",
             "Όταν μια επικόλληση απορριπτόταν και το πρόχειρο την απέρριπτε επίσης, η εφαρμογή έλεγε "
             "πάλι ότι το κείμενο είναι στο πρόχειρο. Τώρα λέει να ανοίξετε το Ιστορικό."),
            ("fixed",
             "A dictation that ended with the tray icon in front is now refused out loud, "
             "the way the desktop and the taskbar already were.",
             "Μια υπαγόρευση που τελείωσε με το εικονίδιο της γραμμής εργασιών μπροστά "
             "απορρίπτεται πλέον φωναχτά, όπως η επιφάνεια εργασίας και η γραμμή εργασιών."),
            ("fixed",
             "A copy run from the build folder no longer takes over the Windows startup "
             "entry. Only the installed copy does.",
             "Ένα αντίγραφο που τρέχει από τον φάκελο χτισίματος δεν παίρνει πια την αυτόματη "
             "εκκίνηση των Windows. Μόνο το εγκατεστημένο αντίγραφο την παίρνει."),
            ("added",
             "Diagnostics has a Key check: press your hotkey and see live what Windows "
             "delivered. When the left Alt arrives while hands-free sits on the right one, "
             "it says so.",
             "Τα Διαγνωστικά έχουν Έλεγχο πλήκτρου: πατάτε το πλήκτρο σας και βλέπετε ζωντανά "
             "τι παρέδωσαν τα Windows. Όταν φτάνει το αριστερό Alt ενώ τα ελεύθερα χέρια "
             "είναι στο δεξί, το λέει."),
            ("changed",
             "The dashboard no longer stutters while it is open, and the wait after the stop "
             "key no longer includes writing to disk. Saving the history and the recovery "
             "recording happen out of the way.",
             "Το ταμπλό δεν κολλάει πια όσο είναι ανοιχτό και η αναμονή μετά το πλήκτρο "
             "δεν περιλαμβάνει πια γράψιμο στον δίσκο. Η αποθήκευση του ιστορικού και της "
             "ηχογράφησης ασφαλείας γίνονται στην άκρη."),
            ("changed",
             "One repeated harmless warning made up two fifths of the diagnostic log. It is "
             "now counted and reported once a minute.",
             "Μία επαναλαμβανόμενη αβλαβής προειδοποίηση αποτελούσε τα δύο πέμπτα του αρχείου "
             "καταγραφής. Τώρα μετριέται και αναφέρεται μία φορά το λεπτό."),
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
