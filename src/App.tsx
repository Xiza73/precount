import { DepreciacionModule } from "@/modules/depreciacion/DepreciacionModule";
import { PlanCuentasModule } from "@/modules/plan-de-cuentas/PlanCuentasModule";
import { useState } from "react";

type ModuleKey = "plan-de-cuentas" | "depreciacion";

export function App() {
  const [active, setActive] = useState<ModuleKey>("depreciacion");

  return (
    <div className="app">
      <header className="app-header">
        <img src="/logo.png" alt="precis" className="app-logo" />
        <h1>precis</h1>
        <nav className="app-nav">
          <button
            type="button"
            data-active={active === "plan-de-cuentas"}
            onClick={() => setActive("plan-de-cuentas")}
          >
            Plan de cuentas
          </button>
          <button
            type="button"
            data-active={active === "depreciacion"}
            onClick={() => setActive("depreciacion")}
          >
            Depreciación
          </button>
        </nav>
      </header>
      <main className="app-main">
        {active === "plan-de-cuentas" ? <PlanCuentasModule /> : <DepreciacionModule />}
      </main>
    </div>
  );
}
