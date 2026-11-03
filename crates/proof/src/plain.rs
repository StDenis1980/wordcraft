//! Languages proofed by their Hunspell dictionary alone: Ukrainian, Catalan, Galician, Basque,
//! French, Romanian, Lithuanian and German.
//!
//! Each reads its installed `.aff`/`.dic` pair ([`crate::hunspell`]; nothing is bundled with the
//! code): Ukrainian `uk_UA` (dict_uk, MPL), Catalan `ca` and Valencian `ca-valencia` (Softcatalà,
//! GPL/LGPL), Galician `gl_ES`
//! (Proxecto Trasno, GPL), Basque `eu` (Xuxen, GPL), French `fr`, Romanian `ro_RO` and
//! Lithuanian `lt_LT` and German `de_DE` (each under its own licence, as the system or the pack
//! installs it; German's compound words come from the dictionary's compound flags). Suggestions use the dictionary's own
//! MAP/REP/TRY/KEY tables ([`crate::cs::suggest::suggest_in`]), and grammar is the generic
//! repeated-word and spacing checks, with messages in the language. Without a dictionary the
//! language has no spelling marks, like the other Hunspell languages. Hyphenation keeps the
//! English fallback, except German's own patterns ([`crate::hyphen`]).

use std::sync::{Arc, RwLock};

use crate::affix::AffixDict;
use crate::{Issue, IssueKind, ProofLang, words};

/// The script a language's dictionary covers: words with none of its letters (a Latin name in
/// Ukrainian text, Cyrillic in Catalan) are not checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Script {
    Latin,
    Cyrillic,
}

impl Script {
    fn has(self, c: char) -> bool {
        c.is_alphabetic()
            && match self {
                Script::Latin => c < '\u{0250}' || ('\u{1E00}'..='\u{1EFF}').contains(&c),
                Script::Cyrillic => {
                    ('\u{0400}'..='\u{052F}').contains(&c) || ('\u{1C80}'..='\u{1C8F}').contains(&c) || ('\u{A640}'..='\u{A69F}').contains(&c)
                }
            }
    }
}

/// One language of this module: its dictionary and its messages.
pub struct Lang {
    lang: ProofLang,
    /// Hunspell file names, the usual one first (it names the dictionary in File › Options and in
    /// load errors).
    names: &'static [&'static str],
    /// English name, for the log.
    english: &'static str,
    script: Script,
    misspelled: &'static str,
    /// `{w}` is the repeated word.
    repeated: &'static str,
    space_before: &'static str,
    double_space: &'static str,
    /// Whether a space before `;`, `:`, `!` and `?` is the language's typography (French), so not
    /// a stray space.
    spaced_punctuation: bool,
    /// Words the language repeats on purpose (French `nous nous levons`), never a repeated word.
    repeats: &'static [&'static str],
    /// `None` until first use, then the installed dictionary (or an empty one).
    dict: RwLock<Option<Arc<AffixDict>>>,
}

/// Ukrainian (`uk`, `uk-UA`).
pub static UK: Lang = Lang {
    lang: ProofLang::Uk,
    names: &["uk_UA", "uk-UA", "uk"],
    english: "Ukrainian",
    script: Script::Cyrillic,
    misspelled: "Можлива орфографічна помилка",
    repeated: "Повторене слово: «{w}»",
    space_before: "Пробіл перед розділовим знаком",
    spaced_punctuation: false,
    repeats: &[],
    double_space: "Подвійний пробіл",
    dict: RwLock::new(None),
};

/// Catalan (`ca`, `ca-ES`).
pub static CA: Lang = Lang {
    lang: ProofLang::Ca,
    names: &["ca", "ca_ES", "ca-ES"],
    english: "Catalan",
    script: Script::Latin,
    misspelled: "Possible error ortogràfic",
    repeated: "Paraula repetida: «{w}»",
    space_before: "Espai abans d'un signe de puntuació",
    spaced_punctuation: false,
    repeats: &[],
    double_space: "Doble espai",
    dict: RwLock::new(None),
};

/// Valencian (`ca-ES-valencia`): the `ca-valencia` dictionary, else the central Catalan one. The
/// messages are the same words in both.
pub static CA_VALENCIA: Lang = Lang {
    lang: ProofLang::CaValencia,
    names: &["ca-valencia", "ca_ES-valencia", "ca-ES-valencia", "ca_ES@valencia", "ca", "ca_ES", "ca-ES"],
    english: "Valencian",
    script: Script::Latin,
    misspelled: "Possible error ortogràfic",
    repeated: "Paraula repetida: «{w}»",
    space_before: "Espai abans d'un signe de puntuació",
    spaced_punctuation: false,
    repeats: &[],
    double_space: "Doble espai",
    dict: RwLock::new(None),
};

