# -*- coding: utf-8 -*-
"""What changed in every released version, in both languages.

One entry per release, newest first. Adding a release means adding a dict here
and running build_changelog.py; nothing else knows the history.

Each line is (kind, English, Greek). How a line is written:
- Say what the user notices now, in one or two short sentences.
- Leave out the story behind it: how the fault was found, test counts,
  internal causes, file names, and anything that reads like an apology.
- Fold small fixes into one "Smaller fixes" line.
- The English page is read worldwide. A line that only matters to Greek
  dictation or a Greek keyboard sets its English text to None, so it shows
  on the Greek page alone. build_changelog.py refuses Greek letters or the
  word Greek in any English line.
- House rules for both languages: no em-dash, no emoji, and in Greek no comma
  before the word "και". The Greek speaks to one reader (σου).
"""

# The kinds of line, in the order they appear on the page.
KINDS = {
    "added": ("New", "Νέα"),
    "improved": ("Improved", "Βελτιώσεις"),
    "fixed": ("Fixed", "Διορθώσεις"),
    "known": ("Known issues", "Γνωστά προβλήματα"),
}

RELEASES = [
    {
        "version": "0.9.12",
        "date": "2026-09-26",
        "summary": (
            "Now on Ubuntu Linux. Also smarter learning from your edits, Dictionary rules that "
            "know their language, and the speech engine open to your own programs.",
            "Τώρα και σε Ubuntu Linux. Επίσης πιο έξυπνη μάθηση από τις διορθώσεις σου, κανόνες "
            "Λεξικού που ξέρουν τη γλώσσα τους και η μηχανή ομιλίας διαθέσιμη στα δικά σου "
            "προγράμματα.",
        ),
        "lines": [
            ("added",
             "Fuck You Flow runs on Ubuntu Linux 22.04 or newer, as a .deb package or an AppImage, "
             "under X11 and Wayland. It updates itself like the Windows version.",
             "Το Fuck You Flow τρέχει σε Ubuntu Linux 22.04 ή νεότερο, ως πακέτο .deb ή AppImage, "
             "σε X11 και Wayland. Ενημερώνεται μόνο του όπως και στα Windows."),
            ("added",
             "A local endpoint for your own programs, in Settings, Privacy. Scripts and bots on "
             "this computer can send audio and get the text back from the speech engine. It "
             "stays off until you turn it on and answers only programs on this computer.",
             "Τοπική θυρίδα για τα δικά σου προγράμματα, στις Ρυθμίσεις, Απόρρητο. Προγράμματα "
             "και bots σε αυτόν τον υπολογιστή στέλνουν ήχο και παίρνουν πίσω το κείμενο από τη "
             "μηχανή ομιλίας. Μένει κλειστή μέχρι να την ανοίξεις και απαντά μόνο σε "
             "προγράμματα του ίδιου υπολογιστή."),
            ("improved",
             "Learning from your edits now works on long transcripts too. A correction is "
             "learned when you make it twice or when it is a name or a technical term, and a "
             "word you leave as it is elsewhere never becomes a rule. Your earlier edits are "
             "read again once, so new suggestions may be waiting under Suggestions.",
             "Η μάθηση από τις διορθώσεις σου δουλεύει πια και σε μεγάλα κείμενα. Μια διόρθωση "
             "μαθαίνεται όταν την κάνεις δύο φορές ή όταν είναι όνομα ή τεχνικός όρος. Μια λέξη "
             "που αφήνεις ίδια σε άλλα κείμενα δεν γίνεται κανόνας. Οι παλιές σου διορθώσεις "
             "διαβάζονται ξανά μία φορά, οπότε μπορεί να σε περιμένουν νέες Προτάσεις μάθησης."),
            ("improved",
             "Each Dictionary rule belongs to the language of its letters and applies to text in "
             "that language. With two languages chosen, the rules of both work together, English "
             "words inside the other language included. The form picks the language as you type.",
             "Κάθε κανόνας του Λεξικού ανήκει στη γλώσσα των γραμμάτων του και εφαρμόζεται σε "
             "κείμενο αυτής της γλώσσας. Με δύο γλώσσες επιλεγμένες, δουλεύουν μαζί οι κανόνες "
             "και των δύο, μαζί και οι αγγλικές λέξεις μέσα στην άλλη γλώσσα. Η φόρμα διαλέγει "
             "τη γλώσσα όσο γράφεις."),
            ("known",
             "Linux: the AppImage needs a one-time keyboard permission, and the app shows the "
             "command to run. The .deb sets it up during install.",
             "Linux: το AppImage χρειάζεται μία φορά άδεια για το πληκτρολόγιο και η εφαρμογή "
             "δείχνει την εντολή που πρέπει να τρέξεις. Το .deb τη ρυθμίζει μόνο του στην "
             "εγκατάσταση."),
            ("known",
             "Linux under Wayland: per-app styles are skipped, because the app cannot see which "
             "program is in front. Dictation works as usual.",
             "Linux σε Wayland: τα στυλ ανά εφαρμογή δεν εφαρμόζονται, γιατί η εφαρμογή δεν "
             "βλέπει ποιο πρόγραμμα είναι μπροστά. Η υπαγόρευση δουλεύει κανονικά."),
        ],
    },
    {
        "version": "0.9.11",
        "date": "2026-09-25",
        "summary": (
            "A new look called Carbon, and your words stay on the clipboard after pasting into "
            "browsers and the apps built on them.",
            "Νέα εμφάνιση με το όνομα Carbon και οι λέξεις σου μένουν στο πρόχειρο μετά την "
            "επικόλληση σε browsers και στις εφαρμογές που είναι χτισμένες πάνω τους.",
        ),
        "lines": [
            ("added",
             "Carbon, the new default look of the dark theme, with panels that have depth. It "
             "adds no load on the graphics card. The classic flat look is in Settings, General.",
             "Carbon, η νέα προεπιλεγμένη εμφάνιση του σκούρου θέματος, με πάνελ που έχουν "
             "βάθος. Δεν επιβαρύνει την κάρτα γραφικών. Η κλασική επίπεδη εμφάνιση είναι στις "
             "Ρυθμίσεις, Γενικά."),
            ("fixed",
             "In Chrome, Edge and apps built on them, such as Claude and Slack, the words stay on "
             "the clipboard after pasting. If they did not arrive, Ctrl+V brings them back.",
             "Στο Chrome, στον Edge και σε εφαρμογές χτισμένες πάνω τους, όπως το Claude και το "
             "Slack, οι λέξεις μένουν στο πρόχειρο μετά την επικόλληση. Αν δεν έφτασαν, το "
             "Ctrl+V τις φέρνει πίσω."),
        ],
    },
    {
        "version": "0.9.10",
        "date": "2026-09-25",
        "summary": (
            "Pasting that checks the words arrived, a guide that picks the right model, and "
            "steadier results.",
            "Επικόλληση που ελέγχει ότι οι λέξεις έφτασαν, οδηγός που διαλέγει το σωστό "
            "μοντέλο και πιο σταθερά αποτελέσματα.",
        ),
        "lines": [
            ("added",
             "Help me choose: two questions and your graphics card pick the model that suits "
             "you. One click downloads it and switches to it.",
             "Βοήθησέ με να διαλέξω: δύο ερωτήσεις μαζί με την κάρτα γραφικών σου διαλέγουν το "
             "μοντέλο που σου ταιριάζει. Ένα κλικ το κατεβάζει και το ενεργοποιεί."),
            ("added",
             "Every model and engine explains in plain words who it is for and what it asks of "
             "your computer.",
             "Κάθε μοντέλο και κάθε μηχανή εξηγεί με απλά λόγια για ποιον είναι και τι ζητάει "
             "από τον υπολογιστή σου."),
            ("added",
             "The bar warns within six seconds when the microphone sends no sound.",
             "Η μπάρα σε ειδοποιεί μέσα σε έξι δευτερόλεπτα όταν το μικρόφωνο δεν στέλνει ήχο."),
            ("fixed",
             "Pasting checks that the program took the words and tries once more when it did "
             "not. If that fails too, the words wait on the clipboard.",
             "Η επικόλληση ελέγχει ότι το πρόγραμμα πήρε τις λέξεις και ξαναδοκιμάζει όταν δεν "
             "τις πήρε. Αν αποτύχει και πάλι, οι λέξεις περιμένουν στο πρόχειρο."),
            ("fixed",
             "Words could go missing in browsers and Electron apps that moved the focus to their "
             "menu just before the paste.",
             "Οι λέξεις μπορούσαν να χαθούν σε browsers και εφαρμογές Electron που μετέφεραν την "
             "εστίαση στο μενού τους λίγο πριν την επικόλληση."),
            ("fixed",
             "The same recording now gives the same text every time.",
             "Η ίδια ηχογράφηση δίνει πια κάθε φορά το ίδιο κείμενο."),
            ("fixed",
             "Learned corrections match only the exact word you fixed and leave web addresses "
             "alone. An edit that fixes two words teaches both.",
             "Οι μαθημένες διορθώσεις πιάνουν μόνο την ακριβή λέξη που διόρθωσες και αφήνουν "
             "ήσυχες τις διευθύνσεις ιστοσελίδων. Μια διόρθωση σε δύο λέξεις τις μαθαίνει και "
             "τις δύο."),
            ("fixed",
             "Subtitle credits that the engine sometimes added after speech are removed.",
             "Αφαιρούνται οι τίτλοι υποτίτλων που πρόσθετε καμιά φορά η μηχανή μετά την ομιλία."),
            ("fixed",
             None,
             "Το «ό,τι» μένει μία λέξη."),
            ("known",
             "In the Claude desktop app a dictation occasionally does not arrive. The words stay "
             "on the clipboard for Ctrl+V.",
             "Στην εφαρμογή Claude για υπολογιστή, μια υπαγόρευση καμιά φορά δεν φτάνει. Οι "
             "λέξεις μένουν στο πρόχειρο για Ctrl+V."),
        ],
    },
    {
        "version": "0.9.9",
        "date": "2026-09-19",
        "summary": (
            "Better recovery when recognition drifts out of your languages, plus fixes to "
            "startup, shortcuts and pasting.",
            "Καλύτερη ανάκτηση όταν η αναγνώριση ξεφεύγει από τις γλώσσες σου και διορθώσεις "
            "στην εκκίνηση, στις συντομεύσεις και στην επικόλληση.",
        ),
        "lines": [
            ("fixed",
             "When recognition returns a language you did not choose, the recording is kept so "
             "you can try again. The same goes for imported audio.",
             "Όταν η αναγνώριση βγάζει γλώσσα που δεν διάλεξες, η ηχογράφηση κρατιέται για νέα "
             "προσπάθεια. Το ίδιο ισχύει και για τα αρχεία ήχου που εισάγεις."),
            ("fixed",
             "Startup shows a retry button when the settings take too long to load.",
             "Η εκκίνηση δείχνει κουμπί επανάληψης όταν οι ρυθμίσεις αργούν να φορτώσουν."),
            ("fixed",
             "A new cloud provider address or model takes effect at once.",
             "Μια νέα διεύθυνση ή νέο μοντέλο του παρόχου στο διαδίκτυο ισχύει αμέσως."),
            ("improved",
             "Imported recordings are always left out of the saved-audio cleanup.",
             "Τα αρχεία ήχου που εισάγεις μένουν πάντα έξω από τον καθαρισμό των αποθηκευμένων "
             "ηχογραφήσεων."),
            ("fixed",
             "Smaller fixes: the stop key, dragging the bar, shortcut fields, dollar signs in "
             "snippets, sentence punctuation, the newest log in Diagnostics and the screen reader "
             "language.",
             "Μικρότερες διορθώσεις: το πλήκτρο διακοπής, το σύρσιμο της μπάρας, τα πεδία "
             "συντομεύσεων, το σύμβολο του δολαρίου στα αποσπάσματα, η στίξη στις προτάσεις, το "
             "νεότερο αρχείο στα Διαγνωστικά και η γλώσσα των αναγνωστών οθόνης."),
            ("known",
             "A blank white window sometimes appears above the listening indicator.",
             "Ένα λευκό παράθυρο εμφανίζεται μερικές φορές πάνω από την ένδειξη ακρόασης."),
            ("known",
             "Text in a third language written in Latin letters can still slip through.",
             "Κείμενο τρίτης γλώσσας με λατινικά γράμματα μπορεί ακόμα να περάσει."),
        ],
    },
    {
        "version": "0.9.8",
        "date": "2026-09-17",
        "summary": (
            "The language lock now checks every part of a dictation.",
            "Το κλείδωμα γλώσσας ελέγχει πια κάθε κομμάτι της υπαγόρευσης.",
        ),
        "lines": [
            ("fixed",
             "The language lock from 0.9.7 now works. Each part of a dictation is checked and "
             "heard again in your language when it strays, and one word in another alphabet is "
             "enough to catch it.",
             "Το κλείδωμα γλώσσας της 0.9.7 δουλεύει πια. Κάθε κομμάτι της υπαγόρευσης ελέγχεται "
             "και ακούγεται ξανά στη γλώσσα σου όταν ξεφύγει. Αρκεί και μία λέξη σε άλλο "
             "αλφάβητο για να το πιάσει."),
            ("known",
             "When the last part of a dictation has to be heard again, the text arrives up to a "
             "second and a half later.",
             "Όταν το τελευταίο κομμάτι μιας υπαγόρευσης χρειάζεται δεύτερο άκουσμα, το κείμενο "
             "φτάνει ως ενάμισι δευτερόλεπτο αργότερα."),
            ("known",
             "A single English word said on its own can come out in the letters of your language.",
             "Μια αγγλική λέξη ειπωμένη μόνη της μπορεί να βγει με τα γράμματα της γλώσσας σου."),
        ],
    },
    {
        "version": "0.9.7",
        "date": "2026-09-16",
        "summary": (
            "Your dictation language stays yours, thirty-three languages to choose from, a bar "
            "you can place anywhere, and a crash fix.",
            "Η γλώσσα της υπαγόρευσης μένει η δική σου, τριάντα τρεις γλώσσες για να διαλέξεις, "
            "μπάρα που μπαίνει όπου θέλεις και μια διόρθωση κατάρρευσης.",
        ),
        "lines": [
            ("added",
             "Choose your language in Settings, Language, from thirty-three. The modes are your "
             "language with English, your language only, English only, or auto-detect. A new "
             "install starts with the language of Windows.",
             "Διαλέγεις τη γλώσσα σου στις Ρυθμίσεις, Γλώσσα, ανάμεσα σε τριάντα τρεις. Οι "
             "λειτουργίες είναι η γλώσσα σου με αγγλικά, μόνο η γλώσσα σου, μόνο αγγλικά ή "
             "αυτόματη ανίχνευση. Μια νέα εγκατάσταση ξεκινά με τη γλώσσα των Windows."),
            ("added",
             "Drag the bar to any edge of the screen and it stays there. Settings, Overlay can "
             "show it, so you can move it without dictating.",
             "Σέρνεις τη μπάρα σε όποια άκρη της οθόνης θέλεις και μένει εκεί. Οι Ρυθμίσεις, "
             "Μπάρα τη δείχνουν, για να τη μετακινήσεις χωρίς να υπαγορεύεις."),
            ("improved",
             "The discreet style is a single dot: red while listening, green when done, amber "
             "when the words wait on the clipboard.",
             "Η διακριτική εμφάνιση είναι μία τελεία: κόκκινη όσο ακούει, πράσινη όταν τελειώσει, "
             "πορτοκαλί όταν οι λέξεις περιμένουν στο πρόχειρο."),
            ("fixed",
             "In the mixed mode a short phrase could come out in an unrelated language. Anything "
             "outside your two languages is now heard again in yours.",
             "Στη μεικτή λειτουργία μια σύντομη φράση μπορούσε να βγει σε άσχετη γλώσσα. Ό,τι "
             "βγαίνει έξω από τις δύο γλώσσες σου ακούγεται πια ξανά στη δική σου."),
            ("fixed",
             "The app could close during the first hour after Windows started.",
             "Η εφαρμογή μπορούσε να κλείσει την πρώτη ώρα μετά την εκκίνηση των Windows."),
            ("fixed",
             "The Windows clipboard history (Win+V) could leave a paste empty.",
             "Το ιστορικό προχείρου των Windows (Win+V) μπορούσε να αφήσει άδεια μια επικόλληση."),
        ],
    },
    {
        "version": "0.9.6",
        "date": "2026-09-11",
        "summary": (
            "Updates that announce themselves, a faster paste, mouse button shortcuts, a steadier "
            "hotkey and many fixes. It includes the unreleased 0.9.4 and 0.9.5.",
            "Ενημερώσεις που ανακοινώνονται μόνες τους, πιο γρήγορη επικόλληση, συντομεύσεις στα "
            "κουμπιά του ποντικιού, πιο σταθερό πλήκτρο και πολλές διορθώσεις. Περιλαμβάνει και "
            "τις 0.9.4 και 0.9.5, που δεν δημοσιεύτηκαν.",
        ),
        "lines": [
            ("added",
             "The app checks for a new version shortly after it opens and shows an Update now "
             "card, waiting until you finish if you are dictating. Check for updates is also in "
             "the tray menu.",
             "Η εφαρμογή ψάχνει νέα έκδοση λίγο αφού ανοίξει και δείχνει μια κάρτα Ενημέρωση "
             "τώρα. Αν υπαγορεύεις εκείνη τη στιγμή, περιμένει να τελειώσεις. Ο Έλεγχος για "
             "ενημερώσεις υπάρχει και στο μενού του εικονιδίου."),
            ("added",
             "The side buttons of a mouse (Mouse4, Mouse5) can be shortcuts, alone or with Ctrl, "
             "Shift and Alt.",
             "Τα πλαϊνά κουμπιά του ποντικιού (Mouse4, Mouse5) γίνονται συντομεύσεις, μόνα τους "
             "ή με Ctrl, Shift και Alt."),
            ("added",
             "Diagnostics has a Key check that shows live what your hotkey sends.",
             "Τα Διαγνωστικά έχουν Έλεγχο πλήκτρου που δείχνει ζωντανά τι στέλνει το πλήκτρο σου."),
            ("improved",
             "Pasting is faster and more reliable: the words are on the clipboard before the key "
             "press.",
             "Η επικόλληση είναι πιο γρήγορη και πιο αξιόπιστη: οι λέξεις μπαίνουν στο πρόχειρο "
             "πριν πατηθεί το πλήκτρο."),
            ("improved",
             "Dictated words stay in the Windows clipboard history (Win+V) and are kept out of "
             "the cloud clipboard.",
             "Οι λέξεις που υπαγορεύεις μένουν στο ιστορικό προχείρου των Windows (Win+V) και "
             "μένουν έξω από το πρόχειρο του σύννεφου."),
            ("improved",
             "The hotkey recovers within a second when a busy Windows drops it.",
             "Το πλήκτρο επανέρχεται μέσα σε ένα δευτερόλεπτο όταν ένα φορτωμένο σύστημα το χάσει."),
            ("improved",
             "The program, its folders and its files carry the product name. Your settings, "
             "history and models move over by themselves.",
             "Το πρόγραμμα, οι φάκελοί του και τα αρχεία του έχουν το όνομα του προϊόντος. Οι "
             "ρυθμίσεις, το ιστορικό και τα μοντέλα σου μεταφέρονται μόνα τους."),
            ("fixed",
             "On keyboards where the right Alt types accented letters and symbols, typing them "
             "no longer starts a dictation.",
             "Σε πληκτρολόγια όπου το δεξί Alt γράφει τονισμένα γράμματα και σύμβολα, η "
             "πληκτρολόγησή τους δεν ξεκινά πια υπαγόρευση."),
            ("fixed",
             None,
             "Η καταγραφή συντόμευσης με το δεξί Alt σε ελληνικό πληκτρολόγιο αποθηκεύεται πια "
             "σωστά."),
            ("fixed",
             "Deleting a History entry that came from an audio file keeps your original file.",
             "Η διαγραφή μιας εγγραφής του Ιστορικού που ήρθε από αρχείο ήχου κρατάει το αρχικό "
             "σου αρχείο."),
            ("fixed",
             "Pictures, files and formatted text on the clipboard come through a dictation "
             "unchanged.",
             "Εικόνες, αρχεία και μορφοποιημένο κείμενο στο πρόχειρο μένουν άθικτα μετά από "
             "υπαγόρευση."),
            ("fixed",
             "If the last part of a long dictation fails, the rest is kept.",
             "Αν αποτύχει το τελευταίο κομμάτι μιας μεγάλης υπαγόρευσης, κρατιέται το υπόλοιπο."),
            ("fixed",
             "Installing over an older version closes the old one first, and your settings "
             "survive a file that cannot be read for a moment. Delete all data also removes the "
             "backups.",
             "Η εγκατάσταση πάνω σε παλαιότερη έκδοση κλείνει πρώτα την παλιά και οι ρυθμίσεις "
             "σου αντέχουν ένα αρχείο που δεν διαβάζεται για μια στιγμή. Η Διαγραφή όλων των "
             "δεδομένων σβήνει και τα αντίγραφα ασφαλείας."),
            ("fixed",
             "Smaller fixes in History (corrections, Paste, Copy), in Settings (shortcuts, the "
             "Save button, the startup switch, retention) and in the note window, which now "
             "follows your language and wraps its text.",
             "Μικρότερες διορθώσεις στο Ιστορικό (διορθώσεις, Επικόλληση, Αντιγραφή), στις "
             "Ρυθμίσεις (συντομεύσεις, κουμπί Αποθήκευση, διακόπτης εκκίνησης, διατήρηση) και στο "
             "μικρό παράθυρο, που ακολουθεί πια τη γλώσσα σου και αναδιπλώνει το κείμενο."),
        ],
    },
    {
        "version": "0.9.3",
        "date": "2026-09-08",
        "summary": (
            "Two small fixes to updating.",
            "Δύο μικρές διορθώσεις στην ενημέρωση.",
        ),
        "lines": [
            ("fixed",
             "The new version bar repeated a word.",
             "Η μπάρα νέας έκδοσης επαναλάμβανε μια λέξη."),
            ("fixed",
             "An automatic check could interrupt an update that was already downloading.",
             "Ένας αυτόματος έλεγχος μπορούσε να διακόψει μια ενημέρωση που κατέβαινε ήδη."),
        ],
    },
    {
        "version": "0.9.2",
        "date": "2026-09-08",
        "summary": (
            "Updates from inside the app, recordings turned into text, and a window that keeps "
            "your words when a paste cannot land.",
            "Ενημερώσεις μέσα από την εφαρμογή, ηχογραφήσεις που γίνονται κείμενο και ένα "
            "παράθυρο που κρατάει τις λέξεις σου όταν μια επικόλληση δεν βρίσκει στόχο.",
        ),
        "lines": [
            ("added",
             "The app checks for a newer version at each start. Nothing downloads until you "
             "press the button.",
             "Η εφαρμογή ελέγχει για νεότερη έκδοση σε κάθε άνοιγμα. Δεν κατεβαίνει τίποτα μέχρι "
             "να πατήσεις το κουμπί."),
            ("added",
             "Turn a recording you already have into text: Home, Sound file to text, up to 90 "
             "minutes.",
             "Μια ηχογράφηση που έχεις ήδη γίνεται κείμενο: Αρχική, Αρχείο ήχου σε κείμενο, έως "
             "90 λεπτά."),
            ("added",
             "When a dictation cannot reach the program you were in, a small window shows the "
             "words. You can switch it off in Settings, Insertion.",
             "Όταν μια υπαγόρευση δεν φτάνει στο πρόγραμμα που ήσουν, ένα μικρό παράθυρο δείχνει "
             "τις λέξεις. Το κλείνεις από τις Ρυθμίσεις, Εισαγωγή κειμένου."),
            ("added",
             "A graphics card left switched off is turned on when it can be used.",
             "Μια κάρτα γραφικών που είχε μείνει κλειστή ανοίγει όταν μπορεί να χρησιμοποιηθεί."),
            ("improved",
             "Speech models move to your data folder on the first run, which keeps updates near "
             "70 MB. They stay when you uninstall.",
             "Τα μοντέλα ομιλίας μεταφέρονται στον φάκελο δεδομένων σου στο πρώτο άνοιγμα, οπότε "
             "οι ενημερώσεις μένουν γύρω στα 70 MB. Μένουν εκεί και αν απεγκαταστήσεις."),
            ("improved",
             "Crashes are written to the logs, so they can be looked into.",
             "Οι καταρρεύσεις γράφονται στα αρχεία καταγραφής, ώστε να μπορούν να ερευνηθούν."),
            ("fixed",
             "A rare crash while reading the clipboard.",
             "Ένα σπάνιο κλείσιμο της εφαρμογής την ώρα που διάβαζε το πρόχειρο."),
            ("fixed",
             "Smaller fixes: the note window's buttons and theme, the Windows startup entry, and "
             "the Settings page keeping what you type.",
             "Μικρότερες διορθώσεις: τα κουμπιά και το θέμα του μικρού παραθύρου, η αυτόματη "
             "εκκίνηση των Windows και η σελίδα Ρυθμίσεις, που κρατάει ό,τι γράφεις."),
        ],
    },
    {
        "version": "0.9.1",
        "date": "2026-09-06",
        "summary": (
            "Clear reporting when a paste goes nowhere, and an interface that follows your "
            "language everywhere.",
            "Καθαρή ενημέρωση όταν μια επικόλληση δεν πάει πουθενά και περιβάλλον που ακολουθεί "
            "τη γλώσσα σου παντού.",
        ),
        "lines": [
            ("fixed",
             "A dictation aimed at the desktop or the taskbar is reported: the text stays on the "
             "clipboard and the app says which window was in front.",
             "Μια υπαγόρευση προς την επιφάνεια εργασίας ή τη γραμμή εργασιών αναφέρεται: το "
             "κείμενο μένει στο πρόχειρο και η εφαρμογή λέει ποιο παράθυρο ήταν μπροστά."),
            ("fixed",
             "Model descriptions follow the interface language.",
             "Οι περιγραφές των μοντέλων ακολουθούν τη γλώσσα της εφαρμογής."),
            ("added",
             "A new install picks up the language of Windows. Both the interface and the "
             "dictation language can be changed in Settings, Language.",
             "Μια νέα εγκατάσταση παίρνει τη γλώσσα των Windows. Η γλώσσα της εφαρμογής και η "
             "γλώσσα υπαγόρευσης αλλάζουν από τις Ρυθμίσεις, Γλώσσα."),
            ("added",
             "Settings, Speech models shows which device the engine runs on.",
             "Οι Ρυθμίσεις, Μοντέλα ομιλίας δείχνουν σε ποια συσκευή τρέχει η μηχανή."),
            ("added",
             "An event log keeps seven days of what happened and what failed. It never holds "
             "the text you dictate.",
             "Ένα ημερολόγιο συμβάντων κρατάει επτά μέρες από ό,τι έγινε και ό,τι απέτυχε. Το "
             "κείμενο που υπαγορεύεις δεν μπαίνει ποτέ μέσα."),
        ],
    },
    {
        "version": "0.9.0",
        "date": "2026-09-06",
        "summary": (
            "The first public release: one key, two languages in one sentence, everything on "
            "your own computer.",
            "Η πρώτη δημόσια έκδοση: ένα πλήκτρο, δύο γλώσσες στην ίδια πρόταση και τα πάντα "
            "στον δικό σου υπολογιστή.",
        ),
        "lines": [
            ("added",
             "Hold the right Alt, speak, press it again. The text appears where your cursor is, "
             "in any program, and two languages can share one sentence.",
             "Κρατάς το δεξί Alt, μιλάς, το πατάς ξανά. Το κείμενο εμφανίζεται εκεί που είναι ο "
             "κέρσορας, σε οποιοδήποτε πρόγραμμα. Δύο γλώσσες χωράνε στην ίδια πρόταση."),
            ("added",
             "The installer includes everything, with graphics acceleration on AMD, Intel and "
             "NVIDIA cards. Nothing downloads later.",
             "Ο installer τα έχει όλα μέσα, με επιτάχυνση σε κάρτες γραφικών AMD, Intel και "
             "NVIDIA. Τίποτα δεν κατεβαίνει αργότερα."),
            ("added",
             "The first start looks at your computer and picks the model and settings that fit.",
             "Το πρώτο άνοιγμα εξετάζει τον υπολογιστή σου και διαλέγει το μοντέλο και τις "
             "ρυθμίσεις που ταιριάζουν."),
        ],
    },
]
