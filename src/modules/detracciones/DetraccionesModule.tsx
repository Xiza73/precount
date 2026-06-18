import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import { useState } from "react";

type Abono = {
  id: string;
  monto: string;
  cuenta: "12121" | "12122";
  numDoc: string;
  ruc: string;
  razonSocial: string;
  tipo: string;
};

const nuevoAbono = (): Abono => ({
  id: crypto.randomUUID(),
  monto: "",
  cuenta: "12121",
  numDoc: "",
  ruc: "",
  razonSocial: "",
  tipo: "6",
});

function n(s: string): number {
  const v = Number.parseFloat(s);
  return Number.isFinite(v) ? v : 0;
}

export function DetraccionesModule() {
  const [mes, setMes] = useState(11);
  const [anio, setAnio] = useState(2025);
  const [tipoCambio, setTipoCambio] = useState(3.368);
  const [abonos, setAbonos] = useState<Abono[]>(() => [nuevoAbono()]);
  const [error, setError] = useState<string | null>(null);
  const [info, setInfo] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const updateAbono = (i: number, patch: Partial<Abono>) =>
    setAbonos((prev) => prev.map((a, idx) => (idx === i ? { ...a, ...patch } : a)));
  const addAbono = () => setAbonos((prev) => [...prev, nuevoAbono()]);
  const removeAbono = (i: number) =>
    setAbonos((prev) => (prev.length > 1 ? prev.filter((_, idx) => idx !== i) : prev));

  const total = abonos.reduce((acc, a) => acc + n(a.monto), 0);

  async function onGenerar() {
    setBusy(true);
    setError(null);
    setInfo(null);
    try {
      const defaultName = `detracciones-${anio}-${String(mes).padStart(2, "0")}.xlsx`;
      const path = await save({
        defaultPath: defaultName,
        filters: [{ name: "Excel", extensions: ["xlsx"] }],
      });
      if (!path) {
        setInfo("Cancelado.");
        return;
      }
      const out = await invoke<number[]>("generar_detracciones_xls", {
        input: {
          mes,
          anio,
          tipoCambio,
          abonos: abonos.map((a) => ({
            monto: n(a.monto),
            cuenta: a.cuenta,
            numDoc: a.numDoc,
            ruc: a.ruc,
            razonSocial: a.razonSocial,
            tipo: a.tipo || "6",
          })),
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

  return (
    <section aria-labelledby="detracciones-title">
      <h2 id="detracciones-title">Detracciones (Origen 13)</h2>
      <p>
        Voucher 1 — abonos recibidos en la cta del Banco de la Nación. Ingresá cada abono con su
        comprobante (RUC / razón social / número). La cuenta de detracciones <code>104201</code>
        se debita por el total automáticamente.
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

      <fieldset className="voucher">
        <legend>
          Abonos · {abonos.length} fila(s) · total {total.toFixed(2)}
        </legend>
        <table className="abonos">
          <thead>
            <tr>
              <th>Monto</th>
              <th>Cuenta</th>
              <th>N° Doc</th>
              <th>RUC</th>
              <th>Razón social</th>
              <th>Tipo</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {abonos.map((a, i) => (
              <tr key={a.id}>
                <td>
                  <input
                    type="number"
                    step="0.01"
                    min={0}
                    value={a.monto}
                    onChange={(e) => updateAbono(i, { monto: e.target.value })}
                  />
                </td>
                <td>
                  <select
                    value={a.cuenta}
                    onChange={(e) =>
                      updateAbono(i, { cuenta: e.target.value as "12121" | "12122" })
                    }
                  >
                    <option value="12121">12121</option>
                    <option value="12122">12122</option>
                  </select>
                </td>
                <td>
                  <input
                    value={a.numDoc}
                    onChange={(e) => updateAbono(i, { numDoc: e.target.value })}
                    placeholder="E001-1234"
                  />
                </td>
                <td>
                  <input
                    value={a.ruc}
                    onChange={(e) => updateAbono(i, { ruc: e.target.value })}
                    placeholder="20XXXXXXXXX"
                    maxLength={11}
                  />
                </td>
                <td>
                  <input
                    value={a.razonSocial}
                    onChange={(e) => updateAbono(i, { razonSocial: e.target.value })}
                  />
                </td>
                <td>
                  <input
                    value={a.tipo}
                    onChange={(e) => updateAbono(i, { tipo: e.target.value })}
                    style={{ width: "3rem" }}
                  />
                </td>
                <td>
                  <button
                    type="button"
                    className="btn-remove"
                    onClick={() => removeAbono(i)}
                    disabled={abonos.length === 1}
                    title="Eliminar"
                  >
                    ×
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        <button type="button" onClick={addAbono} className="btn-secondary">
          + Agregar abono
        </button>
      </fieldset>

      <button type="button" onClick={onGenerar} disabled={busy} className="btn-primary">
        {busy ? "Generando…" : "Generar XLS"}
      </button>

      {info && <p className="msg-ok">{info}</p>}
      {error && <p className="msg-error">{error}</p>}
    </section>
  );
}
