use std::path::{Path, PathBuf};

use super::*;

/// A fresh temporary folder for one test.
fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("tinta-packs-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A shipped dictionaries folder with tiny Ukrainian and Spanish dictionaries, an accepted-words
/// list for each and a licence text.
fn shipped(root: &Path) -> PathBuf {
    let d = root.join("dictionaries");
    std::fs::create_dir_all(d.join("licences")).unwrap();
    std::fs::write(d.join("uk_UA.aff"), "SET UTF-8\n").unwrap();
    std::fs::write(d.join("uk_UA.dic"), "2\nкнига\nхата\n").unwrap();
    std::fs::write(d.join("es_ES.aff"), "SET UTF-8\n").unwrap();
    std::fs::write(d.join("es_ES.dic"), "2\ncasa\nlibro\n").unwrap();
    std::fs::write(d.join("tinta-words-uk.txt"), "# comment\nп’ятничка\n\nслово з пробілом\nфлаг/AB\n12\nп'ятничка\n").unwrap();
    std::fs::write(d.join("tinta-words-es.txt"), "# comment\ntintero\n").unwrap();
    std::fs::write(d.join("licences").join("MPL-1.1.txt"), "Mozilla Public License 1.1\n").unwrap();
    d
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_default()
}

#[test]
fn shipped_dictionaries_become_packs_with_patterns_words_and_licences() {
    let root = temp("write");
    let dict = shipped(&root);
    let packs = root.join("packs");
    let done = seed_into(&packs, &[dict]);
    assert_eq!(done.written, ["es", "uk"], "{done:?}");
    assert!(done.failed.is_empty(), "{done:?}");
    let uk = packs.join("uk");
    let toml = read(&uk.join("pack.toml"));
    assert!(toml.contains("locale = \"uk\"") && toml.contains("name = \"Українська\"") && toml.contains("licence = \"MPL-1.1 AND MIT\""), "{toml}");
    assert!(pack_signature(&uk).is_some(), "{toml}");
    assert_eq!(read(&uk.join("spelling.aff")), "SET UTF-8\n");
    // The accepted words follow the dictionary's own, composed, with a plain apostrophe, each
    // once; a phrase, a word with flags and a number are left out.
    assert_eq!(read(&uk.join("spelling.dic")), "2\nкнига\nхата\n\nп'ятничка\n");
    assert_eq!(read(&uk.join("hyphenation.pat")), UK_PAT);
    assert!(read(&uk.join("LICENCES").join("hyphenation.txt")).contains('%'), "the patterns' notice");
    assert_eq!(read(&uk.join("LICENCES").join("MPL-1.1.txt")), "Mozilla Public License 1.1\n");
    assert_eq!(read_ledger(&packs), ["es", "uk"]);
    assert!(!packs.join("ca").exists(), "no Catalan files shipped, no Catalan pack");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn packs_are_written_again_only_when_the_shipped_files_change() {
    let root = temp("again");
    let dict = shipped(&root);
    let packs = root.join("packs");
    assert_eq!(seed_into(&packs, std::slice::from_ref(&dict)).written.len(), 2);
    assert_eq!(seed_into(&packs, std::slice::from_ref(&dict)), Seeded::default(), "nothing changed");
    std::fs::write(dict.join("uk_UA.dic"), "3\nкнига\nхата\nсад\n").unwrap();
    assert_eq!(seed_into(&packs, std::slice::from_ref(&dict)).written, ["uk"]);
    assert!(read(&packs.join("uk").join("spelling.dic")).contains("сад"));
    assert!(!packs.join(".tinta-uk.new").exists() && !packs.join(".tinta-uk.old").exists(), "no staging left behind");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_persons_own_packs_and_removals_are_kept() {
    let root = temp("own");
    let dict = shipped(&root);
    let packs = root.join("packs");
    // A Spanish pack installed from the store: no `tinta` key.
    std::fs::create_dir_all(packs.join("es")).unwrap();
    std::fs::write(packs.join("es").join("pack.toml"), "locale = \"es\"\nlicence = \"MPL-2.0\"\n").unwrap();
    assert_eq!(seed_into(&packs, std::slice::from_ref(&dict)).written, ["uk"]);
    assert_eq!(read(&packs.join("es").join("pack.toml")), "locale = \"es\"\nlicence = \"MPL-2.0\"\n", "the store's pack is untouched");
    // Removed in Options › Language: it stays removed, also when the shipped files change.
    std::fs::remove_dir_all(packs.join("uk")).unwrap();
    std::fs::write(dict.join("uk_UA.dic"), "1\nкнига\n").unwrap();
    assert_eq!(seed_into(&packs, std::slice::from_ref(&dict)), Seeded::default());
    assert!(!packs.join("uk").exists());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn words_are_added_only_to_a_utf8_dictionary() {
    let root = temp("latin1");
    let dict = shipped(&root);
    std::fs::write(dict.join("es_ES.aff"), "SET ISO8859-1\n").unwrap();
    let packs = root.join("packs");
    seed_into(&packs, &[dict]);
    assert_eq!(read(&packs.join("es").join("spelling.dic")), "2\ncasa\nlibro\n");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_symbolic_link_or_a_missing_half_is_not_a_dictionary() {
    let root = temp("half");
    let d = root.join("dictionaries");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("gl_ES.dic"), "1\ncasa\n").unwrap();
    let packs = root.join("packs");
    assert_eq!(seed_into(&packs, &[d]), Seeded::default());
    assert!(!packs.join("gl").exists());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn accepted_words_skip_what_a_word_list_would_misread() {
    let words = accepted_words("# c\n  tintero  \nTintero\n12\n-\nuno dos\nflag/X\na\\b\n");
    assert_eq!(words, ["tintero", "Tintero"]);
    let long = "a".repeat(MAX_WORD_CHARS + 1);
    assert!(accepted_words(&long).is_empty());
}

#[test]
fn quoted_strings_are_valid_toml() {
    assert_eq!(quoted("Català (valencià)"), "\"Català (valencià)\"");
    assert_eq!(quoted("a\"b\\c\n"), "\"a\\\"b\\\\c\\u000A\"");
}

/// The packaged folder names the packages use (package.ps1, package.sh, the Flatpak and the macOS
/// bundle) are the ones looked in.
#[test]
fn the_shipped_folders_are_where_the_packages_put_them() {
    let dirs = bundled_dirs(Path::new("/opt/tinta/bin/wordcraft"));
    assert_eq!(
        dirs,
        [
            PathBuf::from("/opt/tinta/bin/dictionaries"),
            PathBuf::from("/opt/tinta/share/wordcraft/dictionaries"),
            PathBuf::from("/opt/tinta/Resources/dictionaries")
        ]
    );
}

/// `word` with a hyphen at each unrestricted break point in `lang`.
fn shown(word: &str, lang: wordcraft_proof::ProofLang) -> String {
    let (points, _) = wordcraft_proof::hyphen::word_breaks_in(word, lang);
    let mut s = String::new();
    for (i, c) in word.chars().enumerate() {
        if points.contains(&i) {
            s.push('-');
        }
        s.push(c);
    }
    s
}

/// The packs written are what craft-proof reads: Ukrainian words (with the accepted one) are
/// spelled right and others wrong, and the languages of Spain and Ukrainian break at their
/// syllables with the bundled patterns (the fork's own proofing tests before craft-proof).
#[test]
fn proofing_reads_the_written_packs() {
    use wordcraft_proof::ProofLang;
    let _g = crate::cmd::packs::TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = temp("proof");
    let dict = shipped(&root);
    for (file, word) in [("ca", "factura"), ("ca-valencia", "treballador"), ("gl_ES", "concello"), ("eu", "faktura")] {
        std::fs::write(dict.join(format!("{file}.aff")), "SET UTF-8\n").unwrap();
        std::fs::write(dict.join(format!("{file}.dic")), format!("1\n{word}\n")).unwrap();
    }
    let packs = root.join("packs");
    seed_into(&packs, &[dict]);
    wordcraft_proof::packs::set_packs_dir(Some(packs.clone()));
    let lang = |t: &str| ProofLang::from_tag(Some(t));
    let uk = lang("uk-UA");
    let correct = |w: &str, l| wordcraft_proof::is_correct_in(w, l);
    let spelling = [
        correct("книга", uk),
        correct("п'ятничка", uk),
        !correct("кныга", uk),
        // Spanish spelling is left out: an engine test sets a Dictionaries folder with its own
        // es_ES for the rest of the run, and that folder is searched first.
        correct("factura", lang("ca-ES")),
        correct("treballador", lang("ca-ES-valencia")),
        correct("concello", lang("gl-ES")),
        correct("faktura", lang("eu-ES")),
    ];
    let mut breaks = Vec::new();
    for (tag, words) in [
        ("es-ES", &["ca-mi-se-ta", "pre-su-pues-to", "cons-truc-ción", "ca-rre-te-ra", "Ayun-ta-mien-to", "de-sa-rro-llo"][..]),
        ("es-MX", &["pre-su-pues-to"][..]),
        ("ca-ES", &["fac-tu-ra", "pres-su-post", "col·le-gi", "Ge-ne-ra-li-tat", "pa-rai-gua", "car-re-te-ra"][..]),
        ("ca-ES-valencia", &["co-nei-xe-ment", "tre-ba-lla-dor"][..]),
        ("gl-ES", &["or-za-men-to", "Ga-li-za", "con-ce-llo"][..]),
        ("eu-ES", &["au-rre-kon-tua", "fak-tu-ra", "me-se-dez", "Eus-ka-di"][..]),
        ("uk-UA", &["під-при-єм-ство", "м'я-со", "п’я-тни-ця", "за-пла-ти-ти"][..]),
    ] {
        for want in words {
            breaks.push((tag, *want, shown(&want.replace('-', ""), lang(tag))));
        }
    }
    let installed: Vec<String> = wordcraft_proof::packs::installed().into_iter().map(|p| p.locale).collect();
    wordcraft_proof::packs::set_packs_dir(None);
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(installed, ["ca", "ca-valencia", "es", "eu", "gl", "uk"]);
    assert!(spelling.iter().all(|ok| *ok), "{spelling:?}");
    let wrong: Vec<_> = breaks.iter().filter(|(_, want, got)| want != got).collect();
    assert!(wrong.is_empty(), "{wrong:?}");
}
