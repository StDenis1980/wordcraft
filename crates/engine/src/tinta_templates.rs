//! Tinta Office: templates for Spain (File › New): an invoice, a quote, a formal letter and a
//! CV, in Spanish, on A4, with the fields Spanish rules and customs expect. Our own text and
//! colours, generated in code (no Microsoft template or asset). The bracketed texts are for the
//! user to replace.

use wordcraft_doc::para::InlineObject;
use wordcraft_doc::props::{Align, Border, BorderStyle, Borders, CharProps, NumRef, Rgb, TabAlign, TabLeader, TabStop, TextColor};
use wordcraft_doc::{Block, Document, ListKind, Paragraph, PartKind, Table, para_block};

/// The template ids `file.new` takes, in gallery order.
pub const IDS: [&str; 4] = ["es-invoice", "es-quote", "es-letter", "es-cv"];

/// The template `id` names ([`IDS`]), or `None`.
pub fn template(id: &str) -> Option<Document> {
    Some(match id {
        "es-invoice" => factura(),
        "es-quote" => presupuesto(),
        "es-letter" => carta_formal(),
        "es-cv" => curriculum(),
        _ => return None,
    })
}

/// Tinta's ink blue, and a light tint of it for table headers and totals.
const INK: Rgb = Rgb(0x1F, 0x38, 0x64);
const TINT: Rgb = Rgb(0xDE, 0xE6, 0xF2);
/// A4 with 2.5 cm side margins and 2 cm top and bottom: the text is 453.5 pt wide.
const TEXT_W: f32 = 595.3 - 2.0 * 70.9;

fn base(title: &str) -> Document {
    let mut d = Document::new();
    d.core.title = title.into();
    d.core.language = "es-ES".into();
    d.styles.default_chr.lang = Some("es-ES".into());
    let s = &mut d.last_section;
    (s.page_w, s.page_h) = (595.3, 841.9);
    (s.margin_left, s.margin_right, s.margin_top, s.margin_bottom) = (70.9, 70.9, 56.7, 56.7);
    d
}

fn run(p: &mut Paragraph, text: &str, props: CharProps) {
    let at = p.len();
    let _ = p.insert_text(at, text, &props);
}

fn para(text: &str) -> Paragraph {
    Paragraph::with_text(text, CharProps::default())
}

fn bold() -> CharProps {
    CharProps { bold: Some(true), ..Default::default() }
}

fn ink(bold: bool, size: Option<f32>) -> CharProps {
    CharProps { bold: Some(bold), color: Some(TextColor::Rgb(INK)), size, ..Default::default() }
}

fn small() -> CharProps {
    CharProps { size: Some(8.5), color: Some(TextColor::Rgb(Rgb(0x59, 0x59, 0x59))), ..Default::default() }
}

/// "Label: value" with the label in bold.
fn labelled(label: &str, value: &str) -> Paragraph {
    let mut p = Paragraph::new();
    run(&mut p, label, bold());
    run(&mut p, value, CharProps::default());
    p
}

fn aligned(mut p: Paragraph, align: Align) -> Paragraph {
    p.props.align = Some(align);
    p
}

fn after(mut p: Paragraph, space: f32) -> Paragraph {
    p.props.space_after = Some(space);
    p
}

/// A heading in the documents' own look: ink, bold, a rule under it.
fn heading(text: &str) -> Paragraph {
    let mut p = Paragraph::new();
    run(&mut p, text, ink(true, Some(12.0)));
    p.props.space_before = Some(12.0);
    p.props.space_after = Some(4.0);
    p.props.keep_next = Some(true);
    p.props.borders =
        Some(Borders { bottom: Some(Border { style: BorderStyle::Single, width: 0.75, color: Some(INK), space: 2.0 }), ..Default::default() });
    p
}

fn no_borders() -> Borders {
    Borders::all(Border { style: BorderStyle::None, width: 0.0, color: None, space: 0.0 })
}

