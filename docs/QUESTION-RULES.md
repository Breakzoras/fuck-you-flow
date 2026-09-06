# Question detection rules for Greek and English dictation

Purpose: decide from words alone (no intonation, no audio features) whether a
sentence produced by speech recognition is a question and should end with the
Greek question mark `;` (U+037E or ASCII `;`) or the English `?`.

Guiding principle: a wrong question mark is worse than a missing one. Every
rule below is lexical and positional. Where a cue is unreliable it is listed,
graded, and left off by default. Pure yes/no questions with no lexical cue
("Ήρθε ο Γιώργος;", "Έφυγε το τρένο;") are marked by intonation only
(Arvaniti; school grammar: "δεν υπάρχει ερωτηματική αντωνυμία") and are
therefore undetectable here. Accept the miss.

Input assumptions: one sentence at a time, tokenised on whitespace, Greek
accents (tonos) preserved as the ASR model writes them, case-insensitive
matching, punctuation from the ASR (commas) may or may not be present.

Rhetorical questions still take `;` in Greek orthography (the school grammar
classes them as ερωτηματικές), so "Ποιος ξέρει;" and "Τι να κάνουμε;" are
correct with a question mark. Only exclamatives take `!`.

---

## 1. Greek interrogative pronouns, adverbs and particles

### 1.1 ποιος, ποια, ποιο (who, which)

Declines like adjectives in -ος, -α, -ο, with extra genitive forms in -ανού and
an optional accusative -ν. Never accented (monosyllabic in modern spelling).
Source: school grammar Ε-ΣΤ Δημοτικού 8.7, Gymnasium grammar ch. 5, Wiktionary.

| Case | Masc. sg. | Fem. sg. | Neut. sg. | Masc. pl. | Fem. pl. | Neut. pl. |
|---|---|---|---|---|---|---|
| Nom. | ποιος | ποια | ποιο | ποιοι | ποιες | ποια |
| Gen. | ποιου, ποιανού | ποιας, ποιανής | ποιου, ποιανού | ποιων, ποιανών | ποιων, ποιανών | ποιων, ποιανών |
| Acc. | ποιον, ποιο | ποια, ποιαν | ποιο | ποιους, ποιανούς | ποιες | ποια |

Formal genitive across all genders: τίνος (sg.), τίνων (pl.).
"Τίνος παιδιού είναι το βιβλίο;" (Gymnasium grammar).

Full token list for matching: ποιος ποια ποιο ποιοι ποιες ποιον ποιαν ποιου
ποιας ποιων ποιους ποιανού ποιανής ποιανών ποιανούς τίνος τίνων.

Spelling trap: ποιο (which, neuter) versus πιο (more). "Ποιο τραγούδι ακούς;
Το πιο ωραίο." (Gymnasium grammar). Only ποιο triggers.

### 1.2 πόσος, πόση, πόσο (how much, how many)

Declines like adjectives in -ος, -η, -ο (Wiktionary table):
πόσος πόση πόσο πόσου πόσης πόσοι πόσες πόσα πόσων πόσους.
"Πόσα μολύβια έχεις στην τσάντα σου;" (Gymnasium grammar).
πόσο is also the interrogative adverb of quantity ("Πόσο κάνει;").
Set phrases that are usually questions: πόσων χρονών, πόση ώρα, πόσο καιρό,
κατά πόσο(ν), μέχρι πόσο, από πόσο.
Trap: όσο, όσος, όση, όσοι, όσα (relative, "as much as") never trigger.
"πόσο μάλλον" (let alone) is a connective and never triggers.

### 1.3 τι, τίνος

τι is indeclinable and unaccented ("Τι κάνεις;"). It is the single most
frequent question opener in dictation ("Τι ώρα είναι;", "Τι λες;",
"Τι εννοείς;", "Τι θα κάνουμε;").
Never-trigger neighbours: ότι (that), ό,τι (whatever, which the ASR will
usually write as ότι anyway), κάτι, τίποτα, τίποτε, οτιδήποτε, "αν μη τι
άλλο", "ολίγον τι". Match the bare token τι only.

### 1.4 Interrogative adverbs

