//! Tinta Office: the status bar's "Aptos not installed" notice ([`wordcraft_engine::tinta_fonts`]).
//! It shows while the document asks for an Aptos family the computer doesn't have; a click opens
//! Microsoft's free download, and once Aptos is installed the notice goes away by itself.

use std::cell::RefCell;

use crate::WordApp;

/// The status bar's label.
pub(crate) const LABEL: &str = "Aptos not installed";
/// Its tooltip.
pub(crate) const TIP: &str = "This document uses Aptos, Word's default font, which isn't installed. Tinta Office can't include it, so another font stands in and lines and pages may break differently from Word. Click to download Aptos free from Microsoft, then restart Tinta Texto.";

thread_local! {
    /// The families last found missing, by document revision and identity: the scan runs once
    /// per change, not every frame.
    static CACHE: RefCell<Option<((u64, usize), Vec<String>)>> = const { RefCell::new(None) };
}

/// The Aptos families the open document asks for and the computer lacks (`has_family` says what
/// is installed). Only for a document with a file: a new one is set in Aptos by default, like
/// Word's, and the notice would show to everyone without it.
pub(crate) fn missing_with(app: &WordApp, has_family: impl Fn(&str) -> bool) -> Vec<String> {
    if app.session.path.is_none() {
        return Vec::new();
    }
    let key = (app.session.rev(), std::ptr::from_ref(&app.session.doc) as usize);
    if let Some(hit) = CACHE.with_borrow(|c| c.as_ref().filter(|(k, _)| *k == key).map(|(_, v)| v.clone())) {
        return hit;
    }
    let missing: Vec<String> = wordcraft_engine::tinta_fonts::aptos_families(&app.session.doc).into_iter().filter(|f| !has_family(f)).collect();
    CACHE.with_borrow_mut(|c| *c = Some((key, missing.clone())));
    missing
}

/// [`missing_with`] the installed fonts.
pub(crate) fn missing(app: &WordApp) -> Vec<String> {
    missing_with(app, |f| wordcraft_fonts::FontDb::global().has_family(f))
}

/// Open Microsoft's download page.
pub(crate) fn open_download(app: &mut WordApp) {
    app.canvas.open_url = Some(wordcraft_engine::tinta_fonts::APTOS_DOWNLOAD.to_string());
}

#[cfg(test)]
pub(crate) const STRINGS: [&str; 2] = [LABEL, TIP];

#[cfg(test)]
mod tests {
    use egui_kittest::kittest::Queryable;

    use super::*;
    use crate::Services;

    fn app_with_aptos() -> WordApp {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.session.doc.styles.default_chr.font = Some("Aptos".into());
        a.session.path = Some(std::env::temp_dir().join("informe.docx"));
        a
    }

    #[test]
    fn only_missing_aptos_families_are_reported() {
        let a = app_with_aptos();
        // The default heading styles ask for Aptos Display.
        assert_eq!(missing_with(&a, |_| false), ["Aptos", "Aptos Display"]);
        assert_eq!(missing_with(&a, |_| true), ["Aptos", "Aptos Display"], "kept until the document changes (installing Aptos needs a restart)");
        let b = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        assert!(missing_with(&b, |_| false).is_empty(), "a new document (set in Aptos too) shows nothing");
    }

    /// With Aptos missing (the test machine has no Aptos), the status bar shows the notice and a
    /// click opens Microsoft's download page.
    #[test]
    fn the_status_bar_links_to_the_download() {
        if wordcraft_fonts::FontDb::global().has_family("Aptos") {
            eprintln!("skipped: Aptos is installed here");
            return;
        }
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            app_with_aptos(),
        );
        h.run_steps(3);
        h.get_by_label(LABEL).click();
        h.run_steps(1);
        let link = h
            .output()
            .platform_output
            .commands
            .iter()
            .find_map(|c| if let egui::OutputCommand::OpenUrl(u) = c { Some(u.url.clone()) } else { None })
            .unwrap_or_default();
        assert_eq!(link, wordcraft_engine::tinta_fonts::APTOS_DOWNLOAD);
    }

    #[test]
    fn every_string_is_translated() {
        let rows = include_str!("i18n/tinta.tsv");
        for s in STRINGS {
            let langs =
                rows.lines().filter(|l| l.split('\t').nth(1) == Some(s) && l.split('\t').nth(2).is_some_and(|t| !t.trim().is_empty())).count();
            assert_eq!(langs, 30, "{s:?}");
        }
    }
}
