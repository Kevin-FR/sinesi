//! Response-wait policy. This does not change link speed or protocol lifetimes.
use crate::Diagnostic;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(into = "u8")]
pub struct WaitFactor(u8);

impl<'de> Deserialize<'de> for WaitFactor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl serde::de::Visitor<'_> for Visitor {
            type Value = WaitFactor;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("an integer from 1 to 5")
            }
            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<WaitFactor, E> {
                u8::try_from(value)
                    .ok()
                    .and_then(|v| WaitFactor::try_from(v).ok())
                    .ok_or_else(|| E::custom("simulation.invalid_wait_factor"))
            }
            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<WaitFactor, E> {
                u64::try_from(value)
                    .map_err(E::custom)
                    .and_then(|v| self.visit_u64(v))
            }
            fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<WaitFactor, E> {
                if (1.0..=5.0).contains(&value) && value.fract() == 0.0 {
                    Ok(WaitFactor(value as u8))
                } else {
                    Err(E::custom("simulation.invalid_wait_factor"))
                }
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}

impl Default for WaitFactor {
    fn default() -> Self {
        Self(1)
    }
}
impl TryFrom<u8> for WaitFactor {
    type Error = Diagnostic;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (1..=5).contains(&value) {
            Ok(Self(value))
        } else {
            Err(Diagnostic::new(
                "simulation.invalid_wait_factor",
                "wait_factor",
            ))
        }
    }
}
impl From<WaitFactor> for u8 {
    fn from(value: WaitFactor) -> Self {
        value.0
    }
}
impl WaitFactor {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
    pub fn scale(self, delay_us: u64) -> Result<u64, Diagnostic> {
        delay_us
            .checked_mul(u64::from(self.0))
            .ok_or_else(|| Diagnostic::new("simulation.time_overflow", "response_wait"))
    }
}