Accented forms that trigger: πού (where), πώς (how), πότε (when), πόσο (how
much), γιατί (why). Compound and prepositional forms: από πού, προς τα πού,
για πού, πούθε (dialectal), από πότε, μέχρι πότε, ως πότε, έως πότε, για
πότε, πώς και (how come: "Πώς και ήρθες;"), γιατί όχι, πώς έτσι.

School grammar (Ε-ΣΤ Δημοτικού 12.1) lists these as the head of each adverb
class: τοπικά answer "Πού;", χρονικά "Πότε;", τροπικά "Πώς;", ποσοτικά
"Πόσο;". γιατί as a question word is treated by the syntax sources as an
introducer of ερωτήσεις μερικής άγνοιας alongside the pronouns (users.sch.gr).

### 1.5 Accent distinctions (the accent is the whole rule)

| Accented (question) | Unaccented (never a question) | Source |
|---|---|---|
| πού: "Πού πήγες;" | που: relative pronoun or conjunction, "Αυτό που σου είπα" | Sarantakos 2023, mixanitouxronou, school grammar |
| πώς: "Πώς τα περνάς;" | πως = ότι (that): "Είπε πως θα έρθει" | same |
| πότε: "Πότε φεύγεις;" | ποτέ (never): "Ποτέ δεν το είπα" | accent position differs |
| πόσο: "Πόσο κάνει;" | όσο, τόσο | relative and demonstrative |
| ποιο (which) | πιο (more) | Gymnasium grammar |
| τι | ότι, ό,τι, κάτι, τίποτα | Gymnasium grammar |
| γιατί (sentence-initial) | γιατί mid-sentence after a clause = because | see 1.7 |

