use crate::{
    domain::{asiento::LineaAsiento, error::AppError},
    xls::{depreciacion::parse_depreciacion_xlsx, output::write_asientos},
};
use base64::{Engine, engine::general_purpose::STANDARD};

#[tauri::command]
pub async fn parse_depreciacion(
    bytes: Vec<u8>,
    mes: u8,
    anio: i32,
    tipo_cambio: f64,
) -> Result<Vec<LineaAsiento>, AppError> {
    parse_depreciacion_xlsx(&bytes, mes, anio, tipo_cambio)
}

/// Devuelve el XLS final en base64 listo para descargar desde el frontend.
#[tauri::command]
pub async fn generar_depreciacion_xls(lineas: Vec<LineaAsiento>) -> Result<String, AppError> {
    let bytes = write_asientos(&lineas)?;
    Ok(STANDARD.encode(bytes))
}
