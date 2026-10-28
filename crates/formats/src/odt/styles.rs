use super::xml::{Attrs, Ev, events, get, length, local};
use super::{Align, Arc, Columns, Fmt, HashMap, Kind, Rgb, VAlign, model};

// ---------------------------------------------------------------------------------------------
// Import

#[derive(Clone, Debug, Default)]
struct FmtDelta {
    bold: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    strike: Option<bool>,
    sup: Option<bool>,
    sub: Option<bool>,
    color: Option<Rgb>,
    background: Option<Rgb>,
    size: Option<f32>,
    font: Option<Arc<str>>,
    code: Option<bool>,
}

impl FmtDelta {
    fn apply(&self, f: &mut Fmt) {
        macro_rules! set {
            ($($k:ident),*) => { $( if let Some(v) = self.$k.clone() { f.$k = v; } )* };
        }
        set!(bold, italic, underline, strike, sup, sub, code);
        if self.color.is_some() {
            f.color = self.color;
        }
        if self.background.is_some() {
            f.background = self.background;
        }
        if self.size.is_some() {
            f.size = self.size;
        }
        if let Some(fam) = &self.font {
            f.font = Some(fam.clone());
        }
    }
}

#[derive(Clone, Debug, Default)]
struct StyleInfo {
    parent: Option<String>,
    display: Option<String>,
    auto: bool,
    fmt: FmtDelta,
    align: Option<Align>,
    page_break: bool,
    /// `fo:keep-with-next`: `Some(true)` for "always", `Some(false)` for "auto".
    keep_next: Option<bool>,
    outline: Option<u8>,
    border_bottom: bool,
    /// `fo:background-color` of a table-cell style's `style:table-cell-properties`.
    cell_background: Option<Rgb>,
    /// `style:vertical-align` of a table-cell style's `style:table-cell-properties`.
    cell_valign: Option<VAlign>,
    /// Whether a table-cell style's `fo:border*` attributes draw a line: `Some(false)` when they
    /// are all `none`, `None` when it names none (Tinta Office).
    cell_border: Option<bool>,
    /// `table:align` of a table style's `style:table-properties` (Tinta Office).
    table_align: Option<Align>,
    /// `style:column-width` of a table-column style's `style:table-column-properties`, in points.
    column_width: Option<f32>,
    /// `style:master-page-name` of a paragraph or table style: its paragraphs (tables) start a
    /// page of that master page. Empty: explicitly none, which stops the lookup up the chain.
    master_page: Option<String>,
    /// `style:page-number` of a paragraph or table style (applied only with a master page): `Some(Some(n))` restarts page numbering at n,
    /// `Some(None)` ("auto") continues it.
    page_number: Option<Option<u32>>,
    /// `style:columns` of a section style.
    columns: Option<Columns>,
    /// Spacing, indents and line height of `style:paragraph-properties` (Tinta Office).
    spacing: model::Spacing,
    /// `style:tab-stops` of `style:paragraph-properties`, positions from the paragraph's left
    /// margin as ODF writes them (`text:tab-stops-relative-to-indent`, LibreOffice's default).
    tabs: Option<Vec<wordcraft_doc::props::TabStop>>,
    /// `fo:border*` and `fo:padding*` of `style:paragraph-properties`, when it sets a border.
    borders: Option<wordcraft_doc::props::Borders>,
}

/// A paragraph's direct formatting from its styles ([`Styles::para_direct`]).
#[derive(Default)]
pub(super) struct ParaDirect {
    pub(super) spacing: model::Spacing,
    pub(super) tabs: Vec<wordcraft_doc::props::TabStop>,
    pub(super) borders: Option<wordcraft_doc::props::Borders>,
}

/// Most tab stops read for one style.
const MAX_TABS: usize = 64;

/// No indent or spacing is larger than this many points (22 inches, the largest page).
const MAX_SPACING: f32 = 1584.0;

