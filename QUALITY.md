# Quality

pep: 8.6

Tested commit: ee67a3f (the code and tests; later commits only touch documents).
Date: 2026-11-25. Platform: Linux. Toolchain: stable, rustc 1.99.0, cargo 1.99.0.

- `cargo fmt --check`: clean (exit 0).
- `cargo clippy --all-targets --locked -- -D warnings`: clean (exit 0).
- `cargo test --locked` (debug build): 70 passed, 0 failed, 0 ignored (30 lib, 2 main, 6 claims, 32 cli, 0 doc).
- `cargo test --release --locked`: 70 passed, 0 failed, 0 ignored (30 lib, 2 main, 6 claims, 32 cli, 0 doc).
- Tests: 70 passed, 0 failed (`cargo test --locked`).
- Rust 1.85 (2026-11-25, rustc/cargo 1.85.1, run on 0c8d3f1, same code as ee67a3f): `cargo +1.85 fmt --check` clean (exit 0); `cargo +1.85 clippy --all-targets --locked -- -D warnings` clean (exit 0); `cargo +1.85 test --locked` (debug build): 70 passed, 0 failed (30 lib, 2 main, 6 claims, 32 cli, 0 doc). Release build on 1.85 (`cargo +1.85 test --release --locked`, 2026-11-25, exit 0): 70 passed, 0 failed, 0 ignored (30 lib, 2 main, 6 claims, 32 cli, 0 doc).
- Version 0.1.0. `rust-version` 1.85.

This page records checks. It is not a guarantee and it is not a fuzz report.

## Not run

- Red-before-the-fix measurement: the new test `closed_stdout_is_exit_2` failed before the fix (1 failed, 1 passed of the 2 new stdout tests) and passes at ee67a3f.
- Mutation check of the tests on a copy: not run in this measurement.
- cargo deny: not run.
- cargo audit: not run.
- cargo publish --dry-run: not run.
- GitHub Actions run: not run.
- Fuzzing: not run.
- Mutation testing (`cargo mutants`): not run.
