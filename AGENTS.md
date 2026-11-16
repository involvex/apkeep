# apkeep - Agent Instructions

`apkeep` is a Rust command-line tool for downloading APK files from various sources
(Google Play Store, APKPure, F-Droid). See README.md for user-facing documentation.

Sources used to build this file: README.md, HACKING.md, Cargo.toml, Dockerfile,
USAGE, .github/workflows/docker-build-push.yml.
(CLAUDE.md, GEMINI.md, and .github/copilot-instructions.md do not exist in this repo.)

## Useful Commands

Package manager: cargo (Rust). See Technologies for the toolchain.

| Task | Command |
|------|---------|
| Build debug binary | cargo build |
| Build release binary | cargo build --release |
| Build and run with args | cargo run -- ARGS |
| Build and run release with args | cargo run --release -- ARGS |
| Run tests | cargo test |
| Run tests release | cargo test --release |
| Format code | cargo fmt |
| Check formatting | cargo fmt --check |
| Clippy lints | cargo clippy |
| Clippy lints (release) | cargo clippy --release |
| Install from crates.io | cargo install apkeep |
| Install from git | cargo install --git https://github.com/EFForg/apkeep.git |
| Docker build | docker build -t apkeep . |
| Docker run (download to output dir) | docker run --rm -v OUTPUT_PATH:/output ghcr.io/efforg/apkeep:stable -a com.instagram.android /output |

Example invocations:

cargo run -- -a com.instagram.android .
cargo run -- -a org.mozilla.fennec_fdroid -d f-droid .
cargo run -- -l -a org.mozilla.fennec_fdroid -d f-droid
cargo run -- -a com.instagram.android@1.2.3 .

Binary output goes to target/debug/apkeep or target/release/apkeep.

## Technologies
- Language: Rust (edition 2021)
- Build system: Cargo
- Async runtime: tokio
- HTTP client: reqwest
- CLI parsing: clap
- Crypto: ring, sha1, sha2, base64, hex
- Google Play API: gpapi v6
- Downloads: tempfile, indicatif
- Filesystem: dirs, zip
- Futures: futures-util
- Container: Docker
- CI: GitHub Actions, Dependabot

- Async: Use tokio async/await; respect -r/--parallel concurrency limit (default 4)
- Errors: Use simple-error consistently; propagate errors with ? rather than panicking
- CLI: Add flags in src/cli.rs (clap derive); keep USAGE in sync
- Downloads: Stream to disk with tokio-dl-stream-to-disk; verify sha256 where supported
- Google Play: Requires AAS token; never commit tokens, emails, or credentials to repo
- Formatting: Run cargo fmt before committing; keep clippy warnings resolved
- Commits: Conventional, atomic commits; update CHANGELOG.md for user-facing changes
- Security: Avoid excessive parallel fetches; do not overload app distributor infrastructure
- Android/Termux: Install termux first, then pkg update and pkg install apkeep

- Do not modify application source code unless implementing a feature or fixing a bug
- Prefer Dark Mode as default appearance; use Hacker Theme (black background, green font)
- Use Bun (>=1.3.0) for Node.js tasks; go for Go modules; task for Taskfile.yml automation
- This repo itself is Rust/Cargo
- Use PowerShell-compatible commands on Windows: dir/Get-ChildItem, Remove-Item, Copy-Item
- Never bypass git hooks or CI checks; run cargo clippy and cargo fmt --check before committing
- Before claiming work complete, run verification commands and confirm output passes
- When adding new download sources, create USAGE-<source>.md and register in src/download_sources
- Keep docs consistent with CLI defined in src/cli.rs and USAGE