/// Spacing, indents and line height of a `style:paragraph-properties` element (ODF 1.3 §17.6).
fn paragraph_spacing(a: &Attrs, sp: &mut model::Spacing) {
    use wordcraft_doc::props::LineSpacing;
    let clamp = |v: f32, min: f32| v.clamp(min, MAX_SPACING);
    for (attr, slot, min) in [
        ("fo:margin-top", &mut sp.before, 0.0),
        ("fo:margin-bottom", &mut sp.after, 0.0),
        ("fo:margin-left", &mut sp.left, -MAX_SPACING),
        ("fo:margin-right", &mut sp.right, -MAX_SPACING),
        ("fo:text-indent", &mut sp.first, -MAX_SPACING),
    ] {
        // Percentages (of the parent's value) aren't read.
        if let Some(v) = get(a, attr).filter(|v| !v.trim().ends_with('%')).and_then(length) {
            *slot = Some(clamp(v, min));
        }
    }
    if let Some(v) = get(a, "fo:line-height").map(str::trim) {
        if let Some(pct) = v.strip_suffix('%') {
            if let Some(m) = pct.trim().parse::<f32>().ok().filter(|m| m.is_finite() && *m > 0.0) {
                sp.line = Some(LineSpacing::Multiple((m / 100.0).clamp(0.25, 10.0)));
            }
        } else if v.eq_ignore_ascii_case("normal") {
            sp.line = Some(LineSpacing::Multiple(1.0));
        } else if let Some(l) = length(v).filter(|l| *l > 0.0) {
            sp.line = Some(LineSpacing::Exactly(l.min(MAX_SPACING)));
        }
    } else if let Some(l) = get(a, "style:line-height-at-least").and_then(length).filter(|l| *l >= 0.0) {
        sp.line = Some(LineSpacing::AtLeast(l.min(MAX_SPACING)));
    }
}

/// A table style's `style:table-properties` (ODF 1.3 §17.15).
fn table_properties(a: &Attrs, s: &mut StyleInfo) {
    if let Some(v) = get(a, "style:page-number") {
        s.page_number = Some(v.trim().parse::<u32>().ok().map(|n| n.min(32_767)));
    }
    // `margins` (and `left`) is Word's default: the table starts at the left margin.
    match get(a, "table:align").map(str::trim) {
        Some("center") => s.table_align = Some(Align::Center),
        Some("right") => s.table_align = Some(Align::Right),
        Some("left") | Some("margins") => s.table_align = Some(Align::Left),
        _ => {}
    }
}

/// Does a `style:table-cell-properties` element draw a border? `None` when it sets none of
/// `fo:border`, `fo:border-top`, `fo:border-bottom`, `fo:border-left` and `fo:border-right`.
fn cell_border(a: &Attrs) -> Option<bool> {
    let edges: Vec<&str> =
        ["fo:border", "fo:border-top", "fo:border-bottom", "fo:border-left", "fo:border-right"].iter().filter_map(|k| get(a, k)).collect();
    (!edges.is_empty()).then(|| edges.iter().any(|e| !e.trim().eq_ignore_ascii_case("none") && !e.trim().is_empty()))
}

/// A paragraph style's `style:paragraph-properties` (ODF 1.3 §17.6).
fn paragraph_properties(a: &Attrs, s: &mut StyleInfo) {
    s.align = match get(a, "fo:text-align").map(|v| v.trim().to_ascii_lowercase()).as_deref() {
        Some("center") => Some(Align::Center),
        Some("end") | Some("right") => Some(Align::Right),
        Some("justify") => Some(Align::Justify),
        Some("start") | Some("left") => Some(Align::Left),
        _ => s.align,
    };
    if get(a, "fo:break-before").is_some_and(|b| b.trim().eq_ignore_ascii_case("page")) {
        s.page_break = true;
    }
    match get(a, "fo:keep-with-next") {
        Some("always") => s.keep_next = Some(true),
        Some("auto") => s.keep_next = Some(false),
        _ => {}
    }
    if let Some(v) = get(a, "style:page-number") {
        s.page_number = Some(v.trim().parse::<u32>().ok().map(|n| n.min(32_767)));
    }
    if get(a, "fo:border-bottom").is_some_and(|b| !b.trim().eq_ignore_ascii_case("none")) {
        s.border_bottom = true;
    }
    paragraph_spacing(a, &mut s.spacing);
    if let Some(b) = paragraph_borders(a) {
        s.borders = Some(b);
    }
}

