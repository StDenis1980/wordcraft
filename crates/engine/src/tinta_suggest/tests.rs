use serde_json::{Value, json};
use wordcraft_doc::Document;

use super::{MAX_MAILTO_BYTES, MAX_WORDS, SuggestedWord, eligible, lang_code, mailto};
use crate::Session;

fn spanish() -> Session {
    let mut s = Session::new(Document::new());
    s.doc.styles.default_chr.lang = Some("es-ES".into());
    s
}

fn listed(s: &Session) -> Vec<(String, String)> {
    s.prefs.tinta_suggest.words.iter().map(|w| (w.lang.clone(), w.word.clone())).collect()
}

fn pair(lang: &str, word: &str) -> (String, String) {
    (lang.to_string(), word.to_string())
}

#[test]
fn the_option_is_off_by_default_and_lists_nothing() {
    assert!(!crate::Prefs::default().tinta_suggest.enabled);
    let mut s = spanish();
    s.run("review.addToDictionary", &json!({"word": "tintaapagada"})).unwrap();
    s.run("file.options", &json!({"addWords": ["tintaapagada2"]})).unwrap();
    assert!(listed(&s).is_empty());
}

#[test]
fn while_on_added_words_are_listed_once_with_their_language() {
    let mut s = spanish();
    let r = s.run("review.suggestWords", &json!({"value": true})).unwrap();
    assert_eq!(r["value"], json!(true));
    s.run("review.addToDictionary", &json!({"word": "tintapalabra"})).unwrap();
    s.run("review.addToDictionary", &json!({"word": "TintaPalabraX"})).unwrap();
    s.run("file.options", &json!({"addWords": ["tintaotra", "tintapalabra"]})).unwrap();
    // The same word again (any case) is listed once; a capital inside the word is not listed.
    s.run("review.addToDictionary", &json!({"word": "Tintapalabra"})).unwrap();
    assert_eq!(listed(&s), vec![pair("es", "tintapalabra"), pair("es", "tintaotra")]);
    // Ukrainian text lists under `uk`.
    s.doc.styles.default_chr.lang = Some("uk-UA".into());
    s.run("review.addToDictionary", &json!({"word": "п'ятничка"})).unwrap();
    assert_eq!(listed(&s).last(), Some(&pair("uk", "п'ятничка")));
    let r = s.run("review.suggestedWords", &json!({})).unwrap();
    assert_eq!(r["words"].as_array().map(Vec::len), Some(3));
    assert_eq!(r["to"], json!("hola@tintaoffice.com"));
}

#[test]
fn words_that_may_be_personal_data_are_never_listed() {
    for w in ["McDonald", "iPhone", "X1234567L", "B12345678", "ana@correo", "https://x.es", "c/ Mayor", "12", "-", "'", "ES12 3456", ""] {
        assert!(!eligible(w), "{w:?}");
    }
    for w in ["sagunto", "Sagunto", "café", "l·lus", "d'acord", "п'ятниця", "ґанок", "euskara", "bien-estar"] {
        assert!(eligible(w), "{w:?}");
    }
    assert!(!eligible(&"a".repeat(wordcraft_proof::MAX_CUSTOM_WORD_CHARS + 1)));
}

#[test]
fn turning_it_off_deletes_the_list_and_removing_a_word_forgets_it() {
    let mut s = spanish();
    s.run("review.suggestWords", &json!({})).unwrap();
    assert!(s.prefs.tinta_suggest.enabled, "omitted value toggles");
    s.run("file.options", &json!({"addWords": ["tintauno", "tintados"]})).unwrap();
    s.run("file.options", &json!({"removeWords": ["TINTAUNO"]})).unwrap();
    assert_eq!(listed(&s), vec![pair("es", "tintados")]);
    s.run("review.suggestedWords", &json!({"remove": ["tintados"]})).unwrap();
    assert!(listed(&s).is_empty());
    s.run("file.options", &json!({"addWords": ["tintatres"]})).unwrap();
    s.run("review.suggestedWords", &json!({"clear": true})).unwrap();
    assert!(listed(&s).is_empty());
    s.run("file.options", &json!({"addWords": ["tintacuatro"]})).unwrap();
    let r = s.run("review.suggestWords", &json!({"value": false})).unwrap();
    assert_eq!(r["value"], json!(false));
    assert!(listed(&s).is_empty());
    s.run("file.options", &json!({"addWords": ["tintacinco"]})).unwrap();
    assert!(listed(&s).is_empty(), "nothing is listed while off");
}

