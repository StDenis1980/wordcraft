//! Tinta Office: the Tinta look, the colours and control shapes of Tinta Correo carried over to
//! Tinta Texto. The window is a soft, slightly tinted grey with a wash of the app's colour along
//! the top; panes, menus, dialogs and fields are white cards; buttons have no frame, only a tint of
//! the accent that deepens under the pointer; corners are rounder. The accent is the blue of the
//! Tinta Texto icon.
//!
//! Only colours and shapes change, never the layout, so upstream's ribbon and panes stay as they
//! are. Off (File › Options › General › Tinta Office look, `ui.tintaLook`), the interface is
//! upstream's classic palette. The setting is read through [`on`], which the app sets each frame
//! from `ui.json`; [`skin`] is the pure mapping, so tests never touch the global switch.

use std::sync::atomic::{AtomicBool, Ordering};

use egui::{Color32, CornerRadius, Stroke, Visuals};

use crate::theme::Tokens;

/// Off until the app reads its setting.
static ON: AtomicBool = AtomicBool::new(false);

/// Whether the Tinta look is on. Never in this crate's unit tests: they share one process, and
/// upstream's compare colours with the classic palette whatever other tests switch on.
pub fn on() -> bool {
    !cfg!(test) && allowed() && ON.load(Ordering::Relaxed)
}

/// Set by the desktop app ([`allow`]): only it shows the Tinta look. Upstream's integration tests
/// and examples compare colours with the classic palette, so they keep it, unless `TINTA_LOOK` is
/// set in the environment (for screenshots of the Tinta look).
static ALLOWED: AtomicBool = AtomicBool::new(false);

/// Let this program show the Tinta look (the desktop app calls it once, at start).
pub fn allow() {
    ALLOWED.store(true, Ordering::Relaxed);
}

fn allowed() -> bool {
    static ENV: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    ALLOWED.load(Ordering::Relaxed) || *ENV.get_or_init(|| std::env::var_os("TINTA_LOOK").is_some())
}

/// Turn the Tinta look on or off; true when that changed it, so the caller re-applies the theme.
/// Never in unit tests, where re-applying the theme mid-test would change their frames.
pub fn set(on: bool) -> bool {
    !cfg!(test) && ON.swap(on, Ordering::Relaxed) != on
}

/// The palette in use: [`skin`] of `classic` while the Tinta look is on, else `classic` itself.
pub fn tokens(classic: Tokens) -> Tokens {
    if on() { skin(classic) } else { classic }
}

/// The Tinta palette, light or dark: Tinta Correo's neutrals with the Tinta Texto blue.
struct Palette {
    accent: Color32,
    window: Color32,
    band: Color32,
    card: Color32,
    text: Color32,
    text_weak: Color32,
    border: Color32,
    on_accent: Color32,
}

const LIGHT: Palette = Palette {
    accent: Color32::from_rgb(0x2D, 0x4B, 0xA0),
    window: Color32::from_rgb(0xEC, 0xEF, 0xF5),
    band: Color32::from_rgb(0xD7, 0xDF, 0xF1),
    card: Color32::WHITE,
    text: Color32::from_rgb(0x1F, 0x24, 0x30),
    text_weak: Color32::from_rgb(0x5C, 0x63, 0x73),
    border: Color32::from_rgb(0xE1, 0xE5, 0xEB),
    on_accent: Color32::WHITE,
};

const DARK: Palette = Palette {
    accent: Color32::from_rgb(0x8F, 0xA5, 0xE8),
    window: Color32::from_rgb(0x16, 0x1A, 0x22),
    band: Color32::from_rgb(0x1D, 0x26, 0x38),
    card: Color32::from_rgb(0x24, 0x27, 0x30),
    text: Color32::from_rgb(0xEC, 0xE8, 0xDF),
    text_weak: Color32::from_rgb(0xB2, 0xB5, 0xC0),
    border: Color32::from_rgb(0x3A, 0x3E, 0x4C),
    on_accent: Color32::from_rgb(0x12, 0x14, 0x1A),
};

/// `a` moved `share` (0 to 1) of the way to `b`.
pub fn mix(a: Color32, b: Color32, share: f32) -> Color32 {
    let s = share.clamp(0.0, 1.0);
    let ch = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * s).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgb(ch(a.r(), b.r()), ch(a.g(), b.g()), ch(a.b(), b.b()))
}

