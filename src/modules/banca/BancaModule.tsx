import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { writeFile } from "@tauri-apps/plugin-fs";
import { useState } from "react";

type Movimiento = {
  id: string;
  fecha: string; // YYYY-MM-DD
  descripcion: string; // del PDF, ayuda al contador a recordar
  monto: string;
  esCargo: boolean; // true=cargo, false=abono
  cuenta: string; // contracuenta
  glosa: string; // simplificada para SISCONT
  cCosto: string;
  ruc: string;
  razonSocial: string;
};

const nuevoMov = (): Movimiento => ({
  id: crypto.randomUUID(),
  fecha: "",
  descripcion: "",
  monto: "",
  esCargo: true,
  cuenta: "",
  glosa: "",
  cCosto: "",
  ruc: "",
  razonSocial: "",
});

function n(s: string): number {
  const v = Number.parseFloat(s);
  return Number.isFinite(v) ? v : 0;
}

export function BancaModule() {
  const [mes, setMes] = useState(1);
  const [anio, setAnio] = useState(2025);
  const [tipoCambio, setTipoCambio] = useState(3.72);
  const [movs, setMovs] = useState<Movimiento[]>(() => [nuevoMov()]);
  const [textoPdf, setTextoPdf] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [info, setInfo] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const update = (i: number, patch: Partial<Movimiento>) =>
    setMovs((prev) => prev.map((m, idx) => (idx === i ? { ...m, ...patch } : m)));
  const add = () => setMovs((prev) => [...prev, nuevoMov()]);
  const remove = (i: number) =>
    setMovs((prev) => (prev.length > 1 ? prev.filter((_, idx) => idx !== i) : prev));

  async function onExtraer() {
    if (!textoPdf.trim()) {
      setError("Pegá el texto del estado de cuenta primero.");
      return;
    }
    setBusy(true);
    setError(null);
    setInfo(null);
    try {
      const extraidos = await invoke<
        { fecha: string; descripcion: string; monto: number; esCargo: boolean }[]
      >("extraer_movimientos_banca", { texto: textoPdf, mes, anio });
      if (extraidos.length === 0) {
        setInfo("No se encontraron movimientos del mes en el texto.");
        return;
      }
      setMovs(
        extraidos.map((e) => ({
          ...nuevoMov(),
          fecha: e.fecha,
          descripcion: e.descripcion,
          monto: e.monto.toFixed(2),
          esCargo: e.esCargo,
          glosa: e.descripcion,
        })),
      );
      setInfo(`Extraídos ${extraidos.length} movimientos. Completá cuenta, glosa, RUC.`);
    } catch (err) {
      setError(typeof err === "string" ? err : JSON.stringify(err));
    } finally {
      setBusy(false);
    }
  }

  async function onGenerar() {
    setBusy(true);
    setError(null);
    setInfo(null);
    try {
      const defaultName = `banca-${anio}-${String(mes).padStart(2, "0")}.xlsx`;
      const path = await save({
        defaultPath: defaultName,
        filters: [{ name: "Excel", extensions: ["xlsx"] }],
      });
      if (!path) {
        setInfo("Cancelado.");
        return;
      }
      const out = await invoke<number[]>("generar_banca_xls", {
        input: {
          mes,
          anio,
          tipoCambio,
          movimientos: movs.map((m) => ({
            fecha: m.fecha,
            monto: n(m.monto),
            esCargo: m.esCargo,
            cuenta: m.cuenta,
            glosa: m.glosa,
            cCosto: m.cCosto,
            ruc: m.ruc,
            razonSocial: m.razonSocial,
          })),
        },
      });
      await writeFile(path, new Uint8Array(out));
      setInfo(`OK — ${movs.length} vouchers guardados en ${path}`);
    } catch (err) {
      setError(typeof err === "string" ? err : JSON.stringify(err));
    } finally {
      setBusy(false);
    }
  }

  const totalCargos = movs.filter((m) => m.esCargo).reduce((acc, m) => acc + n(m.monto), 0);
  const totalAbonos = movs.filter((m) => !m.esCargo).reduce((acc, m) => acc + n(m.monto), 0);

  return (
    <section aria-labelledby="banca-title">
      <h2 id="banca-title">Banca (Origen 07)</h2>
      <p>
        Estado de cuenta del banco operativo (<code>1041101</code>). Cada movimiento se convierte en
        un voucher de 2 líneas (banco + contracuenta). La fecha del asiento es el último día del
        mes; cada voucher conserva la fecha real del movimiento en <code>Fec.Doc</code>.
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
        <legend>Pegar texto del estado de cuenta BCP (opcional)</legend>
        <p style={{ fontSize: "0.8125rem", color: "var(--color-muted)", margin: "0 0 0.5rem" }}>
          Copiá el contenido del PDF del BCP y pegalo. La app extrae fecha + descripción + monto de
          cada movimiento del mes. El sufijo <code>-</code> en el monto marca un cargo. Después
          completás cuenta contracuenta, glosa simplificada y RUC/razón social cuando aplique.
        </p>
        <textarea
          value={textoPdf}
          onChange={(e) => setTextoPdf(e.target.value)}
          placeholder="02-01 DE LGG PISCO PERU SA ... 1,558.00 ..."
          rows={6}
          className="pdf-textarea"
        />
        <button
          type="button"
          onClick={onExtraer}
          disabled={busy || !textoPdf.trim()}
          className="btn-secondary"
        >
          Extraer movimientos del texto
        </button>
      </fieldset>

      <fieldset className="voucher">
        <legend>
          Movimientos · {movs.length} · cargos {totalCargos.toFixed(2)} · abonos{" "}
          {totalAbonos.toFixed(2)}
        </legend>
        <div className="banca-table-wrap">
          <table className="banca-table">
            <thead>
              <tr>
                <th>Fecha</th>
                <th>Descripción</th>
                <th>Monto</th>
                <th>Tipo</th>
                <th>Cuenta</th>
                <th>Glosa</th>
                <th>C.Costo</th>
                <th>RUC</th>
                <th>Razón social</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {movs.map((m, i) => (
                <tr key={m.id}>
                  <td>
                    <input
                      type="date"
                      value={m.fecha}
                      onChange={(e) => update(i, { fecha: e.target.value })}
                    />
                  </td>
                  <td>
                    <input
                      value={m.descripcion}
                      onChange={(e) => update(i, { descripcion: e.target.value })}
                      title={m.descripcion}
                    />
                  </td>
                  <td>
                    <input
                      type="number"
                      step="0.01"
                      min={0}
                      value={m.monto}
                      onChange={(e) => update(i, { monto: e.target.value })}
                    />
                  </td>
                  <td>
                    <select
                      value={m.esCargo ? "cargo" : "abono"}
                      onChange={(e) => update(i, { esCargo: e.target.value === "cargo" })}
                    >
                      <option value="cargo">Cargo</option>
                      <option value="abono">Abono</option>
                    </select>
                  </td>
                  <td>
                    <input
                      value={m.cuenta}
                      onChange={(e) => update(i, { cuenta: e.target.value })}
                      placeholder="10101"
                    />
                  </td>
                  <td>
                    <input value={m.glosa} onChange={(e) => update(i, { glosa: e.target.value })} />
                  </td>
                  <td>
                    <input
                      value={m.cCosto}
                      onChange={(e) => update(i, { cCosto: e.target.value })}
                    />
                  </td>
                  <td>
                    <input
                      value={m.ruc}
                      onChange={(e) => update(i, { ruc: e.target.value })}
                      maxLength={11}
                    />
                  </td>
                  <td>
                    <input
                      value={m.razonSocial}
                      onChange={(e) => update(i, { razonSocial: e.target.value })}
                    />
                  </td>
                  <td>
                    <button
                      type="button"
                      className="btn-remove"
                      onClick={() => remove(i)}
                      disabled={movs.length === 1}
                      title="Eliminar"
                    >
                      ×
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <button type="button" onClick={add} className="btn-secondary">
          + Agregar movimiento
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
