use std::{
    num::{NonZeroU32, NonZeroU64, NonZeroUsize},
    path::PathBuf,
};

use clap::{Args, Parser, Subcommand};

/// The clap-derived command tree, before it becomes a ParsedCli.
mod raw;
use self::raw::*;

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
    /// Run a writing Database build.
    DbBuild,
    /// Report workspace and plugin readiness.
    Doctor,
    /// Print generated route inspection output.
    RouteTable,
    /// Reconcile scanned i18n keys into ARB assets.
    I18nBuild,
    /// Validate ARB assets against static i18n keys.
    I18nCheck,
    /// Scan static i18n API calls.
    I18nScan,
    /// Run initial build and then watch for changes.
    Watch,
    /// Update the installed Dust CLI binary from GitHub release artifacts.
    Upgrade,
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
    /// Whether only Database generation/validation should run.
    pub db: bool,
    /// Whether Database should use offline query metadata only.
    pub db_offline: bool,
    /// Whether build should remove Dust outputs and cache before generation.
    pub clean: bool,
    /// The watch poll interval in milliseconds.
    pub poll_interval_ms: u64,
    /// The optional maximum number of watch cycles.
    pub max_cycles: Option<u32>,
    /// Whether i18n build may sync existing fallback-locale messages.
    pub i18n_sync_source: bool,
    /// Whether i18n source sync should only preview planned writes.
    pub i18n_dry_run: bool,
    /// Whether upgrade should only check for an available release.
    pub upgrade_check: bool,
    /// Whether upgrade should verify the selected release without replacing the binary.
    pub upgrade_dry_run: bool,
    /// Explicit GitHub release tag selected for upgrade.
    pub upgrade_tag: Option<String>,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            root: None,
            fail_fast: false,
            jobs: None,
            db: false,
            db_offline: false,
            clean: false,
            poll_interval_ms: DEFAULT_POLL_INTERVAL_MS,
            max_cycles: None,
            i18n_sync_source: false,
            i18n_dry_run: false,
            upgrade_check: false,
            upgrade_dry_run: false,
            upgrade_tag: None,
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
            RawCommand::Db(options) => options.into(),
            RawCommand::Doctor(options) => ParsedCli::new(CliCommand::Doctor, options),
            RawCommand::I18n(options) => options.into(),
            RawCommand::Route(options) => options.into(),
            RawCommand::Watch(options) => ParsedCli::new(CliCommand::Watch, options),
            RawCommand::Upgrade(options) => ParsedCli::new(CliCommand::Upgrade, options),
        }
    }
}

impl From<DbCommandOptions> for ParsedCli {
    fn from(value: DbCommandOptions) -> Self {
        match value.command {
            DbCommand::Build(options) => ParsedCli::new(CliCommand::DbBuild, options),
        }
    }
}

impl From<I18nCommandOptions> for ParsedCli {
    fn from(value: I18nCommandOptions) -> Self {
        match value.command {
            I18nCommand::Build(options) => ParsedCli::new(CliCommand::I18nBuild, options),
            I18nCommand::Check(options) => ParsedCli::new(CliCommand::I18nCheck, options),
            I18nCommand::Scan(options) => ParsedCli::new(CliCommand::I18nScan, options),
        }
    }
}

impl From<RouteCommandOptions> for ParsedCli {
    fn from(value: RouteCommandOptions) -> Self {
        match value.command {
            RouteCommand::Table(options) => ParsedCli::new(CliCommand::RouteTable, options),
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

impl From<I18nBuildOptions> for CliOptions {
    fn from(value: I18nBuildOptions) -> Self {
        Self {
            root: value.root.root,
            i18n_sync_source: value.sync_source,
            i18n_dry_run: value.dry_run,
            ..Self::default()
        }
    }
}

impl From<BuildCommandOptions> for CliOptions {
    fn from(value: BuildCommandOptions) -> Self {
        Self {
            clean: value.clean,
            ..CliOptions::from(value.build)
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

impl From<UpgradeOptions> for CliOptions {
    fn from(value: UpgradeOptions) -> Self {
        Self {
            upgrade_check: value.check,
            upgrade_dry_run: value.dry_run,
            upgrade_tag: value.tag,
            ..Self::default()
        }
    }
}

impl From<DbBuildOptions> for CliOptions {
    fn from(value: DbBuildOptions) -> Self {
        Self {
            root: value.root.root,
            fail_fast: value.fail_fast,
            jobs: value.jobs.map(NonZeroUsize::get),
            db: true,
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
