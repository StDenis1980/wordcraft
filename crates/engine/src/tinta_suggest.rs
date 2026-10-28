//! Tinta Office: "Suggest my words for Tinta's dictionaries" (roadmap priority 9, stage 2).
//!
//! While the option is on (File › Options › Proofing, off by default), each word added to the
//! custom dictionary is listed with its language, on this computer only. Nothing is sent by
//! itself: the user reviews the list and sends it by email ([`mailto`]) or copies it ([`text`]).
//! Words that may be personal data are never listed ([`eligible`]): anything with digits, `@` or
//! other non-letters, and anything with a capital inside the word. Turning the option off deletes
//! the list.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{CmdError, CommandSpec, Session};

/// Where the words go (the Tinta Office mailbox named in the notice beside the option).
pub const TO: &str = "hola@tintaoffice.com";

/// The most words the list keeps; later words are not listed until it is sent and cleared.
pub const MAX_WORDS: usize = 1000;

/// The longest `mailto:` link built, in bytes: mail clients and Windows cut longer links. The
/// words that don't fit stay in the list for the next email (or go with Copy).
pub const MAX_MAILTO_BYTES: usize = 1900;

/// Most words one `review.suggestedWords` call removes.
const MAX_REMOVE_PER_CALL: usize = 1000;

/// The saved state: whether the option is on and the words listed so far.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SuggestPrefs {
    pub enabled: bool,
    pub words: Vec<SuggestedWord>,
}

/// A listed word and the primary subtag of the language it was added in (`es`, `uk`, `und`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SuggestedWord {
    pub word: String,
    pub lang: String,
}

/// A damaged saved value reads as off with no words; saved words go through the same filters as
/// new ones, so a hand-edited file can't list what the app would refuse.
pub fn lenient<'de, D: serde::Deserializer<'de>>(d: D) -> Result<SuggestPrefs, D::Error> {
    let value = Value::deserialize(d)?;
    let saved: SuggestPrefs = serde_json::from_value(value).unwrap_or_default();
    let mut out = SuggestPrefs { enabled: saved.enabled, words: Vec::new() };
    if out.enabled {
        for w in saved.words {
            push(&mut out, &w.word, &w.lang);
        }
    }
    Ok(out)
}

/// May this word be suggested? Letters only (with the apostrophe, hyphen and Catalan middle dot
/// inside words, and combining accents), at most [`wordcraft_proof::MAX_CUSTOM_WORD_CHARS`], and no
/// capital after the first letter: names like `McDonald` or `iPhone`, codes, numbers, addresses
/// and email addresses stay on this computer.
pub fn eligible(word: &str) -> bool {
    let w = word.trim();
    if wordcraft_proof::custom_word(w).is_none() || !w.chars().any(char::is_alphabetic) {
        return false;
    }
    let letters = w.chars().all(|c| c.is_alphabetic() || matches!(c, '\'' | '’' | '-' | '·' | '\u{0300}'..='\u{036F}'));
    letters && !w.chars().skip(1).any(char::is_uppercase)
}

/// The language code a word is listed under: the proofing language's primary subtag, else the
/// tag's own (`fr-FR` → `fr`), else `und`.
pub fn lang_code(tag: Option<&str>) -> String {
    let code = wordcraft_proof::ProofLang::from_tag(tag).code();
    if !code.is_empty() {
        return code.to_string();
    }
    tag.and_then(|t| t.split(['-', '_']).next())
        .filter(|p| (2..=3).contains(&p.len()) && p.chars().all(|c| c.is_ascii_alphabetic()))
        .map(str::to_ascii_lowercase)
        .unwrap_or_else(|| "und".into())
}

/// The language of the text at the caret, as [`lang_code`] lists it.
pub fn caret_lang(s: &Session) -> String {
    let tag = s.doc.para_at(&s.sel.focus).and_then(|p| crate::cmd::para_lang(&s.doc, p)).or_else(|| crate::cmd::default_lang(s));
    lang_code(tag.as_deref())
}

/// List a word just added to the custom dictionary, if the option is on and the word may be
/// suggested. Returns whether it was listed.
pub fn record(s: &mut Session, word: &str, lang: &str) -> bool {
    s.prefs.tinta_suggest.enabled && push(&mut s.prefs.tinta_suggest, word, lang)
}

