import type { LineaAsiento } from "@/modules/asientos/types";
import { invoke } from "@tauri-apps/api/core";
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
      const b64 = await invoke<string>("generar_depreciacion_xls", { lineas });
      descargar(b64, `depreciacion-${anio}-${String(mes).padStart(2, "0")}.xlsx`);
      setInfo(`OK — ${lineas.length} líneas generadas.`);
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

function descargar(b64: string, filename: string) {
  const bin = atob(b64);
  const arr = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) arr[i] = bin.charCodeAt(i);
  const blob = new Blob([arr], {
    type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}
