//! Tinta Office: Suggest my words for Tinta's dictionaries (File › Options › Proofing; the engine's
//! `review.suggestWords` and `review.suggestedWords`). Off by default, with the GDPR notice beside
//! it: turning it on is the consent, so there is no pop-up. "Send them automatically"
//! (`review.suggestWordsAuto`) is a second consent with its own notice, shown while the first is on.

use egui::{RichText, Ui, vec2};
use serde_json::{Value, json};

use super::{note, section};
use crate::WordApp;

/// The option, its notice, and while it is on the listed words with Send by email, Copy and Clear.
pub(super) fn pane(app: &mut WordApp, ui: &mut Ui) {
    section(ui, "Suggest my words");
    let mut on = app.session.prefs.tinta_suggest.enabled;
    if ui.checkbox(&mut on, tl!("Suggest my words for Tinta's dictionaries")).changed() {
        run(app, "review.suggestWords", json!({"value": on}));
    }
    note(ui, NOTICE_WHAT);
    note(ui, NOTICE_RIGHTS);
    if app.session.prefs.tinta_suggest.forget_pending.is_some() {
        note(ui, PENDING);
    }
    if !app.session.prefs.tinta_suggest.enabled {
        return;
    }
    let mut auto = app.session.prefs.tinta_suggest.auto;
    if ui.checkbox(&mut auto, tl!(AUTO)).changed() {
        run(app, "review.suggestWordsAuto", json!({"value": auto}));
    }
    note(ui, NOTICE_AUTO);
    let words = app.session.prefs.tinta_suggest.words.clone();
    if words.is_empty() {
        note(ui, "No words listed yet.");
        return;
    }
    egui::ScrollArea::vertical().id_salt("options_tinta_suggest").max_height(140.0).show(ui, |ui| {
        for w in &words {
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(vec2(220.0, 20.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.set_min_width(220.0);
                    let line = if w.sent { format!("{}  ·  {}  ·  {}", w.word, w.lang, tl!(SENT)) } else { format!("{}  ·  {}", w.word, w.lang) };
                    ui.add(egui::Label::new(line).truncate());
                });
                if ui.small_button(tl!("Remove")).clicked() {
                    run(app, "review.suggestedWords", json!({"remove": [w.word]}));
                }
            });
        }
    });
    let Some(state) = run(app, "review.suggestedWords", json!({})) else { return };
    let in_mail = state["inMail"].as_u64().unwrap_or(0);
    ui.horizontal(|ui| {
        if ui.button(tl!("Send by email")).clicked()
            && let Some(link) = state["mailto"].as_str()
        {
            app.canvas.open_url = Some(link.to_string());
        }
        if ui.button(tl!("Copy the list")).clicked()
            && let Some(text) = state["text"].as_str()
        {
            ui.ctx().copy_text(text.to_string());
        }
        if ui.button(tl!("Clear the list")).clicked() {
            run(app, "review.suggestedWords", json!({"clear": true}));
        }
    });
    if (in_mail as usize) < words.len() {
        let line = crate::i18n::fmt(
            tl!("The email holds {n} of {total} words; send the rest next time, or copy the list."),
            &[("n", &in_mail.to_string()), ("total", &words.len().to_string())],
        );
        ui.label(RichText::new(line).small().weak());
    }
}

/// What is listed and sent, and that nothing goes by itself without the second consent.
const NOTICE_WHAT: &str = "Off by default. While it is on, the words you add to the dictionary are listed here with their language, on this computer only. Unless you also turn on sending them automatically, nothing is sent by itself: you review the list and email it to hola@tintaoffice.com.";

/// The second consent and its notice (tintaoffice.com's privacy policy repeats it).
const AUTO: &str = "Send them automatically";
const NOTICE_AUTO: &str = "Off by default too. While it is on, the listed words in Spanish, Catalan, Galician, Basque and Ukrainian go once a day to report.tintaoffice.com (EU) with a random installation number; the server keeps only a fingerprint of it, never your IP address. A word joins Tinta's dictionaries once 5 installations (10 for Spanish) added it and Wiktionary lists it. Turning this off removes your words from the server's counts.";
/// Beside a word automatic sending sent.
const SENT: &str = "sent";
/// While the server hasn't confirmed a withdrawal yet.
const PENDING: &str = "Removing your words from the server's counts…";

/// Why, how long and how to withdraw (the GDPR notice; tintaoffice.com's privacy policy repeats it).
const NOTICE_RIGHTS: &str = "Only these words and their language are sent, never your documents; words with digits, @ or a capital inside are left out. Tinta Office uses them only to improve its dictionaries and deletes them within 12 months; your email address is used only to reply. To withdraw, turn this off (the list is deleted) and ask hola@tintaoffice.com to delete what you sent.";

fn run(app: &mut WordApp, id: &str, params: Value) -> Option<Value> {
    match app.run(id, params) {
        Ok(v) => Some(v),
        Err(e) => {
            app.status(e);
            None
        }
    }
}

