// Migraciones del plan de cuentas en SQLite (via tauri-plugin-sql).
//
// El plan-de-cuentas vive como SQLite local en `appDataDir`. Las migrations
// se aplicarán con el patrón estándar de tauri-plugin-sql (registradas en lib.rs
// cuando esté implementado el módulo).

pub const PLAN_CUENTAS_V1: &str = r#"
CREATE TABLE IF NOT EXISTS plan_cuentas (
    codigo        TEXT PRIMARY KEY NOT NULL,
    descripcion   TEXT NOT NULL,
    nivel         INTEGER NOT NULL,
    padre         TEXT REFERENCES plan_cuentas(codigo) ON DELETE SET NULL,
    customizada   INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_plan_cuentas_padre ON plan_cuentas(padre);
CREATE INDEX IF NOT EXISTS idx_plan_cuentas_nivel ON plan_cuentas(nivel);
"#;
