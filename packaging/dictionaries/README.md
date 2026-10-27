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
| `ca.aff`, `ca.dic` | Catalan (central standard; the Valencian `ca-valencia` dictionary is not shipped yet) | Softcatalà, by Jaume Ortolà, from [LibreOffice dictionaries](https://github.com/LibreOffice/dictionaries/tree/master/ca) | LGPL-2.1+ or GPL-2.0+, at your option |
| `gl_ES.aff`, `gl_ES.dic` | Galician | Proxecto Trasno, from [LibreOffice dictionaries](https://github.com/LibreOffice/dictionaries/tree/master/gl) | GPL-3.0 |
| `uk_UA.aff`, `uk_UA.dic` | Ukrainian | [dict_uk](https://github.com/brown-uk/dict_uk) by Andriy Rysin and contributors, from [LibreOffice dictionaries](https://github.com/LibreOffice/dictionaries/tree/master/uk_UA) | MPL-1.1 |
| `eu.aff`, `eu.dic` | Basque | Xuxen ([xuxen.eus](https://xuxen.eus)), as packaged by [wooorm/dictionaries](https://github.com/wooorm/dictionaries/tree/main/dictionaries/eu) | GPL-2.0 |

Licence texts: `MPL-1.1.txt`, `GPL-2.0.txt`, `GPL-3.0.txt`, `LGPL-2.1.txt`, `LGPL-3.0.txt`.

## Updating

Change the commit in the URLs of `sources.tsv`, then the SHA-256 column (`sha256sum` of each
downloaded file), and run the check with the new files:

```sh
packaging/dictionaries/fetch.sh /tmp/dicts
TINTA_DICTIONARIES_DIR=/tmp/dicts cargo test -p wordcraft-proof --test tinta_dictionaries -- --include-ignored
```
