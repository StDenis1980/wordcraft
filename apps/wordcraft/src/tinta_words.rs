//! Tinta Office: "Send them automatically", stage 3 of Suggest my words
//! ([`wordcraft_engine::tinta_suggest`] keeps the choice and the list, `wordcraft_ui_egui` shows
//! them in File › Options › Proofing).
//!
//! While both consents are on, a worker thread sends the listed words in the languages Tinta's
//! server checks to `<TINTA_REPORT_URL>/v1/words` once a day (at the start, then every
//! [`SEND_EVERY`]), under the random installation id the engine keeps, and marks them sent. A
//! withdrawal asks `/v1/words/forget` to remove the old id from the server's counts, retried every
//! [`RETRY_EVERY`] until the server confirms. Builds without a report address never send.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use wordcraft_engine::tinta_suggest::{self, SuggestedWord};
use wordcraft_ui_egui::WordApp;

use crate::tinta_report::{ENDPOINT, KEY, agent, new_install_id};

/// How often listed words are sent.
const SEND_EVERY: Duration = Duration::from_secs(24 * 60 * 60);
/// How often a withdrawal the server didn't confirm yet, or a send that failed, is tried again.
const RETRY_EVERY: Duration = Duration::from_secs(10 * 60);
/// The server takes at most 50 words a request and removes at most 400 a forget call.
const WORDS_PER_REQUEST: usize = 50;
const MAX_REQUESTS: usize = 20;
const MAX_FORGET_CALLS: usize = 10;

/// Set at the start when this build sends and settings are kept.
static ENABLED: AtomicBool = AtomicBool::new(false);

/// Start sending (only when the build has a report address and settings are on).
pub fn install(prefs_enabled: bool) {
    ENABLED.store(ENDPOINT.is_some() && prefs_enabled, Ordering::Relaxed);
}

/// What a worker thread did.
enum Outcome {
    Forgotten(String),
    /// Words the server took (or refused for good), and whether every batch went through.
    Sent(Vec<SuggestedWord>, bool),
}

struct Worker {
    next: Option<Instant>,
    working: bool,
    tx: Sender<Outcome>,
    rx: Receiver<Outcome>,
}

static STATE: OnceLock<Mutex<Worker>> = OnceLock::new();

/// Each frame, from the desktop app's `logic`.
pub fn tick(app: &mut WordApp) {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }
    let Some(endpoint) = ENDPOINT else { return };
    let prefs = &mut app.session.prefs.tinta_suggest;
    if prefs.enabled
        && prefs.auto
        && prefs.install_id.is_empty()
        && let Some(id) = new_install_id()
    {
        prefs.install_id = id;
    }
    let state = STATE.get_or_init(|| {
        let (tx, rx) = channel();
        Mutex::new(Worker { next: None, working: false, tx, rx })
    });
    let mut st = state.lock().unwrap_or_else(PoisonError::into_inner);
    while let Ok(outcome) = st.rx.try_recv() {
        st.working = false;
        match outcome {
            Outcome::Forgotten(id) => {
                if prefs.forget_pending.as_deref() == Some(id.as_str()) {
                    prefs.forget_pending = None;
                }
            }
            Outcome::Sent(words, all) => {
                tinta_suggest::mark_sent(prefs, &words);
                if !all {
                    st.next = Some(Instant::now() + RETRY_EVERY);
                }
            }
        }
    }
    if st.working || st.next.is_some_and(|t| Instant::now() < t) {
        return;
    }
    let forget = prefs.forget_pending.clone();
    let unsent = tinta_suggest::unsent(prefs);
    let send_as = (!unsent.is_empty() && !prefs.install_id.is_empty()).then(|| prefs.install_id.clone());
    if forget.is_none() && send_as.is_none() {
        // Nothing to do now; look again soon (a word added, the option turned on).
        st.next = Some(Instant::now() + Duration::from_secs(60));
        return;
    }
    st.next = Some(Instant::now() + if forget.is_some() { RETRY_EVERY } else { SEND_EVERY });
    let tx = st.tx.clone();
    st.working = std::thread::Builder::new()
        .name("tinta-words-send".into())
        .spawn(move || {
            let agent = agent();
            if let Some(id) = forget
                && forget_all(&agent, endpoint, &id)
            {
                let _ = tx.send(Outcome::Forgotten(id));
            }
            let (sent, all) = match send_as {
                Some(id) => send_words(&agent, endpoint, &id, &unsent),
                None => (Vec::new(), true),
            };
            let _ = tx.send(Outcome::Sent(sent, all));
        })
        .is_ok();
}

/// POST JSON; the status and the JSON answer (`Null` when it isn't JSON), or `None` when the
/// server could not be reached.
fn post_json(agent: &ureq::Agent, endpoint: &str, path: &str, body: &Value) -> Option<(u16, Value)> {
    let url = format!("{}{path}", endpoint.trim_end_matches('/'));
    let mut res = agent.post(&url).header("content-type", "application/json").header("x-tinta-report", KEY).send(body.to_string()).ok()?;
    let status = res.status().as_u16();
    let answer =
        res.body_mut().with_config().limit(64 * 1024).read_to_vec().ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or(Value::Null);
    Some((status, answer))
}

