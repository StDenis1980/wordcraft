//! Tinta Office: what the .odt export keeps of a letter, an invoice or a CV made in Word's way
//! (headers and footers with page numbers, paragraph spacing, indents, tab stops, borders,
//! borderless and aligned tables), checked in the XML and back through the importer.

use std::io::Read;

use wordcraft_doc::para::InlineObject;
use wordcraft_doc::props::{Border, BorderStyle, Borders, CharProps, LineSpacing, TabAlign, TabLeader, TabStop};
use wordcraft_doc::{Block, Document, Paragraph, PartKind, StoryRef, Table, para_block};

use super::{Align, Rgb, export, import};

fn part(bytes: &[u8], name: &str) -> String {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut s = String::new();
    z.by_name(name).unwrap().read_to_string(&mut s).unwrap();
    s
}

fn field(instr: &str, result: &str) -> InlineObject {
    InlineObject::Field { instr: instr.into(), result: result.into(), locked: false, dirty: false, code: false, form: String::new() }
}

/// "Página 1 de 1" with both numbers as fields.
fn page_x_of_y() -> Paragraph {
    let mut p = Paragraph::with_text("Página ", CharProps::default());
    let at = p.len();
    let _ = p.insert_object(at, field("PAGE", "1"), &CharProps::default());
    let at = p.len();
    let _ = p.insert_text(at, " de ", &CharProps::default());
    let at = p.len();
    let _ = p.insert_object(at, field(" NUMPAGES \\* MERGEFORMAT ", "1"), &CharProps::default());
    p
}

fn letter() -> Document {
    let mut d = Document::new();
    let s = &mut d.last_section;
    (s.page_w, s.page_h, s.margin_top, s.margin_bottom, s.header, s.footer) = (595.3, 841.9, 70.9, 56.7, 35.4, 28.35);
    d.body = vec![para_block(Paragraph::with_text("Cuerpo", CharProps::default()))];
    let header = d.add_part(PartKind::Header, vec![para_block(Paragraph::with_text("Membrete", CharProps::default()))]);
    let footer = d.add_part(PartKind::Footer, vec![para_block(page_x_of_y())]);
    d.last_section.headers.default = Some(header);
    d.last_section.footers.default = Some(footer);
    d
}

#[test]
fn headers_and_footers_are_written_on_the_master_page_with_page_fields() {
    let bytes = export(&letter()).unwrap();
    let styles = part(&bytes, "styles.xml");
    assert!(styles.contains("<style:header>"), "{styles}");
    assert!(styles.contains("Membrete"), "{styles}");
    assert!(styles.contains("<text:page-number text:select-page=\"current\">1</text:page-number>"), "{styles}");
    assert!(styles.contains("<text:page-count>1</text:page-count>"), "{styles}");
    // The body's automatic styles stay in content.xml; the footer's are its own.
    assert!(!part(&bytes, "content.xml").contains("Membrete"));
}

#[test]
fn headers_footers_page_fields_and_margins_come_back_from_odt() {
    let d = letter();
    let back = import(&export(&d).unwrap()).unwrap();
    let s = &back.last_section;
    assert_eq!(back.plain_text(StoryRef::Part(s.headers.default.unwrap())).trim(), "Membrete");
    let footer = back.parts.get(&s.footers.default.unwrap()).unwrap();
    let Some(Block::Para(p)) = footer.blocks.first().map(|b| &**b) else { panic!("no footer paragraph") };
    let fields: Vec<&str> =
        p.objects.iter().filter_map(|o| if let InlineObject::Field { instr, .. } = o { Some(instr.as_str()) } else { None }).collect();
    assert_eq!(fields, ["PAGE", "NUMPAGES"]);
    // Word's margins and header/footer distances, not ODF's page margins.
    for (got, want) in [(s.margin_top, 70.9), (s.margin_bottom, 56.7), (s.header, 35.4), (s.footer, 28.35)] {
        assert!((got - want).abs() < 0.05, "{got} != {want}");
    }
}

#[test]
fn a_document_without_headers_writes_bare_master_pages() {
    let mut d = letter();
    d.last_section.headers = Default::default();
    d.last_section.footers = Default::default();
    let styles = part(&export(&d).unwrap(), "styles.xml");
    assert!(!styles.contains("style:header") && !styles.contains("style:footer"), "{styles}");
    assert!(styles.contains("fo:margin-top=\"70.9pt\""), "{styles}");
}

