mod launch;
use eframe::egui;
use sns_settings::{
    Settings,
    configuration::Sources,
    diagnostics::{self, Event, Journal, Logging},
};
use std::process::ExitCode;
enum Application {
    Workbench(Box<sns_ui_egui::Workbench>),
    Prototype(Box<sns_ui_egui::prototype::SinesiUi>),
}
struct Desktop {
    ui: Application,
}
impl eframe::App for Desktop {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        match &mut self.ui {
            Application::Workbench(app) => app.show(ui),
            Application::Prototype(app) => app.show(ui),
        }
    }
}
fn run() -> Result<(), String> {
    let launch = launch::parse(std::env::args_os().skip(1))
        .map_err(|code| Settings::default().language.text(code).to_owned())?;
    let cwd = std::env::current_dir().map_err(|_| {
        Settings::default()
            .language
            .text("launch.working_directory_failed")
            .to_owned()
    })?;
    let prepared = launch.prepare(
        &cwd,
        Sources::native(),
        sns_settings::default_session_directory(),
    );
    let language = prepared.configuration.effective.language;
    let (log_path, file) = if let Some(path) = &prepared.log_file {
        let protected = [
            prepared.project.as_deref(),
            prepared.configuration.administrator_path.as_deref(),
            prepared.configuration.user_path.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        let (actual, file) = diagnostics::open_log(path, &protected, prepared.session.as_deref())
            .map_err(|code| language.text(code).to_owned())?;
        let path = format!("{actual:?}");
        if let Ok(message) = language.format("diagnostic_log_path", &[("path", &path)]) {
            eprintln!("{message}");
        }
        (Some(actual), Some(file))
    } else {
        (None, None)
    };
    let logging = Logging::start(launch.verbose, file)
        .map_err(|_| language.text("diagnostic.log_write_failed").to_owned())?;
    let journal = logging.journal();
    journal.record(Event::Startup);
    let outcome = run_prepared(launch, prepared, journal.clone(), log_path);
    journal.record(Event::Exit);
    if logging.finish().is_err() {
        let message = language.text("diagnostic.log_write_failed");
        if outcome.is_ok() {
            return Err(message.to_owned());
        }
        eprintln!("{message}");
    }
    outcome
}

fn run_prepared(
    launch: launch::Launch,
    prepared: launch::Prepared,
    journal: Journal,
    log_path: Option<std::path::PathBuf>,
) -> Result<(), String> {
    let language = prepared.configuration.effective.language;
    journal.record(Event::ConfigurationResolved);
    let initial_diagnostics = prepared.configuration.diagnostics.clone();
    if !initial_diagnostics.is_empty() {
        journal.record(Event::ConfigurationFallback);
    }
    for issue in &initial_diagnostics {
        eprintln!("{}", issue.localized(language));
    }
    if launch.help {
        println!("{}", launch.help_text(&prepared.configuration));
        return Ok(());
    }
    if launch.version {
        println!("Sinesi {}", sns_settings::help::PRODUCT_VERSION);
        return Ok(());
    }
    if launch.check_config {
        let app = sns_ui_egui::Workbench::with_configuration_and_wait_factor(
            prepared.configuration.clone(),
            None,
            prepared.project.clone(),
            prepared.wait_factor,
        )
        .map_err(|_| {
            journal.record(Event::ProjectLoadFailed);
            language.text("launch.project_invalid").to_owned()
        })?;
        journal.record(Event::ProjectReady);
        print_additional_diagnostics(&app, &initial_diagnostics, &journal);
        println!(
            "{}",
            serde_json::to_string_pretty(&launch::report(&prepared, &app)).map_err(|_| {
                journal.record(Event::ReportFailed);
                language.text("launch.report_failed").to_owned()
            })?
        );
        return Ok(());
    }
    let theme = prepared.theme;
    let viewport = prepared.viewport();
    let mut app = if launch.prototype {
        Application::Prototype(Box::new(sns_ui_egui::prototype::SinesiUi::new().map_err(
            |_| {
                journal.record(Event::ProjectLoadFailed);
                language.text("launch.project_invalid").to_owned()
            },
        )?))
    } else {
        Application::Workbench(Box::new(
            sns_ui_egui::Workbench::with_configuration_and_wait_factor(
                prepared.configuration,
                prepared.session,
                prepared.project,
                prepared.wait_factor,
            )
            .map_err(|_| {
                journal.record(Event::ProjectLoadFailed);
                language.text("launch.project_invalid").to_owned()
            })?,
        ))
    };
    journal.record(Event::ProjectReady);
    if let Application::Workbench(app) = &mut app {
        print_additional_diagnostics(app, &initial_diagnostics, &journal);
        app.attach_diagnostics(journal.clone(), log_path);
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    let window_journal = journal.clone();
    eframe::run_native(
        "Sinesi",
        options,
        Box::new(move |context| {
            window_journal.record(Event::WindowReady);
            context.egui_ctx.set_theme(theme);
            Ok(Box::new(Desktop { ui: app }))
        }),
    )
    .map_err(|error| {
        journal.record(Event::WindowFailed);
        format!("{}: {error}", language.text("launch.window_failed"))
    })
}
fn print_additional_diagnostics(
    app: &sns_ui_egui::Workbench,
    previous: &[sns_settings::configuration::ConfigurationDiagnostic],
    journal: &Journal,
) {
    if let Some(configuration) = app.startup_configuration() {
        for issue in configuration
            .diagnostics
            .iter()
            .filter(|issue| !previous.contains(issue))
        {
            journal.record(Event::ConfigurationFallback);
            eprintln!("{}", issue.localized(configuration.effective.language));
        }
    }
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
