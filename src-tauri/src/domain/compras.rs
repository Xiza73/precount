use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TipoComprobante {
    Factura,
    Boleta,
    NotaCredito,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComprobanteCompra {
    pub fecha_emision: String,
    pub tipo_comprobante: TipoComprobante,
    pub serie: String,
    pub numero: String,
    pub ruc_proveedor: String,
    pub razon_social: String,
    pub base_imponible: f64,
    pub igv: f64,
    pub total: f64,
    pub cuenta_contable: String,
}
