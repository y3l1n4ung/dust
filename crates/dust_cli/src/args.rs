use std::path::PathBuf;

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

/// Parses Dust CLI arguments, excluding the executable name.
pub fn parse_cli_args(
    args: impl IntoIterator<Item = impl Into<String>>,
) -> Result<ParsedCli, String> {
    let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    if args.is_empty() {
        return Ok(ParsedCli {
            command: CliCommand::Help,
            options: CliOptions::default(),
        });
    }

    let command = match args[0].as_str() {
        "build" => CliCommand::Build,
        "clean" => CliCommand::Clean,
        "check" => CliCommand::Check,
        "doctor" => CliCommand::Doctor,
        "watch" => CliCommand::Watch,
        "help" | "-h" | "--help" => CliCommand::Help,
        other => return Err(format!("unknown command `{other}`")),
    };

    let mut options = CliOptions::default();
    let mut index = 1;

    while index < args.len() {
        match args[index].as_str() {
            "--root" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "missing value for `--root`".to_owned())?;
                options.root = Some(PathBuf::from(value));
            }
            "--fail-fast" => options.fail_fast = true,
            "--poll-ms" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "missing value for `--poll-ms`".to_owned())?;
                options.poll_interval_ms = value
                    .parse::<u64>()
                    .map_err(|_| format!("invalid integer for `--poll-ms`: `{value}`"))?;
            }
            "--max-cycles" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "missing value for `--max-cycles`".to_owned())?;
                options.max_cycles = Some(
                    value
                        .parse::<u32>()
                        .map_err(|_| format!("invalid integer for `--max-cycles`: `{value}`"))?,
                );
            }
            "-h" | "--help" => {
                return Ok(ParsedCli {
                    command: CliCommand::Help,
                    options,
                });
            }
            other => return Err(format!("unknown flag `{other}`")),
        }

        index += 1;
    }

    Ok(ParsedCli { command, options })
}
