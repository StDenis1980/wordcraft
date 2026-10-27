//! Hunspell-style affix dictionaries — the subset the SJP.PL Polish dictionary (`pl_PL`) uses.
//!
//! A word is correct when it is a stem, a stem with one suffix, a stem with one prefix, or a stem
//! with a prefix and a suffix that both allow cross products; each rule applies only to stems
//! carrying its flag and ending (suffixes) or starting (prefixes) as its condition says. Highly
//! inflected languages store millions of forms this way in a few hundred thousand stems: the
//! Polish dictionary's ~350k stems and ~7,500 rules accept ~4.5 million forms.
//!
//! The rules and stems come from a Hunspell `.aff`/`.dic` pair ([`AffixDict::from_hunspell`]),
//! read at runtime from the dictionaries installed on the system ([`crate::hunspell`]). They can
//! also be stored in WordCraft's compact format ([`AffixDict::to_bytes`]): the rules as text
//! lines and the stems front-coded, raw-deflated together (the Polish dictionary is under 1 MB).
//!
//! Supported: flags of one character (any Unicode character — `FLAG UTF-8`), two characters
//! (`FLAG long`, Catalan) or a number (`FLAG num`, Galician, Basque), at most [`MAX_FLAGS`] distinct; `PFX`/`SFX` with strip,
//! append and condition (`.`, letters, `[abc]`, `[^abc]`), cross products, continuation classes
//! on suffixes (at most two suffixes in a row, as in Hunspell), `FULLSTRIP`, `FORBIDDENWORD`,
//! `NOSUGGEST`, the input tables `IGNORE` and `ICONV` (Ukrainian ignores the stress mark and
//! reads `’` as `'`), `NEEDAFFIX`, `KEEPCASE`, and the suggestion tables `TRY`, `KEY`, `MAP` and
//! `REP`. `COMPOUNDRULE` is read but builds no compounds (Galician's numerals stay unknown), and
//! `ONLYINCOMPOUND` stems are never words. Rejected: the other compounding directives
//! (`COMPOUNDFLAG`…) and the rest of those that change which words are correct.
//!
//! Lookups never panic: stems sit in one string, found through an open-addressing hash table, and
//! a probe allocates only small scratch strings for the candidate roots it tries.

use std::collections::HashMap;
use std::hash::Hasher;

use crate::patterns::{FxHasher, FxMap};

const MAGIC: &str = "WCAFF1";
/// Longest word (in bytes) looked up; longer input is never a dictionary word.
pub const MAX_WORD_BYTES: usize = 120;
/// Empty slot in the stem hash table.
const EMPTY: u32 = u32::MAX;
/// Most distinct flags a dictionary may use (Basque's Xuxen uses about 450).
pub const MAX_FLAGS: usize = Mask::BITS;

/// A set of flags, one bit each (bit `i` is the `i`-th distinct flag of the dictionary).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
struct Mask([u64; 8]);

impl Mask {
    const BITS: usize = 8 * 64;

    /// The set holding flag `i` alone; empty when `i` is out of range.
    fn bit(i: usize) -> Mask {
        let mut m = Mask::default();
        if let Some(w) = m.0.get_mut(i / 64) {
            *w = 1u64 << (i % 64);
        }
        m
    }

    fn is_empty(self) -> bool {
        self.0.iter().all(|&w| w == 0)
    }

    fn has(self, i: usize) -> bool {
        self.0.get(i / 64).is_some_and(|&w| w & (1u64 << (i % 64)) != 0)
    }

    /// Do the two sets share a flag?
    fn intersects(self, o: Mask) -> bool {
        self.0.iter().zip(o.0).any(|(a, b)| a & b != 0)
    }

    /// Does this set hold every flag of `o`?
    fn contains_all(self, o: Mask) -> bool {
        self.0.iter().zip(o.0).all(|(a, b)| a & b == b)
    }

    /// The flags of both sets.
    fn and(self, o: Mask) -> Mask {
        let mut m = self;
        m.0.iter_mut().zip(o.0).for_each(|(a, b)| *a &= b);
        m
    }

    /// The flags of this set that `o` doesn't hold.
    fn minus(self, o: Mask) -> Mask {
        let mut m = self;
        m.0.iter_mut().zip(o.0).for_each(|(a, b)| *a &= !b);
        m
    }

    /// The lowest flag in the set.
    fn first(self) -> Option<usize> {
        self.0.iter().enumerate().find(|(_, w)| **w != 0).map(|(k, w)| k * 64 + w.trailing_zeros() as usize)
    }
}

impl std::ops::BitOr for Mask {
    type Output = Mask;
    fn bitor(mut self, o: Mask) -> Mask {
        for (a, b) in self.0.iter_mut().zip(o.0) {
            *a |= b;
        }
        self
    }
}

impl std::ops::BitOrAssign for Mask {
    fn bitor_assign(&mut self, o: Mask) {
        *self = *self | o;
    }
}

/// Directives that change which words are correct in ways this engine doesn't implement.
const UNSUPPORTED: &[&str] = &[
    "AF",
    "AM",
    "CHECKSHARPS",
    "CIRCUMFIX",
    "COMPLEXPREFIXES",
    "COMPOUNDBEGIN",
    "COMPOUNDEND",
    "COMPOUNDFLAG",
    "COMPOUNDMIDDLE",
    "COMPOUNDPERMITFLAG",
    "LEMMA_PRESENT",
    "OCONV",
    "PSEUDOROOT",
    "SUBSTANDARD",
];

/// One character class of a rule's condition.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Cond {
    Any,
    Char(char),
    Set { negated: bool, chars: Box<[char]> },
}

impl Cond {
    fn matches(&self, c: char) -> bool {
        match self {
            Cond::Any => true,
            Cond::Char(x) => *x == c,
            Cond::Set { negated, chars } => chars.contains(&c) != *negated,
        }
    }
}

/// Parse a Hunspell condition (`.`, `[^i]e`, `[km]anie`…).
fn parse_cond(text: &str) -> Result<Box<[Cond]>, String> {
    if text == "." {
        return Ok(Box::new([]));
    }
    let mut out = Vec::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        match c {
            '.' => out.push(Cond::Any),
            '[' => {
                let mut set = Vec::new();
                let mut negated = false;
                let mut closed = false;
                let mut first = true;
                for d in chars.by_ref() {
                    if d == ']' {
                        closed = true;
                        break;
                    }
                    if first && d == '^' {
                        negated = true;
                    } else {
                        set.push(d);
                    }
                    first = false;
                }
                if !closed {
                    return Err(format!("unclosed `[` in condition `{text}`"));
                }
                out.push(Cond::Set { negated, chars: set.into_boxed_slice() });
            }
            ']' => return Err(format!("stray `]` in condition `{text}`")),
            c => out.push(Cond::Char(c)),
        }
    }
    Ok(out.into_boxed_slice())
}

/// How many suffixes a word may carry in a row through continuation classes (`work` +
/// `able/B` + `s` → `workables` is two). Hunspell strips at most two suffixes, so a third
/// chained suffix is not a word; the bound also keeps hostile input from walking deep recursions.
const MAX_CHAIN: u8 = 2;
/// How many suffixes may sit under the outermost one: the depth `form_has_flag` walks.
const INNER_CHAIN: u8 = MAX_CHAIN - 1;
/// Most forms [`AffixDict::forms`] lists for one stem (real stems have a few hundred at most).
pub const MAX_FORMS: usize = 4096;
/// Most rules [`AffixDict::forms`] tries along continuation chains and prefix–suffix cross
/// products for one stem: real dictionaries need a few thousand, hostile ones are cut off.
const MAX_FORMS_WORK: usize = 1 << 18;

/// One prefix or suffix rule.
#[derive(Clone, Debug)]
struct Rule {
    /// Bit of the rule's flag in the stems' flag masks.
    bit: Mask,
    flag: char,
    cross: bool,
    strip: Box<str>,
    add: Box<str>,
    /// Continuation classes (`SFX A 0 able/B`): flags the derived form carries, licensing
    /// further affixes. Chained application is bounded by [`MAX_CHAIN`].
    cont: Mask,
    cond: Box<[Cond]>,
    cond_text: Box<str>,
}

impl Rule {
    /// Does `root` end (suffix rule) or start (prefix rule) as the condition says?
    fn cond_matches(&self, root: &str, suffix: bool) -> bool {
        if self.cond.is_empty() {
            return true;
        }
        if suffix {
            let mut it = root.chars().rev();
            self.cond.iter().rev().all(|c| it.next().is_some_and(|x| c.matches(x)))
        } else {
            let mut it = root.chars();
            self.cond.iter().all(|c| it.next().is_some_and(|x| c.matches(x)))
        }
    }
}

/// Suffix rules sharing an appended string and a stripped one: one stem lookup serves them all.
#[derive(Clone, Debug, Default)]
struct SfxGroup {
    strip: Box<str>,
    rules: Vec<u32>,
}

/// A stem list with affix rules (see the module docs).
#[derive(Clone, Debug, Default)]
pub struct AffixDict {
    /// All stems, concatenated, sorted by bytes.
    words: String,
    offs: Vec<u32>,
    /// Per stem: index into `sets`.
    flags: Vec<u32>,
    /// Distinct flag masks.
    sets: Vec<Mask>,
    /// The `FORBIDDENWORD` flag's bit; stems carrying it are not words (0: none declared).
    forbidden: Mask,
    /// Stems with NOSUGGEST are correct, but none of their forms are suggested.
    no_suggest: Mask,
    /// `NEEDAFFIX`: stems carrying it aren't words on their own, nor are forms whose affix
    /// carries it as a continuation class.
    need_affix: Mask,
    /// `KEEPCASE`: stems carrying it match only in their own case.
    keep_case: Mask,
    /// `ONLYINCOMPOUND`: excluded like forbidden stems (no compounding here).
    only_in_compound: Mask,
    /// Open-addressing hash table of stem indices (`EMPTY` = free).
    table: Vec<u32>,
    /// Flag char of each bit.
    flag_chars: Vec<char>,
    prefixes: Vec<Rule>,
    suffixes: Vec<Rule>,
    /// Appended string → groups of suffix rules by stripped string.
    sfx_index: FxMap<Box<str>, Vec<SfxGroup>>,
    /// Per flag bit: the suffix rules of that flag, in rule order.
    sfx_by_flag: Vec<Vec<u32>>,
    /// Longest appended suffix, in chars.
    max_sfx_chars: usize,
    try_chars: String,
    key: String,
    map: Vec<Vec<char>>,
    rep: Vec<(String, String)>,
    /// `IGNORE`: characters taken out of a word before it is looked up.
    ignore: Vec<char>,
    /// `ICONV`: replacements made in a word before it is looked up.
    iconv: Vec<(String, String)>,
    /// `FULLSTRIP`: a rule may strip a whole stem, so a form may be all affix.
    fullstrip: bool,
    /// Provenance and licence lines, kept through [`AffixDict::to_bytes`].
    comments: Vec<String>,
}

