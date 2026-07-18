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
    /// Run a writing Database build.
    DbBuild,
    /// Report workspace and plugin readiness.
    Doctor,
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

#[derive(Debug, Parser)]
#[command(
    name = "dust",
    version,
    about = "Rust-powered code generation for Dart and Flutter",
    long_about = None,
    arg_required_else_help = true,
    propagate_version = true,
    after_help = "Examples:\n  dust build\n  dust db build\n  dust check --fail-fast\n  dust watch --poll-ms 100 --jobs 4\n  dust upgrade --check"
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
    Build(BuildCommandOptions),
    /// Remove Dust-generated outputs and cache state.
    Clean(RootOptions),
    /// Run a no-write freshness check.
    Check(BuildOptions),
    /// Report workspace and plugin readiness.
    Doctor(RootOptions),
    /// Database utilities.
    Db(DbCommandOptions),
    /// i18n utilities.
    I18n(I18nCommandOptions),
    /// Run initial build and then watch for changes.
    Watch(WatchOptions),
    /// Update the installed Dust CLI binary from GitHub release artifacts.
    Upgrade(UpgradeOptions),
}

/// Options accepted by the binary upgrade command.
#[derive(Debug, Clone, PartialEq, Eq, Default, Args)]
struct UpgradeOptions {
    /// Check whether a newer release is available without downloading assets.
    #[arg(long, default_value_t = false, conflicts_with = "dry_run")]
    check: bool,
    /// Download and verify the selected release without replacing the binary.
    #[arg(long = "dry-run", default_value_t = false, conflicts_with = "check")]
    dry_run: bool,
    /// Upgrade to a specific release tag such as `v0.1.3`.
    #[arg(long, value_name = "TAG")]
    tag: Option<String>,
}

/// Options for the `db` command group.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
struct DbCommandOptions {
    /// Selected DB subcommand.
    #[command(subcommand)]
    command: DbCommand,
}

/// Database subcommands parsed by Clap.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
enum DbCommand {
    /// Run Database generation and SQL validation.
    Build(DbBuildOptions),
}

/// Build-like options for Database generation and SQL validation.
#[derive(Debug, Clone, PartialEq, Eq, Default, Args)]
struct DbBuildOptions {
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
    /// Use Database offline query metadata.
    #[arg(long, default_value_t = false)]
    offline: bool,
}

/// Options for the `i18n` command group.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
struct I18nCommandOptions {
    /// Selected i18n subcommand.
    #[command(subcommand)]
    command: I18nCommand,
}

/// i18n subcommands parsed by Clap.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
enum I18nCommand {
    /// Reconcile static translation keys into ARB files.
    Build(I18nBuildOptions),
    /// Validate ARB files against static translation keys.
    Check(RootOptions),
    /// Scan static translation API calls.
    Scan(RootOptions),
}

/// Options accepted by the writing i18n build command.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
struct I18nBuildOptions {
    /// Shared workspace root option.
    #[command(flatten)]
    root: RootOptions,
    /// Update existing fallback-locale messages from current `defaultText`.
    #[arg(long = "sync-source", default_value_t = false)]
    sync_source: bool,
    /// Preview source-locale sync without writing ARB or generated files.
    #[arg(long, requires = "sync_source", default_value_t = false)]
    dry_run: bool,
}

/// Options accepted only by the writing build command.
#[derive(Debug, Clone, PartialEq, Eq, Default, Args)]
struct BuildCommandOptions {
    /// Shared build-like options.
    #[command(flatten)]
    build: BuildOptions,
    /// Remove Dust outputs and cache before generating.
    #[arg(long, default_value_t = false)]
    clean: bool,
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
    /// Run only Database generation and SQL validation.
    #[arg(long, default_value_t = false)]
    db: bool,
    /// Use Database offline query metadata.
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
            RawCommand::Db(options) => options.into(),
            RawCommand::Doctor(options) => ParsedCli::new(CliCommand::Doctor, options),
            RawCommand::I18n(options) => options.into(),
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
