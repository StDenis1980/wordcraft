# Tinta Texto — Tinta Office roadmap

> **Status:** 2026-10-27 · approved by the owner (Denys) · **Revision history:** 2026-10-27 created.

Tinta Texto is the Tinta Office build of WordCraft and replaces Microsoft Word. Upstream keeps the
feature-parity roadmap ([ROADMAP.md](../ROADMAP.md), [roadmap.md](roadmap.md),
[gaps.md](gaps.md)); this page lists only what the fork does and which upstream gaps matter most
for users in Spain and Ukraine. It is a fork-only file, so upstream syncs never conflict on it.

## Where it stands (2026-10-27)

Ready for real work ≈ 60% (essentials user ≈ 63%, upstream estimate); 157 commits behind
upstream. Styles, lists, tables, review and mail merge work; the main risk is other people's
.docx: never tested on real files, and charts and SmartArt are dropped silently. Spanish spelling
on Windows needs a Hunspell dictionary the user installs (none bundled), and there is no Ukrainian
spelling.

## Priorities

| # | Task | Where | Stage |
|---|---|---|---|
| 1 | Tinta dictionary pack: es_ES, ca, gl, eu, uk as a separate download or installer option, after a licence check (several are GPL/LGPL/MPL); Ukrainian spelling support upstream | fork (pack) + upstream (code) | 1 |
| 2 | Our .docx test set: typical self-employed documents (invoice, contract, CV, letter) written from scratch; an "open → save → nothing lost" test | fork (tests) → fixes upstream | 1 |
| 3 | Keep charts, SmartArt and OLE through open and save (upstream item 3) | upstream | 1–2 |
| 4 | Templates for Spain: factura, presupuesto, carta formal, currículum — our own text and colours, no Microsoft assets | fork or upstream | 2 |
| 5 | Pagination like Word: column balancing, footnote continuation, Aptos/Calibri substitutes | upstream | 2 |
| 6 | Formatting revisions and content controls | upstream | 2 |
| 7 | .odt export for Spanish administrations that use ODF: check the current state | our check | 2 |
| 8 | Charts, SmartArt, ink (≈ 3% today) | upstream | 3 |

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
