//! What each command may do beyond the open document, for agents' permission layers
//! (tintaoffice-web#26, Vilna OS's Herkos): `list_commands`, `engine.commands` and
//! `wordcraft-cli commands --json` give every command a `risk` object.
//!
//! - `reads_document` / `changes_document`: the open document (from the command's own
//!   `mutates`; every command but a view-only one reads it).
//! - `reads_disk` / `writes_disk`: files outside the document in memory (open, insert a picture,
//!   mail-merge data, templates; save, export, the user dictionary, installed fonts and packs).
//!   The automation roots still hold every such path.
//! - `network`: anything that leaves the device.
//! - `print`: the printer. `sign`: a certificate signature (none in Texto).
//! - `other_program`: hands something to another program (Read Aloud's speech service).
//! - `runs_commands`: runs other commands (macros), each still held to `--allow-commands`.
//! - `ui_only`: only how the window shows the document.
//!
//! The tables below name the commands with those effects. They are reviewed by hand: a new
//! command changes `docs/tinta-commands.tsv`, whose test (`tinta_risk::tests`) fails until the
//! snapshot is updated on purpose, which is the moment to add it here. A table entry that names
//! no command fails the test too, so a rename can't leave a stale entry behind.

use serde_json::{Value, json};

use crate::CommandSpec;

/// Commands that read files from disk (besides the document already open).
const READS_DISK: &[&str] = &[
    "file.open",
    "file.newFromTemplate",
    "file.listTemplates",
    "file.options",
    "file.recover",
    "file.versions",
    "font.add",
    "font.loadSystem",
    "insert.picture",
    "insert.textFromFile",
    "insert.object",
    "picture.change",
    "mailings.recipients",
    "references.sources",
    "review.compare",
    "review.combine",
    "styles.organizer",
    "styles.copyFrom",
    "tools.attachTemplate",
    "tools.updateStyles",
    "proof.packs",
];

/// Commands that write files.
const WRITES_DISK: &[&str] = &[
    "file.save",
    "file.saveAs",
    "file.saveTemplate",
    "file.exportPdf",
    "file.exportPng",
    "file.printPdf",
    "file.recover",
    "file.versions",
    "mailings.finish",
    "review.addToDictionary",
    "font.add",
    "font.remove",
    "proof.installPack",
    "proof.removePack",
];

/// Commands that may reach the network.
const NETWORK: &[&str] =
    &["proof.packs", "proof.installPack", "review.suggestWords", "review.suggestWordsAuto", "review.suggestedWords", "tools.errorReports"];

/// Commands that print.
const PRINT: &[&str] = &["file.print", "mailings.printDocuments"];

/// Commands that sign with a certificate.
const SIGN: &[&str] = &[];

/// Commands that hand something to another program on the computer (speech-dispatcher, or the
/// system's voice, reads the text aloud).
const OTHER_PROGRAM: &[&str] = &["review.readAloud", "readAloud."];

/// Commands that run other commands (macros). Each step is still held to `--allow-commands`.
const RUNS_COMMANDS: &[&str] = &["tools.macros"];

/// Groups (`view.`) and ids whose commands only change how the window shows the document.
const UI_ONLY: &[&str] = &["view."];

fn named(table: &[&str], id: &str) -> bool {
    table.iter().any(|t| if t.ends_with('.') { id.starts_with(t) } else { *t == id })
}

/// The `risk` object of `spec`.
pub fn risk(spec: &CommandSpec) -> Value {
    let id = spec.id;
    let ui_only = named(UI_ONLY, id);
    json!({
        "reads_document": !ui_only,
        "changes_document": spec.mutates,
        "reads_disk": named(READS_DISK, id),
        "writes_disk": named(WRITES_DISK, id),
        "network": named(NETWORK, id),
        "print": named(PRINT, id),
        "sign": named(SIGN, id),
        "other_program": named(OTHER_PROGRAM, id),
        "runs_commands": named(RUNS_COMMANDS, id),
        "ui_only": ui_only,
    })
}

/// One line per command, sorted by id: the id, then the risk flags that are set
/// (`docs/tinta-commands.tsv`).
pub fn snapshot(specs: &[CommandSpec]) -> String {
    let mut lines: Vec<String> = specs
        .iter()
        .map(|spec| {
            let r = risk(spec);
            let set: Vec<&str> =
                r.as_object().map(|o| o.iter().filter(|(_, v)| v.as_bool() == Some(true)).map(|(k, _)| k.as_str()).collect()).unwrap_or_default();
            format!("{}\t{}", spec.id, set.join(","))
        })
        .collect();
    lines.sort();
    lines.dedup();
    let mut out = String::from(
        "# Tinta Texto commands and what they may do (tinta_risk.rs). Generated: TINTA_UPDATE_SNAPSHOT=1 cargo test -p wordcraft-engine tinta_risk\n",
    );
    for line in lines {
        out.push_str(&line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SNAPSHOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/tinta-commands.tsv");

    fn specs() -> Vec<CommandSpec> {
        crate::cmd::registry().all().to_vec()
    }

    /// Command ids are what agents' permission manifests name (tintaoffice-web#33): a command
    /// added, renamed or removed, or a risk flag changed, fails here until the snapshot is
    /// regenerated on purpose (and the change noted in the release notes).
    #[test]
    fn command_ids_and_risks_match_the_snapshot() {
        let now = snapshot(&specs());
        if std::env::var_os("TINTA_UPDATE_SNAPSHOT").is_some() {
            std::fs::write(SNAPSHOT, &now).unwrap();
            return;
        }
        // A Windows checkout may give the snapshot CRLF line endings.
        let saved = std::fs::read_to_string(SNAPSHOT).unwrap_or_default().replace("\r\n", "\n");
        if saved != now {
            let old: std::collections::BTreeSet<&str> = saved.lines().collect();
            let new: std::collections::BTreeSet<&str> = now.lines().collect();
            let gone: Vec<&&str> = old.difference(&new).collect();
            let added: Vec<&&str> = new.difference(&old).collect();
            panic!(
                "the commands changed: gone {gone:?}, new {added:?}. Review tinta_risk.rs for the new ones, then run \
                 TINTA_UPDATE_SNAPSHOT=1 cargo test -p wordcraft-engine tinta_risk"
            );
        }
    }

    /// Every table entry names a command that exists.
    #[test]
    fn every_table_entry_names_a_command() {
        let ids: Vec<&str> = specs().iter().map(|s| s.id).collect();
        for table in [READS_DISK, WRITES_DISK, NETWORK, PRINT, SIGN, OTHER_PROGRAM, RUNS_COMMANDS, UI_ONLY] {
            for entry in table {
                assert!(ids.iter().any(|id| named(&[entry], id)), "{entry} names no command");
            }
        }
    }

    #[test]
    fn saving_printing_and_sending_are_flagged() {
        let all = specs();
        let flags = |id: &str| risk(all.iter().find(|s| s.id == id).unwrap());
        assert_eq!(flags("file.save")["writes_disk"], true);
        assert_eq!(flags("file.print")["print"], true);
        assert_eq!(flags("tools.errorReports")["network"], true);
        assert_eq!(flags("format.bold")["changes_document"], true);
        assert_eq!(flags("format.bold")["writes_disk"], false);
        assert_eq!(flags("view.zoom")["ui_only"], true);
    }
}
