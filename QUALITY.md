# Quality

Tested commit: 0405bf7 (the code and tests; later commits only touch documents).
Date: 2026-11-25. Platform: Linux. Toolchain: stable, rustc 1.99.0, cargo 1.99.0.

- `cargo fmt --check`: clean (exit 0).
- `cargo clippy --all-targets --locked -- -D warnings`: clean (exit 0).
- `cargo test --locked` (debug build): 68 passed, 0 failed, 0 ignored (30 lib, 2 main, 6 claims, 30 cli, 0 doc).
- `cargo test --release --locked`: 68 passed, 0 failed, 0 ignored (30 lib, 2 main, 6 claims, 30 cli, 0 doc).
- Version 0.1.0. `rust-version` 1.85.

This page records checks. It is not a guarantee and it is not a fuzz report.

## Not run

- Red-before-the-fix measurement: not run in this measurement.
- Mutation check of the tests on a copy: not run in this measurement.
- Clippy and tests on Rust 1.85: not run.
- cargo deny: not run.
- cargo audit: not run.
- cargo publish --dry-run: not run.
- GitHub Actions run: not run.
- Fuzzing: not run.
- Mutation testing (`cargo mutants`): not run.
