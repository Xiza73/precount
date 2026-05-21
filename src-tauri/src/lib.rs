mod commands;
mod db;
mod domain;
mod xls;

pub use domain::error::AppError;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::plan_de_cuentas::list_cuentas,
            commands::plan_de_cuentas::upsert_cuenta,
            commands::plan_de_cuentas::delete_cuenta,
            commands::compras::parse_input_xls,
            commands::compras::generate_output_xls,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
