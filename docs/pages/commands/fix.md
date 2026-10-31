# cw fix

Correct commit messages that break the project's rules. Run it on a terminal and it walks you through each offending commit; in scripts, give the corrected first line with `--reword`.

## Usage

```sh
cw fix [OPTIONS]
```

## Examples

**See what is wrong, changing nothing:**

```sh
cw fix --dry-run
```

**Be guided through each offending commit (on a terminal):**

```sh
cw fix
```

**Fix specific commits without prompts:**

```sh
cw fix --reword 1a2b3c4d="fix: handle empty input" --reword 9f8e7d6c="feat: add a button"
```

**Look further back than the unpublished commits:**

```sh
cw fix --from v1.2.0
```

## What it changes — and what it does not

Only commit **messages** change. Each rewritten commit keeps its files, author, committer and dates exactly; `cw fix` checks that the new branch tip has the same files as the old one before it moves anything. Your working tree and index are never touched.

Because a commit's hash depends on its message and its parent, the fixed commits **and every commit after them** get new hashes. Commits before the first fix keep theirs. The commit's body and footers (`Signed-off-by:`, `Refs:`, …) are kept; only the first line is replaced, plus a `BREAKING CHANGE:` footer if you mark the change as breaking.

## Guided mode

For each commit that breaks the rules, oldest first, `cw fix` shows the hash, the message, and why it is wrong, then asks:

1. **What would you like to do?** — *Reword it*, *Skip it*, or *Stop here* (keep what you have decided so far).
2. **Which type is it?** — the type the wording hints at is listed first and marked *(suggested)*: "Fixed the parser" suggests `fix`.
3. **Scope** — asked only if the project allows or requires scopes.
4. **Short subject** — pre-filled with the old message minus any bad prefix, so Enter keeps it.
5. **Is this a breaking change?** — and, if so, a description for the `BREAKING CHANGE:` footer.

If your new message still breaks a rule (a missing required scope, say), it tells you which and asks again. When you are done it lists every change and asks **"Rewrite N message(s)?"** — answering no rewrites nothing.

## Which commits are checked

By default, only commits that are **not on any remote**, since those are the only ones nobody else can have. Use `--from <ref>` to check from a tag, branch or hash instead, and `--tail N` to look only at the newest N.

## Safety

| Situation | What happens |
| --- | --- |
| A commit to fix is already on a remote | **Refused** (exit code 21): rewriting it would break everyone who pulled it. The global `--force` overrides, and cw reminds you to push with `git push --force-with-lease`. |
| A commit in the range is GPG-signed | **Refused**: a rewrite invalidates signatures. `--force` rewrites it anyway; the commits come out unsigned. |
| A merge commit lies in the range | **Refused**: only straight-line history is rewritten. |
| `HEAD` is detached, or a merge, rebase or cherry-pick is in progress | **Refused**. |
| Any `--reword` is itself invalid | **Nothing is rewritten.** Every correction is checked before the first commit is touched. |
| You change your mind | The old tip is saved under `refs/cw/fix-backup/<branch>/<time>` before the branch moves, and the exact undo command is printed: `git update-ref refs/heads/<branch> <old tip>`. |

## Options

| Flag | Description |
| --- | --- |
| `--from <REF>` | Check the commits after this one up to `HEAD`, instead of the unpublished commits |
| `--tail <N>` / `--count <N>` | Only look at the newest N commits (0 = all) |
| `--reword <HASH=HEADER>` | The corrected first line for one commit (repeatable). `HASH` needs at least 4 characters and must identify one offending commit |

## Without a terminal

In scripts and CI, and with `--yes`, `--ci` or `--non-interactive`, there are no questions. Without `--reword` the command stops and says so; `--dry-run` simply lists what is wrong and exits 0. There is no automatic mode that invents messages: only a person can say what a commit meant.

## Global flags

All [global flags](./global-flags.md) are supported. `--dry-run` previews the change without rewriting, and `--force` overrides the published-commit and signed-commit guards.
