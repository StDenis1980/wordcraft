//! Tinta Office: automatic error reports, sent only with the person's consent.
//!
//! The desktop app asks once, at the first start of a build that can send reports, and the answer
//! is kept here ([`ReportsPrefs::consent`]); File › Options › General changes it. This module only
//! keeps the choice: the desktop app (`apps/wordcraft/src/tinta_report.rs`) writes, scrubs and
//! sends the reports to report.tintaoffice.com, makes the random installation id, and deletes the
//! server's copies when asked ([`ReportsPrefs::forget_pending`]). Builds without a report address
//! never ask and never send.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{CmdError, CommandSpec, Session};

/// The saved choice.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReportsPrefs {
    /// `None` until the person answered the first-start question.
    pub consent: Option<bool>,
    /// The random id reports are filed under on the server (36 characters, made by the desktop
    /// app when reports are first allowed). Not tied to the person.
    pub install_id: String,
    /// An id whose reports the server should delete; the desktop app clears it once the server
    /// confirmed, so a deletion asked for offline happens at the next start.
    pub forget_pending: Option<String>,
}

/// A valid installation id: the lowercase 8-4-4-4-12 hex form.
pub fn id_ok(id: &str) -> bool {
    let groups: Vec<&str> = id.split('-').collect();
    groups.len() == 5
        && groups.iter().zip([8, 4, 4, 4, 12]).all(|(g, n)| g.len() == n && g.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
}

/// A damaged saved value reads as "not asked yet"; malformed ids are dropped.
pub fn lenient<'de, D: serde::Deserializer<'de>>(d: D) -> Result<ReportsPrefs, D::Error> {
    let value = Value::deserialize(d)?;
    let mut p: ReportsPrefs = serde_json::from_value(value).unwrap_or_default();
    if !id_ok(&p.install_id) {
        p.install_id.clear();
    }
    p.forget_pending = p.forget_pending.filter(|id| id_ok(id));
    Ok(p)
}

fn state(s: &Session) -> Value {
    let p = &s.prefs.tinta_reports;
    json!({"value": p.consent, "installId": p.install_id, "forgetPending": p.forget_pending.is_some()})
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("tools.errorReports", "Error Reports", "File › Options › General", |s, v| {
            match v.get("value") {
                None | Some(Value::Null) => {}
                Some(Value::Bool(b)) => s.prefs.tinta_reports.consent = Some(*b),
                Some(_) => return Err(CmdError::Params("`value` must be true or false".into())),
            }
            match v.get("forget") {
                None | Some(Value::Null) | Some(Value::Bool(false)) => {}
                // The server deletes the old id's reports; later reports go under a new id.
                Some(Value::Bool(true)) => {
                    let p = &mut s.prefs.tinta_reports;
                    if !p.install_id.is_empty() {
                        p.forget_pending = Some(std::mem::take(&mut p.install_id));
                    }
                }
                Some(_) => return Err(CmdError::Params("`forget` must be true or false".into())),
            }
            Ok(state(s))
        })
        .params(r#"{"value"?: bool (send reports automatically), "forget"?: true (delete this installation's reports from the server)} → {"value": bool|null, "installId", "forgetPending"}"#)
        .pure(),
    ]
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn session() -> Session {
        Session::new(wordcraft_doc::Document::new())
    }

    #[test]
    fn not_asked_until_answered_and_the_answer_is_kept() {
        let mut s = session();
        assert_eq!(s.prefs.tinta_reports.consent, None);
        let v = s.run("tools.errorReports", &json!({"value": true})).unwrap();
        assert_eq!(v["value"], json!(true));
        s.run("tools.errorReports", &json!({"value": false})).unwrap();
        assert_eq!(s.prefs.tinta_reports.consent, Some(false));
        assert!(s.run("tools.errorReports", &json!({"value": "yes"})).is_err());
        assert!(s.run("tools.errorReports", &json!({"forget": 1})).is_err());
    }

    #[test]
    fn forget_moves_the_id_to_the_pending_deletion() {
        let mut s = session();
        let id = "0f8fad5b-d9cb-469f-a165-70867728950e";
        s.prefs.tinta_reports.install_id = id.into();
        let v = s.run("tools.errorReports", &json!({"forget": true})).unwrap();
        assert_eq!(v["forgetPending"], json!(true));
        assert_eq!(s.prefs.tinta_reports.forget_pending.as_deref(), Some(id));
        assert!(s.prefs.tinta_reports.install_id.is_empty());
        // Nothing to forget without an id.
        let mut s = session();
        s.run("tools.errorReports", &json!({"forget": true})).unwrap();
        assert_eq!(s.prefs.tinta_reports.forget_pending, None);
    }

    #[test]
    fn damaged_saved_values_read_as_not_asked() {
        #[derive(serde::Deserialize)]
        struct W {
            #[serde(deserialize_with = "lenient")]
            r: ReportsPrefs,
        }
        let w: W = serde_json::from_value(json!({"r": {"consent": "maybe"}})).unwrap();
        assert_eq!(w.r, ReportsPrefs::default());
        let w: W = serde_json::from_value(json!({"r": {"consent": true, "install_id": "../../etc", "forget_pending": "x"}})).unwrap();
        assert_eq!(w.r, ReportsPrefs { consent: Some(true), ..Default::default() });
        assert!(id_ok("0f8fad5b-d9cb-469f-a165-70867728950e"));
        assert!(!id_ok("0F8FAD5B-d9cb-469f-a165-70867728950e"));
    }
}
