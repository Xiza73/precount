use crate::domain::{asiento::LineaAsiento, error::AppError};
use chrono::NaiveDate;
use std::io::Cursor;
use umya_spreadsheet::{
    reader::xlsx::read_reader, writer::xlsx::write_writer, NumberingFormat,
};

const TEMPLATE: &[u8] = include_bytes!("../../assets/template-siscont.xlsx");
const FECHA_FMT: &str = "dd/mm/yyyy";
const MONTO_FMT: &str = "#,##0.00";
const TCAMBIO_FMT: &str = "0.00";
const TEXT_FMT: &str = "@";

fn xls<E: std::fmt::Display>(e: E) -> AppError {
    AppError::Xls(e.to_string())
}

/// Escribe las líneas de asiento sobre el template embebido y devuelve los bytes
/// del xlsx final. Las filas se agrupan por `num_voucher` para emitir la fórmula
/// `=+F{r1}+F{r2}+…` en el debe de la primera línea, y cadenas de referencias en
/// las columnas K (Fec.Doc) y L (Fec.Ven) tal como en el output del cliente.
pub fn write_asientos(lineas: &[LineaAsiento]) -> Result<Vec<u8>, AppError> {
    let mut book = read_reader(Cursor::new(TEMPLATE.to_vec()), true).map_err(xls)?;
    let ws = book
        .get_sheet_by_name_mut("Hoja1")
        .ok_or_else(|| AppError::Xls("template sin Hoja1".into()))?;

    // Agrupar por num_voucher (las líneas vienen ya ordenadas: debe primero, después haber).
    let mut row: u32 = 2;
    let mut i = 0;
    while i < lineas.len() {
        let voucher = lineas[i].num_voucher;
        let start = i;
        while i < lineas.len() && lineas[i].num_voucher == voucher {
            i += 1;
        }
        let end = i; // exclusive
        let grupo = &lineas[start..end];

        // Filas haber: cada línea con `haber.is_some()` en orden.
        let haber_rows: Vec<u32> = grupo
            .iter()
            .enumerate()
            .filter(|(_, l)| l.haber.is_some())
            .map(|(idx, _)| row + idx as u32)
            .collect();

        for (idx, l) in grupo.iter().enumerate() {
            let r = row + idx as u32;
            write_linea(ws, r, l, &haber_rows)?;
        }
        row = end as u32 - start as u32 + row;
    }

    let mut buf = Cursor::new(Vec::<u8>::new());
    write_writer(&book, &mut buf).map_err(xls)?;
    Ok(buf.into_inner())
}

