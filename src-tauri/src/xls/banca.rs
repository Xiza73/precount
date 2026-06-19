use crate::domain::{
    asiento::LineaAsiento,
    error::AppError,
    fecha::{ultimo_dia_mes, validar_mes},
};
use regex::Regex;
use serde::{Deserialize, Serialize};

const ORIGEN: &str = "07";
const DOC_BANCO: &str = "TR";
const DOC_OTRO: &str = "00";
const MONEDA: &str = "S";
/// Cuenta corriente operativa del banco. Convención del cliente.
const CUENTA_BANCO: &str = "1041101";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BancaInput {
    pub mes: u8,
    pub anio: i32,
    pub tipo_cambio: f64,
    pub movimientos: Vec<MovimientoInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovimientoInput {
    /// YYYY-MM-DD — fecha real del movimiento (fec_doc).
    pub fecha: String,
    pub monto: f64,
    /// `true` = cargo (banco haber), `false` = abono (banco debe).
    pub es_cargo: bool,
    /// Cuenta contracuenta — la que va al otro lado del banco.
    pub cuenta: String,
    /// Glosa simplificada por el contador.
    pub glosa: String,
    /// C.Costo opcional (ej. "5001" para préstamos).
    #[serde(default)]
    pub c_costo: String,
    /// RUC del tercero (si aplica).
    #[serde(default)]
    pub ruc: String,
    /// Razón social del tercero (si aplica).
    #[serde(default)]
    pub razon_social: String,
}

/// Construye los asientos del estado de cuenta bancario (Origen 07).
/// Un voucher por movimiento (numerado 1, 2, 3, …). 2 líneas por voucher:
/// - Cargo: contracuenta debe + banco haber (doc=TR)
/// - Abono: banco debe (doc=TR) + contracuenta haber
pub fn construir_banca(input: &BancaInput) -> Result<Vec<LineaAsiento>, AppError> {
    if input.movimientos.is_empty() {
        return Err(AppError::Validation("no hay movimientos cargados".into()));
    }
    let fecha_asiento = ultimo_dia_mes(input.anio, input.mes)?
        .format("%Y-%m-%d")
        .to_string();

    let mut lineas = Vec::with_capacity(input.movimientos.len() * 2);
    for (i, m) in input.movimientos.iter().enumerate() {
        if m.monto <= 0.0 {
            return Err(AppError::Validation(format!(
                "movimiento #{} con monto <= 0",
                i + 1
            )));
        }
        if m.cuenta.is_empty() {
            return Err(AppError::Validation(format!(
                "movimiento #{} sin cuenta",
                i + 1
            )));
        }
        if m.fecha.is_empty() {
            return Err(AppError::Validation(format!(
                "movimiento #{} sin fecha",
                i + 1
            )));
        }
        let voucher = (i + 1) as u32;

        let (cuenta_debe, doc_debe, cuenta_haber, doc_haber) = if m.es_cargo {
            (m.cuenta.as_str(), DOC_OTRO, CUENTA_BANCO, DOC_BANCO)
        } else {
            (CUENTA_BANCO, DOC_BANCO, m.cuenta.as_str(), DOC_OTRO)
        };

        lineas.push(LineaAsiento {
            origen: ORIGEN.into(),
            num_voucher: voucher,
            fecha: fecha_asiento.clone(),
            cuenta: cuenta_debe.into(),
            debe: Some(m.monto),
            moneda: MONEDA.into(),
            tipo_cambio: input.tipo_cambio,
            doc: doc_debe.into(),
            fec_doc: m.fecha.clone(),
            fec_ven: m.fecha.clone(),
            c_costo: m.c_costo.clone(),
            glosa: m.glosa.clone(),
            ruc: m.ruc.clone(),
            r_social: m.razon_social.clone(),
            ..Default::default()
        });
        lineas.push(LineaAsiento {
            origen: ORIGEN.into(),
            num_voucher: voucher,
            fecha: fecha_asiento.clone(),
            cuenta: cuenta_haber.into(),
            haber: Some(m.monto),
            moneda: MONEDA.into(),
            tipo_cambio: input.tipo_cambio,
            doc: doc_haber.into(),
            fec_doc: m.fecha.clone(),
            fec_ven: m.fecha.clone(),
            glosa: m.glosa.clone(),
            ruc: m.ruc.clone(),
            r_social: m.razon_social.clone(),
            ..Default::default()
        });
    }
    Ok(lineas)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MovimientoExtraido {
    /// YYYY-MM-DD.
    pub fecha: String,
    pub descripcion: String,
    pub monto: f64,
    pub es_cargo: bool,
}

/// Extrae movimientos del texto del estado de cuenta BCP para el mes/año dados.
/// Formato esperado por línea: `DD-MM DESCRIPCION ... MONTO[-] [SALDO]`.
/// El `-` final al monto marca un cargo. Sin `-` es abono.
pub fn extraer_movimientos_bcp(
    texto: &str,
    mes: u8,
    anio: i32,
) -> Result<Vec<MovimientoExtraido>, AppError> {
    validar_mes(mes)?;
    // Captura: día, mes, descripción (greedy hasta el monto), monto (puede empezar
    // con punto: ".10"), sufijo "-" opcional. Saldo siguiente (opcional) se ignora.
    let re = Regex::new(
        r"^(\d{2})-(\d{2})\s+(.+?)\s+((?:\d{1,3}(?:,\d{3})*)?\.\d{2})(-)?(?:\s|$)",
    )
    .map_err(|e| AppError::Internal(format!("regex inválida: {e}")))?;

    let mut movs = Vec::new();
    for linea in texto.lines() {
        let l = linea.trim();
        let Some(cap) = re.captures(l) else { continue };
        let dia: u8 = cap[1].parse().unwrap_or(0);
        let mes_doc: u8 = cap[2].parse().unwrap_or(0);
        if mes_doc != mes || dia == 0 {
            continue;
        }
        let descripcion = cap[3].trim().to_string();
        let monto: f64 = cap[4]
            .replace(',', "")
            .parse()
            .map_err(|_| AppError::Validation(format!("monto invalido: {}", &cap[4])))?;
        let es_cargo = cap.get(5).is_some();
        movs.push(MovimientoExtraido {
            fecha: format!("{anio:04}-{mes:02}-{dia:02}"),
            descripcion,
            monto,
            es_cargo,
        });
    }
    Ok(movs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mov_abono(fecha: &str, monto: f64, cuenta: &str, glosa: &str) -> MovimientoInput {
        MovimientoInput {
            fecha: fecha.into(),
            monto,
            es_cargo: false,
            cuenta: cuenta.into(),
            glosa: glosa.into(),
            c_costo: String::new(),
            ruc: String::new(),
            razon_social: String::new(),
        }
    }
    fn mov_cargo(fecha: &str, monto: f64, cuenta: &str, glosa: &str) -> MovimientoInput {
        MovimientoInput {
            fecha: fecha.into(),
            monto,
            es_cargo: true,
            cuenta: cuenta.into(),
            glosa: glosa.into(),
            c_costo: String::new(),
            ruc: String::new(),
            razon_social: String::new(),
        }
    }

    #[test]
    fn abono_genera_banco_debe_y_contracuenta_haber() {
        let input = BancaInput {
            mes: 1,
            anio: 2025,
            tipo_cambio: 3.72,
            movimientos: vec![mov_abono("2025-01-02", 1558.0, "10101", "CANCELACION FT")],
        };
        let lineas = construir_banca(&input).expect("ok");
        assert_eq!(lineas.len(), 2);
        // Debe: banco
        assert_eq!(lineas[0].cuenta, CUENTA_BANCO);
        assert_eq!(lineas[0].debe, Some(1558.0));
        assert_eq!(lineas[0].doc, "TR");
        // Haber: contracuenta
        assert_eq!(lineas[1].cuenta, "10101");
        assert_eq!(lineas[1].haber, Some(1558.0));
        assert_eq!(lineas[1].doc, "00");
    }

    #[test]
    fn cargo_genera_contracuenta_debe_y_banco_haber() {
        let input = BancaInput {
            mes: 1,
            anio: 2025,
            tipo_cambio: 3.72,
            movimientos: vec![mov_cargo("2025-01-06", 198.9, "41724", "AFP INTEGRA DICIEMBRE")],
        };
        let lineas = construir_banca(&input).expect("ok");
        assert_eq!(lineas.len(), 2);
        assert_eq!(lineas[0].cuenta, "41724");
        assert_eq!(lineas[0].debe, Some(198.9));
        assert_eq!(lineas[0].doc, "00");
        assert_eq!(lineas[1].cuenta, CUENTA_BANCO);
        assert_eq!(lineas[1].haber, Some(198.9));
        assert_eq!(lineas[1].doc, "TR");
    }

    #[test]
    fn cada_movimiento_es_un_voucher_distinto() {
        let input = BancaInput {
            mes: 1,
            anio: 2025,
            tipo_cambio: 3.72,
            movimientos: vec![
                mov_abono("2025-01-02", 1558.0, "10101", "1"),
                mov_cargo("2025-01-02", 200.0, "10101", "2"),
                mov_cargo("2025-01-06", 9904.0, "10101", "DETR"),
            ],
        };
        let lineas = construir_banca(&input).expect("ok");
        assert_eq!(lineas.len(), 6); // 3 mov × 2 líneas
        assert_eq!(lineas[0].num_voucher, 1);
        assert_eq!(lineas[1].num_voucher, 1);
        assert_eq!(lineas[2].num_voucher, 2);
        assert_eq!(lineas[3].num_voucher, 2);
        assert_eq!(lineas[4].num_voucher, 3);
        assert_eq!(lineas[5].num_voucher, 3);
    }

    #[test]
    fn fecha_del_asiento_es_ultimo_dia_del_mes_y_fec_doc_es_propia() {
        let input = BancaInput {
            mes: 1,
            anio: 2025,
            tipo_cambio: 3.72,
            movimientos: vec![mov_abono("2025-01-02", 1558.0, "10101", "x")],
        };
        let lineas = construir_banca(&input).expect("ok");
        assert!(lineas.iter().all(|l| l.fecha == "2025-01-31"));
        assert!(lineas.iter().all(|l| l.fec_doc == "2025-01-02"));
    }

    #[test]
    fn movimientos_vacios_falla() {
        let input = BancaInput {
            mes: 1,
            anio: 2025,
            tipo_cambio: 3.72,
            movimientos: vec![],
        };
        assert!(construir_banca(&input).is_err());
    }

    const TEXTO_MOCK_BCP_ENE_2025: &str = "\
02-01 DE LGG PISCO PERU SA TLC 111-065 018042 17:40 OBEA67 2401 1,558.00 74,201.21
02-01 TRAN.CTAS.TERC.HK BPI 111-023 085690 17:07 HBK984 4701 200.00- 74,001.21
02-01 TRAN.CTAS.TERC.HK BPI 111-023 075814 17:06 HBK968 4701 1,600.00- 72,401.21
02-01 IMPUESTO ITF INT - 0909 .10- 72,401.11
03-01 CHEQUE 00000194 INT 191-000 800292 3901 47,599.37- 24,801.74
06-01 PAGOS AFP INTEGRA BPI 111-041 008623 16:01 AFNA01 4753 198.90- 24,590.49
06-01 DETR.MASIVA 7163289333 BPI 111-034 005232 12:03 SNTPEA 4709 9,904.00- 14,289.89
";

    #[test]
    fn extraer_movimientos_detecta_abonos_y_cargos() {
        let movs = extraer_movimientos_bcp(TEXTO_MOCK_BCP_ENE_2025, 1, 2025).expect("ok");
        assert_eq!(movs.len(), 7);
        assert!(!movs[0].es_cargo); // abono
        assert_eq!(movs[0].monto, 1558.0);
        assert_eq!(movs[0].fecha, "2025-01-02");
        assert!(movs[1].es_cargo); // cargo
        assert_eq!(movs[1].monto, 200.0);
        // monto con leading dot
        assert_eq!(movs[3].monto, 0.10);
        assert!(movs[3].es_cargo);
        // monto grande con coma
        assert_eq!(movs[4].monto, 47599.37);
    }

    #[test]
    fn extraer_movimientos_ignora_otros_meses() {
        let movs = extraer_movimientos_bcp(TEXTO_MOCK_BCP_ENE_2025, 2, 2025).expect("ok");
        assert!(movs.is_empty());
    }
}
