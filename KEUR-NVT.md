build.rs: no build script, because the crate has no code generation or native linking
man: no man page, because the clap --help text documents the options
docs: no docs directory, because README.md and README.nl.md hold all the documentation
samples: no samples directory, because examples/ already holds the sample inputs
scripts: no helper scripts, because the whole workflow is cargo build and cargo test
.github/workflows/release.yml: no release workflow, because nothing is released or tagged yet