/// A word removed from the custom dictionary leaves the list too, in every language.
pub fn forget(s: &mut Session, word: &str) {
    let key = word.trim().to_lowercase();
    s.prefs.tinta_suggest.words.retain(|w| w.word.to_lowercase() != key);
}

fn push(list: &mut SuggestPrefs, word: &str, lang: &str) -> bool {
    let word = word.trim();
    let lang = lang_code(Some(lang));
    if !eligible(word) || list.words.len() >= MAX_WORDS {
        return false;
    }
    let key = word.to_lowercase();
    if list.words.iter().any(|w| w.lang == lang && w.word.to_lowercase() == key) {
        return false;
    }
    list.words.push(SuggestedWord { word: word.to_string(), lang });
    true
}

fn subject() -> String {
    format!("{}: words for the dictionaries", crate::brand::NAME)
}

fn heading() -> String {
    format!("{} {} (language, word):\n", crate::brand::NAME, env!("CARGO_PKG_VERSION"))
}

fn line(w: &SuggestedWord) -> String {
    format!("{}\t{}\n", w.lang, w.word)
}

/// The whole list as text, to paste into an email or the site's form: one `language<TAB>word`
/// line per word, after a heading line.
pub fn text(words: &[SuggestedWord]) -> String {
    let mut out = format!("To: {TO}\n{}", heading());
    for w in words {
        out.push_str(&line(w));
    }
    out
}

/// A `mailto:` link to [`TO`] with the first words that fit in [`MAX_MAILTO_BYTES`], and how many
/// it holds.
pub fn mailto(words: &[SuggestedWord]) -> (String, usize) {
    // Encoding is per byte, so the link grows by each line's own encoding.
    let mut link = format!("mailto:{TO}?subject={}&body={}", encode(&subject()), encode(&heading()));
    let mut fitted = 0;
    for w in words {
        let next = encode(&line(w));
        if link.len().saturating_add(next.len()) > MAX_MAILTO_BYTES {
            break;
        }
        link.push_str(&next);
        fitted += 1;
    }
    (link, fitted)
}

/// Percent-encode everything but RFC 3986's unreserved characters (UTF-8 bytes).
fn encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn state(s: &Session) -> Value {
    let list = &s.prefs.tinta_suggest;
    let (link, in_mail) = mailto(&list.words);
    json!({
        "value": list.enabled,
        "to": TO,
        "words": list.words.iter().map(|w| json!({"word": w.word, "lang": w.lang})).collect::<Vec<_>>(),
        "mailto": link,
        "inMail": in_mail,
        "text": text(&list.words),
    })
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("review.suggestWords", "Suggest My Words", "File › Options › Proofing", |s, v| {
            let on = match v.get("value") {
                None | Some(Value::Null) => !s.prefs.tinta_suggest.enabled,
                Some(Value::Bool(b)) => *b,
                Some(_) => return Err(CmdError::Params("`value` must be true or false".into())),
            };
            s.prefs.tinta_suggest.enabled = on;
            // Turning it off withdraws: the list is deleted.
            if !on {
                s.prefs.tinta_suggest.words.clear();
            }
            Ok(state(s))
        })
        .params(r#"{"value"?: bool (omit to toggle; off deletes the list)} → the list, a mailto link to send it and its text"#)
        .pure(),
        CommandSpec::new("review.suggestedWords", "Suggested Words", "File › Options › Proofing", |s, v| {
            if let Some(x) = v.get("remove").filter(|x| !x.is_null()) {
                let bad = || CmdError::Params("`remove` must be a list of words".into());
                let list = x.as_array().ok_or_else(bad)?;
                if list.len() > MAX_REMOVE_PER_CALL {
                    return Err(CmdError::Params(format!("`remove` takes at most {MAX_REMOVE_PER_CALL} words")));
                }
                let words = list.iter().map(|w| w.as_str().ok_or_else(bad)).collect::<Result<Vec<_>, _>>()?;
                for w in words {
                    forget(s, w);
                }
            }
            match v.get("clear") {
                None | Some(Value::Null) | Some(Value::Bool(false)) => {}
                Some(Value::Bool(true)) => s.prefs.tinta_suggest.words.clear(),
                Some(_) => return Err(CmdError::Params("`clear` must be true or false".into())),
            }
            Ok(state(s))
        })
        .params(r#"{"remove"?: [string], "clear"?: bool} → {"value", "words": [{"word", "lang"}], "mailto", "inMail": n, "text"}"#)
        .pure(),
    ]
}

#[cfg(test)]
mod tests;
