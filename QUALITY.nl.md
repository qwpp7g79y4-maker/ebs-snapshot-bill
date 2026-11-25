# Kwaliteit

Geteste commit: 0405bf7 (de code en de tests; latere commits raken alleen documenten).
Datum: 2026-11-25. Platform: Linux. Toolchain: stable, rustc 1.99.0, cargo 1.99.0.

- `cargo fmt --check`: schoon (exit 0).
- `cargo clippy --all-targets --locked -- -D warnings`: schoon (exit 0).
- `cargo test --locked` (debugbuild): 68 geslaagd, 0 mislukt, 0 genegeerd (30 lib, 2 main, 6 claims, 30 cli, 0 doc).
- `cargo test --release --locked`: 68 geslaagd, 0 mislukt, 0 genegeerd (30 lib, 2 main, 6 claims, 30 cli, 0 doc).
- Versie 0.1.0. `rust-version` 1.85.

Deze pagina legt controles vast. Het is geen garantie en geen fuzzrapport.

## Niet uitgevoerd

- Rood-voor-de-reparatie-meting: not run in deze meting.
- Mutatiecontrole van de tests op een kopie: not run in deze meting.
- Clippy en tests op Rust 1.85: not run.
- cargo deny: not run.
- cargo audit: not run.
- cargo publish --dry-run: not run.
- GitHub Actions-run: not run.
- Fuzzing: not run.
- Mutatietests (`cargo mutants`): not run.
