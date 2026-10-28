# Tinta Texto — Tinta Office roadmap

> **Status:** 2026-10-27 · approved by the owner (Denys) · **Revision history:** 2026-10-27 created;
> 2026-10-27 dictionary pack and uk/ca/gl/eu spelling (priority 1) done; 2026-10-28 analysis of a
> shared dictionary built from users' added words (priority 9).

Tinta Texto is the Tinta Office build of WordCraft and replaces Microsoft Word. Upstream keeps the
feature-parity roadmap ([ROADMAP.md](../ROADMAP.md), [roadmap.md](roadmap.md),
[gaps.md](gaps.md)); this page lists only what the fork does and which upstream gaps matter most
for users in Spain and Ukraine. It is a fork-only file, so upstream syncs never conflict on it.

## Where it stands (2026-10-27)

Ready for real work ≈ 60% (essentials user ≈ 63%, upstream estimate); 157 commits behind
upstream. Styles, lists, tables, review and mail merge work; the main risk is other people's
.docx: never tested on real files, and charts and SmartArt are dropped silently. Spelling in
Spanish, Catalan, Galician, Basque and Ukrainian works out of the box (dictionaries ship with the
app since Preview 3).

## Priorities

| # | Task | Where | Stage |
|---|---|---|---|
| 1 | ✅ Tinta dictionary pack: es_ES, ca, gl_ES, eu, uk_UA ship in every package ([packaging/dictionaries](../packaging/dictionaries/README.md): pinned sources, SHA-256, licences); spelling in Ukrainian, Catalan, Galician and Basque (crate `proof`, module `plain`; the affix engine reads long and numeric flags, up to 512 flags, IGNORE, ICONV, NEEDAFFIX, KEEPCASE). Offer the code upstream. Next: the Valencian `ca-valencia` dictionary, hyphenation patterns for these languages | fork (pack) + upstream (code) | 1 |
| 2 | Our .docx test set: typical self-employed documents (invoice, contract, CV, letter) written from scratch; an "open → save → nothing lost" test | fork (tests) → fixes upstream | 1 |
| 3 | Keep charts, SmartArt and OLE through open and save (upstream item 3) | upstream | 1–2 |
| 4 | Templates for Spain: factura, presupuesto, carta formal, currículum — our own text and colours, no Microsoft assets | fork or upstream | 2 |
| 5 | Pagination like Word: column balancing, footnote continuation, Aptos/Calibri substitutes | upstream | 2 |
| 6 | Formatting revisions and content controls | upstream | 2 |
| 7 | .odt export for Spanish administrations that use ODF: check the current state | our check | 2 |
| 8 | Charts, SmartArt, ink (≈ 3% today) | upstream | 3 |
| 9 | Analysis: a shared Tinta dictionary built from the words users add (see [below](#analysis-a-shared-dictionary-from-users-words)) | fork | 2 (analysis), 3 (build) |

## Analysis: a shared dictionary from users' words

Owner's proposal (2026-10-28): people add words the dictionaries lack (Add to Dictionary); collect
them, check that they really exist, and add the right ones to our dictionaries, so every user gets
them with the next update. To analyse in stage 2, before any code:

- **Privacy first.** Added words are often names, surnames, company names, street names, DNI/NIE
  or account numbers: personal data under the GDPR. Sending must be opt-in (off by default),
  show exactly which words go out, send no document text and no user id, and skip anything with
  digits, `@` or a capital inside the word. Publish what is collected in the privacy policy on
  tintaoffice.com.
- **What we have today.** The custom dictionary is one list for every language (File › Options ›
  Proofing, `ui.json`, at most 10,000 words) and does not record the language a word was added
  in. Collecting needs the language with each word.
- **Collection.** Simplest first step: a "Suggest these words for Tinta's dictionaries" button
  that prepares the list for the user to review and send (email to hola@tintaoffice.com or a
  form on the site). An automatic channel needs a small server (for example a Cloudflare Worker
  next to the site) and counts each word once per install.
- **Validation.** Accept a word only when several different users added it, then check it against
  public sources (RAE's DLE for Spanish, VESUM/dict_uk for Ukrainian, Softcatalà's DIEC lookups
  for Catalan, the RAG for Galician, Euskaltzaindia for Basque); a native speaker approves each
  batch. AI may pre-sort, never approve.
- **Where the words go.** Best: as contributions to the dictionary projects themselves (RLA-ES,
  Softcatalà, Proxecto Trasno, dict_uk, Xuxen), so every app gains them and our pack stays
  unchanged upstream text. Faster: our own small supplementary word list per language, shipped
  in the pack under a permissive licence. That needs the proofing engine to read a second word
  list next to each Hunspell dictionary (not supported today).
- **Updates.** Words arrive with each monthly release; a separate dictionary update without
  reinstalling comes later.
- **Owner decisions.** Whether to collect at all, manual or automatic channel, server and cost,
  who validates each language.

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