fn hash_str(s: &str) -> u64 {
    let mut h = FxHasher::default();
    h.write(s.as_bytes());
    h.finish()
}

/// Letter case of a word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Casing {
    /// No capitals (`kot`).
    Lower,
    /// First letter capital, no other (`Kot`).
    Initial,
    /// Every cased letter a capital, at least two (`KOT`).
    Upper,
    /// Anything else (`McDonald`, `kOT`).
    Mixed,
}

/// The casing of `w`.
pub fn casing(w: &str) -> Casing {
    let (mut upper, mut lower, mut first_upper) = (0usize, 0usize, false);
    for (i, c) in w.chars().enumerate() {
        if c.is_uppercase() {
            upper += 1;
            if i == 0 {
                first_upper = true;
            }
        } else if c.is_lowercase() {
            lower += 1;
        }
    }
    if upper == 0 {
        Casing::Lower
    } else if upper == 1 && first_upper {
        Casing::Initial
    } else if lower == 0 {
        Casing::Upper
    } else {
        Casing::Mixed
    }
}

/// `w` with its first letter a capital.
pub fn capitalize(w: &str) -> String {
    let mut c = w.chars();
    match c.next() {
        Some(f) => f.to_uppercase().chain(c).collect(),
        None => String::new(),
    }
}

/// Parsed rules and tables, shared by the Hunspell reader and the compact format.
#[derive(Default)]
struct Rules {
    prefixes: Vec<RawRule>,
    suffixes: Vec<RawRule>,
    try_chars: String,
    key: String,
    map: Vec<Vec<char>>,
    rep: Vec<(String, String)>,
    /// The `FORBIDDENWORD` flag: stems carrying it are listed but never correct.
    forbidden: Option<char>,
    no_suggest: Option<char>,
    /// `NEEDAFFIX`: stems (and affixes) carrying it are not words without a further affix.
    need_affix: Option<char>,
    /// `KEEPCASE`: stems carrying it are words only in their own case.
    keep_case: Option<char>,
    /// `ONLYINCOMPOUND`: stems carrying it appear only inside compounds, which this engine
    /// doesn't build, so they are never words here.
    only_in_compound: Option<char>,
    flags: FlagNames,
    /// `IGNORE`: characters dropped from stems, affixes and the words checked.
    ignore: Vec<char>,
    /// `ICONV`: replacements made in a word before it is looked up.
    iconv: Vec<(String, String)>,
    /// `FULLSTRIP`: a rule may strip a whole stem.
    fullstrip: bool,
}

/// The directives that name one flag with a meaning of its own.
const FLAG_DIRECTIVES: [&str; 5] = ["FORBIDDENWORD", "NOSUGGEST", "NEEDAFFIX", "KEEPCASE", "ONLYINCOMPOUND"];

impl Rules {
    /// The flag a directive of [`FLAG_DIRECTIVES`] names.
    fn special_mut(&mut self, directive: &str) -> Option<&mut Option<char>> {
        match directive {
            "FORBIDDENWORD" => Some(&mut self.forbidden),
            "NOSUGGEST" => Some(&mut self.no_suggest),
            "NEEDAFFIX" => Some(&mut self.need_affix),
            "KEEPCASE" => Some(&mut self.keep_case),
            "ONLYINCOMPOUND" => Some(&mut self.only_in_compound),
            _ => None,
        }
    }
}

/// How a dictionary writes its flags (`FLAG`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum FlagMode {
    /// One character each (`FLAG UTF-8` and the default).
    #[default]
    Char,
    /// Two characters each (`FLAG long`: `/AaBb` is `Aa` and `Bb`).
    Long,
    /// Decimal numbers separated by commas (`FLAG num`: `/101,3`).
    Num,
}

/// The first character standing for a multi-character flag: each distinct `FLAG long` or
/// `FLAG num` flag becomes one character of Supplementary Private Use Area-A, so the rest of the
/// engine (and the compact format) keeps one character per flag.
const ALIAS_BASE: u32 = 0xF0000;
/// Most distinct multi-character flags (the size of the private use plane, less its two
/// noncharacters); far above the [`MAX_FLAGS`] flags the masks hold.
const MAX_ALIASES: u32 = 0xFFFE;
/// Most `ICONV` entries kept: each word checked tries them at every position.
const MAX_ICONV: usize = 256;
/// Most `IGNORE` characters kept.
const MAX_IGNORE: usize = 64;

/// The flag notation of a dictionary and the characters its multi-character flags became.
#[derive(Debug, Default)]
struct FlagNames {
    mode: FlagMode,
    alias: HashMap<String, char>,
}

impl FlagNames {
    /// The character for the flag `name` in this notation.
    fn intern(&mut self, name: &str) -> Result<char, String> {
        if let Some(&c) = self.alias.get(name) {
            return Ok(c);
        }
        let next = u32::try_from(self.alias.len()).ok().filter(|&n| n < MAX_ALIASES).ok_or("too many distinct flags")?;
        // ALIAS_BASE + next stays inside the private use plane (next < MAX_ALIASES), so it is a
        // valid scalar value; `from_u32` still checks it.
        let c = char::from_u32(ALIAS_BASE + next).ok_or("too many distinct flags")?;
        self.alias.insert(name.to_string(), c);
        Ok(c)
    }

    /// The flags of a flag field (`ab`, `AaBb`, `101,3`), one character each.
    fn parse(&mut self, field: &str) -> Result<String, String> {
        match self.mode {
            FlagMode::Char => Ok(field.chars().filter(|&c| c != EMOJI_SELECTOR).collect()),
            FlagMode::Long => {
                let chars: Vec<char> = field.chars().collect();
                // A stray last character (an odd count) is a flag of its own, as Hunspell reads it.
                chars.chunks(2).map(|pair| self.intern(&pair.iter().collect::<String>())).collect()
            }
            FlagMode::Num => field
                .split(',')
                .filter(|p| !p.is_empty())
                .map(|p| match p.trim().parse::<u32>() {
                    Ok(n) => self.intern(&n.to_string()),
                    Err(_) => Err(format!("flag `{p}` is not a number")),
                })
                .collect(),
        }
    }

    /// The one flag of `field` (an affix class name, `FORBIDDENWORD`'s flag).
    fn one(&mut self, field: &str) -> Result<char, String> {
        let flags = self.parse(field)?;
        let mut it = flags.chars();
        match (it.next(), it.next()) {
            (Some(c), None) if !c.is_whitespace() => Ok(c),
            _ => Err(format!("`{field}` is not one flag")),
        }
    }
}

/// `w` with the `IGNORE` characters taken out (borrowed when it has none).
fn without<'a>(w: &'a str, ignore: &[char]) -> std::borrow::Cow<'a, str> {
    if ignore.is_empty() || !w.contains(ignore) {
        std::borrow::Cow::Borrowed(w)
    } else {
        std::borrow::Cow::Owned(w.chars().filter(|c| !ignore.contains(c)).collect())
    }
}

/// `w` with the `ICONV` table applied: left to right, the longest pattern starting at each
/// position is replaced (borrowed when nothing matches).
fn convert<'a>(w: &'a str, table: &[(String, String)]) -> std::borrow::Cow<'a, str> {
    if table.is_empty() || !table.iter().any(|(from, _)| w.contains(from.as_str())) {
        return std::borrow::Cow::Borrowed(w);
    }
    let mut out = String::with_capacity(w.len());
    let mut i = 0;
    while let Some(rest) = w.get(i..).filter(|r| !r.is_empty()) {
        let best = table.iter().filter(|(from, _)| rest.starts_with(from.as_str())).max_by_key(|(from, _)| from.len());
        match best {
            Some((from, to)) => {
                out.push_str(to);
                i += from.len();
            }
            None => {
                let Some(c) = rest.chars().next() else { break };
                out.push(c);
                i += c.len_utf8();
            }
        }
    }
    std::borrow::Cow::Owned(out)
}

/// A rule as parsed from text, before flags become bits: `(flag, cross, strip, add, cont, cond)`.
type RawRule = (char, bool, String, String, String, String);

/// Most distinct `TRY` characters kept; a hostile `.aff` could otherwise make every suggestion
/// round cost time and memory proportional to the file size.
const MAX_TRY_CHARS: usize = 128;

/// Keeps the first [`MAX_TRY_CHARS`] distinct characters of a `TRY` value, in order.
fn cap_try(s: &str) -> String {
    let mut out = String::new();
    let mut n = 0;
    for c in s.chars() {
        if n >= MAX_TRY_CHARS {
            break;
        }
        if !out.contains(c) {
            out.push(c);
            n += 1;
        }
    }
    out
}

/// `0` means empty in Hunspell's strip and append fields.
fn zero_empty(s: &str) -> String {
    if s == "0" { String::new() } else { s.to_string() }
}

/// U+FE0F VARIATION SELECTOR-16 asks for the emoji form of the character before it. A dictionary
/// that uses an emoji as a flag may write it after the flag (RLA-ES Spanish: `PFX ☎️`, `banco/S☎️`),
/// but it is not a flag of its own: it is dropped wherever flags are read.
const EMOJI_SELECTOR: char = '\u{fe0f}';

