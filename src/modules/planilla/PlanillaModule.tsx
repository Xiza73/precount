import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import { useState } from "react";

type Form = {
  mes: number;
  anio: number;
  tipoCambio: number;
  // Voucher 1 — PLANILLA
  sueldos: string;
  vacaciones: string;
  adelantos: string;
  ir5ta: string;
  essalud: string;
  afp: string;
  neto: string;
  // Voucher 2 — EPS
  eps: string;
  // Voucher 3 — CTS
  cts: string;
  // Voucher 4 — GRATIFICACIÓN
  gratificacion: string;
  bonifExtraord: string;
};

const INITIAL: Form = {
  mes: 12,
  anio: 2025,
  tipoCambio: 3.632,
  sueldos: "",
  vacaciones: "",
  adelantos: "",
  ir5ta: "",
  essalud: "",
  afp: "",
  neto: "",
  eps: "",
  cts: "",
  gratificacion: "",
  bonifExtraord: "",
};

function n(s: string): number {
  const v = Number.parseFloat(s);
  return Number.isFinite(v) ? v : 0;
}

export function PlanillaModule() {
  const [f, setF] = useState<Form>(INITIAL);
  const [error, setError] = useState<string | null>(null);
  const [info, setInfo] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const set = <K extends keyof Form>(k: K, v: Form[K]) => setF((prev) => ({ ...prev, [k]: v }));

  async function onGenerar() {
    setBusy(true);
    setError(null);
    setInfo(null);
    try {
      const defaultName = `planilla-${f.anio}-${String(f.mes).padStart(2, "0")}.xlsx`;
      const path = await save({
        defaultPath: defaultName,
        filters: [{ name: "Excel", extensions: ["xlsx"] }],
      });
      if (!path) {
        setInfo("Cancelado.");
        return;
      }
      const out = await invoke<number[]>("generar_planilla_xls", {
        input: {
          mes: f.mes,
          anio: f.anio,
          tipoCambio: f.tipoCambio,
          sueldos: n(f.sueldos),
          vacaciones: n(f.vacaciones),
          adelantos: n(f.adelantos),
          ir5ta: n(f.ir5ta),
          essalud: n(f.essalud),
          afp: n(f.afp),
          neto: n(f.neto),
          eps: n(f.eps),
          cts: n(f.cts),
          gratificacion: n(f.gratificacion),
          bonifExtraord: n(f.bonifExtraord),
        },
      });
      await writeFile(path, new Uint8Array(out));
      setInfo(`OK — guardado en ${path}`);
    } catch (err) {
      setError(typeof err === "string" ? err : JSON.stringify(err));
    } finally {
      setBusy(false);
    }
  }

  // Pre-cálculo de totales para mostrar partida doble en vivo
  const v1Debe = n(f.sueldos) + n(f.vacaciones);
  const v1Haber = n(f.adelantos) + n(f.ir5ta) + n(f.essalud) + n(f.afp) + n(f.neto);
  const v4Debe = n(f.gratificacion) + n(f.bonifExtraord);
  const v4Haber = v4Debe;

  return (
    <section aria-labelledby="planilla-title">
      <h2 id="planilla-title">Planilla (Origen 11)</h2>
      <p>
        Ingresá los montos ya calculados de cada voucher. Dejá en blanco los que no apliquen.
        Validamos partida doble por voucher antes de generar el XLS.
      </p>

      <div className="form-row">
        <label>
          Mes&nbsp;
          <input
            type="number"
            min={1}
            max={12}
            value={f.mes}
            onChange={(e) => set("mes", Number(e.target.value))}
          />
        </label>
        <label>
          Año&nbsp;
          <input
            type="number"
            min={2000}
            max={2100}
            value={f.anio}
            onChange={(e) => set("anio", Number(e.target.value))}
          />
        </label>
        <label>
          T. Cambio&nbsp;
          <input
            type="number"
            step="0.001"
            min={0}
            value={f.tipoCambio}
            onChange={(e) => set("tipoCambio", Number(e.target.value))}
          />
        </label>
      </div>

      <fieldset className="voucher">
        <legend>
          Voucher 1 — PLANILLA · debe={v1Debe.toFixed(2)} / haber={v1Haber.toFixed(2)}
        </legend>
        <MontoLabel label="Sueldos (62111)" v={f.sueldos} onChange={(s) => set("sueldos", s)} />
        <MontoLabel
          label="Vacaciones (62711)"
          v={f.vacaciones}
          onChange={(s) => set("vacaciones", s)}
        />
        <MontoLabel
          label="Adelantos (14124)"
          v={f.adelantos}
          onChange={(s) => set("adelantos", s)}
        />
        <MontoLabel label="IR 5ta (40173)" v={f.ir5ta} onChange={(s) => set("ir5ta", s)} />
        <MontoLabel label="ESSALUD (40311)" v={f.essalud} onChange={(s) => set("essalud", s)} />
        <MontoLabel label="AFP (41724)" v={f.afp} onChange={(s) => set("afp", s)} />
        <MontoLabel label="Neto a pagar (41111)" v={f.neto} onChange={(s) => set("neto", s)} />
      </fieldset>

      <fieldset className="voucher">
        <legend>Voucher 2 — EPS</legend>
        <MontoLabel label="EPS (40311 ↔ 14124)" v={f.eps} onChange={(s) => set("eps", s)} />
      </fieldset>

      <fieldset className="voucher">
        <legend>Voucher 3 — CTS</legend>
        <MontoLabel label="CTS (62911 ↔ 41511)" v={f.cts} onChange={(s) => set("cts", s)} />
      </fieldset>

      <fieldset className="voucher">
        <legend>
          Voucher 4 — GRATIFICACIÓN · debe={v4Debe.toFixed(2)} / haber={v4Haber.toFixed(2)}
        </legend>
        <MontoLabel
          label="Gratificación (62141 ↔ 41141)"
          v={f.gratificacion}
          onChange={(s) => set("gratificacion", s)}
        />
        <MontoLabel
          label="Bonif. Extraord. (62901 ↔ 41901)"
          v={f.bonifExtraord}
          onChange={(s) => set("bonifExtraord", s)}
        />
      </fieldset>

      <button type="button" onClick={onGenerar} disabled={busy} className="btn-primary">
        {busy ? "Generando…" : "Generar XLS"}
      </button>

      {info && <p className="msg-ok">{info}</p>}
      {error && <p className="msg-error">{error}</p>}
    </section>
  );
}

function MontoLabel({
  label,
  v,
  onChange,
}: { label: string; v: string; onChange: (s: string) => void }) {
  return (
    <label className="monto-label">
      <span>{label}</span>
      <input
        type="number"
        step="0.01"
        min={0}
        value={v}
        onChange={(e) => onChange(e.target.value)}
      />
    </label>
  );
}
