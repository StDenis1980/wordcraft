//! Tinta Office: the spelling dictionaries Tinta Texto ships, served as Language packs.
//!
//! The packages put Hunspell dictionaries for Spain's languages and Ukrainian in a `dictionaries`
//! folder beside the program (`packaging/dictionaries`, [`shipped_dirs`]), and the extra pack adds
//! more languages there. Proofing is craft-proof's, which reads a language's spelling and
//! hyphenation from an installed Language pack ([`crate::packs`]): so at start-up each shipped
//! dictionary is copied once into the packs folder as a pack (`pack.toml`, `spelling.aff`,
//! `spelling.dic`, `hyphenation.pat`, `LICENCES/`), with the hyphenation patterns this fork bundles
//! (`assets/hyphenation`, hyph-utf8) and the words Tinta's server accepted
//! (`tinta-words-<code>.txt`, appended to the word list). Options › Language then lists them as
//! installed, About › Language data shows their licences, and they can be removed there.
//!
//! A pack this module wrote says so in its `pack.toml` (`tinta = "<signature>"`); it is written
//! again when the shipped files change, and never over a pack the person installed from the
//! store (no `tinta` key). The locales written are remembered in `.tinta-seeded` in the packs
//! folder (a dotted name the pack scan skips), so a pack the person removed stays removed.
//! Nothing here panics: every failure is logged and leaves that pack as it was.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// A dictionary Tinta ships, and the pack it becomes.
struct Shipped {
    /// The pack's locale (its folder name), lowercase BCP 47.
    locale: &'static str,
    /// The language's own name.
    name: &'static str,
    /// The Hunspell pair's base name in the dictionaries folder (`es_ES` → `es_ES.aff`, `es_ES.dic`).
    file: &'static str,
    /// The accepted-words list (`tinta-words-<code>.txt`), if Tinta keeps one for the language.
    words: Option<&'static str>,
    /// Hyphenation patterns bundled with this fork (`assets/hyphenation`).
    hyphenation: Option<&'static str>,
    /// SPDX expression of the pack's data (dictionary, then patterns).
    licence: &'static str,
    /// The licence and readme files in the dictionaries folder's `licences/` that go with it.
    licences: &'static [&'static str],
}

const ES_PAT: &str = include_str!("../../../assets/hyphenation/es.pat");
const CA_PAT: &str = include_str!("../../../assets/hyphenation/ca.pat");
const GL_PAT: &str = include_str!("../../../assets/hyphenation/gl.pat");
const EU_PAT: &str = include_str!("../../../assets/hyphenation/eu.pat");
const UK_PAT: &str = include_str!("../../../assets/hyphenation/uk.pat");
const FR_PAT: &str = include_str!("../../../assets/hyphenation/fr.pat");
const LT_PAT: &str = include_str!("../../../assets/hyphenation/lt.pat");

