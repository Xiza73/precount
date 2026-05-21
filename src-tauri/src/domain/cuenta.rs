use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuentaContable {
    pub codigo: String,
    pub descripcion: String,
    pub nivel: u8,
    pub padre: Option<String>,
    pub customizada: bool,
}
