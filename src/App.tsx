import { ComprasModule } from "@/modules/compras/ComprasModule";
import { PlanCuentasModule } from "@/modules/plan-de-cuentas/PlanCuentasModule";
import { useState } from "react";

type ModuleKey = "compras" | "plan-de-cuentas";

export function App() {
  const [active, setActive] = useState<ModuleKey>("plan-de-cuentas");

  return (
    <div className="app">
      <header className="app-header">
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
            data-active={active === "compras"}
            onClick={() => setActive("compras")}
          >
            Compras
          </button>
        </nav>
      </header>
      <main className="app-main">
        {active === "plan-de-cuentas" ? <PlanCuentasModule /> : <ComprasModule />}
      </main>
    </div>
  );
}