fn write_linea(
    ws: &mut umya_spreadsheet::Worksheet,
    r: u32,
    l: &LineaAsiento,
    haber_rows: &[u32],
) -> Result<(), AppError> {
    let fecha = parse_fecha(&l.fecha)?;
    let fecha_serial = excel_serial(fecha);

    // A: Origen (int)
    set_int(ws, "A", r, parse_u32(&l.origen)?);
    // B: Num.Voucher (int)
    set_int(ws, "B", r, l.num_voucher);
    // C: Fecha (datetime + dd/mm/yyyy)
    set_date(ws, "C", r, fecha_serial);
    // D: Cuenta (int)
    set_int(ws, "D", r, parse_u32(&l.cuenta)?);
    // E: Monto Debe — fórmula si es la línea del debe del asiento
    if l.debe.is_some() {
        let formula = if haber_rows.is_empty() {
            None
        } else {
            Some(format!(
                "+{}",
                haber_rows
                    .iter()
                    .map(|h| format!("F{h}"))
                    .collect::<Vec<_>>()
                    .join("+")
            ))
        };
        if let Some(f) = formula {
            ws.get_cell_mut(format!("E{r}").as_str()).set_formula(f);
        } else if let Some(d) = l.debe {
            set_money(ws, "E", r, d);
        }
    }
    // F: Monto Haber (valor con formato moneda)
    if let Some(h) = l.haber {
        set_money(ws, "F", r, h);
    }
    // G: Moneda (text)
    if !l.moneda.is_empty() {
        ws.get_cell_mut(format!("G{r}").as_str()).set_value_string(&l.moneda);
    }
    // H: T.Cambio (float con formato 0.00)
    set_float_fmt(ws, "H", r, l.tipo_cambio, TCAMBIO_FMT);
    // I: Doc (text con formato @)
    if !l.doc.is_empty() {
        set_text(ws, "I", r, &l.doc);
    }
    // J: Num.Doc
    if !l.num_doc.is_empty() {
        set_text(ws, "J", r, &l.num_doc);
    }
    // K: Fec.Doc. Si tiene valor distinto a la fecha del asiento → literal.
    // Si está vacío o coincide con la fecha → fórmula (=+C2 / =+K{r-1}).
    let cell_k = ws.get_cell_mut(format!("K{r}").as_str());
    if !l.fec_doc.is_empty() && l.fec_doc != l.fecha {
        cell_k.set_value_number(excel_serial(parse_fecha(&l.fec_doc)?));
    } else {
        let k_formula = if r == 2 { "+C2".to_string() } else { format!("+K{}", r - 1) };
        cell_k.set_formula(k_formula);
    }
    set_num_format(cell_k, FECHA_FMT);
    // L: Fec.Ven. Mismo criterio: literal si propio, fórmula =+K{r} si coincide.
    let cell_l = ws.get_cell_mut(format!("L{r}").as_str());
    if !l.fec_ven.is_empty() && l.fec_ven != l.fec_doc && l.fec_ven != l.fecha {
        cell_l.set_value_number(excel_serial(parse_fecha(&l.fec_ven)?));
    } else {
        cell_l.set_formula(format!("+K{r}"));
    }
    set_num_format(cell_l, FECHA_FMT);
    // M: Cod.Prov.Clie
    if !l.cod_prov_clie.is_empty() {
        set_text(ws, "M", r, &l.cod_prov_clie);
    }
    // N: C.Costo (int si parsea)
    if !l.c_costo.is_empty() {
        if let Ok(n) = l.c_costo.parse::<u32>() {
            set_int(ws, "N", r, n);
        } else {
            set_text(ws, "N", r, &l.c_costo);
        }
    }
    // O: Presupuesto
    if !l.presupuesto.is_empty() {
        set_text(ws, "O", r, &l.presupuesto);
    }
    // P: F.Efectivo
    if !l.f_efectivo.is_empty() {
        set_text(ws, "P", r, &l.f_efectivo);
    }
    // Q: Glosa (text @)
    if !l.glosa.is_empty() {
        set_text(ws, "Q", r, &l.glosa);
    }
    // R: Libro CVR
    if !l.libro_cvr.is_empty() {
        set_text(ws, "R", r, &l.libro_cvr);
    }
    // S..AA: Mto.Neto 1..9
    let mtos = [
        ("S", &l.mto_neto_1),
        ("T", &l.mto_neto_2),
        ("U", &l.mto_neto_3),
        ("V", &l.mto_neto_4),
        ("W", &l.mto_neto_5),
        ("X", &l.mto_neto_6),
        ("Y", &l.mto_neto_7),
        ("Z", &l.mto_neto_8),
        ("AA", &l.mto_neto_9),
    ];
    for (col, val) in mtos {
        if !val.is_empty() {
            set_text(ws, col, r, val);
        }
    }
    // AB: Mto.IGV
    if !l.mto_igv.is_empty() {
        set_text(ws, "AB", r, &l.mto_igv);
    }
    // AC..AG: Ref.Doc, Ref.Num.Doc, Ref.Fecha, D.Numero, D.Fecha
    let refs = [
        ("AC", &l.ref_doc),
        ("AD", &l.ref_num_doc),
        ("AE", &l.ref_fecha),
        ("AF", &l.d_numero),
        ("AG", &l.d_fecha),
    ];
    for (col, val) in refs {
        if !val.is_empty() {
            set_text(ws, col, r, val);
        }
    }
    // AH: RUC (text)
    if !l.ruc.is_empty() {
        set_text(ws, "AH", r, &l.ruc);
    }
    // AI..AP: R.Social, Tipo, Tip.Doc.Iden, Medio de Pago, Apellido1, Apellido2, Nombre, T.Bien
    let resto = [
        ("AI", &l.r_social),
        ("AJ", &l.tipo),
        ("AK", &l.tip_doc_iden),
        ("AL", &l.medio_pago),
        ("AM", &l.apellido_1),
        ("AN", &l.apellido_2),
        ("AO", &l.nombre),
        ("AP", &l.t_bien),
    ];
    for (col, val) in resto {
        if !val.is_empty() {
            set_text(ws, col, r, val);
        }
    }
    Ok(())
}

fn set_int(ws: &mut umya_spreadsheet::Worksheet, col: &str, r: u32, v: u32) {
    ws.get_cell_mut(format!("{col}{r}").as_str())
        .set_value_number(v as f64);
}

fn set_money(ws: &mut umya_spreadsheet::Worksheet, col: &str, r: u32, v: f64) {
    let cell = ws.get_cell_mut(format!("{col}{r}").as_str());
    cell.set_value_number(v);
    set_num_format(cell, MONTO_FMT);
}

fn set_float_fmt(
    ws: &mut umya_spreadsheet::Worksheet,
    col: &str,
    r: u32,
    v: f64,
    fmt: &str,
) {
    let cell = ws.get_cell_mut(format!("{col}{r}").as_str());
    cell.set_value_number(v);
    set_num_format(cell, fmt);
}

