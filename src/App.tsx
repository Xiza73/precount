import { PlanCuentasModule } from "@/modules/plan-de-cuentas/PlanCuentasModule";

export function App() {
  return (
    <div className="app">
      <header className="app-header">
        <img src="/logo.png" alt="precis" className="app-logo" />
        <h1>precis</h1>
        <nav className="app-nav">
          <button type="button" data-active="true">
            Plan de cuentas
          </button>
        </nav>
      </header>
      <main className="app-main">
        <PlanCuentasModule />
      </main>
    </div>
  );
}