/// The shipped dictionaries (`packaging/dictionaries/sources.tsv`, then `sources-extra.tsv`).
const SHIPPED: &[Shipped] = &[
    Shipped {
        locale: "es",
        name: "Español (España)",
        file: "es_ES",
        words: Some("es"),
        hyphenation: Some(ES_PAT),
        licence: "(GPL-3.0-or-later OR LGPL-3.0-or-later OR MPL-1.1) AND MIT",
        licences: &["es-LICENSE.md", "es-README.txt", "MPL-1.1.txt", "GPL-3.0.txt", "LGPL-3.0.txt"],
    },
    Shipped {
        locale: "ca",
        name: "Català",
        file: "ca",
        words: Some("ca"),
        hyphenation: Some(CA_PAT),
        licence: "(LGPL-2.1-or-later OR GPL-2.0-or-later) AND LPPL-1.3c",
        licences: &["ca-README.txt", "LGPL-2.1.txt", "GPL-2.0.txt"],
    },
    Shipped {
        locale: "ca-valencia",
        name: "Català (valencià)",
        file: "ca-valencia",
        words: Some("ca"),
        hyphenation: Some(CA_PAT),
        licence: "(LGPL-2.1-or-later OR GPL-2.0-or-later) AND LPPL-1.3c",
        licences: &["ca-README.txt", "LGPL-2.1.txt", "GPL-2.0.txt"],
    },
    Shipped {
        locale: "gl",
        name: "Galego",
        file: "gl_ES",
        words: Some("gl"),
        hyphenation: Some(GL_PAT),
        licence: "GPL-3.0-only AND Unlicense",
        licences: &["gl-README.txt", "GPL-3.0.txt"],
    },
    Shipped {
        locale: "eu",
        name: "Euskara",
        file: "eu",
        words: Some("eu"),
        hyphenation: Some(EU_PAT),
        licence: "GPL-2.0-only AND LicenseRef-hyph-eu",
        licences: &["eu-README.md", "GPL-2.0.txt"],
    },
    Shipped {
        locale: "uk",
        name: "Українська",
        file: "uk_UA",
        words: Some("uk"),
        hyphenation: Some(UK_PAT),
        licence: "MPL-1.1 AND MIT",
        licences: &["uk-README.txt", "MPL-1.1.txt"],
    },
    // The extra pack (a separate download unzipped into the dictionaries folder).
    Shipped {
        locale: "fr",
        name: "Français",
        file: "fr",
        words: None,
        hyphenation: Some(FR_PAT),
        licence: "MPL-2.0 AND MIT",
        licences: &["fr-LICENSE.txt", "fr-README.md"],
    },
    Shipped {
        locale: "ro",
        name: "Română",
        file: "ro_RO",
        words: None,
        hyphenation: None,
        licence: "GPL-2.0-only OR LGPL-2.1-only OR MPL-1.1",
        licences: &["GPL-2.0.txt", "LGPL-2.1.txt", "MPL-1.1.txt"],
    },
    Shipped {
        locale: "lt",
        name: "Lietuvių",
        file: "lt_LT",
        words: None,
        hyphenation: Some(LT_PAT),
        licence: "BSD-3-Clause AND MIT",
        licences: &["lt-COPYING.txt", "lt-README.txt"],
    },
    Shipped {
        locale: "pl",
        name: "Polski",
        file: "pl_PL",
        words: None,
        hyphenation: None,
        licence: "GPL-1.0-or-later OR LGPL-2.1-or-later OR MPL-1.1 OR Apache-2.0 OR CC-BY-SA-1.0",
        licences: &["pl-README.txt", "LGPL-2.1.txt", "MPL-1.1.txt"],
    },
    Shipped {
        locale: "it",
        name: "Italiano",
        file: "it_IT",
        words: None,
        hyphenation: None,
        licence: "GPL-3.0-only",
        licences: &["it-README.txt", "GPL-3.0.txt"],
    },
    Shipped {
        locale: "cs",
        name: "Čeština",
        file: "cs_CZ",
        words: None,
        hyphenation: None,
        licence: "GPL-2.0-only",
        licences: &["cs-README.txt", "GPL-2.0.txt"],
    },
    Shipped {
        locale: "de",
        name: "Deutsch (Deutschland)",
        file: "de_DE",
        words: None,
        hyphenation: None,
        licence: "GPL-2.0-only OR GPL-3.0-only",
        licences: &["de-README.txt", "GPL-2.0.txt", "GPL-3.0.txt"],
    },
    Shipped {
        locale: "de-ch",
        name: "Deutsch (Schweiz)",
        file: "de_CH",
        words: None,
        hyphenation: None,
        licence: "GPL-2.0-only OR GPL-3.0-only",
        licences: &["de-README.txt", "GPL-2.0.txt", "GPL-3.0.txt"],
    },
];

/// Bumped when the way packs are written changes, so every Tinta pack is written again.
const LAYOUT: u32 = 1;
/// The packs folder's record of the locales written here.
const LEDGER: &str = ".tinta-seeded";
/// Largest dictionary file copied (craft-proof refuses larger ones too).
const MAX_FILE: u64 = 64 << 20;
/// Largest accepted-words list or licence file read.
const MAX_SMALL: u64 = 4 << 20;
/// Most accepted words added to one dictionary.
const MAX_WORDS: usize = 200_000;
/// Longest accepted word kept, in characters (the server accepts 40).
const MAX_WORD_CHARS: usize = 64;