#[cfg(test)]
pub(super) const STRINGS: [&str; 13] = [
    "Suggest my words",
    "Suggest my words for Tinta's dictionaries",
    NOTICE_WHAT,
    NOTICE_RIGHTS,
    AUTO,
    NOTICE_AUTO,
    SENT,
    PENDING,
    "No words listed yet.",
    "Send by email",
    "Copy the list",
    "Clear the list",
    "The email holds {n} of {total} words; send the rest next time, or copy the list.",
];

#[cfg(test)]
mod tests {
    use egui_kittest::kittest::Queryable;
    use serde_json::json;

    use super::STRINGS;
    use crate::{Services, UiState, WordApp};

    fn harness() -> egui_kittest::Harness<'static, WordApp> {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.session.doc.styles.default_chr.lang = Some("es-ES".into());
        a.run("file.options", json!({"pane": "proofing"})).unwrap();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 2400.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                let ctx = ui.ctx().clone();
                app.logic(&ctx);
                app.ui(ui);
            },
            a,
        );
        h.run_steps(4);
        h
    }

    /// Off by default with the notice beside it; ticking it lists added words, and Send by email
    /// opens a mailto link to Tinta Office with them.
    #[test]
    fn the_option_lists_words_and_sends_them_by_email() {
        let mut h = harness();
        assert!(!h.state().session.prefs.tinta_suggest.enabled);
        let _ = h.get_by_label_contains("Off by default.");
        assert!(h.query_by_label("Send by email").is_none(), "no list while off");
        h.get_by_label("Suggest my words for Tinta's dictionaries").click();
        h.run_steps(2);
        assert!(h.state().session.prefs.tinta_suggest.enabled);
        let _ = h.get_by_label("No words listed yet.");
        h.state_mut().run("file.options", json!({"addWords": ["tintaventana"]})).unwrap();
        h.run_steps(2);
        let _ = h.get_by_label("tintaventana  ·  es");
        h.get_by_label("Send by email").click();
        h.run_steps(1);
        // The app hands the link to the system (the default mail client) the same frame.
        let link = h
            .output()
            .platform_output
            .commands
            .iter()
            .find_map(|c| if let egui::OutputCommand::OpenUrl(u) = c { Some(u.url.clone()) } else { None })
            .unwrap_or_default();
        assert!(link.starts_with("mailto:hola@tintaoffice.com?") && link.contains("es%09tintaventana"), "{link}");
        h.get_by_label("Clear the list").click();
        h.run_steps(2);
        assert!(h.state().session.prefs.tinta_suggest.words.is_empty());
        h.get_by_label("Suggest my words for Tinta's dictionaries").click();
        h.run_steps(2);
        assert!(!h.state().session.prefs.tinta_suggest.enabled);
    }

    /// "Send them automatically" shows only while the option is on, with its own notice; a sent
    /// word is marked, and turning it off waits for the server to forget the installation's words.
    #[test]
    fn automatic_sending_is_a_second_checkbox() {
        let mut h = harness();
        assert!(h.query_by_label("Send them automatically").is_none(), "hidden while the option is off");
        h.get_by_label("Suggest my words for Tinta's dictionaries").click();
        h.run_steps(2);
        let _ = h.get_by_label_contains("Off by default too.");
        assert!(!h.state().session.prefs.tinta_suggest.auto);
        h.get_by_label("Send them automatically").click();
        h.run_steps(2);
        assert!(h.state().session.prefs.tinta_suggest.auto);
        h.state_mut().run("file.options", json!({"addWords": ["tintaenviada"]})).unwrap();
        let p = &mut h.state_mut().session.prefs.tinta_suggest;
        let sent = wordcraft_engine::tinta_suggest::unsent(p);
        wordcraft_engine::tinta_suggest::mark_sent(p, &sent);
        p.install_id = "0f8fad5b-d9cb-469f-a165-70867728950e".into();
        h.run_steps(2);
        let _ = h.get_by_label("tintaenviada  ·  es  ·  sent");
        h.get_by_label("Send them automatically").click();
        h.run_steps(2);
        assert!(!h.state().session.prefs.tinta_suggest.auto);
        let _ = h.get_by_label("Removing your words from the server's counts…");
    }

    /// The option and its list come back after a restart (ui.json).
    #[test]
    fn the_option_and_its_words_survive_a_restart() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        a.run("review.suggestWords", json!({"value": true})).unwrap();
        a.run("file.options", json!({"addWords": ["tintareinicio"]})).unwrap();
        let saved = serde_json::to_vec(&a.prefs()).unwrap();
        let mut b = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), Services::default());
        b.apply_prefs(UiState::from_json(&saved).unwrap());
        assert!(b.session.prefs.tinta_suggest.enabled);
        assert_eq!(b.session.prefs.tinta_suggest.words.iter().map(|w| w.word.as_str()).collect::<Vec<_>>(), ["tintareinicio"]);
    }

    /// Every string of the option, its notice and its buttons is translated in all 29 languages.
    #[test]
    fn every_string_is_translated() {
        let rows = include_str!("../i18n/tinta.tsv");
        for s in STRINGS {
            let langs =
                rows.lines().filter(|l| l.split('\t').nth(1) == Some(s) && l.split('\t').nth(2).is_some_and(|t| !t.trim().is_empty())).count();
            assert_eq!(langs, 29, "{s:?}");
        }
    }
}