Official rule (Sarantakos quoting the school grammar): the interrogatives πού
and πώς keep the accent in direct and in indirect questions ("Δε μας είπες
πού πήγες"), and also in the idioms πού και πού, πώς και πώς, πού να σου τα
λέω. The relative που and the conjunction πως are never accented.

ASR caveat: Sarantakos measured that most writers omit the accent on the
interrogative about 93 percent of the time on social media. An ASR model
trained on such text may emit που for πού. Measure on Lalia's own transcripts
before relying on this. If the model writes που, the accented rule silently
never fires, which is the safe failure. Never treat unaccented που or πως as
a question cue: "που" as relative and "πως" as "that" are far more frequent.

### 1.6 Interrogative particles

| Particle | Function | Reliability without intonation |
|---|---|---|
| μήπως (sentence-initial) | opens a yes/no question of doubt or a polite suggestion: "Μήπως θέλεις να βγούμε;", "Μήπως ήρθε ο Γιώργος;", rhetorical "Μήπως δεν τον βοήθησα τόσες φορές;" (Wiktionary en and el) | High at sentence start. Mid-sentence it is a conjunction meaning "in case, lest": "Φοβάμαι μήπως βρέξει.", "Πάρε μια ζακέτα μήπως και κρυώσεις." (el.wiktionary), and after ρωτώ it opens an indirect question: "Η Τατιάνα ρώτησε μήπως μπορούμε να της χαρίσουμε το φόρεμα." (no mark) |
| άραγε (anywhere) | pure question particle, "I wonder": "Γιατί άραγε αργεί τόσο πολύ;", "άραγε θάρθουν πάλι απόψε;" (Wiktionary en and el) | High anywhere in the sentence. The only non-question use is the proscribed confusion with άρα (therefore), which both Wiktionaries flag as an error |
| μπας και (sentence-initial) | colloquial μήπως expressing fear that something happens: "Βρε μπας και μας κατάλαβαν και δε μας αφήσουν να φύγουμε;" (Λεξικό Κοινής Νεοελληνικής) | High at sentence start. Mid-sentence it means "in case": "πήγαινε στους δίπλα, μπας και έχουν λίγη ζάχαρη" (el.wiktionary), no mark |
| λες να, λέτε να (sentence-initial) | "do you think that": "Λες να έρθει;" | High at sentence start (idiomatic, always a question) |
| τάχα | question particle in older or literary use; in modern speech mostly "supposedly, allegedly": "Ήτανε λέει τάχα άρρωστος." (el.wiktionary) | Low. Leave off |
| αν, μη(ν), μην τυχόν | open indirect yes/no questions only ("Είναι ζήτημα αν έφαγε") | Never a trigger on its own; see 1.8 |

### 1.7 Where the question word can stand

1. Sentence-initial: the normal case for direct questions. Trigger.
2. After a sentence-initial preposition: σε ποιον, σε ποια, με τι, με ποιον,
   για ποιο λόγο, για ποιον, από πότε, από πού, μέχρι πότε, ως πότε, κατά
   πόσο, χωρίς τι, μετά από πόσο, προς τα πού, εναντίον ποιου. Trigger.
   Wiktionary example: "Σε ποιον το έδωσες;".
3. After a vocative or discourse opener: "Γιώργο, πού είσαι;", "Λοιπόν, τι
   κάνουμε;", "Και τώρα, τι;", "Ε, τι γίνεται εδώ;" (el.wiktionary ε).
   Allow up to two leading tokens that are a proper name, λοιπόν, και, αλλά,
   ε, ρε, βρε, μα, τελικά, εντάξει followed by a comma if the ASR gave one.
4. Mid-sentence after a verb of asking or knowing: an indirect question
   (πλάγια ερωτηματική). See 1.8.
5. Mid-sentence γιατί with no asking verb before it means "because"
   ("Δεν ήρθα γιατί έβρεχε."). No trigger.
6. Mid-sentence πού, πώς, πότε, τι, ποιος after anything else (relative-like
   free relatives, "Ξέρω εγώ τι θέλω.", "Έμαθα πότε φεύγει."): indirect
   question with a declarative matrix. No trigger.

### 1.8 Indirect questions: when the mark is allowed

The Modern Greek syntax sources (users.sch.gr; Gymnasium grammar; the scribd
and slideshare handouts derived from them) agree: an indirect question is a
subordinate clause that depends on a verb of asking, doubting, knowing or
perceiving (ρωτώ, εξετάζω, απορώ, αμφιβάλλω, λέω, εξηγώ, σκέφτομαι, θυμάμαι,
βλέπω, μαθαίνω, ακούω, καταλαβαίνω, ξέρω, αναρωτιέμαι), is introduced by αν,
μη, μήπως, μην τυχόν (ολικής άγνοιας) or by τι, ποιος, πόσος, πού, πότε,
πώς, πόσο, γιατί (μερικής άγνοιας), and ends with a full stop because the
question is being reported: "Με ρώτησε τι ώρα είναι."

The sentence takes `;` only when the main clause is itself a question.
Decision rule, applied to the tokens before the first interrogative word:

- Matrix is a second-person verb of knowing, remembering, perceiving or
  saying, or a second-person ability verb plus a speech verb, at or near the
  sentence start: ξέρεις, ξέρετε, θυμάσαι, θυμάστε, καταλαβαίνεις, κατάλαβες,
  βλέπεις, είδες, άκουσες, έμαθες, μου λες, μου λέτε, μπορείς να μου πεις,
  μπορείτε να μου πείτε, θέλεις να μάθεις, έχεις ιδέα, έχεις καταλάβει.
  Result: `;`. "Ξέρεις τι ώρα είναι;", "Θυμάσαι πού το έβαλες;",
  "Μπορείς να μου πεις πότε φεύγει;". High confidence.
- Matrix is imperative (πες μου, πείτε μου, εξήγησέ μου, δείξε μου, θυμήσου,
  κοίτα, δες, μάθε, ρώτα): the sentence is a command. No mark. "Πες μου πώς
  το έκανες." High confidence.
- Matrix is first or third person or negated (ξέρω, δεν ξέρω, δεν θυμάμαι,
  αναρωτιέμαι, απορώ, μου είπε, με ρώτησε, θα δούμε, είναι ζήτημα, εξαρτάται,
  δεν έχει σημασία): no mark. "Μου είπε τι ώρα είναι.", "Δεν ξέρω πού
  αποφάσισε να πάει τελικά" (Lexilogia). High confidence.
- Matrix unclear or the interrogative word is preceded by more than six
  tokens with no recognised verb: no mark (conservative default).

The same rule also handles "Ξέρεις αν έρχεται;" (`;`) versus "Δεν ξέρω αν
έρχεται." (no mark), with αν as the introducer.

---

## 2. Yes/no questions with no question word: lexical cues and honesty

| Cue | Example that is a question | Reliability without intonation | Recommendation |
|---|---|---|---|
| Sentence-initial μήπως | "Μήπως είδες το παιδί μου;" | High. The subordinate sense needs a preceding verb | Enable |
| άραγε anywhere | "Θα έρθει άραγε;" | High | Enable |
| Sentence-initial μπας και | "Μπας και το ξέχασες;" | High at sentence start only | Enable |
| Sentence-initial λες να / λέτε να | "Λες να χάσαμε το τρένο;" | High | Enable |
| Sentence-initial να + first-person verb, short sentence | "Να έρθω;", "Να φύγουμε;", "Να σου πω κάτι;" | Medium. Deliberative subjunctive questions are typically first person, but "Να πάω εγώ." can be a decision and "Να ζήσεις!", "Να έρθεις αύριο." are wishes or commands | Off by default; opt-in only for first person, six tokens or fewer |
| Second-person verb first: Θέλεις, Μπορείς, Έχεις, Είσαι, Ξέρεις, Πας, Έρχεσαι, Θα έρθεις | "Θέλεις καφέ;", "Μπορείς να με πάρεις;" | Low. "Μπορείς να φύγεις." (permission), "Έχεις δίκιο.", "Θέλεις να σου πω κάτι. Άκου." are statements with identical openers. This is the largest false-positive source | Off by default |
| Sentence-initial Δεν + second-person verb | "Δεν έρχεσαι;" | Low. Negative statements dominate | Off |
| Tag: έτσι δεν είναι, δεν είναι έτσι, ή όχι, δε(ν) νομίζεις, δεν συμφωνείς, δεν είναι | "Είναι ωραία εδώ, έτσι δεν είναι;", "Θα έρθεις ή όχι;" | High when sentence-final. Multiword tags have no other reading | Enable |
| Tag: έτσι, ε | "Θα το κάνεις, έτσι;", "Θα έρθεις, ε;" | High if the ASR wrote a comma before the tag. Without a comma "έτσι" is ambiguous ("Το έκανα έτσι.") and "ε" may be a filler | Enable only with comma, or when "ε" is the final token after a clause of three or more words |
| Tag: σωστά | "Έχεις πληρώσει το ρεύμα, σωστά;" (GreekPod101) | Medium. "Το έκανες σωστά." is an adverb | Enable only with comma |
| Tag: εντάξει, ναι, οκ | "Στις οκτώ, εντάξει;" | Medium. "Όλα πήγαν εντάξει.", "Είπε ναι." | Enable only with comma |
| Tag: όχι | "Θα έρθεις, όχι;" | Low. "Απάντησε όχι." | Off |
| One-word utterances: Αλήθεια, Σοβαρά, Ναι, Ορίστε, Παρακαλώ, Δηλαδή | "Αλήθεια;", "Ορίστε;" | Medium. "Ορίστε." (here you are) and "Ναι." are common answers | Off; the wh one-word rule (G3) stays on |
| No cue at all | "Ήρθε ο Γιώργος;" | Undetectable. Arvaniti shows the L* H-L% melody carries the whole contrast | Accept the miss |

---

## 3. Greek counter-examples: same opener, no question mark

1. Exclamatives with τι and πόσο. The Gymnasium grammar's own example of an
   επιφωνηματική πρόταση is "Τι ωραίο γραπτό!". Pattern: τι followed by an
   adjective or adverb of evaluation, or a noun, with no finite verb in the
   clause; πόσο followed by an evaluative adverb or a verb of feeling.
   Blocklist for τι (no mark, `!` optional): τι ωραία, τι ωραίο, τι ωραίος,
   τι όμορφα, τι καλά, τι κρίμα, τι υπέροχα, τι φοβερό, τι χαρά, τι βλακεία,
   τι ντροπή, τι μέρα, τι κόσμος, τι λες (ambiguous, leave unmarked),
   τι θαυμάσια. Blocklist for πόσο: πόσο σε αγαπώ, πόσο χαίρομαι, πόσο μου
   έλειψες, πόσο ωραία, πόσο όμορφα, πόσο καλά, πόσο πολύ, πόσο λυπάμαι.
   Anything else after sentence-initial τι or πόσο stays a question:
   "Τι ώρα είναι;", "Τι νούμερο φοράς;", "Πόσο κάνει;".
2. "τι κι αν", "τι και αν" (concessive, "even if"): "Τι κι αν έβρεχε, βγήκαμε."
   No mark. High confidence.
3. "πού να" idioms of resignation (el.wiktionary marks them with `!` or
   trailing dots): πού να ξέρω, πού να το ξέρω, πού να ξέρεις, πού να 'ξερα,
   πού να σου τα λέω, πού να σας τα λέω, πού να κοιμηθώ, πού να τρέχω τώρα,
   πού να πάει το μυαλό σου. No mark. Other "Πού να" clauses stay questions:
   "Πού να πάμε;", "Πού να το βάλω;", and Wiktionary itself writes
   "Πού να το φανταζόμουν;" with `;`. Medium confidence for the split.
4. "πού και πού" (now and then), "πώς και πώς" (eagerly), "πώς όχι" (of
   course), "πώς!" (emphatic yes) inside or ending a statement: no mark.
5. Relative που: "Το βιβλίο που διάβασα.", "Χαίρομαι που ήρθες." Never.
6. πως = that: "Είδα πως είχε έρθει η γυναίκα που περίμενα" (Lexilogia).
   Never.
7. όσο, όσος, όποιος, όπου, όπως, όποτε, οπότε: relative or indefinite.
   "Όσο περιμένω, διαβάζω." Never. The ASR may confuse όποτε (whenever) and
   οπότε (so); neither triggers.
8. Ποτέ (never) at sentence start: "Ποτέ δεν το είπα." The accent sits on
   the last syllable; πότε (when) has it on the first. Never.
9. γιατί as "because" mid-sentence, and as a bare answer "Γιατί έτσι." in
   dialogue. Only sentence-initial γιατί with at least one following token
   triggers; an isolated "Γιατί" does trigger (it is the one-word question).
10. Indirect questions with a declarative matrix (section 1.8): "Μου είπε τι
    ώρα είναι.", "Ρώτησε τι ώρα είναι." (users.sch.gr), "Είναι απίστευτο πόσο
    με ταλαιπωρείς" (users.sch.gr, exclamative matrix, no mark).
11. Imperative matrix: "Πες μου πού είσαι.", "Δείξε μου πώς γίνεται."

---

## 4. English

### 4.1 Question openers

Wh-words (Wikipedia, English interrogative words; Cambridge Grammar): who,
whom, whose, what, which, when, where, why, how, plus compounds how much, how
many, how long, how often, how old, how far, how come, what time, what kind,
what about, how about, and the archaic whence, whither, wherefore. whether
and if only ever open embedded yes/no questions.

Yes/no questions use subject-auxiliary inversion: a sentence-initial auxiliary
or modal (do, does, did, am, is, are, was, were, have, has, had, can, could,
will, would, shall, should, may, might, must, plus negated contractions isn't,
aren't, don't, doesn't, didn't, can't, couldn't, won't, wouldn't, shouldn't,
haven't, hasn't) followed by a subject: a pronoun (I, you, he, she, it, we,
they), there, this, that, or a determiner plus noun.

Prepositional openers count as wh-openers: "To whom", "In which", "For how
long", "At what time", "By when", "From where", "With what".

Tags (grammarbook.com rule 4, "half statement and half question"): "You do
care, don't you?" A sentence-final auxiliary-plus-pronoun tag (isn't it, is
it, aren't they, don't you, doesn't she, didn't he, won't you, can't we, will
you, shall we) or the words right, correct, okay, yes, no, huh, eh after a
comma.