/// The `dictionaries` folders a package puts beside the executable `exe`: next to it on Windows,
/// `share/wordcraft/dictionaries` on Linux and in the Flatpak, `Contents/Resources/dictionaries`
/// on macOS.
pub fn bundled_dirs(exe: &Path) -> Vec<PathBuf> {
    let Some(bin) = exe.parent() else { return Vec::new() };
    let mut dirs = vec![bin.join("dictionaries")];
    if let Some(prefix) = bin.parent() {
        dirs.push(prefix.join("share").join("wordcraft").join("dictionaries"));
        dirs.push(prefix.join("Resources").join("dictionaries"));
    }
    dirs
}

/// [`bundled_dirs`] of the running program that exist.
pub fn shipped_dirs() -> Vec<PathBuf> {
    std::env::current_exe().map(|exe| bundled_dirs(&exe)).unwrap_or_default().into_iter().filter(|d| d.is_dir()).collect()
}

/// What [`seed_into`] did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Seeded {
    /// Locales written (new, or the shipped files changed).
    pub written: Vec<String>,
    /// (locale, reason) of packs that could not be written.
    pub failed: Vec<(String, String)>,
}

/// Write the shipped dictionaries into the packs folder in use (when there is one) and point
/// proofing at them again if anything changed. Returns what it did.
pub fn seed() -> Seeded {
    let Some(packs) = crate::packs::backend().and_then(|b| b.packs_dir()) else { return Seeded::default() };
    let shipped = shipped_dirs();
    if shipped.is_empty() {
        return Seeded::default();
    }
    let done = seed_into(&packs, &shipped);
    for (locale, e) in &done.failed {
        log::warn!("Tinta dictionary pack {locale}: {e}");
    }
    if !done.written.is_empty() {
        wordcraft_proof::packs::reload();
    }
    done
}

/// [`seed`] on a thread of its own (the desktop app: copying the dictionaries the first time takes
/// a moment), calling `then` when it wrote something.
#[cfg(not(target_arch = "wasm32"))]
pub fn seed_in_background(then: impl FnOnce() + Send + 'static) {
    let spawned = std::thread::Builder::new().name("tinta-packs".into()).spawn(move || {
        if !seed().written.is_empty() {
            then();
        }
    });
    if let Err(e) = spawned {
        log::warn!("Tinta dictionary packs: {e}");
    }
}

/// Write each dictionary found in the first of `shipped` that has it into `packs` as a Language
/// pack, unless one is there already that this module didn't write, or that it wrote from the
/// same files, or the person removed the one it wrote.
pub fn seed_into(packs: &Path, shipped: &[PathBuf]) -> Seeded {
    let mut out = Seeded::default();
    let mut ledger = read_ledger(packs);
    let mut ledger_changed = false;
    for s in SHIPPED {
        let Some(dir) = shipped.iter().find(|d| is_file(&d.join(format!("{}.aff", s.file))) && is_file(&d.join(format!("{}.dic", s.file)))) else {
            continue;
        };
        let signature = match signature(s, dir) {
            Ok(sig) => sig,
            Err(e) => {
                out.failed.push((s.locale.into(), e));
                continue;
            }
        };
        let target = packs.join(s.locale);
        let exists = std::fs::symlink_metadata(&target).is_ok();
        match (exists, pack_signature(&target)) {
            // The person's own pack (from the store, or copied in by hand).
            (true, None) => continue,
            (true, Some(sig)) if sig == signature => continue,
            // Removed in Options › Language after it was written here.
            (false, _) if ledger.iter().any(|l| l == s.locale) => continue,
            _ => {}
        }
        match write_pack(packs, s, dir, &signature) {
            Ok(()) => {
                out.written.push(s.locale.into());
                if !ledger.iter().any(|l| l == s.locale) {
                    ledger.push(s.locale.into());
                    ledger_changed = true;
                }
            }
            Err(e) => out.failed.push((s.locale.into(), e)),
        }
    }
    if ledger_changed && let Err(e) = std::fs::write(packs.join(LEDGER), ledger.join("\n") + "\n") {
        out.failed.push((LEDGER.into(), e.to_string()));
    }
    out
}