/// Galician (`gl`, `gl-ES`).
pub static GL: Lang = Lang {
    lang: ProofLang::Gl,
    names: &["gl_ES", "gl-ES", "gl"],
    english: "Galician",
    script: Script::Latin,
    misspelled: "Posible erro ortográfico",
    repeated: "Palabra repetida: «{w}»",
    space_before: "Espazo antes dun signo de puntuación",
    spaced_punctuation: false,
    repeats: &[],
    double_space: "Dobre espazo",
    dict: RwLock::new(None),
};

/// Basque (`eu`, `eu-ES`).
pub static EU: Lang = Lang {
    lang: ProofLang::Eu,
    names: &["eu", "eu_ES", "eu-ES"],
    english: "Basque",
    script: Script::Latin,
    misspelled: "Ortografia-akats posiblea",
    repeated: "Errepikatutako hitza: «{w}»",
    space_before: "Zuriunea puntuazio-ikurraren aurretik",
    spaced_punctuation: false,
    repeats: &[],
    double_space: "Zuriune bikoitza",
    dict: RwLock::new(None),
};

/// French (`fr`, `fr-FR`, `fr-BE`, `fr-CA`…). French writes a space before `;:!?`.
pub static FR: Lang = Lang {
    lang: ProofLang::Fr,
    names: &["fr", "fr_FR", "fr-FR", "fr-classique", "fr_BE", "fr_CA", "fr_CH"],
    english: "French",
    script: Script::Latin,
    misspelled: "Faute d'orthographe possible",
    repeated: "Mot répété : « {w} »",
    space_before: "Espace avant un signe de ponctuation",
    spaced_punctuation: true,
    repeats: &["nous", "vous"],
    double_space: "Double espace",
    dict: RwLock::new(None),
};

/// Romanian (`ro`, `ro-RO`, `ro-MD`).
pub static RO: Lang = Lang {
    lang: ProofLang::Ro,
    names: &["ro_RO", "ro-RO", "ro"],
    english: "Romanian",
    script: Script::Latin,
    misspelled: "Posibilă greșeală de ortografie",
    repeated: "Cuvânt repetat: „{w}”",
    space_before: "Spațiu înaintea unui semn de punctuație",
    spaced_punctuation: false,
    repeats: &[],
    double_space: "Spațiu dublu",
    dict: RwLock::new(None),
};

/// Lithuanian (`lt`, `lt-LT`).
pub static LT: Lang = Lang {
    lang: ProofLang::Lt,
    names: &["lt_LT", "lt-LT", "lt"],
    english: "Lithuanian",
    script: Script::Latin,
    misspelled: "Galima rašybos klaida",
    repeated: "Pakartotas žodis: „{w}“",
    space_before: "Tarpas prieš skyrybos ženklą",
    spaced_punctuation: false,
    repeats: &[],
    double_space: "Dvigubas tarpas",
    dict: RwLock::new(None),
};

/// German (`de`, `de-DE`, `de-AT`…): `de_DE` (in the pack, LibreOffice's `de_DE_frami`), else
/// another installed German dictionary of Germany or Austria.
pub static DE: Lang = Lang {
    lang: ProofLang::De,
    names: &["de_DE", "de-DE", "de_DE_frami", "de", "de_AT", "de-AT", "de_AT_frami"],
    english: "German",
    script: Script::Latin,
    misspelled: "Möglicher Rechtschreibfehler",
    repeated: "Wiederholtes Wort: „{w}“",
    space_before: "Leerzeichen vor einem Satzzeichen",
    spaced_punctuation: false,
    repeats: &[],
    double_space: "Doppeltes Leerzeichen",
    dict: RwLock::new(None),
};

