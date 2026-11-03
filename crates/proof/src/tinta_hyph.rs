//! Tinta Office: hyphenation patterns read at runtime from Hunspell hyphenation files
//! (`hyph_cs_CZ.dic`, LibreOffice's format), for languages whose patterns can't be bundled:
//! Czech (GPL-2) and Romanian (GPL-2). They come in the extra dictionary pack
//! (`packaging/dictionaries/sources-extra.tsv`) or with the system's hyphenation packages, and are
//! looked for in the same folders as the spelling dictionaries ([`crate::hunspell::search_dirs`]).
//! Without one, the language keeps the English fallback.
//!
//! The format: the first line names the encoding (`UTF-8`, `ISO8859-2`…), then Liang patterns,
//! `%` comments and optional directives in capitals (`LEFTHYPHENMIN 2`, `NEXTLEVEL`). Only plain
//! patterns are read: non-standard ones (`c1c/c=c,1,2`) and the directives are skipped, and
//! patterns after `NEXTLEVEL` (compound-word levels) are ignored.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::lang::ProofLang;
use crate::patterns::Patterns;

/// The file names looked for, by language.
fn names(lang: ProofLang) -> &'static [&'static str] {
    match lang {
        ProofLang::Cs => &["hyph_cs_CZ", "hyph_cs"],
        ProofLang::Ro => &["hyph_ro_RO", "hyph_ro"],
        _ => &[],
    }
}

/// Parsed files by language, with the [`crate::dictionary_generation`] they were looked up in
/// (`None`: no file, or one that couldn't be read).
type Cache = HashMap<ProofLang, (u64, Option<Arc<Patterns>>)>;

static CACHE: RwLock<Option<Cache>> = RwLock::new(None);

/// The installed patterns for `lang`, if it is a language this module serves and a file is found.
/// Looked up once per dictionary-folder change.
pub fn patterns(lang: ProofLang) -> Option<Arc<Patterns>> {
    let names = names(lang);
    if names.is_empty() {
        return None;
    }
    let generation = crate::dictionary_generation();
    if let Some((g, p)) = CACHE.read().unwrap_or_else(std::sync::PoisonError::into_inner).as_ref().and_then(|c| c.get(&lang))
        && *g == generation
    {
        return p.clone();
    }
    let found = find(names).and_then(|path| match load(&path) {
        Ok(p) if !p.is_empty() => Some(Arc::new(p)),
        Ok(_) => {
            log::warn!("{}: no hyphenation patterns", path.display());
            None
        }
        Err(e) => {
            log::warn!("{e}");
            None
        }
    });
    let mut cache = CACHE.write().unwrap_or_else(std::sync::PoisonError::into_inner);
    cache.get_or_insert_with(HashMap::new).insert(lang, (generation, found.clone()));
    found
}

/// The first `<name>.dic` for any of `names` in the dictionary folders.
fn find(names: &[&str]) -> Option<PathBuf> {
    crate::hunspell::search_dirs(None).into_iter().find_map(|dir| find_in(&dir, names))
}

/// The first `<name>.dic` for any of `names` in `dir`.
pub fn find_in(dir: &Path, names: &[&str]) -> Option<PathBuf> {
    names.iter().map(|n| dir.join(format!("{n}.dic"))).find(|p| p.is_file())
}

/// Read and parse a hyphenation file (size-capped like the dictionaries).
pub fn load(path: &Path) -> Result<Patterns, String> {
    parse(&crate::hunspell::read_capped(path)?).map_err(|e| format!("{}: {e}", path.display()))
}

/// Parse a hyphenation file's bytes: the encoding on the first line, then the patterns.
pub fn parse(bytes: &[u8]) -> Result<Patterns, String> {
    let end = bytes.iter().position(|&b| b == b'\n').unwrap_or(bytes.len());
    let (first, body) = bytes.split_at(end);
    let charset = String::from_utf8_lossy(first).trim().trim_start_matches('\u{feff}').to_string();
    if charset.is_empty() || charset.contains(char::is_whitespace) {
        return Err(format!("no encoding on the first line ({charset:?})"));
    }
    let (_, text) = crate::hunspell::decode(format!("SET {charset}\n").as_bytes(), body)?;
    let mut plain = String::with_capacity(text.len());
    'lines: for line in text.lines() {
        let line = line.split('%').next().unwrap_or("").trim();
        for tok in line.split_whitespace() {
            if tok == "NEXTLEVEL" {
                break 'lines;
            }
            // Directives (`LEFTHYPHENMIN 2`) start with a capital; their numbers follow on the line.
            if tok.starts_with(|c: char| c.is_ascii_uppercase()) {
                continue 'lines;
            }
            // Non-standard patterns and anything TeX-like aren't plain Liang patterns.
            if tok.contains(['/', '=', ',', '-', '\\', '{', '}']) {
                continue;
            }
            plain.push_str(tok);
            plain.push('\n');
        }
    }
    Ok(Patterns::parse(&plain))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_encoding_patterns_and_skips_the_rest() {
        // ISO 8859-2: `č` is 0xE8.
        let p = parse(b"ISO8859-2\nLEFTHYPHENMIN 2\n% a comment\n1\xe8\nx1y\nc1c/c=c,1,2\nNEXTLEVEL\n1x\n").unwrap();
        assert_eq!(p.len(), 2, "1č and x1y only");
        assert_eq!(p.points(&"xyč".chars().collect::<Vec<_>>()), Some(vec![1, 2]));
        assert!(parse(b"UTF-8\n1\xff\n").is_err(), "not UTF-8");
        assert!(parse(b"").is_err() && parse(b"\n1a\n").is_err() && parse(b"SHIFT_JIS\n1a\n").is_err());
        assert!(parse(b"UTF-8").unwrap().is_empty());
    }

    #[test]
    fn only_czech_and_romanian_are_looked_for() {
        assert!(patterns(ProofLang::En).is_none() && patterns(ProofLang::Fr).is_none());
        let dir = std::env::temp_dir().join(format!("wordcraft-tinta-hyph-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("hyph_ro.dic"), "UTF-8\n1b\n").unwrap();
        assert_eq!(find_in(&dir, names(ProofLang::Ro)), Some(dir.join("hyph_ro.dic")));
        assert_eq!(find_in(&dir, names(ProofLang::Cs)), None);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
