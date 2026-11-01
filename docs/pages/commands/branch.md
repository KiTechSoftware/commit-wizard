# cw branch

List, switch, create and prune branches.

## Usage

```sh
cw branch [SUBCOMMAND]
```

Run bare on a terminal, `cw branch` opens the branch picker (the same as `cw branch switch`). Without a terminal it lists the branches.

| Subcommand | What it does |
| --- | --- |
| `list` | List branches with what they follow and where they stand |
| `switch [NAME]` | Switch to a branch, picking from a list when you give no name |
| `new [NAME]` | Create a branch named `<type>/<ticket>-<description>` and switch to it |
| `prune` | Delete local branches whose remote branch is gone |

## Listing branches

```sh
cw branch list              # local branches, most recently committed to first
cw branch list --remote     # remote-tracking branches, as of the last fetch
cw branch list --all        # both
```

Each local branch shows the remote branch it follows and its standing:

| Status | Meaning |
| --- | --- |
| `up to date` | Level with its remote branch |
| `ahead N` / `behind N` / `ahead N, behind M` | Commits not yet pushed, not yet pulled, or both |
| `remote deleted` | It follows a remote branch that no longer exists, as after a merged pull request is deleted. These are what `prune` removes. |
| *(blank)* | The branch follows nothing |

The current branch is marked `*`. Statuses reflect the last fetch; run `git fetch --prune` (or `cw branch prune`) to refresh them.

| Flag | Short | Description |
| --- | --- | --- |
| `--remote` | `-r` | Show remote-tracking branches instead of local ones |
| `--all` | `-a` | Show local and remote-tracking branches. Cannot be combined with `--remote` |

## Switching branches

```sh
cw branch switch                 # pick from a list (on a terminal)
cw branch switch feature/login   # no questions
```

The picker lists local branches with their status and last commit, and also branches that exist only on a remote. Choosing one of those creates a local branch that follows it. Without a terminal you must give the name; the error says so.

## Creating a branch

```sh
cw branch new --description "Add login page" --type feature --ticket PROJ-123
# → feature/PROJ-123-add-login-page
```

The name is built from a template, `branch.naming.pattern`, which defaults to `{type}/{ticket}-{slug}`:

| Part | From | Example |
| --- | --- | --- |
| `{type}` | `--type` / `-t`: `feature`, `bugfix`, `hotfix`, `chore`, `docs`, `refactor` and `release` are offered when you are asked, but any word works | `feature/` |
| `{ticket}` | `--ticket`, kept as you write it | `PROJ-123-` |
| `{slug}` | `--description` / `-d`: lower-case words joined by hyphens, cut to 50 characters | `add-login-page` |

What is required:

| Flag | Required? |
| --- | --- |
| `--description` / `-d` | Required **unless a ticket is given**. With `-T`, the ticket alone can name the branch (`bugfix/#42`). Without either, a terminal asks until you give a description, and without a terminal the error names `--description` and `--ticket`. |
| `--type` / `-t` | Optional, unless `branch.naming.require_type = true` |
| `--ticket` / `-T` | Optional, unless `branch.naming.require_ticket = true`; if you give one, `-d` is not needed |

A part you do not give is left out together with its separator, so with no ticket the default gives `feature/add-login-page`, and with neither type nor ticket just `add-login-page`.

Set the layout you want in the config; `{slug}` is required and other text is kept as written:

```toml
[branch.naming]
pattern = "{slug}"                    # add-login-page
pattern = "{type}/{slug}"             # feature/add-login-page
pattern = "{ticket}-{slug}"           # PROJ-123-add-login-page
pattern = "{type}/{ticket}-{slug}"    # feature/PROJ-123-add-login-page  (default)
```

To make branches in your project always carry a type, a ticket or both:

```toml
[branch.naming]
pattern = "{type}/{ticket}-{slug}"
require_type = true
require_ticket = true
```

A required part is asked for on a terminal (and re-asked if left empty), and without a terminal the error names the flag to pass. Requiring a part the template has no place for, such as `require_type` with `{slug}`, is a configuration error.

A flag for a part the template does not have, such as `--ticket` with `{type}/{slug}`, is refused rather than ignored. A template that is not one (an old regular expression, say) is an error that says so.

```sh
cw branch new "Quick fix"                          # same as --description "Quick fix"
cw branch new -d crash -t bugfix -T "#42"          # bugfix/#42-crash
cw branch new -t bugfix -T "#42"                   # bugfix/#42  (the ticket alone is enough)
cw branch new team/exact-name                      # used as written (it contains a /)
cw branch new -d "Try it" --from v1.2.0 --no-switch  # start from a tag, stay where you are
```

The description can also be given as the positional `NAME`, as a shorthand for `--description`; give one or the other. A `NAME` that contains a `/` is taken as the whole branch name, bypassing the template, so it cannot be combined with `--type` or `--ticket`.

On a terminal, anything you leave out is asked, and only for parts the template has: the type (skippable unless required), then the ticket (skippable unless required), then what you are working on. The description is required when you skipped the ticket and optional when you gave one, then a final confirmation. The ticket is checked against `commit.ticket.pattern`. A branch named with the ticket lets `cw commit` find it when `commit.ticket.source` is `branch`.

The branch is refused if the name is not a valid Git branch name or already exists.

| Flag | Short | Description |
| --- | --- | --- |
| `--description <TEXT>` | `-d` | What the branch is for; becomes the `{slug}` part. Not needed when a ticket is given |
| `--type <TYPE>` | `-t` | Kind of branch, used as the `{type}` part |
| `--ticket <TICKET>` | `-T` | Ticket reference, used as the `{ticket}` part |
| `--from <REF>` | | Commit, branch or tag to start from (default: the current commit) |
| `--no-switch` | | Create the branch but stay on the current one |

## Pruning branches

After a pull request is merged and its branch deleted on the remote, your local copy lingers. `cw branch prune` removes those:

```sh
cw branch prune --dry-run   # see what would go
cw branch prune             # list them and ask
cw branch prune --yes       # delete without asking
```

It first runs `git fetch --prune` to learn what the remotes have (this refreshes your remote-tracking branches, like `git fetch`; pass `--no-fetch` to skip it, or if you are offline it carries on with what it already knows). Then it looks at local branches whose remote branch is gone:

| Branch | What happens |
| --- | --- |
| Merged into the branch you are on | Deleted |
| Has commits found nowhere else | **Kept**, and listed as needing `--force`. A branch merged by squash looks like this, but the work may also exist nowhere else. |
| The current branch | Never touched |
| A protected branch (`branch.protected`; `main`, `master` and `release/*` by default) | Never touched |

Each deletion prints the command that restores it, such as `git branch feature/login 1a2b3c4`. Without a terminal, deleting needs `--yes` (or use `--dry-run` to preview), so a script cannot delete by accident. `--dry-run` deletes nothing, though it still fetches.

| Flag | Description |
| --- | --- |
| `--force` | Also delete branches with commits that are not merged anywhere else |
| `--no-fetch` | Do not fetch first; use what the last fetch found |

## Global flags

All [global flags](global-flags.md) are supported. `--dry-run` previews `new`, `switch` and `prune`; `--yes` answers the confirmation questions; `--json` gives machine-readable output.
