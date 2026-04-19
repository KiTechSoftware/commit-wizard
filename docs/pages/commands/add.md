# cw add

Stage files for commit interactively or by explicit path. A thin wrapper around `git add` that integrates with Commit Wizard's workflow.

## Usage

```sh
cw add [OPTIONS] [PATH]...
```

## Examples

**Interactive staging** — opens a file picker to choose what to stage:

```sh
cw add
```

**Stage specific files:**

```sh
cw add src/main.rs Cargo.toml
```

**Stage everything** (equivalent to `git add -A`):

```sh
cw add --all
```

**Unstage files:**

```sh
cw add --unstage src/main.rs
```

**Stage only unstaged files** (skip files already in the index):

```sh
cw add --exclude-staged
```

## Options

| Flag | Short | Description |
| --- | --- | --- |
| `[PATH]...` | | Paths (files or directories) to stage |
| `--path <PATH>` | | Additional path(s); may be repeated |
| `--all` | `-A` | Stage all changes (`git add -A`) |
| `--unstage` | | Unstage instead of staging |
| `--exclude-staged` | | Exclude already-staged files from interactive selection |

## Global flags

All [global flags](../global-flags.md) are supported, including `--dry-run` to preview what would be staged without actually staging it.
