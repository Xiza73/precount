use crate::domain::error::AppError;
use chrono::NaiveDate;

pub const MESES_NOMBRE: [&str; 12] = [
    "ENERO", "FEBRERO", "MARZO", "ABRIL", "MAYO", "JUNIO", "JULIO", "AGOSTO", "SETIEMBRE",
    "OCTUBRE", "NOVIEMBRE", "DICIEMBRE",
];

pub fn ultimo_dia_mes(anio: i32, mes: u8) -> Result<NaiveDate, AppError> {
    let proximo = if mes == 12 {
        NaiveDate::from_ymd_opt(anio + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(anio, (mes + 1) as u32, 1)
    };
    proximo
        .and_then(|d| d.pred_opt())
        .ok_or_else(|| AppError::Validation(format!("fecha invalida: {anio}-{mes}")))
}

pub fn validar_mes(mes: u8) -> Result<(), AppError> {
    if !(1..=12).contains(&mes) {
        return Err(AppError::Validation(format!("mes inválido: {mes}")));
    }
    Ok(())
}

pub fn nombre_mes(mes: u8) -> Result<&'static str, AppError> {
    validar_mes(mes)?;
    Ok(MESES_NOMBRE[(mes - 1) as usize])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ultimo_dia_diciembre_es_31() {
        assert_eq!(
            ultimo_dia_mes(2025, 12).unwrap(),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()
        );
    }

    #[test]
    fn ultimo_dia_febrero_no_bisiesto() {
        assert_eq!(
            ultimo_dia_mes(2025, 2).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 28).unwrap()
        );
    }

    #[test]
    fn ultimo_dia_febrero_bisiesto() {
        assert_eq!(
            ultimo_dia_mes(2024, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()
        );
    }

    #[test]
    fn mes_invalido_es_error() {
        assert!(validar_mes(0).is_err());
        assert!(validar_mes(13).is_err());
        assert!(validar_mes(1).is_ok());
        assert!(validar_mes(12).is_ok());
    }
}
