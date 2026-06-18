use crate::domain::{asiento::LineaAsiento, error::AppError};
use calamine::{Data, Range, Reader, Xlsx};
use chrono::NaiveDate;
use std::io::Cursor;

const CUENTA_GASTO: &str = "68613";
const CUENTA_ACUMULADA: &str = "39613";
const DOC: &str = "00";
const MONEDA: &str = "S";
const C_COSTO_GASTO: &str = "3001";
const ORIGEN: &str = "14";

const MESES_NOMBRE: [&str; 12] = [
    "ENERO", "FEBRERO", "MARZO", "ABRIL", "MAYO", "JUNIO", "JULIO", "AGOSTO", "SETIEMBRE",
    "OCTUBRE", "NOVIEMBRE", "DICIEMBRE",
];

/// Columna 1-indexed del input donde está la depreciación de cada mes.
/// Saltos por columnas de totales semestrales y labels.
const COL_MES: [u32; 12] = [12, 13, 14, 15, 16, 17, 21, 22, 23, 24, 25, 26];

/// Parsea el xlsx de depreciación del cliente y devuelve el asiento del mes/año pedido.
///
/// Schema esperado en la sheet `DEP {anio}`: cabeceras en filas 7-8, datos desde fila 10.
/// Col 2 = código de cuenta del activo (numérico). Col del mes (ver `COL_MES`) = monto de depreciación.
/// Las filas TOTAL tienen texto en col 2 → se saltean automáticamente.
pub fn parse_depreciacion_xlsx(
    bytes: &[u8],
    mes: u8,
    anio: i32,
    tipo_cambio: f64,
) -> Result<Vec<LineaAsiento>, AppError> {
    if !(1..=12).contains(&mes) {
        return Err(AppError::Validation(format!("mes inválido: {mes}")));
    }
    let cursor = Cursor::new(bytes.to_vec());
    let mut book: Xlsx<Cursor<Vec<u8>>> = calamine::open_workbook_from_rs(cursor)
        .map_err(|e: calamine::XlsxError| AppError::Xls(e.to_string()))?;

    let sheet_name = format!("DEP {anio}");
    let range = book
        .worksheet_range(&sheet_name)
        .map_err(|e| AppError::Xls(format!("sheet `{sheet_name}` no encontrada: {e}")))?;

    extract_asiento(&range, mes, anio, tipo_cambio)
}

fn extract_asiento(
    range: &Range<Data>,
    mes: u8,
    anio: i32,
    tipo_cambio: f64,
) -> Result<Vec<LineaAsiento>, AppError> {
    let fecha = ultimo_dia_mes(anio, mes)?
        .format("%Y-%m-%d")
        .to_string();
    let glosa = format!("DEPRECIACIÓN {} {anio}", MESES_NOMBRE[(mes - 1) as usize]);
    // calamine usa coords absolutas con get_value. Headers en 1-indexed → 0-indexed:
    // - cuenta del activo: col 2 (1-idx) = col 1 (0-idx)
    // - mes: COL_MES (1-idx) - 1 = (0-idx)
    let col_cuenta: u32 = 1;
    let col_mes_abs: u32 = COL_MES[(mes - 1) as usize] - 1;
    // Datos desde R10 (1-idx) = row 9 (0-idx)
    let last_row = range.end().map(|(r, _)| r).unwrap_or(0);

    let mut lineas_haber: Vec<LineaAsiento> = Vec::new();
    let mut total: f64 = 0.0;

    for row in 9..=last_row {
        // Col cuenta debe ser numérica (filas TOTAL / encabezados tienen texto → skip).
        let cuenta_cell = range.get_value((row, col_cuenta));
        let es_cuenta = matches!(cuenta_cell, Some(Data::Int(_) | Data::Float(_)));
        if !es_cuenta {
            continue;
        }
        let monto = match range.get_value((row, col_mes_abs)) {
            Some(Data::Float(f)) if *f > 0.0 => *f,
            Some(Data::Int(i)) if *i > 0 => *i as f64,
            _ => continue,
        };
        total += monto;
        lineas_haber.push(LineaAsiento {
            origen: ORIGEN.into(),
            num_voucher: 1,
            fecha: fecha.clone(),
            cuenta: CUENTA_ACUMULADA.into(),
            haber: Some(monto),
            moneda: MONEDA.into(),
            tipo_cambio,
            doc: DOC.into(),
            fec_doc: fecha.clone(),
            fec_ven: fecha.clone(),
            glosa: glosa.clone(),
            ..Default::default()
        });
    }

    if lineas_haber.is_empty() {
        return Err(AppError::Validation(format!(
            "no hay depreciación para {} {anio}",
            MESES_NOMBRE[(mes - 1) as usize]
        )));
    }

    let mut lineas = vec![LineaAsiento {
        origen: ORIGEN.into(),
        num_voucher: 1,
        fecha: fecha.clone(),
        cuenta: CUENTA_GASTO.into(),
        debe: Some(redondear(total)),
        moneda: MONEDA.into(),
        tipo_cambio,
        doc: DOC.into(),
        fec_doc: fecha.clone(),
        fec_ven: fecha,
        c_costo: C_COSTO_GASTO.into(),
        glosa,
        ..Default::default()
    }];
    lineas.extend(lineas_haber);
    Ok(lineas)
}

