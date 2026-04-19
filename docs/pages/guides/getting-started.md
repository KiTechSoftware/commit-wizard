# Getting Started with Commit Wizard

Commit Wizard (`cw`) helps you enforce **Conventional Commits**, automate **Semantic Versioning**, and keep your **changelog** in sync. This guide gets you up and running in 5 minutes.

## Installation

Install via Cargo:

```sh
cargo install commit-wizard
```

Verify installation:

```sh
cw --version
```

**Note:** Homebrew, Windows and Linux installers coming soon.

## Quick start (5 minutes)

### 1. Initialize your project

```sh
cd my-repo
cw init
```

This creates `.cwizard.toml` with sensible defaults. You're done — no further config needed for basic use.

### 2. Stage your work

```sh
cw add
# or use git directly
git add .
```

### 3. Commit with guidance

```sh
cw commit
```

You'll be prompted for:

- **Type** — `feat`, `fix`, `chore`, `docs`, `style`, `refactor`, `perf`, `test`
- **Scope** (optional) — `api`, `ui`, `db`, etc.
- **Subject** — what you changed (max 72 chars)
- **Body** (optional) — detailed explanation
- **Breaking change** (optional) — mark as major version bump
- **Footers** (optional) — `Closes: #123`, `Refs: #456`

```sh
feat(api): add pagination support

Adds cursor-based pagination to the /users endpoint.
Clients should pass `cursor` and `limit` query params.

Refs: #88
```

### 4. Verify compliance (in CI or locally)

```sh
# Check the last 10 commits
cw check --tail 10

# Check commits between two refs
cw check --from v1.0.0 --to HEAD
```

Exit status: `0` = all pass, non-zero = violations found.

### 5. Calculate next version

```sh
cw bump
# Output: v1.2.0 (detected from commits)
```

### 6. Create release tag

```sh
cw tag
# Creates annotated tag: v1.2.0

# Or with custom prefix
cw tag --prefix "" # → 1.2.0
cw tag --suffix -rc1 # → v1.2.0-rc1

# Sign and push in one step
cw tag --sign --push
```

## Common workflows

### Team: Solo developer on a personal project

You just need the defaults. Initialize once, then use `cw commit` for every commit.

```sh
cw init
# Done. Use cw commit and cw tag locally; no CI needed.
```

### Team: 5-10 person startup

Initialize with defaults + add team scope restrictions:

```sh
cw init --set commit.scopes.mode=required --set commit.scopes.restrict_to_defined=true
```

Then edit `.cwizard.toml`:

```toml
[commit.scopes.definitions.api]
title = "API"

[commit.scopes.definitions.ui]
title = "User Interface"

[commit.scopes.definitions.db]
title = "Database"
```

Commit and add to CI:

```sh
# CI gate
cw check --ci --from $MERGE_BASE --to HEAD || exit 1
cw bump
```

### Team: Large team (50+ engineers) with shared standards

Use the **registry system** to centralize config and rules for multiple projects.

```sh
# Create a shared registry repo
mkdir my-org-standards && cd my-org-standards
git init

# Bootstrap it
cw init registry . --rules --section shared --section frontend --section backend
```

Then in each project:

```toml
[registries.org-standards]
url = "https://github.com/my-org/my-org-standards.git"
ref = "main"
section = "frontend"  # or "backend", or "shared"

[registry]
use = "org-standards"
```

## Configuration layers

Commit Wizard resolves configuration in this order (highest priority first):

```sh
CLI args > environment variables > project config > registry config > defaults
```

**Examples:**

```sh
# CLI flag overrides config
cw commit --type feat --scope api --message "my message"

# ENV var overrides config
export CW_COMMIT_SCOPES_MODE=required
cw commit

# Project config (.cwizard.toml) is next
# Registry config (if selected) is next
# Built-in defaults apply if nothing else is set
```

## Execution modes

### Interactive (default)

Missing info? `cw` prompts you:

```sh
cw commit
# → Type? [feat, fix, chore...]: feat
# → Scope? [optional]: api
# → Subject: add pagination
# → Body? [leave blank]: ...
```

