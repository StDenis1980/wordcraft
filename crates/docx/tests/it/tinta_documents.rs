//! Tinta Office's .docx test set: the documents a self-employed person in Spain or Ukraine
//! exchanges every week (an invoice, a service contract, a CV, a formal letter, a quarterly report
//! with a chart, SmartArt and an embedded spreadsheet), written for these tests from ECMA-376 in
//! the shape Word saves them (styles, numbering, settings, headers and footers, notes, fields,
//! pictures, document properties), with no file produced by Word.
//!
//! Each one goes through "open → save → open" and nothing may be lost:
//! - the model reads back equal (after the first save, for the report, whose relationship ids a
//!   save renumbers), and a second save writes the same parts;
//! - every WordprocessingML element and attribute name the original used is still written, and
//!   all of its text survives; every relationship and document property survives; every other
//!   part (pictures, charts, workbooks, SmartArt, embedded objects) comes back unchanged;
//! - the facts a person would check by eye (amounts, clause numbers, accents, Cyrillic, the
//!   header logo, the page number field, the footnote, the link) are still there.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};

use wordcraft_doc::Document;

const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const NS: &str = r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:o="urn:schemas-microsoft-com:office:office""#;

// ---------------------------------------------------------------------------------------------
// The package around each document.

/// Styles shared by the four documents: the usual Normal, headings, title, a table grid, list
/// paragraph, footnote text and reference, hyperlink and header/footer styles.
fn styles(lang: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles {NS}>
<w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Liberation Sans" w:hAnsi="Liberation Sans" w:eastAsia="Liberation Sans" w:cs="Liberation Sans"/><w:sz w:val="22"/><w:szCs w:val="22"/><w:lang w:val="{lang}" w:eastAsia="en-US" w:bidi="ar-SA"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="259" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults>
<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style>
<w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:spacing w:after="240"/><w:contextualSpacing/></w:pPr><w:rPr><w:b/><w:color w:val="1F3864"/><w:spacing w:val="-10"/><w:kern w:val="28"/><w:sz w:val="48"/><w:szCs w:val="48"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:uiPriority w:val="9"/><w:qFormat/><w:pPr><w:keepNext/><w:keepLines/><w:spacing w:before="360" w:after="120"/><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:caps/><w:color w:val="2F5496"/><w:sz w:val="28"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Heading1"/><w:next w:val="Normal"/><w:uiPriority w:val="9"/><w:unhideWhenUsed/><w:qFormat/><w:pPr><w:spacing w:before="200" w:after="80"/><w:outlineLvl w:val="1"/></w:pPr><w:rPr><w:caps w:val="0"/><w:sz w:val="24"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="ListParagraph"><w:name w:val="List Paragraph"/><w:basedOn w:val="Normal"/><w:uiPriority w:val="34"/><w:qFormat/><w:pPr><w:ind w:left="720"/><w:contextualSpacing/></w:pPr></w:style>
<w:style w:type="paragraph" w:styleId="Header"><w:name w:val="header"/><w:basedOn w:val="Normal"/><w:link w:val="HeaderChar"/><w:pPr><w:tabs><w:tab w:val="center" w:pos="4513"/><w:tab w:val="right" w:pos="9026"/></w:tabs><w:spacing w:after="0" w:line="240" w:lineRule="auto"/></w:pPr></w:style>
<w:style w:type="paragraph" w:styleId="Footer"><w:name w:val="footer"/><w:basedOn w:val="Normal"/><w:link w:val="FooterChar"/><w:pPr><w:tabs><w:tab w:val="center" w:pos="4513"/><w:tab w:val="right" w:pos="9026"/></w:tabs><w:spacing w:after="0" w:line="240" w:lineRule="auto"/></w:pPr><w:rPr><w:sz w:val="16"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="FootnoteText"><w:name w:val="footnote text"/><w:basedOn w:val="Normal"/><w:pPr><w:spacing w:after="0" w:line="240" w:lineRule="auto"/></w:pPr><w:rPr><w:sz w:val="18"/></w:rPr></w:style>
<w:style w:type="character" w:default="1" w:styleId="DefaultParagraphFont"><w:name w:val="Default Paragraph Font"/><w:uiPriority w:val="1"/><w:semiHidden/><w:unhideWhenUsed/></w:style>
<w:style w:type="character" w:styleId="HeaderChar"><w:name w:val="Header Char"/><w:basedOn w:val="DefaultParagraphFont"/><w:link w:val="Header"/></w:style>
<w:style w:type="character" w:styleId="FooterChar"><w:name w:val="Footer Char"/><w:basedOn w:val="DefaultParagraphFont"/><w:link w:val="Footer"/><w:rPr><w:sz w:val="16"/></w:rPr></w:style>
<w:style w:type="character" w:styleId="FootnoteReference"><w:name w:val="footnote reference"/><w:basedOn w:val="DefaultParagraphFont"/><w:rPr><w:vertAlign w:val="superscript"/></w:rPr></w:style>
<w:style w:type="character" w:styleId="Hyperlink"><w:name w:val="Hyperlink"/><w:basedOn w:val="DefaultParagraphFont"/><w:uiPriority w:val="99"/><w:unhideWhenUsed/><w:rPr><w:color w:val="0563C1"/><w:u w:val="single"/></w:rPr></w:style>
<w:style w:type="character" w:styleId="Strong"><w:name w:val="Strong"/><w:basedOn w:val="DefaultParagraphFont"/><w:qFormat/><w:rPr><w:b/><w:bCs/></w:rPr></w:style>
<w:style w:type="table" w:default="1" w:styleId="TableNormal"><w:name w:val="Normal Table"/><w:semiHidden/><w:tblPr><w:tblInd w:w="0" w:type="dxa"/><w:tblCellMar><w:top w:w="0" w:type="dxa"/><w:left w:w="108" w:type="dxa"/><w:bottom w:w="0" w:type="dxa"/><w:right w:w="108" w:type="dxa"/></w:tblCellMar></w:tblPr></w:style>
<w:style w:type="table" w:styleId="TableGrid"><w:name w:val="Table Grid"/><w:basedOn w:val="TableNormal"/><w:uiPriority w:val="39"/><w:pPr><w:spacing w:after="0" w:line="240" w:lineRule="auto"/></w:pPr><w:tblPr><w:tblBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="auto"/><w:left w:val="single" w:sz="4" w:space="0" w:color="auto"/><w:bottom w:val="single" w:sz="4" w:space="0" w:color="auto"/><w:right w:val="single" w:sz="4" w:space="0" w:color="auto"/><w:insideH w:val="single" w:sz="4" w:space="0" w:color="auto"/><w:insideV w:val="single" w:sz="4" w:space="0" w:color="auto"/></w:tblBorders></w:tblPr></w:style>
</w:styles>"#
    )
}

/// The passthrough key of the settings part as it was read.
const SETTINGS_AS_READ: &str = "wordcraft:settings.orig";

const SETTINGS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:settings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:zoom w:percent="100"/><w:proofState w:spelling="clean" w:grammar="clean"/><w:defaultTabStop w:val="708"/><w:hyphenationZone w:val="425"/><w:characterSpacingControl w:val="doNotCompress"/>{NOTES}<w:compat><w:compatSetting w:name="compatibilityMode" w:uri="http://schemas.microsoft.com/office/word" w:val="15"/></w:compat><w:themeFontLang w:val="es-ES"/><w:decimalSymbol w:val=","/><w:listSeparator w:val=";"/></w:settings>"#;

const FOOTNOTES_HEAD: &str = r#"<w:footnote w:type="separator" w:id="-1"><w:p><w:pPr><w:spacing w:after="0" w:line="240" w:lineRule="auto"/></w:pPr><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:pPr><w:spacing w:after="0" w:line="240" w:lineRule="auto"/></w:pPr><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>"#;

/// A4 portrait with 2.5 cm side margins, as Spanish Word templates set it, and the given
/// header/footer references.
fn sect(refs: &str, extra: &str) -> String {
    format!(
        r#"<w:sectPr>{refs}<w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1417" w:right="1701" w:bottom="1417" w:left="1701" w:header="708" w:footer="708" w:gutter="0"/><w:cols w:space="708"/>{extra}<w:docGrid w:linePitch="360"/></w:sectPr>"#
    )
}

