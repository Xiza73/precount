use crate::{
    domain::{asiento::LineaAsiento, error::AppError},
    xls::{depreciacion::parse_depreciacion_xlsx, output::write_asientos},
};

#[tauri::command]
pub async fn parse_depreciacion(
    bytes: Vec<u8>,
    mes: u8,
    anio: i32,
    tipo_cambio: f64,
) -> Result<Vec<LineaAsiento>, AppError> {
    parse_depreciacion_xlsx(&bytes, mes, anio, tipo_cambio)
}

/// Devuelve los bytes del XLS final. El frontend abre un save dialog y los
/// escribe vía tauri-plugin-fs — el binario nunca toca disco.
#[tauri::command]
pub async fn generar_depreciacion_xls(lineas: Vec<LineaAsiento>) -> Result<Vec<u8>, AppError> {
    write_asientos(&lineas)
}
