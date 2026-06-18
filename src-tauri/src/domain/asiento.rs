use serde::{Deserialize, Serialize};

/// Una línea del XLS que importa SISCONT. 42 columnas constantes — es el
/// formato compartido por TODOS los módulos (banca, planilla, detracciones,
/// depreciación, y cualquier futuro). Lo que varía entre módulos es `origen`,
/// las cuentas usadas, y el parser del input.
///
/// Los strings vacíos se traducen a celda vacía en el XLS (no a la palabra "").
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineaAsiento {
    pub origen: String,
    pub num_voucher: u32,
    pub fecha: String,
    pub cuenta: String,
    pub debe: Option<f64>,
    pub haber: Option<f64>,
    pub moneda: String,
    pub tipo_cambio: f64,
    pub doc: String,
    pub num_doc: String,
    pub fec_doc: String,
    pub fec_ven: String,
    pub cod_prov_clie: String,
    pub c_costo: String,
    pub presupuesto: String,
    pub f_efectivo: String,
    pub glosa: String,
    pub libro_cvr: String,
    pub mto_neto_1: String,
    pub mto_neto_2: String,
    pub mto_neto_3: String,
    pub mto_neto_4: String,
    pub mto_neto_5: String,
    pub mto_neto_6: String,
    pub mto_neto_7: String,
    pub mto_neto_8: String,
    pub mto_neto_9: String,
    pub mto_igv: String,
    pub ref_doc: String,
    pub ref_num_doc: String,
    pub ref_fecha: String,
    pub d_numero: String,
    pub d_fecha: String,
    pub ruc: String,
    pub r_social: String,
    pub tipo: String,
    pub tip_doc_iden: String,
    pub medio_pago: String,
    pub apellido_1: String,
    pub apellido_2: String,
    pub nombre: String,
    pub t_bien: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Origen {
    Banca,
    Planilla,
    Detracciones,
    Depreciacion,
}

impl Origen {
    pub fn code(&self) -> &'static str {
        match self {
            Origen::Banca => "07",
            Origen::Planilla => "11",
            Origen::Detracciones => "13",
            Origen::Depreciacion => "14",
        }
    }
}
