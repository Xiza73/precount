mod commands;
mod db;
mod domain;
mod xls;

pub use domain::error::AppError;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::plan_de_cuentas::list_cuentas,
            commands::plan_de_cuentas::upsert_cuenta,
            commands::plan_de_cuentas::delete_cuenta,
            commands::depreciacion::parse_depreciacion,
            commands::depreciacion::generar_depreciacion_xls,
            commands::planilla::generar_planilla_xls,
            commands::planilla::resumir_planilla,
            commands::detracciones::generar_detracciones_xls,
            commands::detracciones::extraer_abonos_detracciones,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
