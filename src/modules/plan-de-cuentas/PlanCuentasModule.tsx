export function PlanCuentasModule() {
  return (
    <section aria-labelledby="plan-de-cuentas-title">
      <h2 id="plan-de-cuentas-title">Plan de cuentas</h2>
      <p>
        CRUD del plan contable de la empresa. Pendiente: form individual y bulk import desde
        xls/csv. Persistencia en SQLite local (<code>appDataDir</code>) via{" "}
        <code>tauri-plugin-sql</code>.
      </p>
    </section>
  );
}
