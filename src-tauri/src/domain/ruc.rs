/// Valida un RUC peruano (11 dígitos, módulo 11).
///
/// Algoritmo: pesos `[5, 4, 3, 2, 7, 6, 5, 4, 3, 2]` para los primeros 10 dígitos.
/// El dígito verificador (posición 11) debe coincidir con `11 - (suma % 11)`,
/// donde si el resultado es 10 → 0, y si es 11 → 1.
pub fn is_valid_ruc(ruc: &str) -> bool {
    if ruc.len() != 11 || !ruc.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    const PESOS: [u32; 10] = [5, 4, 3, 2, 7, 6, 5, 4, 3, 2];

    let digits: Vec<u32> = ruc.chars().filter_map(|c| c.to_digit(10)).collect();
    let sum: u32 = digits[..10]
        .iter()
        .zip(PESOS.iter())
        .map(|(d, w)| d * w)
        .sum();

    let resto = sum % 11;
    let esperado = match 11 - resto {
        10 => 0,
        11 => 1,
        n => n,
    };

    digits[10] == esperado
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rechaza_longitud_distinta() {
        assert!(!is_valid_ruc("1234567890"));
        assert!(!is_valid_ruc("123456789012"));
        assert!(!is_valid_ruc(""));
    }

    #[test]
    fn rechaza_no_numericos() {
        assert!(!is_valid_ruc("2051234567A"));
    }

    #[test]
    fn acepta_ruc_valido_sunat_ejemplo() {
        // RUC público de SUNAT (entidad estatal): 20131312955.
        assert!(is_valid_ruc("20131312955"));
    }
}
