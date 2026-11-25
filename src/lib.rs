//! Offline EBS snapshot GB-month cost from `describe-snapshots` JSON.
//! Pricing: <https://aws.amazon.com/ebs/pricing/> (geraadpleegd 2026-11-09).
//! API: <https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeSnapshots.html>
//! (geraadpleegd 2026-11-09).

#![forbid(unsafe_code)]

mod age;
mod bill;
mod error;
mod parse;
mod price;
mod report;

pub use bill::{bill_snapshots, by_volume, Bill, Row};
pub use error::Error;
pub use parse::{check_unique, parse_snapshots, parse_volume_ids, Snapshot};
pub use price::Prices;
pub use report::{render_json, render_text};

use std::fs;
use std::path::{Path, PathBuf};

const FILE_CAP: u64 = 32 * 1024 * 1024;

pub struct Options {
    pub inputs: Vec<PathBuf>,
    pub prices: Option<PathBuf>,
    pub volumes: Option<PathBuf>,
    pub as_of: String,
    pub max_age_days: u64,
    pub json: bool,
}

pub fn run(options: &Options) -> Result<String, Error> {
    let prices = match &options.prices {
        Some(p) => Prices::from_json(&read_capped(p, "prices")?)?,
        None => Prices::builtin(),
    };
    let known = match &options.volumes {
        Some(p) => Some(parse_volume_ids(&read_capped(p, "volumes")?)?),
        None => None,
    };
    let mut snaps = Vec::new();
    for path in &options.inputs {
        if path.is_dir() {
            let mut files: Vec<_> = fs::read_dir(path)
                .map_err(|e| Error::msg(format!("cannot read {}: {e}", path.display())))?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
                .collect();
            files.sort();
            for f in files {
                snaps.extend(parse_snapshots(&read_capped(&f, "input")?)?);
            }
        } else {
            snaps.extend(parse_snapshots(&read_capped(path, "input")?)?);
        }
    }
    check_unique(&snaps)?;
    let bill = bill_snapshots(
        &snaps,
        &prices,
        &options.as_of,
        options.max_age_days,
        known.as_ref(),
    )?;
    if options.json {
        render_json(&bill)
    } else {
        render_text(&bill)
    }
}

fn read_capped(path: &Path, label: &str) -> Result<Vec<u8>, Error> {
    let meta = fs::metadata(path)
        .map_err(|e| Error::msg(format!("cannot read {label} {}: {e}", path.display())))?;
    if !meta.is_file() {
        return Err(Error::msg(format!(
            "{label} {} is not a regular file",
            path.display()
        )));
    }
    if meta.len() > FILE_CAP {
        return Err(Error::msg(format!(
            "{label} {} is larger than 32 MiB",
            path.display()
        )));
    }
    fs::read(path).map_err(|e| Error::msg(format!("cannot read {label} {}: {e}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn run_fixture() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/normal.json");
        let out = run(&Options {
            inputs: vec![path],
            prices: None,
            volumes: None,
            as_of: "2026-11-09".into(),
            max_age_days: 365,
            json: false,
        })
        .unwrap();
        assert!(out.contains("0.40"), "{out}");
    }
}