### 4.2 English counter-examples

1. Embedded (indirect) questions with a declarative matrix take a period
   (grammarbook.com rule 3a, Wordvice, UVic): "I wonder who is coming to the
   party.", "I wonder if he would go with me.", "I don't know where she
   went.", "It depends on what he says.", "That is why I left.", "This is how
   it works.", "No matter what happens.", "Whatever you decide is fine.",
   "Tell me what you want." (imperative matrix), "Let me know when you
   arrive.", "She asked whether I was coming." Blocking frames: I wonder,
   I'm not sure, I don't know, I know, nobody knows, it depends, that's why,
   this is how, here is what, no matter, whatever, whoever, whenever,
   wherever, however (sentence-initial, "However, ..."), tell me, let me
   know, ask, asked, explain, show me, remember to, find out, figure out.
2. The embedded question keeps `?` only when the matrix is itself a question:
   "Do you know what time it is?", "Can you tell me where the station is?",
   "Any idea when he left?" These are already caught by the inversion rule
   or by the sentence-initial "Any idea".
3. Exclamatives (Wikipedia): only what and how form them. "What a lovely
   day!", "What a mess.", "How nice it is!", "How wonderful." Pattern: "What
   a", "What an", "What" plus adjective plus noun with no auxiliary; "How"
   plus adjective or adverb with no auxiliary immediately after it. No mark.