/// Send `words` in batches; the ones the server took or refused for good, and whether all went.
fn send_words(agent: &ureq::Agent, endpoint: &str, id: &str, words: &[SuggestedWord]) -> (Vec<SuggestedWord>, bool) {
    let mut done = Vec::new();
    for batch in words.chunks(WORDS_PER_REQUEST).take(MAX_REQUESTS) {
        let list: Vec<Value> = batch.iter().map(|w| json!({"word": w.word, "lang": w.lang})).collect();
        let body = json!({"app": "texto", "install_id": id, "words": list});
        match post_json(agent, endpoint, "/v1/words", &body) {
            // Taken, or refused for good (malformed, too large): either way these are done. Anything
            // else (a server without /v1/words yet, a wrong key, busy) keeps them for later.
            Some((s, _)) if (200..300).contains(&s) || s == 400 || s == 413 => done.extend_from_slice(batch),
            _ => return (done, false),
        }
    }
    let all = done.len() == words.len();
    (done, all)
}

/// Ask the server to forget `id` until it says nothing remains; whether it did.
fn forget_all(agent: &ureq::Agent, endpoint: &str, id: &str) -> bool {
    for _ in 0..MAX_FORGET_CALLS {
        match post_json(agent, endpoint, "/v1/words/forget", &json!({"install_id": id})) {
            Some((s, answer)) if (200..300).contains(&s) => {
                if answer.get("remaining").and_then(Value::as_u64).unwrap_or(0) == 0 {
                    return true;
                }
            }
            _ => return false,
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;

    use super::*;

    /// A one-thread HTTP server answering each request with the next of `answers`; returns its
    /// address and the request bodies it got.
    fn server(answers: Vec<(u16, &'static str)>) -> (String, std::thread::JoinHandle<Vec<(String, Value)>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = format!("http://{}", listener.local_addr().unwrap());
        let handle = std::thread::spawn(move || {
            let mut got = Vec::new();
            for (status, body) in answers {
                let (stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream);
                let mut first = String::new();
                reader.read_line(&mut first).unwrap();
                let mut len = 0;
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" || line.is_empty() {
                        break;
                    }
                    if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                        len = v.trim().parse().unwrap();
                    }
                }
                let mut buf = vec![0; len];
                reader.read_exact(&mut buf).unwrap();
                let path = first.split(' ').nth(1).unwrap_or_default().to_string();
                got.push((path, serde_json::from_slice(&buf).unwrap()));
                let mut stream = reader.into_inner();
                write!(
                    stream,
                    "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                )
                .unwrap();
            }
            got
        });
        (addr, handle)
    }

    const ID: &str = "0f8fad5b-d9cb-469f-a165-70867728950e";

    fn words(n: usize) -> Vec<SuggestedWord> {
        (0..n).map(|i| SuggestedWord { word: format!("palabra{}", "s".repeat(i)), lang: "es".into(), sent: false }).collect()
    }

    #[test]
    fn words_go_in_batches_with_the_install_id() {
        let (addr, server) = server(vec![(202, "{}"), (202, "{}")]);
        let list = words(WORDS_PER_REQUEST + 3);
        let (sent, all) = send_words(&agent(), &addr, ID, &list);
        assert!(all);
        assert_eq!(sent, list);
        let got = server.join().unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].0, "/v1/words");
        assert_eq!(got[0].1["install_id"], json!(ID));
        assert_eq!(got[0].1["words"].as_array().map(Vec::len), Some(WORDS_PER_REQUEST));
        assert_eq!(got[1].1["words"][2], json!({"word": list[WORDS_PER_REQUEST + 2].word, "lang": "es"}));
    }

    #[test]
    fn a_busy_server_keeps_the_rest_for_later() {
        let (addr, server) = server(vec![(202, "{}"), (503, "{}")]);
        let list = words(WORDS_PER_REQUEST + 1);
        let (sent, all) = send_words(&agent(), &addr, ID, &list);
        assert!(!all);
        assert_eq!(sent.len(), WORDS_PER_REQUEST);
        server.join().unwrap();
        let (addr, missing) = super::tests::server(vec![(404, r#"{"error":"path"}"#)]);
        let (sent, all) = send_words(&agent(), &addr, ID, &list);
        assert!(sent.is_empty() && !all, "a server without /v1/words yet keeps the words");
        missing.join().unwrap();
        let (sent, all) = send_words(&agent(), "http://127.0.0.1:9", ID, &list);
        assert!(sent.is_empty() && !all, "an unreachable server sends nothing");
    }

    #[test]
    fn forget_repeats_until_nothing_remains() {
        let (addr, server) = server(vec![(200, r#"{"deleted":400,"remaining":3}"#), (200, r#"{"deleted":3,"remaining":0}"#)]);
        assert!(forget_all(&agent(), &addr, ID));
        let got = server.join().unwrap();
        assert_eq!(got.len(), 2);
        assert!(got.iter().all(|(p, b)| p == "/v1/words/forget" && b["install_id"] == json!(ID)));
        assert!(!forget_all(&agent(), "http://127.0.0.1:9", ID), "unconfirmed: tried again later");
    }
}
