# Tinta Texto — Tinta Office roadmap

> **Status:** 2026-10-30 · approved by the owner (Denys) · **Revision history:** 2026-10-27 created;
> 2026-10-27 dictionary pack and uk/ca/gl/eu spelling (priority 1) done; 2026-10-28 analysis of a
> shared dictionary built from users' added words (priority 9); 2026-10-28 .docx test set
> (priority 2) done; 2026-10-28 charts, SmartArt and OLE kept on save confirmed (priority 3); 2026-10-28 priority 9: automatic validation of users' words; owner chose button first, then
> automatic sending; the option is off by default with a GDPR notice beside it; 2026-10-28 templates
> for Spain (priority 4) done; .odt export checked (priority 7); 2026-10-28 synced with upstream;
> .odt export keeps headers, footers, page numbers, spacing, indents, tabs, borders (priority 7);
> 2026-10-28 Suggest my words option and list (priority 9, stage 2) done; priorities 5 and 6 checked against upstream;
> 2026-10-28 the .odt importer reads paragraph spacing, indents, line height, tab stops, borderless and aligned tables (priority 7); then paragraph borders and common styles' spacing;
> 2026-10-30 automatic sending and validation of suggested words (priority 9, stage 3); 2026-10-30 Valencian `ca-valencia` dictionary in the pack and Catalan (Valencian) proofing (priority 1); hyphenation patterns for es/ca/gl/eu/uk (priority 1); .odt cell margins read and written (priority 7).

Tinta Texto is the Tinta Office build of WordCraft and replaces Microsoft Word. Upstream keeps the
feature-parity roadmap ([ROADMAP.md](../ROADMAP.md), [roadmap.md](roadmap.md),
[gaps.md](gaps.md)); this page lists only what the fork does and which upstream gaps matter most
for users in Spain and Ukraine. It is a fork-only file, so upstream syncs never conflict on it.

## Where it stands (2026-10-27)

