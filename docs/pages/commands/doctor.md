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
| `fix` | Attempt safe, local repairs for detected issues |

## Examples

**Run diagnostics:**

```sh
cw doctor
```

**Automatically fix detected issues:**

```sh
cw doctor fix
```

**Run diagnostics in CI (non-interactive, structured output):**

```sh
cw doctor --ci --json
```

## What doctor checks

- Git installation and version compatibility
- Whether the current directory is inside a Git repository
- Presence and readability of project config (`.cwizard.toml`)
- Presence and readability of global config
- Registry connectivity (if configured)
- Config schema validity
- Known environment variable conflicts

## Global flags

All [global flags](../global-flags.md) are supported.
