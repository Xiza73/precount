use crate::domain::{
    asiento::LineaAsiento,
    error::AppError,
    fecha::{nombre_mes, ultimo_dia_mes, validar_mes},
};
use calamine::{Data, Range, Reader, Sheets};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

const ORIGEN: &str = "11";
const DOC: &str = "00";
const MONEDA: &str = "S";
const C_COSTO: &str = "3002";

// Cuentas observadas en el output del cliente.
const CTA_SUELDOS: &str = "62111";
const CTA_VACACIONES: &str = "62711";
const CTA_ADELANTOS: &str = "14124";
const CTA_IR_5TA: &str = "40173";
const CTA_ESSALUD: &str = "40311";
const CTA_AFP: &str = "41724";
const CTA_NETO: &str = "41111";
const CTA_CTS_GASTO: &str = "62911";
const CTA_CTS_PASIVO: &str = "41511";
const CTA_GRATI_GASTO: &str = "62141";
const CTA_BONIF_GASTO: &str = "62901";
const CTA_GRATI_PASIVO: &str = "41141";
const CTA_BONIF_PASIVO: &str = "41901";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsientoPlanillaInput {
    pub mes: u8,
    pub anio: i32,
    pub tipo_cambio: f64,

    // Voucher 1 — PLANILLA. Si todos son 0, el voucher se omite.
    #[serde(default)]
    pub sueldos: f64,
    #[serde(default)]
    pub vacaciones: f64,
    #[serde(default)]
    pub adelantos: f64,
    #[serde(default)]
    pub ir_5ta: f64,
    #[serde(default)]
    pub essalud: f64,
    #[serde(default)]
    pub afp: f64,
    #[serde(default)]
    pub neto: f64,

    // Voucher 2 — EPS. Se omite si == 0.
    #[serde(default)]
    pub eps: f64,

    // Voucher 3 — CTS. Se omite si == 0.
    #[serde(default)]
    pub cts: f64,

    // Voucher 4 — GRATIFICACIÓN. Se omite si ambos == 0.
    #[serde(default)]
    pub gratificacion: f64,
    #[serde(default)]
    pub bonif_extraord: f64,
}

pub fn construir_planilla(input: &AsientoPlanillaInput) -> Result<Vec<LineaAsiento>, AppError> {
    let fecha = ultimo_dia_mes(input.anio, input.mes)?
        .format("%Y-%m-%d")
        .to_string();
    let mes = nombre_mes(input.mes)?;

    let mut lineas = Vec::new();
    let mut voucher: u32 = 0;

    // === Voucher 1: PLANILLA ===
    if input.sueldos > 0.0 {
        voucher += 1;
        let glosa = format!("PLANILLA {mes} {}", input.anio);
        let v = voucher;
        if input.sueldos > 0.0 {
            lineas.push(debe(v, &fecha, input.tipo_cambio, CTA_SUELDOS, input.sueldos, &glosa, true));
        }
        if input.vacaciones > 0.0 {
            lineas.push(debe(v, &fecha, input.tipo_cambio, CTA_VACACIONES, input.vacaciones, &glosa, true));
        }
        if input.adelantos > 0.0 {
            lineas.push(haber(v, &fecha, input.tipo_cambio, CTA_ADELANTOS, input.adelantos, &glosa));
        }
        if input.ir_5ta > 0.0 {
            lineas.push(haber(v, &fecha, input.tipo_cambio, CTA_IR_5TA, input.ir_5ta, &glosa));
        }
        if input.essalud > 0.0 {
            lineas.push(haber(v, &fecha, input.tipo_cambio, CTA_ESSALUD, input.essalud, &glosa));
        }
        if input.afp > 0.0 {
            lineas.push(haber(v, &fecha, input.tipo_cambio, CTA_AFP, input.afp, &glosa));
        }
        if input.neto > 0.0 {
            lineas.push(haber(v, &fecha, input.tipo_cambio, CTA_NETO, input.neto, &glosa));
        }
        validar_partida_doble(&lineas, v, "PLANILLA")?;
    }

    // === Voucher 2: EPS ===
    if input.eps > 0.0 {
        voucher += 1;
        let glosa = format!("EPS {mes} {}", input.anio);
        lineas.push(debe(voucher, &fecha, input.tipo_cambio, CTA_ESSALUD, input.eps, &glosa, false));
        lineas.push(haber(voucher, &fecha, input.tipo_cambio, CTA_ADELANTOS, input.eps, &glosa));
    }

    // === Voucher 3: CTS ===
    if input.cts > 0.0 {
        voucher += 1;
        let glosa = format!("CTS {mes} {}", input.anio);
        lineas.push(debe(voucher, &fecha, input.tipo_cambio, CTA_CTS_GASTO, input.cts, &glosa, true));
        lineas.push(haber(voucher, &fecha, input.tipo_cambio, CTA_CTS_PASIVO, input.cts, &glosa));
    }

    // === Voucher 4: GRATIFICACIÓN ===
    if input.gratificacion > 0.0 || input.bonif_extraord > 0.0 {
        voucher += 1;
        let glosa = format!("GRATIFICACION {mes} {}", input.anio);
        let v = voucher;
        if input.gratificacion > 0.0 {
            lineas.push(debe(v, &fecha, input.tipo_cambio, CTA_GRATI_GASTO, input.gratificacion, &glosa, true));
        }
        if input.bonif_extraord > 0.0 {
            lineas.push(debe(v, &fecha, input.tipo_cambio, CTA_BONIF_GASTO, input.bonif_extraord, &glosa, true));
        }
        if input.gratificacion > 0.0 {
            lineas.push(haber(v, &fecha, input.tipo_cambio, CTA_GRATI_PASIVO, input.gratificacion, &glosa));
        }
        if input.bonif_extraord > 0.0 {
            lineas.push(haber(v, &fecha, input.tipo_cambio, CTA_BONIF_PASIVO, input.bonif_extraord, &glosa));
        }
        validar_partida_doble(&lineas, v, "GRATIFICACION")?;
    }

    if lineas.is_empty() {
        return Err(AppError::Validation(
            "no hay ningún voucher con montos > 0".into(),
        ));
    }

    Ok(lineas)
}

