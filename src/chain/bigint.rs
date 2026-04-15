use crate::chain::fil;
use anyhow::{Context, Result};
use num_bigint::BigInt as NumBigInt;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BigInt(pub NumBigInt);

impl Default for BigInt {
    fn default() -> Self {
        Self::zero()
    }
}

impl BigInt {
    pub fn zero() -> Self {
        Self(NumBigInt::from(0))
    }

    pub fn parse_atto_str(s: &str) -> Result<Self> {
        let value = NumBigInt::from_str(s.trim())
            .with_context(|| format!("invalid attoFIL integer: {}", s))?;
        Ok(Self(value))
    }

    pub fn parse_token_amount(s: &str) -> Result<Self> {
        Ok(Self(fil::parse_fil(s)?))
    }
}

impl fmt::Display for BigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for BigInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for BigInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        BigInt::parse_atto_str(&s).map_err(serde::de::Error::custom)
    }
}
