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
        "version": "0.9.5",
        "date": "2026-09-10",
        "summary": (
            "The day the right Alt key stopped working. Proving it was never the key led to "
            "twenty-three more faults, a rebuilt paste that lands on the first try, shortcuts "
            "on the mouse, and the program finally carrying its own name.",
            "Η μέρα που το δεξί Alt σταμάτησε να δουλεύει. Η απόδειξη ότι δεν έφταιγε ποτέ το "
            "πλήκτρο οδήγησε σε άλλα είκοσι τρία λάθη, σε μια επικόλληση χτισμένη από την αρχή "
            "που πετυχαίνει με την πρώτη, σε συντομεύσεις στο ποντίκι και στο να αποκτήσει "
            "επιτέλους το πρόγραμμα το δικό του όνομα.",
        ),
        "lines": [
            ("fixed",
             "The hotkey went dead for up to half a minute at a time. Windows removes the "
             "part of the app that listens for keys when one busy moment takes too long, "
             "and tells nobody. The app now puts it back every second, where it used to be every "
             "thirty. On a machine at full load, eleven key presses in a row reached "
             "Windows and none of them reached the app.",
             "Το πλήκτρο νέκρωνε ως και μισό λεπτό κάθε φορά. Τα Windows αφαιρούν το "
             "κομμάτι της εφαρμογής που ακούει τα πλήκτρα όταν μια στιγμή φόρτου κρατήσει "
             "πολύ, χωρίς να το πουν πουθενά. Τώρα ξαναμπαίνει κάθε δευτερόλεπτο αντί για "
             "κάθε τριάντα. Σε μηχάνημα στο φόρτο, έντεκα συνεχόμενα πατήματα έφτασαν στα "
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
