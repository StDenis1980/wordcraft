//! Original partial-cell-margin fixture (ECMA-376 §17.4.69).
use std::io::{Cursor, Read, Write};

fn fixture() -> Vec<u8> {
    let document = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:tbl><w:tblPr><w:tblCellMar><w:top w:w="55" w:type="dxa"/><w:left w:w="45" w:type="dxa"/><w:bottom w:w="55" w:type="dxa"/><w:right w:w="55" w:type="dxa"/></w:tblCellMar></w:tblPr><w:tblGrid><w:gridCol w:w="4320"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcMar><w:left w:w="0" w:type="dxa"/></w:tcMar></w:tcPr><w:p><w:r><w:t>Point</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body></w:document>"#;
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file("word/document.xml", zip::write::SimpleFileOptions::default()).unwrap();
    zip.write_all(document.as_bytes()).unwrap();
    zip.finish().unwrap().into_inner()
}

#[test]
fn partial_margin_stays_partial() {
    let doc = wordcraft_docx::read(&fixture()).unwrap();
    let table = doc.body[0].as_table().unwrap();
    assert_eq!(table.rows[0].cells[0].props.margins, None, "a left-only margin is not a complete override");
    assert_eq!(table.rows[0].cells[0].props.resolved_margins([6.0, 7.0, 8.0, 9.0]), [6.0, 0.0, 8.0, 9.0]);
    let bytes = wordcraft_docx::write(&doc).unwrap();
    let reopened = wordcraft_docx::read(&bytes).unwrap();
    let cell = &reopened.body[0].as_table().unwrap().rows[0].cells[0];
    assert_eq!(cell.props.resolved_margins([6.0, 7.0, 8.0, 9.0]), [6.0, 0.0, 8.0, 9.0]);
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut xml = String::new();
    zip.by_name("word/document.xml").unwrap().read_to_string(&mut xml).unwrap();
    let margins = xml.split("<w:tcMar>").nth(1).unwrap().split("</w:tcMar>").next().unwrap();
    assert!(margins.contains("<w:left w:w=\"0\""));
    assert!(!margins.contains("<w:top"));
    assert!(!margins.contains("<w:bottom"));
    assert!(!margins.contains("<w:right"));
}

/// Tinta Office: a table that sets only `w:left` in `w:tblCellMar` keeps its style's right margin
/// (Word's PDFs of Balearic forms wrap each cell's text 5.4 pt before the cell's edge), and a
/// table without a style keeps Word's own 0.08" one.
#[test]
fn a_left_only_table_margin_keeps_the_other_sides() {
    let ns = r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main""#;
    let root = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="r1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let rels = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="s1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"#;
    let styles = format!(
        r#"<w:styles {ns}><w:style w:type="table" w:styleId="Grid"><w:name w:val="Table Grid"/><w:tblPr><w:tblCellMar><w:top w:w="20" w:type="dxa"/><w:left w:w="108" w:type="dxa"/><w:bottom w:w="40" w:type="dxa"/><w:right w:w="160" w:type="dxa"/></w:tblCellMar></w:tblPr></w:style></w:styles>"#
    );
    let table = |style: &str| {
        format!(
            r#"<w:tbl><w:tblPr>{style}<w:tblCellMar><w:left w:w="107" w:type="dxa"/></w:tblCellMar></w:tblPr><w:tblGrid><w:gridCol w:w="4320"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>Cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/>"#
        )
    };
    let document = format!(r#"<w:document {ns}><w:body>{}{}</w:body></w:document>"#, table(r#"<w:tblStyle w:val="Grid"/>"#), table(""));
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in [
        ("_rels/.rels", root),
        ("word/_rels/document.xml.rels", rels),
        ("word/document.xml", document.as_str()),
        ("word/styles.xml", styles.as_str()),
    ] {
        zip.start_file(name, zip::write::SimpleFileOptions::default()).unwrap();
        zip.write_all(data.as_bytes()).unwrap();
    }
    let doc = wordcraft_docx::read(&zip.finish().unwrap().into_inner()).unwrap();
    let margins: Vec<_> = doc.body.iter().filter_map(|b| b.as_table()).map(|t| t.props.cell_margins).collect();
    assert_eq!(
        margins,
        vec![Some([1.0, 5.35, 2.0, 8.0]), Some([0.0, 5.35, 0.0, 5.4])],
        "the left side is the table's, the rest the style's or Word's"
    );
}
