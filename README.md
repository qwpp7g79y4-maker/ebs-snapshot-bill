# ebs-snapshot-bill

## Why

Estimate **Amazon EBS snapshot** GB-month cost from a local `describe-snapshots`
JSON export, and list old or orphan snapshots. Separate from `ebs-wees-bill`
(which looks at volumes).

It works offline: it reads local files only and makes no network calls.
A Dutch version is in `README.nl.md`.

The tests and the measured numbers are in `QUALITY.md`.

## Installation

Version 0.1.0. From crates.io, once 0.1.0 is published there:

```sh
cargo install ebs-snapshot-bill
```

Or build from source.

Rust 1.85+. From a checkout:

```sh
cargo install --path .
```

## Quick start

```sh
ebs-snapshot-bill examples/normal.json
ebs-snapshot-bill examples/old.json --max-age-days 30
ebs-snapshot-bill examples/normal.json --json
```

## Quickstart in 5 minutes

1. Build or install the tool (see Installation).
2. Run `ebs-snapshot-bill examples/normal.json`.
3. Add `--max-age-days 30` to list older snapshots, or `--json` for JSON.
4. Export your own with `aws ec2 describe-snapshots --owner-ids self > snaps.json`
   and run the tool on that file. The Dutch guide is in `README.nl.md`.

## Options

- `PATH...`: describe-snapshots JSON files or a directory of `*.json`.
- `--prices FILE`: price file. Default: built-in us-east-1 rates (`prices/us-east-1.json`).
- `--volumes FILE`: describe-volumes JSON; snapshots whose volume is missing from it are orphans. Accepted: `{"Volumes":[{"VolumeId":"..."}]}`, a top-level list `[{"VolumeId":"..."}]` (the output of `describe-volumes --query 'Volumes[]'`), or a list of id strings. An empty list, an element without a string `VolumeId`, or a repeated `VolumeId` or `Volumes` key is an error (exit 2). The same goes for a repeated key anywhere in a snapshot file or the price file, and for a repeated `SnapshotId` (also when the same file is given twice).
- `--as-of DATE`: reference date for age, `YYYY-MM-DD`. Default 2026-11-09.
- `--max-age-days DAYS`: snapshots older than this are listed as old. Default 90.
- `--json`: print JSON (`schema_version` 1).

## Example

```sh
ebs-snapshot-bill examples/old.json --max-age-days 30 --volumes tests/fixtures/volumes.json
```

The report lists the estimated GB-month cost, old and orphan snapshots, and a
by_volume roll-up.

## How it counts

- Input: `describe-snapshots` JSON (`Snapshots[]`) as in
  [DescribeSnapshots](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeSnapshots.html)
  (geraadpleegd 2026-11-09).
- Cost: `VolumeSize` (GiB) × rate. Built-in us-east-1 rates from
  [Amazon EBS pricing](https://aws.amazon.com/ebs/pricing/) (geraadpleegd
  2026-11-09): **standard $0.05** / GB-month, **archive $0.0125** / GB-month.
  Incremental savings are not modeled; this is a full-size upper-bound estimate.
- **Old:** `StartTime` older than `--max-age-days` relative to `--as-of`.
- **Orphan:** missing/`vol-ffffffff` volume id, or volume id absent from
  optional `--volumes` describe-volumes JSON.
- Also prints a **by_volume** roll-up.

## Exit codes

- `0`: the report was produced.
- `2`: input or usage error (unreadable or invalid file, bad option, a price file with a rate below 0, or a rate so large that a cost does not fit, a `--volumes` file in an unknown shape, a GB total that overflows 64 bits, a repeated key, or a repeated `SnapshotId`). There is no made-up maximum: only a sum that really does not fit gives exit 2.

## Limitations

- Offline only; no AWS client in v0.1.
- Does not model incremental snapshot chaining or restore fees.
- Does not modify `ebs-wees-bill`.
- Each file ≤ 32 MiB.
- Text output replaces control and invisible characters in ids and currency with `?` and puts an apostrophe before a field that starts with `=`, `+`, `-` or `@`. `--json` keeps the original text, escaped. Error messages on stderr get the same replacement, also for paths.

## Maintenance

Prices change. The rates in `prices/us-east-1.json` are a dated copy (2026-11-09),
so re-check them before you rely on a number. Fixes arrive as patches, see
`CONTRIBUTING.md`. Security reports: see `SECURITY.md`. Changes are in `CHANGELOG.md`.

## Prices and responsibility

The rates come from `prices/us-east-1.json` (source:
https://aws.amazon.com/ebs/pricing/, date 2026-11-09). This is an estimate from a
local file, not an AWS bill. You are responsible for checking the rates and the
result against your own AWS invoice before you act on them.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The suite has 68 tests (68 passed, 0 failed). See `QUALITY.md`.

## License

MIT OR Apache-2.0.

The licenses of the third-party crates built into the binary are listed in [THIRD-PARTY-LICENSES.html](THIRD-PARTY-LICENSES.html).