4. Imperatives that start with an auxiliary-looking verb: "Do it now.",
   "Have a seat.", "Be careful.", "Do the dishes." The inversion rule must
   require a pronoun or there/this/that subject, so "Do you" fires and "Do
   the" does not. Accept the miss on "Is the door locked" (determiner
   subject) unless you enable the determiner variant at medium confidence.
5. Relative and free-relative uses mid-sentence: "The man who called.",
   "I know what you mean.", "That is where I live.", "The reason why."
   Never trigger mid-sentence.
6. Rhetorical requests ("Why don't you take a break.", grammarbook.com rule
   3b) are a stylistic exception; marking them `?` is standard and acceptable.
7. Reported speech: "He said what he thought." No mark (mid-sentence rule).

---

## 5. Compact rule table

Confidence: High (enable by default), Medium (opt-in), Low (leave off).
"Blocker" rules suppress the mark and always run before trigger rules.

| ID | Pattern | Conf. | Gets a mark | Must not | Default |
|---|---|---|---|---|---|
| G1 | Sentence-initial Greek wh-token (1.1 to 1.4 lists) followed by one or more tokens, after blockers G14 to G17 | High | Ποιος είναι ο επισκέπτης; | Τι ωραίο γραπτό! (G14) | On |
| G2 | Sentence-initial preposition + wh-token (σε, με, για, από, μέχρι, ως, έως, προς, κατά, χωρίς, μετά, πριν, εναντίον) | High | Σε ποιον το έδωσες; | Σε αυτόν που είπες. | On |
| G3 | Whole utterance is a single wh-token (Τι, Πού, Πώς, Πότε, Γιατί, Ποιος, Πόσο) | High | Γιατί; | Ναι. | On |
| G4 | Sentence-initial μήπως (up to two discourse tokens before it) | High | Μήπως θέλεις να βγούμε; | Φοβάμαι μήπως βρέξει. | On |
| G5 | άραγε anywhere in the sentence | High | Γιατί άραγε αργεί τόσο πολύ; | (only the proscribed άρα confusion) | On |
| G6 | Sentence-initial μπας και | High | Μπας και μας κατάλαβαν; | Πήγαινε δίπλα, μπας και έχουν ζάχαρη. | On |
| G7 | Sentence-initial λες να / λέτε να | High | Λες να έρθει; | Λες ότι θα έρθει. | On |
| G8a | Sentence-final multiword tag: έτσι δεν είναι, δεν είναι έτσι, ή όχι, δε(ν) νομίζεις, δεν συμφωνείς, δεν είναι | High | Είναι ωραία εδώ, έτσι δεν είναι; | Δεν είναι εδώ. (only when "δεν είναι" ends a longer clause with a comma before it) | On |
| G8b | Sentence-final one-word tag after a comma: έτσι, ε, σωστά, εντάξει, ναι | Medium | Θα έρθεις, ε; | Το έκανα έτσι. | Opt-in, comma required |
| G8c | Sentence-final όχι | Low | Θα έρθεις, όχι; | Απάντησε όχι. | Off |
| G9 | Second-person knowing/asking matrix at sentence start + wh-token or αν (list in 1.8) | High | Ξέρεις τι ώρα είναι; | (see G10) | On |
| G10 | Blocker: first/third-person, negated or imperative matrix before the wh-token or αν | High | (none) | Μου είπε τι ώρα είναι. Πες μου πού είσαι. Δεν ξέρω αν έρχεται. | On |
| G11 | Sentence-initial να + first-person verb, six tokens or fewer | Medium | Να έρθω; | Να ζήσεις! | Off |
| G12 | Sentence-initial second-person verb without any other cue (Θέλεις, Μπορείς, Έχεις, Είσαι) | Low | Θέλεις καφέ; | Μπορείς να φύγεις. | Off |
| G13 | Sentence-initial Δεν + second-person verb | Low | Δεν έρχεσαι; | Δεν ήρθες χτες. | Off |
| G14 | Blocker: exclamative τι/πόσο collocations (3.1 list) | High | (none) | Τι ωραία! Πόσο σε αγαπώ! | On |
| G15 | Blocker: τι κι αν, τι και αν, ότι, ό,τι, κάτι, τίποτα, πόσο μάλλον, πού και πού, πώς και πώς, πού να ξέρω family | High | (none) | Τι κι αν έβρεχε, βγήκαμε. | On |
| G16 | Blocker: unaccented που, πως, and όσο/όποιος/όπου/όπως/όποτε/οπότε/ποτέ never count as wh-tokens | High | (none) | Είπε πως θα έρθει. Αυτό που σου είπα. | On |
| G17 | Blocker: wh-token that is neither sentence-initial (after G2/1.7 openers) nor governed by a G9 matrix | High | (none) | Δεν ήρθα γιατί έβρεχε. Έμαθα πότε φεύγει. | On |
| G18 | τάχα as question particle | Low | Τάχα θα έρθει; | Ήταν τάχα άρρωστος. | Off |
| E1 | Sentence-initial wh-word (4.1 list) with one or more following tokens, after E6/E7 | High | What time is it? | What a mess. | On |
| E2 | Sentence-initial preposition + wh-word | High | To whom did you send it? | To the man who called. | On |
| E3 | Sentence-initial auxiliary/modal (incl. negated contractions) + pronoun/there/this/that | High | Do you have a minute? Isn't it late? | Do it now. Have a seat. | On |
| E3b | Same with a determiner + noun subject | Medium | Is the door locked? | Do the dishes. | Off |
| E4a | Sentence-final auxiliary + pronoun tag, or ", right" | High | You do care, don't you? | You know I do. | On |
| E4b | Sentence-final ", okay", ", correct", ", yes", ", no", ", huh" | Medium | We leave at eight, okay? | It went okay. | Opt-in, comma required |
| E5 | Interrogative matrix + embedded wh/if/whether ("Do you know", "Can you tell me", "Any idea") | High | Do you know where she went? | (see E6) | On (mostly subsumed by E3) |
| E6 | Blocker: declarative or imperative matrix before the wh-word (I wonder, I don't know, tell me, let me know, that's why, this is how, no matter, whatever...) | High | (none) | I wonder who is coming. Tell me what you want. | On |
| E7 | Blocker: exclamative "What a/an", "What + adj + noun" with no auxiliary, "How + adj/adv" with no auxiliary | High | (none) | What a lovely day! How nice. | On |
| E8 | Single-word wh utterance | High | Why? | Okay. | On |
| E9 | Single-word Really, Okay, Sorry, Pardon | Medium | Really? | Okay. (agreement) | Off |

