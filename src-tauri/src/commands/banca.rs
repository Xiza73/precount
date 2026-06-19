use crate::{
    domain::error::AppError,
    xls::{
        banca::{construir_banca, extraer_movimientos_bcp, BancaInput, MovimientoExtraido},
        output::write_asientos,
    },
};

#[tauri::command]
pub async fn generar_banca_xls(input: BancaInput) -> Result<Vec<u8>, AppError> {
    let lineas = construir_banca(&input)?;
    write_asientos(&lineas)
}

#[tauri::command]
pub async fn extraer_movimientos_banca(
    texto: String,
    mes: u8,
    anio: i32,
) -> Result<Vec<MovimientoExtraido>, AppError> {
    extraer_movimientos_bcp(&texto, mes, anio)
}
