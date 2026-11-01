//! Tinta Office: the words Tinta Texto's users suggested and Tinta's server accepted (roadmap
//! priority 9, stage 3), a supplementary word list per language next to the Hunspell dictionaries.
//!
//! The release puts `tinta-words-<code>.txt` (`es`, `ca`, `gl`, `eu`, `uk`) in the app's
//! dictionaries folder (packaging/dictionaries/tinta-words). Each line is one word; empty lines and
//! lines starting with `#` are skipped. A word there is spelled correctly in its language, also
//! capitalized or in capitals (`Tintero`, `TINTERO`), the way the dictionaries treat their own
//! lower-case words. The lists are read from the same folders as the dictionaries
//! ([`crate::hunspell::search_dirs`]), the first found wins, and a missing or unreadable list is
//! simply an empty one.

use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::Path;
use std::sync::{PoisonError, RwLock};

use crate::ProofLang;

/// Largest list read, and most words kept from it: real lists stay far below.
const MAX_FILE: u64 = 4 << 20;
const MAX_WORDS: usize = 200_000;
/// Longest word kept, in characters (the server accepts 40).
const MAX_CHARS: usize = 64;

/// Loaded lists by language code; `None` until first used, cleared by [`reload`].
static LISTS: RwLock<Option<HashMap<&'static str, HashSet<String>>>> = RwLock::new(None);

/// The list's language code, for the languages Tinta ships one for.
fn code(lang: ProofLang) -> Option<&'static str> {
    match lang {
        ProofLang::Es | ProofLang::EsMx | ProofLang::Es419 | ProofLang::EsUy => Some("es"),
        ProofLang::Ca | ProofLang::CaValencia => Some("ca"),
        ProofLang::Gl => Some("gl"),
        ProofLang::Eu => Some("eu"),
        ProofLang::Uk => Some("uk"),
        _ => None,
    }
}

/// The file name of `code`'s list.
pub fn file_name(code: &str) -> String {
    format!("tinta-words-{code}.txt")
}

/// The words of one list file: composed (NFC), typographic apostrophes as `'`, trimmed; comments,
/// empty lines and over-long words skipped; at most [`MAX_WORDS`].
pub fn parse(text: &str) -> HashSet<String> {
    use unicode_normalization::UnicodeNormalization;
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#') && l.chars().count() <= MAX_CHARS)
        .map(|l| l.nfc().map(|c| if matches!(c, '’' | 'ʼ') { '\'' } else { c }).collect::<String>())
        .take(MAX_WORDS)
        .collect()
}

fn read(path: &Path) -> Option<HashSet<String>> {
    let file = std::fs::File::open(path).ok()?;
    let mut bytes = Vec::new();
    file.take(MAX_FILE).read_to_end(&mut bytes).ok()?;
    Some(parse(&String::from_utf8_lossy(&bytes)))
}

fn load(code: &'static str) -> HashSet<String> {
    let name = file_name(code);
    crate::hunspell::search_dirs(None).iter().map(|d| d.join(&name)).find(|p| p.is_file()).and_then(|p| read(&p)).unwrap_or_default()
}

/// Is `word` (composed) in Tinta's list for `lang`? Also capitalized or all capitals.
pub fn known(word: &str, lang: ProofLang) -> bool {
    let Some(code) = code(lang) else { return false };
    let lower = word.to_lowercase().replace(['’', 'ʼ'], "'");
    // As written, capitalized or in capitals; `pressuPostar` is not the listed word.
    let mut rest = word.chars().skip(1);
    if !(rest.all(|c| !c.is_uppercase()) || word.chars().all(|c| !c.is_lowercase())) {
        return false;
    }
    let found = |lists: &HashMap<&'static str, HashSet<String>>| lists.get(code).map(|set| set.contains(&lower));
    if let Some(hit) = LISTS.read().unwrap_or_else(PoisonError::into_inner).as_ref().and_then(found) {
        return hit;
    }
    let mut lists = LISTS.write().unwrap_or_else(PoisonError::into_inner);
    let lists = lists.get_or_insert_with(HashMap::new);
    lists.entry(code).or_insert_with(|| load(code)).contains(&lower)
}

/// Forget the loaded lists (the dictionaries folder changed); they are read again when used.
pub fn reload() {
    *LISTS.write().unwrap_or_else(PoisonError::into_inner) = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_are_parsed_leniently() {
        let words = parse("# Tinta Office: words added by Tinta Texto users, es.\n\ntintero\n  presupuestar \nm’ясо\nte\u{301}rmino\n# x\n");
        let mut v: Vec<_> = words.into_iter().collect();
        v.sort();
        assert_eq!(v, ["m'ясо", "presupuestar", "tintero", "término"]);
        assert!(parse(&"a".repeat(MAX_CHARS + 1)).is_empty());
        assert!(parse("").is_empty());
    }

    #[test]
    fn only_tintas_languages_have_a_list() {
        assert_eq!(code(ProofLang::EsMx), Some("es"));
        assert_eq!(code(ProofLang::CaValencia), Some("ca"));
        assert_eq!(code(ProofLang::Pl), None);
        assert!(!known("tintero", ProofLang::Pl));
    }
}