/// A table of `rows` with column widths `widths`; the first row is a repeated header on ink.
fn lines_table(rows: &[Vec<&str>], widths: &[f32], right_from: usize) -> Table {
    let mut t = Table::new(rows.len(), widths.len(), widths.iter().sum());
    t.grid = widths.to_vec();
    for (r, row) in rows.iter().enumerate() {
        let Some(tr) = t.rows.get_mut(r) else { continue };
        tr.props.header = r == 0;
        for (c, text) in row.iter().enumerate() {
            let Some(cell) = tr.cells.get_mut(c) else { continue };
            cell.props.width = widths.get(c).copied();
            let mut p = Paragraph::new();
            if r == 0 {
                cell.props.shading = Some(INK);
                run(&mut p, text, CharProps { bold: Some(true), color: Some(TextColor::Rgb(Rgb(0xFF, 0xFF, 0xFF))), ..Default::default() });
            } else {
                run(&mut p, text, CharProps::default());
            }
            if c >= right_from {
                p.props.align = Some(Align::Right);
            }
            cell.blocks = vec![para_block(p)];
        }
    }
    t
}

/// The totals block under a lines table: right-aligned label/amount rows, the last one in bold
/// on the tint.
fn totals(rows: &[(&str, &str)]) -> Table {
    let widths = [170.0, 100.0];
    let mut t = Table::new(rows.len(), 2, widths.iter().sum());
    t.grid = widths.to_vec();
    t.props.align = Some(Align::Right);
    t.props.borders = Some(no_borders());
    let last = rows.len().saturating_sub(1);
    for (r, (label, value)) in rows.iter().enumerate() {
        let Some(tr) = t.rows.get_mut(r) else { continue };
        for (c, text) in [label, value].into_iter().enumerate() {
            let Some(cell) = tr.cells.get_mut(c) else { continue };
            cell.props.width = widths.get(c).copied();
            let mut p = Paragraph::new();
            run(&mut p, text, if r == last { ink(true, Some(12.0)) } else { CharProps::default() });
            p.props.align = Some(Align::Right);
            if r == last {
                cell.props.shading = Some(TINT);
            }
            cell.blocks = vec![para_block(p)];
        }
    }
    t
}

/// Two side-by-side blocks (sender and recipient) in a borderless two-column table.
fn two_blocks(left: &[Paragraph], right: &[Paragraph]) -> Table {
    let half = TEXT_W / 2.0;
    let mut t = Table::new(1, 2, TEXT_W);
    t.grid = vec![half, half];
    t.props.borders = Some(no_borders());
    if let Some(tr) = t.rows.first_mut() {
        for (cell, blocks) in tr.cells.iter_mut().zip([left, right]) {
            cell.props.width = Some(half);
            cell.blocks = blocks.iter().cloned().map(para_block).collect();
        }
    }
    t
}

/// A footer with "Página X de Y" centred, in small grey type.
fn page_footer(d: &mut Document, note: &str) {
    let field =
        |instr: &str| InlineObject::Field { instr: instr.into(), result: "1".into(), locked: false, dirty: false, code: false, form: String::new() };
    let mut f = Paragraph::new().styled("Footer");
    f.props.align = Some(Align::Center);
    if !note.is_empty() {
        run(&mut f, &format!("{note} · "), small());
    }
    run(&mut f, "Página ", small());
    let _ = f.insert_object(f.len(), field("PAGE"), &small());
    run(&mut f, " de ", small());
    let _ = f.insert_object(f.len(), field("NUMPAGES"), &small());
    let id = d.add_part(PartKind::Footer, vec![para_block(f)]);
    d.last_section.footers.default = Some(id);
}

fn sender() -> Vec<Paragraph> {
    vec![
        after(labelled("", "[Nombre y apellidos o razón social]"), 0.0),
        after(para("NIF: [00000000X]"), 0.0),
        after(para("[Calle, número, piso] · [CP] [Localidad] ([Provincia])"), 0.0),
        para("[correo@ejemplo.es] · [600 000 000]"),
    ]
}

fn client(title: &str) -> Vec<Paragraph> {
    let mut head = Paragraph::new();
    run(&mut head, title, ink(true, None));
    vec![
        after(head, 0.0),
        after(para("[Nombre o razón social del cliente]"), 0.0),
        after(para("NIF/CIF: [B00000000]"), 0.0),
        para("[Dirección completa del cliente]"),
    ]
}

