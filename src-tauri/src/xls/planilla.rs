use crate::domain::{
    asiento::LineaAsiento,
    error::AppError,
    fecha::{nombre_mes, ultimo_dia_mes},
};
use serde::Deserialize;

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