/// A regular file (never a symbolic link).
fn is_file(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|m| m.is_file())
}

fn read_ledger(packs: &Path) -> Vec<String> {
    read_small(&packs.join(LEDGER)).map(|t| t.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string).collect()).unwrap_or_default()
}

/// A small text file, at most [`MAX_SMALL`] bytes read; `None` when missing or not UTF-8.
fn read_small(path: &Path) -> Option<String> {
    if !is_file(path) {
        return None;
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path).ok()?.take(MAX_SMALL).read_to_end(&mut bytes).ok()?;
    String::from_utf8(bytes).ok()
}

/// What the pack written from `dir` depends on: the dictionary files' sizes and times, the
/// accepted words and the bundled patterns.
fn signature(s: &Shipped, dir: &Path) -> Result<String, String> {
    let stamp = |name: String| -> Result<String, String> {
        let m = std::fs::metadata(dir.join(&name)).map_err(|e| format!("{name}: {e}"))?;
        let at = m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs());
        Ok(format!("{}@{at}", m.len()))
    };
    let words = s.words.and_then(|w| read_small(&dir.join(words_file(w)))).unwrap_or_default();
    Ok(format!(
        "{LAYOUT}:{}:{}:{:016x}:{:016x}",
        stamp(format!("{}.aff", s.file))?,
        stamp(format!("{}.dic", s.file))?,
        fnv(words.as_bytes()),
        fnv(s.hyphenation.unwrap_or_default().as_bytes())
    ))
}

/// The accepted-words list's file name for `code` (packaging/dictionaries/fetch.sh).
fn words_file(code: &str) -> String {
    format!("tinta-words-{code}.txt")
}

/// FNV-1a, 64 bits: enough to tell a changed list from the same one.
fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01b3))
}

/// The `tinta` signature in the pack at `dir` (`None`: no pack, or not one written here).
fn pack_signature(dir: &Path) -> Option<String> {
    // Only this module writes the key, one line `tinta = "<signature>"` (no escapes in it).
    let text = read_small(&dir.join("pack.toml"))?;
    text.lines().find_map(|l| l.trim().strip_prefix("tinta = \"")?.strip_suffix('"').map(str::to_string))
}

/// Write the pack for `s` from `dir` into `packs`, replacing what is there as one step: it is
/// built in a dotted folder (which the pack scan skips) and renamed into place.
fn write_pack(packs: &Path, s: &Shipped, dir: &Path, signature: &str) -> Result<(), String> {
    std::fs::create_dir_all(packs).map_err(|e| e.to_string())?;
    let stage = packs.join(format!(".tinta-{}.new", s.locale));
    let old = packs.join(format!(".tinta-{}.old", s.locale));
    for d in [&stage, &old] {
        if std::fs::symlink_metadata(d).is_ok() {
            std::fs::remove_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
        }
    }
    let built = build(&stage, s, dir, signature);
    if let Err(e) = built {
        let _ = std::fs::remove_dir_all(&stage);
        return Err(e);
    }
    let target = packs.join(s.locale);
    let had = std::fs::symlink_metadata(&target).is_ok();
    if had {
        std::fs::rename(&target, &old).map_err(|e| format!("{}: {e}", target.display()))?;
    }
    if let Err(e) = std::fs::rename(&stage, &target) {
        if had {
            let _ = std::fs::rename(&old, &target);
        }
        let _ = std::fs::remove_dir_all(&stage);
        return Err(format!("{}: {e}", target.display()));
    }
    if had {
        let _ = std::fs::remove_dir_all(&old);
    }
    Ok(())
}

