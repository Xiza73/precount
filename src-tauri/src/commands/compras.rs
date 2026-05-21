use crate::domain::{compras::ComprobanteCompra, error::AppError};

#[tauri::command]
pub async fn parse_input_xls(_bytes: Vec<u8>) -> Result<Vec<ComprobanteCompra>, AppError> {
    // TODO: parsear xls/csv con calamine, validar filas, devolver comprobantes.
    Ok(Vec::new())
}

#[tauri::command]
pub async fn generate_output_xls(_comprobantes: Vec<ComprobanteCompra>) -> Result<Vec<u8>, AppError> {
    // TODO: generar xls con rust_xlsxwriter en el layout de SISCONT y devolver bytes.
    Ok(Vec::new())
}
