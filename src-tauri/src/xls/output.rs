use crate::domain::{asiento::LineaAsiento, error::AppError};
use rust_xlsxwriter::Workbook;

const HEADERS: [&str; 42] = [
    "Origen", "Num.Voucher", "Fecha    ", "Cuenta   ", "Monto Debe", "Monto Haber",
    "Moneda S/D ", "T.Cambio", "Doc", "Num.Doc     ", "Fec.Doc     ", "Fec.Ven    ",
    "Cod.Prov.Clie", "C.Costo", "Presupuesto", "F.Efectivo", "Glosa     ", "Libro C/V/R",
    "Mto.Neto 1", "Mto.Neto 2", "Mto.Neto 3", "Mto.Neto 4", "Mto.Neto 5", "Mto.Neto 6",
    "Mto.Neto 7", "Mto.Neto 8", "Mto.Neto 9", "Mto.IGV   ", "Ref.Doc", "Ref.Num.Doc",
    "Ref.Fecha", "D.Numero", "D.Fecha", "RUC        ", "R.Social", "Tipo",
    "Tip.Doc.Iden", "Medio de Pago", "Apellido 1   ", "Apellido 2   ", "Nombre       ", "T.Bien   ",
];

fn xls<E: std::fmt::Display>(e: E) -> AppError {
    AppError::Xls(e.to_string())
}

/// Serializa un Vec<LineaAsiento> al formato XLS que importa SISCONT.
/// Devuelve los bytes (in-memory, no toca disco) listos para base64 al frontend.
pub fn write_asientos(lineas: &[LineaAsiento]) -> Result<Vec<u8>, AppError> {
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.set_name("Hoja1").map_err(xls)?;

    for (col, h) in HEADERS.iter().enumerate() {
        ws.write_string(0, col as u16, *h).map_err(xls)?;
    }

    for (i, l) in lineas.iter().enumerate() {
        let r = (i + 1) as u32;
        let strs: [&str; 42] = [
            &l.origen, "", &l.fecha, &l.cuenta, "", "",
            &l.moneda, "", &l.doc, &l.num_doc, &l.fec_doc, &l.fec_ven,
            &l.cod_prov_clie, &l.c_costo, &l.presupuesto, &l.f_efectivo, &l.glosa, &l.libro_cvr,
            &l.mto_neto_1, &l.mto_neto_2, &l.mto_neto_3, &l.mto_neto_4, &l.mto_neto_5, &l.mto_neto_6,
            &l.mto_neto_7, &l.mto_neto_8, &l.mto_neto_9, &l.mto_igv, &l.ref_doc, &l.ref_num_doc,
            &l.ref_fecha, &l.d_numero, &l.d_fecha, &l.ruc, &l.r_social, &l.tipo,
            &l.tip_doc_iden, &l.medio_pago, &l.apellido_1, &l.apellido_2, &l.nombre, &l.t_bien,
        ];
        for (c, s) in strs.iter().enumerate() {
            if !s.is_empty() {
                ws.write_string(r, c as u16, *s).map_err(xls)?;
            }
        }
        ws.write_number(r, 1, l.num_voucher as f64).map_err(xls)?;
        if let Some(d) = l.debe {
            ws.write_number(r, 4, d).map_err(xls)?;
        }
        if let Some(h) = l.haber {
            ws.write_number(r, 5, h).map_err(xls)?;
        }
        ws.write_number(r, 7, l.tipo_cambio).map_err(xls)?;
    }

    wb.save_to_buffer().map_err(xls)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genera_xlsx_con_headers_y_filas() {
        let lineas = vec![
            LineaAsiento {
                origen: "14".into(),
                num_voucher: 1,
                fecha: "2025-12-31".into(),
                cuenta: "68613".into(),
                debe: Some(323.54),
                moneda: "S".into(),
                tipo_cambio: 3.36,
                doc: "00".into(),
                fec_doc: "2025-12-31".into(),
                fec_ven: "2025-12-31".into(),
                glosa: "DEPRECIACION DICIEMBRE 2025".into(),
                ..Default::default()
            },
            LineaAsiento {
                origen: "14".into(),
                num_voucher: 1,
                fecha: "2025-12-31".into(),
                cuenta: "39613".into(),
                haber: Some(323.54),
                moneda: "S".into(),
                tipo_cambio: 3.36,
                doc: "00".into(),
                fec_doc: "2025-12-31".into(),
                fec_ven: "2025-12-31".into(),
                glosa: "DEPRECIACION DICIEMBRE 2025".into(),
                ..Default::default()
            },
        ];
        let bytes = write_asientos(&lineas).expect("write");
        // xlsx es zip: empieza con PK\x03\x04
        assert_eq!(&bytes[..4], b"PK\x03\x04");
        assert!(bytes.len() > 1000, "xlsx demasiado chico");
    }
}