/// One `fo:border` value (ODF 1.3 §20.183): `0.74pt solid #1f3864`, in any order; `none` and
/// zero widths draw nothing.
fn border_value(v: &str) -> wordcraft_doc::props::Border {
    use wordcraft_doc::props::{Border, BorderStyle};
    let mut b = Border { style: BorderStyle::None, width: 0.0, color: None, space: 0.0 };
    let mut style = None;
    for part in v.split_whitespace().take(8) {
        match part.to_ascii_lowercase().as_str() {
            "none" | "hidden" => return b,
            "solid" | "groove" | "ridge" | "inset" | "outset" => style = Some(BorderStyle::Single),
            "double" => style = Some(BorderStyle::Double),
            "dotted" => style = Some(BorderStyle::Dotted),
            "dashed" => style = Some(BorderStyle::Dashed),
            "thin" => b.width = 0.5,
            "medium" => b.width = 1.0,
            "thick" => b.width = 2.0,
            _ => {
                if let Some(c) = model::parse_color(part) {
                    b.color = Some(c);
                } else if let Some(w) = length(part).filter(|w| *w >= 0.0) {
                    b.width = w.min(12.0);
                }
            }
        }
    }
    if b.width > 0.0 {
        b.style = style.unwrap_or(BorderStyle::Single);
    }
    b
}

/// A paragraph's borders and their distance from the text (`fo:border*`, `fo:padding*`), when any
/// border attribute is set.
fn paragraph_borders(a: &Attrs) -> Option<wordcraft_doc::props::Borders> {
    let all = get(a, "fo:border").map(border_value);
    let pad_all = get(a, "fo:padding").and_then(length);
    let mut out = wordcraft_doc::props::Borders::default();
    let mut any = all.is_some();
    for (side, slot) in [("top", &mut out.top), ("bottom", &mut out.bottom), ("left", &mut out.left), ("right", &mut out.right)] {
        let edge = get(a, &format!("fo:border-{side}")).map(border_value);
        any |= edge.is_some();
        if let Some(mut e) = edge.or(all) {
            let pad = get(a, &format!("fo:padding-{side}")).and_then(length).or(pad_all);
            e.space = pad.unwrap_or(0.0).clamp(0.0, 31.0);
            *slot = Some(e);
        }
    }
    any.then_some(out)
}

/// A `style:tab-stops` element starts a style's own list of stops (none, when it is empty); each
/// `style:tab-stop` inside adds one, up to [`MAX_TABS`].
fn tab_element(s: &mut StyleInfo, name: &str, a: &Attrs) {
    if name == "tab-stops" {
        s.tabs = Some(Vec::new());
    } else if let Some(tabs) = &mut s.tabs
        && tabs.len() < MAX_TABS
        && let Some(t) = tab_stop(a)
    {
        tabs.push(t);
    }
}

/// A `style:tab-stop` element (ODF 1.3 §17.8), its position as written.
fn tab_stop(a: &Attrs) -> Option<wordcraft_doc::props::TabStop> {
    use wordcraft_doc::props::{TabAlign, TabLeader, TabStop};
    let pos = get(a, "style:position").and_then(length)?.clamp(0.0, MAX_SPACING);
    let align = match get(a, "style:type").map(str::trim) {
        Some("center") => TabAlign::Center,
        Some("right") => TabAlign::Right,
        Some("char") => TabAlign::Decimal,
        _ => TabAlign::Left,
    };
    let style = get(a, "style:leader-style").map(str::trim).filter(|s| *s != "none");
    let leader = match get(a, "style:leader-text").map(str::trim) {
        Some(".") => TabLeader::Dot,
        Some("-") => TabLeader::Hyphen,
        Some("_") => TabLeader::Underscore,
        Some("·") => TabLeader::MiddleDot,
        Some(" ") | Some("") => TabLeader::None,
        // No leader text: the leader style draws it.
        _ => match style {
            Some("dotted") => TabLeader::Dot,
            Some("dash") | Some("long-dash") | Some("dot-dash") | Some("dot-dot-dash") => TabLeader::Hyphen,
            Some(_) => TabLeader::Underscore,
            None => TabLeader::None,
        },
    };
    Some(TabStop { pos, align, leader })
}

