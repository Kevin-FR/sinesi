//! Live presentation commands; the engine owns changes made during a simulation.
use super::*;
impl Workbench {
    pub(super) fn cloud_checkbox(
        &mut self,
        ui: &mut egui::Ui,
        device: &Device,
        config: &mut LanConfig,
    ) {
        if device.kind != DeviceKind::Switch {
            return;
        }
        let l = self.language;
        let mut cloud = if self.mode == Mode::Simulation {
            self.project().lan.get(&device.id).and_then(|c| c.symbol) == Some(DeviceSymbol::Cloud)
        } else {
            config.symbol == Some(DeviceSymbol::Cloud)
        };
        if ui
            .add_enabled(
                !self.worker.pending_commands(),
                egui::Checkbox::new(&mut cloud, l.text("switch_cloud")),
            )
            .changed()
        {
            let symbol = cloud.then_some(DeviceSymbol::Cloud);
            if self.mode == Mode::Simulation {
                self.worker.submit(Job::Command(Command::SetSymbol {
                    device: device.id.clone(),
                    symbol,
                }));
            } else {
                self.apply(Edit::SetSymbol {
                    device: device.id.clone(),
                    symbol,
                });
                config.symbol = self.project().lan.get(&device.id).and_then(|c| c.symbol);
            }
        }
        ui.label(l.text("switch_cloud_hint"));
    }
}
