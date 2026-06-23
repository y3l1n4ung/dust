use std::{
    num::{NonZeroU32, NonZeroU64, NonZeroUsize},
    path::PathBuf,
};

use clap::{Args, Parser, Subcommand};

/// Default watch polling interval in milliseconds.
const DEFAULT_POLL_INTERVAL_MS: u64 = 250;

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
    /// Whether only Dust DB generation/validation should run.
    pub db: bool,
    /// Whether Dust DB should use offline query metadata only.
    pub db_offline: bool,
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
            db: false,
            db_offline: false,
            poll_interval_ms: DEFAULT_POLL_INTERVAL_MS,
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
/// Clap-owned representation of the top-level Dust CLI.
struct RawCli {
    /// Selected raw subcommand.
    #[command(subcommand)]
    command: RawCommand,
}

/// Raw subcommands parsed by Clap before conversion to driver requests.
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

/// Shared `--root` option group.
#[derive(Debug, Clone, PartialEq, Eq, Default, Args)]
struct RootOptions {
    /// The workspace root override.
    #[arg(long, value_name = "PATH")]
    root: Option<PathBuf>,
}

/// Build-like options shared by build, check, and watch.
#[derive(Debug, Clone, PartialEq, Eq, Default, Args)]
struct BuildOptions {
    /// Shared workspace root option.
    #[command(flatten)]
    root: RootOptions,
    /// Stop after the first observed worker error diagnostic.
    ///
    /// Parallel builds do not guarantee that this is the lexically first file.
    #[arg(long, default_value_t = false)]
    fail_fast: bool,
    /// The optional parallel worker count.
    #[arg(long, value_name = "N")]
    jobs: Option<NonZeroUsize>,
    /// Run only Dust DB generation and SQL validation.
    #[arg(long, default_value_t = false)]
    db: bool,
    /// Use Dust DB offline query metadata.
    #[arg(long, requires = "db", default_value_t = false)]
    offline: bool,
}

/// Watch-specific options plus build-like options.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
struct WatchOptions {
    /// Build options applied to the initial and rebuild passes.
    #[command(flatten)]
    build: BuildOptions,
    /// The watch poll interval in milliseconds.
    #[arg(long = "poll-ms", value_name = "MS", default_value_t = default_poll_interval())]
    poll_interval_ms: NonZeroU64,
    /// The optional maximum number of watch cycles.
    #[arg(long = "max-cycles", value_name = "N")]
    max_cycles: Option<NonZeroU32>,
}

/// Parses Dust CLI arguments, excluding the executable name.
pub fn parse_cli_args(
    args: impl IntoIterator<Item = impl Into<String>>,
) -> Result<ParsedCli, clap::Error> {
    RawCli::try_parse_from(
        std::iter::once("dust".to_owned()).chain(args.into_iter().map(Into::into)),
    )
    .map(ParsedCli::from)
}

/// Parses Dust CLI arguments from the current process environment.
pub fn parse_cli_from_env() -> Result<ParsedCli, clap::Error> {
    RawCli::try_parse().map(ParsedCli::from)
}

impl From<RawCli> for ParsedCli {
    fn from(value: RawCli) -> Self {
        value.command.into()
    }
}

impl From<RawCommand> for ParsedCli {
    fn from(value: RawCommand) -> Self {
        match value {
            RawCommand::Build(options) => ParsedCli::new(CliCommand::Build, options),
            RawCommand::Clean(options) => ParsedCli::new(CliCommand::Clean, options),
            RawCommand::Check(options) => ParsedCli::new(CliCommand::Check, options),
            RawCommand::Doctor(options) => ParsedCli::new(CliCommand::Doctor, options),
            RawCommand::Watch(options) => ParsedCli::new(CliCommand::Watch, options),
        }
    }
}

impl ParsedCli {
    /// Creates parsed CLI output from a command and convertible options.
    fn new(command: CliCommand, options: impl Into<CliOptions>) -> Self {
        Self {
            command,
            options: options.into(),
        }
    }
}

/// Returns the non-zero default poll interval required by Clap.
fn default_poll_interval() -> NonZeroU64 {
    NonZeroU64::new(DEFAULT_POLL_INTERVAL_MS).expect("default poll interval must be non-zero")
}

impl From<RootOptions> for CliOptions {
    fn from(value: RootOptions) -> Self {
        Self {
            root: value.root,
            ..Self::default()
        }
    }
}

impl From<BuildOptions> for CliOptions {
    fn from(value: BuildOptions) -> Self {
        Self {
            root: value.root.root,
            fail_fast: value.fail_fast,
            jobs: value.jobs.map(NonZeroUsize::get),
            db: value.db,
            db_offline: value.offline,
            ..Self::default()
        }
    }
}

impl From<WatchOptions> for CliOptions {
    fn from(value: WatchOptions) -> Self {
        let build = CliOptions::from(value.build);
        Self {
            poll_interval_ms: value.poll_interval_ms.get(),
            max_cycles: value.max_cycles.map(NonZeroU32::get),
            ..build
        }
    }
}