/// Columns of a `style:columns` element (ODF 1.3 §17.12): count and gap, clamped as the RTF and
/// DOCX readers do; equal widths only (`style:column` widths are not read).
pub(super) fn columns_of(a: &Attrs) -> Columns {
    let mut c = Columns::default();
    if let Some(n) = get(a, "fo:column-count").and_then(|v| v.trim().parse::<u64>().ok()) {
        c.count = n.clamp(1, 45) as u32;
    }
    if let Some(g) = get(a, "fo:column-gap").and_then(length).filter(|v| *v >= 0.0) {
        c.space = g.min(1584.0);
    }
    c
}

/// A `style:column-sep` (ODF 1.3 §17.13) draws a line between columns unless its style is "none".
pub(super) fn column_sep(a: &Attrs) -> bool {
    get(a, "style:style") != Some("none")
}

fn default_font(a: &Attrs) -> Option<String> {
    let family = get(a, "fo:font-family").or_else(|| get(a, "style:font-name"))?;
    let family = family.split(',').next().unwrap_or("").trim().trim_matches(['\'', '"']);
    (!family.is_empty() && !model::is_mono(family)).then(|| family.to_string())
}

fn text_properties(a: &Attrs, style: &mut StyleInfo) {
    let fmt = &mut style.fmt;
    let is_bold = |weight: &str| {
        let weight = weight.trim();
        weight.eq_ignore_ascii_case("bold") || weight.parse::<u32>().is_ok_and(|n| n >= 600)
    };
    // Latin attributes take precedence; Asian and complex-script attributes are fallbacks.
    if let Some(weight) = get(a, "fo:font-weight").or_else(|| get(a, "style:font-weight-asian")).or_else(|| get(a, "style:font-weight-complex")) {
        fmt.bold = Some(is_bold(weight));
    }
    let is_italic = |value: &str| value.trim().eq_ignore_ascii_case("italic") || value.trim().eq_ignore_ascii_case("oblique");
    if let Some(value) = get(a, "fo:font-style").or_else(|| get(a, "style:font-style-asian")).or_else(|| get(a, "style:font-style-complex")) {
        fmt.italic = Some(is_italic(value));
    }
    let is_not_none = |value: &str| !value.trim().eq_ignore_ascii_case("none");
    if let Some(value) = get(a, "style:text-underline-style").or_else(|| get(a, "style:text-underline-type")) {
        fmt.underline = Some(is_not_none(value));
    }
    if let Some(value) = get(a, "style:text-line-through-style").or_else(|| get(a, "style:text-line-through-type")) {
        fmt.strike = Some(is_not_none(value));
    }
    if let Some(value) = get(a, "style:text-position") {
        let first = value.split_whitespace().next().unwrap_or("");
        let pct = first.trim_end_matches('%').parse::<f32>().unwrap_or(0.0);
        fmt.sup = Some(first.eq_ignore_ascii_case("super") || pct > 0.0);
        fmt.sub = Some(first.eq_ignore_ascii_case("sub") || pct < 0.0);
    }
    if let Some(color) = get(a, "fo:color").and_then(model::parse_color) {
        fmt.color = Some(color);
    }
    if let Some(color) = get(a, "fo:background-color").and_then(model::parse_color) {
        fmt.background = Some(color);
    }
    if let Some(size) = get(a, "fo:font-size").and_then(length).filter(|size| (1.0..=1638.0).contains(size)) {
        fmt.size = Some(size);
    }
    if let Some(family) = get(a, "fo:font-family").or_else(|| get(a, "style:font-name")) {
        let family = family.split(',').next().unwrap_or("").trim().trim_matches(['\'', '"']).to_string();
        if model::is_mono(&family) || get(a, "style:font-pitch").is_some_and(|pitch| pitch.trim().eq_ignore_ascii_case("fixed")) {
            fmt.code = Some(true);
        } else if !family.is_empty() {
            fmt.font = model::kept_font(&family);
        }
    }
}

