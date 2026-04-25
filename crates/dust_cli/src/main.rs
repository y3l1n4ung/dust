use std::process::ExitCode;

fn main() -> ExitCode {
    let run = dust_cli::run_from_env();

    if !run.stdout.is_empty() {
        print!("{}", run.stdout);
    }
    if !run.stderr.is_empty() {
        eprint!("{}", run.stderr);
    }

    ExitCode::from(run.exit_code as u8)
}
