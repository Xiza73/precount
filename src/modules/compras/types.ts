export type TipoComprobante = "FACTURA" | "BOLETA" | "NOTA_CREDITO";

export type ComprobanteCompra = {
  fechaEmision: string;
  tipoComprobante: TipoComprobante;
  serie: string;
  numero: string;
  rucProveedor: string;
  razonSocial: string;
  baseImponible: number;
  igv: number;
  total: number;
  cuentaContable: string;
};