#[test]
fn the_list_is_capped() {
    let mut s = spanish();
    s.run("review.suggestWords", &json!({"value": true})).unwrap();
    for i in 0..MAX_WORDS + 5 {
        let w: String = format!("{i:04}").chars().map(|c| char::from(b'a' + c.to_digit(10).unwrap_or(0) as u8)).collect();
        super::record(&mut s, &format!("tinta{w}"), "es");
    }
    assert_eq!(s.prefs.tinta_suggest.words.len(), MAX_WORDS);
}

fn decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            out.push(u8::from_str_radix(&s[i + 1..i + 3], 16).unwrap());
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).unwrap()
}

#[test]
fn the_email_holds_the_words_that_fit() {
    let words = vec![
        SuggestedWord { word: "tinta".into(), lang: "es".into(), ..Default::default() },
        SuggestedWord { word: "п'ятниця".into(), lang: "uk".into(), ..Default::default() },
    ];
    let (link, n) = mailto(&words);
    assert_eq!(n, 2);
    let (head, body) = link.split_once("&body=").unwrap();
    assert!(head.starts_with("mailto:hola@tintaoffice.com?subject="), "{head}");
    assert!(!body.contains(['&', '?', '#', ' ', '\n', '\t']), "{body}");
    let body = decode(body);
    assert!(body.ends_with("es\ttinta\nuk\tп'ятниця\n"), "{body}");
    assert!(super::text(&words).starts_with("To: hola@tintaoffice.com\n"));

    let many: Vec<SuggestedWord> =
        (0..500).map(|i| SuggestedWord { word: format!("palabra{}", "a".repeat(i % 30)), lang: "es".into(), ..Default::default() }).collect();
    let (link, n) = mailto(&many);
    assert!(link.len() <= MAX_MAILTO_BYTES, "{}", link.len());
    assert!(n > 10 && n < many.len(), "{n}");
    assert_eq!(decode(link.split_once("&body=").unwrap().1).lines().count(), n + 1);
}

#[test]
fn language_codes() {
    assert_eq!(lang_code(Some("es-MX")), "es");
    assert_eq!(lang_code(Some("ca-ES")), "ca");
    assert_eq!(lang_code(Some("fr-FR")), "fr");
    assert_eq!(lang_code(Some("x-junk-12")), "und");
    assert_eq!(lang_code(None), "en");
}

#[test]
fn saved_lists_are_read_leniently_and_filtered() {
    let prefs: crate::Prefs = serde_json::from_value(json!({
        "checkSpelling": false,
        "tintaSuggest": {"enabled": true, "words": [{"word": "tinta", "lang": "es"}, {"word": "McDonald", "lang": "en"}, {"word": "tinta", "lang": "es"}, {"word": "dato", "lang": "es-ES"}]}
    }))
    .unwrap();
    assert!(!prefs.check_spelling);
    let w: Vec<_> = prefs.tinta_suggest.words.iter().map(|w| (w.lang.as_str(), w.word.as_str())).collect();
    assert_eq!(w, vec![("es", "tinta"), ("es", "dato")]);
    // A damaged value is off with no words, and the other preferences survive.
    for bad in [json!(5), json!({"enabled": "yes"}), json!({"enabled": true, "words": 3})] {
        let prefs: crate::Prefs = serde_json::from_value(json!({"checkSpelling": false, "tintaSuggest": bad})).unwrap();
        assert!(!prefs.check_spelling);
        assert_eq!(prefs.tinta_suggest, Default::default());
    }
    // Saved while off: no words come back.
    let prefs: crate::Prefs =
        serde_json::from_value(json!({"tintaSuggest": {"enabled": false, "words": [{"word": "tinta", "lang": "es"}]}})).unwrap();
    assert!(prefs.tinta_suggest.words.is_empty());
    let round: crate::Prefs = serde_json::from_value(serde_json::to_value(spanish_with_words().prefs).unwrap()).unwrap();
    assert_eq!(round.tinta_suggest, spanish_with_words().prefs.tinta_suggest);
}

