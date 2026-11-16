//! Repository checks, shared by contributors and CI. No application dependency.
use std::{
    env, io,
    path::Path,
    process::{Command, ExitCode},
};

#[derive(Debug, PartialEq)]
struct Step {
    program: &'static str,
    arguments: Vec<&'static str>,
}

fn python(script: &'static str) -> Step {
    Step {
        program: "python3",
        arguments: vec![script],
    }
}

fn cargo(arguments: &[&'static str]) -> Step {
    Step {
        program: "cargo",
        arguments: arguments.to_vec(),
    }
}

fn plan(task: &str) -> Result<Vec<Step>, String> {
    let checks = || {
        vec![
            cargo(&["fmt", "--all", "--check"]),
            python("scripts/check_layers.py"),
            python("scripts/test_layers.py"),
            python("scripts/check_i18n.py"),
            python("scripts/check_network_assets.py"),
            python("scripts/check_fonts.py"),
            python("scripts/test_docs.py"),
            python("scripts/test_filius_inspection.py"),
            Step {
                program: "python3",
                arguments: vec!["scripts/docs.py", "check"],
            },
        ]
    };
    match task {
        "check" => Ok(checks()),
        "layers" => Ok(vec![
            python("scripts/check_layers.py"),
            python("scripts/test_layers.py"),
        ]),
        "ci" => {
            let mut steps = checks();
            steps.push(cargo(&[
                "clippy",
                "--workspace",
                "--all-targets",
                "--locked",
                "--offline",
                "--",
                "-D",
                "warnings",
            ]));
            steps.push(cargo(&["test", "--workspace", "--locked", "--offline"]));
            for name in ["icons", "diagram", "symbols-icons", "symbols-diagram"] {
                let (image, metadata) = match name {
                    "icons" => (
                        "docs/verification/j7-png-icons.png",
                        "docs/verification/j7-png-icons.json",
                    ),
                    "diagram" => (
                        "docs/verification/j7-png-diagram.png",
                        "docs/verification/j7-png-diagram.json",
                    ),
                    "symbols-icons" => (
                        "docs/verification/j7-png-symbols-icons.png",
                        "docs/verification/j7-png-symbols-icons.json",
                    ),
                    _ => (
                        "docs/verification/j7-png-symbols-diagram.png",
                        "docs/verification/j7-png-symbols-diagram.json",
                    ),
                };
                steps.push(Step {
                    program: "python3",
                    arguments: vec![
                        "scripts/check_png_exports.py",
                        image,
                        "--metadata",
                        metadata,
                    ],
                });
            }
            Ok(steps)
        }
        "ui" => Ok(vec![cargo(&[
            "test",
            "-p",
            "sns-ui-egui",
            "--features",
            "ui-snapshots",
            "--test",
            "png",
            "--test",
            "report",
            "--test",
            "help",
            "--test",
            "preferences",
            "--test",
            "configuration",
            "--test",
            "student",
            "--locked",
            "--offline",
        ])]),
        _ => Err(format!(
            "Unknown task: {task}. Use check, layers, ci or ui."
        )),
    }
}

fn command(step: &Step, root: &Path) -> Command {
    // Use argument arrays: paths and user environment are never shell programs.
    let program = match step.program {
        "cargo" => env::var_os("CARGO").unwrap_or_else(|| "cargo".into()),
        "python3" => env::var_os("SNS_PYTHON").unwrap_or_else(|| "python3".into()),
        program => program.into(),
    };
    let mut command = Command::new(program);
    command
        .args(&step.arguments)
        // egui_kittest can rewrite references through the environment alone.
        .env_remove("UPDATE_SNAPSHOTS")
        .current_dir(root);
    command
}

fn run(step: &Step, root: &Path) -> io::Result<()> {
    eprintln!("+ {} {}", step.program, step.arguments.join(" "));
    let status = command(step, root).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "{} failed: {status}",
            step.program
        )))
    }
}

fn main() -> ExitCode {
    let arguments: Vec<_> = env::args().skip(1).collect();
    if arguments.is_empty() || arguments.as_slice() == ["--help"] {
        println!(
            "cargo xtask <check|layers|ci|ui>\ncheck: static checks; ci: checks + lint + workspace tests + PNG proofs; ui: current PNG/PDF/help/preferences/configuration GPU suites.\nci requires cached dependencies (cargo fetch --locked first). It neither regenerates snapshots nor proves native/three-OS workflows."
        );
        return ExitCode::SUCCESS;
    }
    let [task] = arguments.as_slice() else {
        eprintln!("Expected one task. Use cargo xtask --help.");
        return ExitCode::FAILURE;
    };
    let result = plan(task).and_then(|steps| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or("Missing workspace root")?;
        for step in steps {
            run(&step, root).map_err(|error| error.to_string())?;
        }
        Ok(())
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xtask: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ci_contains_checks_and_fails_closed_on_unknown_tasks() {
        let checks = plan("check").unwrap();
        let ci = plan("ci").unwrap();
        assert!(ci.starts_with(&checks));
        assert!(
            ci.iter()
                .any(|step| step.arguments.starts_with(&["clippy", "--workspace"]))
        );
        assert!(
            ci.iter()
                .any(|step| step.arguments.starts_with(&["test", "--workspace"]))
        );
        assert!(plan("unknown").is_err());
    }

    #[test]
    fn failed_subprocess_is_reported() {
        let step = Step {
            program: "cargo",
            arguments: vec!["--sns-invalid-command"],
        };
        assert!(run(&step, Path::new(env!("CARGO_MANIFEST_DIR"))).is_err());
    }

    #[test]
    fn ui_comparison_removes_snapshot_update_from_child_environment() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        for step in plan("ui").unwrap() {
            assert!(
                command(&step, root)
                    .get_envs()
                    .any(|(name, value)| { name == "UPDATE_SNAPSHOTS" && value.is_none() })
            );
        }
    }
}
