use crate::{
    domain::error::AppError,
    xls::{
        detracciones::{construir_detracciones, DetraccionesInput},
        output::write_asientos,
    },
};

#[tauri::command]
pub async fn generar_detracciones_xls(input: DetraccionesInput) -> Result<Vec<u8>, AppError> {
    let lineas = construir_detracciones(&input)?;
    write_asientos(&lineas)
}