/// Read Hunspell `.aff` text (already decoded to UTF-8).
fn parse_aff(aff: &str) -> Result<Rules, String> {
    let mut r = Rules::default();
    let mut cross: HashMap<(bool, char), bool> = HashMap::new();
    let mut map_header = false;
    let mut rep_header = false;
    let mut iconv_header = false;
    for (n, line) in aff.lines().enumerate() {
        let f: Vec<&str> = line.split_whitespace().collect();
        let Some(&dir) = f.first() else { continue };
        if dir.starts_with('#') {
            continue;
        }
        let at = |k: usize| f.get(k).copied().unwrap_or("");
        match dir {
            "SET" | "LANG" | "NAME" | "VERSION" | "HOME" | "WORDCHARS" | "MAXNGRAMSUGS" | "MAXDIFF" | "ONLYMAXDIFF" | "NOSPLITSUGS"
            | "SUGSWITHDOTS" | "WARN" | "FORBIDWARN" | "MAXCPDSUGS" => {}
            "FLAG" => {
                // `FLAG UTF-8` (VERO's pt_BR) is the default one-character notation.
                r.flags.mode = match at(1) {
                    "UTF-8" | "DEFAULT" | "utf-8" | "default" | "UTF8" | "" => FlagMode::Char,
                    "long" | "LONG" => FlagMode::Long,
                    "num" | "NUM" => FlagMode::Num,
                    other => return Err(format!("line {}: FLAG {other} is not supported", n + 1)),
                };
            }
            d if FLAG_DIRECTIVES.contains(&d) => {
                let c = r.flags.one(at(1)).map_err(|e| format!("line {}: {d}: {e}", n + 1))?;
                if let Some(slot) = r.special_mut(d) {
                    *slot = Some(c);
                }
            }
            // Compound rules only add words (Galician's build numerals like `1º`); without them
            // those words are flagged, and every other word is checked as before.
            "COMPOUNDRULE" | "COMPOUNDMIN" => {}
            "IGNORE" => r.ignore = at(1).chars().take(MAX_IGNORE).collect(),
            "FULLSTRIP" => r.fullstrip = true,
            "ICONV" => {
                if !iconv_header && f.len() == 2 && at(1).parse::<usize>().is_ok() {
                    iconv_header = true;
                } else if f.len() >= 3 && r.iconv.len() < MAX_ICONV {
                    r.iconv.push((at(1).to_string(), at(2).to_string()));
                }
            }
            "TRY" => r.try_chars = cap_try(at(1)),
            "KEY" => r.key = at(1).to_string(),
            "MAP" => {
                if !map_header && at(1).parse::<usize>().is_ok() {
                    map_header = true;
                } else if !at(1).contains('(') {
                    r.map.push(at(1).chars().collect());
                }
            }
            "REP" => {
                if !rep_header && f.len() == 2 && at(1).parse::<usize>().is_ok() {
                    rep_header = true;
                } else if f.len() >= 3 {
                    r.rep.push((at(1).replace('_', " "), at(2).replace('_', " ")));
                }
            }
            "PFX" | "SFX" => {
                let suffix = dir == "SFX";
                let flag = r.flags.one(at(1)).map_err(|e| format!("line {}: {e}", n + 1))?;
                // Only a header ignores trailing fields; a rule condition beginning with # is literal.
                let is_header = f.len() >= 4 && matches!(at(2), "Y" | "N") && at(3).parse::<usize>().is_ok() && !cross.contains_key(&(suffix, flag));
                if is_header {
                    cross.insert((suffix, flag), at(2) == "Y");
                    continue;
                }
                if f.len() < 4 {
                    return Err(format!("line {}: short affix rule", n + 1));
                }
                // `able/B`: the appended text and its continuation classes.
                let (add, cont) = match at(3).split_once('/') {
                    Some((a, c)) => (a, c),
                    None => (at(3), ""),
                };
                let cond = if f.len() >= 5 { at(4) } else { "." };
                parse_cond(cond).map_err(|e| format!("line {}: {e}", n + 1))?;
                let cont = r.flags.parse(cont).map_err(|e| format!("line {}: {e}", n + 1))?;
                let c = cross.get(&(suffix, flag)).copied().unwrap_or(false);
                let rule: RawRule = (flag, c, zero_empty(at(2)), zero_empty(add), cont, cond.to_string());
                if suffix {
                    r.suffixes.push(rule);
                } else {
                    r.prefixes.push(rule);
                }
            }
            d if UNSUPPORTED.contains(&d) => return Err(format!("line {}: {d} is not supported", n + 1)),
            _ => {}
        }
    }
    Ok(r)
}

impl AffixDict {
    /// Build from Hunspell `.aff` and `.dic` text (decoded to UTF-8; the `.dic` count line and
    /// morphological fields are ignored). `comments` (provenance, licence) are kept with the data.
    pub fn from_hunspell(aff: &str, dic: &str, comments: &[&str]) -> Result<AffixDict, String> {
        // Some dictionaries open with a UTF-8 BOM; it must not glue itself to the first
        // directive or turn the `.dic` count line into a stem.
        let aff = aff.strip_prefix('\u{feff}').unwrap_or(aff);
        let dic = dic.strip_prefix('\u{feff}').unwrap_or(dic);
        let mut rules = parse_aff(aff)?;
        if !rules.ignore.is_empty() {
            let ig = rules.ignore.clone();
            for r in rules.prefixes.iter_mut().chain(rules.suffixes.iter_mut()) {
                r.2 = without(&r.2, &ig).into_owned();
                r.3 = without(&r.3, &ig).into_owned();
            }
        }
        let mut stems: Vec<(String, String)> = Vec::new();
        for (n, line) in dic.lines().enumerate() {
            let Some(tok) = line.split_whitespace().next() else { continue };
            if n == 0 && tok.parse::<usize>().is_ok() {
                continue;
            }
            // `\/` is a literal slash in a word; the first plain `/` starts the flags.
            let mut word = String::new();
            let mut flags = String::new();
            let mut chars = tok.chars().peekable();
            let mut in_flags = false;
            while let Some(c) = chars.next() {
                if in_flags {
                    if c != EMOJI_SELECTOR {
                        flags.push(c);
                    }
                } else if c == '\\' && chars.peek() == Some(&'/') {
                    word.push('/');
                    chars.next();
                } else if c == '/' {
                    in_flags = true;
                } else {
                    word.push(c);
                }
            }
            let word = without(&word, &rules.ignore).into_owned();
            if word.is_empty() || word.len() > MAX_WORD_BYTES || word.contains(['\n', '/']) {
                continue;
            }
            // A stem whose flags can't be read keeps none (it is still a word on its own).
            let flags = rules.flags.parse(&flags).unwrap_or_default();
            stems.push((word, flags));
        }
        AffixDict::build(rules, stems, comments.iter().map(ToString::to_string).collect())
    }

    fn build(rules: Rules, mut stems: Vec<(String, String)>, comments: Vec<String>) -> Result<AffixDict, String> {
        // Flags: every one a rule or a stem uses gets a bit.
        let mut flag_chars: Vec<char> = rules.prefixes.iter().chain(&rules.suffixes).map(|r| r.0).collect();
        flag_chars.extend(stems.iter().flat_map(|s| s.1.chars()));
        flag_chars.sort_unstable();
        flag_chars.dedup();
        if flag_chars.len() > MAX_FLAGS {
            return Err(format!("{} distinct flags; at most {MAX_FLAGS} are supported", flag_chars.len()));
        }
        // `flag_chars` is sorted: a binary search finds a flag's bit.
        let bit_of = |c: char| flag_chars.binary_search(&c).map_or(Mask::default(), Mask::bit);
        let forbidden = rules.forbidden.map_or(Mask::default(), bit_of);
        // Stems sorted by bytes; homonyms (the same word twice) are merged.
        stems.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        let mut d = AffixDict {
            words: String::with_capacity(stems.iter().map(|s| s.0.len()).sum()),
            offs: Vec::with_capacity(stems.len() + 1),
            flag_chars: flag_chars.clone(),
            forbidden,
            no_suggest: rules.no_suggest.map_or(Mask::default(), bit_of),
            need_affix: rules.need_affix.map_or(Mask::default(), bit_of),
            keep_case: rules.keep_case.map_or(Mask::default(), bit_of),
            only_in_compound: rules.only_in_compound.map_or(Mask::default(), bit_of),
            try_chars: rules.try_chars,
            key: rules.key,
            map: rules.map,
            rep: rules.rep,
            ignore: rules.ignore,
            iconv: rules.iconv,
            fullstrip: rules.fullstrip,
            comments,
            ..AffixDict::default()
        };
        d.offs.push(0);
        let mut set_ids: HashMap<Mask, u32> = HashMap::new();
        // Flags about the stem itself hold for a merged homonym only when every entry has them:
        // Galician lists `por` plainly and again with NEEDAFFIX, and `por` alone is a word.
        let stem_only = d.need_affix | d.keep_case | d.only_in_compound;
        let mut i = 0;
        while i < stems.len() {
            let Some((word, _)) = stems.get(i) else { break };
            let mut mask = Mask::default();
            let mut common = stem_only;
            let mut j = i;
            while let Some((w, f)) = stems.get(j)
                && w == word
            {
                let m = f.chars().map(bit_of).fold(Mask::default(), |a, b| a | b);
                mask |= m;
                common = common.and(m);
                j += 1;
            }
            let mask = mask.minus(stem_only) | common;
            let next_id = u32::try_from(d.sets.len()).map_err(|_| "too many distinct flag sets".to_string())?;
            let id = *set_ids.entry(mask).or_insert(next_id);
            if id as usize == d.sets.len() {
                d.sets.push(mask);
            }
            d.words.push_str(word);
            d.offs.push(u32::try_from(d.words.len()).map_err(|_| "stems too large".to_string())?);
            d.flags.push(id);
            i = j;
        }
        let mk = |(flag, cross, strip, add, cont, cond): RawRule| -> Result<Rule, String> {
            let cont = cont.chars().fold(Mask::default(), |m, c| m | bit_of(c));
            Ok(Rule { bit: bit_of(flag), flag, cross, strip: strip.into(), add: add.into(), cont, cond: parse_cond(&cond)?, cond_text: cond.into() })
        };
        d.prefixes = rules.prefixes.into_iter().map(mk).collect::<Result<_, _>>()?;
        d.suffixes = rules.suffixes.into_iter().map(mk).collect::<Result<_, _>>()?;
        d.index();
        Ok(d)
    }

