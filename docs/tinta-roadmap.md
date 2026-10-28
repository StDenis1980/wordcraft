# Tinta Texto — Tinta Office roadmap

> **Status:** 2026-10-27 · approved by the owner (Denys) · **Revision history:** 2026-10-27 created;
> 2026-10-27 dictionary pack and uk/ca/gl/eu spelling (priority 1) done; 2026-10-28 analysis of a
> shared dictionary built from users' added words (priority 9); 2026-10-28 .docx test set
> (priority 2) done; 2026-10-28 charts, SmartArt and OLE kept on save confirmed (priority 3); 2026-10-28 priority 9: automatic validation of users' words; owner chose button first, then
> automatic sending; the option is off by default with a GDPR notice beside it.

Tinta Texto is the Tinta Office build of WordCraft and replaces Microsoft Word. Upstream keeps the
feature-parity roadmap ([ROADMAP.md](../ROADMAP.md), [roadmap.md](roadmap.md),
[gaps.md](gaps.md)); this page lists only what the fork does and which upstream gaps matter most
for users in Spain and Ukraine. It is a fork-only file, so upstream syncs never conflict on it.

## Where it stands (2026-10-27)

Ready for real work ≈ 60% (essentials user ≈ 63%, upstream estimate); 157 commits behind
upstream. Styles, lists, tables, review and mail merge work; the main risk is other people's
.docx: our own test set of typical documents, a report with a chart, SmartArt and an embedded spreadsheet included, now passes open → save with nothing lost (charts, SmartArt and OLE objects are kept on save since upstream #319; upstream's ROADMAP still says they are dropped). Charts from Word can't be edited yet. Spelling in
Spanish, Catalan, Galician, Basque and Ukrainian works out of the box (dictionaries ship with the
app since Preview 3).

## Priorities

| # | Task | Where | Stage |
|---|---|---|---|
| 1 | ✅ Tinta dictionary pack: es_ES, ca, gl_ES, eu, uk_UA ship in every package ([packaging/dictionaries](../packaging/dictionaries/README.md): pinned sources, SHA-256, licences); spelling in Ukrainian, Catalan, Galician and Basque (crate `proof`, module `plain`; the affix engine reads long and numeric flags, up to 512 flags, IGNORE, ICONV, NEEDAFFIX, KEEPCASE). Offer the code upstream. Next: the Valencian `ca-valencia` dictionary, hyphenation patterns for these languages | fork (pack) + upstream (code) | 1 |
| 2 | ✅ Our .docx test set ([`crates/docx/tests/it/tinta_documents.rs`](../crates/docx/tests/it/tinta_documents.rs)): a Spanish invoice with a header logo, IVA and IRPF, a service contract with numbered clauses and a footnote, a Ukrainian CV in a layout table, a Catalan letter with a date field, written from the spec in the shape Word saves them. "Open → save → open" keeps the model, every element, attribute and paragraph of text, every relationship and document property; a second save is byte-identical. It found one loss, now fixed: the language, status, identifier, version and last-printed date of the document properties. Next: more documents as users report problems (offer the fix and the tests upstream) | fork (tests) → fixes upstream | 1 |
| 3 | ✅ Keep charts, SmartArt and OLE through open and save: already done upstream (#319); our report document (`informe` in the test set: a bar chart with its workbook, a SmartArt process, an embedded spreadsheet) proves it. It found one bug, fixed: an OLE object's picture got a new name on every save (`image2_2_2….png`). Editing Word's charts is upstream gap 13 (stage 3) | upstream; tests and fix in the fork, to offer upstream | 1 |
| 4 | Templates for Spain: factura, presupuesto, carta formal, currículum — our own text and colours, no Microsoft assets | fork or upstream | 2 |
| 5 | Pagination like Word: column balancing, footnote continuation, Aptos/Calibri substitutes | upstream | 2 |
| 6 | Formatting revisions and content controls | upstream | 2 |
| 7 | .odt export for Spanish administrations that use ODF: check the current state | our check | 2 |
| 8 | Charts, SmartArt, ink (≈ 3% today) | upstream | 3 |
| 9 | A shared Tinta dictionary built from the words users add (see [below](#analysis-a-shared-dictionary-from-users-words)): the "Suggest these words" button and automatic validation, then automatic sending | fork | 2 (button), 3 (automatic) |

## Analysis: a shared dictionary from users' words

Owner's proposal (2026-10-28): people add words the dictionaries lack (Add to Dictionary); collect
them, check that they really exist, and add the right ones to our dictionaries, so every user gets
them with the next update. To analyse in stage 2, before any code:

- **Privacy first.** Added words are often names, surnames, company names, street names, DNI/NIE
  or account numbers: personal data under the GDPR. Owner's decision (2026-10-28): the
  "Suggest my words" option lives in File › Options › Proofing and is **off by default**; next to
  it, a short GDPR notice says what is sent (only the added words and their language, never
  document text or a user id), why (to improve Tinta's dictionaries), where it goes, how long it
  is kept and how to withdraw (turn the option off; ask hola@tintaoffice.com to delete). Turning
  it on is the consent, so no separate pop-up warning. The app still skips anything with digits,
  `@` or a capital inside the word, and the privacy policy on tintaoffice.com repeats the notice.
- **What we have today.** The custom dictionary is one list for every language (File › Options ›
  Proofing, `ui.json`, at most 10,000 words) and does not record the language a word was added
  in. Collecting needs the language with each word.
- **Collection.** Simplest first step: a "Suggest these words for Tinta's dictionaries" button
  that prepares the list for the user to review and send (email to hola@tintaoffice.com or a
  form on the site). An automatic channel needs a small server (for example a Cloudflare Worker
  next to the site) and counts each word once per install.
- **Validation: automatic, no person in the loop** (owner's decision, 2026-10-28). The server,
  never the app, checks each candidate, so no user's words reach a third party. A word is
  added when all of these hold:
  1. at least 5 different installs added it in that language (more for Spanish);
  2. it passes the filters: letters of the language only, 2–40 characters, all lower case
     (names and brands stay out);
  3. a public dictionary source has an entry for it in that language, used within its terms
     and licence. Candidates: Wiktionary in every language (dumps or API, CC BY-SA); for
     Ukrainian, VESUM, the open source of dict_uk; for Catalan, Softcatalà's and LanguageTool's
     public data; a LanguageTool server we run ourselves (LGPL) as an external checker for
     all five languages. Not RAE's DLE: its terms forbid automated queries;
  4. it is not a one-letter change of a word the dictionary already knows, unless the source
     in step 3 lists it (catches common typos such as `recivir`).

  Words that pass 1–2 but not 3 wait in a queue and are re-checked monthly. Each added word
  carries where it was found; a word reported as wrong ("this is a typo") is removed from the
  next release. The user's own custom dictionary is never touched.
- **Where the words go.** Best: as contributions to the dictionary projects themselves (RLA-ES,
  Softcatalà, Proxecto Trasno, dict_uk, Xuxen), so every app gains them and our pack stays
  unchanged upstream text. Faster: our own small supplementary word list per language, shipped
  in the pack under a permissive licence. That needs the proofing engine to read a second word
  list next to each Hunspell dictionary (not supported today).
- **Updates.** Words arrive with each monthly release; a separate dictionary update without
  reinstalling comes later.
- **Owner decisions (2026-10-28).** Collect the words; validate them automatically without a
  person; send them first by hand (stage 2: the "Suggest these words" button, the user reviews
  the list and sends it), then automatically after a one-time consent (stage 3: the server).

## Suite-wide work (all five Tinta Office apps)

Stages: **1** = November 2026 (Preview 3), **2** = December 2026 – January 2027 (public beta of
the most ready apps), **3** = 2027 (1.0). Dates are indicative.

| Work | Stage | What |
|---|---|---|
| Upstream sync | ongoing | Weekly, Mondays. Catch up before Preview 3. |
| Our changes upstream | 1 | Offer storytold the configurable brand, Presenta's translations, the codec feature flag and the ca/gl/eu catalogs (craft-i18n#53), so the fork stays small. |
| GitHub Actions | 1 | GitHub disabled Actions in pdfcraft, wordcraft and deckcraft; ask GitHub Support. Until then, run the gates locally. |
| Releases | 1 | A preview every month for all five apps, release notes in es/uk/en, one build script. |
| Linux | 1 | AppImage + .deb with Tinta names for all five (built in the cloud); Flatpak in stage 2. |
| Windows installers | 1 | Desktop shortcuts for Texto and Calculo; Presenta registered for .pptx. |
| Code signing | 2 | Windows (Certum Open Source or Azure Trusted Signing; scripts exist), macOS (Apple Developer + notarization). Owner decision and payment. |
| Automated tests | 1–2 | Coverage audit per app; tests for every fork change (brand, languages, installers); a smoke test of each release installer on Windows; investigate the flaky tests upstream. |
| User guide | 2 | A short "first steps" guide per app in Spanish and Ukrainian on tintaoffice.com (Correo has one). |
| Fork documentation | 1 | This file: what the fork changes, how to build a release, how to sync. |
| Translations | 1–2 | Ukrainian reviewed by the owner, Spanish by a native tester, ca/gl/eu through communities. Marked as drafts until reviewed. |
| Feedback | 1 | Help › Report a problem → the fork's issues or hola@tintaoffice.com; issue templates in Spanish and Ukrainian. |

Rules: upstream first (fixes useful to everyone go to storytold as PRs); every change ships with
automated tests and updated docs in the same PR; files people exchange (.docx, .xlsx, .pptx, PDF)
come first; Spain and Ukraine first (UI language, spelling, number/date formats, national
certificates).
