//! Tinta Office: the first-start question about automatic error reports and their settings in
//! File › Options › General ([`wordcraft_engine::tinta_reports`] keeps the choice; the desktop app
//! sends). Both show only in builds that can send reports: the desktop app sets
//! [`crate::Services::error_reports_dir`] then.

use egui::{RichText, Ui};
use serde_json::json;

use crate::WordApp;

pub(crate) const TITLE: &str = "Send error reports automatically?";
pub(crate) const WHAT: &str = "When the app crashes or stops responding, it can send a report so the problem gets fixed without you writing to us. A report holds the app version, the system, the language, where in the code the error happened and the last lines of the technical log. Never the text of your documents, file names, folders, e-mail addresses or passwords.";
pub(crate) const RIGHTS: &str = "Reports go to Tinta Office (report.tintaoffice.com, stored in the EU) and are deleted after 90 days. You can change this at any time in File › Options › General and delete your reports from the server there. Details: tintaoffice.com/aviso-legal";
pub(crate) const YES: &str = "Yes, send reports";
pub(crate) const NO: &str = "No";
pub(crate) const SECTION: &str = "Error reports";
pub(crate) const CHECKBOX: &str = "Send error reports automatically";
pub(crate) const ID: &str = "Installation id: {id}";
pub(crate) const FORGET: &str = "Delete my reports from the server";
pub(crate) const PENDING: &str = "Deletion requested: it completes the next time the app is online.";
pub(crate) const FOLDER: &str = "Open the reports folder";

fn available(app: &WordApp) -> bool {
    app.services.error_reports_dir.is_some()
}

fn set(app: &mut WordApp, params: serde_json::Value) {
    if let Err(e) = app.run("tools.errorReports", params) {
        app.status(e);
    }
}

/// The question, over everything, until it is answered once.
pub(crate) fn first_start(app: &mut WordApp, ctx: &egui::Context) {
    if !available(app) || app.session.prefs.tinta_reports.consent.is_some() {
        return;
    }
    egui::Modal::new(egui::Id::unique("tinta.errorReports")).show(ctx, |ui| {
        ui.set_max_width(460.0);
        ui.label(RichText::new(tl!(TITLE)).strong().size(17.0));
        ui.add_space(8.0);
        ui.label(tl!(WHAT));
        ui.add_space(6.0);
        ui.label(RichText::new(tl!(RIGHTS)).small().weak());
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui.button(tl!(YES)).clicked() {
                set(app, json!({"value": true}));
            }
            if ui.button(tl!(NO)).clicked() {
                set(app, json!({"value": false}));
            }
        });
    });
}

/// The section in File › Options › General.
pub(crate) fn options(app: &mut WordApp, ui: &mut Ui, section: fn(&mut Ui, &str)) {
    if !available(app) {
        return;
    }
    ui.add_space(8.0);
    section(ui, SECTION);
    let mut on = app.session.prefs.tinta_reports.consent == Some(true);
    if ui.checkbox(&mut on, tl!(CHECKBOX)).changed() {
        set(app, json!({"value": on}));
    }
    ui.label(RichText::new(tl!(WHAT)).small().weak());
    ui.label(RichText::new(tl!(RIGHTS)).small().weak());
    let p = app.session.prefs.tinta_reports.clone();
    if !p.install_id.is_empty() {
        ui.label(RichText::new(crate::i18n::fmt(tl!(ID), &[("id", &p.install_id)])).small().monospace());
    }
    ui.horizontal(|ui| {
        if ui.add_enabled(!p.install_id.is_empty(), egui::Button::new(tl!(FORGET))).clicked() {
            set(app, json!({"forget": true}));
        }
        if let Some(dir) = app.services.error_reports_dir.clone()
            && ui.button(tl!(FOLDER)).clicked()
        {
            let _ = std::fs::create_dir_all(&dir);
            app.canvas.open_url = Some(format!("file://{}", dir.display()));
        }
    });
    if p.forget_pending.is_some() {
        ui.label(RichText::new(tl!(PENDING)).small().weak());
    }
}

#[cfg(test)]
pub(crate) const STRINGS: [&str; 11] = [TITLE, WHAT, RIGHTS, YES, NO, SECTION, CHECKBOX, ID, FORGET, PENDING, FOLDER];

#[cfg(test)]
mod tests {
    use egui_kittest::kittest::Queryable;

    use super::*;
    use crate::{Services, WordApp};

    fn harness(dir: Option<std::path::PathBuf>) -> egui_kittest::Harness<'static, WordApp> {
        let services = Services { error_reports_dir: dir, ..Default::default() };
        let a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), services);
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1440.0, 1000.0)).build_ui_state(
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

    /// A build that can send asks once; the answer closes the question for good.
    #[test]
    fn the_first_start_asks_once() {
        let mut h = harness(Some(std::env::temp_dir().join("tinta-reports-test")));
        let _ = h.get_by_label(TITLE);
        h.get_by_label(YES).click();
        h.run_steps(3);
        assert_eq!(h.state().session.prefs.tinta_reports.consent, Some(true));
        assert!(h.query_by_label(TITLE).is_none());
    }

    /// Without a report address (upstream builds, tests, the web) nothing asks.
    #[test]
    fn builds_that_cannot_send_never_ask() {
        let h = harness(None);
        assert!(h.query_by_label(TITLE).is_none());
        assert_eq!(h.state().session.prefs.tinta_reports.consent, None);
    }

    /// The settings show in File › Options › General, and Delete asks the server to forget.
    #[test]
    fn options_turn_reports_off_and_delete_them() {
        let mut h = harness(Some(std::env::temp_dir().join("tinta-reports-test")));
        h.state_mut().run("tools.errorReports", serde_json::json!({"value": true})).unwrap();
        h.state_mut().session.prefs.tinta_reports.install_id = "0f8fad5b-d9cb-469f-a165-70867728950e".into();
        h.state_mut().run("file.options", serde_json::json!({"pane": "general"})).unwrap();
        h.run_steps(4);
        h.get_by_label(CHECKBOX).click();
        h.run_steps(2);
        assert_eq!(h.state().session.prefs.tinta_reports.consent, Some(false));
        h.get_by_label(FORGET).click();
        h.run_steps(2);
        assert!(h.state().session.prefs.tinta_reports.forget_pending.is_some());
        let _ = h.get_by_label(PENDING);
    }

    const LANGS: [&str; 29] = [
        "zh-hans", "zh-hant", "ja", "ko", "uk", "es", "pt-br", "sr", "sr-latn", "pl", "cs", "et", "de", "ru", "tr", "it", "hu", "fr", "bg", "vi",
        "sv", "hr", "nl", "da", "ro", "sl", "ca", "gl", "eu",
    ];

    /// Every string is translated in every language the app offers.
    #[test]
    fn every_string_is_translated() {
        let rows = include_str!("i18n/tinta.tsv");
        for s in STRINGS {
            for lang in LANGS {
                let found = rows.lines().any(|l| {
                    let mut c = l.split('\t');
                    c.next() == Some(lang) && c.next() == Some(s) && c.next().is_some_and(|t| !t.trim().is_empty())
                });
                assert!(found, "{lang}: {s:?}");
            }
        }
    }
}