/// The paragraph kind a style gives by its name (Heading 1, Title, Quotations, Preformatted
/// Text…) or its outline level; `None` for a plain paragraph style.
fn kind_of(name: &str, s: &StyleInfo) -> Option<Kind> {
    let dn = s.display.as_deref().unwrap_or(name).to_ascii_lowercase().replace("_20_", " ");
    let nn = name.to_ascii_lowercase().replace("_20_", " ");
    let mut kind = None;
    for cand in [dn.as_str(), nn.as_str()] {
        if let Some(d) = cand.strip_prefix("heading ").and_then(|r| r.trim().parse::<u8>().ok()) {
            kind = Some(Kind::Heading(d.clamp(1, 6)));
        } else if cand == "title" {
            kind = Some(Kind::Title);
        } else if cand == "quotations" || cand == "quote" {
            kind = Some(Kind::Quote);
        } else if cand == "preformatted text" || cand == "source text" || cand == "code" {
            kind = Some(Kind::Code);
        }
    }
    kind.or_else(|| s.outline.filter(|o| (1..=10).contains(o)).map(|o| Kind::Heading(o.min(6))))
}

#[derive(Default)]
pub(super) struct Styles {
    map: HashMap<String, StyleInfo>,
    pub(super) lists: HashMap<String, Vec<bool>>,
    /// Font of `style:default-style` for paragraphs (the document default, ODF 1.3 §16.4).
    default_font: Option<String>,
}

impl Styles {
    pub(super) fn read(&mut self, xml: &[u8], auto_section_is_auto: bool) {
        let mut in_auto = false;
        let mut cur: Option<(String, StyleInfo)> = None;
        let mut in_default = false;
        let mut list: Option<(String, Vec<bool>)> = None;
        events(xml, |ev| match ev {
            Ev::Start(q, a) => match local(&q) {
                "automatic-styles" => in_auto = true,
                "style" if q.starts_with("style:") => {
                    if let Some(name) = get(&a, "style:name") {
                        let info = StyleInfo {
                            parent: get(&a, "style:parent-style-name").map(String::from),
                            display: get(&a, "style:display-name").map(String::from),
                            auto: in_auto && auto_section_is_auto,
                            outline: get(&a, "style:default-outline-level").and_then(|v| v.trim().parse::<u8>().ok()),
                            master_page: get(&a, "style:master-page-name").map(String::from),
                            ..Default::default()
                        };
                        cur = Some((name.to_string(), info));
                    }
                }
                "default-style" if q.starts_with("style:") => in_default = get(&a, "style:family") == Some("paragraph"),
                "text-properties" if in_default && cur.is_none() => {
                    self.default_font = default_font(&a);
                }
                "text-properties" => {
                    if let Some((_, s)) = &mut cur {
                        text_properties(&a, s);
                    }
                }
                "table-properties" => {
                    if let Some((_, s)) = &mut cur {
                        table_properties(&a, s);
                    }
                }
                "table-column-properties" => {
                    if let Some((_, s)) = &mut cur
                        && let Some(w) = get(&a, "style:column-width").and_then(length)
                    {
                        s.column_width = Some(w);
                    }
                }
                "table-cell-properties" => {
                    if let Some((_, s)) = &mut cur {
                        if let Some(c) = get(&a, "fo:background-color").and_then(model::parse_color) {
                            s.cell_background = Some(c);
                        }
                        s.cell_border = cell_border(&a).or(s.cell_border);
                        if let Some(va) = get(&a, "style:vertical-align") {
                            s.cell_valign = match va.trim() {
                                "top" => Some(VAlign::Top),
                                "middle" | "center" => Some(VAlign::Center),
                                "bottom" => Some(VAlign::Bottom),
                                _ => None,
                            };
                        }
                    }
                }
                "paragraph-properties" => {
                    if let Some((_, s)) = &mut cur {
                        paragraph_properties(&a, s);
                    }
                }
                "tab-stops" | "tab-stop" if q.starts_with("style:") => {
                    if let Some((_, s)) = &mut cur {
                        tab_element(s, local(&q), &a);
                    }
                }
                "columns" if q.starts_with("style:") => {
                    if let Some((_, s)) = &mut cur {
                        s.columns = Some(columns_of(&a));
                    }
                }
                "column-sep" => {
                    if let Some((_, s)) = &mut cur
                        && let Some(c) = &mut s.columns
                    {
                        c.separator = column_sep(&a);
                    }
                }
                "list-style" => list = get(&a, "style:name").map(|n| (n.to_string(), vec![false; 10])),
                "list-level-style-number" | "list-level-style-bullet" | "list-level-style-image" => {
                    if let Some((_, v)) = &mut list {
                        let lv = get(&a, "text:level").and_then(|l| l.trim().parse::<usize>().ok()).unwrap_or(1).clamp(1, 10);
                        let ordered = local(&q) == "list-level-style-number" && get(&a, "style:num-format").is_some_and(|f| !f.is_empty());
                        if let Some(slot) = v.get_mut(lv - 1) {
                            *slot = ordered;
                        }
                    }
                }
                _ => {}
            },
            Ev::End(q) => match local(&q) {
                "automatic-styles" => in_auto = false,
                "default-style" if q.starts_with("style:") => in_default = false,
                "style" if q.starts_with("style:") => {
                    if let Some((n, s)) = cur.take()
                        && self.map.len() < 100_000
                    {
                        self.map.insert(n, s);
                    }
                }
                "list-style" => {
                    if let Some((n, v)) = list.take() {
                        self.lists.insert(n, v);
                    }
                }
                _ => {}
            },
            Ev::Text(_) => {}
        });
    }

