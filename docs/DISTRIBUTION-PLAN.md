# FU Flow: where to list it, and how

Research only. Nothing was submitted anywhere and no account was created.
Prepared 6 September 2026 for "Fuck You Flow" / "FU Flow" 0.9.0, https://fuckyouflow.app

Note on this file: `site/README.md` says the deploy step copies everything in this folder to the
server. Add `DISTRIBUTION-PLAN.md` to the exclusion list, or move it up one level, so it stays
private.

---

## Summary table, ordered by priority

| # | Where | Type | Value | New account needed | Cost |
|---|---|---|---|---|---|
| 1 | winget (microsoft/winget-pkgs) | package manager | HIGH | No, existing GitHub | Free |
| 2 | AlternativeTo, plus the Wispr Flow page | software directory | HIGH | Yes | Free, optional 5 USD queue skip |
| 3 | Hacker News, Show HN | launch platform | HIGH | Yes | Free |
| 4 | r/software | community | HIGH | Yes, Reddit | Free |
| 5 | r/LocalLLaMA | community | HIGH | Same Reddit account | Free |
| 6 | Scoop, your own bucket | package manager | MEDIUM | No, existing GitHub | Free |
| 7 | whisper.cpp Show and tell | community | MEDIUM to HIGH | No, existing GitHub | Free |
| 8 | r/opensource | community | MEDIUM to HIGH | Same Reddit account | Free |
| 9 | Chocolatey community repository | package manager | MEDIUM to HIGH | Yes, chocolatey.org | Free |
| 10 | Product Hunt | launch platform | MEDIUM to HIGH | Yes, personal profile | Free |
| 11 | r/SideProject | community | MEDIUM | Same Reddit account | Free |
| 12 | r/coolgithubprojects | community | MEDIUM | Same Reddit account | Free |
| 13 | Scoop Extras bucket | package manager | MEDIUM | No, existing GitHub | Free |
| 14 | pluja/awesome-privacy | open-source index | MEDIUM | No, existing GitHub | Free |
| 15 | 0PandaDEV/awesome-windows | open-source index | MEDIUM | No, existing GitHub | Free |
| 16 | OpenAI Whisper Show and tell | community | MEDIUM | No, existing GitHub | Free |
| 17 | SourceForge, mirror plus listing | directory and file host | MEDIUM | Yes, plus SMS check | Free |
| 18 | Softpedia | software directory | MEDIUM | Probably no | Free |
| 19 | Uptodown | app catalogue | MEDIUM | Yes | Free |
| 20 | MajorGeeks | software directory | MEDIUM | No, email pitch | Free |
| 21 | Slant | community comparison | MEDIUM | Yes | Free |
| 22 | DevHunt | launch platform | MEDIUM | No, existing GitHub | Free |
| 23 | Indie Hackers products | community directory | LOW to MEDIUM | Yes | Free |
| 24 | SaaSHub | software directory | LOW to MEDIUM | Yes | Free |
| 25 | sindresorhus/awesome-whisper | open-source index | MEDIUM, blocked today | No, existing GitHub | Free |
| 26 | FSF Free Software Directory | open-source index | LOW to MEDIUM | Yes, wiki account | Free |
| 27 | Privacy Guides, Project Showcase | privacy index | LOW to MEDIUM | Yes, forum account | Free |
| 28 | PRISM Break | privacy index | LOW | No, GitLab or GitHub | Free |
| 29 | LibHunt | open-source index | LOW | No | Free |
| 30 | FossHub | download host | LOW to MEDIUM | Yes | Free |
| 31 | awesome-talon | niche list | LOW to MEDIUM | No, existing GitHub | Free |
| 32 | Greek FOSS sites (dwrean.net, ellak.gr, opensoft.gr) | community media | LOW to MEDIUM | No, email | Free |
| 33 | Uneed | launch platform | LOW | Yes | Free tier plus paid |
| 34 | Peerlist Launchpad | launch platform | LOW | Yes, personal | Free |
| 35 | MicroLaunch | launch platform | LOW | Yes | Free tier, boost from 39 USD |
| 36 | Fazier | launch platform | LOW | Yes | Free plus paid |
| 37 | Launching Next | launch platform | LOW | Unclear | Free |
| 38 | There's An AI For That | AI directory | LOW | Yes | 49 USD, or 347 USD |
| 39 | OpenTools, TopAI.tools, opensourcesai.com | AI directories | LOW | Yes | Free tiers exist |
| 40 | Softonic | software directory | LOW | Yes | Free listing |
| 41 | openalternative.co | open-source index | LOW | No | 97 USD minimum |
| 42 | opensourcealternative.to | open-source index | LOW | No | Free queue, or 29 USD |
| 43 | Futurepedia, Toolify, aitools.fyi | AI directories | SKIP | Yes | 99 to 497 USD |
| 44 | Accessibility and AT catalogues | institutional | LOW | Varies, email | Free |

**Do not attempt:** Microsoft Store (paid developer account plus a content policy the name fails),
Scoop main bucket (needs 500 stars and 150 forks and a non-graphical tool), BetaList (accepts only
products that are unreleased or launched days ago, and now charges), Lobsters (invite only, and new
accounts are barred for 70 days from the "show" tag and from new domains), FileHippo (does not
accept publisher submissions), The Portable Freeware Collection (collects only installer-free
apps), download.cnet.com (submissions reportedly go unprocessed), AbleData (shut down in 2020),
awesome-selfhosted (covers hosted services).