/// Factura: the data a Spanish invoice must carry (Real Decreto 1619/2012, art. 6): number and
/// series, date, both parties' names, NIF and address, the description, the tax base, the VAT
/// rate and amount and the total; the IRPF withholding most autónomos apply to businesses.
pub fn factura() -> Document {
    let mut d = base("Factura");
    let mut title = Paragraph::new();
    run(&mut title, "FACTURA", ink(true, Some(26.0)));
    let mut emitter = vec![title];
    emitter.extend(sender());
    let mut data = vec![
        aligned(after(labelled("Nº de factura: ", "[2026-001]"), 0.0), Align::Right),
        aligned(after(labelled("Fecha de expedición: ", "[dd/mm/aaaa]"), 0.0), Align::Right),
        aligned(labelled("Fecha de la operación: ", "[si es distinta]"), Align::Right),
    ];
    data.extend(client("Facturar a"));
    let mut b = vec![Block::Table(two_blocks(&emitter, &data))];
    b.push(Block::Para(para("")));
    let rows = vec![
        vec!["Concepto", "Cantidad", "Precio unitario", "Importe"],
        vec!["[Descripción del servicio o producto]", "1", "0,00 €", "0,00 €"],
        vec!["[Descripción]", "1", "0,00 €", "0,00 €"],
        vec!["", "", "", ""],
    ];
    b.push(Block::Table(lines_table(&rows, &[TEXT_W - 255.0, 65.0, 95.0, 95.0], 1)));
    b.push(Block::Para(para("")));
    b.push(Block::Table(totals(&[
        ("Base imponible", "0,00 €"),
        ("IVA (21 %)", "0,00 €"),
        ("Retención IRPF (−15 %)", "0,00 €"),
        ("TOTAL A PAGAR", "0,00 €"),
    ])));
    b.push(Block::Para(heading("Forma de pago")));
    b.push(Block::Para(after(para("Transferencia bancaria a la cuenta IBAN [ES00 0000 0000 0000 0000 0000]."), 0.0)));
    b.push(Block::Para(para("Vencimiento: [dd/mm/aaaa].")));
    let mut note = Paragraph::new();
    run(
        &mut note,
        "[Si la operación está exenta de IVA, indique aquí el artículo de la Ley 37/1992 que la ampara. Si no aplica retención, borre esa línea.]",
        small(),
    );
    b.push(Block::Para(note));
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    page_footer(&mut d, "[Nombre o razón social] · NIF [00000000X]");
    d
}

/// Presupuesto: a quote with its number, validity, lines, totals, conditions and a place for
/// the client's acceptance.
pub fn presupuesto() -> Document {
    let mut d = base("Presupuesto");
    let mut title = Paragraph::new();
    run(&mut title, "PRESUPUESTO", ink(true, Some(26.0)));
    let mut emitter = vec![title];
    emitter.extend(sender());
    let mut data = vec![
        aligned(after(labelled("Nº de presupuesto: ", "[P-2026-001]"), 0.0), Align::Right),
        aligned(after(labelled("Fecha: ", "[dd/mm/aaaa]"), 0.0), Align::Right),
        aligned(labelled("Válido hasta: ", "[dd/mm/aaaa] (30 días)"), Align::Right),
    ];
    data.extend(client("Para"));
    let mut b = vec![Block::Table(two_blocks(&emitter, &data))];
    b.push(Block::Para(heading("Descripción del trabajo")));
    b.push(Block::Para(para("[Explique en dos o tres frases qué incluye el trabajo y qué no.]")));
    let rows = vec![
        vec!["Concepto", "Cantidad", "Precio unitario", "Importe"],
        vec!["[Fase o partida]", "1", "0,00 €", "0,00 €"],
        vec!["[Fase o partida]", "1", "0,00 €", "0,00 €"],
        vec!["", "", "", ""],
    ];
    b.push(Block::Table(lines_table(&rows, &[TEXT_W - 255.0, 65.0, 95.0, 95.0], 1)));
    b.push(Block::Para(para("")));
    b.push(Block::Table(totals(&[("Base imponible", "0,00 €"), ("IVA (21 %)", "0,00 €"), ("TOTAL", "0,00 €")])));
    b.push(Block::Para(heading("Condiciones")));
    let mut num = d.numbering.clone();
    let list = num.add_list(ListKind::Bullet);
    d.numbering = num;
    for line in [
        "Plazo de entrega: [n] días desde la aceptación.",
        "Forma de pago: [50 % a la aceptación y 50 % a la entrega].",
        "Los precios no incluyen [desplazamientos, licencias, impresión…].",
    ] {
        let mut p = para(line).styled("ListParagraph");
        p.props.numbering = Some(NumRef { num: list, level: 0 });
        b.push(Block::Para(p));
    }
    b.push(Block::Para(heading("Aceptación")));
    b.push(Block::Para(para("Conforme con el presupuesto:")));
    let mut sign = para("Fdo.: ______________________________\tFecha: ____ / ____ / ________");
    sign.props.space_before = Some(36.0);
    sign.props.tabs = Some(vec![TabStop { pos: TEXT_W, align: TabAlign::Right, leader: TabLeader::None }]);
    b.push(Block::Para(sign));
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    page_footer(&mut d, "");
    d
}

