# cw doctor

Diagnose common issues with your Commit Wizard setup and optionally apply safe fixes. Useful when something isn't behaving as expected or after upgrading.

## Usage

```sh
cw doctor [SUBCOMMAND]
```

## Subcommands

| Subcommand | Description |
| --- | --- |
| *(none)* | Run diagnostics and report issues |
| `fix` | Create a standard project config when the repository has none (with `--dry-run`, only says what it would create) |

## Examples

**Run diagnostics:**

```sh
cw doctor
```

**Create the project config if the repository has none:**

```sh
cw doctor fix
```

**Run diagnostics in CI (non-interactive, structured output):**

```sh
cw doctor --ci --json
```

## What doctor checks

Each check is reported as ok, info, warning or error; the command fails only on errors.

- **Git:** `git` is available on your `PATH` (an error if not), and whether the current directory is inside a Git repository (a warning if not).
- **Project config:** whether `.cwizard.toml` (or another supported name and format) is found, and whether it parses (an error if malformed). A missing config is only informational: the default policy is used.
- **Competing config files:** a warning when more than one project config exists, naming the one in use and the ones ignored.
- **Unknown keys:** a warning for top-level keys `cw` does not recognise in the project or global config, with a "did you mean" suggestion.
- **Rules and global config:** the project rules file and the global config and rules, each reported found or malformed.
- **Registries:** which registries loaded, and a warning for each one that failed to load. Registries are read from their cache or fetched as the config says; `doctor` does not run a separate connectivity test.

After the checks, `doctor` prints the config sources it found (project, global, environment, CLI), the registry pool, the saved registry state and the AI settings.

## Global flags

All [global flags](global-flags.md) are supported.