---

## Measured facts this plan rests on

| Fact | Measured value | Method |
|---|---|---|
| Installer size | 1,535,457,121 bytes, so 1.43 GB | HTTP HEAD on the release asset |
| Repo created | 6 September 2026, today | GitHub API |
| Stars and forks | 0 and 0 | GitHub API |
| Licence as GitHub reads it | "Other" (NOASSERTION) | GitHub licence API |
| Downloads of v0.9.0 | 13 | GitHub API |
| Repo homepage field | Empty | GitHub API |
| Largest installer already in winget | Nvidia.CUDA at 2,531,940,368 bytes, so 2.53 GB | winget show plus HTTP HEAD |
| Profanity already inside a winget manifest | Tag `mountain-fuck` on Iterate.MountainDuck | winget show, live index |
| Packages named with the word in winget | Zero | winget search, live index |
| Packages named with the word in Chocolatey | Zero, while "vlc" returns six | Chocolatey search, live feed |

---

## Fix these five things before submitting anywhere

Under an hour of work in total, and each one removes a rejection reason.

1. **The LICENSE file does not read as MIT to machines.** It carries correct MIT text with a
   third-party note appended after a horizontal rule, so GitHub's detector returns "Other" and the
   repo shows no MIT badge. Every index that scrapes the GitHub licence field sees "Other", and
   r/software and r/opensource both gate on a recognised open-source licence. Fix: keep `LICENSE`
   as pure MIT text and move the third-party sentence to a `NOTICE` file or the README. Confirm the
   GitHub sidebar afterwards reads "MIT license".
2. **Set the repo homepage field** to `https://fuckyouflow.app`. It is empty today.
3. **Publish a portable ZIP asset beside the installer.** It makes the Scoop manifest simple, gives
   cautious users a route around the SmartScreen warning, and removes FossHub's bundled-installer
   objection. One change, three listings improved.
4. **Prove the installer runs unattended, and watch the language dialog.** Both winget and
   Chocolatey require a silent install. `src-tauri/tauri.conf.json` was read directly: the bundle
   target is `nsis`, the install mode is `currentUser`, and the silent switch is therefore `/S`.
   The same file sets `"displayLanguageSelector": true` with Greek and English, so the installer
   opens a language chooser. **That dialog is the specific thing to test**, because a prompt that
   survives silent mode fails winget's unattended-install requirement and Chocolatey's automated
   verifier at the same time. Run `Fuck.You.Flow.Setup.0.9.0.exe /S` in a clean Windows Sandbox,
   confirm it finishes with zero clicks, and record the Add and Remove Programs entry values, which
   both manifests need. If the chooser appears under `/S`, set `displayLanguageSelector` to false
   for the packaged build.
5. **Build the asset pack once and reuse it.** Directories keep asking for the same items:
   - icon at 256x256 and 512x512 PNG (both exist in `site/assets/`)
   - four to six screenshots, 1280x800 or larger PNG (six exist in `site/assets/`)
   - Product Hunt specifically: thumbnail 240x240, at least two gallery images at 1270x760, each
     file under 3 MB
   - a 40 character tagline, a 60 character tagline, a 160 character short description, and a 500
     to 1500 character long description, in English and Greek
   - the installer SHA256, already published in `SHA256SUMS.txt`
   - a 30 to 60 second screen recording on YouTube, which Product Hunt and several directories
     reward

---

# THE NAME PROBLEM

The app is called "Fuck You Flow". The effect is smaller than you would expect in some places and
absolute in others. Here is what the evidence shows, quoted where a written policy exists.

## The evidence, measured

**The word already survives inside the Windows package ecosystem.** The winget community repository
ships today a manifest for Mountain Duck by Iterate GmbH carrying the tag `mountain-fuck`, read
from the live index with `winget show Iterate.MountainDuck`. So the automated keyword scanner is no
blanket substring ban.

**No package is actually named with the word.** A live winget search for "fuck" returns exactly one
hit, that tag. A Chocolatey community feed search returns zero packages, while the control search
for "vlc" returns six. So there is no precedent for a displayed package name carrying it.

**On community platforms the word is normal.** `nvbn/thefuck` has roughly 97,000 stars on GitHub and
appears across awesome-lists. AlternativeTo carries a live catalogue entry for "The Fuck" at
alternativeto.net/software/the-fuck. SourceForge hosts three projects with the word in the name
(`the-fuck.mirror`, `fuck-zuck-social`, `fuckff`). On Hacker News, "Show HN: FuckFuckAdblock"
reached 353 points on 14 December 2015 and "Show HN: JFDIN, Just Fucking Do It Now" reached 63
points on 13 March 2013, both confirmed through the HN Algolia search API.

## The written policies, quoted

**winget.** Microsoft Learn, Windows Package Manager repository policies, section 2.9, "Excessive
Profanity and Inappropriate Content":

> The Product must not contain excessive or gratuitous profanity.
> The Product must not contain or display content that a reasonable person would consider to be obscene.

Section 2.1, "General Content Requirements":

> Metadata and other content you submit to accompany your submission may contain only content that
> would merit a rating of PEGI 12, ESRB EVERYONE 10+, or lower.

Section 2.5, "Offensive Content":

> The Product and associated metadata must not contain potentially sensitive or offensive content.