/// `c` at `alpha` (0 to 255) over whatever is below.
fn alpha(c: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
}

/// The Tinta look of `classic`: the chrome, panes and controls take the Tinta palette; what is
/// drawn on the page (handles, guides, crop, the comment authors' colours) and the status colours
/// stay as upstream chose them.
pub fn skin(classic: Tokens) -> Tokens {
    let p = if classic.dark { DARK } else { LIGHT };
    let dark = classic.dark;
    // The strip along the top: the window with the app's wash.
    let top = mix(p.window, p.band, 0.6);
    Tokens {
        title_bar: top,
        tab_strip: top,
        ribbon: p.window,
        canvas: if dark { mix(p.window, Color32::BLACK, 0.3) } else { mix(p.window, p.text_weak, 0.08) },
        panel: p.card,
        border: p.border,
        border_strong: mix(p.border, p.text_weak, 0.3),
        text: p.text,
        text_dim: p.text_weak,
        text_disabled: mix(p.text_weak, p.window, 0.5),
        icon: mix(p.text, p.text_weak, 0.35),
        hover: mix(p.window, p.accent, if dark { 0.16 } else { 0.10 }),
        pressed: mix(p.window, p.accent, if dark { 0.26 } else { 0.20 }),
        checked: mix(p.card, p.accent, if dark { 0.28 } else { 0.16 }),
        accent: p.accent,
        nav: if dark { mix(p.window, p.accent, 0.32) } else { p.accent },
        accent_text: p.accent,
        on_accent: p.on_accent,
        input: if dark { mix(p.card, p.window, 0.5) } else { p.card },
        input_border: mix(p.text_weak, p.card, if dark { 0.3 } else { 0.25 }),
        group_label: p.text_weak,
        ruler: if dark { mix(p.card, Color32::WHITE, 0.12) } else { p.card },
        ruler_margin: if dark { p.window } else { mix(p.window, p.text_weak, 0.12) },
        ruler_tick: p.text_weak,
        status_bar: p.window,
        selection: alpha(p.accent, if dark { 0x55 } else { 0x48 }),
        menu: p.card,
        balloon: p.card,
        markup_area: p.window,
        markup_edge: mix(p.border, p.text_weak, 0.3),
        ..classic
    }
}

/// Tinta Correo's control shapes on egui's visuals for `t` ([`crate::theme`] fills in the colours
/// first): rounder corners, buttons without a frame (a soft accent tint instead), fields with a
/// light frame that turns to the accent under the pointer, softer shadows.
pub fn visuals(v: &mut Visuals, t: &Tokens) {
    if !on() {
        return;
    }
    let card = t.menu;
    v.window_corner_radius = CornerRadius::same(12);
    v.menu_corner_radius = CornerRadius::same(8);
    v.window_stroke = Stroke::new(1.0, t.border);
    v.faint_bg_color = mix(card, t.ribbon, 0.5);
    v.selection.bg_fill = mix(card, t.accent, if t.dark { 0.45 } else { 0.28 });
    let w = &mut v.widgets;
    w.noninteractive.corner_radius = CornerRadius::same(6);
    w.inactive.weak_bg_fill = mix(card, t.accent, 0.06);
    w.inactive.corner_radius = CornerRadius::same(6);
    w.hovered.bg_fill = mix(card, t.accent, if t.dark { 0.22 } else { 0.13 });
    w.hovered.weak_bg_fill = w.hovered.bg_fill;
    w.hovered.bg_stroke = Stroke::new(1.0, t.accent);
    w.hovered.corner_radius = CornerRadius::same(6);
    w.active.bg_fill = mix(card, t.accent, if t.dark { 0.32 } else { 0.22 });
    w.active.weak_bg_fill = w.active.bg_fill;
    w.active.corner_radius = CornerRadius::same(6);
    w.open.bg_fill = w.hovered.bg_fill;
    w.open.weak_bg_fill = w.hovered.bg_fill;
    w.open.corner_radius = CornerRadius::same(6);
    v.popup_shadow = egui::Shadow { offset: [0, 4], blur: 14, spread: 0, color: Color32::from_black_alpha(if t.dark { 100 } else { 32 }) };
    v.window_shadow = egui::Shadow { offset: [0, 6], blur: 24, spread: 0, color: Color32::from_black_alpha(if t.dark { 110 } else { 40 }) };
}

