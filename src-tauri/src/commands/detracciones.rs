use crate::{
    domain::error::AppError,
    xls::{
        detracciones::{
            construir_detracciones, extraer_abonos_de_texto, AbonoExtraido, DetraccionesInput,
        },
        output::write_asientos,
    },
};

#[tauri::command]
pub async fn generar_detracciones_xls(input: DetraccionesInput) -> Result<Vec<u8>, AppError> {
    let lineas = construir_detracciones(&input)?;
    write_asientos(&lineas)
}

#[tauri::command]
pub async fn extraer_abonos_detracciones(
    texto: String,
    mes: u8,
    anio: i32,
) -> Result<Vec<AbonoExtraido>, AppError> {
    extraer_abonos_de_texto(&texto, mes, anio)
}
