//! Tinta Office: the spelling dictionaries the packages ship (`packaging/dictionaries`).
//!
//! The manifest is checked on every run. The dictionaries themselves are downloaded only by the
//! packaging scripts, so the test that reads them runs when `TINTA_DICTIONARIES_DIR` names a
//! folder `fetch.sh` filled:
//!
//! ```sh
//! packaging/dictionaries/fetch.sh /tmp/dicts
//! TINTA_DICTIONARIES_DIR=/tmp/dicts cargo test -p wordcraft-proof --test tinta_dictionaries -- --include-ignored
//! ```

use std::path::{Path, PathBuf};

use wordcraft_proof::{ProofLang, affix::AffixDict, hunspell};

/// One row of `sources.tsv`.
struct Source {
    file: String,
    url: String,
    sha256: String,
    licence: String,
}

fn manifest_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packaging/dictionaries")
}

fn sources() -> Vec<Source> {
    let text = std::fs::read_to_string(manifest_dir().join("sources.tsv")).expect("sources.tsv");
    text.lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let cells: Vec<&str> = l.split('\t').collect();
            assert_eq!(cells.len(), 5, "five columns: {l}");
            Source { file: cells[0].into(), url: cells[1].into(), sha256: cells[2].into(), licence: cells[4].into() }
        })
        .collect()
}

/// The dictionaries shipped, by the language each serves.
const SHIPPED: [(&str, ProofLang); 5] =
    [("es_ES", ProofLang::Es), ("ca", ProofLang::Ca), ("gl_ES", ProofLang::Gl), ("uk_UA", ProofLang::Uk), ("eu", ProofLang::Eu)];

#[test]
fn the_manifest_pins_every_file_and_licence() {
    let rows = sources();
    let readme = std::fs::read_to_string(manifest_dir().join("README.md")).expect("README.md");
    for s in &rows {
        // Pinned: a raw GitHub URL at a full commit hash, never a branch.
        let rest = s.url.strip_prefix("https://raw.githubusercontent.com/").unwrap_or_else(|| panic!("{}: not a raw GitHub URL", s.file));
        let commit = rest.split('/').nth(2).unwrap_or("");
        assert!(commit.len() == 40 && commit.bytes().all(|b| b.is_ascii_hexdigit()), "{}: not pinned to a commit", s.file);
        assert!(s.sha256.len() == 64 && s.sha256.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')), "{}: bad SHA-256", s.file);
        assert!(!s.file.contains("..") && !s.file.starts_with('/') && !s.file.contains('\\'), "{}: stays in the folder", s.file);
        let dictionary = s.file.ends_with(".aff") || s.file.ends_with(".dic");
        assert_eq!(dictionary, !s.file.starts_with("licences/"), "{}: dictionaries at the top, texts in licences/", s.file);
        assert_eq!(dictionary, s.licence != "-", "{}: each dictionary file names its licence", s.file);
    }
    for (name, _) in SHIPPED {
        for ext in ["aff", "dic"] {
            assert!(rows.iter().any(|s| s.file == format!("{name}.{ext}")), "{name}.{ext} is listed");
        }
        assert!(readme.contains(&format!("`{name}.aff`")), "the README describes {name}");
    }
    // Every licence a dictionary is under has its text in the pack.
    for (id, text) in [
        ("MPL-1.1", "MPL-1.1.txt"),
        ("GPL-2.0", "GPL-2.0.txt"),
        ("GPL-3.0", "GPL-3.0.txt"),
        ("LGPL-2.1", "LGPL-2.1.txt"),
        ("LGPL-3.0", "LGPL-3.0.txt"),
    ] {
        if rows.iter().any(|s| s.licence.contains(id)) {
            assert!(rows.iter().any(|s| s.file == format!("licences/{text}")), "{text} ships with the {id} dictionaries");
        }
    }
    let files: std::collections::HashSet<&str> = rows.iter().map(|s| s.file.as_str()).collect();
    assert_eq!(files.len(), rows.len(), "no file is listed twice");
}

/// Words a business letter in each language uses, and a misspelling of one of them.
const SAMPLES: [(ProofLang, &[&str], &str); 5] = [
    (ProofLang::Es, &["factura", "presupuesto", "señor", "envío", "también", "Estimado"], "facturra"),
    (ProofLang::Ca, &["factura", "pressupost", "l'empresa", "col·laboradora", "envieu-me", "també"], "facturra"),
    (ProofLang::Gl, &["factura", "orzamento", "envíame", "tamén", "por", "favor"], "facturra"),
    (ProofLang::Uk, &["рахунок", "договір", "п'ятниці", "п’ятниці", "мʼясо", "будь-який", "Україна", "заплати\u{301}ть"], "рахунак"),
    (ProofLang::Eu, &["faktura", "aurrekontua", "mesedez", "bidali", "eskolara", "etxea"], "fakturra"),
];

#[test]
#[ignore = "needs TINTA_DICTIONARIES_DIR (see the module docs)"]
fn the_shipped_dictionaries_load_and_check_real_words() {
    let Some(dir) = std::env::var_os("TINTA_DICTIONARIES_DIR").map(PathBuf::from) else {
        panic!("set TINTA_DICTIONARIES_DIR to a folder filled by packaging/dictionaries/fetch.sh");
    };
    for (name, _) in SHIPPED {
        let (aff, dic) = hunspell::find_in(&dir, &[name]).unwrap_or_else(|| panic!("{name} in {}", dir.display()));
        let started = std::time::Instant::now();
        let d: AffixDict = hunspell::load(&aff, &dic).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(d.len() > 50_000, "{name}: {} stems", d.len());
        assert!(started.elapsed().as_secs() < 20, "{name} loads in {:?}", started.elapsed());
    }
    wordcraft_proof::set_dictionaries_folder(Some(dir.clone()));
    for (lang, words, wrong) in SAMPLES {
        assert!(
            wordcraft_proof::installed_dictionaries().iter().any(|(l, p)| *l == lang && p.as_deref().is_some_and(|p| p.starts_with(&dir))),
            "{lang:?}"
        );
        for w in words {
            assert!(wordcraft_proof::is_correct_in(w, lang), "{lang:?}: {w}");
        }
        assert!(!wordcraft_proof::is_correct_in(wrong, lang), "{lang:?}: {wrong}");
        let right = words.first().copied().unwrap_or_default();
        assert!(
            wordcraft_proof::suggest_in(wrong, 8, lang).iter().any(|s| s == right),
            "{lang:?}: {wrong} → {:?}",
            wordcraft_proof::suggest_in(wrong, 8, lang)
        );
    }
}
