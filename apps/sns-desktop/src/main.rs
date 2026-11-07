use eframe::egui;
struct Desktop {
    ui: sns_ui_egui::SinesiUi,
}
impl eframe::App for Desktop {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ui.show(ui);
    }
}
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Sinesi — Simple network simulator")
            .with_inner_size([1180.0, 780.0])
            .with_min_inner_size([820.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Sinesi",
        options,
        Box::new(|context| {
            context.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(Desktop {
                ui: sns_ui_egui::SinesiUi::new().map_err(std::io::Error::other)?,
            }))
        }),
    )
}
