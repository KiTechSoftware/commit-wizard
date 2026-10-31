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
| `--ci` | | bool | `false` | Strict non-interactive mode: no questions, defaults accepted. Conflicts with `--non-interactive` |
| `--non-interactive` | | bool | `false` | Never ask; answers must come from flags. Conflicts with `--ci` |
| `--yes` | `-y` | bool | `false` | Accept the default answer to any question that has one |
| `--help` | `-h` | bool | | Show help for `cw` or for any command (`cw tag --help`) |
| `--version` | `-V` | bool | | Print the `cw` version |
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

## Guided by default, scriptable by flag

On a terminal, cw is **guided**: a command that is missing information asks for it (`cw commit` walks you through the message, `cw add` shows a file picker, `cw changelog` offers a menu). Every question can also be answered up front with a flag, and giving the flags skips the question. That is the path for scripts and for anyone who already knows what they want.

cw does not ask in these situations, and runs as if `--non-interactive` was given:

- standard input or output is **not a terminal** (a script, a pipe, a cron job, a CI runner);
- the output is **machine-readable** (`--json`);
- `--non-interactive` or `--ci` is passed.

If a command still needs an answer it cannot get, it stops with `Interactive input required` (or a message such as `type is required in non-interactive mode`) and names the flag to pass. It never waits for input that cannot arrive.

| Flag | Behaviour |
| --- | --- |
| `--yes` / `-y` | Accepts the **default** answer to any question that has one (`Continue? [Y/n]`, `Remote [origin]`). A question with no default — such as which commit type to use — is never guessed; it fails and tells you which flag to pass. |
| `--ci` | Strict, non-interactive mode for pipelines: no questions, defaults accepted where they exist. |
| `--non-interactive` | No questions at all, even ones with defaults. Anything that needs an answer must be passed as a flag. |

```sh
cw commit                              # guided on a terminal
cw commit -t feat -m "add parser"      # no questions: flags give every answer
cw changelog --yes                     # take the defaults instead of the menu
cw commit --non-interactive            # error: type is required in non-interactive mode
```