/// The pack's files in `stage`.
fn build(stage: &Path, s: &Shipped, dir: &Path, signature: &str) -> Result<(), String> {
    std::fs::create_dir_all(stage.join("LICENCES")).map_err(io("the pack folder"))?;
    let aff = dir.join(format!("{}.aff", s.file));
    let dic = dir.join(format!("{}.dic", s.file));
    for f in [&aff, &dic] {
        let len = std::fs::metadata(f).map_err(io(&f.display().to_string()))?.len();
        if len > MAX_FILE {
            return Err(format!("{}: larger than {MAX_FILE} bytes", f.display()));
        }
    }
    std::fs::copy(&aff, stage.join("spelling.aff")).map_err(io("spelling.aff"))?;
    std::fs::copy(&dic, stage.join("spelling.dic")).map_err(io("spelling.dic"))?;
    if let Some(code) = s.words
        && let Some(list) = read_small(&dir.join(words_file(code)))
    {
        let words = accepted_words(&list);
        // Only a UTF-8 dictionary takes them as they are (all of Tinta's are).
        if !words.is_empty() && aff_is_utf8(&aff) {
            let mut f = std::fs::OpenOptions::new().append(true).open(stage.join("spelling.dic")).map_err(io("spelling.dic"))?;
            let mut text = String::from("\n");
            for w in &words {
                text.push_str(w);
                text.push('\n');
            }
            f.write_all(text.as_bytes()).map_err(io("spelling.dic"))?;
        }
    }
    if let Some(p) = s.hyphenation {
        std::fs::write(stage.join("hyphenation.pat"), p).map_err(io("hyphenation.pat"))?;
        let notice: String = p.lines().take_while(|l| l.starts_with('%') || l.trim().is_empty()).map(|l| format!("{l}\n")).collect();
        if !notice.trim().is_empty() {
            std::fs::write(stage.join("LICENCES").join("hyphenation.txt"), notice).map_err(io("LICENCES/hyphenation.txt"))?;
        }
    }
    for name in s.licences {
        let from = dir.join("licences").join(name);
        if let Some(text) = read_small(&from) {
            std::fs::write(stage.join("LICENCES").join(name), text).map_err(io(name))?;
        }
    }
    let toml = format!(
        "# Written by Tinta Texto from the dictionaries it ships (packaging/dictionaries).\nlocale = {}\nname = {}\nlicence = {}\ntinta = {}\n",
        quoted(s.locale),
        quoted(s.name),
        quoted(s.licence),
        quoted(signature)
    );
    std::fs::write(stage.join("pack.toml"), toml).map_err(io("pack.toml"))?;
    Ok(())
}

/// An I/O failure while writing `what`, as the message [`Seeded::failed`] carries.
fn io(what: &str) -> impl Fn(std::io::Error) -> String + '_ {
    move |e| format!("{what}: {e}")
}

/// A TOML basic string.
fn quoted(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Whether an `.aff` file says its words are UTF-8 (`SET UTF-8`), reading its first lines only.
fn aff_is_utf8(aff: &Path) -> bool {
    let Ok(f) = std::fs::File::open(aff) else { return false };
    let mut head = Vec::new();
    if f.take(64 << 10).read_to_end(&mut head).is_err() {
        return false;
    }
    String::from_utf8_lossy(&head).lines().map(str::trim).any(|l| {
        let mut parts = l.split_whitespace();
        parts.next() == Some("SET") && parts.next().is_some_and(|v| v.eq_ignore_ascii_case("UTF-8"))
    })
}

/// The words of an accepted-words list as dictionary lines: composed (NFC), typographic
/// apostrophes as `'`, trimmed; comments, empty lines, over-long words and anything a word list
/// would read as flags or a count (`/`, spaces, a leading digit run) skipped; at most
/// [`MAX_WORDS`], each once.
fn accepted_words(text: &str) -> Vec<String> {
    use unicode_normalization::UnicodeNormalization;
    let mut out: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for l in text.lines().map(str::trim) {
        if l.is_empty() || l.starts_with('#') || l.chars().count() > MAX_WORD_CHARS {
            continue;
        }
        let w: String = l.nfc().map(|c| if matches!(c, '’' | 'ʼ') { '\'' } else { c }).collect();
        if w.contains(['/', '\t', ' ', '\\']) || w.chars().all(|c| c.is_ascii_digit()) || !w.chars().any(char::is_alphabetic) {
            continue;
        }
        if seen.insert(w.clone()) {
            out.push(w);
            if out.len() >= MAX_WORDS {
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests;
