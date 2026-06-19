use crate::domain::{
    asiento::LineaAsiento,
    error::AppError,
    fecha::{nombre_mes, ultimo_dia_mes, validar_mes},
};
use regex::Regex;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbonoExtraido {
    pub monto: f64,
    /// Día del mes (1-31).
    pub dia: u8,
}

/// Extrae los abonos (`VA 1721`) del **texto** del estado de cuenta de Banco
/// de la Nación. El user copia el texto del PDF (selectable) y lo pega en la
/// UI; este parser lo procesa.
///
/// Filtra estrictamente: solo retiene líneas con `VA 1721` cuya fecha caiga
/// dentro del mes pedido. RUC, razón social y num_doc no están en el estado
/// de cuenta — los carga el contador en la tabla.
pub fn extraer_abonos_de_texto(
    text: &str,
    mes: u8,
    anio: i32,
) -> Result<Vec<AbonoExtraido>, AppError> {
    validar_mes(mes)?;
    // Patrón observado: línea con "VA 1721" + monto (formato 1,526.00) + fecha DD/MM/AAAA.
    // Los cargos NOT 1612 NO matchean este regex (descartados por filtro VA 1721).
    let re_abono = Regex::new(
        r"VA\s+1721[^\n]*?(\d{1,3}(?:,\d{3})*\.\d{2})[^\n]*?(\d{2})/(\d{2})/(\d{4})",
    )
    .map_err(|e| AppError::Internal(format!("regex inválida: {e}")))?;

    let mut abonos = Vec::new();
    for cap in re_abono.captures_iter(text) {
        let monto: f64 = cap[1]
            .replace(',', "")
            .parse()
            .map_err(|e| AppError::Validation(format!("monto invalido `{}`: {e}", &cap[1])))?;
        let dia: u8 = cap[2].parse().unwrap_or(0);
        let mes_doc: u8 = cap[3].parse().unwrap_or(0);
        let anio_doc: i32 = cap[4].parse().unwrap_or(0);
        if mes_doc == mes && anio_doc == anio && dia > 0 {
            abonos.push(AbonoExtraido { monto, dia });
        }
    }
    Ok(abonos)
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

    /// Mock con el formato literal del estado de cuenta BdN del cliente.
    /// Sintético para que el test pase en CI sin necesitar el PDF del cliente.
    const TEXTO_MOCK_NOV_2025: &str = "\
SALDO ANTERIOR | | 31/10/2025 | 12,377.95 |
VA 1721 | | 1,526.00 | 13,903.95 | 10/11/2025
VA 1721 | | 896.00 | | 11/11/2025
VA 1721 | | 623.00 | 15,422.95 | 11/11/2025
VA 1721 | | 908.00 | 16,330.95 | 17/11/2025
NOT 1612 | 910.00 | | | 21/11/2025
NOT 1612 | 1,053.00 | | | 21/11/2025
NOT 1612 | 693.00 | | 13,674.95 | 21/11/2025
NOT 1612 | 1,268.00 | | 12,406.95 | 24/11/2025
VA 1721 | | 4,036.00 | | 28/11/2025
VA 1721 | | 1,399.00 | 17,841.95 | 28/11/2025
";

    #[test]
    fn extraer_abonos_filtra_solo_VA_1721_del_mes() {
        let r = extraer_abonos_de_texto(TEXTO_MOCK_NOV_2025, 11, 2025).expect("ok");
        assert_eq!(r.len(), 6, "esperados 6 abonos VA 1721, vi {r:?}");
        let total: f64 = r.iter().map(|a| a.monto).sum();
        assert!((total - 9388.0).abs() < 0.01, "total={total}");
        let dias: Vec<u8> = r.iter().map(|a| a.dia).collect();
        assert_eq!(dias, vec![10, 11, 11, 17, 28, 28]);
        let montos: Vec<f64> = r.iter().map(|a| a.monto).collect();
        assert_eq!(montos, vec![1526.0, 896.0, 623.0, 908.0, 4036.0, 1399.0]);
    }

    #[test]
    fn extraer_abonos_ignora_otros_meses() {
        let r = extraer_abonos_de_texto(TEXTO_MOCK_NOV_2025, 12, 2025).expect("ok");
        assert!(r.is_empty(), "no debería haber abonos de diciembre, vi {r:?}");
    }

    #[test]
    fn extraer_abonos_texto_vacio() {
        let r = extraer_abonos_de_texto("", 11, 2025).expect("ok");
        assert!(r.is_empty());
    }
}
