use crate::domain::{
    asiento::LineaAsiento,
    error::AppError,
    fecha::{nombre_mes, ultimo_dia_mes},
};
use serde::Deserialize;

const ORIGEN: &str = "13";
const DOC_ABONO: &str = "01";
const DOC_DEBE: &str = "00";
const MONEDA: &str = "S";
const CUENTA_BN: &str = "104201";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetraccionesInput {
    pub mes: u8,
    pub anio: i32,
    pub tipo_cambio: f64,
    pub abonos: Vec<AbonoDetraccion>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbonoDetraccion {
    pub monto: f64,
    /// `12121` (MN) o `12122` (ME), según el cliente.
    pub cuenta: String,
    pub num_doc: String,
    pub ruc: String,
    pub razon_social: String,
    /// SISCONT: tipo de operación. Default observado en el cliente: "6".
    #[serde(default = "tipo_default")]
    pub tipo: String,
}

fn tipo_default() -> String {
    "6".into()
}

/// Construye el asiento mensual de detracciones (Voucher 1, Origen 13):
/// - 1 línea debe en 104201 (cta detracciones BN) por la suma de abonos
/// - N líneas haber, una por abono, con doc=01, num_doc, RUC, razón social
///
/// La fecha es siempre el último día del mes. Glosa: "DETRACCIONES {MES} {ANIO}".
pub fn construir_detracciones(
    input: &DetraccionesInput,
) -> Result<Vec<LineaAsiento>, AppError> {
    if input.abonos.is_empty() {
        return Err(AppError::Validation("no hay abonos cargados".into()));
    }
    let fecha = ultimo_dia_mes(input.anio, input.mes)?
        .format("%Y-%m-%d")
        .to_string();
    let glosa = format!("DETRACCIONES {} {}", nombre_mes(input.mes)?, input.anio);

    // Validación por abono
    for (i, a) in input.abonos.iter().enumerate() {
        if a.monto <= 0.0 {
            return Err(AppError::Validation(format!(
                "abono #{} con monto <= 0",
                i + 1
            )));
        }
        if a.cuenta.is_empty() {
            return Err(AppError::Validation(format!(
                "abono #{} sin cuenta",
                i + 1
            )));
        }
    }

    let total: f64 = input.abonos.iter().map(|a| a.monto).sum();

    let mut lineas = Vec::with_capacity(1 + input.abonos.len());
    // Línea debe
    lineas.push(LineaAsiento {
        origen: ORIGEN.into(),
        num_voucher: 1,
        fecha: fecha.clone(),
        cuenta: CUENTA_BN.into(),
        debe: Some(total),
        moneda: MONEDA.into(),
        tipo_cambio: input.tipo_cambio,
        doc: DOC_DEBE.into(),
        glosa: glosa.clone(),
        ..Default::default()
    });
    // Líneas haber (una por abono)
    for a in &input.abonos {
        lineas.push(LineaAsiento {
            origen: ORIGEN.into(),
            num_voucher: 1,
            fecha: fecha.clone(),
            cuenta: a.cuenta.clone(),
            haber: Some(a.monto),
            moneda: MONEDA.into(),
            tipo_cambio: input.tipo_cambio,
            doc: DOC_ABONO.into(),
            num_doc: a.num_doc.clone(),
            ruc: a.ruc.clone(),
            r_social: a.razon_social.clone(),
            tipo: a.tipo.clone(),
            glosa: glosa.clone(),
            ..Default::default()
        });
    }

    Ok(lineas)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn abono(monto: f64, cuenta: &str, num_doc: &str, ruc: &str, razon: &str) -> AbonoDetraccion {
        AbonoDetraccion {
            monto,
            cuenta: cuenta.into(),
            num_doc: num_doc.into(),
            ruc: ruc.into(),
            razon_social: razon.into(),
            tipo: "6".into(),
        }
    }

    /// Caso de referencia: 6 abonos de noviembre 2025 del output del cliente.
    fn caso_referencia() -> DetraccionesInput {
        DetraccionesInput {
            mes: 11,
            anio: 2025,
            tipo_cambio: 3.368,
            abonos: vec![
                abono(1526.0, "12122", "E001-2185", "20262478964", "METSO PERU S.A."),
                abono(623.0, "12121", "E001-2229", "20106897914", "ENTEL PERU S.A."),
                abono(896.0, "12121", "E001-2226", "20106897914", "ENTEL PERU S.A."),
                abono(908.0, "12122", "E001-2048", "20608893289", "A3 ELECTRIC MOBILITY PERU S.A.C."),
                abono(1399.0, "12121", "E001-2231", "20610799869", "VIRU S.A."),
                abono(4036.0, "12121", "E001-2228", "20524269440", "MACROSOURCE PERÚ"),
            ],
        }
    }

    #[test]
    fn caso_referencia_genera_7_lineas() {
        let lineas = construir_detracciones(&caso_referencia()).expect("ok");
        assert_eq!(lineas.len(), 7); // 1 debe + 6 haber
        assert_eq!(lineas[0].cuenta, "104201");
        assert_eq!(lineas[0].debe, Some(9388.0));
        assert!(lineas[1..].iter().all(|l| l.haber.is_some()));
        assert!(lineas.iter().all(|l| l.origen == "13" && l.num_voucher == 1));
    }

    #[test]
    fn partida_doble_cuadra() {
        let lineas = construir_detracciones(&caso_referencia()).expect("ok");
        let debe: f64 = lineas.iter().filter_map(|l| l.debe).sum();
        let haber: f64 = lineas.iter().filter_map(|l| l.haber).sum();
        assert!((debe - haber).abs() < 0.01);
    }

    #[test]
    fn fecha_es_ultimo_dia_del_mes() {
        let lineas = construir_detracciones(&caso_referencia()).expect("ok");
        assert!(lineas.iter().all(|l| l.fecha == "2025-11-30"));
    }

    #[test]
    fn glosa_consistente_en_todas() {
        let lineas = construir_detracciones(&caso_referencia()).expect("ok");
        assert!(
            lineas
                .iter()
                .all(|l| l.glosa == "DETRACCIONES NOVIEMBRE 2025")
        );
    }

    #[test]
    fn abonos_vacios_falla() {
        let mut input = caso_referencia();
        input.abonos.clear();
        assert!(construir_detracciones(&input).is_err());
    }

    #[test]
    fn abono_con_monto_invalido_falla() {
        let mut input = caso_referencia();
        input.abonos[0].monto = 0.0;
        assert!(construir_detracciones(&input).is_err());
    }

    #[test]
    fn abono_sin_cuenta_falla() {
        let mut input = caso_referencia();
        input.abonos[0].cuenta = String::new();
        assert!(construir_detracciones(&input).is_err());
    }
}