Ordering inside the engine: run blockers (G10, G14 to G17, E6, E7) first;
then G1 to G9 and E1 to E5 with High confidence; then any opt-in rules. If two
rules disagree, the blocker wins. Output exactly one mark, `;` for Greek input
and `?` for English, and only when the ASR left the sentence without a
terminal `;`, `?`, `!` or `.`. Whisper-family models sometimes emit `;` or `?`
themselves from intonation; keep whatever mark the ASR already wrote. These
rules only add a mark.

---

## Sources

Greek grammars and dictionaries

- Manolis Triantafyllidis, Νεοελληνική Γραμματική (1941), first edition PDF:
  https://www.openbook.gr/neoelliniki-grammatiki/ (the accent and pronoun
  rules in sections 1.1 to 1.5 follow this tradition as codified in the two
  school grammars below).
- Γραμματική Ε΄ και ΣΤ΄ Δημοτικού, 8.7 Ερωτηματικές αντωνυμίες:
  https://ebooks.edu.gr/ebooks/v/html/8547/2009/Grammatiki_E-ST-Dimotikou_html-apli/index_C8g.html
  and 12.1 Είδη επιρρημάτων:
  https://ebooks.edu.gr/ebooks/v/html/8547/2009/Grammatiki_E-ST-Dimotikou_html-apli/index_C12a.html
