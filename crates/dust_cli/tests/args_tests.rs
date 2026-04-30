use std::path::PathBuf;

use dust_cli::{CliCommand, parse_cli_args};

#[test]
fn parses_build_with_root_and_fail_fast() {
    let parsed = parse_cli_args(["build", "--root", "/tmp/work", "--fail-fast"]).unwrap();

    assert_eq!(parsed.command, CliCommand::Build);
    assert_eq!(parsed.options.root, Some(PathBuf::from("/tmp/work")));
    assert!(parsed.options.fail_fast);
}

#[test]
fn parses_clean_with_root() {
    let parsed = parse_cli_args(["clean", "--root", "/tmp/work"]).unwrap();

    assert_eq!(parsed.command, CliCommand::Clean);
    assert_eq!(parsed.options.root, Some(PathBuf::from("/tmp/work")));
    assert!(!parsed.options.fail_fast);
}

#[test]
fn parses_watch_specific_options() {
    let parsed = parse_cli_args(["watch", "--poll-ms", "25", "--max-cycles", "3"]).unwrap();

    assert_eq!(parsed.command, CliCommand::Watch);
    assert_eq!(parsed.options.poll_interval_ms, 25);
    assert_eq!(parsed.options.max_cycles, Some(3));
}

#[test]
fn empty_args_show_help() {
    let parsed = parse_cli_args(Vec::<String>::new()).unwrap();
    assert_eq!(parsed.command, CliCommand::Help);
}

#[test]
fn rejects_unknown_command() {
    let error = parse_cli_args(["unknown"]).unwrap_err();
    assert!(error.contains("unrecognized subcommand"));
}

#[test]
fn rejects_missing_flag_value() {
    let error = parse_cli_args(["watch", "--poll-ms"]).unwrap_err();
    assert!(error.contains("--poll-ms"));
    assert!(error.contains("required"));
}