fn set_text(ws: &mut umya_spreadsheet::Worksheet, col: &str, r: u32, v: &str) {
    let cell = ws.get_cell_mut(format!("{col}{r}").as_str());
    cell.set_value_string(v);
    set_num_format(cell, TEXT_FMT);
}

fn set_date(ws: &mut umya_spreadsheet::Worksheet, col: &str, r: u32, serial: f64) {
    let cell = ws.get_cell_mut(format!("{col}{r}").as_str());
    cell.set_value_number(serial);
    set_num_format(cell, FECHA_FMT);
}

fn set_num_format(cell: &mut umya_spreadsheet::Cell, code: &str) {
    let mut nf = NumberingFormat::default();
    nf.set_format_code(code);
    cell.get_style_mut().set_numbering_format(nf);
}

fn parse_fecha(s: &str) -> Result<NaiveDate, AppError> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|e| AppError::Validation(format!("fecha invalida `{s}`: {e}")))
}

fn parse_u32(s: &str) -> Result<u32, AppError> {
    s.parse::<u32>()
        .map_err(|e| AppError::Validation(format!("entero invalido `{s}`: {e}")))
}

/// Excel "1900-mode" serial number. Epoch base es 1899-12-30 para que el bug
/// del año-bisiesto-1900 quede compensado y `excel_serial(1900-01-01) == 1`.
fn excel_serial(date: NaiveDate) -> f64 {
    let epoch = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
    (date - epoch).num_days() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn linea_debe(voucher: u32, cuenta: &str, debe: f64, glosa: &str) -> LineaAsiento {
        LineaAsiento {
            origen: "14".into(),
            num_voucher: voucher,
            fecha: "2025-12-31".into(),
            cuenta: cuenta.into(),
            debe: Some(debe),
            moneda: "S".into(),
            tipo_cambio: 3.36,
            doc: "00".into(),
            c_costo: "3001".into(),
            glosa: glosa.into(),
            ..Default::default()
        }
    }
    fn linea_haber(voucher: u32, cuenta: &str, haber: f64, glosa: &str) -> LineaAsiento {
        LineaAsiento {
            origen: "14".into(),
            num_voucher: voucher,
            fecha: "2025-12-31".into(),
            cuenta: cuenta.into(),
            haber: Some(haber),
            moneda: "S".into(),
            tipo_cambio: 3.36,
            doc: "00".into(),
            glosa: glosa.into(),
            ..Default::default()
        }
    }

    #[test]
    fn excel_serial_29_dic_1899_es_cero() {
        assert_eq!(excel_serial(NaiveDate::from_ymd_opt(1899, 12, 30).unwrap()), 0.0);
        assert_eq!(excel_serial(NaiveDate::from_ymd_opt(1900, 1, 1).unwrap()), 2.0);
    }

    #[test]
    fn genera_xlsx_valido_es_zip() {
        let glosa = "DEPRECIACIÓN DICIEMBRE 2025";
        let lineas = vec![
            linea_debe(1, "68613", 323.54, glosa),
            linea_haber(1, "39613", 213.79, glosa),
            linea_haber(1, "39613", 109.75, glosa),
        ];
        let bytes = write_asientos(&lineas).expect("write");
        assert_eq!(&bytes[..4], b"PK\x03\x04", "no es zip");
        assert!(bytes.len() > 4000);
    }

    #[test]
    fn formula_debe_referencia_filas_haber_correctas() {
        let glosa = "DEPRECIACIÓN DICIEMBRE 2025";
        let lineas = vec![
            linea_debe(1, "68613", 323.54, glosa),
            linea_haber(1, "39613", 213.79, glosa),
            linea_haber(1, "39613", 109.75, glosa),
        ];
        let bytes = write_asientos(&lineas).expect("write");
        // Releemos lo escrito para verificar la fórmula del debe.
        let book =
            read_reader(Cursor::new(bytes), true).expect("read back");
        let ws = book.get_sheet_by_name("Hoja1").expect("Hoja1");
        let e2 = ws.get_cell("E2").expect("E2 existe");
        assert_eq!(e2.get_formula(), "+F3+F4");
        let k2 = ws.get_cell("K2").expect("K2");
        assert_eq!(k2.get_formula(), "+C2");
        let l2 = ws.get_cell("L2").expect("L2");
        assert_eq!(l2.get_formula(), "+K2");
        let k3 = ws.get_cell("K3").expect("K3");
        assert_eq!(k3.get_formula(), "+K2");
        let k4 = ws.get_cell("K4").expect("K4");
        assert_eq!(k4.get_formula(), "+K3");
    }
}
