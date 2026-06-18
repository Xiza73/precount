import { DepreciacionModule } from "@/modules/depreciacion/DepreciacionModule";
import { DetraccionesModule } from "@/modules/detracciones/DetraccionesModule";
import { PlanCuentasModule } from "@/modules/plan-de-cuentas/PlanCuentasModule";
import { PlanillaModule } from "@/modules/planilla/PlanillaModule";
import { useState } from "react";

type ModuleKey = "plan-de-cuentas" | "depreciacion" | "planilla" | "detracciones";

const TABS: { key: ModuleKey; label: string }[] = [
  { key: "plan-de-cuentas", label: "Plan de cuentas" },
  { key: "depreciacion", label: "Depreciación" },
  { key: "planilla", label: "Planilla" },
  { key: "detracciones", label: "Detracciones" },
];

export function App() {
  const [active, setActive] = useState<ModuleKey>("depreciacion");

  return (
    <div className="app">
      <header className="app-header">
        <img src="/logo.png" alt="precis" className="app-logo" />
        <h1>precis</h1>
        <nav className="app-nav">
          {TABS.map((t) => (
            <button
              key={t.key}
              type="button"
              data-active={active === t.key}
              onClick={() => setActive(t.key)}
            >
              {t.label}
            </button>
          ))}
        </nav>
      </header>
      <main className="app-main">
        {active === "plan-de-cuentas" && <PlanCuentasModule />}
        {active === "depreciacion" && <DepreciacionModule />}
        {active === "planilla" && <PlanillaModule />}
        {active === "detracciones" && <DetraccionesModule />}
      </main>
    </div>
  );
}
