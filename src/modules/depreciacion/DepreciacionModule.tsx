import type { LineaAsiento } from "@/modules/asientos/types";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import { useState } from "react";

export function DepreciacionModule() {
  const [mes, setMes] = useState(12);
  const [anio, setAnio] = useState(2025);
  const [tipoCambio, setTipoCambio] = useState(3.36);
  const [error, setError] = useState<string | null>(null);
  const [info, setInfo] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function onFileChange(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    setBusy(true);
    setError(null);
    setInfo(null);
    try {
      const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
      const lineas = await invoke<LineaAsiento[]>("parse_depreciacion", {
        bytes,
        mes,
        anio,
        tipoCambio,
      });

      const defaultName = `depreciacion-${anio}-${String(mes).padStart(2, "0")}.xlsx`;
      const path = await save({
        defaultPath: defaultName,
        filters: [{ name: "Excel", extensions: ["xlsx"] }],
      });
      if (!path) {
        setInfo("Cancelado.");
        return;
      }

      const out = await invoke<number[]>("generar_depreciacion_xls", { lineas });
      await writeFile(path, new Uint8Array(out));
      setInfo(`OK — ${lineas.length} líneas guardadas en ${path}`);
    } catch (err) {
      setError(typeof err === "string" ? err : JSON.stringify(err));
    } finally {
      setBusy(false);
      e.target.value = "";
    }
  }

  return (
    <section aria-labelledby="depreciacion-title">
      <h2 id="depreciacion-title">Depreciación (Origen 14)</h2>
      <p>
        Cargá el xlsx de depreciación de la empresa. Generamos el asiento del mes/año indicado
        contra las cuentas <code>68613</code> (debe) / <code>39613</code> (haber).
      </p>
      <div className="form-row">
        <label>
          Mes&nbsp;
          <input
            type="number"
            min={1}
            max={12}
            value={mes}
            onChange={(e) => setMes(Number(e.target.value))}
          />
        </label>
        <label>
          Año&nbsp;
          <input
            type="number"
            min={2000}
            max={2100}
            value={anio}
            onChange={(e) => setAnio(Number(e.target.value))}
          />
        </label>
        <label>
          T. Cambio&nbsp;
          <input
            type="number"
            step="0.001"
            min={0}
            value={tipoCambio}
            onChange={(e) => setTipoCambio(Number(e.target.value))}
          />
        </label>
      </div>
      <input type="file" accept=".xlsx,.xls" onChange={onFileChange} disabled={busy} />
      {busy && <p>Procesando…</p>}
      {info && <p className="msg-ok">{info}</p>}
      {error && <p className="msg-error">{error}</p>}
    </section>
  );
}
