use num_bigint::BigInt as NumBigInt;
use anyhow::{anyhow, Result};
use std::str::FromStr;

// 1 FIL = 10^18 attoFIL
pub const FILECOIN_PRECISION: u64 = 1_000_000_000_000_000_000;

#[allow(dead_code)]
pub fn parse_fil(s: &str) -> Result<NumBigInt> {
    let s = s.trim();

    // 分离数字和单位
    let (num_str, unit) = split_number_unit(s);

    let is_attofil = match unit.to_lowercase().as_str() {
        "" | "fil" => false,
        "attofil" | "afil" => true,
        _ => return Err(anyhow!("unrecognized unit: {}", unit)),
    };

    if num_str.contains('.') {
        parse_decimal(num_str, is_attofil)
    } else {
        let value = NumBigInt::from_str(num_str)
            .map_err(|_| anyhow!("invalid number: {}", num_str))?;

        if is_attofil {
            Ok(value)
        } else {
            Ok(value * NumBigInt::from(FILECOIN_PRECISION))
        }
    }
}

fn split_number_unit(s: &str) -> (&str, &str) {
    let idx = s.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
        .unwrap_or(s.len());
    (&s[..idx], s[idx..].trim())
}

fn parse_decimal(s: &str, is_attofil: bool) -> Result<NumBigInt> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(anyhow!("invalid decimal"));
    }

    let int_part = parts[0];
    let dec_part = parts[1];

    if is_attofil {
        if dec_part.chars().all(|c| c == '0') {
            return NumBigInt::from_str(int_part)
                .map_err(|_| anyhow!("invalid number: {}", s));
        }
        return Err(anyhow!("attoFIL cannot have decimals"));
    }

    if dec_part.len() > 18 {
        return Err(anyhow!("too many decimal places for FIL: {}", s));
    }

    let padded = format!("{:0<18}", dec_part);
    let combined = format!("{}{}", int_part, padded);

    NumBigInt::from_str(&combined)
        .map_err(|_| anyhow!("invalid number: {}", s))
}

pub fn format_fil(attofil: &NumBigInt) -> String {
    let precision = NumBigInt::from(FILECOIN_PRECISION);
    let int_part = attofil / &precision;
    let dec_part = attofil % &precision;

    if dec_part == NumBigInt::from(0) {
        format!("{} FIL", int_part)
    } else {
        let dec_str = format!("{:018}", dec_part);
        let trimmed = dec_str.trim_end_matches('0');
        format!("{}.{} FIL", int_part, trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::parse_fil;

    #[test]
    fn parses_decimal_fil_to_attofil() {
        assert_eq!(
            parse_fil("0.1").unwrap().to_string(),
            "100000000000000000"
        );
    }

    #[test]
    fn parses_explicit_attofil() {
        assert_eq!(parse_fil("123attofil").unwrap().to_string(), "123");
    }

    #[test]
    fn rejects_unknown_unit() {
        assert!(parse_fil("1nanofil").is_err());
    }
}