The winget FAQ describes the check that enforces this:

> The last automated check is a content validation to ensure that the package description and other
> metadata fields don't violate one of the policies in place such as those against excessively
> profane language or adult content.

The repository has a pull request label for it, `Policy-Test-2.9`, listed in
`doc/ValidationFailureGuide.md` under "Content Policy Labels", which states that such a pull request
"will undergo additional manual review". A second label, `Internal-Error-Keyword-Policy`, is
described as "Error during manifest content policy validation".

**Softonic.** The only directory found with a written, quotable ban. Its content policy bars content
"likely to shock or disgust", naming "content using profane language", and its software policy adds
that Softonic "reserves the right to immediately reject any software" that breaches the policy or is
"considered harmful to its image". Use "FU Flow" there, with no exceptions.

**Chocolatey.** No profanity clause was found, but its packaging guide states that the package title
"should be the same as the name of the application", and every new package passes human moderation,
where a package id that breaks conventions is "grounds for rejecting immediately".

**Everyone else.** No written profanity policy was found at AlternativeTo, Softpedia, MajorGeeks,
Uptodown, FossHub, SourceForge, Product Hunt, awesome-whisper, awesome-windows, awesome-privacy,
Privacy Guides or PRISM Break. Every code of conduct located is a Contributor Covenant governing
contributor behaviour, which says nothing about project branding. The risk at those places is
editorial discretion.

## The trap: the word is in more than the name field

Listing as "FU Flow" does not by itself produce a clean submission, because the word also sits in:

- the repo slug, `github.com/Breakzoras/fuck-you-flow`
- the release asset filename, `Fuck.You.Flow.Setup.0.9.0.exe`
- therefore the installer URL, which is a required field in every package manifest
- the domain `fuckyouflow.app`, which fills the homepage and support URL fields

A human moderator approves every winget manifest and every Chocolatey package, and they will see all
four. So the choice is real:

**Option A, submit as "FU Flow" with today's URLs.** Cheapest. Expect the `Policy-Test-2.9` label and
manual review. The outcome is uncertain and any rejection is public in the pull request thread.

**Option B, add a clean download path first.** Publish the same installer under a neutral filename
such as `FUFlow-Setup-0.9.0.exe` and serve it from `https://fuckyouflow.app/download/...`. The domain
still carries the word, so this improves the odds without settling the question.

**Option C, register a neutral domain and a mirror repo** for the package-manager channel only, for
example `fuflow.app`. This is the only route that clears both the automated check and the human one.
Cost is roughly 12 euro a year plus half a day. Worth it only if winget matters to you, and winget is
the largest free Windows distribution channel that exists.

## Where each name goes

| Target | Name to use | Why |
|---|---|---|
| Hacker News Show HN | Full name | Precedent exists at 353 points, and the name is the hook |
| Reddit tech subreddits | Full name | No site-wide profanity rule; the audience enjoys it |
| GitHub, awesome-lists, LibHunt | Full name | `thefuck` sits in these lists with 97,000 stars |
| SourceForge | Full name | Three profane project names are live today |
| AlternativeTo | Full name works, "The Fuck" is listed | Community-editable, precedent confirmed |
| Softpedia, MajorGeeks, Uptodown, FossHub | "FU Flow (also known as Fuck You Flow)" | Human editors, no written rule, discretion applies |
| Product Hunt | "FU Flow" | Community guidelines cover offensive content, launch is once only |
| winget, Chocolatey | "FU Flow" plus one of options A, B, C | Written policy plus mandatory human review |
| Softonic | "FU Flow" | Written, quotable profanity ban |
| Microsoft Store | Skip entirely | Same policy family plus a paid account |
| Accessibility, charity, university, government catalogues | "FU Flow" only | No written rule found, but these are conservative institutions |
| Greek FOSS sites and Greek groups | "FU Flow" | You appear there as Luram AI Agency as well |

**The rule of thumb:** the full name where a crowd decides, "FU Flow" where a policy or an editor
decides.

---

# THE CODE-SIGNING PROBLEM

There is no code-signing certificate, so Windows SmartScreen shows "Windows protected your PC" on
first run for every user.

**Which places require signing:** none of the directories researched publish a hard code-signing
requirement. No signing requirement was found at winget, Chocolatey, Scoop, Softpedia, MajorGeeks,
Uptodown, FossHub, SourceForge or any awesome-list. The Microsoft Store requires it, and the Store is
already out for other reasons.

**Where the absence of a certificate actually bites:**

- **winget, the real risk.** The validation pipeline has a documented failure mode called
  `URL-Validation-Error`, triggered when a "URL has a poor SmartScreen reputation". A brand-new,
  unsigned, 1.4 GB executable with 13 downloads fits that profile. And `doc/Policies.md` states:
  "If a package is flagged by any of the security scans in the validation pipeline, it cannot be
  accepted into the repository, regardless of the application's legitimacy or intent."
- **Uptodown** publishes the sharpest antivirus rule of the group: it rejects apps "with malware
  detected by Virustotal that we consider harmful to the user". A large unsigned installer bundling
  model weights is a realistic false-positive candidate.
- **FossHub** states: "We DO NOT accept software titles that use installers/bundles or any other
  third-party offers that can be identified as adware or malware", and adds "we reserve the right not
  to list your software". The portable ZIP helps here.
