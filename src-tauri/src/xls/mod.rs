// Parsing + generación de xls/csv en memoria.
// REGLA DURA: nada de lo que pase por acá puede tocar disco.
// Input: Vec<u8>. Output: Vec<u8> (que el frontend convierte a base64).

pub mod depreciacion;
pub mod output;
pub mod planilla;
