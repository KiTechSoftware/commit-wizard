# cw push

Push local commits to a remote repository with optional pre-push policy checks. Acts as a policy-aware wrapper around `git push`.

## Usage

```sh
cw push [OPTIONS]
```

## Examples

**Push the current branch to `origin`:**

```sh
cw push
```

**Push to a specific remote and branch:**

```sh
cw push --remote upstream --branch release/1.0.0
```

**Push with a commit range check:**

```sh
cw push --from v1.0.0 --to HEAD
```

**Dry-run — show what would be pushed without pushing:**

```sh
cw push --dry-run
```

**Push to a protected branch (requires `--force` or config override):**

```sh
cw push --force
```

**Push in CI with strict checking:**

```sh
cw push --ci --from $BASE_SHA
```

## Options

| Flag | Default | Description |
| --- | --- | --- |
| `--remote <NAME>` | `origin` | Remote to push to |
| `--branch <BRANCH>` | current branch | Branch to push |
| `--from <REF>` | | Start of commit range for pre-push check |
| `--to <REF>` | `HEAD` | End of commit range for pre-push check |
| `--allow-empty` | | Allow pushing when there are no new commits |

## Pre-push checks

By default `cw push` validates commits and branch policy before pushing. You can control this behaviour in config:

```toml
[push.check]
commits = true          # run cw check on commits being pushed
branch_policy = true    # enforce branch naming and protection rules

[push.allow]
protected = false       # allow pushing to protected branches
force = false           # allow force push
```

These can also be overridden with the `--force` global flag or via environment variables (`CW_PUSH_CHECK_COMMITS`, `CW_PUSH_ALLOW_PROTECTED`, etc.).

## Global flags

All [global flags](../global-flags.md) are supported. `--dry-run` simulates the push and any checks without making network calls.