fn spanish_with_words() -> Session {
    let mut s = spanish();
    s.prefs.tinta_suggest.enabled = true;
    super::record(&mut s, "tintaguardada", "es");
    s
}

#[test]
fn bad_params_are_errors() {
    let mut s = spanish();
    for (id, v) in [
        ("review.suggestWords", json!({"value": "yes"})),
        ("review.suggestedWords", json!({"remove": "tinta"})),
        ("review.suggestedWords", json!({"remove": [1]})),
        ("review.suggestedWords", json!({"clear": 1})),
        ("review.suggestedWords", json!({"remove": vec![Value::from("a"); 1001]})),
        ("review.suggestWordsAuto", json!({"value": 1})),
        ("review.suggestWordsAuto", json!({"value": true})),
    ] {
        assert!(s.run(id, &v).is_err(), "{id} {v}");
    }
}

const ID: &str = "0f8fad5b-d9cb-469f-a165-70867728950e";

/// Automatic sending is a second consent: off by default, only while the option is on, and only
/// for the languages the server checks.
#[test]
fn automatic_sending_is_a_second_consent() {
    let mut s = spanish_with_words();
    assert!(!s.prefs.tinta_suggest.auto);
    assert!(super::unsent(&s.prefs.tinta_suggest).is_empty(), "nothing goes by itself while it is off");
    let r = s.run("review.suggestWordsAuto", &json!({"value": true})).unwrap();
    assert_eq!(r["auto"], json!(true));
    super::record(&mut s, "motclau", "fr");
    super::record(&mut s, "п'ятничка", "uk");
    let unsent = super::unsent(&s.prefs.tinta_suggest);
    let w: Vec<_> = unsent.iter().map(|w| (w.lang.as_str(), w.word.as_str())).collect();
    assert_eq!(w, vec![("es", "tintaguardada"), ("uk", "п'ятничка")], "French is only sent by email");
    super::mark_sent(&mut s.prefs.tinta_suggest, &unsent);
    assert!(super::unsent(&s.prefs.tinta_suggest).is_empty());
    let r = s.run("review.suggestedWords", &json!({})).unwrap();
    assert_eq!(r["words"][0]["sent"], json!(true));
    assert_eq!(r["words"][1]["sent"], json!(false));
    // Needs the option on.
    let mut off = spanish();
    assert!(off.run("review.suggestWordsAuto", &json!({"value": true})).is_err());
    assert!(!off.prefs.tinta_suggest.auto);
}

/// Turning automatic sending or the option off asks the server to forget the installation's
/// words; later words go under a new id.
#[test]
fn withdrawing_asks_the_server_to_forget() {
    for off in [("review.suggestWordsAuto", json!({"value": false})), ("review.suggestWords", json!({"value": false}))] {
        let mut s = spanish_with_words();
        s.run("review.suggestWordsAuto", &json!({})).unwrap();
        s.prefs.tinta_suggest.install_id = ID.into();
        let r = s.run(off.0, &off.1).unwrap();
        assert_eq!(r["auto"], json!(false), "{}", off.0);
        assert_eq!(r["forgetPending"], json!(true), "{}", off.0);
        assert!(s.prefs.tinta_suggest.install_id.is_empty());
        assert_eq!(s.prefs.tinta_suggest.forget_pending.as_deref(), Some(ID));
    }
}

#[test]
fn saved_automatic_state_is_read_leniently() {
    let read = |v: Value| serde_json::from_value::<crate::Prefs>(json!({"tintaSuggest": v})).unwrap().tinta_suggest;
    let p = read(json!({"enabled": true, "auto": true, "install_id": ID, "words": [{"word": "tinta", "lang": "es", "sent": true}]}));
    assert!(p.auto && p.install_id == ID && p.words[0].sent);
    // Auto without the option is off, and its id goes to the pending deletion.
    let p = read(json!({"enabled": false, "auto": true, "install_id": ID}));
    assert!(!p.auto && p.install_id.is_empty());
    assert_eq!(p.forget_pending.as_deref(), Some(ID));
    let p = read(json!({"enabled": true, "auto": true, "install_id": "../../x", "forget_pending": "nope"}));
    assert!(p.auto && p.install_id.is_empty() && p.forget_pending.is_none());
}
