# Contributing to UheGit

Thank you for your interest in contributing to UheGit!

## Setting Up the Development Environment

### Prerequisites

- **Rust 1.75 or later** — install via [rustup](https://rustup.rs/)
- **Git 2.x** — must be available in your PATH

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify
rustc --version
cargo --version
git --version
```

### Clone and Build

```bash
git clone https://github.com/tabrejansary/uhegit
cd uhegit
cargo build
```

---

## Development Workflow

### Run the application

```bash
cargo run -- [args]

# Examples:
cargo run                        # workspace overview
cargo run -- scan --path ~/Projects
cargo run -- --json
```

### Run tests

```bash
cargo test --all-targets
```

Integration tests use real temporary Git repositories created with `tempfile`. They do **not** require network access or a GitHub account. All temporary files are cleaned up automatically.

### Check formatting

```bash
cargo fmt --all -- --check
# To apply fixes:
cargo fmt --all
```

### Run Clippy

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Full CI check (what CI runs)

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```

---

## Code Style

- Follow standard Rust style enforced by `cargo fmt`
- Use `anyhow` for application-level error propagation
- Use `thiserror` for typed error definitions
- Document all public functions and types with doc comments
- Prefer descriptive error messages with context (`.with_context(|| ...)`)
- Never use shell interpolation for user-controlled input — always use argument arrays for Git commands
- Keep functions focused and small

---

## Architecture Overview

```
src/
├── main.rs          # Binary entry point
├── lib.rs           # Library crate (for integration tests)
├── cli.rs           # CLI argument definitions (clap)
├── types.rs         # Core domain types
├── error.rs         # Typed error definitions
├── commands/        # One module per CLI command
│   ├── mod.rs       # Command dispatch + auto-discovery
│   ├── status.rs    # Workspace overview
│   ├── scan.rs      # Repository scanning
│   └── ...
├── git/
│   ├── executor.rs  # Safe git subprocess execution
│   ├── parser.rs    # git status --porcelain=v1 parser
│   └── status.rs    # Status collection
├── discovery/
│   └── scanner.rs   # Recursive directory scanner
├── storage/
│   ├── config.rs    # Configuration persistence
│   └── state.rs     # App state (repos + sessions)
└── output/
    ├── display.rs   # Terminal colored output
    └── json.rs      # JSON serialization
```

---

## Submitting a Pull Request

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Make your changes
4. Ensure all checks pass:
   ```bash
   cargo fmt --all
   cargo test --all-targets
   cargo clippy --all-targets --all-features -- -D warnings
   ```
5. Write or update tests for any behavior changes
6. Update documentation if commands or configuration change
7. Open a pull request with a clear description of the change

### Pull Request Requirements

- All CI checks must pass
- New features should include tests
- Changes to CLI commands should update the README
- Changes to configuration keys should update the README configuration reference
- Keep commits focused; squash if needed before merge

---

## Reporting Bugs

Open a GitHub Issue with:
- Operating system and version
- Rust and Git versions (`rustc --version`, `git --version`)
- The command you ran
- The expected vs actual output
- Any relevant error messages

---

## Security

See [SECURITY.md](SECURITY.md) for how to report security vulnerabilities.

---

## Code of Conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
