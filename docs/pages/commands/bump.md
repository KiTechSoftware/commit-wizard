# cw bump

Calculate the next semantic version based on Conventional Commit history and active versioning rules. Does not create a tag — see [`cw tag`](./tag.md) for that.

## Usage

```sh
cw bump [OPTIONS]
```

## Examples

**Preview the next version from recent history:**

```sh
cw bump
```

**Bump based on a specific commit range:**

```sh
cw bump --from v1.2.0 --to HEAD
```

**Bump using the last 20 commits:**

```sh
cw bump --tail 20
```

**Preview in JSON (useful for CI scripting):**

```sh
cw bump --json
```

## How versioning works

`cw bump` scans commits in the specified range and applies [Semantic Versioning](https://semver.org) rules:

| Commit type | Version bump |
| --- | --- |
| Any `BREAKING CHANGE` footer or `!` suffix | **major** |
| `feat` | **minor** |
| `fix`, `perf`, and other patch-level types | **patch** |

The bump level is determined by the _highest_ impact commit in the range. You can customize which types bump which level via `[commit.types.<type>].bump` in your config.

## Options

| Flag | Description |
| --- | --- |
| `--from <REF>` | Start of commit range (tag, branch, or commit SHA) |
| `--to <REF>` | End of commit range (default: `HEAD`) |
| `--tail <N>` / `--count <N>` | Evaluate last N commits (0 = all reachable) |

## Global flags

All [global flags](../global-flags.md) are supported. Use `--dry-run` to preview the result without side effects, or `--json` to get machine-readable output.
