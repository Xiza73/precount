use crate::domain::{cuenta::CuentaContable, error::AppError};

#[tauri::command]
pub async fn list_cuentas() -> Result<Vec<CuentaContable>, AppError> {
    // TODO: implementar lectura desde SQLite via tauri-plugin-sql.
    Ok(Vec::new())
}

#[tauri::command]
pub async fn upsert_cuenta(_cuenta: CuentaContable) -> Result<(), AppError> {
    // TODO: implementar INSERT OR REPLACE en SQLite.
    Ok(())
}

#[tauri::command]
pub async fn delete_cuenta(_codigo: String) -> Result<(), AppError> {
    // TODO: implementar DELETE en SQLite.
    Ok(())
}