- **MajorGeeks** hand-tests every submission in a virtual machine before rating it, so a SmartScreen
  block is seen by a person.
- **Every user, everywhere.** Put a short calm paragraph on the download page and in the README
  explaining the warning and the two clicks past it, and publish the SHA256 beside it.

**Two free actions worth taking now.** Upload the installer to VirusTotal so a clean multi-engine
result exists to point at, and submit it to Microsoft at
https://www.microsoft.com/en-us/wdsi/filesubmission as a suspected false positive.

**The cheap paid fix.** Azure Trusted Signing, renamed Azure Artifact Signing in 2026, is 9.99 US
dollars a month on the Basic tier for up to 5,000 signatures. Individuals can sign up in the USA and
Canada; in the EU and UK it is open to organisations, so Luram AI Agency would apply as one. Stated
plainly: signing grants no instant SmartScreen trust. Reputation builds across consecutive releases
signed with the same identity.

---

# TIER 1: DO THESE FIRST

## 1. winget, the Windows Package Manager community repository

- **Submission URL:** https://github.com/microsoft/winget-pkgs, a pull request adding
  `manifests/l/LuramAIAgency/FUFlow/0.9.0/`
- **Type:** package manager, first party to Windows 10 and 11
- **Value: HIGH.** winget ships inside Windows. Largest free Windows distribution channel there is,
  and the listing lasts forever with no upkeep beyond version bumps.
- **Account:** no new account. The existing GitHub account `Breakzoras` is enough. You must sign the
  Microsoft Contributor Licence Agreement, a click-through linked from the pull request; the label
  `Needs-CLA` sits on the pull request until you do.
- **Cost:** free.
- **What it asks for:** three YAML files, metadata only. No screenshots, no icon, no video. Field
  limits read from the 1.12.0 schema:

| Field | Required | Limit |
|---|---|---|
| PackageIdentifier | yes | max 128 chars, must match the folder path exactly, case sensitive |
| PackageVersion | yes | max 128 chars |
| Publisher | yes | 2 to 256 chars |
| PackageName | yes | 2 to 256 chars |
| License | yes | 3 to 512 chars |
| ShortDescription | yes | 3 to 256 chars |
| Description | no | max 10,000 chars |
| Tags | no | maximum 16 |
| Copyright, ReleaseNotes | no | 512 and 10,000 chars |

- **Ready to paste:**

```yaml
PackageIdentifier: LuramAIAgency.FUFlow
PackageVersion: 0.9.0
Publisher: Luram AI Agency
PublisherUrl: https://luram.gr
PublisherSupportUrl: https://github.com/Breakzoras/fuck-you-flow/issues
Author: Luram AI Agency
PackageName: FU Flow
PackageUrl: https://fuckyouflow.app
License: MIT
LicenseUrl: https://github.com/Breakzoras/fuck-you-flow/blob/main/LICENSE
ShortDescription: Offline dictation for Windows. Press right Alt, speak, press it again, and the text lands at the cursor in any program. Whisper large-v3 runs on your own GPU. English and Greek.
Moniker: fuflow
Tags:
  - dictation
  - speech-to-text
  - voice-typing
  - whisper
  - offline
  - privacy
  - greek
  - transcription
  - vulkan
  - gpu
ManifestType: defaultLocale
ManifestVersion: 1.12.0
```

  The installer manifest values were confirmed from `src-tauri/tauri.conf.json`:
  `InstallerType: nullsoft`, `Scope: user` (the config sets `installMode: currentUser`),
  `Silent: /S`, `Architecture: x64`. The `LicenseUrl` above uses `main`, which is confirmed to be
  the repository default branch.

- **How to build it:** `wingetcreate new <installer-url>` detects the installer type, computes the
  SHA256 and writes all three files. Then run, in this order, `winget validate --manifest <folder>`,
  `winget install --manifest <folder>`, and `Tools/SandboxTest.ps1`, before opening the pull request.
- **What rejects or delays it:**
  - `Policy-Test-2.9`, the profanity check. Main risk, see the name section.
  - `URL-Validation-Error` from SmartScreen reputation on an unsigned new binary.
  - Any hit from the static or dynamic malware scan is final.
  - The installer must install with no user interaction. Scripts as installers are banned outright.
  - One package and one version per pull request, with no other files touched.
  - `Needs-Author-Feedback` auto-closes a quiet pull request after 5 plus 3 days.
- **Size is settled.** `Nvidia.CUDA` is in winget today with a 2.53 GB installer, measured directly.
  The 100 MB figure that circulates online comes from an issue closed in 2020.

## 2. AlternativeTo, including the Wispr Flow page

- **Submission URL:** https://alternativeto.net, account menu, "Suggest new application". Separately,
  open https://alternativeto.net/software/wispr-flow/ and use the in-page control to add FU Flow as
  an alternative.
- **Type:** software directory, community edited
- **Value: HIGH.** This is the single best intent match anywhere: people searching "Wispr Flow
  alternative" land on that exact page, and your own comparison page is built for the same query.
  Roughly 2.9 million monthly US visits by third-party estimate.
- **Account: YES, a new account, with a verified email.** Their FAQ states: "You need to verify your
  email address before you can submit a new app."
- **Cost:** free. An optional one-time 5 US dollars skips the review queue.
- **What it asks for:** platforms (tick Windows), licence (Open Source, MIT), short and long
  description, tags, source code link, screenshots.
