<div align="center">

<h1>
  <img src="https://raw.githubusercontent.com/tabrejansary/uhegit/main/assets/logo.svg" width="48" alt="UheGit logo" />
  <br/>
  uhegit
</h1>

<p align="center">
  <strong>Automatic Git workspace discovery, live status overview, and work-session memory — all from your terminal.</strong>
</p>

<p align="center">
  <a href="https://github.com/tabrejansary/uhegit/actions/workflows/ci.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/tabrejansary/uhegit/ci.yml?branch=main&label=CI&logo=github&style=flat-square" alt="CI">
  </a>
  <a href="https://github.com/tabrejansary/uhegit/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License: MIT">
  </a>
  <a href="https://github.com/tabrejansary/uhegit/blob/main/Cargo.toml">
    <img src="https://img.shields.io/badge/MSRV-1.75-dea584?style=flat-square&logo=rust" alt="MSRV 1.75">
  </a>
  <img src="https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-lightgrey?style=flat-square" alt="Platform">
  <img src="https://img.shields.io/github/languages/top/tabrejansary/uhegit?style=flat-square&color=dea584" alt="Language: Rust">
  <br/>
  <a href="https://github.com/tabrejansary/uhegit/releases/latest">
    <img src="https://img.shields.io/github/v/release/tabrejansary/uhegit?label=release&style=flat-square&color=blue" alt="GitHub Release">
  </a>
  <a href="https://github.com/tabrejansary/uhegit/releases">
    <img src="https://img.shields.io/github/downloads/tabrejansary/uhegit/total?label=downloads&style=flat-square&color=blue" alt="Downloads">
  </a>
</p>

<p align="center">
  <a href="#-install">Install</a> ·
  <a href="#-quick-start">Quick Start</a> ·
  <a href="#-all-commands">Commands</a> ·
  <a href="#-how-it-works">How It Works</a> ·
  <a href="#-configuration">Configuration</a> ·
  <a href="#-contributing">Contributing</a>
</p>

</div>

---

> **One command. Every repository. Instant context.**

You work across many projects. `uhegit` finds all your Git repositories automatically, shows their live status in one clean table, and remembers exactly where you left off on each branch — so you can switch projects in seconds, not minutes.

```
  UheGit — Your Workspace
  ──────────────────────────────────────────────────────────────────────

  REPOSITORY    BRANCH          STATUS                    SYNC
  api-service   feat/payments   Uncommitted (3~ 1- 2?)    Up to date
  mobile-app    main            Uncommitted (8~ 5?)       No upstream
  dashboard     fix/dark-mode   ↑ Unpushed                ↑ 2 ahead*
  my-cli-tool   main            ✓ Clean                   Up to date
  backend       main            ✓ Clean                   Up to date
  docs-site     main            ✓ Clean                   Up to date

  6 repositories | 3 clean | 3 need attention

  NEXT TASKS
  api-service: Finish payment webhook handler
  dashboard:   Review PR before merging
```

---

## ✨ Features

| | Feature | Description |
|--|--|--|
| 🔍 | **Auto-discovery** | Finds all your Git repos in common dev folders. No manual registration needed. |
| 📊 | **Live workspace** | One table showing branch, changes, and sync state for every repo simultaneously. |
| 🧵 | **Parallel execution** | All repos inspected concurrently via a Rayon thread pool. 15 repos = same time as 1. |
| 📝 | **Session memory** | Notes and next tasks saved per-repo and per-branch. Switching branches preserves each branch's context. |
| 💾 | **Checkpoints** | Record the current state of a repo (branch, commit, changes) so you can resume later. |
| 🔁 | **Resume** | See full session context — branch, changes, your note, next task, and a ready-to-paste `cd` command. |
| 🔒 | **Read-only** | Never runs `git fetch`, `git push`, or any state-changing operation. Your repos are safe. |
| 📡 | **JSON output** | Every command supports `--json` for scripting and tooling. |
| 🐚 | **Shell completions** | Tab completions for Bash, Zsh, Fish, and PowerShell. |
| 🪟 | **Cross-platform** | Windows 10+, macOS, Linux — same binary, same behaviour. |

---

## 📦 Install

### From crates.io *(coming soon)*

```bash
cargo install uhegit
```

### From GitHub Releases *(pre-built binaries)*

