use std::path::PathBuf;

use clap::error::ErrorKind;
use dust_cli::{CliCommand, parse_cli_args};

#[test]
fn parses_build_with_root_fail_fast_and_jobs() {
    let parsed =
        parse_cli_args(["build", "--root", "/tmp/work", "--fail-fast", "--jobs", "4"]).unwrap();

    assert_eq!(parsed.command, CliCommand::Build);
    assert_eq!(parsed.options.root, Some(PathBuf::from("/tmp/work")));
    assert!(parsed.options.fail_fast);
    assert_eq!(parsed.options.jobs, Some(4));
}

#[test]
fn parses_clean_with_root() {
    let parsed = parse_cli_args(["clean", "--root", "/tmp/work"]).unwrap();

    assert_eq!(parsed.command, CliCommand::Clean);
    assert_eq!(parsed.options.root, Some(PathBuf::from("/tmp/work")));
    assert!(!parsed.options.fail_fast);
    assert_eq!(parsed.options.jobs, None);
}

#[test]
fn parses_watch_specific_options() {
    let parsed = parse_cli_args([
        "watch",
        "--poll-ms",
        "25",
        "--max-cycles",
        "3",
        "--jobs",
        "2",
    ])
    .unwrap();

    assert_eq!(parsed.command, CliCommand::Watch);
    assert_eq!(parsed.options.jobs, Some(2));
    assert_eq!(parsed.options.poll_interval_ms, 25);
    assert_eq!(parsed.options.max_cycles, Some(3));
}

#[test]
fn empty_args_show_generated_help() {
    let error = parse_cli_args(Vec::<String>::new()).unwrap_err();

    assert_eq!(
        error.kind(),
        ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
    );
    assert!(
        error
            .to_string()
            .contains("Fast Dart code generation without build_runner")
    );
}

#[test]
fn rejects_unknown_command() {
    let error = parse_cli_args(["unknown"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidSubcommand);
    assert!(error.to_string().contains("unrecognized subcommand"));
}

#[test]
fn rejects_missing_flag_value() {
    let error = parse_cli_args(["watch", "--poll-ms"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidValue);
    assert!(error.to_string().contains("--poll-ms"));
    assert!(error.to_string().contains("a value is required"));
}

#[test]
fn rejects_fail_fast_on_clean() {
    let error = parse_cli_args(["clean", "--fail-fast"]).unwrap_err();

    assert_eq!(error.kind(), ErrorKind::UnknownArgument);
    assert!(error.to_string().contains("--fail-fast"));
}

#[test]
fn version_flag_uses_clap_display_path() {
    let error = parse_cli_args(["--version"]).unwrap_err();

    assert_eq!(error.kind(), ErrorKind::DisplayVersion);
    assert!(error.to_string().contains(env!("CARGO_PKG_VERSION")));
}