- **Rejection risk:** they acknowledge a rising rejection rate. The name is safe here: "The Fuck" is
  a live catalogue entry.
- **Do the second half too.** Being listed on the Wispr Flow alternatives page is worth more than the
  standalone entry. Also add yourself under Dictation.io and Nerd Dictation.

## 3. Hacker News, Show HN

- **Submission URL:** https://news.ycombinator.com/submit, rules at
  https://news.ycombinator.com/showhn.html
- **Type:** launch platform
- **Value: HIGH.** One good Show HN outranks twenty directory listings. The audience is exactly
  developers on Windows who dislike cloud dictation, and the name is an asset there.
- **Account: YES, a new free account.** No email is required to register.
- **Cost:** free.
- **Eligibility, quoted:** Show HN is for "things people can run on their computers or hold in their
  hands", must be "something you've worked on personally and which you're around to discuss", it
  "needn't be complicated or look slick", and early-stage work is fine. It should be "easy for users
  to try your thing out, ideally without barriers such as signups or emails". A downloadable beta
  satisfies this.
- **What kills it:** "quickly-generated one-offs", a bare landing page, and minor version bumps.
- **Practical:** post the GitHub repo or the site, title beginning "Show HN:", then stay at the
  keyboard for several hours answering every comment. Expect hard questions about the 1.4 GB
  download, the missing signature, and why the models ship inside the installer. Have honest answers
  ready.

## 4. r/software

- **Submission URL:** https://www.reddit.com/r/software/submit
- **Type:** community
- **Value: HIGH.** Its self-promotion rule was read today and it exempts exactly this app.
- **Account: YES, a Reddit account**, reusable for every subreddit below.
- **Cost:** free.
- **The rule, quoted from the live rules:** "Don't promote your own software unless its open source.
  Permanent ban. The one exception is if you share on Wednesdays, use the appropriate flair and
  provide something of value to the community rather than just the software."
- **The Releases rule, quoted:** "Releases should be for new programs or big updates... The program
  should be open-source. This is not necessary, but is highly suggested. It should also not be behind
  a paywall, you may have donation links, but the program should be fully usable without payments."
- **The gate to plan around, quoted:** "We require users to have a certain amount of karma before
  making submissions. Our AutoModerator will remove submissions from users who fall below the
  threshold. The threshold is a secret... if you really need to post something, do it, and send us
  Modmail. We'll happily manually review requests to post."
- **So:** a fresh account will be auto-removed. Either build karma for a week by commenting, or post
  and immediately send modmail. Use the Releases flair. Fix the LICENSE file first, because the
  open-source exemption is what makes the post legal here.

## 5. r/LocalLLaMA

- **Submission URL:** https://www.reddit.com/r/LocalLLaMA/submit
- **Type:** community, roughly 700,000 to 800,000 members by third-party estimate
- **Value: HIGH.** Perfect audience: people who already run models on their own GPU and who care
  about nothing being uploaded.
- **Account:** the same Reddit account.
- **Cost:** free.
- **The rules, quoted live:** "The 1/10th rule is a good guideline: self-promotion should not be more
  than 10% of your content" and "Affiliation must be disclosed: No engagement farming, No 'I found
  this..', etc."
- **The genuine risk:** rule 2 reads "Posts must be related to Llama or the topic of LLMs." Whisper
  is speech recognition, so an off-topic removal is possible. Lead with the local-inference angle,
  the Vulkan backend that works on AMD and Intel, and the measured latency figures. Disclose that you
  built it, in the first line.
- **Also banned there:** "Completely/primarily LLM generated copy, code is not allowed." Write the
  post yourself.

## 6. Scoop, your own bucket

- **Submission URL:** none. Create `https://github.com/Breakzoras/scoop-bucket` from
  https://github.com/ScoopInstaller/BucketTemplate
- **Type:** package manager, self-published channel
- **Value: MEDIUM.** Small audience, live in ten minutes, cannot be rejected by anyone, and gives the
  site a copy-paste install line for developer readers.
- **Account:** no new account, existing GitHub only.
- **Cost:** free.
- **What it asks for:** one JSON file with version, description, homepage, license, url, hash,
  installer arguments, shortcuts, and optionally checkver and autoupdate for automatic version bumps.
- **Users then run:** `scoop bucket add fuflow https://github.com/Breakzoras/scoop-bucket` followed by
  `scoop install fu-flow`
- **Rejection risk:** none, it is your repository.

## 7. whisper.cpp Show and tell

- **Submission URL:**
  https://github.com/ggml-org/whisper.cpp/discussions/categories/show-and-tell
- **Type:** upstream project community, roughly 53,000 stars
- **Value: MEDIUM to HIGH.** These are the people already convinced that local Whisper is the right
  approach. Free, instant, no gatekeeper.
- **Account:** existing GitHub.
- **Cost:** free.
- **Worth knowing:** the README "Bindings" table is for language bindings only, so an end-user app
  belongs in Show and tell, which is a separate place from the README.
- **Bonus:** thank the maintainers and say which build flags and which Vulkan path you use. That is
  what earns replies there.

## 8. r/opensource

