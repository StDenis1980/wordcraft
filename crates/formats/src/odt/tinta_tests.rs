//! Tinta Office: what the .odt export keeps of a letter, an invoice or a CV made in Word's way
//! (headers and footers with page numbers, paragraph spacing, indents, tab stops, borders,
//! borderless and aligned tables), checked in the XML and back through the importer; and what
//! the importer reads of a paragraph's own spacing, indents, line height and tab stops.

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

fn package(content: &str) -> Vec<u8> {
    use std::io::Write;
    let mut z = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let o = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, data) in [("mimetype", "application/vnd.oasis.opendocument.text"), ("content.xml", content)] {
        z.start_file(name, o).unwrap();
        z.write_all(data.as_bytes()).unwrap();
    }
    z.finish().unwrap().into_inner()
}

const NS: &str = r#"xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0""#;

/// The body paragraphs' properties of an .odt whose `office:automatic-styles` and body are given.
fn imported(auto: &str, body: &str) -> Vec<wordcraft_doc::props::ParaProps> {
    let content = format!(
        r#"<office:document-content {NS}><office:automatic-styles>{auto}</office:automatic-styles><office:body><office:text>{body}</office:text></office:body></office:document-content>"#
    );
    let d = import(&package(&content)).unwrap();
    d.body.iter().filter_map(|b| if let Block::Para(p) = &**b { Some(p.props.clone()) } else { None }).collect()
}

#[test]
fn paragraph_spacing_indents_line_height_and_tabs_are_read() {
    let auto = r#"<style:style style:name="P1" style:family="paragraph" style:parent-style-name="Standard"><style:paragraph-properties fo:margin-top="0.4236cm" fo:margin-bottom="4pt" fo:margin-left="1.27cm" fo:margin-right="0.5in" fo:text-indent="-18pt" fo:line-height="150%"><style:tab-stops><style:tab-stop style:position="200pt" style:type="right" style:leader-style="dotted" style:leader-text="."/><style:tab-stop style:position="1cm" style:type="center"/><style:tab-stop style:position="3cm" style:type="char" style:char=","/></style:tab-stops></style:paragraph-properties></style:style>
        <style:style style:name="P2" style:family="paragraph"><style:paragraph-properties fo:line-height="14pt"/></style:style>
        <style:style style:name="P3" style:family="paragraph"><style:paragraph-properties style:line-height-at-least="16pt"/></style:style>"#;
    let p = imported(
        auto,
        r#"<text:p text:style-name="P1">Ref.<text:tab/>Exp.</text:p><text:p text:style-name="P2">b</text:p><text:p text:style-name="P3">c</text:p><text:p>d</text:p>"#,
    );
    let close = |a: Option<f32>, b: f32| a.is_some_and(|a| (a - b).abs() < 0.05);
    assert!(close(p[0].space_before, 12.0) && close(p[0].space_after, 4.0), "{:?}", p[0]);
    assert!(close(p[0].indent_left, 36.0) && close(p[0].indent_right, 36.0) && close(p[0].indent_first, -18.0), "{:?}", p[0]);
    assert_eq!(p[0].line_spacing, Some(LineSpacing::Multiple(1.5)));
    let tabs = p[0].tabs.clone().unwrap();
    // From the text column's left edge, as Word measures them: the indent is added back.
    assert!(close(Some(tabs[0].pos), 236.0) && tabs[0].align == TabAlign::Right && tabs[0].leader == TabLeader::Dot, "{tabs:?}");
    assert!(tabs[1].align == TabAlign::Center && tabs[1].leader == TabLeader::None && tabs[2].align == TabAlign::Decimal, "{tabs:?}");
    assert_eq!(p[1].line_spacing, Some(LineSpacing::Exactly(14.0)));
    assert_eq!(p[2].line_spacing, Some(LineSpacing::AtLeast(16.0)));
    assert!(p[3].space_before.is_none() && p[3].tabs.is_none() && p[3].line_spacing.is_none());
}

