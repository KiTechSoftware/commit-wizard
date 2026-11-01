# How cw is built

This page records the decisions that shape `cw`, for contributors and for anyone who wants to know why it behaves as it does. The code layout is in `ARCHITECTURE.md` in the repository.

## Layers

`cw` is one crate in four layers. Each may use the ones below it and never the ones above.

| Layer | Holds |
| --- | --- |
| `cli` | Argument parsing, one module per command |
| `core` | The use cases (what each command does) and the shared context |
| `engine` | The models, configuration, rules and error types. Pure: no I/O |
| `infra` | Git (libgit2 and the `git` program), the terminal UI |

Rules such as branch naming and prune planning live in the engine, so they are tested without a repository.

## Configuration

Layers are merged in a fixed order, and a higher one wins: flags, environment variables, project config, registry config, defaults. A setting that cannot be read is reported and skipped, so a typo cannot silently change behaviour. Edits through `cw config` change only the key you name and write the file atomically. See [Configuration](configs/recommended.md) and [Environment variables](environment.md).

## Guided and non-interactive

A command that is missing information asks for it on a terminal, and never asks in scripts, pipes, `--json` or `--ci`. The two modes share one code path: a flag answers the question in advance. Details are in [global flags](../commands/global-flags.md).

## Safety rules

- **Protected branches** (`branch.protected`) are never deleted, and pushes to them need `push.allow.protected`.
- **Force pushes** need `push.allow.force` or `--force`.
- **Published history** is not rewritten: `cw fix` refuses commits that are on a remote unless `--force` is given.
- **Outward-facing steps** (pushing, deleting remote branches) always confirm, even with `--force`, unless `--yes` is given.
- **`--dry-run`** reports what would happen and changes nothing. It still reads the remote where a command needs to (a prune fetches first).
- Destructive steps print the command that undoes them.

## Git

Most work uses libgit2. The commands that must behave exactly like git, such as pushing, deleting remote branches, signing tags and fetching with pruning, run the `git` program, so your credentials, hooks and config apply. `git` must therefore be on the `PATH`. `cw doctor` works outside a repository; the other commands need one, and fail with `E4002` (or `E10001` when git itself reports it) naming the directory. Use `--cwd` to point at one.

## Versions and tags

The next version comes from the commits since the last tag: a breaking change gives a major bump, `feat` a minor, `fix` a patch. The tag prefix (`versioning.tag_prefix`, `v` by default) is added when tagging and ignored when reading, so `v1.2.3` and `1.2.3` mean the same version. A prerelease label gives `1.4.0-rc.1`.

## Changelog

`cw changelog` groups commits into sections by type, optionally by scope, and writes new releases after the header of an existing `CHANGELOG.md` without touching older entries. Types that are not mapped to a section are left out unless you ask for them with `--include-non-cc`.

## Errors

Every failure has a stable id and an exit code from its kind. See [errors and exit codes](exit-codes.md).

## Distribution

Releases ship as a crate on crates.io and as prebuilt archives on GitHub Releases, with checksums and installers. See [Installation](installation.md).
