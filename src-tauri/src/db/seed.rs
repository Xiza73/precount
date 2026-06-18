/// Seed del plan de cuentas genérico — derivado de las cuentas que aparecen
/// en los outputs de referencia (banca, detracciones, depreciación, planilla).
/// Sirve para arrancar sin trabarse hasta que el cliente entregue su plan real.
///
/// Tupla: (codigo, descripcion, nivel, customizada)
pub const SEED_PLAN_CUENTAS: &[(&str, &str, u8, bool)] = &[
    // Caja y bancos
    ("10101", "Caja", 5, false),
    ("1041101", "Cta corriente operativa", 7, true),
    ("1041201", "Cta corriente alterna", 7, true),
    ("104201", "Cta detracciones Banco de la Nación", 6, false),
    ("104202", "Cta detracciones BN alterna", 6, false),
    // Cuentas por cobrar
    ("12121", "Facturas por cobrar - MN", 5, false),
    ("12122", "Facturas por cobrar - ME", 5, false),
    ("14124", "Adelantos al personal", 5, false),
    ("167101", "Ctas por cobrar diversas - depreciación", 6, true),
    ("167104", "Ctas por cobrar diversas - renta", 6, true),
    ("1689104", "Otros activos - anticipos", 7, true),
    // Inmuebles, maq, equipo / depreciación
    ("39613", "Depreciación acumulada - inmuebles", 5, false),
    ("68613", "Gasto por depreciación", 5, false),
    // Tributos
    ("40173", "IR 5ta categoría por pagar", 5, false),
    ("40311", "ESSALUD por pagar", 5, false),
    ("401841", "Otros tributos por pagar - SUNAT", 6, true),
    ("409115", "Fraccionamiento SUNAT - julio 2024", 6, true),
    ("409116", "Fraccionamiento SUNAT - agosto 2024", 6, true),
    // Remuneraciones (pasivos)
    ("41111", "Sueldos por pagar", 5, false),
    ("41141", "Gratificaciones por pagar", 5, false),
    ("41511", "CTS por pagar", 5, false),
    ("41724", "AFP por pagar", 5, false),
    ("41901", "Otros pasivos - personal", 5, false),
    // Préstamos
    ("4511105", "Préstamos bancarios", 7, true),
    // Gastos - personal
    ("62111", "Sueldos y salarios", 5, false),
    ("62141", "Gratificaciones", 5, false),
    ("62711", "Vacaciones - gastos", 5, true),
    ("62901", "Otros gastos de personal", 5, true),
    ("62911", "Otros gastos remunerativos", 5, true),
    // Gastos - servicios y tributos
    ("63296", "Servicios diversos - publicidad", 5, true),
    ("645201", "Tributos varios", 6, true),
    ("645901", "Otros tributos", 6, true),
    ("67311", "Intereses de préstamos", 5, false),
];
