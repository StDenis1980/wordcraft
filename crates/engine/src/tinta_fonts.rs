//! Tinta Office: documents set in Aptos (roadmap priority 5, owner's choice 2026-11-01).
//!
//! Aptos, Word's default font since 2024, is not an open font: no open face has its widths, so
//! without it lines and pages break differently from Word. Tinta never ships it. When a document
//! asks for an Aptos family that isn't installed, the status bar says so and links to Microsoft's
//! free download ([`APTOS_DOWNLOAD`]); once the person installs it, Texto lays the text out with
//! it. This module finds the Aptos families a document asks for ([`aptos_families`]); the front
//! end checks which are installed, for documents that come from a file (a new document is set in
//! Aptos too, as in Word, and would show the notice to everyone without it).

use wordcraft_doc::{CharProps, Document, StoryRef};

/// Microsoft's download page for the Aptos fonts.
pub const APTOS_DOWNLOAD: &str = "https://www.microsoft.com/download/details.aspx?id=106087";

/// Paragraphs looked at; a longer document is judged by its styles and first paragraphs.
const MAX_PARAGRAPHS: usize = 20_000;
/// Distinct families reported.
const MAX_FAMILIES: usize = 8;

/// Is `family` one of the Aptos families (`Aptos`, `Aptos Display`, `Aptos Narrow`, `Aptos Serif`…)?
pub fn is_aptos(family: &str) -> bool {
    let f = family.trim();
    f.get(..5).is_some_and(|p| p.eq_ignore_ascii_case("aptos")) && f.get(5..).is_some_and(|rest| rest.chars().next().is_none_or(|c| c == ' '))
}

fn add(out: &mut Vec<String>, props: &CharProps) {
    if let Some(f) = props.font.as_deref().map(str::trim)
        && is_aptos(f)
        && out.len() < MAX_FAMILIES
        && !out.iter().any(|o| o.eq_ignore_ascii_case(f))
    {
        out.push(f.to_string());
    }
}

/// The Aptos families `doc` asks for, in its default font, its styles and the body's runs and
/// paragraph marks, sorted, at most [`MAX_FAMILIES`].
pub fn aptos_families(doc: &Document) -> Vec<String> {
    let mut out = Vec::new();
    add(&mut out, &doc.styles.default_chr);
    for s in &doc.styles.styles {
        add(&mut out, &s.chr);
    }
    for path in doc.para_paths(StoryRef::Body).iter().take(MAX_PARAGRAPHS) {
        if let Some(p) = doc.para(StoryRef::Body, path) {
            add(&mut out, &p.mark);
            for r in &p.runs {
                add(&mut out, &r.props);
            }
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aptos_names() {
        for f in ["Aptos", "aptos", " Aptos Display", "Aptos Narrow", "Aptos Serif"] {
            assert!(is_aptos(f), "{f}");
        }
        for f in ["Aptosx", "Calibri", "Apto", "", "Ápto", "AptosMono"] {
            assert!(!is_aptos(f), "{f}");
        }
    }

    #[test]
    fn families_come_from_defaults_styles_and_runs() {
        // A new document is set in Aptos already (Word's default); start from one without it.
        let plain = || {
            let mut doc = Document::new();
            doc.styles.default_chr.font = Some("Calibri".into());
            doc.styles.styles.iter_mut().for_each(|s| s.chr.font = None);
            doc
        };
        let mut doc = plain();
        assert!(aptos_families(&doc).is_empty());
        doc.styles.default_chr.font = Some("Aptos".into());
        if let Some(s) = doc.styles.styles.first_mut() {
            s.chr.font = Some("Aptos Display".into());
        }
        doc.styles
            .styles
            .push(wordcraft_doc::styles::Style { chr: CharProps { font: Some("aptos".into()), ..Default::default() }, ..Default::default() });
        assert_eq!(aptos_families(&doc), ["Aptos", "Aptos Display"], "one entry per family, any case");
        let mut doc = plain();
        let path = doc.para_paths(StoryRef::Body).into_iter().next().unwrap();
        let p = doc.para_mut(StoryRef::Body, &path).unwrap();
        p.text = "Hola".into();
        p.runs = vec![wordcraft_doc::Run { len: 4, props: CharProps { font: Some("Aptos Narrow".into()), ..Default::default() } }];
        assert_eq!(aptos_families(&doc), ["Aptos Narrow"]);
    }
}
