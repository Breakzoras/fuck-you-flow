# -*- coding: utf-8 -*-
"""The plain-words guide: voice to text on a Windows PC, in English and Greek.

People rarely search for "dictation software". They type "voice to text",
"speak to text", "voice typing" or "talk to text on pc". Keyword Planner, all
locations, Sept 2025 to Aug 2026, measured on 16 September 2026: "voice to text"
and "speak to text" 100k to 1m a month, "voice typing" and "speech to text free"
10k to 100k, and "voice to text windows", "voice to text pc", "speech to text
windows", "speech to text for pc", "voice typing windows", "talk to text" and
"speak to type" 1k to 10k each. This page answers all of them in one place.

The Windows voice typing facts come from Microsoft's support page
https://support.microsoft.com/en-us/windows/use-voice-typing-to-talk-instead-of-type-on-your-pc-fec94565-c4bd-329d-e59a-af033fa5689f
read on 16 September 2026: Windows key + H, online speech recognition powered by
Azure Speech services, an internet connection required, and a Windows 11
language list without Greek. Recheck them before changing any sentence here.
"""

MS_VOICE_TYPING = ("https://support.microsoft.com/en-us/windows/"
                   "use-voice-typing-to-talk-instead-of-type-on-your-pc-fec94565-c4bd-329d-e59a-af033fa5689f")

