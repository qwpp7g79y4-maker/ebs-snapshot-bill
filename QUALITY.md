# Quality

Tested commit: a74c727 (the code and tests; later commits only touch documents).

- Tests: 68 passed, 0 failed, 0 ignored (30 lib, 2 main, 6 claims, 30 cli; `cargo test --locked --no-fail-fast`, 2026-11-24, debug build, Linux).
- Red before the fix, as measured: the source of d84fb31 with the tests of a74c727 gave `test result: FAILED. 23 passed; 7 failed; 0 ignored; 0 measured; 0 filtered out` for `tests/cli.rs`. The 7 failing tests: `stderr_snapshot_id_clean`, `stderr_volume_size_clean`, `stderr_start_time_clean`, `stderr_path_clean`, `duplicate_keys_exit_2`, `duplicate_price_keys_exit_2`, `duplicate_snapshot_id_exit_2`.
- The 3 new tests for `--as-of`, the age limit and directory input (`as_of_changes_age_and_old`, `age_equal_to_max_is_not_old`, `directory_input_is_read`) pass on d84fb31 too: they pin existing behaviour. Measured on a copy: replacing `as_of: cli.as_of` by a fixed date, `if path.is_dir()` by `if false && path.is_dir()`, or `age > max_age_days` by `>=` makes them fail.
- Also run: `cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings` (stable), both clean.
- Version 0.1.0. `rust-version` 1.85.

This page records checks. It is not a guarantee and it is not a fuzz report.

## Not run

Not run: clippy and tests on Rust 1.85, cargo deny, cargo audit, cargo publish --dry-run, a GitHub Actions run, fuzzing, and mutation testing (`cargo mutants`). The CI file lists several of these; they have not run on GitHub for this tree, because this tree is not pushed.