/// Swiss German (`de-CH`, `de-LI`): `de_CH` (in the pack, LibreOffice's `de_CH_frami`, with `ss`
/// for `ß`), else the German of Germany. Swiss quotation marks are guillemets.
pub static DE_CH: Lang = Lang {
    lang: ProofLang::DeCh,
    names: &["de_CH", "de-CH", "de_CH_frami", "de_DE", "de-DE", "de_DE_frami", "de"],
    english: "Swiss German",
    script: Script::Latin,
    misspelled: "Möglicher Rechtschreibfehler",
    repeated: "Wiederholtes Wort: «{w}»",
    space_before: "Leerzeichen vor einem Satzzeichen",
    spaced_punctuation: false,
    repeats: &[],
    double_space: "Doppeltes Leerzeichen",
    dict: RwLock::new(None),
};

/// Every language of this module.
pub const ALL: [&Lang; 10] = [&UK, &CA, &CA_VALENCIA, &GL, &EU, &FR, &RO, &LT, &DE, &DE_CH];

/// The module's entry for `lang`, if it is one of its languages.
pub fn of(lang: ProofLang) -> Option<&'static Lang> {
    ALL.into_iter().find(|l| l.lang == lang)
}

impl Lang {
    /// The proofing language.
    pub fn lang(&self) -> ProofLang {
        self.lang
    }