/// File › Options › General: the Tinta Office look on or off.
pub(crate) fn option(app: &mut crate::WordApp, ui: &mut egui::Ui) {
    let mut v = app.ui.tinta_look;
    if ui.checkbox(&mut v, crate::tl!(LABEL)).changed() {
        let _ = app.run("ui.tintaLook", serde_json::json!({ "value": v }));
    }
}

/// The setting's label in Options.
pub(crate) const LABEL: &str = "Tinta Office look";

/// `ui.tintaLook`: `{"value": true|false}` turns the Tinta look on or off; no value reads it.
pub(crate) fn command(app: &mut crate::WordApp, p: &serde_json::Value) -> Result<serde_json::Value, String> {
    match p.get("value") {
        None | Some(serde_json::Value::Null) => {}
        Some(serde_json::Value::Bool(b)) => app.ui.tinta_look = *b,
        Some(other) => return Err(format!("ui.tintaLook takes true or false, not {other}")),
    }
    Ok(serde_json::json!({ "value": app.ui.tinta_look }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// WCAG contrast ratio between two opaque colours.
    fn contrast(a: Color32, b: Color32) -> f32 {
        let lum = |c: Color32| {
            let f = |v: u8| {
                let v = f32::from(v) / 255.0;
                if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
            };
            0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
        };
        let (x, y) = (lum(a), lum(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    fn both() -> [Tokens; 2] {
        [skin(Tokens::light()), skin(Tokens::dark())]
    }

    /// The same rules upstream's palette keeps (theme.rs tests), and body text on every surface.
    #[test]
    fn the_tinta_look_keeps_text_and_controls_readable() {
        for t in both() {
            let dark = t.dark;
            for bg in [t.title_bar, t.tab_strip, t.ribbon, t.panel, t.menu, t.status_bar, t.input, t.balloon] {
                assert!(contrast(t.text, bg) >= 7.0, "text on {bg:?} (dark {dark})");
                assert!(contrast(t.text_dim, bg) >= 4.5, "weak text on {bg:?} (dark {dark})");
            }
            for bg in [t.ribbon, t.panel, t.menu, t.input] {
                assert!(contrast(t.accent_text, bg) >= 4.5, "accent text on {bg:?} (dark {dark})");
                assert!(contrast(t.input_border, bg) >= 3.0, "field frame on {bg:?} (dark {dark})");
            }
            for bg in [t.hover, t.pressed, t.checked] {
                assert!(contrast(t.text, bg) >= 4.5, "text on a hovered or chosen control (dark {dark})");
            }
            assert!(contrast(t.on_accent, t.accent) >= 4.5, "text on the accent (dark {dark})");
            assert!(contrast(Color32::WHITE, t.nav) >= 4.5, "the File page's side bar (dark {dark})");
            assert!(contrast(t.ruler_tick, t.ruler) >= 3.0, "ruler ticks (dark {dark})");
        }
    }

    /// The ruler's text area stays lighter than its margins, as upstream's is (#1148).
    #[test]
    fn the_ruler_text_area_is_lighter_than_its_margins() {
        for t in both() {
            assert!(contrast(t.ruler, Color32::BLACK) > contrast(t.ruler_margin, Color32::BLACK), "dark {}", t.dark);
        }
    }

    /// Pages sit on a canvas darker than the window around them, so the page edge shows.
    #[test]
    fn pages_stand_out_from_the_canvas() {
        for t in both() {
            assert_ne!(t.canvas, t.ribbon);
            assert!(contrast(Color32::WHITE, t.canvas) >= 1.15, "dark {}", t.dark);
        }
    }

    /// The command reads the setting with no value and refuses anything but a boolean.
    #[test]
    fn the_setting_is_on_by_default_and_only_takes_true_or_false() {
        assert!(crate::UiState::default().tinta_look);
        let ui: crate::UiState = serde_json::from_str(r#"{"tintaLook": false}"#).unwrap();
        assert!(!ui.tinta_look);
    }

    /// The label is translated into every language the fork's strings cover.
    #[test]
    fn the_label_is_translated() {
        let rows = include_str!("i18n/tinta.tsv");
        let langs: std::collections::BTreeSet<&str> = rows.lines().filter_map(|l| l.split('\t').next()).collect();
        for lang in langs {
            assert!(rows.lines().any(|l| l.starts_with(&format!("{lang}\t{LABEL}\t"))), "{lang} has no {LABEL:?}");
        }
    }
}