/// An .odt with these common styles in `styles.xml` and this body.
fn with_styles(common: &str, body: &str) -> Document {
    let content = format!(r#"<office:document-content {NS}><office:body><office:text>{body}</office:text></office:body></office:document-content>"#);
    let styles = format!(r#"<office:document-styles {NS}><office:styles>{common}</office:styles></office:document-styles>"#);
    use std::io::Write;
    let mut z = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let o = zip::write::SimpleFileOptions::default();
    for (name, data) in [("mimetype", "application/vnd.oasis.opendocument.text"), ("content.xml", &content), ("styles.xml", &styles)] {
        z.start_file(name, o).unwrap();
        z.write_all(data.as_bytes()).unwrap();
    }
    import(&z.finish().unwrap().into_inner()).unwrap()
}

fn props(d: &Document) -> Vec<wordcraft_doc::props::ParaProps> {
    d.body.iter().filter_map(|b| if let Block::Para(p) = &**b { Some(p.props.clone()) } else { None }).collect()
}

/// A plain common style's spacing before and after and line height reach its paragraphs (as its
/// text formatting does); a heading style's are the heading's look, `Standard`'s the document
/// defaults, and a common style's indents stay with the style (they would fight a list's).
#[test]
fn a_common_styles_spacing_applies_but_not_a_headings_or_standards() {
    let common = r#"<style:style style:name="Standard" style:family="paragraph"><style:paragraph-properties fo:margin-bottom="8pt" fo:line-height="115%"/></style:style>
        <style:style style:name="Text_20_body" style:display-name="Text body" style:family="paragraph" style:parent-style-name="Standard"><style:paragraph-properties fo:margin-top="3pt" fo:margin-bottom="7pt" fo:line-height="120%" fo:margin-left="20pt"/></style:style>
        <style:style style:name="Heading_20_1" style:display-name="Heading 1" style:family="paragraph" style:parent-style-name="Standard"><style:paragraph-properties fo:margin-top="30pt"/></style:style>"#;
    let p = props(&with_styles(
        common,
        r#"<text:p text:style-name="Text_20_body">a</text:p><text:h text:style-name="Heading_20_1" text:outline-level="1">b</text:h><text:p text:style-name="Standard">c</text:p>"#,
    ));
    assert_eq!((p[0].space_before, p[0].space_after, p[0].line_spacing), (Some(3.0), Some(7.0), Some(LineSpacing::Multiple(1.2))), "{:?}", p[0]);
    assert!(p[0].indent_left.is_none(), "{:?}", p[0]);
    assert!(p[1].space_before.is_none(), "the heading's: {:?}", p[1]);
    assert!(p[2].space_after.is_none() && p[2].line_spacing.is_none(), "Standard's: {:?}", p[2]);
}

/// A paragraph's borders come back with their colour, width and distance from the text: LibreOffice
/// writes Word's heading rule as `fo:border-bottom` on the paragraph's style.
#[test]
fn paragraph_borders_are_read() {
    let auto = r#"<style:style style:name="P5" style:family="paragraph" style:parent-style-name="Standard"><style:paragraph-properties fo:padding-bottom="0.028in" fo:border-left="none" fo:border-right="none" fo:border-top="none" fo:border-bottom="0.74pt solid #1f3864"/></style:style>
        <style:style style:name="P6" style:family="paragraph"><style:paragraph-properties fo:border="thin double #ff0000" fo:padding="4pt"/></style:style>
        <style:style style:name="P7" style:family="paragraph"><style:paragraph-properties fo:border="none"/></style:style>"#;
    let p = imported(
        auto,
        r#"<text:p text:style-name="P5">Descripción</text:p><text:p text:style-name="P6">x</text:p><text:p text:style-name="P7">y</text:p>"#,
    );
    let b = p[0].borders.unwrap();
    let bottom = b.bottom.unwrap();
    assert!(bottom.style == BorderStyle::Single && (bottom.width - 0.74).abs() < 0.01 && bottom.color == Some(Rgb(0x1F, 0x38, 0x64)), "{bottom:?}");
    assert!((bottom.space - 2.016).abs() < 0.01, "{bottom:?}");
    assert!(b.top.is_some_and(|t| !t.is_visible()), "{b:?}");
    let all = p[1].borders.unwrap();
    for e in [all.top, all.bottom, all.left, all.right] {
        let e = e.unwrap();
        assert!(e.style == BorderStyle::Double && e.width == 0.5 && e.space == 4.0 && e.color == Some(Rgb(0xFF, 0, 0)), "{e:?}");
    }
    assert!(p[2].borders.is_none(), "nothing visible: {:?}", p[2]);
}

#[test]
fn hostile_spacing_and_tabs_are_clamped() {
    let mut stops = String::new();
    for _ in 0..500 {
        stops.push_str(r#"<style:tab-stop style:position="1e30pt"/><style:tab-stop style:position="NaNpt"/>"#);
    }
    let auto = format!(
        r#"<style:style style:name="P1" style:family="paragraph"><style:paragraph-properties fo:margin-top="1e30pt" fo:margin-left="-1e30pt" fo:text-indent="infpt" fo:margin-bottom="-5pt" fo:line-height="1e9%" fo:margin-right="50%" fo:border="1e30pt solid #zzzzzz NaNpt" fo:padding="1e30pt"><style:tab-stops>{stops}</style:tab-stops></style:paragraph-properties></style:style>
        <style:style style:name="P2" style:family="paragraph"><style:paragraph-properties fo:line-height="-3%"/></style:style>"#
    );
    let p = imported(&auto, r#"<text:p text:style-name="P1">x</text:p><text:p text:style-name="P2">y</text:p>"#);
    assert_eq!(p[0].space_before, Some(1584.0));
    assert_eq!(p[0].space_after, Some(0.0));
    assert_eq!(p[0].indent_left, Some(-1584.0));
    assert!(p[0].indent_first.is_none() && p[0].indent_right.is_none(), "{:?}", p[0]);
    assert_eq!(p[0].line_spacing, Some(LineSpacing::Multiple(10.0)));
    let tabs = p[0].tabs.clone().unwrap();
    assert!(tabs.len() <= 64 && tabs.iter().all(|t| t.pos.is_finite() && (0.0..=1584.0).contains(&t.pos)), "{}", tabs.len());
    assert!(p[1].line_spacing.is_none());
    let b = p[0].borders.unwrap().bottom.unwrap();
    assert!(b.width == 12.0 && b.space == 31.0 && b.color.is_none(), "{b:?}");
}

/// What the export writes comes back: the round trip the Spain templates take.
#[test]
fn spacing_indents_and_tabs_survive_odt() {
    let mut d = Document::new();
    let mut p = Paragraph::with_text("Ref.\tExpediente", CharProps::default());
    p.props.space_before = Some(12.0);
    p.props.space_after = Some(4.0);
    p.props.indent_left = Some(36.0);
    p.props.indent_first = Some(28.35);
    p.props.line_spacing = Some(LineSpacing::Multiple(1.15));
    p.props.tabs = Some(vec![TabStop { pos: 236.0, align: TabAlign::Right, leader: TabLeader::Dot }]);
    d.body = vec![para_block(p.clone())];
    let back = import(&export(&d).unwrap()).unwrap();
    let Block::Para(q) = &*back.body[0] else { panic!() };
    let close = |a: Option<f32>, b: Option<f32>| match (a, b) {
        (Some(a), Some(b)) => (a - b).abs() < 0.05,
        (a, b) => a == b,
    };
    assert!(close(q.props.space_before, p.props.space_before) && close(q.props.space_after, p.props.space_after), "{:?}", q.props);
    assert!(close(q.props.indent_left, p.props.indent_left) && close(q.props.indent_first, p.props.indent_first), "{:?}", q.props);
    assert_eq!(q.props.line_spacing, p.props.line_spacing);
    let tabs = q.props.tabs.clone().unwrap();
    assert!(close(Some(tabs[0].pos), Some(236.0)) && tabs[0].align == TabAlign::Right && tabs[0].leader == TabLeader::Dot, "{tabs:?}");
}

/// A layout table whose cells all say `fo:border="none"` (LibreOffice writes Word's borderless
/// tables so) opens without a grid; a table with any bordered or unstyled cell keeps it. A
/// table's alignment comes back too.
#[test]
fn a_table_whose_cells_have_no_borders_opens_without_a_grid() {
    let auto = r#"<style:style style:name="N" style:family="table-cell"><style:table-cell-properties fo:border="none"/></style:style>
        <style:style style:name="E" style:family="table-cell"><style:table-cell-properties fo:border-top="none" fo:border-bottom="none" fo:border-left="none" fo:border-right="none"/></style:style>
        <style:style style:name="G" style:family="table-cell"><style:table-cell-properties fo:border="0.5pt solid #000000"/></style:style>"#;
    let table = |a: &str, b: &str| {
        format!(
            r#"<table:table><table:table-column table:number-columns-repeated="2"/><table:table-row><table:table-cell table:style-name="{a}"><text:p>a</text:p></table:table-cell><table:table-cell{b}><text:p>b</text:p></table:table-cell></table:table-row></table:table>"#
        )
    };
    let body = [table("N", r#" table:style-name="E""#), table("N", r#" table:style-name="G""#), table("N", "")].concat().replacen(
        "<table:table>",
        r#"<table:table table:style-name="T">"#,
        1,
    );
    let auto = format!(r#"{auto}<style:style style:name="T" style:family="table"><style:table-properties table:align="right"/></style:style>"#);
    let content = format!(
        r#"<office:document-content {NS} xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0"><office:automatic-styles>{auto}</office:automatic-styles><office:body><office:text>{body}</office:text></office:body></office:document-content>"#
    );
    let d = import(&package(&content)).unwrap();
    let styles: Vec<Option<String>> =
        d.body.iter().filter_map(|b| if let Block::Table(t) = &**b { Some(t.props.style.clone()) } else { None }).collect();
    assert_eq!(styles.len(), 3);
    assert!(styles[0].is_none(), "the layout table: {styles:?}");
    assert!(styles[1].is_some() && styles[2].is_some(), "the grids stay: {styles:?}");
}

/// The cells' margins of the first table in an .odt whose `office:automatic-styles` and one row's
/// cells are given.
fn cell_margins(auto: &str, cells: &str) -> Vec<Option<[f32; 4]>> {
    let content = format!(
        r#"<office:document-content {NS} xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0"><office:automatic-styles>{auto}</office:automatic-styles><office:body><office:text><table:table><table:table-column table:number-columns-repeated="4"/><table:table-row>{cells}</table:table-row></table:table></office:text></office:body></office:document-content>"#
    );
    let d = import(&package(&content)).unwrap();
    let Some(Block::Table(t)) = d.body.iter().map(|b| &**b).find(|b| matches!(b, Block::Table(_))) else { panic!("no table") };
    t.rows[0].cells.iter().map(|c| c.props.margins).collect()
}

#[test]
fn cell_padding_is_read_as_cell_margins() {
    // LibreOffice's default (0.097cm on every side), one side of four, Word's own and none.
    let auto = r#"<style:style style:name="L" style:family="table-cell"><style:table-cell-properties fo:padding="0.097cm" fo:border="0.5pt solid #000000"/></style:style>
        <style:style style:name="S" style:family="table-cell"><style:table-cell-properties fo:padding="0.1in" fo:padding-left="0.25in" fo:padding-right="0pt"/></style:style>
        <style:style style:name="W" style:family="table-cell"><style:table-cell-properties fo:padding-top="0pt" fo:padding-bottom="0pt" fo:padding-left="5.4pt" fo:padding-right="5.4pt"/></style:style>"#;
    let cell = |s: &str| format!(r#"<table:table-cell{s}><text:p>x</text:p></table:table-cell>"#);
    let cells = [cell(r#" table:style-name="L""#), cell(r#" table:style-name="S""#), cell(r#" table:style-name="W""#), cell("")].concat();
    let m = cell_margins(auto, &cells);
    let l = m[0].expect("LibreOffice's padding");
    assert!(l.iter().all(|v| (v - 2.75).abs() < 0.01), "{l:?}");
    let s = m[1].expect("one side of four");
    assert!(s.iter().zip([7.2, 18.0, 7.2, 0.0]).all(|(v, want)| (v - want).abs() < 0.01), "top, left, bottom, right: {s:?}");
    assert_eq!(m[2], None, "Word's default margins add nothing");
    assert_eq!(m[3], None, "no style: Word's default");
    // Hostile paddings are clamped, never negative or infinite.
    let hostile = r#"<style:style style:name="H" style:family="table-cell"><style:table-cell-properties fo:padding="1e30pt" fo:padding-left="-5pt" fo:padding-top="NaNpt"/></style:style>"#;
    let h = cell_margins(hostile, &cell(r#" table:style-name="H""#))[0].expect("clamped");
    assert!(h.iter().all(|v| v.is_finite() && (0.0..=144.0).contains(v)), "{h:?}");
}

#[test]
fn cell_margins_survive_odt_and_padding_is_written_once() {
    let mut d = Document::new();
    let mut t = Table::new(1, 2, 270.0);
    t.props.cell_margins = Some([3.0, 10.0, 3.0, 10.0]);
    // A cell border with a distance used to write a second `fo:padding-top`.
    let edge = Border { style: BorderStyle::Single, width: 1.0, color: None, space: 4.0 };
    t.rows[0].cells[1].props.borders = Some(Borders::all(edge));
    t.rows[0].cells[1].props.margins = Some([1.0, 2.0, 3.0, 4.0]);
    d.body = vec![std::sync::Arc::new(Block::Table(t))];
    let bytes = export(&d).unwrap();
    let content = part(&bytes, "content.xml");
    for style in content.split("<style:table-cell-properties").skip(1) {
        let props = style.split("/>").next().unwrap_or("");
        assert_eq!(props.matches("fo:padding-top=").count(), 1, "{props}");
    }
    let back = import(&bytes).unwrap();
    let Some(Block::Table(t)) = back.body.first().map(|b| &**b) else { panic!("no table") };
    assert_eq!(t.rows[0].cells[0].props.margins, Some([3.0, 10.0, 3.0, 10.0]), "the table's margins");
    assert_eq!(t.rows[0].cells[1].props.margins, Some([1.0, 2.0, 3.0, 4.0]), "the cell's own");
    // A table at Word's default writes Word's default and reads back as nothing set.
    let plain = import(
        &export(&{
            let mut d = Document::new();
            d.body = vec![std::sync::Arc::new(Block::Table(Table::new(1, 1, 100.0)))];
            d
        })
        .unwrap(),
    )
    .unwrap();
    let Some(Block::Table(t)) = plain.body.first().map(|b| &**b) else { panic!("no table") };
    assert_eq!(t.rows[0].cells[0].props.margins, None);
}