    /// The document's default font: the Standard paragraph style's, else the default style's.
    pub(super) fn doc_font(&self) -> Option<String> {
        self.map.get("Standard").filter(|s| !s.auto).and_then(|s| s.fmt.font.as_deref().map(str::to_string)).or_else(|| self.default_font.clone())
    }

    fn chain<'a>(&'a self, name: &'a str) -> Vec<(&'a str, &'a StyleInfo)> {
        let mut out = Vec::new();
        let mut cur = Some(name);
        while let Some(n) = cur {
            if out.len() >= 16 {
                break;
            }
            let Some(s) = self.map.get(n) else { break };
            if out.iter().any(|(m, _): &(&str, &StyleInfo)| *m == n) {
                break;
            }
            out.push((n, s));
            cur = s.parent.as_deref();
        }
        out
    }

    /// Text formatting of a text style (whole chain).
    pub(super) fn text_fmt(&self, name: &str, base: &Fmt) -> Fmt {
        let mut f = base.clone();
        for (_, s) in self.chain(name).iter().rev() {
            s.fmt.apply(&mut f);
        }
        f
    }

    /// The page a paragraph style starts (`style:master-page-name` and `style:page-number`,
    /// ODF 1.3 §16.2 and §17.6): the nearest master page and page-number setting in its chain.
    pub(super) fn page_start<'a>(&'a self, name: &'a str) -> (Option<&'a str>, Option<Option<u32>>) {
        let chain = self.chain(name);
        let master = chain.iter().find_map(|(_, s)| s.master_page.as_deref()).filter(|m| !m.is_empty());
        (master, chain.iter().find_map(|(_, s)| s.page_number))
    }

    /// Columns of a section style (whole chain).
    pub(super) fn columns(&self, name: &str) -> Option<Columns> {
        self.chain(name).iter().find_map(|(_, s)| s.columns.clone())
    }

    /// Width in points of a table-column style (whole chain).
    pub(super) fn column_width(&self, name: &str) -> Option<f32> {
        self.chain(name).iter().find_map(|(_, s)| s.column_width)
    }

    /// Background colour of a table-cell style (whole chain).
    pub(super) fn cell_background(&self, name: &str) -> Option<Rgb> {
        self.chain(name).iter().find_map(|(_, s)| s.cell_background)
    }

    /// The alignment of a table style (whole chain).
    pub(super) fn table_align(&self, name: &str) -> Option<Align> {
        self.chain(name).iter().find_map(|(_, s)| s.table_align)
    }

    /// Whether a table-cell style draws a border (whole chain; `None`: it doesn't say).
    pub(super) fn cell_border(&self, name: &str) -> Option<bool> {
        self.chain(name).iter().find_map(|(_, s)| s.cell_border)
    }

    /// Vertical alignment of a table-cell style (whole chain).
    pub(super) fn cell_valign(&self, name: &str) -> Option<VAlign> {
        self.chain(name).iter().find_map(|(_, s)| s.cell_valign)
    }

    /// What a paragraph's own styles give it as direct formatting: spacing before and after, line
    /// height and borders from every style in its chain up to the one that gives its kind (as
    /// [`Styles::para`] takes text formatting; `Standard`'s are document defaults), and indents
    /// and tab stops from its automatic styles only (a common style's indents would fight a
    /// list's). Each is the nearest in the chain. Tab positions are from the text column's left
    /// edge, as in Word.
    pub(super) fn para_direct(&self, name: &str) -> ParaDirect {
        let chain = self.chain(name);
        let own: Vec<&StyleInfo> =
            chain.iter().take_while(|(n, s)| kind_of(n, s).is_none()).filter(|(n, s)| s.auto || *n != "Standard").map(|(_, s)| *s).collect();
        let auto: Vec<&StyleInfo> = chain.iter().take_while(|(_, s)| s.auto).map(|(_, s)| *s).collect();
        let pick = |from: &[&StyleInfo], f: &dyn Fn(&model::Spacing) -> Option<f32>| from.iter().find_map(|s| f(&s.spacing));
        let spacing = model::Spacing {
            before: pick(&own, &|x| x.before),
            after: pick(&own, &|x| x.after),
            left: pick(&auto, &|x| x.left),
            right: pick(&auto, &|x| x.right),
            first: pick(&auto, &|x| x.first),
            line: own.iter().find_map(|s| s.spacing.line),
        };
        let indent = spacing.left.unwrap_or(0.0);
        let tabs = auto
            .iter()
            .find_map(|s| s.tabs.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|mut t| {
                t.pos = (t.pos + indent).clamp(0.0, MAX_SPACING);
                t
            })
            .collect();
        let borders = own.iter().find_map(|s| s.borders);
        ParaDirect { spacing, tabs, borders }
    }

    /// Paragraph kind, alignment, page break, rule, keep-with-next and base formatting. The text formatting of
    /// every style in the chain applies (automatic and common, ODF 1.3 §16.2 inheritance), except
    /// the style that gives the paragraph its kind and that style's ancestors: their formatting
    /// is the kind's own look (a heading's size and colour), not direct formatting. `Standard`,
    /// the default paragraph style, holds document defaults and is left out too.
    pub(super) fn para(&self, name: &str) -> (Kind, Option<Align>, bool, bool, bool, Fmt) {
        let chain = self.chain(name);
        let kind_at = chain.iter().position(|(n, s)| kind_of(n, s).is_some()).unwrap_or(chain.len());
        let kind = chain.get(kind_at).and_then(|(n, s)| kind_of(n, s)).unwrap_or(Kind::Normal);
        let align = chain.iter().find_map(|(_, s)| s.align);
        let pb = chain.iter().take_while(|(_, s)| s.auto).any(|(_, s)| s.page_break);
        let rule = chain.iter().any(|(_, s)| s.border_bottom);
        let mut f = Fmt::default();
        for (_, s) in chain.iter().take(kind_at).rev().filter(|(n, s)| s.auto || *n != "Standard") {
            s.fmt.apply(&mut f);
        }
        // Like the text formatting, the kind style's own `fo:keep-with-next` (a heading's) is the kind's look, not direct formatting.
        let keep_next = chain.iter().take(kind_at).find_map(|(_, s)| s.keep_next).unwrap_or(false);
        (kind, align, pb, rule, keep_next, f)
    }
}
