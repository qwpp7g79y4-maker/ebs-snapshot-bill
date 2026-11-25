use clap::Parser;
use ebs_snapshot_bill::{run, Options};
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::process::ExitCode;

/// Offline estimate of AWS EBS snapshot GB-month cost from describe-snapshots JSON. Files only, no network.
#[derive(Parser, Debug)]
#[command(
    name = "ebs-snapshot-bill",
    version,
    about = "Offline estimate of AWS EBS snapshot GB-month cost from describe-snapshots JSON. Files only, no network.",
    after_help = "Exit codes: 0 ok, 2 input or usage error.\nPricing: https://aws.amazon.com/ebs/pricing/ (geraadpleegd 2026-11-09)."
)]
struct Cli {
    /// describe-snapshots JSON files or a directory of *.json.
    #[arg(required = true, value_name = "PATH")]
    inputs: Vec<PathBuf>,
    /// Price file. Default: built-in us-east-1 rates from the EBS pricing page (2026-11-09).
    #[arg(long, value_name = "FILE")]
    prices: Option<PathBuf>,
    /// Optional describe-volumes JSON; snapshots whose VolumeId is missing from it are orphans.
    #[arg(long, value_name = "FILE")]
    volumes: Option<PathBuf>,
    /// Reference date for age (YYYY-MM-DD). Default 2026-11-09.
    #[arg(long, value_name = "DATE", default_value = "2026-11-09")]
    as_of: String,
    /// Snapshots older than this many days are listed as old. Default 90.
    #[arg(long, value_name = "DAYS", default_value_t = 90)]
    max_age_days: u64,
    /// Print JSON (`schema_version` 1).
    #[arg(long)]
    json: bool,
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            let code = if e.use_stderr() { 2 } else { 0 };
            let _ = e.print();
            return ExitCode::from(code);
        }
    };
    match run(&Options {
        inputs: cli.inputs,
        prices: cli.prices,
        volumes: cli.volumes,
        as_of: cli.as_of,
        max_age_days: cli.max_age_days,
        json: cli.json,
    }) {
        Ok(text) => {
            let mut out = stdout().lock();
            if out.write_all(text.as_bytes()).is_err() || out.flush().is_err() {
                let _ = writeln!(
                    std::io::stderr(),
                    "ebs-snapshot-bill: could not write output"
                );
                return ExitCode::from(2);
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            let _ = writeln!(std::io::stderr(), "ebs-snapshot-bill: {err}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;
    #[test]
    fn cli_builds() {
        crate::Cli::command().debug_assert();
    }
    #[test]
    fn help_mentions_flags() {
        let mut cmd = crate::Cli::command();
        let mut buf = Vec::new();
        cmd.write_long_help(&mut buf).unwrap();
        let h = String::from_utf8(buf).unwrap();
        assert!(h.contains("--max-age-days"));
        assert!(h.contains("--volumes"));
        assert!(h.contains("ebs/pricing"));
    }
}
