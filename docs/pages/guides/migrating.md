# Migrating from earlier versions

The first prototypes of Commit Wizard (before 0.0.1) used a different configuration and different flags. If you wrote a config or a script against them, this page lists what changed.

## Environment variables

Variables are now `CW_<SECTION>_<KEY>` and mirror the config keys. The old double-underscore names (`CWIZARD__SECTION__KEY`) are not read.

| Before | Now |
| --- | --- |
| `CWIZARD_ALLOW_ENV_OVERRIDE` | `CW_ALLOW_ENV_OVERRIDE` |
| `CWIZARD__COMMIT__SUBJECT_MAX` | `CW_COMMIT_SUBJECT_MAX_LENGTH` |
| `CWIZARD__POLICY__ALLOW_PUSH_TO_MAIN` | `CW_PUSH_ALLOW_PROTECTED` |
| `CWIZARD__POLICY__REQUIRE_CONVENTIONAL_FOR_ALL_COMMITS` | `CW_CHECK_REQUIRE_CONVENTIONAL` |
| `CWIZARD__POLICY__CHECK_MODE` | `CW_CHECK_COMMITS_ENABLED` and `CW_CHECK_COMMITS_ENFORCE_ON` |
| `CWIZARD_CI_CONFIG`, `CWIZARD_IGNORE_CONFIG_FILE` | removed: pass `--config <PATH>` |
| `CWIZARD__DEFAULT__BRANCH` | removed: set `branch.protected` |
| `CWIZARD_SKIP_GUARD` | removed |

The full list is in [Environment variables](environment.md).

## Configuration

The `[policy]` section was split up: commit rules are in `[commit]` and `[check]`, push rules in `[push]`, branch rules in `[branch]`. Run `cw config list --effective` to see every setting and where its value came from, and `cw doctor` to find a missing or broken config. The full key list is in the [reference](public-api.md).

## Flags that no longer exist

| Before | Now |
| --- | --- |
| `--section.key=value` overrides on any command | Use `cw config set section.key value`, or the matching `CW_*` variable for one run |
| `cw add --diff <PATH>` | Use `git diff <PATH>`; `cw add` shows the diff in its picker |
| `cw changelog --append` | New releases are always inserted after the header; `--force` rewrites the file |
| `cw push --tags` | Use `cw tag --push` to push a tag |
| `cw bump --to-ref` | Use `--to` |
| `cw fix --auto` | Use `cw fix --yes` or `--ci` |

## Exit codes

The old codes `1`-`8` were replaced. See [errors and exit codes](exit-codes.md); in particular a rule violation is now `30` and a Git failure `21`.

## Hooks, AI and pull-request settings

`[pr]`, `[release]`, `[hooks]` and `[ai]` are reserved: accepted, not read.
