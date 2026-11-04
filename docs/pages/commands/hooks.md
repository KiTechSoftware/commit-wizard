# cw hooks

Install, remove and inspect the Git hooks that run `cw`.

## Usage

```sh
cw hooks install [--hook commit-msg|pre-push]...
cw hooks uninstall [--hook commit-msg|pre-push]...
cw hooks status
```

## Hooks

| Hook | Runs |
| --- | --- |
| `commit-msg` | `cw check --message-file` on each message as it is written |
| `pre-push` | `cw check` on the commits about to be pushed |

With no `--hook`, `install` uses the hooks enabled under `[hooks]`, or both when none are. Hooks go where Git looks for them (`core.hooksPath` is honoured). Each file `cw` writes carries a marker line: an existing hook without it is never replaced unless you pass the global `--force`, and `uninstall` only removes hooks `cw` wrote. `--dry-run` previews.

If `cw` is not on `PATH`, a hook prints a note and lets the operation continue.
