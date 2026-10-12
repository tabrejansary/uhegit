# Security Policy

## Supported Versions

| Version | Supported |
|---|---|
| 0.1.x | ✅ Current |

## Reporting a Vulnerability

If you discover a security vulnerability in UheGit, please **do not** open a public GitHub issue.

Instead, use one of the following options:

1. **GitHub Private Security Reporting** — if enabled on this repository, use the "Report a vulnerability" button on the Security tab.
2. **GitHub Issue (public)** — for low-severity issues that do not expose sensitive data, you may open a regular issue labeled `security`.

Please include:
- A description of the vulnerability
- Steps to reproduce
- The potential impact
- Any suggested fix (optional but appreciated)

We aim to acknowledge reports within 72 hours and provide a fix or mitigation plan within 14 days for confirmed vulnerabilities.

## Security Design of UheGit

UheGit is designed to be safe to run repeatedly on your local repositories:

- **No shell interpolation** — all Git commands use argument arrays, never constructed shell strings
- **Read-only** — no `git fetch`, `git push`, `git commit`, `git reset`, `git clean`, or any state-changing Git operations
- **No credential storage** — no authentication tokens, passwords, or API keys are ever written
- **No network access** — all sync information is based on local tracking refs
- **No source code stored** — only notes, timestamps, commit hashes, and working-tree summaries
- **Atomic state writes** — state files are written atomically to prevent corruption
- **No repository-defined scripts** — UheGit does not invoke hooks or other repo-defined scripts
- **Symlink safety** — recursive scanning does not follow symbolic links by default

Despite these protections, please report anything that looks like it could be exploited.
