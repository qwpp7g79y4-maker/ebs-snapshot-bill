# Changelog

## [0.1.0] - 2026-11-09

Changes made after 2026-11-09; the version number did not change.

- Fix: text output replaces control and invisible characters with `?` and prefixes fields that start with `=`, `+`, `-` or `@` with an apostrophe (JSON unchanged).
- Fix: `--volumes` accepts a top-level `[{"VolumeId":...}]` list and exits 2 on an empty list, an element without a string `VolumeId`, or a duplicate key, instead of marking everything orphan.
- Fix: total and per-volume GB use checked addition and exit 2 on overflow instead of sticking at `u64::MAX`.
- Fix: a price file with a rate below 0, or a rate too large for the cost, now exits 2 instead of panicking (exit 101).
- Fix: every error message is cleaned in one place (`Error::msg`), so ids, sizes, times and paths from input no longer put raw escapes or fake lines on stderr.
- Fix: a repeated key in a snapshot object or the price file, and a repeated SnapshotId (also the same file twice), now exit 2 instead of counting the last value or both snapshots.
- Tests: `--as-of`, the `age == --max-age-days` limit and directory input are now pinned by tests.
- Tests: old, orphan and volumes tests check the real row.
- Docs: Dutch README, SECURITY, CONTRIBUTING, QUALITY.nl.md, price and maintenance sections.


- First release: offline EBS snapshot GB-month estimate from describe-snapshots JSON.
- Old and orphan lists; by-volume roll-up; standard/archive rates; `--json`.
