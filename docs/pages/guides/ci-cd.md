# CI/CD

`cw` is built to be the same tool on a laptop and in a pipeline. On a laptop it asks; in CI it never does, and it fails with an [exit code](exit-codes.md) a pipeline can act on.

## Install it in the pipeline

Prebuilt binaries are faster than compiling. See [Installation](installation.md) for details and pinning.

```sh
curl -fsSL https://raw.githubusercontent.com/KiTechSoftware/commit-wizard/main/scripts/install.sh | VERSION=0.0.4 bash
```

Or `cargo install commit-wizard --locked` where a Rust toolchain is already cached.

## What makes it non-interactive

`cw` does not ask when standard input or output is not a terminal, when `--json` is given, or when you pass `--ci` or `--non-interactive`. `--ci` also accepts defaults where they exist; `--non-interactive` asks nothing at all. See [global flags](../commands/global-flags.md).

Use `--ci` in pipelines so the intent is explicit even when a runner attaches a pseudo-terminal.

## Settings in CI

Settings come from, in order: command-line flags, [environment variables](environment.md), the project config, the registry config, then defaults. Setting `CW_ALLOW_ENV_OVERRIDE=false` ignores environment variables entirely, which keeps a pipeline's behaviour tied to the repository.

```sh
CW_COMMIT_SCOPES_MODE=required cw check --ci
```

## Checking commits on a pull request

A shallow clone cannot see the history `cw check` needs. Fetch it all.

```yaml
# GitHub Actions
- uses: actions/checkout@v4
  with:
    fetch-depth: 0
- run: cw check --ci --from origin/${{ github.base_ref }} --to HEAD
```

```yaml
# GitLab CI
check-commits:
  script:
    - git fetch origin $CI_MERGE_REQUEST_TARGET_BRANCH_NAME
    - cw check --ci --from origin/$CI_MERGE_REQUEST_TARGET_BRANCH_NAME --to HEAD
```

`cw check` exits `30` when a commit breaks the rules. `check.commits.enforce_on` decides on which branches that is enforced.

## Releasing from CI

```sh
VERSION=$(cw bump --plain)         # only the next version, such as 0.2.0
cw changelog --ci --next-tag "$VERSION" --yes   # writes CHANGELOG.md
cw tag --ci --push                 # create and push the tag
```

Pushing a tag is outward-facing. `cw push` and `cw tag --push` respect `push.allow.protected` and `push.allow.force`, and `--force` overrides them, so do not give a pipeline `--force` unless you mean it.

## Machine-readable output

`--json` wraps results in an envelope so a pipeline can read them instead of scraping text. `--format jsonl` is one JSON value per line for streaming. `--plain` removes decoration. Combine with `-q` to keep logs short, or `-v` for more detail when a job fails.

## A checklist for a pipeline job

- fetch full history (`fetch-depth: 0`)
- use `--ci`
- branch on the exit code, not on message text
- pin the `cw` version (`VERSION=` for the installer, `--locked` for cargo)
- run `cw doctor --ci --json` once to confirm the setup