- Γραμματική Νέας Ελληνικής Γλώσσας Α΄ Β΄ Γ΄ Γυμνασίου (Χατζησαββίδης), ch. 5
  Αντωνυμίες:
  https://ebooks.edu.gr/ebooks/v/html/8547/2334/Grammatiki-Neas-Ellinikis-Glossas_A-B-G-Gymnasiou_html-apli/index_C_05.html
  and ch. D1 Είδη προτάσεων (ολικής/μερικής άγνοιας, επιφωνηματικές):
  https://ebooks.edu.gr/ebooks/v/html/8547/2334/Grammatiki-Neas-Ellinikis-Glossas_A-B-G-Gymnasiou_html-apli/index_D_01.html
- Ονοματικές πλάγιες ερωτηματικές προτάσεις (school syntax notes):
  http://users.sch.gr/elnas/joomla2015/index.php/mathimata/gymnasio/nea-elliniki-glossa/syntaktiko-n-e-glossas/48-onomatikes-plagies-erotimatikes-protaseis
- Λεξικό της Κοινής Νεοελληνικής (Ίδρυμα Τριανταφυλλίδη), entry μπας και:
  https://greek-language.gr/greekLang/modern_greek/tools/lexica/triantafyllides/search.html?lq=%22%CE%BC%CF%80%CE%B1%CF%82+%CE%BA%CE%B1%CE%B9%22
  (site returns 403 to automated fetches; example taken from the search snippet).
