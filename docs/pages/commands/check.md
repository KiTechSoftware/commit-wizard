# cw check

Validate commit messages in your Git history against active [Conventional Commits](https://www.conventionalcommits.org) rules. Use this locally before pushing, or in CI as a gate.

## Usage

```sh
cw check [OPTIONS]
```

## Examples

**Check the last 10 commits:**

```sh
cw check --tail 10
```

**Check all commits between two refs:**

```sh
cw check --from v1.0.0 --to HEAD
```

**Check every commit reachable from HEAD:**

```sh
cw check --tail 0
```

**Use in CI (non-interactive, exit non-zero on failure):**

```sh
cw check --ci --from $BASE_SHA --to $HEAD_SHA
```

**Show full commit hashes in output:**

```sh
cw check --tail 20 --full-hash
```

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | All commits pass |
| non-zero | One or more commits violate the active rules |

## Options

| Flag | Short | Description |
| --- | --- | --- |
| `--tail <N>` / `--count <N>` | | Validate last N commits (0 = all reachable) |
| `--from <REF>` | | Start of commit range |
| `--to <REF>` | | End of commit range |
| `--full-commit-hash` / `--full-hash` | `-H` | Show full commit hash in output instead of abbreviated |

## What gets checked

- **Type** — must be one of the allowed commit types defined in config (`feat`, `fix`, etc.)
- **Scope** — validated against defined scopes when `commit.scopes.restrict_to_defined = true`
- **Subject length** — enforced against `commit.subject_max_length` (default: 72)
- **Ticket reference** — required when `commit.ticket.required = true`

Use [`cw config show`](./config.md) to inspect the active rules that `cw check` enforces.

## Global flags

All [global flags](../global-flags.md) are supported. `--ci` or `--non-interactive` are recommended in automation contexts.