#[test]
fn a_different_first_page_and_even_pages_get_their_own_or_empty_headers() {
    let mut d = letter();
    let first = d.add_part(PartKind::Header, vec![para_block(Paragraph::with_text("Portada", CharProps::default()))]);
    d.last_section.headers.first = Some(first);
    d.last_section.title_page = true;
    d.settings.even_odd_headers = true;
    let styles = part(&export(&d).unwrap(), "styles.xml");
    assert!(styles.contains("<style:header-first>") && styles.contains("Portada"), "{styles}");
    // No even header of its own: an empty one, as Word leaves those pages.
    assert!(styles.contains("<style:header-left><text:p text:style-name=\"Standard\"/></style:header-left>"), "{styles}");
    let back = import(&export(&d).unwrap()).unwrap();
    assert_eq!(back.plain_text(StoryRef::Part(back.last_section.headers.first.unwrap())).trim(), "Portada");
}

#[test]
fn paragraph_spacing_indents_tabs_and_borders_are_written() {
    let mut d = Document::new();
    let mut p = Paragraph::with_text("Ref.\tExpediente", CharProps::default());
    p.props.space_before = Some(12.0);
    p.props.space_after = Some(4.0);
    p.props.indent_left = Some(36.0);
    p.props.indent_first = Some(28.35);
    p.props.line_spacing = Some(LineSpacing::Multiple(1.15));
    p.props.tabs = Some(vec![TabStop { pos: 236.0, align: TabAlign::Right, leader: TabLeader::Dot }]);
    p.props.borders = Some(Borders {
        bottom: Some(Border { style: BorderStyle::Single, width: 0.75, color: Some(Rgb(0x1F, 0x38, 0x64)), space: 2.0 }),
        ..Default::default()
    });
    d.body = vec![para_block(p)];
    let content = part(&export(&d).unwrap(), "content.xml");
    for want in [
        "fo:margin-top=\"12pt\"",
        "fo:margin-bottom=\"4pt\"",
        "fo:margin-left=\"36pt\"",
        "fo:text-indent=\"28.35pt\"",
        "fo:line-height=\"115%\"",
        "fo:border-bottom=\"0.75pt solid #1f3864\"",
        "fo:padding-bottom=\"2pt\"",
        // From the indent, as ODF measures it.
        "<style:tab-stops><style:tab-stop style:position=\"200pt\" style:type=\"right\" style:leader-style=\"dotted\" style:leader-text=\".\"/></style:tab-stops></style:paragraph-properties>",
    ] {
        assert!(content.contains(want), "{want} in {content}");
    }
}

#[test]
fn hostile_lengths_never_reach_the_xml() {
    let mut d = Document::new();
    let mut p = Paragraph::with_text("x", CharProps::default());
    p.props.space_before = Some(f32::NAN);
    p.props.indent_left = Some(f32::INFINITY);
    p.props.indent_first = Some(-1e30);
    p.props.line_spacing = Some(LineSpacing::Multiple(f32::NAN));
    p.props.tabs = Some(vec![TabStop { pos: f32::NAN, align: TabAlign::Left, leader: TabLeader::None }; 1000]);
    p.props.borders = Some(Borders::all(Border { style: BorderStyle::Single, width: f32::NAN, color: None, space: f32::INFINITY }));
    d.body = vec![para_block(p)];
    let content = part(&export(&d).unwrap(), "content.xml");
    assert!(!content.to_ascii_lowercase().contains("nan") && !content.contains("inf"), "{content}");
    assert!(content.contains("fo:text-indent=\"-1584pt\""), "{content}");
    assert!(import(&export(&d).unwrap()).is_ok());
}

#[test]
fn a_layout_table_without_borders_has_no_grid_and_keeps_its_alignment() {
    let mut d = Document::new();
    let mut t = Table::new(2, 2, 270.0);
    t.props.borders = Some(Borders::all(Border { style: BorderStyle::None, width: 0.0, color: None, space: 0.0 }));
    t.props.align = Some(Align::Right);
    let grid = Table::new(1, 1, 100.0);
    d.body = vec![std::sync::Arc::new(Block::Table(t)), std::sync::Arc::new(Block::Table(grid))];
    let content = part(&export(&d).unwrap(), "content.xml");
    assert!(content.contains("table:align=\"right\""), "{content}");
    assert!(content.contains("fo:border=\"none\""), "the layout table: {content}");
    assert!(content.contains("fo:border=\"0.5pt solid #000000\""), "the grid table keeps its grid: {content}");
}