    /// The usual Hunspell name of the dictionary (`uk_UA`).
    pub fn dictionary_name(&self) -> &'static str {
        self.names.first().copied().unwrap_or("")
    }

    /// The dictionary: the installed one (read on first use), one given with
    /// [`Lang::install_dictionary`], or an empty one when none is available.
    pub fn dictionary(&self) -> Arc<AffixDict> {
        if let Some(d) = self.dict.read().unwrap_or_else(std::sync::PoisonError::into_inner).as_ref() {
            return d.clone();
        }
        let loaded = Arc::new(self.load_installed());
        let mut slot = self.dict.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        // Another thread (or an install) may have filled it meanwhile: that one wins.
        slot.get_or_insert(loaded).clone()
    }

    fn load_installed(&self) -> AffixDict {
        let key = self.dictionary_name();
        let Some((aff, dic)) = crate::hunspell::find(self.names) else {
            crate::hunspell::clear_load_error(key);
            log::info!("{} spelling: no Hunspell {key} dictionary installed", self.english);
            return AffixDict::default();
        };
        crate::hunspell::note_load(key, &dic, crate::hunspell::load(&aff, &dic)).unwrap_or_else(|e| {
            log::warn!("{} spelling: {e}", self.english);
            AffixDict::default()
        })
    }

    /// Use this dictionary from now on (Hunspell `.aff` and `.dic` text, decoded to UTF-8). On
    /// error the current dictionary stays.
    pub fn install_dictionary(&self, aff: &str, dic: &str) -> Result<(), String> {
        let d = AffixDict::from_hunspell(aff, dic, &[])?;
        *self.dict.write().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::new(d));
        Ok(())
    }

    /// Forget the loaded dictionary, so the next check looks for an installed one again.
    pub fn reload_dictionary(&self) {
        *self.dict.write().unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }

    /// The `.dic` file found in the dictionaries folder or the system locations now; `None` when
    /// there is none. A dictionary given with [`Lang::install_dictionary`] isn't reported.
    pub fn installed_path(&self) -> Option<std::path::PathBuf> {
        crate::hunspell::find(self.names).map(|(_, dic)| dic)
    }

    /// Is a dictionary available (spelling is checked)?
    pub fn has_dictionary(&self) -> bool {
        !self.dictionary().is_empty()
    }

    /// Is `word` spelled correctly? Numbers, single letters, short words in capitals (with
    /// Ignore words in UPPERCASE), words with digits (with Ignore words that contain numbers) and
    /// words without a letter of the language's script are accepted, and so is every word when
    /// no dictionary is installed. A hyphenated word is correct when the dictionary has it whole
    /// (`будь-який`, `dóna-li`) or has each part.
    pub fn is_correct_with_options(&self, word: &str, opts: &crate::Options) -> bool {
        self.check(&self.dictionary(), word, opts)
    }

    fn check(&self, d: &AffixDict, word: &str, opts: &crate::Options) -> bool {
        if d.is_empty() {
            return true;
        }
        let w = word.trim_matches(|c: char| !c.is_alphanumeric() && !is_apostrophe(c));
        let w = w.trim_matches(is_apostrophe);
        let n = w.chars().count();
        if n <= 1 || (opts.ignore_numbers && w.chars().any(|c| c.is_ascii_digit())) || !w.chars().any(char::is_alphabetic) {
            return true;
        }
        let acronym = |p: &str| opts.ignore_uppercase && p.chars().all(|c| !c.is_lowercase()) && p.chars().count() <= 6;
        if acronym(w) || !w.chars().any(|c| self.script.has(c)) {
            return true;
        }
        known(d, w) || (w.contains('-') && w.split('-').all(|p| p.chars().count() <= 1 || acronym(p) || known(d, p)))
    }

    /// Spelling suggestions, best first, at most `max`, in the word's case.
    pub fn suggest(&self, word: &str, max: usize) -> Vec<String> {
        let d = self.dictionary();
        // A form made to join a hyphenated compound (German `-straße`) is no suggestion alone.
        let edge = |s: &String| s.starts_with('-') || s.ends_with('-');
        crate::cs::suggest::suggest_in(&d, word, max.saturating_add(2)).iter().filter(|s| !edge(s)).take(max).map(|s| d.output(s)).collect()
    }

    /// The message of a spelling mark.
    pub fn misspelled(&self) -> &'static str {
        self.misspelled
    }

    /// Grammar issues: a word written twice, a space before punctuation, two spaces.
    pub fn grammar(&self, text: &str) -> Vec<Issue> {
        let issue = |start: usize, end: usize, message: String, suggestions: Vec<String>| Issue {
            start,
            end,
            kind: IssueKind::Grammar,
            message,
            suggestions,
            lang: self.lang,
        };
        let mut v = Vec::new();
        for pair in words(text).windows(2) {
            let [(a0, b0), (a1, b1)] = pair else { continue };
            let (Some(w0), Some(w1), Some(gap)) = (text.get(*a0..*b0), text.get(*a1..*b1), text.get(*b0..*a1)) else { continue };
            if !gap.is_empty()
                && gap.chars().all(char::is_whitespace)
                && w0.chars().all(char::is_alphabetic)
                && w0.to_lowercase() == w1.to_lowercase()
                && !self.repeats.contains(&w1.to_lowercase().as_str())
            {
                v.push(issue(*a0, *b1, self.repeated.replace("{w}", w1), vec![w0.to_string()]));
            }
        }
        for (i, c) in text.char_indices() {
            if c != ' ' {
                continue;
            }
            let mut it = text.get(i + 1..).unwrap_or("").chars();
            let Some(p) = it.next() else { continue };
            let after = it.next();
            // A ratio or a score keeps its spaces: digit, space, colon, space, digit.
            let ratio = p == ':'
                && text.get(..i).and_then(|before| before.chars().next_back()).is_some_and(|c| c.is_ascii_digit())
                && after == Some(' ')
                && it.next().is_some_and(|c| c.is_ascii_digit());
            let flag = !ratio
                && match p {
                    // French spaces these (`Bonjour !`, `Note : …`).
                    ';' | ':' | '!' | '?' if self.spaced_punctuation => false,
                    ',' => true,
                    // `;)` and `;-)` are emoticons.
                    ';' => !matches!(after, Some(')' | '(' | '-' | 'P' | 'D')),
                    // ` ...` is an ellipsis, ` .5` a number.
                    '.' => after != Some('.') && !after.is_some_and(|a| a.is_ascii_digit()),
                    // ` :)` and ` :D` are emoticons; ` ?!` is still a stray space.
                    ':' | '!' | '?' => after.is_none_or(|a| a.is_whitespace() || matches!(a, '"' | '»' | '”' | '’' | '!' | '?')),
                    _ => false,
                };
            if flag {
                v.push(issue(i, i + 1 + p.len_utf8(), self.space_before.into(), vec![p.to_string()]));
            }
        }
        for (i, _) in text.match_indices("  ") {
            if i > 0 && text.get(..i).is_some_and(|t| t.ends_with(' ')) {
                continue;
            }
            v.push(issue(i, i + 2, self.double_space.into(), vec![" ".into()]));
        }
        v.sort_by_key(|i| (i.start, i.end));
        v
    }
}

/// The apostrophes words are written with: `'`, `’` and the Ukrainian `ʼ`.
fn is_apostrophe(c: char) -> bool {
    matches!(c, '\'' | '’' | 'ʼ')
}

/// Is one word in the dictionary or the user dictionary?
fn known(d: &AffixDict, w: &str) -> bool {
    d.check(w) || crate::in_user_dictionary(&w.to_lowercase())
}

#[cfg(test)]
pub(crate) mod test_dict {
    //! Hand-written synthetic test data, one small dictionary per language, using each real
    //! dictionary's flag notation and input tables. No words, rules or tables were copied from a
    //! published dictionary.