### Scripted (explicit input)

Provide all inputs, no prompts:

```sh
cw commit \
  --type feat \
  --scope api \
  --message "add pagination" \
  --body "Adds cursor-based pagination to /users endpoint"
```

Useful for hooks or CI that need to generate commits programmatically.

### CI mode (`--ci`)

Strict, never prompts, fails on missing input:

```sh
cw check --ci --from main --to HEAD || exit 1
cw bump --ci --to HEAD
```

### Dry-run mode (`--dry-run`)

See what would happen without making changes:

```sh
cw commit --dry-run --type feat --message "test"
cw tag --dry-run
cw push --dry-run
```

## Verbosity

Control output detail:

```sh
cw commit              # Normal output
cw commit -v           # INFO level (progress messages)
cw commit -vv          # DEBUG level (config resolution details)
cw commit -vvv         # TRACE level (everything)

cw commit -q           # WARN level (warnings only)
cw commit -qq          # ERROR level (errors only)
```

Set `RUST_LOG` directly for fine-grained control:

```sh
RUST_LOG=debug cw check --from v1.0.0
```

## JSON output

For scripting or integration:

```sh
cw bump --json
# → { "current": "1.0.0", "next": "1.1.0", ... }

cw check --json
# → { "passed": 5, "failed": 0, "results": [...] }
```

## Configuration reference

The `.cwizard.toml` file (created by `cw init`) supports these sections:

| Section | Purpose |
| --- | --- |
| `[commit]` | Commit type/scope definitions, subject length, breaking change behavior |
| `[branch]` | Protected branches, branch naming patterns |
| `[pr]` | PR title requirements (requires additional tooling; v0.0.2 basic support) |
| `[check]` | Which branches require conventional commits |
| `[push]` | Protected branch / force push restrictions |
| `[versioning]` | Tag prefix, version format |
| `[changelog]` | Output file, format, layout |
| `[registry]` | Active registry selection |
| `[rules]` | Reusable values referenced as `@rules.*` |

See [Configuration Guides](../guides/configs/standard.md) for full reference and examples.

## Common commands

| Command | Purpose |
| --- | --- |
| `cw add` | Stage files interactively or explicitly |
| `cw commit` | Write a conventional commit interactively or via flags |
| `cw check` | Validate commit history for compliance |
| `cw bump` | Calculate next semantic version |
| `cw tag` | Create a Git tag for the next release |
| `cw push` | Push with policy checks |
| `cw config show` | Inspect active configuration |
| `cw init` | Create `.cwizard.toml` with a profile |
| `cw doctor` | Diagnose issues and suggest fixes |

See [Command Reference](../commands/) for full details.

## Next steps

- **Customize config** — Adjust scopes, types, and validation rules in `.cwizard.toml`
- **Use in CI** — Run `cw check` as a pull request gate; `cw bump` and `cw tag` in release pipelines
- **Share config** — Set up a registry for teams or organizations
- **Configure hooks** — (Future in v0.0.3+) Integrate with Git hooks via `cw hook install`

## Help & troubleshooting

```sh
# General help
cw --help

# Command-specific help
cw commit --help
cw check --help

# Diagnose issues
cw doctor
cw doctor fix

# Increase logging
cw commit -vv

# Check what config is active
cw config show

# Inspect a single value
cw config get commit.scopes.mode
```

## Glossary

- **Conventional Commit** — A commit message format that includes type, scope, and breaking change info
- **Scope** — Optional context for a commit (e.g., `api`, `ui`)
- **Breaking Change** — A commit that requires a major version bump
- **Semantic Versioning** — Version format `major.minor.patch` (e.g., `1.2.3`)
- **Registry** — A shared Git repo with centralized config and rules
- **Dry-run** — Simulation mode; no mutations to Git state
- **CI mode** — Strict, non-interactive execution for automation

## Support

- [GitHub Issues](https://github.com/KiTechSoftware/commit-wizard/issues)
- [Documentation](../)
