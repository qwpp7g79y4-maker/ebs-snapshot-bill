# Kwaliteit

Geteste commit: a74c727 (de code en de tests; latere commits raken alleen documenten).

- Tests: 68 geslaagd, 0 mislukt, 0 genegeerd (30 lib, 2 main, 6 claims, 30 cli; `cargo test --locked --no-fail-fast`, 2026-11-24, debugbuild, Linux).
- Rood voor de reparatie, gemeten: de bron van d84fb31 met de tests van a74c727 gaf `test result: FAILED. 23 passed; 7 failed; 0 ignored; 0 measured; 0 filtered out` voor `tests/cli.rs`. De 7 mislukte tests: `stderr_snapshot_id_clean`, `stderr_volume_size_clean`, `stderr_start_time_clean`, `stderr_path_clean`, `duplicate_keys_exit_2`, `duplicate_price_keys_exit_2`, `duplicate_snapshot_id_exit_2`.
- De 3 nieuwe tests voor `--as-of`, de leeftijdsgrens en mapinvoer (`as_of_changes_age_and_old`, `age_equal_to_max_is_not_old`, `directory_input_is_read`) slagen ook op d84fb31: ze leggen bestaand gedrag vast. Gemeten op een kopie: `as_of: cli.as_of` vervangen door een vaste datum, `if path.is_dir()` door `if false && path.is_dir()`, of `age > max_age_days` door `>=` laat ze falen.
- Ook gedraaid: `cargo fmt --check` en `cargo clippy --all-targets --locked -- -D warnings` (stable), beide schoon.
- Versie 0.1.0. `rust-version` 1.85.

Deze pagina legt controles vast. Het is geen garantie en geen fuzzrapport.

## Niet uitgevoerd

Niet uitgevoerd: clippy en tests op Rust 1.85, cargo deny, cargo audit, cargo publish --dry-run, een GitHub Actions-run, fuzzing en mutatietests (`cargo mutants`). Het CI-bestand noemt er meerdere; ze zijn voor deze boom niet op GitHub gedraaid, omdat deze boom niet gepusht is.
