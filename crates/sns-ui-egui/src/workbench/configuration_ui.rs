//! Initial configuration diagnostics are presentation data, never network facts.
use super::*;
use sns_settings::configuration::ResolvedConfiguration;
#[derive(Default)]
pub(super) struct ConfigurationUi {
    snapshot: Option<ResolvedConfiguration>,
    open: bool,
}
impl Workbench {
    pub fn startup_configuration(&self) -> Option<&ResolvedConfiguration> {
        self.configuration_ui.snapshot.as_ref()
    }
    pub(super) fn load_startup_configuration(&mut self, mut configuration: ResolvedConfiguration) {
        if configuration
            .administrator_path
            .as_ref()
            .zip(configuration.user_path.as_ref())
            .is_some_and(|(a, b)| sns_project::destination::same_destination(a, b))
        {
            configuration.user_writable = false;
            configuration
                .diagnostics
                .push(sns_settings::configuration::ConfigurationDiagnostic {
                    source: sns_settings::configuration::Source::User,
                    field: "configuration".into(),
                    code: "configuration.path_conflict",
                });
        }
        self.load_preferences(configuration.effective.clone());
        self.preferences_ui.persistent = configuration.persistent.clone();
        self.settings_writable = configuration.user_writable;
        self.configuration_ui.open = !configuration.diagnostics.is_empty();
        self.configuration_ui.snapshot = Some(configuration);
    }
    pub(super) fn configuration_output_allowed(
        &self,
        path: &std::path::Path,
    ) -> Result<(), String> {
        if self
            .log_path
            .as_ref()
            .is_some_and(|p| sns_project::destination::same_destination(path, p))
        {
            return Err("diagnostic.log_conflict".into());
        }
        if self
            .configuration_ui
            .snapshot
            .as_ref()
            .and_then(|s| s.administrator_path.as_ref())
            .is_some_and(|p| sns_project::destination::same_destination(path, p))
        {
            return Err("configuration.protected_path".into());
        }
        Ok(())
    }
    pub(super) fn open_configuration_diagnostics(&mut self) {
        self.configuration_ui.open = self
            .configuration_ui
            .snapshot
            .as_ref()
            .is_some_and(|c| !c.diagnostics.is_empty());
    }
    pub(super) fn configuration_window(&mut self, ctx: &egui::Context) {
        if !self.configuration_ui.open {
            return;
        }
        let Some(configuration) = &self.configuration_ui.snapshot else {
            return;
        };
        let l = self.language;
        let mut open = true;
        let mut close = false;
        egui::Window::new(l.text("configuration_title"))
            .id(egui::Id::unique("startup-configuration"))
            .default_pos([90., 90.])
            .default_size([760., 360.])
            .max_size((ctx.content_rect().size() - Vec2::splat(24.)).max(Vec2::splat(100.)))
            .vscroll(true)
            .hscroll(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(l.text("configuration_fallback_hint"));
                for issue in &configuration.diagnostics {
                    ui.label(issue.localized(l));
                }
                if !configuration.user_writable {
                    ui.label(l.text("configuration_readonly"));
                }
                ui.collapsing(l.text("configuration_files"), |ui| {
                    for (source, path) in [
                        (
                            sns_settings::configuration::Source::Administrator,
                            &configuration.administrator_path,
                        ),
                        (
                            sns_settings::configuration::Source::User,
                            &configuration.user_path,
                        ),
                    ] {
                        if let Some(path) = path {
                            ui.label(l.text(source.label_key()));
                            ui.add(
                                egui::Label::new(path.to_string_lossy())
                                    .selectable(true)
                                    .wrap(),
                            );
                        }
                    }
                });
                close = ui.button(l.text("close_view")).clicked();
            });
        self.configuration_ui.open = open && !close;
    }
}
