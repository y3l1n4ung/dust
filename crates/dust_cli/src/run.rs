use std::env;

use dust_driver::{
    BuildRequest, CheckRequest, CleanRequest, CommandRequest, CommandResult, DoctorRequest,
    WatchRequest, run, run_build_with_progress, run_watch_with_progress,
};

use crate::{
    args::{CliCommand, ParsedCli, parse_cli_args},
    exit_code::ExitCode,
    render::{render_help, render_result},
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
        Err(message) => {
            return CliRun {
                exit_code: ExitCode::Failure as i32,
                stdout: String::new(),
                stderr: format!("{message}\n\n{}\n", render_help()),
            };
        }
    };

    if matches!(parsed.command, CliCommand::Help) {
        return CliRun {
            exit_code: ExitCode::Success as i32,
            stdout: format!("{}\n", render_help()),
            stderr: String::new(),
        };
    }

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
                jobs: None,
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
            jobs: None,
        })),
        CliCommand::Doctor => run(CommandRequest::Doctor(DoctorRequest { cwd })),
        CliCommand::Watch => {
            let request = WatchRequest {
                cwd,
                fail_fast: parsed.options.fail_fast,
                jobs: None,
                poll_interval_ms: parsed.options.poll_interval_ms,
                max_cycles: parsed.options.max_cycles,
            };
            if let Some(progress) = progress {
                run_watch_with_progress(request, |event| handle_progress(progress, event))
            } else {
                run(CommandRequest::Watch(request))
            }
        }
        CliCommand::Help => unreachable!("help is handled before command dispatch"),
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