/// Resumen de campos derivables del xls de planilla del cliente.
/// Los campos no derivables (vacaciones, ESSALUD final, gratif efectiva,
/// bonif, EPS, CTS) NO se incluyen — los completa el contador a mano.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumenPlanilla {
    pub sueldos: f64,
    pub afp: f64,
    pub ir_5ta: f64,
    pub neto: f64,
    pub adelantos: f64,
}

/// Pre-carga de campos del form a partir del xls (legacy .xls o .xlsx).
/// Devuelve 0 en cada campo que no logró extraer — no falla por sheets faltantes.
pub fn resumir_planilla_xls(
    bytes: &[u8],
    mes: u8,
    anio: i32,
) -> Result<ResumenPlanilla, AppError> {
    validar_mes(mes)?;
    let cursor = Cursor::new(bytes.to_vec());
    let mut book: Sheets<Cursor<Vec<u8>>> = calamine::open_workbook_auto_from_rs(cursor)
        .map_err(|e: calamine::Error| AppError::Xls(e.to_string()))?;

    let nombre = nombre_mes(mes)?;
    let sheet_names = book.sheet_names().to_vec();

    let mut r = ResumenPlanilla::default();

    // Sheet de planilla del mes: nombre del mes (DICIEMBRE, ENERO, ...).
    if let Some(sn) = sheet_names
        .iter()
        .find(|n| n.trim().eq_ignore_ascii_case(nombre))
    {
        if let Ok(rng) = book.worksheet_range(sn) {
            let (s, a) = extraer_de_planilla_sheet(&rng);
            r.sueldos = s;
            r.afp = a;
        }
    }

    // Sheet R. QUINTA del año.
    let prefix_quinta = format!("R. QUINTA {anio}");
    if let Some(sn) = sheet_names
        .iter()
        .find(|n| n.trim().to_uppercase().starts_with(&prefix_quinta))
    {
        if let Ok(rng) = book.worksheet_range(sn) {
            r.ir_5ta = extraer_ir_5ta(&rng, nombre);
        }
    }

    // Boletas: sheets nombradas con números puros ("1", "2", ...).
    for sn in &sheet_names {
        if sn.trim().parse::<u32>().is_ok() {
            if let Ok(rng) = book.worksheet_range(sn) {
                let (n, a) = extraer_de_boleta(&rng);
                r.neto += n;
                r.adelantos += a;
            }
        }
    }

    Ok(r)
}

fn read_num(rng: &Range<Data>, row: u32, col: u32) -> f64 {
    match rng.get_value((row, col)) {
        Some(Data::Float(f)) => *f,
        Some(Data::Int(i)) => *i as f64,
        _ => 0.0,
    }
}

fn read_str<'a>(rng: &'a Range<Data>, row: u32, col: u32) -> Option<&'a str> {
    match rng.get_value((row, col)) {
        Some(Data::String(s)) => Some(s.as_str()),
        _ => None,
    }
}

