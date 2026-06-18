use crate::{
    domain::error::AppError,
    xls::{
        output::write_asientos,
        planilla::{construir_planilla, resumir_planilla_xls, AsientoPlanillaInput, ResumenPlanilla},
    },
};

#[tauri::command]
pub async fn generar_planilla_xls(input: AsientoPlanillaInput) -> Result<Vec<u8>, AppError> {
    let lineas = construir_planilla(&input)?;
    write_asientos(&lineas)
}

#[tauri::command]
pub async fn resumir_planilla(
    bytes: Vec<u8>,
    mes: u8,
    anio: i32,
) -> Result<ResumenPlanilla, AppError> {
    resumir_planilla_xls(&bytes, mes, anio)
}
