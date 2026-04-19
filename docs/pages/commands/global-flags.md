# Global Flags

The following flags are available on every `cw` subcommand.

| Flag | Short | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `--verbose` | `-v` | count | `0` | Increase log verbosity. Repeat for more detail: `-v` = INFO, `-vv` = DEBUG, `-vvv` = TRACE |
| `--quiet` | `-q` | count | `0` | Decrease log verbosity. `-q` = WARN, `-qq` = ERROR only |
| `--json` | | bool | `false` | Wrap all output in a JSON envelope |
| `--format` | | enum | `auto` | Output payload format: `auto`, `text`, `markdown`, `json`, `jsonl`, `plain` |
| `--plain` | `-p` | bool | `false` | Equivalent to `--format plain`; conflicts with `--format` |
| `--dry-run` | | bool | `false` | Simulate the action without making any changes |
| `--color` | | enum | `auto` | Color policy: `auto`, `always`, `never` |
| `--cwd` | `-C` | path | `.` | Run as if started from this directory |
| `--ci` | | bool | `false` | Non-interactive mode, assume yes. Conflicts with `--non-interactive` |
| `--non-interactive` | | bool | `false` | Non-interactive mode, assume no. Conflicts with `--ci` |
| `--yes` | `-y` | bool | `false` | Accept all defaults without prompting |
| `--force` | | bool | `false` | Override protections (e.g. push to a protected branch) |
| `--config <PATH>` | | path | | Use a specific config file instead of the auto-discovered one |
| `--registry <URL>` | | string | | Use a specific registry (local path or Git URL) |
| `--registry-ref <REF>` | | string | | Registry Git ref (branch, tag, or commit SHA) |
| `--registry-section <NAME>` | | string | | Section within the registry to use |

## Verbosity levels

| Flags | Tracing level | What you see |
| --- | --- | --- |
| `-qq` | ERROR | Fatal errors only |
| `-q` | WARN | Warnings and errors |
| *(none)* | WARN | Default (same as `-q`) |
| `-v` | INFO | Progress messages |
| `-vv` | DEBUG | Internal decisions and config resolution |
| `-vvv` | TRACE | Everything |

You can also set `RUST_LOG` directly to override the level (e.g. `RUST_LOG=debug cw check`).

## Non-interactive modes

| Flag | Behaviour |
| --- | --- |
| `--ci` | Assumes **yes** to all prompts. Exits non-zero if any check fails. |
| `--non-interactive` | Assumes **no** to all prompts. |
| `--yes` | Assumes **yes** to prompts, but still allows interactive output where possible. |