- **Submission URL:** https://www.reddit.com/r/opensource/submit
- **Account:** the same Reddit account. **Cost:** free.
- **Value: MEDIUM to HIGH.** Roughly 210,000 members, and open source is the entry ticket.
- **The blocking rule, quoted live:** "Code or repositories linked to MUST have a LICENSE file that
  MUST be an OSI listed Open Source license." Today GitHub reports the licence as "Other", so fix
  item 1 in the prep list before posting here.
- **Also quoted:** "Reddit recommends that <10% of your posts promote your content. We're a little
  more forgiving", and "All AI-generated content is low-effort and ban worthy."
- **Mechanics:** use the `Promotional` flair, which their flair rule reserves for "when you are
  sharing a project, yours or otherwise". Do not drive-by post; their karma-farm rule removes posts
  from people who never return to the thread.

## 9. Chocolatey community repository

- **Submission URL:** push with `choco push` to https://community.chocolatey.org/packages/upload
- **Type:** package manager, the established third-party one for Windows
- **Value: MEDIUM to HIGH.** Large installed base in workplaces and among IT people. Slower and more
  bureaucratic than winget.
- **Account: YES, a chocolatey.org account.** Its API key is what authorises the push. This is the
  only package manager here needing a new account.
- **Cost:** free.
- **What it asks for:** a `.nuspec` plus `tools/chocolateyInstall.ps1`, with these requirements taken
  from the moderation documentation and the validator rule list:
  - package id lowercase, dashes allowed, no dots; they "suggest the id split if over 25 chars with
    no '-' in the id". Use `fu-flow`.
  - title "should be the same as the name of the application", which is the name question again
  - `projectUrl` is required for the community feed
  - description between 30 and 4,000 characters, rules CPMR0032 and CPMR0026
  - `licenseUrl` required where a licence exists; `iconUrl` strongly encouraged
  - iconUrl rules are specific: raw GitHub links are banned, a CDN such as jsDelivr or Statically is
    required, PNG preferred. Since you are both author and maintainer you may host it yourself at
    `https://fuckyouflow.app/assets/icon-256.png`
  - tags must carry no commas and must not include "chocolatey"
  - copyright, authors and licence attribution are checked closely by moderators
- **The 1.4 GB question:** the community feed caps an uploaded package at **200 MB**, so the installer
  cannot travel inside it. That is the normal pattern anyway: the package downloads from your GitHub
  release URL and verifies the checksum with
  `Install-ChocolateyPackage -Url64bit ... -Checksum ... -ChecksumType sha256 -SilentArgs '/S'`.
- **What rejects or delays it:** human moderation of every version; an id that breaks the naming
  convention is "grounds for rejecting immediately"; an automated verifier installs and uninstalls
  the package on a clean machine, so a failing silent install fails the package; downloading from
  FossHub or SourceForge is flagged by rules CPMR0028 and CPMR0034, so keep the GitHub release URL.

## 10. Product Hunt

- **Submission URL:** https://www.producthunt.com/launch
- **Type:** launch platform
- **Value: MEDIUM to HIGH.** One day of attention plus a permanent page that ranks. Its audience skews
  towards web products, so a Windows download will do less well than a SaaS, and the page keeps
  earning afterwards.
- **Account: YES, and it must be a personal profile.** Their help centre states: "You'll need a
  personal account to post... Company accounts cannot hunt or post products", and profiles must
  "appear human and represent an individual, not a company, service, brand, or organization". So it
  launches under your own name, with Luram AI Agency mentioned in the description.
- **Cost:** free.
- **What it asks for:** thumbnail 240x240 (GIF allowed, under 3 MB); at least two gallery images at
  1270x760, three to five recommended, or a 30 to 90 second video; tagline maximum 60 characters;
  description limit is stated inconsistently in their own documentation, 260 characters in the help
  centre and 500 in the newer guide, so write to 260; video is optional and YouTube only.
- **Beta is fine:** the form has a field to mark a product as still in beta.
- **Name:** use "FU Flow". Their community guidelines cover offensive language, and you get one
  launch.

## 11 to 13. The rest of the first wave