fn core(title: &str, creator: &str, lang: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><dc:title>{title}</dc:title><dc:creator>{creator}</dc:creator><cp:lastModifiedBy>{creator}</cp:lastModifiedBy><cp:revision>3</cp:revision><dc:language>{lang}</dc:language><cp:contentStatus>Final</cp:contentStatus><dc:identifier>2026-{lang}</dc:identifier><cp:version>1.2</cp:version><cp:lastPrinted>2026-10-05T17:35:00Z</cp:lastPrinted><dcterms:created xsi:type="dcterms:W3CDTF">2026-10-02T09:15:00Z</dcterms:created><dcterms:modified xsi:type="dcterms:W3CDTF">2026-10-05T17:40:00Z</dcterms:modified></cp:coreProperties>"#
    )
}

/// A small PNG logo (an ink-blue square with a lighter drop), drawn in code.
fn logo_png() -> Vec<u8> {
    let img = image::RgbaImage::from_fn(24, 24, |x, y| {
        let (dx, dy) = (x as i32 - 12, y as i32 - 13);
        if dx * dx + dy * dy < 49 { image::Rgba([120, 170, 230, 255]) } else { image::Rgba([20, 60, 140, 255]) }
    });
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

struct Pkg {
    /// Body content (everything inside `w:body`, the final `w:sectPr` included).
    body: String,
    lang: &'static str,
    title: &'static str,
    creator: &'static str,
    numbering: Option<String>,
    header: Option<String>,
    footer: Option<String>,
    footnotes: Option<String>,
    /// The header holds the logo (`rIdLogo` in the header's relationships).
    header_logo: bool,
    /// Document relationships beyond the standard ones (id, type suffix, target, external).
    links: Vec<(&'static str, &'static str, bool)>,
    /// Parts the body points at (charts, diagrams, embedded objects).
    objects: Vec<Object>,
}

/// A part the document relates to, with its own relationships.
struct Object {
    /// The document's relationship id for it (none for a part only another part relates to).
    id: Option<&'static str>,
    /// The relationship type: a suffix of the officeDocument relationships, or a whole URI.
    rel: &'static str,
    /// The part's name under `word/`.
    name: &'static str,
    content_type: &'static str,
    bytes: Vec<u8>,
    /// The part's own relationships (id, whole type URI, target relative to the part).
    rels: Vec<(&'static str, String, &'static str)>,
}

impl Pkg {
    fn new(body: String, lang: &'static str, title: &'static str, creator: &'static str) -> Self {
        Pkg {
            body,
            lang,
            title,
            creator,
            numbering: None,
            header: None,
            footer: None,
            footnotes: None,
            header_logo: false,
            links: Vec::new(),
            objects: Vec::new(),
        }
    }

    fn bytes(&self) -> Vec<u8> {
        let mut over = vec![
            ("/word/document.xml", "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"),
            ("/word/styles.xml", "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"),
            ("/word/settings.xml", "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml"),
            ("/docProps/core.xml", "application/vnd.openxmlformats-package.core-properties+xml"),
        ];
        let mut rels = vec![("rIdStyles", "styles", "styles.xml".to_string(), false), ("rIdSettings", "settings", "settings.xml".to_string(), false)];
        let mut parts: Vec<(String, Vec<u8>)> = Vec::new();
        if let Some(n) = &self.numbering {
            over.push(("/word/numbering.xml", "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"));
            rels.push(("rIdNumbering", "numbering", "numbering.xml".into(), false));
            parts.push((
                "word/numbering.xml".into(),
                format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:numbering {NS}>{n}</w:numbering>"#).into_bytes(),
            ));
        }
        if let Some(h) = &self.header {
            over.push(("/word/header1.xml", "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"));
            rels.push(("rIdHeader", "header", "header1.xml".into(), false));
            parts.push((
                "word/header1.xml".into(),
                format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:hdr {NS}>{h}</w:hdr>"#).into_bytes(),
            ));
            if self.header_logo {
                parts.push((
                    "word/_rels/header1.xml.rels".into(),
                    format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdLogo" Type="{REL}/image" Target="media/image1.png"/></Relationships>"#).into_bytes(),
                ));
                parts.push(("word/media/image1.png".into(), logo_png()));
            }
        }
        if let Some(f) = &self.footer {
            over.push(("/word/footer1.xml", "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"));
            rels.push(("rIdFooter", "footer", "footer1.xml".into(), false));
            parts.push((
                "word/footer1.xml".into(),
                format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:ftr {NS}>{f}</w:ftr>"#).into_bytes(),
            ));
        }
        if let Some(n) = &self.footnotes {
            over.push(("/word/footnotes.xml", "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml"));
            rels.push(("rIdFootnotes", "footnotes", "footnotes.xml".into(), false));
            parts.push((
                "word/footnotes.xml".into(),
                format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:footnotes {NS}>{FOOTNOTES_HEAD}{n}</w:footnotes>"#).into_bytes(),
            ));
        }
        for (id, target, external) in &self.links {
            rels.push((id, "hyperlink", target.to_string(), *external));
        }
        let mut object_types = Vec::new();
        for o in &self.objects {
            if let Some(id) = o.id {
                rels.push((id, o.rel, o.name.to_string(), false));
            }
            object_types.push((format!("/word/{}", o.name), o.content_type));
            parts.push((format!("word/{}", o.name), o.bytes.clone()));
            if !o.rels.is_empty() {
                let (dir, file) = o.name.rsplit_once('/').unwrap_or(("", o.name));
                let mut x = String::from(
                    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
                );
                for (id, ty, target) in &o.rels {
                    x.push_str(&format!(r#"<Relationship Id="{id}" Type="{ty}" Target="{target}"/>"#));
                }
                x.push_str("</Relationships>");
                let rels_name = if dir.is_empty() { format!("word/_rels/{file}.rels") } else { format!("word/{dir}/_rels/{file}.rels") };
                parts.push((rels_name, x.into_bytes()));
            }
        }
        let mut ct = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Default Extension="png" ContentType="image/png"/>"#,
        );
        for (name, ty) in &over {
            ct.push_str(&format!(r#"<Override PartName="{name}" ContentType="{ty}"/>"#));
        }
        for (name, ty) in &object_types {
            ct.push_str(&format!(r#"<Override PartName="{name}" ContentType="{ty}"/>"#));
        }
        ct.push_str("</Types>");
        let mut doc_rels = String::from(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
        );
        for (id, ty, target, external) in &rels {
            let mode = if *external { r#" TargetMode="External""# } else { "" };
            let ty = if ty.starts_with("http") { ty.to_string() } else { format!("{REL}/{ty}") };
            doc_rels.push_str(&format!(r#"<Relationship Id="{id}" Type="{ty}" Target="{target}"{mode}/>"#));
        }
        doc_rels.push_str("</Relationships>");
        let root = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="{REL}/officeDocument" Target="word/document.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/></Relationships>"#
        );
        let document =
            format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document {NS}><w:body>{}</w:body></w:document>"#, self.body);
        let mut entries: Vec<(String, Vec<u8>)> = vec![
            ("[Content_Types].xml".into(), ct.into_bytes()),
            ("_rels/.rels".into(), root.into_bytes()),
            ("word/document.xml".into(), document.into_bytes()),
            ("word/_rels/document.xml.rels".into(), doc_rels.into_bytes()),
            ("word/styles.xml".into(), styles(self.lang).into_bytes()),
            (
                "word/settings.xml".into(),
                // Word lists the separators in the settings only when the document has notes.
                SETTINGS
                    .replace(
                        "{NOTES}",
                        if self.footnotes.is_some() { r#"<w:footnotePr><w:footnote w:id="-1"/><w:footnote w:id="0"/></w:footnotePr>"# } else { "" },
                    )
                    .into_bytes(),
            ),
            ("docProps/core.xml".into(), core(self.title, self.creator, self.lang).into_bytes()),
        ];
        entries.extend(parts);
        let mut zw = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, bytes) in &entries {
            zw.start_file(name.as_str(), zip::write::SimpleFileOptions::default()).unwrap();
            zw.write_all(bytes).unwrap();
        }
        zw.finish().unwrap().into_inner()
    }
}

// ---------------------------------------------------------------------------------------------
// Building blocks of WordprocessingML, so each document below reads like the text it holds.

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// A run with optional run properties.
fn r(rpr: &str, text: &str) -> String {
    let rpr = if rpr.is_empty() { String::new() } else { format!("<w:rPr>{rpr}</w:rPr>") };
    format!(r#"<w:r>{rpr}<w:t xml:space="preserve">{}</w:t></w:r>"#, esc(text))
}

/// A paragraph with optional paragraph properties and its runs.
fn p(ppr: &str, runs: &str) -> String {
    let ppr = if ppr.is_empty() { String::new() } else { format!("<w:pPr>{ppr}</w:pPr>") };
    format!("<w:p>{ppr}{runs}</w:p>")
}

/// A paragraph of plain text.
fn pt(ppr: &str, text: &str) -> String {
    p(ppr, &r("", text))
}

/// A complex field (`fldChar` begin / `instrText` / separate / result / end), as Word writes
/// PAGE, NUMPAGES and DATE.
fn field(rpr: &str, instr: &str, result: &str) -> String {
    let rpr = if rpr.is_empty() { String::new() } else { format!("<w:rPr>{rpr}</w:rPr>") };
    format!(
        r#"<w:r>{rpr}<w:fldChar w:fldCharType="begin"/></w:r><w:r>{rpr}<w:instrText xml:space="preserve"> {} </w:instrText></w:r><w:r>{rpr}<w:fldChar w:fldCharType="separate"/></w:r><w:r>{rpr}<w:t>{}</w:t></w:r><w:r>{rpr}<w:fldChar w:fldCharType="end"/></w:r>"#,
        esc(instr),
        esc(result)
    )
}

/// A table cell of the given width (twips) with optional cell properties and paragraphs.
fn tc(width: u32, tcpr: &str, content: &str) -> String {
    format!(r#"<w:tc><w:tcPr><w:tcW w:w="{width}" w:type="dxa"/>{tcpr}</w:tcPr>{content}</w:tc>"#)
}

// ---------------------------------------------------------------------------------------------
// The four documents.

/// Factura: an autónomo's invoice with a logo in the header, the client block on right tabs,
/// a lines table (repeated header row, shading, right-aligned amounts, a merged totals label),
/// IVA and IRPF, payment details and "Página X de Y" in the footer.
fn factura() -> Pkg {
    let right = r#"<w:jc w:val="right"/>"#;
    let head = |t: &str, w: u32, jc: &str| {
        tc(
            w,
            r#"<w:shd w:val="clear" w:color="auto" w:fill="1F3864"/><w:vAlign w:val="center"/>"#,
            &p(jc, &r(r#"<w:b/><w:color w:val="FFFFFF"/>"#, t)),
        )
    };
    let line = |concepto: &str, uds: &str, precio: &str, importe: &str| {
        format!(
            "<w:tr>{}{}{}{}</w:tr>",
            tc(4536, "", &pt("", concepto)),
            tc(1134, "", &pt(r#"<w:jc w:val="center"/>"#, uds)),
            tc(1701, "", &pt(right, precio)),
            tc(1701, "", &pt(right, importe))
        )
    };
    let total = |label: &str, value: &str, bold: bool| {
        let rpr = if bold { "<w:b/>" } else { "" };
        format!(
            "<w:tr>{}{}</w:tr>",
            tc(7371, r#"<w:gridSpan w:val="3"/><w:tcBorders><w:left w:val="nil"/><w:bottom w:val="nil"/></w:tcBorders>"#, &p(right, &r(rpr, label))),
            tc(1701, if bold { r#"<w:shd w:val="clear" w:color="auto" w:fill="D9E2F3"/>"# } else { "" }, &p(right, &r(rpr, value)))
        )
    };
    let tabs = r#"<w:tabs><w:tab w:val="left" w:pos="4820"/></w:tabs><w:spacing w:after="0"/>"#;
    let body = [
        p(r#"<w:pStyle w:val="Title"/>"#, &format!("{}{}", r("", "FACTURA "), r(r#"<w:color w:val="2F5496"/>"#, "nº 2026-014"))),
        p(tabs, &format!("{}<w:r><w:tab/></w:r>{}", r("<w:b/>", "Emisor"), r("<w:b/>", "Cliente"))),
        p(tabs, &format!("{}<w:r><w:tab/></w:r>{}", r("", "María Núñez Ibáñez"), r("", "Construcciones Ferrándiz, S.L."))),
        p(tabs, &format!("{}<w:r><w:tab/></w:r>{}", r("", "NIF 12345678Z"), r("", "CIF B-46000000"))),
        p(tabs, &format!("{}<w:r><w:tab/></w:r>{}", r("", "C/ de la Señoría, 7, 2º 3ª"), r("", "Avda. del Puerto, 112"))),
        p(r#"<w:tabs><w:tab w:val="left" w:pos="4820"/></w:tabs>"#, &format!("{}<w:r><w:tab/></w:r>{}", r("", "46500 Sagunto (Valencia)"), r("", "46024 València"))),
        p("", &format!("{}{}", r("<w:b/>", "Fecha de expedición: "), r("", "5 de octubre de 2026"))),
        format!(
            r#"<w:tbl><w:tblPr><w:tblStyle w:val="TableGrid"/><w:tblW w:w="9072" w:type="dxa"/><w:tblLook w:val="04A0" w:firstRow="1" w:lastRow="0" w:firstColumn="1" w:lastColumn="0" w:noHBand="0" w:noVBand="1"/></w:tblPr><w:tblGrid><w:gridCol w:w="4536"/><w:gridCol w:w="1134"/><w:gridCol w:w="1701"/><w:gridCol w:w="1701"/></w:tblGrid><w:tr><w:trPr><w:tblHeader/><w:trHeight w:val="397"/></w:trPr>{}{}{}{}</w:tr>{}{}{}{}{}{}{}</w:tbl>"#,
            head("Concepto", 4536, ""),
            head("Uds.", 1134, r#"<w:jc w:val="center"/>"#),
            head("Precio", 1701, right),
            head("Importe", 1701, right),
            line("Diseño de la identidad corporativa", "1", "850,00 €", "850,00 €"),
            line("Maquetación del catálogo (24 págs.)", "24", "25,00 €", "600,00 €"),
            line("Fotografía de producto — sesión de mañana", "1", "180,00 €", "180,00 €"),
            total("Base imponible", "1.630,00 €", false),
            total("IVA (21 %)", "342,30 €", false),
            total("Retención IRPF (−15 %)", "−244,50 €", false),
            total("TOTAL A PAGAR", "1.727,80 €", true),
        ),
        p(r#"<w:spacing w:before="240"/>"#, &format!("{}{}", r("<w:b/>", "Forma de pago: "), r("", "transferencia a ES91 2100 0418 4502 0005 1332 antes del 4 de noviembre de 2026."))),
        p(r#"<w:spacing w:after="0"/><w:jc w:val="both"/>"#, &r(r#"<w:i/><w:sz w:val="16"/>"#, "Operación sujeta a retención conforme al art. 101 de la Ley 35/2006 del IRPF.")),
        sect(r#"<w:headerReference w:type="default" r:id="rIdHeader"/><w:footerReference w:type="default" r:id="rIdFooter"/>"#, ""),
    ]
    .concat();
    let mut pkg = Pkg::new(body, "es-ES", "Factura 2026-014", "María Núñez Ibáñez");
    pkg.header = Some(p(
        r#"<w:pStyle w:val="Header"/>"#,
        &format!(
            r#"<w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="457200" cy="457200"/><wp:effectExtent l="0" t="0" r="0" b="0"/><wp:docPr id="1" name="Logotipo" descr="Logotipo de María Núñez, diseño gráfico"/><wp:cNvGraphicFramePr><a:graphicFrameLocks noChangeAspect="1"/></wp:cNvGraphicFramePr><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:nvPicPr><pic:cNvPr id="0" name="image1.png"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="rIdLogo"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="457200" cy="457200"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r><w:r><w:tab/></w:r><w:r><w:tab/></w:r>{}"#,
            r(r#"<w:b/><w:color w:val="1F3864"/>"#, "María Núñez · Diseño gráfico")
        ),
    ));
    pkg.header_logo = true;
    pkg.footer = Some(p(
        r#"<w:pStyle w:val="Footer"/><w:jc w:val="center"/>"#,
        &format!("{}{}{}{}", r("", "Página "), field("", "PAGE   \\* MERGEFORMAT", "1"), r("", " de "), field("", "NUMPAGES   \\* MERGEFORMAT", "1")),
    ));
    pkg
}

/// Contrato: a service contract with multi-level numbered clauses (1., 1.1.), justified text,
/// a footnote citing the law, a page break before the signatures and a borderless signature
/// table.
fn contrato() -> Pkg {
    let clause = |lvl: u8, text: &str| {
        p(
            &format!(r#"<w:pStyle w:val="ListParagraph"/><w:numPr><w:ilvl w:val="{lvl}"/><w:numId w:val="1"/></w:numPr><w:jc w:val="both"/>"#),
            &r("", text),
        )
    };
    let just = r#"<w:jc w:val="both"/>"#;
    let sign = |who: &str, name: &str| {
        tc(
            4252,
            "",
            &format!(
                "{}{}{}",
                pt(r#"<w:jc w:val="center"/>"#, who),
                p(r#"<w:spacing w:before="960"/><w:jc w:val="center"/>"#, &r("", "Fdo.: ")),
                pt(r#"<w:jc w:val="center"/>"#, name)
            ),
        )
    };
    let body = [
        p(r#"<w:pStyle w:val="Title"/><w:jc w:val="center"/>"#, &r("", "CONTRATO DE PRESTACIÓN DE SERVICIOS")),
        pt(just, "En Sagunto, a 2 de octubre de 2026."),
        p(r#"<w:pStyle w:val="Heading1"/>"#, &r("", "Reunidos")),
        p(just, &format!("{}{}", r("<w:b/>", "De una parte, "), r("", "D.ª María Núñez Ibáñez, mayor de edad, con NIF 12345678Z, que actúa en su propio nombre (en adelante, «la Profesional»)."))),
        p(just, &format!("{}{}", r("<w:b/>", "De otra parte, "), r("", "D. Jordi Ferrándiz i Puig, en nombre y representación de Construcciones Ferrándiz, S.L. (en adelante, «el Cliente»)."))),
        p(r#"<w:pStyle w:val="Heading1"/>"#, &r("", "Cláusulas")),
        clause(0, "Objeto. La Profesional diseñará la identidad corporativa del Cliente."),
        clause(1, "El encargo incluye el logotipo, la paleta de colores y el manual de uso."),
        clause(1, "Quedan excluidos la impresión y el registro de la marca."),
        clause(0, "Precio. El precio total es de 1.630,00 €, más el IVA correspondiente."),
        p(
            r#"<w:pStyle w:val="ListParagraph"/><w:numPr><w:ilvl w:val="1"/><w:numId w:val="1"/></w:numPr><w:jc w:val="both"/>"#,
            &format!(
                "{}{}{}",
                r("", "La Profesional practicará la retención del IRPF que proceda"),
                r#"<w:r><w:rPr><w:rStyle w:val="FootnoteReference"/></w:rPr><w:footnoteReference w:id="1"/></w:r>"#,
                r("", ".")
            ),
        ),
        clause(0, "Plazo. El trabajo se entregará antes del 30 de noviembre de 2026."),
        clause(0, "Protección de datos. Ambas partes cumplirán el RGPD y la LOPDGDD."),
        p(just, &format!("{}{}{}", r("", "Y, en prueba de conformidad, firman el presente contrato por duplicado."), "<w:r><w:br w:type=\"page\"/></w:r>", "")),
        format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="8504" w:type="dxa"/><w:jc w:val="center"/><w:tblBorders><w:top w:val="nil"/><w:left w:val="nil"/><w:bottom w:val="nil"/><w:right w:val="nil"/><w:insideH w:val="nil"/><w:insideV w:val="nil"/></w:tblBorders><w:tblLook w:val="0000" w:firstRow="0" w:lastRow="0" w:firstColumn="0" w:lastColumn="0" w:noHBand="0" w:noVBand="0"/></w:tblPr><w:tblGrid><w:gridCol w:w="4252"/><w:gridCol w:w="4252"/></w:tblGrid><w:tr><w:trPr><w:cantSplit/></w:trPr>{}{}</w:tr></w:tbl>"#,
            sign("La Profesional", "María Núñez Ibáñez"),
            sign("El Cliente", "Jordi Ferrándiz i Puig")
        ),
        p("", ""),
        sect(r#"<w:footerReference w:type="default" r:id="rIdFooter"/>"#, r#"<w:pgNumType w:start="1"/><w:titlePg w:val="0"/>"#),
    ]
    .concat();
    let mut pkg = Pkg::new(body, "es-ES", "Contrato de prestación de servicios", "María Núñez Ibáñez");
    pkg.numbering = Some(
        r#"<w:abstractNum w:abstractNumId="0"><w:multiLevelType w:val="multilevel"/><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="decimal"/><w:lvlText w:val="%1."/><w:lvlJc w:val="left"/><w:pPr><w:ind w:left="360" w:hanging="360"/></w:pPr><w:rPr><w:b/></w:rPr></w:lvl><w:lvl w:ilvl="1"><w:start w:val="1"/><w:numFmt w:val="decimal"/><w:lvlText w:val="%1.%2."/><w:lvlJc w:val="left"/><w:pPr><w:ind w:left="792" w:hanging="432"/></w:pPr></w:lvl></w:abstractNum><w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num>"#
            .into(),
    );
    pkg.footnotes = Some(format!(
        r#"<w:footnote w:id="1">{}</w:footnote>"#,
        p(
            r#"<w:pStyle w:val="FootnoteText"/>"#,
            &format!(
                "{}{}",
                r#"<w:r><w:rPr><w:rStyle w:val="FootnoteReference"/></w:rPr><w:footnoteRef/></w:r>"#,
                r("", " Art. 101.5 de la Ley 35/2006, de 28 de noviembre, del IRPF.")
            )
        )
    ));
    pkg.footer = Some(p(r#"<w:pStyle w:val="Footer"/><w:jc w:val="right"/>"#, &field("", "PAGE", "1")));
    pkg
}

/// Резюме: a Ukrainian CV in a two-column layout table (a merged photo-free side column), with
/// headings, a bulleted list, a mailto and a web link, Cyrillic with the apostrophe and ґ/є/ї.
fn rezume() -> Pkg {
    let bullet = |t: &str| p(r#"<w:pStyle w:val="ListParagraph"/><w:numPr><w:ilvl w:val="0"/><w:numId w:val="2"/></w:numPr>"#, &r("", t));
    let side = |content: &str, merge: &str| tc(2835, &format!(r#"{merge}<w:shd w:val="clear" w:color="auto" w:fill="DEEAF6"/>"#), content);
    let main = |content: &str| tc(6237, "", content);
    let link = |id: &str, t: &str| format!(r#"<w:hyperlink r:id="{id}" w:history="1">{}</w:hyperlink>"#, r(r#"<w:rStyle w:val="Hyperlink"/>"#, t));
    let rows = [
        format!(
            "<w:tr>{}{}</w:tr>",
            side(
                &[
                    p(r#"<w:pStyle w:val="Heading2"/>"#, &r("", "Контакти")),
                    pt("", "Сагунто, Іспанія"),
                    p("", &link("rIdMail", "olena.kovalchuk@example.com")),
                    p("", &link("rIdWeb", "kovalchuk.example.org")),
                    p(r#"<w:pStyle w:val="Heading2"/>"#, &r("", "Мови")),
                    pt("", "Українська — рідна"),
                    pt("", "Іспанська — C1 (DELE)"),
                    pt("", "Англійська — B2"),
                ]
                .concat(),
                r#"<w:vMerge w:val="restart"/>"#
            ),
            main(
                &[
                    p(r#"<w:pStyle w:val="Title"/>"#, &r("", "Олена Ковальчук")),
                    p("", &r(r#"<w:i/><w:color w:val="595959"/>"#, "Бухгалтерка, консультантка з податків для самозайнятих (autónomos)")),
                    p(r#"<w:pStyle w:val="Heading1"/>"#, &r("", "Досвід роботи")),
                    p(
                        "",
                        &format!(
                            "{}{}",
                            r("<w:b/>", "2023 – тепер. "),
                            r("", "Gestoría Levante, Валенсія: ведення обліку 80 клієнтів, декларації 303 і 130.")
                        )
                    ),
                    p("", &format!("{}{}", r("<w:b/>", "2016 – 2022. "), r("", "ТОВ «Ґрунт-Агро», Житомир: головна бухгалтерка."))),
                ]
                .concat()
            )
        ),
        format!(
            "<w:tr>{}{}</w:tr>",
            side(&p("", ""), r#"<w:vMerge/>"#),
            main(
                &[
                    p(r#"<w:pStyle w:val="Heading1"/>"#, &r("", "Навички")),
                    bullet("Облік і звітність в Іспанії та в Україні"),
                    bullet("Програми: Tinta Calculo, A3, 1С, M.E.Doc"),
                    bullet("Комунікація з клієнтами українською, іспанською та англійською"),
                    p(r#"<w:pStyle w:val="Heading1"/>"#, &r("", "Освіта")),
                    pt("", "Житомирський державний університет імені Івана Франка, магістр з обліку й аудиту, 2016."),
                    pt("", "Пам'ятка: дипломи визнано в Іспанії (homologación), 2024."),
                ]
                .concat()
            )
        ),
    ];
    let body = [
        format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="9072" w:type="dxa"/><w:tblBorders><w:top w:val="nil"/><w:left w:val="nil"/><w:bottom w:val="nil"/><w:right w:val="nil"/><w:insideH w:val="nil"/><w:insideV w:val="nil"/></w:tblBorders><w:tblLayout w:type="fixed"/><w:tblCellMar><w:left w:w="170" w:type="dxa"/><w:right w:w="170" w:type="dxa"/></w:tblCellMar><w:tblLook w:val="0000" w:firstRow="0" w:lastRow="0" w:firstColumn="0" w:lastColumn="0" w:noHBand="0" w:noVBand="0"/></w:tblPr><w:tblGrid><w:gridCol w:w="2835"/><w:gridCol w:w="6237"/></w:tblGrid>{}</w:tbl>"#,
            rows.concat()
        ),
        p("", ""),
        sect("", ""),
    ]
    .concat();
    let mut pkg = Pkg::new(body, "uk-UA", "Резюме — Олена Ковальчук", "Олена Ковальчук");
    pkg.numbering = Some(
        r#"<w:abstractNum w:abstractNumId="1"><w:multiLevelType w:val="hybridMultilevel"/><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="bullet"/><w:lvlText w:val="•"/><w:lvlJc w:val="left"/><w:pPr><w:ind w:left="360" w:hanging="360"/></w:pPr><w:rPr><w:rFonts w:ascii="Liberation Sans" w:hAnsi="Liberation Sans" w:hint="default"/><w:color w:val="2F5496"/></w:rPr></w:lvl></w:abstractNum><w:num w:numId="2"><w:abstractNumId w:val="1"/></w:num>"#
            .into(),
    );
    pkg.links = vec![("rIdMail", "mailto:olena.kovalchuk@example.com", true), ("rIdWeb", "https://kovalchuk.example.org/", true)];
    pkg
}

/// Carta: a formal letter in Catalan (l·l, accents) with a right-aligned date field, the
/// recipient's address, a bookmark on the subject, a "Ref." line with a tab leader and a
/// signature block, in a Valencian-style layout with a different first page footer.
fn carta() -> Pkg {
    let body = [
        p(r#"<w:spacing w:after="0"/>"#, &r("<w:b/>", "Maria Núñez Ibáñez")),
        pt(r#"<w:spacing w:after="0"/>"#, "Carrer de la Senyoria, 7, 2n 3a"),
        pt("", "46500 Sagunt"),
        p(r#"<w:jc w:val="right"/>"#, &format!("{}{}", r("", "Sagunt, "), field("", r#"DATE \@ "d 'de' MMMM 'de' yyyy""#, "5 d’octubre de 2026"))),
        pt(r#"<w:spacing w:after="0"/>"#, "Ajuntament de València"),
        pt(r#"<w:spacing w:after="0"/>"#, "Servei de Llicències"),
        pt(r#"<w:spacing w:after="360"/>"#, "Plaça de l’Ajuntament, 1, 46002 València"),
        p(
            r#"<w:tabs><w:tab w:val="right" w:leader="dot" w:pos="8504"/></w:tabs>"#,
            &format!("{}<w:r><w:tab/></w:r>{}", r("<w:b/>", "Ref.:"), r("", "expedient 2026/0417-LA")),
        ),
        p(
            "",
            &format!(
                r#"<w:bookmarkStart w:id="0" w:name="Assumpte"/>{}{}<w:bookmarkEnd w:id="0"/>"#,
                r("<w:b/>", "Assumpte: "),
                r("<w:b/>", "sol·licitud de llicència d’activitat per a un estudi de disseny")
            ),
        ),
        pt("", "Senyores i senyors,"),
        pt(
            r#"<w:jc w:val="both"/><w:ind w:firstLine="567"/>"#,
            "Em dirigeixo a vostès per sol·licitar la llicència d’activitat per al local del carrer de la Senyoria, on exerciré com a dissenyadora gràfica autònoma. Adjunte el projecte tècnic, el certificat de compatibilitat urbanística i el justificant del pagament de la taxa.",
        ),
        pt(
            r#"<w:jc w:val="both"/><w:ind w:firstLine="567"/>"#,
            "Resto a la seua disposició per a qualsevol aclariment. Gràcies per l’atenció.",
        ),
        pt(r#"<w:spacing w:before="240"/>"#, "Atentament,"),
        p(r#"<w:spacing w:before="720" w:after="0"/>"#, &r("", "Maria Núñez Ibáñez")),
        p("", &r(r#"<w:sz w:val="18"/>"#, "NIF 12345678Z · hola@example.com")),
        sect(r#"<w:footerReference w:type="default" r:id="rIdFooter"/>"#, r#"<w:titlePg/>"#),
    ]
    .concat();
    let mut pkg = Pkg::new(body, "ca-ES", "Sol·licitud de llicència d’activitat", "Maria Núñez Ibáñez");
    pkg.footer = Some(p(
        r#"<w:pStyle w:val="Footer"/><w:pBdr><w:top w:val="single" w:sz="4" w:space="4" w:color="8EAADB"/></w:pBdr><w:jc w:val="center"/>"#,
        &format!("{}{}", r("", "Pàgina "), field("", "PAGE", "2")),
    ));
    pkg
}

/// Stands in for an embedded workbook: opaque bytes the save must carry unchanged.
fn workbook(name: &str) -> Vec<u8> {
    format!("PK\u{3}\u{4} stand-in workbook: {name}").into_bytes()
}

const C_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/chart";
const DGM_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/diagram";
const DSP_NS: &str = "http://schemas.microsoft.com/office/drawing/2008/diagram";

/// Informe trimestral: a quarterly report with a bar chart of income per quarter (its workbook
/// embedded), a three-step SmartArt process and a spreadsheet embedded as an OLE object.
fn informe() -> Pkg {
    let (cx, cy) = (5486400, 2743200);
    let inline = |id: u32, name: &str, uri: &str, data: &str| {
        format!(
            r#"<w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="{cx}" cy="{cy}"/><wp:effectExtent l="0" t="0" r="0" b="0"/><wp:docPr id="{id}" name="{name}"/><wp:cNvGraphicFramePr/><a:graphic><a:graphicData uri="{uri}">{data}</a:graphicData></a:graphic></wp:inline></w:drawing></w:r>"#
        )
    };
    let body = [
        p(r#"<w:pStyle w:val="Title"/>"#, &r("", "Informe del tercer trimestre de 2026")),
        p(r#"<w:pStyle w:val="Heading1"/>"#, &r("", "Ingresos por trimestre")),
        p(r#"<w:jc w:val="center"/>"#, &inline(1, "Gráfico 1", C_NS, &format!(r#"<c:chart xmlns:c="{C_NS}" r:id="rIdChart"/>"#))),
        p(r#"<w:pStyle w:val="Heading1"/>"#, &r("", "Del presupuesto al cobro")),
        p(
            r#"<w:jc w:val="center"/>"#,
            &inline(2, "Diagrama 2", DGM_NS, &format!(r#"<dgm:relIds xmlns:dgm="{DGM_NS}" r:dm="rIdDgmData" r:lo="rIdDgmLayout" r:qs="rIdDgmStyle" r:cs="rIdDgmColors"/>"#)),
        ),
        p(r#"<w:pStyle w:val="Heading1"/>"#, &r("", "Detalle de gastos")),
        p(
            "",
            r##"<w:r><w:object w:dxaOrig="5760" w:dyaOrig="2160"><v:shape id="_x0000_i1025" type="#_x0000_t75" style="width:288pt;height:108pt" o:ole=""><v:imagedata r:id="rIdOlePreview" o:title=""/></v:shape><o:OLEObject Type="Embed" ProgID="Excel.Sheet.12" ShapeID="_x0000_i1025" DrawAspect="Content" ObjectID="_1790000001" r:id="rIdOle"/></w:object></w:r>"##,
        ),
        pt(r#"<w:jc w:val="both"/>"#, "Los gastos del trimestre (alquiler, cuota de autónomos, software) suman 1.912,40 €."),
        sect("", ""),
    ]
    .concat();
    let mut pkg = Pkg::new(body, "es-ES", "Informe 3T 2026", "María Núñez Ibáñez");
    let chart = format!(
        r##"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<c:chartSpace xmlns:c="{C_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="{REL}"><c:date1904 val="0"/><c:lang val="es-ES"/><c:roundedCorners val="0"/><c:chart><c:title><c:tx><c:rich><a:bodyPr/><a:p><a:r><a:t>Ingresos 2026 (€)</a:t></a:r></a:p></c:rich></c:tx><c:overlay val="0"/></c:title><c:autoTitleDeleted val="0"/><c:plotArea><c:layout/><c:barChart><c:barDir val="col"/><c:grouping val="clustered"/><c:varyColors val="0"/><c:ser><c:idx val="0"/><c:order val="0"/><c:tx><c:strRef><c:f>Hoja1!$B$1</c:f><c:strCache><c:ptCount val="1"/><c:pt idx="0"><c:v>Ingresos</c:v></c:pt></c:strCache></c:strRef></c:tx><c:cat><c:strRef><c:f>Hoja1!$A$2:$A$4</c:f><c:strCache><c:ptCount val="3"/><c:pt idx="0"><c:v>1T</c:v></c:pt><c:pt idx="1"><c:v>2T</c:v></c:pt><c:pt idx="2"><c:v>3T</c:v></c:pt></c:strCache></c:strRef></c:cat><c:val><c:numRef><c:f>Hoja1!$B$2:$B$4</c:f><c:numCache><c:formatCode>#,##0.00</c:formatCode><c:ptCount val="3"/><c:pt idx="0"><c:v>4210.5</c:v></c:pt><c:pt idx="1"><c:v>5120</c:v></c:pt><c:pt idx="2"><c:v>4890.75</c:v></c:pt></c:numCache></c:numRef></c:val></c:ser><c:gapWidth val="150"/><c:axId val="111"/><c:axId val="222"/></c:barChart><c:catAx><c:axId val="111"/><c:scaling><c:orientation val="minMax"/></c:scaling><c:delete val="0"/><c:axPos val="b"/><c:crossAx val="222"/></c:catAx><c:valAx><c:axId val="222"/><c:scaling><c:orientation val="minMax"/></c:scaling><c:delete val="0"/><c:axPos val="l"/><c:numFmt formatCode="#,##0" sourceLinked="0"/><c:crossAx val="111"/></c:valAx></c:plotArea><c:plotVisOnly val="1"/></c:chart><c:externalData r:id="rId1"><c:autoUpdate val="0"/></c:externalData></c:chartSpace>"##
    );
    let point = |id: &str, kind: &str, text: &str| {
        let t = if text.is_empty() {
            "<dgm:t><a:bodyPr/><a:p><a:endParaRPr/></a:p></dgm:t>".to_string()
        } else {
            format!("<dgm:t><a:bodyPr/><a:p><a:r><a:t>{text}</a:t></a:r></a:p></dgm:t>")
        };
        format!(r#"<dgm:pt modelId="{id}" type="{kind}"><dgm:prSet/><dgm:spPr/>{t}</dgm:pt>"#)
    };
    let data = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<dgm:dataModel xmlns:dgm="{DGM_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><dgm:ptLst>{}{}{}{}</dgm:ptLst><dgm:cxnLst><dgm:cxn modelId="{{10}}" srcId="{{0}}" destId="{{1}}" srcOrd="0" destOrd="0"/><dgm:cxn modelId="{{11}}" srcId="{{0}}" destId="{{2}}" srcOrd="1" destOrd="0"/><dgm:cxn modelId="{{12}}" srcId="{{0}}" destId="{{3}}" srcOrd="2" destOrd="0"/></dgm:cxnLst><dgm:bg/><dgm:whole/><dgm:extLst><a:ext uri="http://schemas.microsoft.com/office/drawing/2008/diagram"><dsp:dataModelExt xmlns:dsp="{DSP_NS}" relId="rIdDgmDrawing" minVer="http://schemas.openxmlformats.org/drawingml/2006/diagram"/></a:ext></dgm:extLst></dgm:dataModel>"#,
        point("{0}", "doc", ""),
        point("{1}", "node", "Presupuesto"),
        point("{2}", "node", "Factura"),
        point("{3}", "node", "Cobro")
    );
    let layout = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<dgm:layoutDef xmlns:dgm="{DGM_NS}" uniqueId="urn:tintaoffice.com/test/layout/steps"><dgm:title val="Pasos"/><dgm:layoutNode name="steps"><dgm:alg type="lin"/><dgm:forEach name="each" axis="ch" ptType="node"><dgm:layoutNode name="step"><dgm:alg type="tx"/><dgm:shape type="roundRect"/></dgm:layoutNode></dgm:forEach></dgm:layoutNode></dgm:layoutDef>"#
    );
    let style = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><dgm:styleDef xmlns:dgm="{DGM_NS}" uniqueId="urn:tintaoffice.com/test/style/plain"><dgm:title val="Simple"/></dgm:styleDef>"#
    );
    let colors = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><dgm:colorsDef xmlns:dgm="{DGM_NS}" uniqueId="urn:tintaoffice.com/test/colors/ink"><dgm:title val="Tinta"/></dgm:colorsDef>"#
    );
    let shape = |id: &str, x: i64, text: &str| {
        format!(
            r#"<dsp:sp modelId="{id}"><dsp:nvSpPr><dsp:cNvPr id="0" name=""/><dsp:cNvSpPr/></dsp:nvSpPr><dsp:spPr><a:xfrm><a:off x="{x}" y="914400"/><a:ext cx="1554480" cy="914400"/></a:xfrm><a:prstGeom prst="roundRect"><a:avLst/></a:prstGeom><a:solidFill><a:srgbClr val="1F3864"/></a:solidFill></dsp:spPr><dsp:txBody><a:bodyPr anchor="ctr"/><a:lstStyle/><a:p><a:pPr algn="ctr"/><a:r><a:rPr lang="es-ES"><a:solidFill><a:srgbClr val="FFFFFF"/></a:solidFill></a:rPr><a:t>{text}</a:t></a:r></a:p></dsp:txBody></dsp:sp>"#
        )
    };
    let drawing = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<dsp:drawing xmlns:dsp="{DSP_NS}" xmlns:dgm="{DGM_NS}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><dsp:spTree><dsp:nvGrpSpPr><dsp:cNvPr id="0" name=""/><dsp:cNvGrpSpPr/></dsp:nvGrpSpPr><dsp:grpSpPr/>{}{}{}</dsp:spTree></dsp:drawing>"#,
        shape("{1}", 0, "Presupuesto"),
        shape("{2}", 1965960, "Factura"),
        shape("{3}", 3931920, "Cobro")
    );
    let ct_dgm = |k: &str| match k {
        "data" => "application/vnd.openxmlformats-officedocument.drawingml.diagramData+xml",
        "layout" => "application/vnd.openxmlformats-officedocument.drawingml.diagramLayout+xml",
        "style" => "application/vnd.openxmlformats-officedocument.drawingml.diagramStyle+xml",
        "colors" => "application/vnd.openxmlformats-officedocument.drawingml.diagramColors+xml",
        _ => "application/vnd.ms-office.drawingml.diagramDrawing+xml",
    };
    let object = |id, rel, name, content_type, bytes: Vec<u8>| Object { id: Some(id), rel, name, content_type, bytes, rels: Vec::new() };
    pkg.objects = vec![
        Object {
            rels: vec![("rId1", format!("{REL}/package"), "../embeddings/Microsoft_Excel_Worksheet.xlsx")],
            ..object(
                "rIdChart",
                "chart",
                "charts/chart1.xml",
                "application/vnd.openxmlformats-officedocument.drawingml.chart+xml",
                chart.into_bytes(),
            )
        },
        Object {
            id: None,
            ..object(
                "",
                "package",
                "embeddings/Microsoft_Excel_Worksheet.xlsx",
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                workbook("chart"),
            )
        },
        object("rIdDgmData", "diagramData", "diagrams/data1.xml", ct_dgm("data"), data.into_bytes()),
        object("rIdDgmLayout", "diagramLayout", "diagrams/layout1.xml", ct_dgm("layout"), layout.into_bytes()),
        object("rIdDgmStyle", "diagramQuickStyle", "diagrams/quickStyle1.xml", ct_dgm("style"), style.into_bytes()),
        object("rIdDgmColors", "diagramColors", "diagrams/colors1.xml", ct_dgm("colors"), colors.into_bytes()),
        object(
            "rIdDgmDrawing",
            "http://schemas.microsoft.com/office/2007/relationships/diagramDrawing",
            "diagrams/drawing1.xml",
            ct_dgm("drawing"),
            drawing.into_bytes(),
        ),
        object("rIdOlePreview", "image", "media/image2.png", "image/png", logo_png()),
        object(
            "rIdOle",
            "package",
            "embeddings/Microsoft_Excel_Worksheet1.xlsx",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            workbook("gastos"),
        ),
    ];
    pkg
}

fn all() -> Vec<(&'static str, Pkg)> {
    vec![("factura", factura()), ("contrato", contrato()), ("rezume", rezume()), ("carta", carta()), ("informe", informe())]
}

// ---------------------------------------------------------------------------------------------
// What "nothing lost" means, part by part.

fn unzip(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut out = BTreeMap::new();
    for i in 0..z.len() {
        let mut f = z.by_index(i).unwrap();
        let mut b = Vec::new();
        f.read_to_end(&mut b).unwrap();
        out.insert(f.name().to_string(), b);
    }
    out
}

/// What a WordprocessingML part says, as names: each `w:` element and each `w:` attribute on
/// it (`element@attribute`), and the text of each paragraph and field code. Both sides use
/// the `w` prefix (the fixtures and WordCraft's writer).
#[derive(Default)]
struct Vocabulary {
    names: BTreeSet<String>,
    text: String,
}

fn vocabulary(xml: &[u8]) -> Vocabulary {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_reader(xml);
    let mut v = Vocabulary::default();
    let mut in_text = false;
    let mut buf = Vec::new();
    loop {
        buf.clear();
        match reader.read_event_into(&mut buf).unwrap() {
            Event::Start(e) | Event::Empty(e) => {
                let qname = String::from_utf8(e.name().as_ref().to_vec()).unwrap();
                let Some(name) = qname.strip_prefix("w:") else { continue };
                in_text = matches!(name, "t" | "instrText");
                if name == "instrText" {
                    v.text.push('|');
                }
                v.names.insert(name.to_string());
                for a in e.attributes() {
                    let a = a.unwrap();
                    if let Some(attr) = std::str::from_utf8(a.key.as_ref()).unwrap().strip_prefix("w:") {
                        v.names.insert(format!("{name}@{attr}"));
                    }
                }
            }
            Event::Text(t) if in_text => v.text.push_str(&t.xml10_content().unwrap()),
            Event::GeneralRef(r) if in_text => {
                let entity = String::from_utf8(r.to_vec()).unwrap();
                v.text.push_str(match entity.as_str() {
                    "amp" => "&",
                    "lt" => "<",
                    "gt" => ">",
                    "quot" => "\"",
                    "apos" => "'",
                    other => panic!("entity {other}"),
                });
            }
            Event::End(e) => {
                // Runs may be split or joined differently; paragraphs and field codes may not.
                if matches!(e.name().as_ref(), b"w:p" | b"w:instrText") {
                    v.text.push('|');
                }
                in_text = false;
            }
            Event::Eof => break,
            _ => {}
        }
    }
    v
}

/// The parts WordCraft writes itself from its model: the ones `vocabulary` and `relationships`
/// compare. Any other part must come back unchanged.
fn rewritten(part: &str) -> bool {
    part == "[Content_Types].xml"
        || part.ends_with(".rels")
        || part.starts_with("docProps/")
        || ["document", "styles", "settings", "numbering", "header1", "footer1", "footnotes"].iter().any(|p| part == format!("word/{p}.xml"))
}

/// The content type `[Content_Types].xml` gives a part: its override, else its extension's
/// default.
fn content_type(types: &str, part: &str) -> Option<String> {
    let attr = |entry: &str, name: &str| entry.split(&format!("{name}=\"")).nth(1).and_then(|v| v.split('"').next()).map(str::to_string);
    let ext = part.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_default();
    types
        .split("<Override ")
        .skip(1)
        .find(|o| attr(o, "PartName").as_deref() == Some(&format!("/{part}")))
        .and_then(|o| attr(o, "ContentType"))
        .or_else(|| {
            types
                .split("<Default ")
                .skip(1)
                .find(|d| attr(d, "Extension").map(|e| e.to_ascii_lowercase()) == Some(ext.clone()))
                .and_then(|d| attr(d, "ContentType"))
        })
}

/// The (type, target) of each relationship in a `.rels` part.
fn relationships(xml: &[u8]) -> Vec<(String, String)> {
    let s = String::from_utf8_lossy(xml);
    s.split("<Relationship ")
        .skip(1)
        .map(|r| {
            let attr = |name: &str| r.split(&format!("{name}=\"")).nth(1).and_then(|v| v.split('"').next()).unwrap_or("").to_string();
            (attr("Type"), attr("Target"))
        })
        .collect()
}

/// Normalizes field instructions, whose spacing a writer may change (` PAGE ` / `PAGE`).
fn squash(text: &str) -> String {
    text.split('|').map(|t| t.split_whitespace().collect::<Vec<_>>().join(" ")).filter(|t| !t.is_empty()).collect::<Vec<_>>().join("|")
}

/// Names a save may leave out because the source only spelled out the default: what Word
/// itself writes in some places and WordCraft does not, with the same meaning.
const DEFAULTS_LEFT_OUT: &[(&str, &str)] = &[
    // `Normal Table` with `w:tblInd w:w="0"`: no indent is the default (§17.4.50).
    ("word/styles.xml", "tblInd"),
    ("word/styles.xml", "tblInd@w"),
    ("word/styles.xml", "tblInd@type"),
    // `w:titlePg w:val="0"`: off is the default (§17.10.6). A real different first page is
    // checked in `the_tinta_documents_still_say_what_they_said` (the letter).
    ("word/document.xml", "titlePg"),
    ("word/document.xml", "titlePg@val"),
    // `w:hint="default"` on a bullet's font: the default script hint (§17.3.2.26).
    ("word/numbering.xml", "rFonts@hint"),
];

fn save(doc: &Document) -> Vec<u8> {
    wordcraft_docx::write(doc).expect("write")
}

fn open(bytes: &[u8]) -> Document {
    wordcraft_docx::read(bytes).expect("read")
}

#[test]
fn the_tinta_documents_read_back_equal_after_a_save() {
    for (name, pkg) in all() {
        let first = open(&pkg.bytes());
        let saved = save(&first);
        let second = open(&saved);
        // The settings part as read is kept to merge into the next save (`settings_merge.rs`);
        // after a save it is WordCraft's own settings, so it is the one thing allowed to differ.
        let (mut a, mut b) = (first.clone(), second.clone());
        for d in [&mut a, &mut b] {
            d.passthrough.remove(SETTINGS_AS_READ);
        }
        // A document with charts, SmartArt or embedded objects keeps the relationship ids and
        // media names it was read with, and a save numbers them afresh; after that save it reads
        // back the same every time (below), and `a_save_keeps_every_…` checks nothing was lost.
        if pkg.objects.is_empty() {
            assert!(a == b, "{name}: the document changed after open → save → open");
        }
        let mut third = open(&save(&second));
        third.passthrough.remove(SETTINGS_AS_READ);
        assert!(b == third, "{name}: the document changed after a second save");
        // A second save of the reopened document writes the same parts, byte for byte.
        let again = unzip(&save(&second));
        let saved = unzip(&saved);
        assert_eq!(saved.keys().collect::<Vec<_>>(), again.keys().collect::<Vec<_>>(), "{name}: parts differ between saves");
        for (part, bytes) in &saved {
            if part == "docProps/core.xml" || part == "docProps/app.xml" {
                continue; // the save time and statistics
            }
            assert!(again.get(part) == Some(bytes), "{name}: {part} differs between the first and the second save");
        }
    }
}

#[test]
fn a_save_keeps_every_element_attribute_and_text_of_the_tinta_documents() {
    let mut losses = Vec::new();
    for (name, pkg) in all() {
        let original = unzip(&pkg.bytes());
        let saved = unzip(&save(&open(&pkg.bytes())));
        for (part, bytes) in &original {
            let is_wml = part.starts_with("word/") && part.ends_with(".xml") && !part.contains("_rels");
            if !is_wml {
                continue;
            }
            let Some(out) = saved.get(part) else {
                losses.push(format!("{name}: the save dropped {part}"));
                continue;
            };
            let (before, after) = (vocabulary(bytes), vocabulary(out));
            let lost: Vec<_> =
                before.names.difference(&after.names).filter(|n| !DEFAULTS_LEFT_OUT.iter().any(|(p, d)| p == part && d == n)).collect();
            if !lost.is_empty() {
                losses.push(format!("{name}: {part} lost {lost:?}"));
            }
            if squash(&before.text) != squash(&after.text) {
                losses.push(format!("{name}: {part} text changed:\n  {}\n  {}", squash(&before.text), squash(&after.text)));
            }
        }
        // Relationships: every hyperlink target, and a relationship of every type, survive.
        for (part, bytes) in original.iter().filter(|(p, _)| p.ends_with(".rels")) {
            let after = saved.get(part).map(|b| relationships(b)).unwrap_or_default();
            for (ty, target) in relationships(bytes) {
                let kept = after.iter().any(|(t, g)| *t == ty && (g == &target || !ty.ends_with("/hyperlink")));
                if !kept {
                    losses.push(format!("{name}: {part} lost the {ty} relationship to {target}"));
                }
            }
        }
        // Document properties: every element of the core properties is still written.
        let core_names = |b: &[u8]| -> BTreeSet<String> {
            let s = String::from_utf8_lossy(b);
            s.split('<')
                .filter_map(|t| t.split(['>', ' ', '/']).next())
                .filter(|t| t.contains(':') && !t.starts_with('?'))
                .map(str::to_string)
                .collect()
        };
        let (before, after) = (core_names(&original["docProps/core.xml"]), saved.get("docProps/core.xml").map(|b| core_names(b)).unwrap_or_default());
        let lost: Vec<_> = before.difference(&after).collect();
        if !lost.is_empty() {
            losses.push(format!("{name}: docProps/core.xml lost {lost:?}"));
        }
        // Every other part (pictures, charts and their workbooks, SmartArt, embedded objects) is
        // carried byte for byte, under the same content type, whatever name it gets.
        let (types_before, types_after) = (
            String::from_utf8_lossy(&original["[Content_Types].xml"]).into_owned(),
            saved.get("[Content_Types].xml").map(|b| String::from_utf8_lossy(b).into_owned()).unwrap_or_default(),
        );
        for (part, bytes) in original.iter().filter(|(p, _)| !rewritten(p)) {
            // A SmartArt data part names its drawing by relationship id, which a save renumbers.
            let unnumbered = |b: &[u8]| {
                let s = String::from_utf8_lossy(b);
                match s.split_once(" relId=\"") {
                    Some((head, tail)) => format!("{head} relId=\"{}", tail.split_once('"').map(|(_, t)| t).unwrap_or("")).into_bytes(),
                    None => b.to_vec(),
                }
            };
            match saved.iter().find(|(_, b)| *b == bytes || unnumbered(b) == unnumbered(bytes)) {
                None => losses.push(format!("{name}: the save dropped {part}")),
                Some((kept, _)) => {
                    let (before, after) = (content_type(&types_before, part), content_type(&types_after, kept));
                    if before != after {
                        losses.push(format!("{name}: {part} (now {kept}) changed content type {before:?} → {after:?}"));
                    }
                }
            }
        }
    }
    assert!(losses.is_empty(), "open → save lost:\n{}", losses.join("\n"));
}

fn body_text(doc: &Document) -> String {
    let mut out = String::new();
    for b in doc.body.iter() {
        match &**b {
            wordcraft_doc::Block::Para(p) => out.push_str(&p.plain_text()),
            wordcraft_doc::Block::Table(t) => {
                for row in &t.rows {
                    for cell in &row.cells {
                        for b in cell.blocks.iter() {
                            if let Some(p) = b.as_para() {
                                out.push_str(&p.plain_text());
                                out.push('\n');
                            }
                        }
                    }
                }
            }
        }
        out.push('\n');
    }
    out
}

/// What a person checks by eye after reopening: the words, numbers and parts are all there.
#[test]
fn the_tinta_documents_still_say_what_they_said() {
    let docs: BTreeMap<_, _> = all().into_iter().map(|(n, p)| (n, open(&save(&open(&p.bytes()))))).collect();

    let f = &docs["factura"];
    let text = body_text(f);
    for s in [
        "FACTURA nº 2026-014",
        "María Núñez Ibáñez",
        "1.630,00 €",
        "IVA (21 %)",
        "Retención IRPF (−15 %)",
        "1.727,80 €",
        "ES91 2100 0418 4502 0005 1332",
    ] {
        assert!(text.contains(s), "factura lost {s:?}: {text}");
    }
    assert_eq!(f.media.len(), 1, "factura: the header logo");
    assert_eq!(f.core.title, "Factura 2026-014");
    assert_eq!(f.core.creator, "María Núñez Ibáñez");
    assert_eq!((f.core.language.as_str(), f.core.content_status.as_str(), f.core.version.as_str()), ("es-ES", "Final", "1.2"));
    assert_eq!(f.core.last_printed, "2026-10-05T17:35:00Z");
    assert_eq!((f.last_section.page_w, f.last_section.page_h), (11906.0 / 20.0, 16838.0 / 20.0), "factura: A4");

    let c = &docs["contrato"];
    let text = body_text(c);
    for s in ["CONTRATO DE PRESTACIÓN DE SERVICIOS", "«la Profesional»", "Jordi Ferrándiz i Puig", "RGPD"] {
        assert!(text.contains(s), "contrato lost {s:?}");
    }
    let numbered = c.body.iter().filter_map(|b| b.as_para()).filter(|p| p.props.numbering.is_some()).count();
    assert_eq!(numbered, 7, "contrato: the numbered clauses");

    let z = &docs["rezume"];
    let text = body_text(z);
    for s in ["Олена Ковальчук", "Пам'ятка", "«Ґрунт-Агро»", "Житомирський", "olena.kovalchuk@example.com"]
    {
        assert!(text.contains(s), "резюме lost {s:?}");
    }

    let k = &docs["carta"];
    let text = body_text(k);
    for s in ["sol·licitud de llicència d’activitat", "Plaça de l’Ajuntament", "expedient 2026/0417-LA", "Atentament,"] {
        assert!(text.contains(s), "carta lost {s:?}");
    }
    assert!(k.last_section.title_page, "carta: different first page");

    let i = &docs["informe"];
    let objects: Vec<_> = i.body.iter().filter_map(|b| b.as_para()).flat_map(|p| p.objects.iter()).collect();
    let kinds: Vec<_> =
        objects.iter().filter_map(|o| if let wordcraft_doc::InlineObject::Graphic { graphic, .. } = o { Some(graphic.kind) } else { None }).collect();
    assert_eq!(
        kinds,
        [wordcraft_doc::graphic::GraphicKind::Chart, wordcraft_doc::graphic::GraphicKind::Diagram],
        "informe: the chart and the SmartArt"
    );
    assert!(objects.iter().any(|o| matches!(o, wordcraft_doc::InlineObject::Image { ole: Some(_), .. })), "informe: the embedded spreadsheet");
}