Ready for real work ≈ 60% (essentials user ≈ 63%, upstream estimate); in step with
upstream (synced 2026-10-28). Styles, lists, tables, review and mail merge work; the main risk is other people's
.docx: our own test set of typical documents, a report with a chart, SmartArt and an embedded spreadsheet included, now passes open → save with nothing lost (charts, SmartArt and OLE objects are kept on save since upstream #319; upstream's ROADMAP still says they are dropped). Charts from Word can't be edited yet. Spelling in
Spanish, Catalan, Galician, Basque and Ukrainian works out of the box (dictionaries ship with the
app since Preview 3).

## Priorities

| # | Task | Where | Stage |
|---|---|---|---|
| 1 | ✅ Tinta dictionary pack: es_ES, ca, ca-valencia, gl_ES, eu, uk_UA ship in every package ([packaging/dictionaries](../packaging/dictionaries/README.md): pinned sources, SHA-256, licences); spelling in Ukrainian, Catalan, Galician and Basque (crate `proof`, module `plain`; the affix engine reads long and numeric flags, up to 512 flags, IGNORE, ICONV, NEEDAFFIX, KEEPCASE). Valencian: text marked Catalan (Valencian), `ca-ES-valencia` (Review › Language), is checked with Softcatalà's `ca-valencia` dictionary, which accepts the Valencian forms and accents the central one marks (`este`, `vore`, `conéixer`, `anglés`, `café`), else with `ca`. Hyphenation: hyph-utf8's TeX patterns for es, ca (also Valencian), gl, eu and uk are bundled (`assets/hyphenation`, unmodified, MIT/LPPL/Unlicense/permissive; see ATTRIBUTION.md), so these languages break at their own syllables (`pre-su-pues-to`, `l'em-pre-sa`, `під-при-єм-ство`) instead of by the English rules. Offer the code upstream | fork (pack) + upstream (code) | 1 |
| 2 | ✅ Our .docx test set ([`crates/docx/tests/it/tinta_documents.rs`](../crates/docx/tests/it/tinta_documents.rs)): a Spanish invoice with a header logo, IVA and IRPF, a service contract with numbered clauses and a footnote, a Ukrainian CV in a layout table, a Catalan letter with a date field, written from the spec in the shape Word saves them. "Open → save → open" keeps the model, every element, attribute and paragraph of text, every relationship and document property; a second save is byte-identical. It found one loss, now fixed: the language, status, identifier, version and last-printed date of the document properties. Next: more documents as users report problems (offer the fix and the tests upstream) | fork (tests) → fixes upstream | 1 |
| 3 | ✅ Keep charts, SmartArt and OLE through open and save: already done upstream (#319); our report document (`informe` in the test set: a bar chart with its workbook, a SmartArt process, an embedded spreadsheet) proves it. It found one bug, fixed: an OLE object's picture got a new name on every save (`image2_2_2….png`). Editing Word's charts is upstream gap 13 (stage 3) | upstream; tests and fix in the fork, to offer upstream | 1 |
| 4 | ✅ Templates for Spain on File › New ([`crates/engine/src/tinta_templates.rs`](../crates/engine/src/tinta_templates.rs), `file.new` ids `es-invoice`, `es-quote`, `es-letter`, `es-cv`): factura with the fields of RD 1619/2012 (number, date, both NIFs, base, IVA, IRPF retention, IBAN), presupuesto with validity, conditions and acceptance, carta formal with Ref. and Asunto, currículum with the RGPD/LOPDGDD consent line. A4, Spanish language, page numbers in the footer, our own text and colours; the gallery labels are translated in all 29 interface languages. Tests: each fits one page, File › New opens it, and it survives .docx and .odt | fork | 2 |
| 5 | Pagination like Word. Checked 2026-10-28: done upstream for column balancing before a continuous section break (#350) and footnote separators and continuation notices (#352). Still open upstream (gap 4): no metric-matched substitute for Aptos (needs an owner decision on a font), legacy compatibility options only partly covered, no page-by-page comparison with Word | upstream | 2 |
| 6 | ✅ Formatting revisions and content controls: done upstream (formatting revisions #41, content controls #381, move tracking #400) and in the fork since the 2026-10-28 sync. Still open upstream (gap 5): tracked style-definition changes, content-control data binding, drag-and-drop moves | upstream | 2 |
| 7 | ✅ .odt export for Spanish administrations that use ODF. Checked 2026-10-28 with the Spain templates, then fixed: the export now writes headers and footers on the master pages (default, even and first page) with page-number and page-count fields, Word's top/bottom margins and header/footer distances, paragraph spacing, indents and line spacing, tab stops with leaders, paragraph borders, cell borders, layout tables without a grid and table alignment; the importer reads the page fields and the margins back. Checked by opening the exported templates in LibreOffice: they look as in Texto. Tests: `crates/formats/src/odt/tinta_tests.rs` and the templates' .odt round trip. Since 2026-10-28 the importer also reads a paragraph's own spacing before and after, indents, first-line indent, line height (proportional, exact, at least) and tab stops with their alignment and leaders, layout tables whose cells have no borders open without a grid, and a table keeps its alignment, so an .odt from LibreOffice or a Spanish administration keeps its layout when opened (checked with a quote saved by LibreOffice). Paragraph borders (a heading's rule, with colour, width and distance) and a plain common style's spacing before and after and line height (LibreOffice's Text body) are read too. Since 2026-10-30 a cell's padding is read as its margins (LibreOffice's 0.097 cm, a style's own sides) and saved to .docx, and the export writes a table's or cell's own margins as the cell padding (it used to write Word's defaults always, and a cell border with a distance wrote the padding twice). Next: offer it upstream | fork → upstream | 2 |
| 8 | Charts, SmartArt, ink (≈ 3% today) | upstream | 3 |
| 9 | A shared Tinta dictionary built from the words users add (see [below](#analysis-a-shared-dictionary-from-users-words)). Stage 2 ✅ (2026-10-28, [`crates/engine/src/tinta_suggest.rs`](../crates/engine/src/tinta_suggest.rs), `review.suggestWords`, `review.suggestedWords`): File › Options › Proofing › "Suggest my words for Tinta's dictionaries", off by default, the GDPR notice beside it in all 29 interface languages; while on, each word added to the dictionary is listed with its language on this computer only, words with digits, `@`, other non-letters or a capital inside are never listed, and the user removes words, then sends the list by email to hola@tintaoffice.com (Send by email opens a mailto link; Copy the list for webmail) or clears it; turning it off deletes the list. The privacy policy on tintaoffice.com repeats the notice (tintaoffice-web#8). Stage 3 (2026-10-30, `review.suggestWordsAuto`, [`apps/wordcraft/src/tinta_words.rs`](../apps/wordcraft/src/tinta_words.rs)): a second checkbox, "Send them automatically", off by default with its own notice, sends the listed es/ca/gl/eu/uk words once a day to report.tintaoffice.com under a random installation id (not the error reports' one) and marks them sent; turning it off asks the server to remove them from its counts. The server (tintaoffice-web `report-worker/words.js`) keeps only a fingerprint of the id and accepts a word, with no person in the loop, once 5 installs (10 for Spanish) added it, it passes the filters and the English Wiktionary lists it in that language; the rest are looked up again every 30 days; accepted words are public at `/v1/words/<lang>.txt`. Still to do: deploy the worker, and ship the accepted words in the pack (the proofing engine reading a second word list per language) | fork | 2 (button ✅), 3 (automatic ✅ in code) |

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
| Error reports | 1 | Automatic, only with consent (decided 2026-10-30): the first start asks "Send error reports automatically?"; File › Options › General changes it, shows the installation id and deletes the reports from the server. Panics and a window busy for 15 s write a scrubbed report (no document text, file names, paths, e-mail addresses, long numbers) to `<settings>/error-reports/pending`; while allowed it goes to report.tintaoffice.com (tintaoffice-web `report-worker/`: R2 in the EU, 90 days, one GitHub issue per distinct crash labelled `auto-report`). Texto: `crates/engine/src/tinta_reports.rs`, `crates/ui-egui/src/tinta_reports_ui.rs`, `apps/wordcraft/src/tinta_report.rs`, address in `.cargo/config.toml` (`TINTA_REPORT_URL`). |

Rules: upstream first (fixes useful to everyone go to storytold as PRs); every change ships with
automated tests and updated docs in the same PR; files people exchange (.docx, .xlsx, .pptx, PDF)
come first; Spain and Ukraine first (UI language, spelling, number/date formats, national
certificates).
