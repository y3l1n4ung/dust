use std::env;

use dust_driver::{
    BuildRequest, CheckRequest, CleanRequest, CommandRequest, CommandResult, DoctorRequest,
    WatchRequest, run, run_build_with_progress, run_watch_with_progress,
};

use crate::{
    args::{CliCommand, ParsedCli, parse_cli_args},
    exit_code::ExitCode,
    render::render_result,
    terminal::{ProgressHandle, create_progress_handle, finish_progress, handle_progress},
};

/// The fully rendered result of running the CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliRun {
    /// The process exit code.
    pub exit_code: i32,
    /// The text written to standard output.
    pub stdout: String,
    /// The text written to standard error.
    pub stderr: String,
}

/// Runs the CLI from the current process environment.
pub fn run_from_env() -> CliRun {
    let args = env::args().skip(1).collect::<Vec<_>>();
    run_cli(args)
}

/// Runs the CLI with explicit arguments, excluding the executable name.
pub fn run_cli(args: impl IntoIterator<Item = impl Into<String>>) -> CliRun {
    let parsed = match parse_cli_args(args) {
        Ok(parsed) => parsed,
        Err(error) => {
            let output = ensure_trailing_newline(error.to_string());
            let exit_code = match error.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
                    ExitCode::Success as i32
                }
                _ => ExitCode::Failure as i32,
            };
            if exit_code == ExitCode::Success as i32 {
                return CliRun {
                    exit_code,
                    stdout: output,
                    stderr: String::new(),
                };
            }
            return CliRun {
                exit_code,
                stdout: String::new(),
                stderr: output,
            };
        }
    };

    let command = parsed.command.clone();
    let progress = create_progress_handle(&command);
    let result = run_command(parsed, progress.as_ref());
    if let Some(progress) = &progress {
        finish_progress(progress);
    }
    let exit_code = ExitCode::from_result(&command, &result) as i32;
    let rendered = render_result(&command, &result);

    let (stdout, stderr) = split_output(exit_code, &command, &result, rendered);
    CliRun {
        exit_code,
        stdout,
        stderr,
    }
}

fn run_command(parsed: ParsedCli, progress: Option<&ProgressHandle>) -> CommandResult {
    let cwd = parsed
        .options
        .root
        .clone()
        .unwrap_or_else(|| env::current_dir().expect("current directory must be available"));

    match parsed.command {
        CliCommand::Build => {
            let request = BuildRequest {
                cwd,
                fail_fast: parsed.options.fail_fast,
                jobs: parsed.options.jobs,
            };
            if let Some(progress) = progress {
                run_build_with_progress(request, |event| handle_progress(progress, event))
            } else {
                run(CommandRequest::Build(request))
            }
        }
        CliCommand::Clean => run(CommandRequest::Clean(CleanRequest { cwd })),
        CliCommand::Check => run(CommandRequest::Check(CheckRequest {
            cwd,
            fail_fast: parsed.options.fail_fast,
            jobs: parsed.options.jobs,
        })),
        CliCommand::Doctor => run(CommandRequest::Doctor(DoctorRequest { cwd })),
        CliCommand::Watch => {
            let request = WatchRequest {
                cwd,
                fail_fast: parsed.options.fail_fast,
                jobs: parsed.options.jobs,
                poll_interval_ms: parsed.options.poll_interval_ms,
                max_cycles: parsed.options.max_cycles,
            };
            if let Some(progress) = progress {
                run_watch_with_progress(request, |event| handle_progress(progress, event))
            } else {
                run(CommandRequest::Watch(request))
            }
        }
    }
}

fn split_output(
    exit_code: i32,
    command: &CliCommand,
    result: &CommandResult,
    rendered: String,
) -> (String, String) {
    if exit_code == ExitCode::Success as i32 {
        return (rendered, String::new());
    }

    if matches!(command, CliCommand::Check)
        && !result.has_errors()
        && result.checked_libraries.iter().any(|library| library.stale)
    {
        return (String::new(), rendered);
    }

    (String::new(), rendered)
}

fn ensure_trailing_newline(mut text: String) -> String {
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use dust_diagnostics::Diagnostic;
    use dust_driver::{CheckedLibrary, CommandResult};

    use super::*;

    #[test]
    fn split_output_keeps_success_on_stdout() {
        let rendered = "build result\n".to_owned();
        let (stdout, stderr) = split_output(
            ExitCode::Success as i32,
            &CliCommand::Build,
            &CommandResult::default(),
            rendered.clone(),
        );

        assert_eq!(stdout, rendered);
        assert!(stderr.is_empty());
    }

    #[test]
    fn split_output_routes_stale_check_to_stderr() {
        let result = CommandResult {
            checked_libraries: vec![CheckedLibrary {
                source_path: PathBuf::from("lib/user.dart"),
                output_path: PathBuf::from("lib/user.g.dart"),
                stale: true,
                cached: false,
            }],
            ..CommandResult::default()
        };

        let (stdout, stderr) = split_output(
            ExitCode::Stale as i32,
            &CliCommand::Check,
            &result,
            "check stale\n".to_owned(),
        );

        assert!(stdout.is_empty());
        assert_eq!(stderr, "check stale\n");
    }

    #[test]
    fn split_output_routes_real_errors_to_stderr() {
        let result = CommandResult {
            diagnostics: vec![Diagnostic::error("broken")],
            ..CommandResult::default()
        };

        let (stdout, stderr) = split_output(
            ExitCode::Failure as i32,
            &CliCommand::Build,
            &result,
            "error output\n".to_owned(),
        );

        assert!(stdout.is_empty());
        assert_eq!(stderr, "error output\n");
    }

    #[test]
    fn ensure_trailing_newline_appends_once() {
        assert_eq!(ensure_trailing_newline("hello".to_owned()), "hello\n");
        assert_eq!(ensure_trailing_newline("hello\n".to_owned()), "hello\n");
    }
}