/// Sheet `DICIEMBRE` (o el mes): fila TOTAL en col 3 (idx 2). Sueldos = básico + asig fam
/// de esa fila. AFP = subtotal de la fila inmediatamente siguiente, col 26 (idx 25).
fn extraer_de_planilla_sheet(rng: &Range<Data>) -> (f64, f64) {
    let last_row = rng.end().map(|(r, _)| r).unwrap_or(0);
    let mut total_row: Option<u32> = None;
    for r in 0..=last_row {
        if let Some(s) = read_str(rng, r, 2) {
            if s.trim().eq_ignore_ascii_case("TOTAL") {
                total_row = Some(r);
                break;
            }
        }
    }
    let Some(r) = total_row else {
        return (0.0, 0.0);
    };
    let sueldo_basico = read_num(rng, r, 10); // col 11 (1-idx) = idx 10
    let asig_fam = read_num(rng, r, 13); // col 14 = idx 13
    let afp = read_num(rng, r + 1, 25); // fila siguiente, col 26 = idx 25
    (sueldo_basico + asig_fam, afp)
}

/// Sheet `R. QUINTA {anio}`: hacia el final del sheet hay una sección por mes con la
/// retención de cada trabajador en cols 3 / 6 / 9 (idx 2 / 5 / 8). Suma absoluta.
fn extraer_ir_5ta(rng: &Range<Data>, nombre_mes_upper: &str) -> f64 {
    let last_row = rng.end().map(|(r, _)| r).unwrap_or(0);
    for r in 0..=last_row {
        if let Some(s) = read_str(rng, r, 1) {
            if s.trim().eq_ignore_ascii_case(nombre_mes_upper) {
                let v1 = read_num(rng, r, 2).abs();
                let v2 = read_num(rng, r, 5).abs();
                let v3 = read_num(rng, r, 8).abs();
                // Filtrar el match dentro de la tabla de ingresos (que también tiene
                // los meses pero con números positivos altos en col 3).
                // En la sección de retenciones los valores son chicos (< 1000 típico).
                if v1 < 5000.0 && v2 < 5000.0 && v3 < 5000.0 {
                    return v1 + v2 + v3;
                }
            }
        }
    }
    0.0
}

/// Cada boleta: busca filas con label "Neto a Pagar" (col 9 = idx 8) y código "0706"
/// (col 8 = idx 7). Suma ambos a lo largo de la sheet.
fn extraer_de_boleta(rng: &Range<Data>) -> (f64, f64) {
    let last_row = rng.end().map(|(r, _)| r).unwrap_or(0);
    let mut neto = 0.0;
    let mut adelantos = 0.0;
    for r in 0..=last_row {
        if let Some(s) = read_str(rng, r, 1) {
            let s_trim = s.trim();
            if s_trim.eq_ignore_ascii_case("Neto a Pagar") {
                neto += read_num(rng, r, 8);
            } else if s_trim == "0706" {
                adelantos += read_num(rng, r, 7);
            }
        }
    }
    (neto, adelantos)
}

fn debe(
    voucher: u32,
    fecha: &str,
    tipo_cambio: f64,
    cuenta: &str,
    monto: f64,
    glosa: &str,
    con_c_costo: bool,
) -> LineaAsiento {
    LineaAsiento {
        origen: ORIGEN.into(),
        num_voucher: voucher,
        fecha: fecha.into(),
        cuenta: cuenta.into(),
        debe: Some(monto),
        moneda: MONEDA.into(),
        tipo_cambio,
        doc: DOC.into(),
        c_costo: if con_c_costo { C_COSTO.into() } else { String::new() },
        glosa: glosa.into(),
        ..Default::default()
    }
}

fn haber(
    voucher: u32,
    fecha: &str,
    tipo_cambio: f64,
    cuenta: &str,
    monto: f64,
    glosa: &str,
) -> LineaAsiento {
    LineaAsiento {
        origen: ORIGEN.into(),
        num_voucher: voucher,
        fecha: fecha.into(),
        cuenta: cuenta.into(),
        haber: Some(monto),
        moneda: MONEDA.into(),
        tipo_cambio,
        doc: DOC.into(),
        glosa: glosa.into(),
        ..Default::default()
    }
}

