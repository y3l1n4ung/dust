use std::{
    num::{NonZeroU32, NonZeroU64, NonZeroUsize},
    path::PathBuf,
};

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
}

/// Shared CLI options understood by Dust commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliOptions {
    /// The workspace root override.
    pub root: Option<PathBuf>,
    /// Whether the command should stop after the first error diagnostic.
    pub fail_fast: bool,
    /// The optional parallel worker count for build/check/watch.
    pub jobs: Option<usize>,
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
            jobs: None,
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
    version,
    about = "Fast Dart code generation without build_runner",
    long_about = None,
    arg_required_else_help = true,
    propagate_version = true,
    after_help = "Examples:\n  dust build\n  dust check --fail-fast\n  dust watch --poll-ms 100 --jobs 4"
)]
struct RawCli {
    #[command(subcommand)]
    command: RawCommand,
}

#[derive(Debug, Subcommand)]
enum RawCommand {
    /// Run a writing build.
    Build(BuildOptions),
    /// Remove Dust-generated outputs and cache state.
    Clean(RootOptions),
    /// Run a no-write freshness check.
    Check(BuildOptions),
    /// Report workspace and plugin readiness.
    Doctor(RootOptions),
    /// Run initial build and then watch for changes.
    Watch(WatchOptions),
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Args)]
struct RootOptions {
    /// The workspace root override.
    #[arg(long, value_name = "PATH")]
    root: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Args)]
struct BuildOptions {
    #[command(flatten)]
    root: RootOptions,
    /// Whether the command should stop after the first error diagnostic.
    #[arg(long, default_value_t = false)]
    fail_fast: bool,
    /// The optional parallel worker count.
    #[arg(long, value_name = "N")]
    jobs: Option<NonZeroUsize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
struct WatchOptions {
    #[command(flatten)]
    build: BuildOptions,
    /// The watch poll interval in milliseconds.
    #[arg(long = "poll-ms", value_name = "MS", default_value = "250")]
    poll_interval_ms: NonZeroU64,
    /// The optional maximum number of watch cycles.
    #[arg(long = "max-cycles", value_name = "N")]
    max_cycles: Option<NonZeroU32>,
}

/// Parses Dust CLI arguments, excluding the executable name.
pub fn parse_cli_args(
    args: impl IntoIterator<Item = impl Into<String>>,
) -> Result<ParsedCli, clap::Error> {
    let parsed = RawCli::try_parse_from(
        std::iter::once("dust".to_owned()).chain(args.into_iter().map(Into::into)),
    )?;

    Ok(match parsed.command {
        RawCommand::Build(options) => ParsedCli {
            command: CliCommand::Build,
            options: build_options(options),
        },
        RawCommand::Clean(options) => ParsedCli {
            command: CliCommand::Clean,
            options: CliOptions {
                root: options.root,
                ..CliOptions::default()
            },
        },
        RawCommand::Check(options) => ParsedCli {
            command: CliCommand::Check,
            options: build_options(options),
        },
        RawCommand::Doctor(options) => ParsedCli {
            command: CliCommand::Doctor,
            options: CliOptions {
                root: options.root,
                ..CliOptions::default()
            },
        },
        RawCommand::Watch(options) => ParsedCli {
            command: CliCommand::Watch,
            options: CliOptions {
                root: options.build.root.root,
                fail_fast: options.build.fail_fast,
                jobs: options.build.jobs.map(NonZeroUsize::get),
                poll_interval_ms: options.poll_interval_ms.get(),
                max_cycles: options.max_cycles.map(NonZeroU32::get),
            },
        },
    })
}

fn build_options(options: BuildOptions) -> CliOptions {
    CliOptions {
        root: options.root.root,
        fail_fast: options.fail_fast,
        jobs: options.jobs.map(NonZeroUsize::get),
        ..CliOptions::default()
    }
}
