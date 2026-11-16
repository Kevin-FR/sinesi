//! Presentation changes preserve topology identity and protocol configuration.
use crate::*;
pub const PRESENTATION_PROFILE: &str = "sns-educational-presentation-v1";
impl Project {
    pub fn with_symbol(
        &self,
        device: &Id,
        symbol: Option<DeviceSymbol>,
    ) -> Result<Self, Diagnostic> {
        let target = self
            .device(device)
            .ok_or_else(|| Diagnostic::new("project.invalid_reference", device.as_str()))?;
        if symbol == Some(DeviceSymbol::Cloud) && target.kind != DeviceKind::Switch {
            return Err(Diagnostic::new(
                "presentation.invalid_symbol",
                device.as_str(),
            ));
        }
        let mut next = self.clone();
        if next.lan.get(device).and_then(|c| c.symbol) == symbol {
            return Ok(next);
        }
        next.lan.entry(device.clone()).or_default().symbol = symbol;
        if symbol == Some(DeviceSymbol::Cloud) && next.schema_version < 8 {
            next.schema_version = 8;
            next.semantic_profile = PRESENTATION_PROFILE.into();
        } else if symbol.is_some() && next.schema_version < 4 {
            next.schema_version = 4;
            next.semantic_profile = "sns-educational-routing-v1".into();
        }
        next.validate()?;
        Ok(next)
    }
}