fn validar_partida_doble(
    lineas: &[LineaAsiento],
    voucher: u32,
    nombre: &str,
) -> Result<(), AppError> {
    let total_debe: f64 = lineas
        .iter()
        .filter(|l| l.num_voucher == voucher)
        .filter_map(|l| l.debe)
        .sum();
    let total_haber: f64 = lineas
        .iter()
        .filter(|l| l.num_voucher == voucher)
        .filter_map(|l| l.haber)
        .sum();
    if (total_debe - total_haber).abs() > 0.01 {
        return Err(AppError::Validation(format!(
            "{nombre}: partida doble no cuadra (debe={total_debe:.2}, haber={total_haber:.2})"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caso_referencia() -> AsientoPlanillaInput {
        // Output real del cliente DICIEMBRE 2025.
        AsientoPlanillaInput {
            mes: 12,
            anio: 2025,
            tipo_cambio: 3.632,
            sueldos: 9295.0,
            vacaciones: 836.55,
            adelantos: 872.52,
            ir_5ta: 176.96,
            essalud: 836.55,
            afp: 580.05,
            neto: 7665.47,
            eps: 103.79,
            cts: 419.57,
            gratificacion: 774.58,
            bonif_extraord: 61.06,
        }
    }

    #[test]
    fn caso_referencia_genera_15_lineas_4_vouchers() {
        let input = caso_referencia();
        let lineas = construir_planilla(&input).expect("ok");
        assert_eq!(lineas.len(), 15);
        // 4 vouchers distintos
        let vouchers: std::collections::BTreeSet<u32> =
            lineas.iter().map(|l| l.num_voucher).collect();
        assert_eq!(vouchers.into_iter().collect::<Vec<_>>(), vec![1, 2, 3, 4]);
        // Todas con Origen 11
        assert!(lineas.iter().all(|l| l.origen == "11"));
        // Todas con fecha 2025-12-31
        assert!(lineas.iter().all(|l| l.fecha == "2025-12-31"));
    }

    #[test]
    fn partida_doble_cuadra_por_voucher() {
        let input = caso_referencia();
        let lineas = construir_planilla(&input).expect("ok");
        for v in 1..=4 {
            let debe: f64 = lineas
                .iter()
                .filter(|l| l.num_voucher == v)
                .filter_map(|l| l.debe)
                .sum();
            let haber: f64 = lineas
                .iter()
                .filter(|l| l.num_voucher == v)
                .filter_map(|l| l.haber)
                .sum();
            assert!(
                (debe - haber).abs() < 0.01,
                "voucher {v}: debe={debe} haber={haber}"
            );
        }
    }

    #[test]
    fn solo_planilla_sin_eps_cts_grati_genera_un_voucher() {
        let input = AsientoPlanillaInput {
            mes: 1,
            anio: 2025,
            tipo_cambio: 3.5,
            sueldos: 1000.0,
            vacaciones: 0.0,
            adelantos: 0.0,
            ir_5ta: 0.0,
            essalud: 90.0,
            afp: 130.0,
            neto: 780.0,
            eps: 0.0,
            cts: 0.0,
            gratificacion: 0.0,
            bonif_extraord: 0.0,
        };
        let lineas = construir_planilla(&input).expect("ok");
        assert!(lineas.iter().all(|l| l.num_voucher == 1));
        assert_eq!(lineas.len(), 4); // sueldos + essalud + afp + neto
    }

    #[test]
    fn partida_doble_descuadrada_falla() {
        let mut input = caso_referencia();
        input.neto = 9999.99; // rompe la suma
        let err = construir_planilla(&input).unwrap_err();
        assert!(err.to_string().contains("PLANILLA"));
    }

    /// Valida contra el xls real del cliente. Solo corre con `cargo test --ignored`
    /// porque el archivo no se commitea (gitignored por data sensible).
    #[test]
    #[ignore]
    fn resumir_diciembre_2025_extrae_valores_del_input() {
        let bytes = std::fs::read("../public/input-planilla.xls").expect("input local");
        let r = resumir_planilla_xls(&bytes, 12, 2025).expect("resumir ok");
        assert!((r.sueldos - 9295.0).abs() < 0.01, "sueldos={}", r.sueldos);
        assert!((r.afp - 580.05).abs() < 0.01, "afp={}", r.afp);
        // IR 5ta: 152.20 + 24.73 = 176.93; el cliente puso 176.96 (dif 0.03 por redondeo)
        assert!(
            (r.ir_5ta - 176.93).abs() < 0.05,
            "ir_5ta={} (esperado ~176.93)",
            r.ir_5ta
        );
        assert!((r.neto - 7665.47).abs() < 0.01, "neto={}", r.neto);
        assert!((r.adelantos - 872.52).abs() < 0.01, "adelantos={}", r.adelantos);
    }

    #[test]
    fn input_completamente_vacio_falla() {
        let input = AsientoPlanillaInput {
            mes: 1,
            anio: 2025,
            tipo_cambio: 3.5,
            sueldos: 0.0,
            vacaciones: 0.0,
            adelantos: 0.0,
            ir_5ta: 0.0,
            essalud: 0.0,
            afp: 0.0,
            neto: 0.0,
            eps: 0.0,
            cts: 0.0,
            gratificacion: 0.0,
            bonif_extraord: 0.0,
        };
        assert!(construir_planilla(&input).is_err());
    }
}
