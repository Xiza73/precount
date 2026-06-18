use crate::{
    domain::error::AppError,
    xls::{
        output::write_asientos,
        planilla::{construir_planilla, AsientoPlanillaInput},
    },
};

#[tauri::command]
pub async fn generar_planilla_xls(input: AsientoPlanillaInput) -> Result<Vec<u8>, AppError> {
    let lineas = construir_planilla(&input)?;
    write_asientos(&lineas)
}