    /// Ukrainian: stress mark ignored, apostrophes converted, Latin letters made unknown.
    pub const UK_AFF: &str = "SET UTF-8\nIGNORE \u{301}\nICONV 4\nICONV ʼ '\nICONV ’ '\nICONV a 0\nICONV o 0\nTRY оаеиінтрсвлкмдпу'\nMAP 1\nMAP иі\nREP 1\nREP и і\nSFX A Y 2\nSFX A 0 у [^о]\nSFX A о а о\nSFX B Y 1\nSFX B ти ла ти\n";
    pub const UK_DIC: &str = "8\nрахунок\nрахунку\nм'ясо/A\nдоговір/A\nзаплатити/B\nУкраїна\nбудь-який\nкнига\n";

    /// Catalan: two-character flags, a forbidden word, apostrophes converted.
    pub const CA_AFF: &str = "SET UTF-8\nFLAG long\nFORBIDDENWORD ZZ\nICONV 1\nICONV ’ '\nTRY esairtocnlumdpgv\nSFX Fa Y 1\nSFX Fa 0 es [^a]\nSFX Fa a es a\nPFX El Y 1\nPFX El 0 l' [aeiouh]\n";
    pub const CA_DIC: &str = "7\ncasa/Fa\nhome/El\nfactura/Fa\ncol·legi\ndóna-li\npressupost\nhomes/ZZ\n";

    /// Valencian: Catalan's notation, with Valencian forms and accents (`este`, `vore`, `café`)
    /// the central test dictionary doesn't have.
    pub const CA_VALENCIA_AFF: &str = "SET UTF-8\nFLAG long\nICONV 1\nICONV ’ '\nTRY esairtocnlumdpgv\nSFX Fa Y 1\nSFX Fa 0 es [^a]\nSFX Fa a es a\n";
    pub const CA_VALENCIA_DIC: &str = "6\ncasa/Fa\nfactura/Fa\neste\nvore\ncafé\nanglés\n";

    /// Galician: numeric flags, a NEEDAFFIX stem.
    pub const GL_AFF: &str = "SET UTF-8\nFLAG num\nNEEDAFFIX 666\nTRY aeosrnidlctu\nSFX 10 Y 1\nSFX 10 0 s .\nSFX 20 Y 1\nSFX 20 0 mos .\n";
    pub const GL_DIC: &str = "4\ncasa/10\nfactura/10\norzamento/10\ncanta/20,666\n";

    /// Basque: numeric flags with suffix chains.
    pub const EU_AFF: &str = "SET UTF-8\nFLAG num\nTRY aeiournktzs\nSFX 1 Y 2\nSFX 1 0 a/2 [^a]\nSFX 1 0 k [a]\nSFX 2 Y 1\nSFX 2 0 k .\n";
    pub const EU_DIC: &str = "3\netxe/1\nfaktura/1\nkaixo\n";

    /// French: a plural suffix and an elided article prefix, apostrophes converted.
    pub const FR_AFF: &str = "SET UTF-8\nICONV 1\nICONV ’ '\nTRY esaitnrulodcpmévqfbghjàxèyêzçôùâûîœ\nSFX S Y 1\nSFX S 0 s .\n";
    pub const FR_DIC: &str = "4\nfacture/S\nmaison/S\nété\nnous\n";

    /// Romanian: comma-below letters and a feminine plural.
    pub const RO_AFF: &str = "SET UTF-8\nTRY aeirtnulcsoămpdîvșțgbfzhâjxk\nSFX A Y 1\nSFX A ă i ă\n";
    pub const RO_DIC: &str = "3\nfactură/A\ncasă/A\nțară\n";

    /// Lithuanian: a masculine plural.
    pub const LT_AFF: &str = "SET UTF-8\nTRY iasoetnukrlmvdpjgšėžybųčcįzūęą\nSFX A Y 1\nSFX A as ai as\n";
    pub const LT_DIC: &str = "3\nnamas/A\nsąskaita\nprašau\n";