**r/SideProject** (https://www.reddit.com/r/SideProject/submit). Same Reddit account, free. Its rules
list is empty apart from Reddit's site rules, read live today, so the culture is the only constraint:
show the working product. Estimates of its size range from 180,000 to 790,000 members.

**r/coolgithubprojects** (https://www.reddit.com/r/coolgithubprojects/submit). Same account, free.
Its rules list is also empty apart from Reddit's site rules. Roughly 75,000 members, GitHub-hosted
projects only, and the sidebar convention is a title of the form `[Language] name, what it does`.

**Scoop Extras bucket** (https://github.com/ScoopInstaller/Extras). Existing GitHub, free. The
process is fixed and it starts with an issue. Quoted from the Scoop contributing guide: "If you want
to work on something that there is no GitHub issue for, including for submitting a new package",
you must "Create a new GitHub issue... and propose your change there", then "Wait for a project
maintainer to evaluate your issue", because "We are very reluctant to accept random pull requests
without a related issue created first." The Scoop main bucket is out of reach and should be skipped:
its criteria demand "at least 500 stars and 150 forks" and "a non-GUI tool", and this app has 0 stars
and a graphical interface.

---

# TIER 2: WORTH DOING, LOWER RETURN

## Open-source and privacy indexes

**pluja/awesome-privacy** (https://github.com/pluja/awesome-privacy, roughly 19,700 stars). Pull
request, existing GitHub, free. **No minimum star count**, which makes it the most reachable good
list today. Its contributing rules require a clear privacy policy and no user tracking on the project
website. It already has an "Artificial Intelligence, Speech to Text" section listing OpenWhispr as an
open-source alternative to Wispr Flow, so the shelf is built and empty next to it.

**0PandaDEV/awesome-windows** (https://github.com/0PandaDEV/awesome-windows, roughly 2,800 stars,
pushed within the last week). Pull request, existing GitHub, free. No star minimum. Title case,
alphabetical order. Its contributing file opens: "Vibecoded slop and tools that don't fall in the
category of 'awesome' are not welcomed on this list and PR's will be rejected." Note that the older
`Awesome-Windows/Awesome` repo no longer exists.

**sindresorhus/awesome-whisper** (https://github.com/sindresorhus/awesome-whisper, roughly 2,400
stars). Pull request, existing GitHub, free. Perfect topical fit and it already lists Windows
dictation apps. **Blocked today by one line in its contributing file:** "If the submitted project is
an open-source project on GitHub: It should have at least 100 stars." Put this on the calendar for
when the repo passes 100 stars, which a good Show HN can deliver in a day.

**OpenAI Whisper Show and tell**
(https://github.com/openai/whisper/discussions/categories/show-and-tell). Existing GitHub, free,
no gatekeeper, right audience.

**FSF Free Software Directory** (https://directory.fsf.org). Account required, free. MIT qualifies.
Windows-only software is welcome; they maintain a Windows collection specifically to help Windows
users find free programs. Multi-part wiki form. Slow, small traffic, good for credibility and a
durable link.

**Privacy Guides** (https://discuss.privacyguides.net). Forum account required, free. As the
developer you must use the **Project Showcase** self-submission category, which is separate from
community tool suggestions. Their criteria require you to disclose affiliation, explain what the
project brings, and state the exact threat model, and they warn that "unmaintained projects will be
removed in most cases". Honest assessment: a project one day old with no track record will most
likely be told to come back later. Revisit in six months.

**PRISM Break** (https://gitlab.com/prism-break/prism-break, last activity 30 May 2026). Pull request
adding an entry to `source/db/en-projects.json` with name, description, logo, url, categories and
development stage. No new account if you use GitLab through GitHub sign-in or the GitHub mirror.
Free. Small traffic, quick win.

**LibHunt** (https://www.libhunt.com). Free, no account needed for the "Add a project" form. Mostly
self-populating: it monitors Reddit, Hacker News and Dev.to and records repository mentions, so the
Show HN and Reddit posts feed it automatically. Submit and forget.

**awesome-talon** (https://github.com/trillium/awesome-talon). Existing GitHub, free. Small, and the
audience is precisely the RSI and hands-free crowd who need this app most.

**openalternative.co** (https://openalternative.co/submit). No separate account. **Paid only**, with
three tiers: Standard 97 US dollars, Premium 137, Ultimate 197 a month. Approved entries also
populate the linked GitHub list, which has roughly 6,700 stars. Skip unless you decide to buy one
listing, in which case Premium is the only tier with a do-follow link.

**opensourcealternative.to** (submission form on the site). Free waitlist stated at six months or
more, or 29 US dollars for a 48 hour review. Their form text mentions the project being self-hosted,
which fits poorly for a desktop app and should be checked before paying.

## Software directories and download hosts

**SourceForge** (https://sourceforge.net, create a project). Account required, and a first project
also needs SMS phone verification. Free. Two separate wins: a directory listing with roughly 23.7
million monthly visits by third-party estimate, and a legitimate mirror for the 1.4 GB installer,
since their per-file cap is 2 GB. No content objection to the name, proven by three live profane
project names. Note that Chocolatey rule CPMR0034 flags packages that download from SourceForge, so
keep GitHub as the canonical URL.

**Softpedia** (https://www.softpedia.com/user/submit.shtml, also accepts a PAD file). No account
requirement surfaced, so treat it as no account until the form says otherwise. Free. Roughly 3.55
million monthly visits. They run their own malware scan for the "100% CLEAN" badge, which is worth
having as a counterweight to the SmartScreen warning. No written naming policy found; the only
precedent found is mild ("DAMN NFO Viewer"), so submit as "FU Flow".

**Uptodown** (https://en.uptodown.com/developers-console, "Add new app"). Account required, free.
The largest audience in this group by a wide margin, roughly 130 to 143 million monthly visits by
third-party estimate, and multilingual, which suits the Greek support. Wants a promotional banner at
exactly 1024x500 and descriptions in several languages. Their stated rejection trigger is the
antivirus one quoted in the signing section, so run VirusTotal first.

**MajorGeeks** (email tim@majorgeeks.com or jim@majorgeeks.com; no public form exists). No account,
free. Roughly 3.25 million monthly visits and a trusted enthusiast audience. Every submission is
hand-tested in a virtual machine before it gets a star rating, so the unsigned installer meets a
human. Send a short email with the download link, the SHA256, the VirusTotal link and two
screenshots.

**FossHub** (https://www.fosshub.com/signup.html for developers). Account required, free. Small,
high-trust audience for open-source Windows software. Their exclusion is quoted in the signing
section above; the portable ZIP is the answer to it.

**Softonic** (https://publishing-center.softonic.com/home). Account required, free listing, optional
paid campaigns. Large claimed audience, self-reported. This is the one place with a written
profanity ban, so it is "FU Flow" or nothing.

## Communities and launch platforms

**Slant** (https://www.slant.co). Account likely required, free. Add FU Flow as an option under the
existing dictation and speech-to-text questions, with a written argument. Slant pages rank well for
"best X" queries and the format rewards a specific, honest pitch.

**DevHunt** (https://devhunt.org, listings via pull request to
https://github.com/MarsX-dev/devhunt). Existing GitHub, free. Weekly developer-tool launch board, and
the submission path is native to an open-source project.

**Indie Hackers** (https://www.indiehackers.com/products, "Add Your Product"). New account, free.
Modest traffic, decent for a build-in-public narrative.

**SaaSHub** (https://www.saashub.com/services/submit). New account, free, product needs verification.
Alternative-focused, so it pairs with the Wispr Flow angle, though the site leans towards SaaS.

**Uneed** (https://www.uneed.best/submit), **Peerlist Launchpad** (https://peerlist.io/launchpad,
personal profile only, explicitly accepts beta products), **MicroLaunch** (free tier plus a boost
from 39 US dollars; requires a 60 character tagline, a 250 character description, a 512 pixel or
larger logo, and two to three screenshots), **Fazier** (https://fazier.com/submit), **Launching
Next** (https://www.launchingnext.com/submit/, slow free queue). All new accounts, all low value
individually. Batch them into one hour on a slow afternoon using the prepared asset pack.

## AI tool directories, and the truth about them

Most of this category is pay-to-play with unverifiable traffic. Ranked honestly:

- **There's An AI For That** (https://theresanaiforthat.com/submit/). Account required. 49 US dollars
  for a listing, 347 for newsletter and featured placement. Their own FAQ describes the only free
  route as a monthly thread where one indie tool gets picked, which is a lottery. The single paid AI
  directory with enough traffic to argue for.
- **OpenTools** (https://opentools.ai/tool-submit) and **TopAI.tools** (https://topai.tools/submit,
  free tier with no turnaround guarantee, 47 US dollars for 48 hours). Free routes exist. Low value,
  low cost, no harm.
- **opensourcesai.com/tools** and **best-ai.org/tools/open-source**. Free, small, and filtered for
  local and open-source tools, which is the right shelf.
- **Futurepedia** (247 US dollars, currently sold out, or 497 "Verified"), **Toolify.ai** (99 US
  dollars flat), **aitools.fyi** (its submit button redirects to a 47 US dollar third-party upsell).
  **Skip all three.** Bad value for a free tool with no revenue.
- The wider cluster of near-identical "submit your AI tool" pages charging 5 to 160 US dollars is the
  same pattern repeated. Ignore it.

**Nothing exists that is a genuine speech-to-text-only directory.** The "best dictation software"
roundups that dominate those searches are content marketing for competing commercial products, and
they have no reason to list a free rival.

## Accessibility, assistive technology, and Greek reach

Low volume, high goodwill, and the name needs care here.

- **AbleData is dead**, discontinued in 2020. Skip it.
- **AbilityNet** (https://abilitynet.org.uk) has no self-serve submission. It takes a direct pitch to
  their editorial contact. Worth one email framed around hands-free computing for people with RSI or
  limited hand mobility, under the name "FU Flow".
- **EASTIN, AT3 Center and similar registries** are procurement databases for physical assistive
  equipment fed by national agencies. Poor fit.
- **University accessibility pages** often invite tool suggestions by email. Free, slow, one at a
  time. A handful of emails at most.
- **The Talon Voice community** (https://talon.wiki and their Slack) is exactly the RSI audience,
  though it is a support community for Talon and not a directory. Participate as a person, and expect
  no listing.
- **No profanity policy was found at any of these**, and none of them publishes one. The caution is a
  judgement call about conservative institutions.
- **Greek FOSS media, free, by email:** dwrean.net explicitly invites suggestions, ellak.gr is the
  established Greek open-source association, and opensoft.gr keeps a FOSS catalogue with a contact
  page. These are the cheapest test of how the Greek audience reacts to the name, and dwrean.net,
  which asks for suggestions outright, is the natural first try.

---

## A four-week order of operations

**Week 1, the free foundations, no new accounts beyond Reddit and Hacker News.**
Fix the five preparation items. Publish the portable ZIP. Create the Scoop bucket. Post to the
whisper.cpp Show and tell. Upload to VirusTotal and submit the false-positive report to Microsoft.

**Week 2, the attention spike.** Show HN on a Tuesday or Wednesday morning US time, then the Reddit
posts spaced over several days, r/software on a Wednesday because their rule says so, then
r/LocalLLaMA, r/opensource, r/SideProject, r/coolgithubprojects. Answer every comment. This week is
also what earns the stars that unlock awesome-whisper.

**Week 3, the permanent listings.** AlternativeTo including the Wispr Flow page, winget after the
name decision is made, Chocolatey, the Scoop Extras issue, awesome-privacy, awesome-windows,
SourceForge, Softpedia, Uptodown, MajorGeeks.

**Week 4, the long tail.** Product Hunt with the full asset pack and the video, Slant, DevHunt,
SaaSHub, Indie Hackers, LibHunt, PRISM Break, the FSF directory, the small launch boards in one
batch, then the Greek and accessibility emails.

**One decision blocks Week 3:** which of options A, B or C you take on the name for the package
managers. Everything else can proceed without it.
