# Tinta Office spelling dictionaries

Tinta Texto ships Hunspell spelling dictionaries for the languages of Spain and for Ukrainian, so
spelling works right after installation, with nothing else to download. They are not part of the
source tree: `fetch.sh` (Linux, macOS) and `fetch.ps1` (Windows) download the files listed in
[`sources.tsv`](sources.tsv), each pinned to a commit and checked against its SHA-256, and the
packaging scripts put them in the app's `dictionaries` folder:

| Package | Folder |
|---|---|
| Windows MSI and portable zip | `dictionaries\` next to `wordcraft.exe` |
| .deb, .rpm, .tar.gz, AppImage | `/usr/share/wordcraft/dictionaries/` (`share/wordcraft/dictionaries` beside `bin`) |
| macOS app | `Tinta Texto.app/Contents/Resources/dictionaries/` |

The app looks there after the user's own dictionary folders (File › Options › Proofing, and
`WORDCRAFT_DICTIONARIES`), so a dictionary the user installs always wins. A language whose
dictionary is missing simply has no spelling marks.

## The dictionaries and their licences

Each dictionary is a separate work under its own licence, distributed unchanged next to the app
(an aggregate, not a combined work): the app is MIT/Apache-2.0 and reads the files at run time.
The licence texts and the original readme files are in `licences/`; the files themselves are the
form their authors publish and edit, and their sources are listed below.

| File | Language | Source | Licence |
|---|---|---|---|
| `es_ES.aff`, `es_ES.dic` | Spanish (Spain) | RLA-ES, from [LibreOffice dictionaries](https://github.com/LibreOffice/dictionaries/tree/master/es) | GPL-3.0+, LGPL-3.0+ or MPL-1.1+, at your option |
| `ca.aff`, `ca.dic` | Catalan (central standard) | Softcatalà, by Jaume Ortolà, from [LibreOffice dictionaries](https://github.com/LibreOffice/dictionaries/tree/master/ca) | LGPL-2.1+ or GPL-2.0+, at your option |
| `ca-valencia.aff`, `ca-valencia.dic` | Valencian (text marked Catalan (Valencian), `ca-ES-valencia`) | Softcatalà, by Jaume Ortolà, from [LibreOffice dictionaries](https://github.com/LibreOffice/dictionaries/tree/master/ca) | LGPL-2.1+ or GPL-2.0+, at your option |
| `gl_ES.aff`, `gl_ES.dic` | Galician | Proxecto Trasno, from [LibreOffice dictionaries](https://github.com/LibreOffice/dictionaries/tree/master/gl) | GPL-3.0 |
| `uk_UA.aff`, `uk_UA.dic` | Ukrainian | [dict_uk](https://github.com/brown-uk/dict_uk) by Andriy Rysin and contributors, from [LibreOffice dictionaries](https://github.com/LibreOffice/dictionaries/tree/master/uk_UA) | MPL-1.1 |
| `eu.aff`, `eu.dic` | Basque | Xuxen ([xuxen.eus](https://xuxen.eus)), as packaged by [wooorm/dictionaries](https://github.com/wooorm/dictionaries/tree/main/dictionaries/eu) | GPL-2.0 |

Licence texts: `MPL-1.1.txt`, `GPL-2.0.txt`, `GPL-3.0.txt`, `LGPL-2.1.txt`, `LGPL-3.0.txt`.

## The extra pack (separate download)

French, Romanian, Lithuanian, Polish, Italian, Czech and German are a separate download of
about 20 MB, so the main pack stays small. [`sources-extra.tsv`](sources-extra.tsv) lists them the same
way; `fetch.sh DEST sources-extra.tsv` (Windows: `fetch.ps1 -Dest DEST -Manifest
sources-extra.tsv`) fills a folder to zip as a release asset. Unzipped into the app's
`dictionaries` folder, or a folder set in File › Options › Proofing, the languages are checked. On
Linux the distribution's `hunspell-fr`, `hunspell-ro`… packages work too: the app also reads
`/usr/share/hunspell`.

German is LibreOffice's `de_DE_frami` (igerman98 with additions), saved as `de_DE`: it builds
compound words (`Rechnungsnummer`, `Krankenversicherungskarte`) from parts the dictionary marks,
which the spelling engine reads. Swiss German writes `ss` for `ß`, which this dictionary flags;
a system `de_CH` dictionary isn't used for it yet.

| File | Language | Source | Licence |
|---|---|---|---|
| `fr.aff`, `fr.dic` | French | Grammalecte's dictionary (Olivier R.), as packaged by [wooorm/dictionaries](https://github.com/wooorm/dictionaries/tree/main/dictionaries/fr) | MPL-2.0 |
| `ro_RO.aff`, `ro_RO.dic` | Romanian | rospell (Lucian Constantin), from [LibreOffice dictionaries](https://github.com/LibreOffice/dictionaries/tree/master/ro) | GPL-2.0, LGPL-2.1 or MPL-1.1, at your option |
| `lt_LT.aff`, `lt_LT.dic` | Lithuanian | [ispell-lt](https://github.com/ispell-lt/ispell-lt) by Albertas Agejevas and contributors, from [LibreOffice dictionaries](https://github.com/LibreOffice/dictionaries/tree/master/lt_LT) | BSD-3-Clause |
| `pl_PL.aff`, `pl_PL.dic` | Polish | [sjp.pl](https://sjp.pl), from [LibreOffice dictionaries](https://github.com/LibreOffice/dictionaries/tree/master/pl_PL) | GPL, LGPL, MPL, Apache-2.0 or CC BY-SA, at your option |
| `it_IT.aff`, `it_IT.dic` | Italian | Italian dictionary project, from [LibreOffice dictionaries](https://github.com/LibreOffice/dictionaries/tree/master/it_IT) | GPL-3.0 |
| `cs_CZ.aff`, `cs_CZ.dic` | Czech | Czech dictionary (from csTeX), from [LibreOffice dictionaries](https://github.com/LibreOffice/dictionaries/tree/master/cs_CZ) | GPL-2.0 |
| `de_DE.aff`, `de_DE.dic` | German (Germany, Austria) | `de_DE_frami`: igerman98 by Björn Jacke, extended by Franz Michael Baumann, from [LibreOffice dictionaries](https://github.com/LibreOffice/dictionaries/tree/master/de) | GPL-2.0 or GPL-3.0 |

The extra pack carries its own copies of the licence texts it needs, so it stands alone.

## Tinta's own word lists

`tinta-words/<lang>.txt` (es, ca, gl, eu, uk) hold the words Tinta Texto's users suggested with
"Send them automatically" and Tinta's server accepted (each added by at least 5 installations, 10
for Spanish, and listed by the English Wiktionary in that language). The fetch scripts copy them
beside the dictionaries as `tinta-words-<lang>.txt`, and the spelling check accepts their words in
that language (crate `proof`, module `tinta_words`). They are lists of words, released as CC0 1.0.
Before a release, `update-tinta-words.sh` refreshes them from the server; review the diff (a word
reported as wrong is rejected on the server and drops out at the next refresh) and commit it.

## Updating

Change the commit in the URLs of `sources.tsv`, then the SHA-256 column (`sha256sum` of each
downloaded file), and run the check with the new files:

```sh
packaging/dictionaries/fetch.sh /tmp/dicts
TINTA_DICTIONARIES_DIR=/tmp/dicts cargo test -p wordcraft-proof --test tinta_dictionaries -- --include-ignored
```