    /// German: compound parts by flags (`Haus` may begin a compound through a suffix's
    /// continuation classes, lower-case `tür` only ends one), `ß` written `SS` in capitals, and a
    /// homonym listed once for compounds only and once with NEEDAFFIX.
    pub const DE_AFF: &str = "SET UTF-8\nTRY esianrtolcdugmphbßäöü\nCHECKSHARPS\nCOMPOUNDBEGIN x\nCOMPOUNDMIDDLE y\nCOMPOUNDEND z\nCOMPOUNDPERMITFLAG c\nONLYINCOMPOUND o\nNEEDAFFIX h\nCOMPOUNDMIN 2\nSFX E Y 1\nSFX E 0 en .\nSFX N Y 1\nSFX N 0 n .\nSFX j Y 2\nSFX j 0 0/xoc .\nSFX j 0 s/xoc .\nPFX k Y 1\nPFX k 0 0/coy .\n";
    pub const DE_DIC: &str = "8\nHaus/Ej\nTür/E\ntür/Eoz\ntür/hk\nRechnung/j\nnummer/oz\nStraße/N\nschön\n";
    /// Swiss German: the same rules, with `ss` where Germany writes `ß`.
    pub const DE_CH_DIC: &str = "8\nHaus/Ej\nTür/E\ntür/Eoz\ntür/hk\nRechnung/j\nnummer/oz\nStrasse/N\nFussball\n";

