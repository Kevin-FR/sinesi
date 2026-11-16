//! Explicit runtime conditions. Projects retain physical link defaults; scenarios/checkpoints store changes.
use crate::*;
pub const MAX_LINK_DELAY_US: u64 = 60_000_000;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkConditions {
    pub delay_us: Decimal,
    pub jitter_us: Decimal,
    pub loss_ppm: u32,
}
impl LinkConditions {
    pub fn validate(&self) -> Result<(), Diagnostic> {
        if self.loss_ppm > 1_000_000
            || self
                .delay_us
                .0
                .checked_add(self.jitter_us.0)
                .is_none_or(|n| n > MAX_LINK_DELAY_US)
        {
            return Err(Diagnostic::new(
                "simulation.invalid_link_conditions",
                "link_conditions",
            ));
        }
        Ok(())
    }
}