/// Carta formal: a Spanish formal letter (to an administration or a company): sender, place
/// and date, recipient, reference and subject, salutation, body, closing and signature.
pub fn carta_formal() -> Document {
    let mut d = base("Carta formal");
    let mut b: Vec<Block> = sender().into_iter().map(Block::Para).collect();
    b.push(Block::Para(aligned(para("[Localidad], [día] de [mes] de [año]"), Align::Right)));
    for line in ["[Nombre del destinatario]", "[Cargo]", "[Organismo o empresa]", "[Dirección]"] {
        b.push(Block::Para(after(para(line), 0.0)));
    }
    b.push(Block::Para(para("[CP] [Localidad]")));
    let mut reference = labelled("Ref.: ", "[número de expediente o de cliente]");
    reference.props.tabs = Some(vec![TabStop { pos: TEXT_W, align: TabAlign::Right, leader: TabLeader::Dot }]);
    b.push(Block::Para(after(reference, 0.0)));
    b.push(Block::Para(labelled("Asunto: ", "[en pocas palabras, de qué trata la carta]")));
    b.push(Block::Para(para("Estimado/a [señor/señora + apellido]:")));
    for text in [
        "[Primer párrafo: quién es usted y por qué escribe.]",
        "[Segundo párrafo: los hechos o la petición, con fechas, importes y documentos que adjunta.]",
        "[Último párrafo: qué espera que ocurra y cómo pueden contactar con usted.]",
    ] {
        let mut p = aligned(para(text), Align::Justify);
        p.props.indent_first = Some(28.35);
        b.push(Block::Para(p));
    }
    b.push(Block::Para(para("Sin otro particular, reciba un cordial saludo.")));
    let mut name = para("[Nombre y apellidos]");
    name.props.space_before = Some(48.0);
    b.push(Block::Para(after(name, 0.0)));
    b.push(Block::Para(para("[Cargo, si procede]")));
    let mut annex = Paragraph::new();
    run(&mut annex, "Documentos adjuntos: [lista]", small());
    b.push(Block::Para(annex));
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    d
}