    /// Install them all.
    pub fn install() {
        for (l, aff, dic) in [
            (&super::UK, UK_AFF, UK_DIC),
            (&super::CA, CA_AFF, CA_DIC),
            (&super::CA_VALENCIA, CA_VALENCIA_AFF, CA_VALENCIA_DIC),
            (&super::GL, GL_AFF, GL_DIC),
            (&super::EU, EU_AFF, EU_DIC),
            (&super::FR, FR_AFF, FR_DIC),
            (&super::RO, RO_AFF, RO_DIC),
            (&super::LT, LT_AFF, LT_DIC),
            (&super::DE, DE_AFF, DE_DIC),
            (&super::DE_CH, DE_AFF, DE_CH_DIC),
        ] {
            if let Err(e) = l.install_dictionary(aff, dic) {
                panic!("{}: {e}", l.english);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(l: &Lang, w: &str) -> bool {
        l.is_correct_with_options(w, &crate::Options::default())
    }

    #[test]
    fn ukrainian_apostrophes_stress_and_hyphens() {
        test_dict::install();
        for w in [
            "рахунок",
            "м'ясо",
            "м’ясо",
            "мʼясо",
            "м'яса",
            "Рахунок",
            "РАХУНОК",
            "заплатити",
            "заплатила",
            "запла\u{301}тити",
            "будь-який",
            "Україна",
            "Kyiv",
            "ПДВ",
        ] {
            assert!(ok(&UK, w), "{w}");
        }
        for w in ["рахунак", "мясо", "україна", "заплатитти", "будь-якиі", "книгa"] {
            assert!(!ok(&UK, w), "{w}");
        }
    }

    #[test]
    fn catalan_elision_middle_dot_and_forbidden_words() {
        test_dict::install();
        for w in ["casa", "cases", "factures", "l'home", "l’home", "col·legi", "dóna-li", "Pressupost"] {
            assert!(ok(&CA, w), "{w}");
        }
        for w in ["kasa", "casaes", "l'casa", "homes", "colegi"] {
            assert!(!ok(&CA, w), "{w}");
        }
    }

    /// Tinta Office: Valencian text is checked with the Valencian dictionary, so its own forms
    /// aren't marked, and Catalan text keeps the central one.
    #[test]
    fn valencian_has_its_own_dictionary() {
        test_dict::install();
        for w in ["este", "vore", "café", "anglés", "factures"] {
            assert!(ok(&CA_VALENCIA, w), "{w}");
            assert!(crate::is_correct_in(w, ProofLang::CaValencia), "{w}");
        }
        for w in ["este", "café"] {
            assert!(!ok(&CA, w), "{w}");
            assert!(!crate::is_correct_in(w, ProofLang::Ca), "{w}");
        }
        assert!(!ok(&CA_VALENCIA, "cafè"), "only its own words");
        let text = "Este café, factura.";
        assert!(crate::check_text(text, &[(0..text.len(), ProofLang::from_tag(Some("ca-ES-valencia")))]).is_empty());
        let marked = crate::check_text(text, &[(0..text.len(), ProofLang::from_tag(Some("ca-ES")))]);
        assert_eq!(marked.iter().filter_map(|i| text.get(i.start..i.end)).collect::<Vec<_>>(), ["Este", "café"]);
        assert_eq!(crate::check_grammar_in("la la casa", ProofLang::CaValencia).first().map(|i| i.lang), Some(ProofLang::CaValencia));
    }

    /// German: compounds from parts the dictionary marks, `ß` as `SS` in capitals, and compound
    /// parts that are no words alone.
    #[test]
    fn german_compounds_and_sharp_s() {
        test_dict::install();
        for w in [
            "Haus",
            "Tür",
            "Türen",
            "Haustür",
            "Haustüren",
            "Rechnungsnummer",
            "Haustürnummer",
            "Straße",
            "Straßen",
            "STRASSE",
            "STRASSEN",
            "SCHÖN",
            "schön",
        ] {
            assert!(ok(&DE, w), "{w}");
            assert!(crate::is_correct_in(w, ProofLang::De), "{w}");
        }
        // `tür` and `nummer` only end compounds; a part must be marked for its place; `ss` is `ß`
        // only in capitals.
        for w in ["tür", "türen", "nummer", "Nummer", "Strasse", "strasse", "Türhaus", "HausHaus", "Rechnungsnumer", "Hausnummerr"] {
            assert!(!ok(&DE, w), "{w}");
        }
        assert_eq!(crate::check_grammar_in("das das Haus", ProofLang::De).first().map(|i| i.message.as_str()), Some("Wiederholtes Wort: „das“"));
        assert!(crate::suggest_in("Hauss", 4, ProofLang::De).iter().any(|s| s == "Haus"), "{:?}", crate::suggest_in("Hauss", 4, ProofLang::De));
    }

    /// Switzerland writes `ss` for `ß`, compounds the same way, and quotes with guillemets.
    #[test]
    fn swiss_german_writes_ss() {
        test_dict::install();
        for w in ["Strasse", "Strassen", "STRASSE", "Fussball", "Haustüren", "Rechnungsnummer"] {
            assert!(crate::is_correct_in(w, ProofLang::DeCh), "{w}");
        }
        for w in ["Straße", "Fußball", "tür"] {
            assert!(!crate::is_correct_in(w, ProofLang::DeCh), "{w}");
        }
        assert!(!crate::is_correct_in("Strasse", ProofLang::De) && crate::is_correct_in("Straße", ProofLang::De), "Germany keeps `ß`");
        assert_eq!(ProofLang::from_tag(Some("de-CH")), ProofLang::DeCh);
        assert!(
            crate::suggest_in("Strase", 4, ProofLang::DeCh).iter().any(|s| s == "Strasse"),
            "{:?}",
            crate::suggest_in("Strase", 4, ProofLang::DeCh)
        );
        let issues = crate::check_grammar_in("das das Haus", ProofLang::DeCh);
        assert_eq!(issues.first().map(|i| (i.message.as_str(), i.lang)), Some(("Wiederholtes Wort: «das»", ProofLang::DeCh)));
    }

    #[test]
    fn galician_and_basque_numeric_flags() {
        test_dict::install();
        assert!(ok(&GL, "casas") && ok(&GL, "orzamentos") && ok(&GL, "cantamos"));
        assert!(!ok(&GL, "canta"), "a NEEDAFFIX stem is no word alone");
        assert!(!ok(&GL, "kasa"));
        assert!(ok(&EU, "etxea") && ok(&EU, "etxeak") && ok(&EU, "fakturak") && ok(&EU, "kaixo"));
        assert!(!ok(&EU, "etxxea") && !ok(&EU, "etxeakk"));
    }

    #[test]
    fn dispatch_messages_suggestions_and_options() {
        test_dict::install();
        for (lang, tag, bad, want, message) in [
            (ProofLang::Uk, "uk-UA", "рахунак", "рахунок", "Можлива орфографічна помилка"),
            (ProofLang::Ca, "ca-ES", "factuar", "factura", "Possible error ortogràfic"),
            (ProofLang::CaValencia, "ca-ES-valencia", "factuar", "factura", "Possible error ortogràfic"),
            (ProofLang::Gl, "gl", "facturra", "factura", "Posible erro ortográfico"),
            (ProofLang::Eu, "eus", "faktuar", "faktura", "Ortografia-akats posiblea"),
            (ProofLang::Fr, "fr-FR", "factre", "facture", "Faute d'orthographe possible"),
            (ProofLang::Ro, "ro-RO", "factur", "factură", "Posibilă greșeală de ortografie"),
            (ProofLang::Lt, "lt-LT", "sąskata", "sąskaita", "Galima rašybos klaida"),
        ] {
            assert_eq!(ProofLang::from_tag(Some(tag)), lang, "{tag}");
            assert!(lang.is_proofed() && crate::has_dictionary(Some(tag)), "{tag}");
            let text = format!("{want} {bad}");
            let issues = crate::check_text(&text, &[(0..text.len(), lang)]);
            assert_eq!(issues.len(), 1, "{tag}: {issues:?}");
            assert_eq!(text.get(issues[0].start..issues[0].end), Some(bad));
            assert_eq!(issues[0].message, message);
            assert_eq!(issues[0].lang, lang);
            assert!(crate::suggest_in(bad, 8, lang).contains(&want.to_string()), "{tag}: {:?}", crate::suggest_in(bad, 8, lang));
            assert!(crate::missing_dictionary(Some(tag)).is_none());
        }
        let caps = crate::Options { ignore_uppercase: false, ..Default::default() };
        assert!(crate::is_correct_in_with("РАХУНАК", ProofLang::Uk, &crate::Options::default()), "short capitals are acronyms by default");
        assert!(!crate::is_correct_in_with("РАХУНАК", ProofLang::Uk, &caps));
        let digits = crate::Options { ignore_numbers: false, ..Default::default() };
        assert!(!crate::is_correct_in_with("casa2", ProofLang::Ca, &digits) && crate::is_correct_in("casa2", ProofLang::Ca));
    }

    #[test]
    fn grammar_messages_are_in_the_language() {
        let text = "Це це рахунок , так.  Кінець";
        let issues = crate::check_grammar_in(text, ProofLang::Uk);
        let messages: Vec<&str> = issues.iter().map(|i| i.message.as_str()).collect();
        assert_eq!(messages, ["Повторене слово: «це»", "Пробіл перед розділовим знаком", "Подвійний пробіл"]);
        assert!(issues.iter().all(|i| i.lang == ProofLang::Uk));
        let issues = crate::check_grammar_in("la la casa", ProofLang::Ca);
        assert_eq!(issues.first().map(|i| i.message.as_str()), Some("Paraula repetida: «la»"));
        assert!(crate::check_grammar_in("Proba 3 : 1 :)", ProofLang::Gl).is_empty(), "scores and emoticons keep their spaces");
        assert_eq!(crate::check_grammar_in("etxea etxea", ProofLang::Eu).len(), 1);
        // French typography spaces `;:!?`, and `nous nous` is a reflexive verb, not a repeat.
        assert!(crate::check_grammar_in("Bonjour ! Note : nous nous levons ; oui ?", ProofLang::Fr).is_empty());
        let issues = crate::check_grammar_in("la la maison , oui", ProofLang::Fr);
        let messages: Vec<&str> = issues.iter().map(|i| i.message.as_str()).collect();
        assert_eq!(messages, ["Mot répété : « la »", "Espace avant un signe de ponctuation"]);
        assert_eq!(crate::check_grammar_in("Bună ziua !", ProofLang::Ro).len(), 1, "Romanian has no space before `!`");
        assert_eq!(crate::check_grammar_in("Labas  rytas", ProofLang::Lt).first().map(|i| i.message.as_str()), Some("Dvigubas tarpas"));
    }

    #[test]
    fn missing_dictionaries_are_quiet() {
        let empty = AffixDict::default();
        assert!(UK.check(&empty, "рахунак", &crate::Options::default()));
        let d = AffixDict::from_hunspell(test_dict::UK_AFF, test_dict::UK_DIC, &[]).unwrap();
        for w in ["", "а", "2026", "-", "''", "日本語", "ʼʼ"] {
            assert!(UK.check(&d, w, &crate::Options::default()), "{w:?}");
        }
        assert!(EU.install_dictionary("COMPOUNDFLAG X\n", "1\netxe\n").is_err());
    }

    #[test]
    fn dictionary_files_are_found_under_each_name() {
        let dir = std::env::temp_dir().join(format!("wordcraft-plain-names-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for l in ALL {
            for name in l.names {
                std::fs::write(dir.join(format!("{name}.aff")), "SET UTF-8\n").unwrap();
                std::fs::write(dir.join(format!("{name}.dic")), "1\nx\n").unwrap();
                let (aff, dic) = crate::hunspell::find_in(&dir, l.names).unwrap();
                assert!(crate::hunspell::load(&aff, &dic).is_ok());
                std::fs::remove_file(aff).unwrap();
                std::fs::remove_file(dic).unwrap();
            }
        }
        std::fs::remove_dir(dir).unwrap();
    }
}