Download the binary for your platform from [GitHub Releases](https://github.com/tabrejansary/uhegit/releases/latest):

| Platform | File |
|---|---|
| Windows x64 | `uhegit-windows-x86_64.exe` |
| macOS Apple Silicon | `uhegit-macos-aarch64` |
| macOS Intel | `uhegit-macos-x86_64` |
| Linux x64 | `uhegit-linux-x86_64` |
| Linux ARM64 | `uhegit-linux-aarch64` |

### Build from Source *(requires Rust 1.75+)*

```bash
git clone https://github.com/tabrejansary/uhegit
cd uhegit
cargo install --path .
```

### Requirements

- **Git** must be installed and available in your `PATH`
- Rust 1.75+ (only needed when building from source)

---

## 🚀 Quick Start

```bash
# Show your workspace — auto-discovers repos on first run
uhegit

# Scan a specific folder for repositories
uhegit scan --path ~/Projects

# Save context before switching to another project
uhegit note my-app "Auth flow half-done, token refresh still broken"
uhegit next my-app "Add retry handler for network errors"

# Jump back in later
uhegit resume my-app

# Get the path for scripting (bash/zsh)
cd "$(uhegit path my-app)"

# Get the path for scripting (PowerShell)
Set-Location (uhegit path my-app)
```

---

## 📖 All Commands

### Workspace Overview

```bash
uhegit                        # Show all repos (default)
uhegit status                 # Same as above
uhegit --json                 # Machine-readable JSON output
```

**Status Filters** — combine with any overview command:

```bash
uhegit --dirty                # Only repos with uncommitted changes
uhegit --unpushed             # Only repos with commits ahead of upstream
uhegit --behind               # Only repos behind upstream
uhegit --conflicts            # Only repos with merge conflicts
uhegit --clean                # Only clean repos
uhegit --stale 7d             # Only repos with no activity in 7+ days
uhegit --stale 30d
uhegit --dirty --json         # Filters and JSON work together
```

---

### 🔍 Discovery

```bash
uhegit scan                          # Re-scan configured discovery roots
uhegit scan --path ~/Projects        # Scan a specific directory
uhegit scan --path "D:\Work"         # Windows paths work too
uhegit scan --home                   # Scan your entire home directory
uhegit scan --depth 8                # Go deeper (default depth: 6)
uhegit scan --depth 3                # Shallower scan for large dirs

uhegit add ~/Projects/my-repo        # Manually add a specific repository
```

**What scanning skips automatically:** `node_modules`, `target`, `.venv`, `vendor`,
`dist`, `build`, `AppData`, `Library`, `.cargo`, `.rustup`, system directories,
and anything inside `.git` internals. Symbolic links are never followed.

---

### 📋 Managing Repos

```bash
uhegit repos                  # List all tracked repositories
uhegit repos --json           # As JSON

uhegit remove <repo>          # Stop tracking a repo (files are NEVER deleted)
uhegit remove my-old-project
```

---

### 📝 Notes & Next Tasks *(per branch)*

```bash
# Save a note for the current branch of a repository
uhegit note <repo> "your note"
uhegit note my-app "Auth flow half-done — token refresh still failing"
uhegit note my-api "Waiting for design assets from team"

# Clear a note
uhegit note my-app ""

# Set what to do next when you return
uhegit next <repo> "your next task"
uhegit next my-app "Add retry logic for the network layer"
uhegit next my-cli "Write help text for the new subcommand"

# Clear the next task
uhegit next my-app ""
```

> **Notes are per branch.** If you're on `feat/auth` and switch to `main`,
> each branch keeps its own note and next task independently.

---

### 💾 Checkpoints & Resuming Work

```bash
# Record a read-only snapshot of the current repo state
uhegit checkpoint <repo>
uhegit checkpoint my-app
# → Saves: branch, commit hash, working-tree summary
# → Does NOT modify your repository in any way

# Resume work — shows everything you need to get back in context
uhegit resume <repo>
uhegit resume my-app
uhegit resume my-app --json     # As JSON
```

**Resume output:**

```
  UheGit — Resume Session
  ──────────────────────────────────────────────────

  Repository: my-app
  Path:       /home/you/Projects/my-app
  Branch:     feat/auth
  Commit:     abc1234 — Add token refresh endpoint (2 hours ago)
  Changes:    3 modified, 1 untracked
  Upstream:   2 ahead of origin/feat/auth *
              * Based on local tracking refs. Run 'git fetch' to refresh.

  Last note:
  Auth flow half-done — token refresh still failing

  Next task:
  Add retry logic for the network layer

  Last checkpoint:
  abc1234 at 2026-10-12 14:30 UTC
  3 modified, 1 untracked

  Continue:
  cd "/home/you/Projects/my-app"
```

---

### 📂 Path Command *(for scripting)*

```bash
uhegit path <repo>

# Jump to a project in bash/zsh
cd "$(uhegit path my-app)"

# Jump to a project in PowerShell
Set-Location (uhegit path my-app)

# Open in VS Code
code "$(uhegit path my-app)"
```

---

### 🗑️ Forget Session Memory

```bash
# Remove stored notes, tasks, and checkpoints (asks for confirmation)
uhegit forget <repo>
uhegit forget my-app
# → Your repository files are NOT affected
```

---

### ⚙️ Configuration

```bash
uhegit config show                    # Show current settings as JSON
uhegit config path                    # Show config file location

# Adjust settings:
uhegit config set max-depth 8         # Scan 8 levels deep (default: 6)
uhegit config set max-workers 4       # Parallel git workers (default: 8)
uhegit config set git-timeout 30      # Git timeout in seconds (default: 15)
uhegit config set color false         # Disable terminal colors
uhegit config set color true          # Enable terminal colors
```

**Config file locations:**

| Platform | Path |
|---|---|
| Windows | `%APPDATA%\uhegit\uhegit\config\config.json` |
| macOS | `~/Library/Application Support/io.uhegit.uhegit/config.json` |
| Linux | `~/.config/uhegit/config.json` |

---

### 🛠️ Utilities

```bash
uhegit version                        # Print version information

# Generate shell completions:
uhegit completion bash                # Bash
uhegit completion zsh                 # Zsh
uhegit completion fish                # Fish
uhegit completion powershell          # PowerShell

# Install completions permanently:
uhegit completion bash >> ~/.bashrc
uhegit completion zsh >> ~/.zshrc
uhegit completion fish > ~/.config/fish/completions/uhegit.fish
uhegit completion powershell >> $PROFILE
```

---

### Complete Command Reference

| Command | What it does |
|---|---|
| `uhegit` | Workspace overview (auto-discovers repos first run) |
| `uhegit --dirty` | Only repos with uncommitted changes |
| `uhegit --unpushed` | Only repos with unpushed commits |
| `uhegit --behind` | Only repos behind upstream |
| `uhegit --conflicts` | Only repos with merge conflicts |
| `uhegit --clean` | Only clean repos |
| `uhegit --stale Nd` | Only repos idle for N+ days |
| `uhegit --json` | Any command as JSON |
| `uhegit scan` | Re-scan discovery roots |
| `uhegit scan --path <dir>` | Scan a specific directory |
| `uhegit scan --home` | Scan from home directory |
| `uhegit scan --depth N` | Set scan depth |
| `uhegit add <path>` | Manually add a repository |
| `uhegit remove <repo>` | Stop tracking (no files deleted) |
| `uhegit repos` | List all tracked repos |
| `uhegit note <repo> "..."` | Save a branch-specific note |
| `uhegit next <repo> "..."` | Set the next task for a branch |
| `uhegit checkpoint <repo>` | Save a read-only state snapshot |
| `uhegit resume <repo>` | Show full session + cd command |
| `uhegit path <repo>` | Print path (for scripting) |
| `uhegit forget <repo>` | Clear session memory |
| `uhegit config show` | Show current configuration |
| `uhegit config path` | Show config file location |
| `uhegit config set <k> <v>` | Set a configuration value |
| `uhegit version` | Print version info |
| `uhegit completion <shell>` | Generate tab completions |

---

## 🔬 How It Works

### Architecture

```
uhegit
  │
  ├── commands/         One module per command
  │     ├── status.rs   ─── parallel repo inspection (rayon)
  │     ├── scan.rs     ─── recursive directory discovery
  │     ├── resume.rs   ─── session context display
  │     └── ...
  │
  ├── git/
  │     ├── executor.rs ─── safe subprocess, 15s timeout, no shell injection
  │     ├── parser.rs   ─── git status --porcelain=v1 -z (NUL-safe)
  │     └── status.rs   ─── status + commit info collection
  │
  ├── discovery/
  │     └── scanner.rs  ─── recursive walker, exclusions, depth limit
  │
  ├── storage/
  │     ├── config.rs   ─── JSON config, atomic writes, platform paths
  │     └── state.rs    ─── repo registry + per-branch session memory
  │
  └── output/
        ├── display.rs  ─── colored terminal table
        └── json.rs     ─── stable JSON serialization
```

### Status Categories (Priority Order)

| Priority | Category | What it means |
|---|---|---|
| 🔴 1 | **Conflicts** | Merge conflicts or unresolvable state |
| 🟠 2 | **Dirty** | Staged, modified, untracked, or deleted files |
| 🟡 3 | **Diverged** | Commits both ahead AND behind upstream |
| 🔵 4 | **Unpushed** | Commits ahead of upstream only |
| 🟣 5 | **Behind** | Commits behind upstream only |
| ✅ 6 | **Clean** | Clean working tree, in sync with upstream |

Repositories are sorted by priority (worst first), then alphabetically within each group.

### Sync Status Note

Sync information (ahead/behind) is based on **local tracking refs only**.
UheGit never runs `git fetch`. Rows marked `*` mean sync info may be stale.
Run `git fetch` in that repo to refresh.

### JSON Output Format

```bash
uhegit --json
```

```json
{
  "repositories": [
    {
      "name": "my-app",
      "path": "/home/you/Projects/my-app",
      "git": {
        "branch": "feat/auth",
        "detached_head": false,
        "commit_hash": "abc1234",
        "commit_subject": "Add token refresh endpoint",
        "commit_time_relative": "2 hours ago",
        "staged": 0,
        "modified": 3,
        "untracked": 1,
        "conflicted": 0,
        "deleted": 0,
        "upstream": "origin/feat/auth",
        "ahead": 2,
        "behind": null,
        "status": "Uncommitted changes",
        "sync_stale": false
      },
      "session": {
        "branch": "feat/auth",
        "last_session": "2026-10-12T14:30:00Z",
        "note": "Auth flow half-done — token refresh still failing",
        "next_task": "Add retry logic for the network layer",
        "checkpoint_hash": "abc1234",
        "checkpoint_time": "2026-10-12T14:00:00Z",
        "checkpoint_summary": "3 modified, 1 untracked"
      },
      "error": null
    }
  ],
  "summary": {
    "total": 6,
    "clean": 3,
    "dirty": 2,
    "needs_attention": 3,
    "with_errors": 0
  }
}
```

---

## ⚙️ Configuration Reference

| Key | Default | Description |
|---|---|---|
| `max-depth` | `6` | Maximum directory scan depth |
| `max-workers` | `8` | Concurrent Git worker threads |
| `git-timeout` | `15` | Git subprocess timeout (seconds) |
| `color` | `true` | Enable terminal colors |

---

## 🔒 Privacy & Safety

UheGit is designed to be completely safe to run on your machine:

- ✅ **Read-only** — no `git fetch`, `git push`, `git commit`, `git checkout`, or any write operation
- ✅ **No network access** — all sync info is based on local tracking refs
- ✅ **No credentials stored** — no tokens, passwords, or API keys ever written
- ✅ **No source code stored** — only notes, timestamps, and commit hashes
- ✅ **No shell interpolation** — all Git commands use argument arrays, never shell strings
- ✅ **No hooks invoked** — repository-defined scripts are never executed
- ✅ **Atomic writes** — state files written via temp→rename, never corrupted
- ✅ **No symlink traversal** — scanning never follows symlinks (prevents loops)

---

## 🏗️ Building from Source

```bash
git clone https://github.com/tabrejansary/uhegit
cd uhegit

# Debug build
cargo build

# Run all tests (72 tests)
cargo test --all-targets

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format check
cargo fmt --all -- --check

# Optimized release binary
cargo build --release
# → target/release/uhegit  (or uhegit.exe on Windows)
```

**Minimum supported Rust version:** 1.75

---

## ⚖️ Comparison

| Feature | uhegit | git-summary | mgitstatus | gita |
|---|:---:|:---:|:---:|:---:|
| Auto-discovery | ✅ | ❌ | ❌ | ❌ |
| Session memory (notes/tasks) | ✅ | ❌ | ❌ | ❌ |
| Per-branch notes | ✅ | ❌ | ❌ | ❌ |
| Work resume / checkpoint | ✅ | ❌ | ❌ | ❌ |
| JSON output | ✅ | ❌ | ❌ | ❌ |
| Read-only (safe) | ✅ | ✅ | ✅ | ❌ |
| Shell completions | ✅ | ❌ | ❌ | ✅ |
| Cross-platform | ✅ | ❌ | ❌ | ✅ |
| Written in Rust | ✅ | ❌ | ❌ | ❌ |

---

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for:

- How to set up the development environment
- How to run tests
- Code style expectations
- How to submit a pull request

```bash
# Quick start for contributors
git clone https://github.com/tabrejansary/uhegit
cd uhegit
cargo test --all-targets           # Run all 72 tests
cargo clippy -- -D warnings        # Zero warnings policy
cargo fmt --all                    # Auto-format
```

---

## 📜 License

MIT — see [LICENSE](LICENSE) for full text.

---

## 🙏 Acknowledgements

Built with:
- [clap](https://github.com/clap-rs/clap) — CLI argument parsing
- [rayon](https://github.com/rayon-rs/rayon) — data parallelism
- [crossterm](https://github.com/crossterm-rs/crossterm) — cross-platform terminal colors
- [serde](https://github.com/serde-rs/serde) + [serde_json](https://github.com/serde-rs/json) — serialization
- [anyhow](https://github.com/dtolnay/anyhow) + [thiserror](https://github.com/dtolnay/thiserror) — error handling
- [directories](https://github.com/dirs-dev/directories-rs) — platform config paths
- [tempfile](https://github.com/Stebalien/tempfile) — atomic writes

---

<div align="center">
  <sub>Made with ❤️ by <a href="https://github.com/tabrejansary">tabrejansary</a> · MIT License · Cross-platform Rust</sub>
</div>