- Kleris and Babiniotis, Γραμματική της Νέας Ελληνικής (2005), record only:
  https://www.greek-language.gr/greekLang/modern_greek/bibliographies/grammar/09_klairis-babibiotis/index.html
  (text not reachable online; the ολικής/μερικής άγνοιας terminology matches
  the school grammar).
- Holton, Mackridge, Philippaki-Warburton, Spyropoulos, Greek: A Comprehensive
  Grammar of the Modern Language, 2nd ed. (Routledge 2012):
  https://www.routledge.com/Greek-A-Comprehensive-Grammar-of-the-Modern-Language/Holton-Mackridge-Philippaki-Warburton-Spyropoulos/p/book/9780415592024
  (basis for the statement that yes/no questions keep declarative word order
  and are marked by intonation or by μήπως, άραγε, τάχα; the online preview
  was unreadable, so section numbers are omitted).
- Arvaniti, The intonation of yes-no questions in Greek (L* H-L% melody):
  https://ejournals.lib.auth.gr/thal/article/view/6195
- Sarantakos, Εσείς βάζετε τόνο στα (ερωτηματικά) «πού» και «πώς»; (2023):
  https://sarantakos.wordpress.com/2023/04/20/poupws/
- Lexilogia, Πλάγιες ερωτηματικές προτάσεις και τόνος:
  https://www.lexilogia.gr/threads/9899/
- Μηχανή του Χρόνου, Πότε τονίζονται οι λέξεις πού, πως, τι, ποιος:
  https://www.mixanitouxronou.gr/pote-tonizonte-i-lexis-pou-pos-ti-pios-ke-alles-monosillaves-i-kanones-tonismou-pou-echoume-xechasi-ke-ta-pio-sichna-lathi-sto-grapto-logo/
- Wiktionary (English), entries ποιος, πόσος, μήπως, άραγε:
  https://en.wiktionary.org/wiki/ποιος (replace the last path segment).
- Wiktionary (Greek), entries ποιος, μήπως, άραγε, τάχα, μπας_και, τι, πού, ε:
  https://el.wiktionary.org/wiki/ποιος (replace the last path segment).
- GreekPod101, Tag questions with σωστά:
  https://www.greekpod101.com/lesson/mustknow-greek-sentence-structures-22-tag-questions

English

- GrammarBook, Question Marks (rules 1, 3a, 3b, 4):
  https://www.grammarbook.com/punctuation/qMarks.asp
- Wikipedia, English interrogative words:
  https://en.wikipedia.org/wiki/English_interrogative_words
- Cambridge Grammar, Questions: interrogative pronouns:
  https://dictionary.cambridge.org/grammar/british-grammar/questions-interrogative-pronouns-what-who
- UVic Study Zone, Embedded questions:
  https://continuingstudies.uvic.ca/elc/studyzone/410/grammar/410-embedded-questions