    /// Build the suffix index and the stem hash table.
    fn index(&mut self) {
        let mut idx: FxMap<Box<str>, Vec<SfxGroup>> = FxMap::default();
        // Each (appended, stripped) pair's group, found by hash rather than by scanning the
        // appended string's groups: a dictionary with many strips per suffix stays linear.
        let mut group_of: FxMap<(&str, &str), usize> = FxMap::default();
        let mut by_flag: Vec<Vec<u32>> = vec![Vec::new(); MAX_FLAGS];
        for (i, r) in self.suffixes.iter().enumerate() {
            let groups = idx.entry(r.add.clone()).or_default();
            let Ok(i) = u32::try_from(i) else { break };
            match group_of.get(&(&*r.add, &*r.strip)).and_then(|&g| groups.get_mut(g)) {
                Some(g) => g.rules.push(i),
                None => {
                    group_of.insert((&r.add, &r.strip), groups.len());
                    groups.push(SfxGroup { strip: r.strip.clone(), rules: vec![i] });
                }
            }
            if let Some(list) = r.bit.first().and_then(|b| by_flag.get_mut(b)) {
                list.push(i);
            }
        }
        self.sfx_by_flag = by_flag;
        self.max_sfx_chars = self.suffixes.iter().map(|r| r.add.chars().count()).max().unwrap_or(0);
        self.sfx_index = idx;
        let n = self.len();
        let size = (n.max(1) * 2).next_power_of_two();
        let mut table = vec![EMPTY; size];
        let mask = size - 1;
        for i in 0..n {
            let Some(w) = self.word(i) else { continue };
            let mut slot = hash_str(w) as usize & mask;
            while table.get(slot).is_some_and(|&t| t != EMPTY) {
                slot = (slot + 1) & mask;
            }
            if let (Some(t), Ok(i)) = (table.get_mut(slot), u32::try_from(i)) {
                *t = i;
            }
        }
        self.table = table;
    }

    /// Number of stems.
    pub fn len(&self) -> usize {
        self.flags.len()
    }

    pub fn is_empty(&self) -> bool {
        self.flags.is_empty()
    }

    /// Number of prefix and suffix rules.
    pub fn rule_count(&self) -> usize {
        self.prefixes.len() + self.suffixes.len()
    }

    fn word(&self, i: usize) -> Option<&str> {
        let a = *self.offs.get(i)? as usize;
        let b = *self.offs.get(i + 1)? as usize;
        self.words.get(a..b)
    }

    /// The flag characters of `mask`, in flag order (how the compact format stores flags).
    fn flag_chars_for(&self, mask: Mask) -> String {
        self.flag_chars.iter().enumerate().filter(|&(i, _)| mask.has(i)).map(|(_, c)| c).collect()
    }

    /// The character of `mask` when it holds exactly one flag.
    fn flag_char(&self, mask: Mask) -> Option<char> {
        let s = self.flag_chars_for(mask);
        let mut it = s.chars();
        match (it.next(), it.next()) {
            (Some(c), None) => Some(c),
            _ => None,
        }
    }

