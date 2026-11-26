# Kwaliteit

pep: 8.6

Geteste commit: ee67a3f (de code en de tests; latere commits raken alleen documenten).
Datum: 2026-11-25. Platform: Linux. Toolchain: stable, rustc 1.99.0, cargo 1.99.0.

- `cargo fmt --check`: schoon (exit 0).
- `cargo clippy --all-targets --locked -- -D warnings`: schoon (exit 0).
- `cargo test --locked` (debugbuild): 70 geslaagd, 0 mislukt, 0 genegeerd (30 lib, 2 main, 6 claims, 32 cli, 0 doc).
- `cargo test --release --locked`: 70 geslaagd, 0 mislukt, 0 genegeerd (30 lib, 2 main, 6 claims, 32 cli, 0 doc).
- Rust 1.85 (2026-11-25, rustc/cargo 1.85.1, gedraaid op 0c8d3f1, dezelfde code als ee67a3f): `cargo +1.85 fmt --check` schoon (exit 0); `cargo +1.85 clippy --all-targets --locked -- -D warnings` schoon (exit 0); `cargo +1.85 test --locked` (debugbuild): 70 geslaagd, 0 mislukt (30 lib, 2 main, 6 claims, 32 cli, 0 doc). Releasebuild op 1.85 (`cargo +1.85 test --release --locked`, 2026-11-25, exit 0): 70 geslaagd, 0 mislukt, 0 genegeerd (30 lib, 2 main, 6 claims, 32 cli, 0 doc).
- Versie 0.1.0. `rust-version` 1.85.

Deze pagina legt controles vast. Het is geen garantie en geen fuzzrapport.

## Niet uitgevoerd

- Rood-voor-de-reparatie-meting: de nieuwe test `closed_stdout_is_exit_2` faalde voor de reparatie (1 mislukt, 1 geslaagd van de 2 nieuwe stdout-tests) en slaagt op ee67a3f.
- Mutatiecontrole van de tests op een kopie: not run in deze meting.
- cargo deny: not run.
- cargo audit: not run.
- cargo publish --dry-run: not run.
- GitHub Actions-run: not run.
- Fuzzing: not run.
- Mutatietests (`cargo mutants`): not run.
