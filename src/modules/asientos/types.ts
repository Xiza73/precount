/**
 * Espejo TS de `LineaAsiento` en Rust (`src-tauri/src/domain/asiento.rs`).
 * Las 42 columnas del XLS SISCONT — schema compartido por todos los módulos.
 */
export type LineaAsiento = {
  origen: string;
  numVoucher: number;
  fecha: string;
  cuenta: string;
  debe: number | null;
  haber: number | null;
  moneda: string;
  tipoCambio: number;
  doc: string;
  numDoc: string;
  fecDoc: string;
  fecVen: string;
  codProvClie: string;
  cCosto: string;
  presupuesto: string;
  fEfectivo: string;
  glosa: string;
  libroCvr: string;
  mtoNeto1: string;
  mtoNeto2: string;
  mtoNeto3: string;
  mtoNeto4: string;
  mtoNeto5: string;
  mtoNeto6: string;
  mtoNeto7: string;
  mtoNeto8: string;
  mtoNeto9: string;
  mtoIgv: string;
  refDoc: string;
  refNumDoc: string;
  refFecha: string;
  dNumero: string;
  dFecha: string;
  ruc: string;
  rSocial: string;
  tipo: string;
  tipDocIden: string;
  medioPago: string;
  apellido1: string;
  apellido2: string;
  nombre: string;
  tBien: string;
};

export type Origen = "07" | "11" | "13" | "14";

export const ORIGEN_LABEL: Record<Origen, string> = {
  "07": "Banca",
  "11": "Planilla",
  "13": "Detracciones",
  "14": "Depreciación",
};
