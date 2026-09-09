//! The clap-derived command tree, before it becomes a ParsedCli.

use super::*;

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
pub(super) struct RawCli {
    /// Selected raw subcommand.
    #[command(subcommand)]
    pub(super) command: RawCommand,
}

/// Raw subcommands parsed by Clap before conversion to driver requests.
#[derive(Debug, Subcommand)]
pub(super) enum RawCommand {
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
    /// Route inspection utilities.
    Route(RouteCommandOptions),
    /// Run initial build and then watch for changes.
    Watch(WatchOptions),
    /// Update the installed Dust CLI binary from GitHub release artifacts.
    Upgrade(UpgradeOptions),
}

/// Options accepted by the binary upgrade command.
#[derive(Debug, Clone, PartialEq, Eq, Default, Args)]
pub(super) struct UpgradeOptions {
    /// Check whether a newer release is available without downloading assets.
    #[arg(long, default_value_t = false, conflicts_with = "dry_run")]
    pub(super) check: bool,
    /// Download and verify the selected release without replacing the binary.
    #[arg(long = "dry-run", default_value_t = false, conflicts_with = "check")]
    pub(super) dry_run: bool,
    /// Upgrade to a specific release tag such as `v0.1.3`.
    #[arg(long, value_name = "TAG")]
    pub(super) tag: Option<String>,
}

/// Options for the `db` command group.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(super) struct DbCommandOptions {
    /// Selected DB subcommand.
    #[command(subcommand)]
    pub(super) command: DbCommand,
}

/// Database subcommands parsed by Clap.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(super) enum DbCommand {
    /// Run Database generation and SQL validation.
    Build(DbBuildOptions),
}

/// Build-like options for Database generation and SQL validation.
#[derive(Debug, Clone, PartialEq, Eq, Default, Args)]
pub(super) struct DbBuildOptions {
    /// Shared workspace root option.
    #[command(flatten)]
    pub(super) root: RootOptions,
    /// Stop after the first observed worker error diagnostic.
    ///
    /// Parallel builds do not guarantee that this is the lexically first file.
    #[arg(long, default_value_t = false)]
    pub(super) fail_fast: bool,
    /// The optional parallel worker count.
    #[arg(long, value_name = "N")]
    pub(super) jobs: Option<NonZeroUsize>,
    /// Use Database offline query metadata.
    #[arg(long, default_value_t = false)]
    pub(super) offline: bool,
}

/// Options for the `i18n` command group.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(super) struct I18nCommandOptions {
    /// Selected i18n subcommand.
    #[command(subcommand)]
    pub(super) command: I18nCommand,
}

/// i18n subcommands parsed by Clap.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(super) enum I18nCommand {
    /// Reconcile static translation keys into ARB files.
    Build(I18nBuildOptions),
    /// Validate ARB files against static translation keys.
    Check(RootOptions),
    /// Scan static translation API calls.
    Scan(RootOptions),
}

/// Options for the `route` command group.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(super) struct RouteCommandOptions {
    /// Selected route subcommand.
    #[command(subcommand)]
    pub(super) command: RouteCommand,
}

/// Route inspection subcommands parsed by Clap.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub(super) enum RouteCommand {
    /// Print a generated route table.
    Table(RootOptions),
}

/// Options accepted by the writing i18n build command.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(super) struct I18nBuildOptions {
    /// Shared workspace root option.
    #[command(flatten)]
    pub(super) root: RootOptions,
    /// Update existing fallback-locale messages from current `defaultText`.
    #[arg(long = "sync-source", default_value_t = false)]
    pub(super) sync_source: bool,
    /// Preview source-locale sync without writing ARB or generated files.
    #[arg(long, requires = "sync_source", default_value_t = false)]
    pub(super) dry_run: bool,
}

/// Options accepted only by the writing build command.
#[derive(Debug, Clone, PartialEq, Eq, Default, Args)]
pub(super) struct BuildCommandOptions {
    /// Shared build-like options.
    #[command(flatten)]
    pub(super) build: BuildOptions,
    /// Remove Dust outputs and cache before generating.
    #[arg(long, default_value_t = false)]
    pub(super) clean: bool,
}

/// Shared `--root` option group.
#[derive(Debug, Clone, PartialEq, Eq, Default, Args)]
pub(super) struct RootOptions {
    /// The workspace root override.
    #[arg(long, value_name = "PATH")]
    pub(super) root: Option<PathBuf>,
}

/// Build-like options shared by build, check, and watch.
#[derive(Debug, Clone, PartialEq, Eq, Default, Args)]
pub(super) struct BuildOptions {
    /// Shared workspace root option.
    #[command(flatten)]
    pub(super) root: RootOptions,
    /// Stop after the first observed worker error diagnostic.
    ///
    /// Parallel builds do not guarantee that this is the lexically first file.
    #[arg(long, default_value_t = false)]
    pub(super) fail_fast: bool,
    /// The optional parallel worker count.
    #[arg(long, value_name = "N")]
    pub(super) jobs: Option<NonZeroUsize>,
    /// Run only Database generation and SQL validation.
    #[arg(long, default_value_t = false)]
    pub(super) db: bool,
    /// Use Database offline query metadata.
    #[arg(long, requires = "db", default_value_t = false)]
    pub(super) offline: bool,
}

/// Watch-specific options plus build-like options.
#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub(super) struct WatchOptions {
    /// Build options applied to the initial and rebuild passes.
    #[command(flatten)]
    pub(super) build: BuildOptions,
    /// The watch poll interval in milliseconds.
    #[arg(long = "poll-ms", value_name = "MS", default_value_t = default_poll_interval())]
    pub(super) poll_interval_ms: NonZeroU64,
    /// The optional maximum number of watch cycles.
    #[arg(long = "max-cycles", value_name = "N")]
    pub(super) max_cycles: Option<NonZeroU32>,
}