/// Currículum: a one-page Spanish CV with personal data, profile, experience, education,
/// languages (CEFR levels) and skills, and the data-protection consent line employers in Spain
/// expect.
pub fn curriculum() -> Document {
    let mut d = base("Currículum vítae");
    let mut name = Paragraph::new();
    run(&mut name, "[Nombre y apellidos]", ink(true, Some(24.0)));
    let mut b = vec![Block::Para(after(name, 2.0))];
    let mut role = Paragraph::new();
    run(
        &mut role,
        "[Profesión o puesto al que opta]",
        CharProps { italic: Some(true), color: Some(TextColor::Rgb(Rgb(0x59, 0x59, 0x59))), size: Some(12.0), ..Default::default() },
    );
    b.push(Block::Para(role));
    b.push(Block::Para(para("[Localidad] · [600 000 000] · [correo@ejemplo.es] · [web o LinkedIn]")));
    let dated = |what: &str, when: &str| {
        let mut p = Paragraph::new();
        run(&mut p, what, bold());
        run(&mut p, &format!("\t{when}"), CharProps { italic: Some(true), ..Default::default() });
        p.props.tabs = Some(vec![TabStop { pos: TEXT_W, align: TabAlign::Right, leader: TabLeader::None }]);
        p.props.space_after = Some(0.0);
        p
    };
    b.push(Block::Para(heading("Perfil profesional")));
    b.push(Block::Para(para("[Tres o cuatro líneas: qué sabe hacer, años de experiencia y qué busca.]")));
    b.push(Block::Para(heading("Experiencia profesional")));
    for (what, when) in
        [("[Puesto] — [Empresa], [Localidad]", "[mes/año] – actualidad"), ("[Puesto] — [Empresa], [Localidad]", "[mes/año] – [mes/año]")]
    {
        b.push(Block::Para(dated(what, when)));
        b.push(Block::Para(para("[Funciones y logros, mejor con cifras.]")));
    }
    b.push(Block::Para(heading("Formación")));
    b.push(Block::Para(dated("[Título] — [Centro]", "[año]")));
    b.push(Block::Para(para("[Cursos complementarios, con horas y año.]")));
    b.push(Block::Para(heading("Idiomas")));
    b.push(Block::Para(after(para("Español: nativo"), 0.0)));
    b.push(Block::Para(para("[Idioma]: [nivel A1–C2 del MCER] ([certificado])")));
    b.push(Block::Para(heading("Informática y otras competencias")));
    b.push(Block::Para(para("[Programas, carné de conducir, disponibilidad…]")));
    let mut consent = Paragraph::new();
    consent.props.space_before = Some(18.0);
    run(
        &mut consent,
        "Autorizo el tratamiento de mis datos personales incluidos en este currículum para participar en procesos de selección, conforme al Reglamento (UE) 2016/679 (RGPD) y la Ley Orgánica 3/2018 (LOPDGDD).",
        small(),
    );
    b.push(Block::Para(consent));
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_template_is_spanish_a4_and_lays_out() {
        for id in IDS {
            let d = template(id).unwrap();
            assert_eq!(d.styles.default_chr.lang.as_deref(), Some("es-ES"), "{id}");
            assert_eq!(d.core.language, "es-ES", "{id}");
            assert_eq!((d.last_section.page_w, d.last_section.page_h), (595.3, 841.9), "{id}: A4");
            assert!(d.word_count() > 30, "{id}");
            let l = wordcraft_layout::layout(&d, &mut wordcraft_layout::LayoutCache::new(), &Default::default());
            assert_eq!(l.pages.len(), 1, "{id} fits on one page");
        }
        assert!(template("no-such-template").is_none());
    }

    #[test]
    fn the_invoice_carries_what_spanish_law_asks_of_an_invoice() {
        let text = factura().plain_text(wordcraft_doc::StoryRef::Body);
        for s in [
            "FACTURA",
            "Nº de factura",
            "Fecha de expedición",
            "NIF",
            "Concepto",
            "Base imponible",
            "IVA (21 %)",
            "Retención IRPF",
            "TOTAL A PAGAR",
            "IBAN",
        ] {
            assert!(text.contains(s), "the invoice lacks {s:?}");
        }
    }

    #[test]
    fn the_cv_ends_with_the_data_protection_consent() {
        let text = curriculum().plain_text(wordcraft_doc::StoryRef::Body);
        assert!(text.trim_end().ends_with("(LOPDGDD)."), "{text}");
    }

    /// File › New opens each template by id, and an unknown id still opens a blank document.
    #[test]
    fn file_new_opens_the_spanish_templates() {
        let mut s = crate::Session::new(Document::new());
        for id in IDS {
            s.run("file.new", &serde_json::json!({ "template": id })).unwrap();
            let want = template(id).unwrap().plain_text(wordcraft_doc::StoryRef::Body);
            assert_eq!(s.doc.plain_text(wordcraft_doc::StoryRef::Body), want, "{id}");
        }
        s.run("file.new", &serde_json::json!({ "template": "es-nothing" })).unwrap();
        assert_eq!(s.doc.plain_text(wordcraft_doc::StoryRef::Body).trim(), "");
    }

    /// A template saved as .docx or .odt (what Spanish administrations ask for) reopens with the
    /// same text, the same tables and the same page.
    #[test]
    fn the_templates_survive_docx_and_odt() {
        for id in IDS {
            let d = template(id).unwrap();
            let tables = |d: &Document| d.body.iter().filter(|b| matches!(***b, Block::Table(_))).count();
            for name in ["t.docx", "t.odt"] {
                let bytes = crate::io::save_bytes(name, &d).unwrap();
                let back = crate::io::open_bytes(name, &bytes).unwrap();
                let words = |d: &Document| d.plain_text(wordcraft_doc::StoryRef::Body).split_whitespace().map(str::to_owned).collect::<Vec<_>>();
                assert_eq!(words(&back), words(&d), "{id} {name}");
                assert_eq!(tables(&back), tables(&d), "{id} {name}");
                assert!((back.last_section.page_w - 595.3).abs() < 0.5 && (back.last_section.page_h - 841.9).abs() < 0.5, "{id} {name}: A4");
                assert_eq!(back.last_section.footers.default.is_some(), d.last_section.footers.default.is_some(), "{id} {name}: footer");
                assert!((back.last_section.margin_top - d.last_section.margin_top).abs() < 0.05, "{id} {name}: top margin");
            }
        }
    }
}