fn ultimo_dia_mes(anio: i32, mes: u8) -> Result<NaiveDate, AppError> {
    let primer_dia_siguiente = if mes == 12 {
        NaiveDate::from_ymd_opt(anio + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(anio, (mes + 1) as u32, 1)
    };
    primer_dia_siguiente
        .and_then(|d| d.pred_opt())
        .ok_or_else(|| AppError::Validation(format!("fecha invalida: {anio}-{mes}")))
}

fn redondear(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test con el xlsx real del cliente. Solo corre con `cargo test --ignored`
    /// porque el archivo no se commitea (gitignore por data sensible).
    #[test]
    #[ignore]
    fn diciembre_2025_replica_output_referencia() {
        let bytes =
            std::fs::read("../public/input-depreciación.xlsx").expect("input local existe");
        let lineas = parse_depreciacion_xlsx(&bytes, 12, 2025, 3.36).expect("parse ok");

        assert_eq!(lineas.len(), 3, "1 debe + 2 haber esperados");
        // Línea 0 = debe en 68613 por 323.54
        assert_eq!(lineas[0].cuenta, "68613");
        assert_eq!(lineas[0].debe, Some(323.54));
        assert_eq!(lineas[0].c_costo, "3001");
        // Líneas 1-2 = haber en 39613
        let total_haber: f64 = lineas[1..].iter().filter_map(|l| l.haber).sum();
        assert!(
            (total_haber - 323.54).abs() < 0.01,
            "haber={total_haber} != 323.54"
        );
        assert!(lineas[1..].iter().all(|l| l.cuenta == "39613"));
        // Partida doble
        let td: f64 = lineas.iter().filter_map(|l| l.debe).sum();
        let th: f64 = lineas.iter().filter_map(|l| l.haber).sum();
        assert!((td - th).abs() < 0.01);
        // Metadata común
        assert!(
            lineas
                .iter()
                .all(|l| l.origen == "14" && l.fecha == "2025-12-31" && l.doc == "00")
        );
    }

    #[test]
    fn ultimo_dia_diciembre_es_31() {
        assert_eq!(
            ultimo_dia_mes(2025, 12).unwrap(),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()
        );
    }

    #[test]
    fn ultimo_dia_febrero_no_bisiesto() {
        assert_eq!(
            ultimo_dia_mes(2025, 2).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 28).unwrap()
        );
    }

    #[test]
    fn ultimo_dia_febrero_bisiesto() {
        assert_eq!(
            ultimo_dia_mes(2024, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()
        );
    }

    #[test]
    fn mes_invalido_falla() {
        assert!(parse_depreciacion_xlsx(&[], 0, 2025, 3.36).is_err());
        assert!(parse_depreciacion_xlsx(&[], 13, 2025, 3.36).is_err());
    }
}
