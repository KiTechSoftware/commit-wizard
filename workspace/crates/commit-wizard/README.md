# <img src="https://commit-wizard.kitechsoftware.com/assets/img/icon.png" alt="Commit Wizard Icon" width="50" align="left"/> Commit Wizard (`cw`)

[![Crates.io](https://img.shields.io/crates/v/commit-wizard.svg)](https://crates.io/crates/commit-wizard)
[![docs.rs](https://img.shields.io/docsrs/commit-wizard)](https://docs.rs/commit-wizard)
[![CI](https://github.com/KiTechSoftware/commit-wizard/actions/workflows/ci.yml/badge.svg)](https://github.com/KiTechSoftware/commit-wizard/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/commit-wizard.svg)](#license)

A spellbook for your commits.

> A command-line assistant for **Conventional Commits**, semantic versioning, and changelog automation.

`cw` guides you through writing structured commits, checks history against your project's rules, works out the next version, tags releases, writes changelogs, and repairs messages that broke the rules. It walks newcomers through each step and gets out of the way for people who already know what they want: everything it asks can also be given as a flag.

<img src="https://commit-wizard.kitechsoftware.com/assets/img/mascot.png" alt="Commit Wizard Mascot" width="200" align="right" />

**Full user manual: <https://commit-wizard.kitechsoftware.com>**

## Install

```sh
cargo install commit-wizard --locked
cw --version
```

This installs the `cw` binary. It needs a recent stable Rust toolchain (**1.89 or newer**) and `git` on your `PATH`.

## Quick start

```sh
cd my-repo
cw init            # a short guided setup that writes .cwizard.toml
cw add             # pick the files to stage
cw commit          # type, scope, subject, breaking change... then commit
cw check           # validate history against the rules
cw tag             # work out the next version from your commits and tag it
cw changelog       # write CHANGELOG.md from the history
```

## Commands

| Command | What it does |
| --- | --- |
| `cw commit` | Write a Conventional Commit through guided prompts, or from flags |
| `cw add` | Stage and unstage files with a picker, paths, or `--all` |
| `cw check` | Validate commits against the rules; exits non-zero when any break them |
| `cw fix` | Rewrite commit messages that break the rules, safely, with a backup |
| `cw bump` | Calculate the next semantic version from the commits since the last tag |
| `cw tag` | Create (and optionally sign and push) the next version tag, release candidates included |
| `cw changelog` | Generate or update a changelog from the tag history, in Markdown or JSON |
| `cw push` | Push with your branch and commit policies applied |
| `cw init` | Create a project or user config through a guided wizard |
| `cw config` | Show, edit, list and explain configuration, including where each value comes from |
| `cw doctor` | Diagnose configuration and environment problems |

## Guided for newcomers, scriptable for everyone else

On a terminal, `cw` asks for whatever it is missing. Give it the flags and it asks nothing:

```sh
cw commit                                 # walks you through it
cw commit -t feat -s api -m "add parser"  # no questions
```

Scripts and CI never get prompts: with no terminal, or with `--non-interactive`, `--ci` or `--json`, `cw` runs on flags alone and stops with a clear message naming the flag it needs. `--yes` accepts the default answer to any question that has one. `--dry-run` previews anything that would change something.

In a pipeline:

```sh
cw check                  # fails the job if any commit breaks the rules
cw bump --json            # machine-readable next version
cw tag --push             # tag and push the release
```

## Configuration

Settings live in `.cwizard.toml` (or `cwizard.toml`) in your project, and in `~/.config/cwizard/config.toml` for you. Files can be **TOML, YAML or JSON**.

- Layers are merged setting by setting: command-line flags, then `CW_*` environment variables, then the project file, then a shared **registry**, then built-in defaults. `cw config list --effective` shows the result and which layer each value came from.
- A **registry** is a Git repository of shared conventions (such as the Angular or gitmoji styles) that a team or organisation can point every project at, then override locally.
- Config edits keep your comments (TOML) and are written atomically, so an interrupted write never leaves a broken file.

See the manual for the full schema.

## Status

`cw` is pre-1.0: the command line and configuration are what you should rely on, and they may still change between `0.x` releases. The Rust library behind it is an implementation detail and is not a stable API.

## Contributing

Contributions are welcome; please read [CONTRIBUTING.md](https://github.com/KiTechSoftware/commit-wizard/blob/main/CONTRIBUTING.md) first.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/KiTechSoftware/commit-wizard/blob/main/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE](https://github.com/KiTechSoftware/commit-wizard/blob/main/LICENSE) or <http://opensource.org/licenses/MIT>)

at your option.
