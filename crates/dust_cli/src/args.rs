use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// One supported Dust CLI command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
    /// Run a writing build.
    Build,
    /// Remove Dust-generated outputs and cache state.
    Clean,
    /// Run a no-write freshness check.
    Check,
    /// Report workspace and plugin readiness.
    Doctor,
    /// Run initial build and then watch for changes.
    Watch,
    /// Print help text.
    Help,
}

/// Shared CLI options understood by Dust commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliOptions {
    /// The workspace root override.
    pub root: Option<PathBuf>,
    /// Whether the command should stop after the first error diagnostic.
    pub fail_fast: bool,
    /// The watch poll interval in milliseconds.
    pub poll_interval_ms: u64,
    /// The optional maximum number of watch cycles.
    pub max_cycles: Option<u32>,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            root: None,
            fail_fast: false,
            poll_interval_ms: 250,
            max_cycles: None,
        }
    }
}

/// The parsed CLI command plus its options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCli {
    /// The selected command.
    pub command: CliCommand,
    /// The parsed command options.
    pub options: CliOptions,
}

#[derive(Debug, Parser)]
#[command(
    name = "dust",
    no_binary_name = true,
    disable_help_subcommand = true,
    disable_version_flag = true
)]
struct RawCli {
    #[command(subcommand)]
    command: Option<RawCommand>,
}

#[derive(Debug, Subcommand)]
enum RawCommand {
    /// Run a writing build.
    Build(CommonOptions),
    /// Remove Dust-generated outputs and cache state.
    Clean(CommonOptions),
    /// Run a no-write freshness check.
    Check(CommonOptions),
    /// Report workspace and plugin readiness.
    Doctor(CommonOptions),
    /// Run initial build and then watch for changes.
    Watch(WatchOptions),
    /// Print help text.
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Args)]
struct CommonOptions {
    /// The workspace root override.
    #[arg(long)]
    root: Option<PathBuf>,
    /// Whether the command should stop after the first error diagnostic.
    #[arg(long, default_value_t = false)]
    fail_fast: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
struct WatchOptions {
    #[command(flatten)]
    common: CommonOptions,
    /// The watch poll interval in milliseconds.
    #[arg(long = "poll-ms", default_value_t = 250)]
    poll_interval_ms: u64,
    /// The optional maximum number of watch cycles.
    #[arg(long = "max-cycles")]
    max_cycles: Option<u32>,
}

/// Parses Dust CLI arguments, excluding the executable name.
pub fn parse_cli_args(
    args: impl IntoIterator<Item = impl Into<String>>,
) -> Result<ParsedCli, String> {
    let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    let parsed = match RawCli::try_parse_from(args) {
        Ok(parsed) => parsed,
        Err(error) if error.kind() == clap::error::ErrorKind::DisplayHelp => {
            return Ok(ParsedCli {
                command: CliCommand::Help,
                options: CliOptions::default(),
            });
        }
        Err(error) => return Err(format_clap_error(&error)),
    };

    Ok(match parsed.command {
        Some(RawCommand::Build(options)) => ParsedCli {
            command: CliCommand::Build,
            options: build_options(options),
        },
        Some(RawCommand::Clean(options)) => ParsedCli {
            command: CliCommand::Clean,
            options: build_options(options),
        },
        Some(RawCommand::Check(options)) => ParsedCli {
            command: CliCommand::Check,
            options: build_options(options),
        },
        Some(RawCommand::Doctor(options)) => ParsedCli {
            command: CliCommand::Doctor,
            options: build_options(options),
        },
        Some(RawCommand::Watch(options)) => ParsedCli {
            command: CliCommand::Watch,
            options: CliOptions {
                root: options.common.root,
                fail_fast: options.common.fail_fast,
                poll_interval_ms: options.poll_interval_ms,
                max_cycles: options.max_cycles,
            },
        },
        Some(RawCommand::Help) | None => ParsedCli {
            command: CliCommand::Help,
            options: CliOptions::default(),
        },
    })
}

fn build_options(options: CommonOptions) -> CliOptions {
    CliOptions {
        root: options.root,
        fail_fast: options.fail_fast,
        ..CliOptions::default()
    }
}

fn format_clap_error(error: &clap::Error) -> String {
    error
        .to_string()
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(str::trim)
        .unwrap_or("invalid arguments")
        .to_owned()
}