    /// The stems, in byte order.
    pub fn stems(&self) -> impl Iterator<Item = &str> + '_ {
        (0..self.len()).filter_map(|i| self.word(i))
    }

    /// The stems starting with `prefix`, in byte order (a binary search, then a scan).
    pub fn stems_with_prefix<'a>(&'a self, prefix: &str) -> impl Iterator<Item = &'a str> + use<'a> {
        let prefix = prefix.to_string();
        let first = {
            let (mut lo, mut hi) = (0usize, self.len());
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                if self.word(mid).is_some_and(|w| w.as_bytes() < prefix.as_bytes()) {
                    lo = mid + 1;
                } else {
                    hi = mid;
                }
            }
            lo
        };
        (first..self.len()).map_while(move |i| self.word(i).filter(|w| w.starts_with(prefix.as_str())))
    }

    /// Flag mask of a stem, if `w` is one.
    fn stem_flags(&self, w: &str) -> Option<Mask> {
        self.stem_flags_excluding(w, self.never())
    }

    fn stem_flags_excluding(&self, w: &str, excluded: Mask) -> Option<Mask> {
        if self.table.is_empty() {
            return None;
        }
        let mask = self.table.len() - 1;
        let mut slot = hash_str(w) as usize & mask;
        for _ in 0..self.table.len() {
            let t = *self.table.get(slot)?;
            if t == EMPTY {
                return None;
            }
            if self.word(t as usize) == Some(w) {
                let id = *self.flags.get(t as usize)?;
                let mask = self.sets.get(id as usize).copied();
                // All derivations use this lookup: exclusions apply to stems and their forms.
                return mask.filter(|m| !m.intersects(excluded));
            }
            slot = (slot + 1) & mask;
        }
        None
    }

    /// Does `w` carry any flag of `flags` — as a stem, or as the form of a suffix rule whose
    /// continuation classes include one (chained suffixation, depth-bounded)?
    ///
    /// The rules of a group that could lead to `w` are merged into one mask of their flags and
    /// followed once, not once per rule: the answer is the same, but thousands of rules of one
    /// flag cost one walk, not one each (#3613).
    fn form_has_flag(&self, w: &str, flags: Mask, depth: u8, excluded: Mask) -> bool {
        if self.stem_flags_excluding(w, excluded).is_some_and(|f| f.intersects(flags)) {
            return true;
        }
        if depth == 0 {
            return false;
        }
        let mut root = String::with_capacity(w.len() + 8);
        let cuts = std::iter::once(w.len()).chain(w.char_indices().rev().map(|(i, _)| i).filter(|&i| i > 0 || self.fullstrip));
        for b in cuts.take(self.max_sfx_chars + 1) {
            let (Some(base), Some(add)) = (w.get(..b), w.get(b..)) else { continue };
            let Some(groups) = self.sfx_index.get(add) else { continue };
            for g in groups {
                root.clear();
                root.push_str(base);
                root.push_str(&g.strip);
                let inner = g
                    .rules
                    .iter()
                    .filter_map(|&i| self.suffixes.get(i as usize))
                    .filter(|r| r.cont.intersects(flags) && r.cond_matches(&root, true))
                    .fold(Mask::default(), |m, r| m | r.bit);
                if !inner.is_empty() && self.form_has_flag(&root, inner, depth - 1, excluded) {
                    return true;
                }
            }
        }
        false
    }

    /// Is `w` exactly (case and all) a stem or an affixed form of one?
    pub fn contains(&self, w: &str) -> bool {
        self.contains_excluding(&self.input(w), self.never())
    }

    /// Exactly spelled dictionary words eligible for suggestions (NOSUGGEST stems excluded).
    pub fn contains_suggestible(&self, w: &str) -> bool {
        self.contains_excluding(&self.input(w), self.never() | self.no_suggest)
    }

    /// Flags whose stems are never words here: `FORBIDDENWORD` and `ONLYINCOMPOUND`.
    fn never(&self) -> Mask {
        self.forbidden | self.only_in_compound
    }

    /// `w` as the dictionary spells its words: `ICONV` applied, `IGNORE` characters taken out.
    fn input<'a>(&self, w: &'a str) -> std::borrow::Cow<'a, str> {
        // Far too long to be a word (the lookup refuses it): not worth converting.
        if w.len() > MAX_WORD_BYTES * 4 {
            return std::borrow::Cow::Borrowed(w);
        }
        match convert(w, &self.iconv) {
            std::borrow::Cow::Borrowed(b) => without(b, &self.ignore),
            std::borrow::Cow::Owned(o) => std::borrow::Cow::Owned(without(&o, &self.ignore).into_owned()),
        }
    }

    fn contains_excluding(&self, w: &str, excluded: Mask) -> bool {
        if w.is_empty() || w.len() > MAX_WORD_BYTES {
            return false;
        }
        if self.stem_flags_excluding(w, excluded).is_some_and(|f| !f.intersects(self.need_affix)) || self.suffixed(w, Mask::default(), excluded) {
            return true;
        }
        let mut root = String::with_capacity(w.len() + 8);
        for r in &self.prefixes {
            let Some(rest) = w.strip_prefix(&*r.add) else { continue };
            if rest.is_empty() && !self.fullstrip {
                continue;
            }
            root.clear();
            root.push_str(&r.strip);
            root.push_str(rest);
            if !r.cond_matches(&root, false) {
                continue;
            }
            // A prefix whose continuation classes need an affix makes a word only with a suffix.
            let alone = !r.cont.intersects(self.need_affix);
            if alone && self.stem_flags_excluding(&root, excluded).is_some_and(|f| f.intersects(r.bit)) {
                return true;
            }
            if alone && self.form_has_flag(&root, r.bit, INNER_CHAIN, excluded) {
                return true;
            }
            if r.cross && self.suffixed(&root, r.bit, excluded) {
                return true;
            }
        }
        false
    }

    /// Is `w` a stem with one suffix? With `need` set (a prefix was taken off), the stem must
    /// carry those flags too and the suffix must allow cross products.
    fn suffixed(&self, w: &str, need: Mask, excluded: Mask) -> bool {
        let mut root = String::with_capacity(w.len() + 8);
        // Each split point from the end: the appended string is `w[b..]`, at most the longest
        // suffix, and something must stay before it.
        let cuts = std::iter::once(w.len()).chain(w.char_indices().rev().map(|(i, _)| i).filter(|&i| i > 0 || self.fullstrip));
        for b in cuts.take(self.max_sfx_chars + 1) {
            let (Some(base), Some(add)) = (w.get(..b), w.get(b..)) else { continue };
            let Some(groups) = self.sfx_index.get(add) else { continue };
            for g in groups {
                root.clear();
                root.push_str(base);
                root.push_str(&g.strip);
                let rules = || g.rules.iter().filter_map(|&i| self.suffixes.get(i as usize));
                // The stem path: `root` is a stem carrying the rule's flag (and the prefix's,
                // when a cross product is being checked).
                if let Some(f) = self.stem_flags_excluding(&root, excluded)
                    && f.contains_all(need)
                    && rules().any(|r| {
                        // NEEDAFFIX in the continuation classes: the form needs another affix (a
                        // prefix being checked with it counts).
                        let complete = !need.is_empty() || !r.cont.intersects(self.need_affix);
                        complete && f.intersects(r.bit) && (need.is_empty() || r.cross) && r.cond_matches(&root, true)
                    })
                {
                    return true;
                }
                // Continuation classes: `root` need not be a stem carrying this rule's flag —
                // it may be a form that gained the flag from another rule's continuation. The
                // matching rules' flags are looked for together, in one walk.
                if need.is_empty() {
                    let flags = rules()
                        .filter(|r| !r.cont.intersects(self.need_affix) && r.cond_matches(&root, true))
                        .fold(Mask::default(), |m, r| m | r.bit);
                    if !flags.is_empty() && self.form_has_flag(&root, flags, INNER_CHAIN, excluded) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Is `w` correct, Hunspell style: as written, or — for a capitalized word or one in capitals
    /// — in lower case (and capitalized), since a sentence may start with any word and headings
    /// may be in capitals. Lower-case input never matches a capitalized stem (`warszawa`).
    pub fn check(&self, w: &str) -> bool {
        self.check_excluding(w, self.never())
    }

    /// Whether a suggestion is correct in its displayed case and has an eligible stem.
    pub fn can_suggest(&self, w: &str) -> bool {
        self.check_excluding(w, self.never() | self.no_suggest)
    }

    fn check_excluding(&self, w: &str, excluded: Mask) -> bool {
        let w = &*self.input(w);
        if self.contains_excluding(w, excluded) {
            return true;
        }
        match casing(w) {
            Casing::Lower | Casing::Mixed => false,
            // KEEPCASE stems match only as written.
            Casing::Initial => self.contains_excluding(&w.to_lowercase(), excluded | self.keep_case),
            Casing::Upper => {
                let lower = w.to_lowercase();
                let excluded = excluded | self.keep_case;
                self.contains_excluding(&capitalize(&lower), excluded) || self.contains_excluding(&lower, excluded)
            }
        }
    }

    /// Every form of the stem `stem` (itself included), unsorted, possibly with repeats; at most
    /// [`MAX_FORMS`], and at most [`MAX_FORMS_WORK`] rules tried along the continuation chains and
    /// cross products, so a dictionary with thousands of self-continuing rules can't make one stem
    /// cost millions of strings (#3613).
    pub fn forms(&self, stem: &str) -> Vec<String> {
        let Some(f) = self.stem_flags(stem) else { return Vec::new() };
        let mut out = vec![stem.to_string()];
        let mut cross_sfx = Vec::new();
        // Suffix forms queued with the continuation classes they carry, for chained expansion;
        // each (form, classes) pair is expanded once.
        let mut chain: Vec<(String, Mask)> = Vec::new();
        let mut queued: std::collections::HashSet<(String, Mask)> = std::collections::HashSet::new();
        let mut work = 0usize;
        for r in &self.suffixes {
            if !f.intersects(r.bit) || !r.cond_matches(stem, true) {
                continue;
            }
            let Some(base) = stem.strip_suffix(&*r.strip) else { continue };
            let form = format!("{base}{}", r.add);
            if r.cross {
                cross_sfx.push(form.clone());
            }
            if !r.cont.is_empty() && queued.insert((form.clone(), r.cont)) {
                chain.push((form.clone(), r.cont));
            }
            out.push(form);
            if out.len() >= MAX_FORMS {
                return out;
            }
        }
        // Continuation classes chain suffixes (`workable/B` → `workables`), up to [`MAX_CHAIN`]
        // suffixes in all — the same bound the checker walks.
        'chain: for _ in 0..INNER_CHAIN {
            let level = std::mem::take(&mut chain);
            if level.is_empty() {
                break;
            }
            for (form, flags) in level {
                for r in self.suffixes_with(flags).into_iter().filter_map(|i| self.suffixes.get(i as usize)) {
                    work += 1;
                    if work > MAX_FORMS_WORK {
                        break 'chain;
                    }
                    if !r.cond_matches(&form, true) {
                        continue;
                    }
                    let Some(base) = form.strip_suffix(&*r.strip) else { continue };
                    let next = format!("{base}{}", r.add);
                    if !r.cont.is_empty() && queued.insert((next.clone(), r.cont)) {
                        chain.push((next.clone(), r.cont));
                    }
                    out.push(next);
                    if out.len() >= MAX_FORMS {
                        return out;
                    }
                }
            }
        }
        for r in &self.prefixes {
            if !f.intersects(r.bit) || !r.cond_matches(stem, false) {
                continue;
            }
            let Some(rest) = stem.strip_prefix(&*r.strip) else { continue };
            out.push(format!("{}{rest}", r.add));
            if r.cross {
                for s in &cross_sfx {
                    work += 1;
                    if out.len() >= MAX_FORMS || work > MAX_FORMS_WORK {
                        return out;
                    }
                    if let Some(rest) = s.strip_prefix(&*r.strip) {
                        out.push(format!("{}{rest}", r.add));
                    }
                }
            }
            if out.len() >= MAX_FORMS {
                return out;
            }
        }
        out
    }

    /// The suffix rules carrying any flag of `mask`, in rule order.
    fn suffixes_with(&self, mask: Mask) -> Vec<u32> {
        let mut rules: Vec<u32> = Vec::new();
        let mut lists = 0;
        for (bit, list) in self.sfx_by_flag.iter().enumerate() {
            if mask.has(bit) && !list.is_empty() {
                rules.extend_from_slice(list);
                lists += 1;
            }
        }
        if lists > 1 {
            rules.sort_unstable();
        }
        rules
    }

    /// The `TRY` letters, most frequent first (suggestions try these).
    pub fn try_chars(&self) -> &str {
        &self.try_chars
    }

    /// The `KEY` keyboard rows, separated by `|` (adjacent letters are likely mistypes).
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The `MAP` groups: letters often confused with each other (`aą`, `zżź`, `oóu`…).
    pub fn map(&self) -> &[Vec<char>] {
        &self.map
    }

    /// The `REP` table: common misspellings, `(wrong, right)`.
    pub fn rep(&self) -> &[(String, String)] {
        &self.rep
    }

    /// The provenance and licence lines.
    pub fn comments(&self) -> &[String] {
        &self.comments
    }

    /// Serialize to WordCraft's compact format: raw-deflate of `WCAFF1`, the comments, the
    /// tables and rules as text lines, a `WORDS` line, then one line per stem, front-coded
    /// against the previous one (a byte with the shared prefix length in bytes, the rest of the
    /// word, `/` and its flags if any).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = String::new();
        out.push_str(MAGIC);
        out.push('\n');
        for c in &self.comments {
            out.push_str("# ");
            out.push_str(c.trim());
            out.push('\n');
        }
        if !self.try_chars.is_empty() {
            out.push_str(&format!("TRY {}\n", self.try_chars));
        }
        if !self.key.is_empty() {
            out.push_str(&format!("KEY {}\n", self.key));
        }
        for m in &self.map {
            out.push_str(&format!("MAP {}\n", m.iter().collect::<String>()));
        }
        for (a, b) in &self.rep {
            out.push_str(&format!("REP {} {}\n", a.replace(' ', "_"), b.replace(' ', "_")));
        }
        if !self.ignore.is_empty() {
            out.push_str(&format!("IGNORE {}\n", self.ignore.iter().collect::<String>()));
        }
        for (from, to) in &self.iconv {
            out.push_str(&format!("ICONV {from} {to}\n"));
        }
        if self.fullstrip {
            out.push_str("FULLSTRIP\n");
        }
        for (directive, mask) in [
            ("FORBIDDENWORD", self.forbidden),
            ("NOSUGGEST", self.no_suggest),
            ("NEEDAFFIX", self.need_affix),
            ("KEEPCASE", self.keep_case),
            ("ONLYINCOMPOUND", self.only_in_compound),
        ] {
            if !mask.is_empty()
                && let Some(c) = self.flag_char(mask)
            {
                out.push_str(&format!("{directive} {c}\n"));
            }
        }
        let z = |s: &str| if s.is_empty() { "0".to_string() } else { s.to_string() };
        for (kind, rules) in [("PFX", &self.prefixes), ("SFX", &self.suffixes)] {
            for r in rules {
                let y = if r.cross { "Y" } else { "N" };
                // Continuation classes ride on the appended field, as in the `.aff` source.
                let mut add = z(&r.add);
                let cont = self.flag_chars_for(r.cont);
                if !cont.is_empty() {
                    add.push('/');
                    add.push_str(&cont);
                }
                out.push_str(&format!("{kind} {} {y} {} {} {}\n", r.flag, z(&r.strip), add, r.cond_text));
            }
        }
        out.push_str("WORDS\n");
        let mut raw = out.into_bytes();
        let mut prev: &[u8] = &[];
        for i in 0..self.len() {
            let Some(w) = self.word(i) else { continue };
            let b = w.as_bytes();
            let p = prev.iter().zip(b).take_while(|(x, y)| x == y).count().min(255);
            raw.push(p as u8);
            raw.extend_from_slice(b.get(p..).unwrap_or_default());
            let mask = self.flags.get(i).and_then(|&id| self.sets.get(id as usize)).copied().unwrap_or_default();
            if !mask.is_empty() {
                raw.push(b'/');
                raw.extend_from_slice(self.flag_chars_for(mask).as_bytes());
            }
            raw.push(b'\n');
            prev = b;
        }
        miniz_oxide::deflate::compress_to_vec(&raw, 10)
    }

    /// Read data written by [`AffixDict::to_bytes`].
    pub fn from_bytes(data: &[u8]) -> Result<AffixDict, String> {
        let raw = miniz_oxide::inflate::decompress_to_vec_with_limit(data, 64 << 20)
            .map_err(|e| format!("affix dictionary: inflate failed: {:?}", e.status))?;
        let words_at = raw.windows(7).position(|w| w == b"\nWORDS\n").ok_or("affix dictionary: no WORDS section")?;
        let head = std::str::from_utf8(raw.get(..words_at).unwrap_or_default()).map_err(|_| "affix dictionary: bad utf-8 in the rules")?;
        let mut lines = head.lines();
        if lines.next() != Some(MAGIC) {
            return Err("affix dictionary: bad header".into());
        }
        let mut rules = Rules::default();
        let mut comments = Vec::new();
        for line in lines {
            if let Some(c) = line.strip_prefix("# ") {
                comments.push(c.to_string());
                continue;
            }
            let f: Vec<&str> = line.split_whitespace().collect();
            let at = |k: usize| f.get(k).copied().unwrap_or("");
            match at(0) {
                "TRY" => rules.try_chars = cap_try(at(1)),
                "KEY" => rules.key = at(1).to_string(),
                "MAP" => rules.map.push(at(1).chars().collect()),
                "REP" => rules.rep.push((at(1).replace('_', " "), at(2).replace('_', " "))),
                "IGNORE" => rules.ignore = at(1).chars().take(MAX_IGNORE).collect(),
                "ICONV" if f.len() == 3 && rules.iconv.len() < MAX_ICONV => rules.iconv.push((at(1).to_string(), at(2).to_string())),
                "FULLSTRIP" => rules.fullstrip = true,
                directive if FLAG_DIRECTIVES.contains(&directive) => {
                    let mut it = at(1).chars();
                    match (it.next(), it.next(), rules.special_mut(directive)) {
                        (Some(c), None, Some(slot)) if !c.is_whitespace() => *slot = Some(c),
                        _ => return Err(format!("affix dictionary: bad {directive} in `{line}`")),
                    }
                }
                kind @ ("PFX" | "SFX") => {
                    let mut fc = at(1).chars();
                    let (Some(flag), None) = (fc.next(), fc.next()) else { return Err(format!("affix dictionary: bad flag in `{line}`")) };
                    if f.len() != 6 {
                        return Err(format!("affix dictionary: bad rule `{line}`"));
                    }
                    // Continuation classes ride on the appended field, as in the `.aff` source.
                    let (add, cont) = match at(4).split_once('/') {
                        Some((a, c)) => (zero_empty(a), c.to_string()),
                        None => (zero_empty(at(4)), String::new()),
                    };
                    let rule: RawRule = (flag, at(2) == "Y", zero_empty(at(3)), add, cont, at(5).to_string());
                    if kind == "SFX" {
                        rules.suffixes.push(rule);
                    } else {
                        rules.prefixes.push(rule);
                    }
                }
                "" => {}
                other => return Err(format!("affix dictionary: unknown line `{other}`")),
            }
        }
        let body = raw.get(words_at + 7..).unwrap_or_default();
        let mut stems: Vec<(String, String)> = Vec::new();
        let mut cur: Vec<u8> = Vec::new();
        let mut i = 0;
        while let Some(&p) = body.get(i) {
            let rest = body.get(i + 1..).unwrap_or_default();
            let len = rest.iter().position(|&b| b == b'\n').ok_or("affix dictionary: truncated")?;
            let line = rest.get(..len).unwrap_or_default();
            i += len + 2;
            let p = p as usize;
            if p > cur.len() {
                return Err("affix dictionary: bad prefix".into());
            }
            let (tail, flags) = match line.iter().position(|&b| b == b'/') {
                Some(k) => (line.get(..k).unwrap_or_default(), line.get(k + 1..).unwrap_or_default()),
                None => (line, &[][..]),
            };
            cur.truncate(p);
            cur.extend_from_slice(tail);
            let word = std::str::from_utf8(&cur).map_err(|_| "affix dictionary: bad utf-8 in a stem")?;
            let flags = std::str::from_utf8(flags).map_err(|_| "affix dictionary: bad utf-8 in flags")?;
            if word.is_empty() || word.len() > MAX_WORD_BYTES {
                return Err("affix dictionary: bad stem".into());
            }
            stems.push((word.to_string(), flags.to_string()));
        }
        AffixDict::build(rules, stems, comments)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    const AFF: &str = "SET UTF-8\nTRY aeoiułkr\n\nPFX b Y 1\nPFX b   0   nie   .\n\nSFX a Y 3\nSFX a   0   y     [^y]\nSFX a   ek  ka    ek\nSFX a   0   em    .\n\nSFX c N 1\nSFX c   a   ą     a\n\nMAP 2\nMAP aą\nMAP oóu\nREP 2\nREP ż rz\nREP u ó\n";
    const DIC: &str = "4\nkot/ab\npiesek/a\nkrowa/c\nWarszawa/c\n";

    fn dict() -> AffixDict {
        AffixDict::from_hunspell(AFF, DIC, &["test data"]).unwrap()
    }

    #[test]
    fn affix_headers_allow_comments_but_rule_conditions_stay_literal() {
        let aff = "SET UTF-8\nPFX X Y 1 # prefix comment\nPFX X 0 ne . # rule comment\nSFX y Y 1 # suffix comment\nSFX y 0 y . # rule comment\n";
        let d = AffixDict::from_hunspell(aff, "1\nstrom/Xy\n", &[]).unwrap();
        assert_eq!(d.rule_count(), 2, "headers must not become affix rules");
        for w in ["strom", "stromy", "nestrom", "nestromy"] {
            assert!(d.contains(w), "{w}");
        }
        for w in ["stromY1", "stromN1", "X", "y"] {
            assert!(!d.contains(w), "{w}");
        }
        let hash = AffixDict::from_hunspell("SFX H Y 1\nSFX H 0 # .\nTRY a#b\nKEY a#b\n", "1\nstrom/H\n", &[]).unwrap();
        assert!(hash.contains("strom#"), "a hash in an affix field is literal");
        let literal = AffixDict::from_hunspell(
            "PFX P Y 1 # header\nPFX P 0 pre #\nSFX S Y 1 # header\nSFX S 0 post #\n",
            "3\n#stem/P\nstem#/S\nstem/PS\n",
            &[],
        )
        .unwrap();
        assert!(literal.contains("pre#stem") && literal.contains("stem#post"));
        assert!(!literal.contains("prestem") && !literal.contains("stempost"), "# must remain a literal condition");

        assert_eq!(hash.try_chars(), "a#b");
        assert_eq!(hash.key(), "a#b");
        assert_eq!(AffixDict::from_bytes(&hash.to_bytes()).unwrap().key(), "a#b");
    }

    #[test]
    fn try_alphabet_is_capped_and_deduplicated() {
        let aff = format!("TRY {}\n", "abcdefghijklmnopqrstuvwxyz".repeat(10_000));
        let d = AffixDict::from_hunspell(&aff, "1\nkot\n", &[]).unwrap();
        assert_eq!(d.try_chars(), "abcdefghijklmnopqrstuvwxyz");
        let compact = AffixDict::from_bytes(&d.to_bytes()).unwrap();
        assert_eq!(compact.try_chars(), "abcdefghijklmnopqrstuvwxyz");

        // More distinct characters than the cap, each repeated: keep the first 128 in order.
        let many: String = (0x100u32..0x100 + 300).filter_map(char::from_u32).collect();
        let aff = format!("TRY {many}{many}\n");
        let want: String = many.chars().take(128).collect();
        let d = AffixDict::from_hunspell(&aff, "1\nkot\n", &[]).unwrap();
        assert_eq!(d.try_chars(), want);
        let compact = AffixDict::from_bytes(&d.to_bytes()).unwrap();
        assert_eq!(compact.try_chars(), want);

        // An oversized TRY line fed straight to the compact reader is capped as well.
        let raw = format!("{MAGIC}\nTRY {many}{many}\nWORDS\n\0kot\n");
        let bytes = miniz_oxide::deflate::compress_to_vec(raw.as_bytes(), 6);
        assert_eq!(AffixDict::from_bytes(&bytes).unwrap().try_chars(), want);
    }

    /// Shared hand-written fixture: collectors must reject every path to NOSUGGEST forms.
    pub(crate) fn assert_nosuggest_is_honoured(suggest: impl Fn(&AffixDict, &str, usize) -> Vec<String>) {
        let aff = "SET UTF-8\nNOSUGGEST N\nFORBIDDENWORD F\nTRY abeklmnostxy\nMAP 1\nMAP ae\nKEY ak\nREP 2\nREP xy toka\nREP zz normal\nPFX X Y 1\nPFX X 0 pre .\nSFX A Y 1\nSFX A 0 ble/B .\nSFX B Y 1\nSFX B 0 s .\n";
        let d = AffixDict::from_hunspell(aff, "4\ntoka/NXA\nnormal/XA\nTkname/N\nbanned/F\n", &[]).unwrap();
        for word in d.forms("toka").into_iter().chain(["TOKA".into(), "Toka".into(), "Tkname".into(), "TKNAME".into()]) {
            assert!(d.check(&word), "NOSUGGEST is still correct: {word}");
            assert!(!d.can_suggest(&word), "NOSUGGEST form: {word}");
        }
        for input in ["tokax", "tokablesx", "pretokax", "pretokablesx", "toka", "TOKA", "tkname", "teka", "xy", "normaltoka", "toak", "taak"] {
            let results = suggest(&d, input, 32);
            assert!(results.iter().all(|word| word.split(' ').all(|part| d.can_suggest(part))), "{input}: {results:?}");
            assert!(results.iter().all(|word| !word.to_lowercase().contains("toka") && word.to_lowercase() != "tkname"), "{input}: {results:?}");
        }
        assert!(suggest(&d, "norrmal", 8).contains(&"normal".into()));
        assert!(suggest(&d, "zz", 8).contains(&"normal".into()));
        // Filtering happens before display casing: ordinary mixed-case stems retain the
        // existing all-caps suggestion behaviour, with and without a NOSUGGEST declaration.
        for aff in ["", "NOSUGGEST N\n"] {
            let ordinary = AffixDict::from_hunspell(aff, "2\nmixCase\nhidden/N\n", &[]).unwrap();
            assert!(suggest(&ordinary, "MIXCASX", 8).contains(&"MIXCASE".into()));
        }
    }

    #[test]
    fn nosuggest_is_correct_but_not_suggestible_and_survives_compact_format() {
        let d = AffixDict::from_hunspell("NOSUGGEST N\nSFX S Y 1\nSFX S 0 s .\n", "2\nsecret/NS\nnormal/S\n", &[]).unwrap();
        for dict in [&d, &AffixDict::from_bytes(&d.to_bytes()).unwrap()] {
            for word in ["secret", "secrets", "Secret", "SECRETS"] {
                assert!(dict.check(word), "{word}");
                assert!(!dict.can_suggest(word), "{word}");
            }
            assert!(dict.contains_suggestible("normal") && dict.can_suggest("NORMALS"));
        }
        assert!(AffixDict::from_hunspell("NOSUGGEST NN\n", "1\nword\n", &[]).is_err());
    }

    #[test]
    fn stems_suffixes_prefixes_and_cross_products() {
        let d = dict();
        assert_eq!(d.len(), 4);
        for w in ["kot", "koty", "kotem", "piesek", "pieska", "piesekem", "krowa", "krową", "niekot", "niekoty", "niekotem", "Warszawa", "Warszawą"]
        {
            assert!(d.contains(w), "{w}");
        }
        for w in ["kotą", "pieseka", "pieskem", "niepiesek", "niekrową", "krowy", "warszawa", "", "nie", "y", "ka"] {
            assert!(!d.contains(w), "{w}");
        }
    }

    #[test]
    fn case_variants() {
        let d = dict();
        assert!(d.check("Kot") && d.check("KOT") && d.check("Koty") && d.check("WARSZAWĄ") && d.check("Warszawą"));
        assert!(!d.check("warszawa") && !d.check("kOT") && !d.check("KoT"));
        assert_eq!(casing("Kot"), Casing::Initial);
        assert_eq!(casing("KOT"), Casing::Upper);
        assert_eq!(casing("K"), Casing::Initial);
        assert_eq!(casing("kot"), Casing::Lower);
        assert_eq!(casing("McDonald"), Casing::Mixed);
        assert_eq!(capitalize("żółw"), "Żółw");
    }

    #[test]
    fn forms_are_all_accepted() {
        let d = dict();
        let mut forms = d.forms("kot");
        forms.sort();
        forms.dedup();
        assert_eq!(forms, ["kot", "kotem", "koty", "niekot", "niekotem", "niekoty"]);
        for stem in d.stems().map(str::to_string).collect::<Vec<_>>() {
            for f in d.forms(&stem) {
                assert!(d.contains(&f), "{stem} → {f}");
            }
        }
    }

    #[test]
    fn compact_format_round_trips() {
        let d = dict();
        let back = AffixDict::from_bytes(&d.to_bytes()).unwrap();
        assert_eq!(back.len(), d.len());
        assert_eq!(back.rule_count(), d.rule_count());
        assert_eq!(back.comments(), ["test data"]);
        assert_eq!(back.rep(), d.rep());
        assert_eq!(back.map(), d.map());
        assert_eq!(back.try_chars(), "aeoiułkr");
        for w in ["kotem", "niekoty", "pieska", "krową", "Warszawą"] {
            assert!(back.contains(w), "{w}");
        }
        assert!(!back.contains("kotą"));
    }

    #[test]
    fn flag_utf8_and_default_modes_are_accepted() {
        // `FLAG UTF-8` (VERO pt_BR) and `FLAG DEFAULT` are per-character flag modes: the same
        // mode the reader already assumes. Only `long` and `num` stay unsupported.
        for mode in ["UTF-8", "DEFAULT"] {
            let aff = format!("SET UTF-8\nFLAG {mode}\nSFX a Y 1\nSFX a   0   y   [^y]\n");
            let d = AffixDict::from_hunspell(&aff, "1\nkot/a\n", &[]).unwrap();
            assert!(d.contains("kot") && d.contains("koty"), "{mode}");
        }
        assert!(AffixDict::from_hunspell("FLAG UTF-8\n", "0\n", &[]).is_ok(), "FLAG alone is harmless");
    }

    #[test]
    fn emoji_flags_with_a_variation_selector_load() {
        // RLA-ES (the Spanish dictionaries in LibreOffice and most Linux distributions) writes its
        // telephone flag as ☎ followed by U+FE0F, in the rule and on the stems.
        let aff = "SET UTF-8\nFLAG UTF-8\nPFX \u{260e}\u{fe0f} Y 1\nPFX \u{260e}\u{fe0f} 0 tele .\nSFX S Y 1\nSFX S 0 s [aeiou]\n";
        let d = AffixDict::from_hunspell(aff, "2\nbanco/S\u{260e}\u{fe0f}\nbasura/S\n", &[]).unwrap();
        assert!(d.contains("banco") && d.contains("bancos"));
        assert!(d.contains("telebanco") && d.contains("telebancos"), "the ☎ prefix applies");
        assert!(d.contains("basuras") && !d.contains("telebasura"), "only to stems that carry it");
        assert!(!d.flag_chars.contains(&EMOJI_SELECTOR), "the selector is not a flag");
    }

    #[test]
    fn emoji_forbiddenword_and_nosuggest_flags_load() {
        // The same selector after the flag of the directives that name one flag.
        let aff = "SET UTF-8\nFLAG UTF-8\nFORBIDDENWORD \u{260e}\u{fe0f}\nNOSUGGEST \u{1f6ab}\u{fe0f}\n";
        let d = AffixDict::from_hunspell(aff, "3\nbanco\nbanko/\u{260e}\u{fe0f}\nbancu/\u{1f6ab}\u{fe0f}\n", &[]).unwrap();
        assert!(d.contains("banco") && d.contains("bancu"));
        assert!(!d.contains("banko"), "a stem with the FORBIDDENWORD flag is not a word");
    }

    #[test]
    fn long_flags_name_rules_continuations_and_forbidden_words() {
        // Catalan's notation: two characters per flag.
        let aff = "FLAG long\nFORBIDDENWORD ZZ\nSFX Aa Y 1\nSFX Aa 0 s .\nPFX Bb Y 1\nPFX Bb 0 re .\nSFX Cc Y 1\nSFX Cc 0 ment/Aa .\n";
        let d = AffixDict::from_hunspell(aff, "3\ndo/AaBb\nagree/Cc\nbadword/ZZ\n", &[]).unwrap();
        for w in ["do", "dos", "redo", "redos", "agree", "agreement", "agreements"] {
            assert!(d.check(w), "{w}");
        }
        for w in ["agrees", "reagree", "badword", "Badword", "a", "Aa"] {
            assert!(!d.check(w), "{w}");
        }
        let back = AffixDict::from_bytes(&d.to_bytes()).unwrap();
        assert!(back.check("agreements") && back.check("redos") && !back.check("badword"), "the compact format keeps aliased flags");
    }

    #[test]
    fn numeric_flags_and_need_affix() {
        // Galician's and Basque's notation: numbers separated by commas.
        let aff = "FLAG num\nNEEDAFFIX 7\nSFX 101 Y 1\nSFX 101 0 s .\nSFX 5 Y 1\nSFX 5 0 a/7 .\nSFX 6 Y 1\nSFX 6 0 k .\n";
        let d = AffixDict::from_hunspell(aff, "3\ncat/101\nbound/101,7\nkot/5,6\n", &[]).unwrap();
        assert!(d.check("cat") && d.check("cats"));
        assert!(!d.check("bound") && d.check("bounds"), "a NEEDAFFIX stem is a word only with an affix");
        assert!(!d.check("kota"), "a suffix carrying NEEDAFFIX needs another one");
        assert!(d.check("kot") && d.check("kotk"));
        let back = AffixDict::from_bytes(&d.to_bytes()).unwrap();
        assert!(!back.check("bound") && back.check("bounds") && !back.check("kota"), "NEEDAFFIX survives the compact format");
        // A homonym listed plainly and again with NEEDAFFIX (Galician `por`) is a word alone.
        let d = AffixDict::from_hunspell(aff, "3\npor\npor/101,7\nsó/7\n", &[]).unwrap();
        assert!(d.check("por") && d.check("pors") && !d.check("só"));
    }

    #[test]
    fn more_than_128_flags_load() {
        // Basque's Xuxen uses about 450 numeric flags.
        let mut aff = String::from("FLAG num\n");
        for i in 1..=300 {
            aff.push_str(&format!("SFX {i} Y 1\nSFX {i} 0 x{i} .\n"));
        }
        let all: Vec<String> = (1..=300).map(|i| i.to_string()).collect();
        let d = AffixDict::from_hunspell(&aff, &format!("2\nbase/{}\nother/300\n", all.join(",")), &[]).unwrap();
        for w in ["base", "basex1", "basex129", "basex300", "otherx300"] {
            assert!(d.check(w), "{w}");
        }
        assert!(!d.check("otherx1") && !d.check("basex301"));
        let too_many: String = (1..=MAX_FLAGS + 1).map(|i| format!("SFX {i} Y 1\nSFX {i} 0 y .\n")).collect();
        assert!(AffixDict::from_hunspell(&format!("FLAG num\n{too_many}"), "1\nw\n", &[]).is_err());
    }

    #[test]
    fn ignore_and_iconv_normalize_the_word_checked() {
        // Ukrainian's tables: the stress mark is ignored, typographic apostrophes read as `'`.
        let aff = "SET UTF-8\nIGNORE \u{301}\nICONV 3\nICONV ’ '\nICONV ʼ '\nICONV a 0\nSFX A Y 1\nSFX A о у о\n";
        let d = AffixDict::from_hunspell(aff, "2\nм'ясо/A\nзапла\u{301}тить\n", &[]).unwrap();
        for w in ["м'ясо", "м’ясо", "мʼясо", "м'ясу", "м’ясу", "заплатить", "запла\u{301}тить", "М’ЯСО"]
        {
            assert!(d.check(w), "{w}");
        }
        assert!(!d.check("м'яс"));
        assert!(!d.check("заплaтить"), "a Latin letter inside a Cyrillic word is converted away from the dictionary's spelling");
        let back = AffixDict::from_bytes(&d.to_bytes()).unwrap();
        assert!(back.check("мʼясу") && back.check("запла\u{301}тить"), "the tables survive the compact format");
    }

    #[test]
    fn fullstrip_lets_a_rule_replace_the_whole_stem() {
        let rules = "SFX A Y 1\nSFX A ser és ser\n";
        let d = AffixDict::from_hunspell(&format!("FULLSTRIP\n{rules}"), "1\nser/A\n", &[]).unwrap();
        assert!(d.check("ser") && d.check("és"));
        assert!(AffixDict::from_bytes(&d.to_bytes()).unwrap().check("és"));
        let d = AffixDict::from_hunspell(rules, "1\nser/A\n", &[]).unwrap();
        assert!(!d.check("és"), "without FULLSTRIP a form keeps part of its stem");
    }

    #[test]
    fn keepcase_and_onlyincompound_stems() {
        let aff = "FLAG num\nKEEPCASE 1\nONLYINCOMPOUND 2\nCOMPOUNDMIN 1\nCOMPOUNDRULE 1\nCOMPOUNDRULE (2)(3)\n";
        let d = AffixDict::from_hunspell(aff, "4\niPhone/1\nmm/1\nParis\ndigit/2\n", &[]).unwrap();
        assert!(d.check("iPhone") && d.check("mm") && d.check("Paris") && d.check("PARIS"));
        assert!(!d.check("IPHONE") && !d.check("MM") && !d.check("Mm"), "KEEPCASE words match only as written");
        assert!(!d.check("digit"), "ONLYINCOMPOUND stems aren't words alone");
        let back = AffixDict::from_bytes(&d.to_bytes()).unwrap();
        assert!(!back.check("MM") && back.check("mm") && !back.check("digit"));
    }

    proptest::proptest! {
        #[test]
        fn flag_fields_never_panic(field in "\\PC{0,24}") {
            for mode in [FlagMode::Char, FlagMode::Long, FlagMode::Num] {
                let mut names = FlagNames { mode, ..FlagNames::default() };
                let _ = names.parse(&field);
                let _ = names.one(&field);
            }
            let _ = AffixDict::from_hunspell(&format!("FLAG long\nIGNORE {field}\nICONV 1\nICONV {field} x\n"), &format!("1\nw/{field}\n"), &[]);
            let _ = AffixDict::from_hunspell(&format!("FLAG num\nSFX 1 Y 1\nSFX 1 0 a/{field} .\n"), &format!("1\n{field}/1\n"), &[]);
        }
    }

    #[test]
    fn more_than_64_distinct_flags_load() {
        // VERO pt_BR uses 106 distinct flags (FLAG UTF-8). Build one with every basic Latin
        // letter, the digits and eight accented letters: 70 flags, 64 < n <= 128.
        let mut flags = String::new();
        let mut aff = String::from("SET UTF-8\nFLAG UTF-8\n");
        let chars: Vec<char> = ('a'..='z').chain('A'..='Z').chain('0'..='9').chain(['á', 'é', 'í', 'ó', 'ú', 'ç', 'ã', 'õ']).collect();
        assert_eq!(chars.len(), 70);
        for c in chars {
            aff.push_str(&format!("SFX {c} N 1\nSFX {c}   0   {c}   .\n"));
            flags.push(c);
        }
        let d = AffixDict::from_hunspell(&aff, &format!("1\nverbo/{flags}\n"), &[]).unwrap();
        assert!(d.contains("verbo"));
        assert!(d.contains("verboa") && d.contains("verboz") && d.contains("verbo9"));
        assert!(d.contains("verboç") && d.contains("verboã"), "multi-byte UTF-8 flags");
        assert!(!d.contains("verboß"), "ß is not a flag and not a stem");
    }

    #[test]
    fn forbidden_words_are_rejected() {
        // VERO pt_BR marks spellings it must reject (flag ý under FLAG UTF-8) as stems carrying
        // FORBIDDENWORD: listed, but never correct and never inflected.
        let aff = "SET UTF-8\nFLAG UTF-8\nFORBIDDENWORD ý\nSFX a Y 1\nSFX a   0   y   [^y]\n";
        let d = AffixDict::from_hunspell(aff, "2\nkot/a\nimundo/ý\n", &[]).unwrap();
        assert!(d.check("kot") && d.check("koty") && d.check("Kot") && d.check("KOT"));
        assert!(!d.check("imundo") && !d.check("Imundo") && !d.check("IMUNDO"));
        assert!(!d.contains("imundoy"), "a forbidden stem must not inflect");
        assert!(d.forms("imundo").is_empty());
    }

    #[test]
    fn continuation_classes_chain_suffixes() {
        // `SFX A 0 able/B`: the form derived by A carries flag B, so a suffix of class B may
        // follow (workable → workables). The bare stem has A only, so `works` stays wrong.
        let aff = "SET UTF-8\nFLAG UTF-8\nSFX A N 1\nSFX A   0   able/B   .\nSFX B N 1\nSFX B   0   s   .\n";
        let d = AffixDict::from_hunspell(aff, "1\nwork/A\n", &[]).unwrap();
        assert!(d.contains("work") && d.contains("workable") && d.contains("workables"));
        assert!(!d.contains("works"), "the stem has A, not B");
        assert!(!d.contains("workabless"), "B declares no continuation");
        assert!(d.check("Workable") && d.check("WORKABLES"));
    }

    #[test]
    fn continuation_chains_stop_at_two_suffixes() {
        // Hunspell strips at most two suffixes: work + able/B + s/C is a word, a third (C) is not.
        let aff = "SET UTF-8\nFLAG UTF-8\nSFX A N 1\nSFX A   0   able/B   .\nSFX B N 1\nSFX B   0   s/C   .\nSFX C N 1\nSFX C   0   x   .\n";
        let d = AffixDict::from_hunspell(aff, "1\nwork/A\n", &[]).unwrap();
        assert!(d.contains("workable") && d.contains("workables"));
        assert!(!d.contains("workablesx"), "three chained suffixes exceed the limit");
        assert!(!d.forms("work").iter().any(|f| f == "workablesx"));
    }

    #[test]
    fn forms_include_continuation_chains() {
        let aff = "SET UTF-8\nFLAG UTF-8\nSFX A N 1\nSFX A   0   able/B   .\nSFX B N 1\nSFX B   0   s   .\n";
        let d = AffixDict::from_hunspell(aff, "1\nwork/A\n", &[]).unwrap();
        let mut f = d.forms("work");
        f.sort();
        f.dedup();
        assert_eq!(f, ["work", "workable", "workables"]);
        for form in d.forms("work") {
            assert!(d.contains(&form), "every listed form is accepted: {form}");
        }
    }

    #[test]
    fn continuation_classes_merge_the_flags_of_a_group() {
        // `s` under B and under C share one group (same append, same strip): a form carrying
        // either flag takes it, and the stem carrying neither doesn't.
        let aff = "SET UTF-8\nSFX A N 1\nSFX A 0 able/C .\nSFX D N 1\nSFX D 0 er/B .\nSFX B N 1\nSFX B 0 s .\nSFX C N 1\nSFX C 0 s [e]\n";
        let d = AffixDict::from_hunspell(aff, "2\nwork/A\nplay/D\n", &[]).unwrap();
        assert!(d.contains("workables") && d.contains("players"));
        assert!(!d.contains("works") && !d.contains("plays") && !d.contains("workabless"));
        assert!(d.forms("work").iter().any(|f| f == "workables") && d.forms("play").iter().any(|f| f == "players"));
    }

    /// `n` suffix rules of flag A on one stem, `rule(i)` giving each rule's strip, append and condition.
    fn many_rules(n: usize, rule: impl Fn(usize) -> String) -> AffixDict {
        let mut aff = format!("SET UTF-8\nSFX A Y {n}\n");
        for i in 0..n {
            aff.push_str(&format!("SFX A {}\n", rule(i)));
        }
        AffixDict::from_hunspell(&aff, "1\nkot/A\n", &[]).unwrap()
    }

    #[test]
    fn self_continuing_rules_check_in_linear_time() {
        // Thousands of empty-append rules that continue with their own flag: each check used to
        // follow every rule from every rule, millions of steps a word (#3613). The checks take
        // about a millisecond; the old checker took 14 s for two of them at 16,000 rules, so at
        // 40,000 (quadratic: minutes) the 10 s bound leaves a busy machine a wide margin.
        let d = many_rules(40_000, |_| "0 0/A .".into());
        let t = std::time::Instant::now();
        assert!(d.check("kot") && !d.check("pies") && !d.check("kotx"));
        assert!(t.elapsed().as_secs_f64() < 10.0, "{:?}", t.elapsed());
    }

    #[test]
    fn forms_of_self_continuing_rules_are_bounded() {
        // Every rule's form continues with every rule: millions of strings for one stem (#3613).
        let same = many_rules(3000, |_| "0 0/A .".into());
        let distinct = many_rules(3000, |i| format!("0 x{i}/A ."));
        let t = std::time::Instant::now();
        let (a, b) = (same.forms("kot"), distinct.forms("kot"));
        // The cap on the count is what tells the old code (9 million forms each) apart; the
        // generous time bound (tens of ms here) only backs up the rule-visit cap.
        assert!(t.elapsed().as_secs_f64() < 10.0, "{:?}", t.elapsed());
        assert!(a.len() <= MAX_FORMS && b.len() <= MAX_FORMS, "{} {}", a.len(), b.len());
        assert!(b.iter().any(|f| f == "kotx0x1"), "chained forms are still listed");
    }

    #[test]
    fn many_strips_per_suffix_index_in_linear_time() {
        // One append with thousands of strips: the index used to scan the append's groups for
        // each rule. Building 200,000 rules takes well under a second; the old index took 3 s at
        // 30,000 and over 10 minutes at 300,000.
        let n = 200_000;
        let t = std::time::Instant::now();
        let d = many_rules(n, |i| format!("k{i} y ."));
        assert!(t.elapsed().as_secs_f64() < 10.0, "{:?}", t.elapsed());
        assert_eq!(d.sfx_index.get("y").map(Vec::len), Some(n));
        assert!(!d.contains("koty"));
    }

    #[test]
    fn compact_format_round_trips_continuations_and_forbidden() {
        let aff = "SET UTF-8\nFLAG UTF-8\nFORBIDDENWORD ý\nSFX A N 1\nSFX A   0   able/B   .\nSFX B N 1\nSFX B   0   s   .\n";
        let d = AffixDict::from_hunspell(aff, "2\nwork/A\nimundo/ý\n", &[]).unwrap();
        let back = AffixDict::from_bytes(&d.to_bytes()).unwrap();
        assert!(back.contains("work") && back.contains("workable") && back.contains("workables"));
        assert!(!back.contains("works"), "the stem has A, not B, after a round trip");
        assert!(!back.check("imundo") && !back.contains("imundoy"), "forbidden survives a round trip");
    }

    #[test]
    fn dic_bom_glued_to_the_count_line_is_skipped() {
        // VERO's .dic opens with a UTF-8 BOM glued to its count line; it must not become a stem.
        let aff = "SET UTF-8\nSFX a Y 1\nSFX a   0   y   [^y]\n";
        let d = AffixDict::from_hunspell(aff, "\u{feff}1\nkot/a\n", &[]).unwrap();
        assert_eq!(d.len(), 1);
        assert!(d.contains("kot") && d.contains("koty"));
    }

    #[test]
    fn unsupported_and_broken_input_is_an_error() {
        assert!(AffixDict::from_hunspell("FLAG utf-16\n", DIC, &[]).is_err());
        assert!(AffixDict::from_hunspell("FLAG num\nSFX x1 Y 1\n", DIC, &[]).is_err(), "a numeric flag that isn't a number");
        assert!(AffixDict::from_hunspell("FLAG long\nFORBIDDENWORD ABC\n", DIC, &[]).is_err(), "two flags where one is named");
        assert!(AffixDict::from_hunspell("COMPOUNDFLAG X\n", DIC, &[]).is_err());
        assert!(AffixDict::from_hunspell("SFX a Y 1\nSFX a 0 y [ab\n", DIC, &[]).is_err());
        assert!(AffixDict::from_bytes(b"not deflate").is_err());
        let ok = dict().to_bytes();
        assert!(AffixDict::from_bytes(&ok[..ok.len() / 2]).is_err());
        let garbage = miniz_oxide::deflate::compress_to_vec(b"WCAFF1\nWORDS\n\x09abc\n", 6);
        assert!(AffixDict::from_bytes(&garbage).is_err(), "prefix longer than the previous word");
        let empty = AffixDict::default();
        assert!(!empty.contains("kot") && !empty.check("Kot") && empty.forms("kot").is_empty());
    }

    proptest::proptest! {
        #[test]
        fn lookups_never_panic(s in "\\PC{0,40}", bytes in proptest::collection::vec(proptest::num::u8::ANY, 0..200)) {
            let d = dict();
            let _ = d.check(&s);
            let _ = d.forms(&s);
            let _ = AffixDict::from_bytes(&bytes);
            let _ = AffixDict::from_bytes(&miniz_oxide::deflate::compress_to_vec(&bytes, 1));
        }
    }
}
