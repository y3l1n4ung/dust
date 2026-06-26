use std::{env, path::PathBuf};

use dust_driver::{
    BuildRequest, CheckRequest, CleanRequest, CommandRequest, CommandResult, DbRequestOptions,
    DoctorRequest, WatchRequest, run, run_build_with_progress, run_watch_with_progress,
};

use crate::{
    args::{CliCommand, ParsedCli, parse_cli_args, parse_cli_from_env},
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
    run_parsed_cli(parse_cli_from_env())
}

/// Runs the CLI with explicit arguments, excluding the executable name.
pub fn run_cli(args: impl IntoIterator<Item = impl Into<String>>) -> CliRun {
    run_parsed_cli(parse_cli_args(args))
}

/// Runs a parsed CLI result or converts Clap parse errors to CLI output.
fn run_parsed_cli(parsed: Result<ParsedCli, clap::Error>) -> CliRun {
    let parsed = match parsed {
        Ok(parsed) => parsed,
        Err(error) => {
            return cli_error_output(error);
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

    let (stdout, stderr) = split_output(exit_code, rendered);
    CliRun {
        exit_code,
        stdout,
        stderr,
    }
}

/// Executes one parsed CLI command through the driver.
fn run_command(parsed: ParsedCli, progress: Option<&ProgressHandle>) -> CommandResult {
    let cwd = command_root(&parsed);

    match parsed.command {
        CliCommand::Build => {
            if parsed.options.clean {
                let clean = run(CommandRequest::Clean(CleanRequest { cwd: cwd.clone() }));
                if clean.has_errors() {
                    return clean;
                }
            }
            let request = BuildRequest {
                cwd,
                fail_fast: parsed.options.fail_fast,
                jobs: parsed.options.jobs,
                db: db_options(&parsed),
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
            db: db_options(&parsed),
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

/// Converts parsed DB flags into driver DB request options.
fn db_options(parsed: &ParsedCli) -> DbRequestOptions {
    DbRequestOptions {
        only_db: parsed.options.db,
        offline: parsed.options.db_offline,
    }
}

/// Routes rendered output to stdout for success and stderr otherwise.
fn split_output(exit_code: i32, rendered: String) -> (String, String) {
    if exit_code == ExitCode::Success as i32 {
        return (rendered, String::new());
    }

    (String::new(), rendered)
}

/// Resolves the command root, defaulting to the current directory.
fn command_root(parsed: &ParsedCli) -> PathBuf {
    parsed
        .options
        .root
        .clone()
        .unwrap_or_else(|| env::current_dir().expect("current directory must be available"))
}

/// Converts a Clap error into process output and exit code.
fn cli_error_output(error: clap::Error) -> CliRun {
    let output = ensure_trailing_newline(error.to_string());
    match error.kind() {
        clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => CliRun {
            exit_code: ExitCode::Success as i32,
            stdout: output,
            stderr: String::new(),
        },
        _ => CliRun {
            exit_code: ExitCode::Failure as i32,
            stdout: String::new(),
            stderr: output,
        },
    }
}

/// Ensures Clap-rendered output ends with exactly at least one newline.
fn ensure_trailing_newline(mut text: String) -> String {
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod tests {
    use dust_diagnostics::Diagnostic;
    use dust_driver::CommandResult;

    use super::*;

    #[test]
    fn split_output_keeps_success_on_stdout() {
        let rendered = "build result\n".to_owned();
        let (stdout, stderr) = split_output(ExitCode::Success as i32, rendered.clone());

        assert_eq!(stdout, rendered);
        assert!(stderr.is_empty());
    }

    #[test]
    fn split_output_routes_stale_check_to_stderr() {
        let (stdout, stderr) = split_output(ExitCode::Stale as i32, "check stale\n".to_owned());

        assert!(stdout.is_empty());
        assert_eq!(stderr, "check stale\n");
    }

    #[test]
    fn split_output_routes_real_errors_to_stderr() {
        let result = CommandResult {
            diagnostics: vec![Diagnostic::error("broken")],
            ..CommandResult::default()
        };

        let exit_code = ExitCode::from_result(&CliCommand::Build, &result) as i32;
        let (stdout, stderr) = split_output(exit_code, "error output\n".to_owned());

        assert!(stdout.is_empty());
        assert_eq!(stderr, "error output\n");
    }

    #[test]
    fn ensure_trailing_newline_appends_once() {
        assert_eq!(ensure_trailing_newline("hello".to_owned()), "hello\n");
        assert_eq!(ensure_trailing_newline("hello\n".to_owned()), "hello\n");
    }
}
