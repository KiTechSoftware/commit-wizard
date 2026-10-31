# Architecture

A guide for people who want to read, fix or extend `cw`. For using it, see the [user manual](https://commit-wizard.kitechsoftware.com); for how to send a change, see [CONTRIBUTING.md](CONTRIBUTING.md).

`cw` is a command-line tool: one binary built from one crate, `commit-wizard`, in `workspace/crates/commit-wizard`. It is published as a library only so the binary, the tests and docs.rs can share code, so nothing inside it is a stable API.

## Repository map

| Path | What is there |
| --- | --- |
| `workspace/crates/commit-wizard/src` | The crate (see the layers below) |
| `workspace/crates/commit-wizard/tests` | End-to-end tests that run the real `cw` binary |
| `docs/` | The user manual: pages in `docs/pages`, navigation in `docs/mkdocs.yml` |
| `scripts/` | Build, test and release helpers (`scripts/sync-readme.sh` among them) |
| `docker/` | Images, including the one that builds the manual |
| `.github/workflows/` | CI and release pipelines |
| `README.md` | The repository README; the crate carries a synced copy that crates.io and docs.rs show |

## The four layers

The crate is layered from the outside in. Each layer may use the ones below it, never the ones above.

| Layer | Directory | Job |
| --- | --- | --- |
| **cli** | `src/cli` | Parses arguments (`cli/args.rs`, one `Args` struct per command in `cli/cmd/<command>`) and dispatches to a use case. No logic. |
| **core** | `src/core` | One **use case** per command in `core/usecases/`. Orchestrates: reads config, calls git, asks questions, prints the result. Also the `Context` every command runs in and the bootstrap that builds it. |
| **engine** | `src/engine` | The rules: models, configuration, validation, version maths, changelog grouping, message composition. Mostly pure functions, so it is cheap to unit test. Defines the traits (`PromptTrait`, `LoggerTrait`) that `infra` implements. |
| **infra** | `src/infra` | The outside world: Git (`infra/git`, built on `git2`, shelling out to `git` for pushing and GPG signing) and the terminal (`infra/ui`, built on the `scriba` crate). |

A rule of thumb for where new code goes: if it can be a function from data to data, it belongs in `engine/capabilities`; if it talks to Git or the terminal, it belongs in `infra`; the use case in `core` is the glue between them.

## How a command runs

Following `cw tag`:

1. `main.rs` calls `cli::run()`.
2. `cli::run` parses the arguments with clap (`cli/args.rs`) and calls `core::bootstrap::build_app_context`.
3. **Bootstrap** works out the *run mode* (see below), builds a `Runtime` from the global flags, finds the Git repository, then resolves every configuration source (CLI flags, `CW_*` environment, the project file, the global file, registries) and finally the active configuration and the policy models derived from it. If that fails, the command never starts.
4. `cli::cmd::tag::run` hands the parsed flags to `core::usecases::version::tag::run`.
5. The use case asks `engine` for the version (`engine/capabilities/versioning`), asks `infra::git` to create the tag, and builds an output document.
6. The `Ui` prints the document (text, Markdown or a JSON envelope, depending on `--format` and `--json`) and the process exits with the code from the returned `Result`.

## Key ideas

### Configuration is layered, typed and format-agnostic

- **Discovery** (`engine/constants/paths.rs`): `.cwizard.*` then `cwizard.*` in the working directory or repository root; `config.*` and `rules.*` in the per-user directory; `config.*` in a registry.
- **Format** (`engine/config/format.rs`): `ConfigFormat` reads and writes TOML, YAML and JSON from the file extension. Everything after this point works on the same typed structures (`BaseConfig` and its children), whatever the file looked like.
- **Layers** (`Runtime::config_layers`): registry, then project, then environment, then CLI flags, each overriding the one before. The global config is used only when no other layer exists, unless the project sets `[global] inherit = true`. `cw config list --effective` prints exactly this and where each value came from, and both it and the real resolver use the same function so they cannot drift.
- **Merging** (`BaseConfig::merge`): a recursive merge through the TOML representation. Tables combine key by key, the higher layer wins per value, arrays are replaced.
- **Rules**: `@rules.<name>` in any text setting is replaced from the `[rules]` table when the configuration loads.
- **Policy** (`engine/models/policy`): the resolved configuration is turned into plain models (`CommitModel`, `VersioningModel`, ...) that the use cases read. Use cases should not poke at raw config.

`[pr]`, `[release]`, `[hooks]` and `[ai]` are parsed and validated but no command reads them yet.

### Guided by default, scriptable always

Every command must work from flags alone, and a command that is missing information may ask for it only when a person is at a terminal.

- `RunMode` (`Interactive`, `NonInteractive`, `Ci`) is decided once in bootstrap: no terminal on stdin or stdout, or `--json`, means non-interactive.
- `Ui` carries a `PromptPolicy`: `Ask` on a terminal, `Defaults` for `--yes` and `--ci` (the default answer where one exists, otherwise an error), `Refuse` for `--non-interactive`. A prompt can therefore never hang a script.
- A guided flow checks `ctx.is_interactive() && !ctx.auto_yes()` and asks only about what the flags left open. See `core/usecases/version/tag.rs` or `core/usecases/configs/wizard.rs`.

### Errors and exit codes

`ErrorCode` (`engine/error/code.rs`) is declared once with a kind, a number and a message; it prints as `E<kind><nn>` and decides the exit status by kind:

| Kind | Exit | Kind | Exit |
| --- | --- | --- | --- |
| User input | 2 | Validation (e.g. `cw check` found bad commits) | 30 |
| Config | 10 | Release | 31 |
| State | 11 | Hook | 32 |
| Registry | 20 | AI | 33 |
| Git (e.g. a published commit refused by `cw fix`) | 21 | Runtime | 50 |
| I/O / Process | 25 / 26 | | |

Add context with `.with_context("key", value)`; it is printed under the message.

### Writing files

- **Always** use `engine::fs::write_text`. It writes a temporary file beside the target, flushes it and renames it into place, keeps the permissions and follows symlinks, so an interrupted write never leaves a broken config. Do not call `std::fs::write` for user files.
- TOML config edits go through `engine/capabilities/config/patch.rs`, which changes only the keys that differ and keeps the user's comments and layout. The result is re-parsed and checked, with a full re-render as the fallback. YAML is re-rendered (its comments are not preserved); JSON is rewritten in full.

### History rewriting

`cw fix` is the one command that rewrites history, so it is deliberately cautious (`infra/git`: `rewrite_span` and `rewrite_messages`). It recreates commits with the same tree, author and dates, checks the new tip has the old tip's tree before moving anything, saves the old tip under `refs/cw/fix-backup/`, and refuses published, signed and merge commits. Keep that discipline for anything similar.

## Adding a command

1. **Args**: `src/cli/cmd/<name>/mod.rs` with an `Args` struct and `run`; add `pub mod <name>;` in `cli/cmd/mod.rs`, a variant in `Command` in `cli/args.rs`, and a match arm in `cli/mod.rs`.
2. **Use case**: `src/core/usecases/<area>/<name>.rs` (and `pub mod` it).
3. **Logic**: put anything that can be a pure function in `src/engine/capabilities/<area>` with unit tests beside it.
4. **Output**: build the result with `ui.new_output_content()`, so `--json`, `--format` and `--dry-run` work for free.
5. **Prompts**: only through `ctx.ui()`; ask only about what flags did not give, and only when guided (see above).
6. **Tests**: end-to-end tests in `tests/<name>.rs` (below), including a no-terminal test and, if it is guided, a pty test.
7. **Docs**: a page in `docs/pages/commands/<name>.md`, an entry in `docs/mkdocs.yml`, and a row in the command table in `docs/pages/guides/getting-started.md`. Every flag must appear on its page.

## Tests

- **Unit tests** live beside the code, mostly in `engine`.
- **End-to-end tests** (`workspace/crates/commit-wizard/tests`) run the real binary. `tests/common` provides a `Repo` that creates a temporary Git repository, a local bare remote, and an isolated `HOME` and Git configuration so nothing depends on the developer's machine. `repo.cw(&[...])` runs `cw` as a script would (no terminal); `repo.cw_pty(&[...])` runs it on a pseudo-terminal so guided flows can be driven with `expect` and `line`. GPG helpers create a throwaway key for signing tests.
- **No network by default.** Registries are served from `file://` repositories. `tests/live_registry.rs` hits the real public registry and is `#[ignore]`d: run it with `cargo test --test live_registry -- --ignored`.
- When a test guards a safety property, prove it by breaking the code on purpose and watching the test fail.

## Quality gates

CI (`.github/workflows/ci.yml`) runs `lint` (rustfmt and clippy with `-D warnings`), `build`, `test`, `coverage`, `cargo-audit`, `docs` (rustdoc with warnings denied, plus doctests), `package` (`cargo publish --dry-run`, license files and README present) and `msrv` (the minimum Rust version in `Cargo.toml`). Locally, from `workspace/`:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
```

CI uses the current stable Rust, whose clippy is newer than an older local toolchain's; update yours if CI flags something locally clean.

The internal modules are hidden from docs.rs, so to browse them run `cargo doc --document-private-items --open`.

## Keeping the docs in step

- The user manual (`docs/pages`) is part of the change: a new or changed flag is not done until its page says so.
- The crate README is a copy of the repository README. After editing `README.md`, run `scripts/sync-readme.sh`; a test fails if they differ. Links in it must be absolute, because docs.rs and crates.io show it away from the repository.

## Releases

Work lands on `develop`. A release is a pull request from `develop` through a `release/*` branch into `main`, then a tag; the version lives in `workspace/Cargo.toml` and `VERSION`. Maintainers handle this; contributors do not need to.