VOICE_TO_TEXT = {
    "slug": "voice-to-text-pc",
    "en": {
        "title": "Free Voice to Text on PC: Talk and It Types",
        "description": ("Free voice to text for Windows 10 and 11. Talk and it types in any app, "
                        "offline and with no word limit. Speech to text compared with Windows voice "
                        "typing (Win+H)."),
        "lead": ("Voice to text, speech to text, voice typing, talk to text and dictation are five names "
                 "for one job: you speak and your words appear where the cursor is. A Windows PC can do "
                 "it with the built-in voice typing or with a free local app such as FU Flow. This guide "
                 "shows both and where each one fits."),
        "blocks": [
            ("h2", "Voice to text, speech to text and voice typing are the same thing"),
            ("p", "Every one of these names describes software that listens to your microphone and writes "
                  "what you say as text. Some people call it speak to type or talk to text, others call it "
                  "dictation. On a computer the text goes into the program you are already using: a "
                  "document, an email, a chat window or an AI prompt box."),
            ("p", "Two things decide which tool suits you: where your voice is turned into text, on your PC "
                  "or on a server, and whether the tool speaks your language."),
            ("h2", "Option 1: Windows voice typing (Win+H)"),
            ("p", "Windows has voice typing built in. Microsoft's support page, read on 16 September 2026, "
                  "says it starts with the Windows key and H, uses online speech recognition powered by "
                  "Azure Speech services, and needs an internet connection, a working microphone and the "
                  "cursor in a text box."),
            ("p", "The same page lists the voice typing languages for Windows 11: more than forty, from "
                  "Bulgarian to Vietnamese. That list leaves out Greek."),
            ("h2", "Option 2: free offline voice to text with FU Flow"),
            ("p", "FU Flow is a free, open-source voice to text app for Windows 10 and 11, released under "
                  "MIT. The speech models come inside the installer and run on your PC, so dictation keeps "
                  "working without internet once it is installed. There is no account, no subscription "
                  "and no weekly word limit for the local engine."),
            ("p", "You pick your own language from thirty-three in Settings, and the default mode takes "
                  "English words mixed into your sentences. When the engine guesses a third language, "
                  "the recording is heard again in the one you chose."),
            ("table", ["", "Windows voice typing", "FU Flow"], [
                ["Start", "Windows key + H", "Right Alt, or a key you choose"],
                ["Where speech becomes text", "Microsoft online speech recognition", "On your PC by default"],
                ["Internet", "Required, per Microsoft", "Not needed for the local engine"],
                ["Greek", "Missing from Microsoft's Windows 11 list", "Supported, with English mixed in"],
                ["Cost", "Included with Windows", "Free, MIT licensed"],
                ["Word limit", "Microsoft publishes no cap on that page", "None for the local engine"],
            ]),
            ("h2", "How to talk to text on a PC with FU Flow"),
            ("ol", ["Download the Windows installer from this page. The speech models are included, so the "
                    "first dictation works without a separate download.",
                    "Open Settings, pick the microphone you actually use and choose your language.",
                    "Click into any text box: Word, Gmail, Slack, a browser or a chat with an AI assistant.",
                    "Press right Alt, say a sentence, then press right Alt again. The text appears at the cursor.",
                    "Read it once. Add names the engine gets wrong to the Dictionary, and it spells them your "
                    "way from then on."]),
            ("h2", "Tips for accurate speech to text"),
            ("ul", ["Use a headset or a microphone close to your mouth. Room echo costs more accuracy than "
                    "an accent does.",
                    "Speak in whole sentences with short pauses. The engine punctuates from the rhythm of speech.",
                    "Keep the mixed mode if you use English terms in your own language, and your own language "
                    "alone if you never do.",
                    "Teach the Dictionary your names, brands and jargon once.",
                    "Say a question the way you would ask it. A rising voice at the end gets a question mark."]),
            ("h2", "What your computer needs"),
            ("p", "Windows 10 or 11 and room for the installer, which is about 1.7 GB with the models. A "
                  "graphics card from NVIDIA, AMD or Intel speeds it up through Vulkan. Without one it runs "
                  "on the processor, more slowly on a low-power PC."),
            ("links", [("/guides/word-dictation/", "Voice typing in Word"),
                       ("/guides/ai-prompts/", "Voice typing for ChatGPT and Claude"),
                       ("/wispr-flow-alternative/", "FU Flow compared with Wispr Flow"),
                       ("/free-offline-dictation-alternatives/", "Best free dictation software for Windows"),
                       (MS_VOICE_TYPING, "Microsoft: voice typing on a Windows PC")]),
        ],
        "faq": [
            ("Is there free voice to text for Windows?",
             "Yes. Windows includes voice typing (Windows key + H), which Microsoft says needs an internet "
             "connection. FU Flow is a free, open-source app that turns voice to text on your PC without "
             "internet after installation, with no word limit."),
            ("Can I use speech to text offline on a PC?",
             "Yes, with an app that runs the speech model on your computer. FU Flow ships its models inside "
             "the installer, so speech to text keeps working with the network off."),
            ("How do I type with my voice in Word, Gmail or ChatGPT?",
             "Click where you want the text, press the dictation key, speak, and press it again. FU Flow "
             "inserts the words at the cursor in any Windows program that accepts typing."),
            ("Does Windows voice typing support Greek?",
             "Greek is missing from the Windows 11 voice typing language list on Microsoft's support page, "
             "read on 16 September 2026. FU Flow dictates Greek, including English words inside Greek "
             "sentences."),
            ("What is the difference between dictation and voice typing?",
             "None in practice. Dictation, voice typing, speech to text and voice to text all mean speaking "
             "and getting written text. Tools differ in where the speech is processed, which languages they "
             "handle and what they cost."),
            ("Is there a word limit?",
             "FU Flow has no word limit for its local engine. Some subscription dictation apps cap free "
             "use, for example Wispr Flow's free desktop plan at 2,000 words a week."),
        ],
    },
    "el": {
        "title": "Φωνητική πληκτρολόγηση στα ελληνικά, δωρεάν για Windows",
        "description": ("Δωρεάν φωνητική πληκτρολόγηση και υπαγόρευση κειμένου στα ελληνικά για Windows 10 "
                        "και 11. Μιλάτε και γράφει σε κάθε πρόγραμμα, χωρίς ίντερνετ και χωρίς όριο λέξεων."),
        "lead": ("Φωνητική πληκτρολόγηση, υπαγόρευση κειμένου, μετατροπή φωνής σε κείμενο: τρία ονόματα "
                 "για την ίδια δουλειά. Μιλάτε και οι λέξεις σας γράφονται εκεί που είναι ο κέρσορας. Ο "
                 "οδηγός δείχνει τι προσφέρουν τα Windows και πώς γράφετε ελληνικά με τη φωνή σας με το "
                 "δωρεάν FU Flow."),
        "blocks": [
            ("h2", "Τι είναι η φωνητική πληκτρολόγηση"),
            ("p", "Είναι πρόγραμμα που ακούει το μικρόφωνο και γράφει όσα λέτε. Θα το βρείτε και ως "
                  "υπαγόρευση, μετατροπή ομιλίας σε κείμενο ή, στα αγγλικά, voice to text και speech to "
                  "text. Το κείμενο μπαίνει στο πρόγραμμα που ήδη χρησιμοποιείτε: έγγραφο, email, "
                  "συνομιλία ή εντολή σε βοηθό AI."),
            ("p", "Δύο πράγματα κρίνουν ποιο εργαλείο σάς ταιριάζει. Πρώτο, πού γίνεται η μετατροπή της "
                  "φωνής: στον υπολογιστή σας ή σε διακομιστή. Δεύτερο, αν το εργαλείο μιλάει ελληνικά."),
            ("h2", "Η φωνητική πληκτρολόγηση των Windows (Win+H)"),
            ("p", "Τα Windows έχουν ενσωματωμένη φωνητική πληκτρολόγηση. Η σελίδα υποστήριξης της Microsoft, "
                  "όπως τη διαβάσαμε στις 16 Σεπτεμβρίου 2026, αναφέρει ότι ξεκινά με το πλήκτρο Windows και "
                  "το H, ότι χρησιμοποιεί online αναγνώριση ομιλίας από τις υπηρεσίες Azure Speech και ότι "
                  "χρειάζεται σύνδεση στο ίντερνετ."),
            ("p", "Στην ίδια σελίδα, η λίστα γλωσσών φωνητικής πληκτρολόγησης για Windows 11 έχει πάνω από "
                  "σαράντα γλώσσες. Τα ελληνικά δεν περιλαμβάνονται."),
            ("h2", "Δωρεάν υπαγόρευση στα ελληνικά με το FU Flow"),
            ("p", "Το FU Flow είναι δωρεάν εφαρμογή ανοιχτού κώδικα για Windows 10 και 11, με άδεια MIT. Τα "
                  "μοντέλα ομιλίας έρχονται μέσα στην εγκατάσταση και τρέχουν στον υπολογιστή σας. Μετά την "
                  "εγκατάσταση γράφετε με τη φωνή σας και χωρίς ίντερνετ, χωρίς λογαριασμό, συνδρομή ή "
                  "εβδομαδιαίο όριο λέξεων για την τοπική μηχανή."),
            ("p", "Η προεπιλογή δέχεται ελληνικά μαζί με αγγλικές λέξεις μέσα στην ίδια πρόταση. Όταν η "
                  "μηχανή μαντέψει τρίτη γλώσσα, η ηχογράφηση ακούγεται ξανά στα ελληνικά."),
            ("table", ["", "Φωνητική πληκτρολόγηση Windows", "FU Flow"], [
                ["Έναρξη", "Πλήκτρο Windows + H", "Δεξί Alt ή πλήκτρο της επιλογής σας"],
                ["Πού γίνεται η μετατροπή", "Online αναγνώριση ομιλίας της Microsoft", "Στον υπολογιστή σας από προεπιλογή"],
                ["Ίντερνετ", "Απαραίτητο, κατά τη Microsoft", "Περιττό για την τοπική μηχανή"],
                ["Ελληνικά", "Εκτός της λίστας Windows 11 της Microsoft", "Υποστηρίζονται, μαζί με αγγλικά"],
                ["Κόστος", "Μέρος των Windows", "Δωρεάν, άδεια MIT"],
                ["Όριο λέξεων", "Η σελίδα της Microsoft δεν αναφέρει όριο", "Κανένα για την τοπική μηχανή"],
            ]),
            ("h2", "Πώς γράφετε με τη φωνή σας στον υπολογιστή"),
            ("ol", ["Κατεβάστε την εγκατάσταση για Windows από αυτή τη σελίδα. Τα μοντέλα ομιλίας "
                    "περιλαμβάνονται, άρα η πρώτη υπαγόρευση δουλεύει αμέσως.",
                    "Ανοίξτε τις Ρυθμίσεις, διαλέξτε το μικρόφωνο που χρησιμοποιείτε και κρατήστε τα ελληνικά "
                    "ως γλώσσα σας.",
                    "Πατήστε μέσα σε οποιοδήποτε πεδίο κειμένου: Word, Gmail, Viber, πρόγραμμα περιήγησης ή "
                    "συνομιλία με βοηθό AI.",
                    "Πατήστε το δεξί Alt, πείτε μια πρόταση και πατήστε ξανά το δεξί Alt. Το κείμενο εμφανίζεται "
                    "στη θέση του κέρσορα.",
                    "Διαβάστε το μία φορά. Προσθέστε στο Λεξικό ονόματα που γράφονται λάθος και από εκεί και "
                    "πέρα γράφονται όπως τα θέλετε."]),
            ("h2", "Συμβουλές για σωστό κείμενο"),
            ("ul", ["Προτιμήστε ακουστικά με μικρόφωνο ή μικρόφωνο κοντά στο στόμα. Η ηχώ του δωματίου "
                    "κοστίζει περισσότερο από την προφορά.",
                    "Μιλήστε με ολόκληρες προτάσεις και μικρές παύσεις. Η στίξη βγαίνει από τον ρυθμό της ομιλίας.",
                    "Κρατήστε τη μεικτή λειτουργία αν χρησιμοποιείτε αγγλικούς όρους και τα σκέτα ελληνικά "
                    "αν δεν χρησιμοποιείτε ποτέ.",
                    "Μάθετε στο Λεξικό ονόματα, επωνυμίες και ορολογία της δουλειάς σας μία φορά.",
                    "Κάντε την ερώτηση με τον τόνο ερώτησης. Η ανεβασμένη φωνή στο τέλος φέρνει ερωτηματικό."]),
            ("h2", "Τι χρειάζεται ο υπολογιστής"),
            ("p", "Windows 10 ή 11 και χώρο για την εγκατάσταση, περίπου 1,7 GB μαζί με τα μοντέλα. Κάρτα "
                  "γραφικών NVIDIA, AMD ή Intel την επιταχύνει μέσω Vulkan. Χωρίς κάρτα τρέχει στον "
                  "επεξεργαστή, πιο αργά σε αδύναμο υπολογιστή."),
            ("links", [("/el/guides/word-dictation/", "Υπαγόρευση κειμένου στο Word"),
                       ("/el/guides/ai-prompts/", "Φωνητική πληκτρολόγηση για ChatGPT και Claude"),
                       ("/el/guides/greek-and-english/", "Ελληνικά και αγγλικά στην ίδια πρόταση"),
                       ("/el/wispr-flow-alternative/", "Σύγκριση FU Flow και Wispr Flow"),
                       (MS_VOICE_TYPING, "Microsoft: φωνητική πληκτρολόγηση στα Windows (στα αγγλικά)")]),
        ],
        "faq": [
            ("Υπάρχει δωρεάν φωνητική πληκτρολόγηση στα ελληνικά για Windows;",
             "Ναι. Το FU Flow είναι δωρεάν εφαρμογή ανοιχτού κώδικα που γράφει ελληνικά με τη φωνή σας σε "
             "Windows 10 και 11, χωρίς ίντερνετ μετά την εγκατάσταση και χωρίς όριο λέξεων."),
            ("Η φωνητική πληκτρολόγηση των Windows γράφει ελληνικά;",
             "Τα ελληνικά λείπουν από τη λίστα γλωσσών φωνητικής πληκτρολόγησης για Windows 11 στη σελίδα "
             "υποστήριξης της Microsoft, όπως τη διαβάσαμε στις 16 Σεπτεμβρίου 2026."),
            ("Γίνεται υπαγόρευση κειμένου χωρίς ίντερνετ;",
             "Ναι, με εφαρμογή που τρέχει το μοντέλο ομιλίας στον υπολογιστή σας. Το FU Flow έχει τα μοντέλα "
             "μέσα στην εγκατάσταση και συνεχίζει να γράφει με το δίκτυο κλειστό."),
            ("Πώς γράφω με τη φωνή μου στο Word ή στο Gmail;",
             "Πατάτε εκεί που θέλετε το κείμενο, πατάτε το πλήκτρο υπαγόρευσης, μιλάτε και το πατάτε ξανά. "
             "Οι λέξεις μπαίνουν στη θέση του κέρσορα σε κάθε πρόγραμμα Windows που δέχεται πληκτρολόγηση."),
            ("Γράφει αγγλικές λέξεις μέσα σε ελληνικές προτάσεις;",
             "Ναι. Η προεπιλεγμένη μεικτή λειτουργία δέχεται ελληνικά και αγγλικά στην ίδια πρόταση. Ονόματα "
             "που γράφονται λάθος διορθώνονται μία φορά στο Λεξικό."),
            ("Υπάρχει όριο λέξεων;",
             "Το FU Flow δεν έχει όριο λέξεων για την τοπική μηχανή. Κάποιες εφαρμογές με συνδρομή περιορίζουν "
             "τη δωρεάν χρήση, για παράδειγμα το δωρεάν πρόγραμμα υπολογιστή του Wispr Flow στις 2.000 λέξεις "
             "την εβδομάδα."),
        ],
    },
}
