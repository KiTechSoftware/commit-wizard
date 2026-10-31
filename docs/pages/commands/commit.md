# cw commit

Interactively guides you through writing a valid [Conventional Commit](https://www.conventionalcommits.org) message, then commits staged changes. You can also bypass the wizard entirely by supplying flags directly.

## Usage

```sh
cw commit [OPTIONS]
```

## Examples

**Full interactive wizard** — prompts for type, scope, message, and more:

```sh
cw commit
```

**Supply everything on the command line (no prompts):**

```sh
cw commit --type feat --scope api --message "add pagination support"
```

**Breaking change with a description:**

```sh
cw commit --type feat --scope auth --message "remove password login" \
  --breaking --breaking-message "Password login has been removed in favour of SSO"
```

**Commit with body and footer:**

```sh
cw commit -t fix -m "handle nil pointer in parser" \
  -b "The parser was not guarding against nil input on line 42." \
  -f "Fixes: #123"
```

**Stage and commit in one step** — `-a` stages changes to files git already tracks (like `git commit -a`), `-A` stages everything including new files:

```sh
cw commit -a -t fix -m "handle nil pointer in parser"
cw commit -A -t feat -m "add the parser module"
```

**Commit with a ticket reference:**

```sh
cw commit -t feat -m "add ticketing" --ticket Ticket-012
# → feat: Ticket-012: add ticketing
```

**Commit with no staged changes** (useful for empty commits in CI):

```sh
cw commit --allow-empty -t chore -m "trigger pipeline"
```

**Dry-run — see the commit message without committing:**

```sh
cw commit --dry-run -t feat -m "my new feature"
```

## Options

| Flag | Short | Description |
| --- | --- | --- |
| `--type <TYPE>` | `-t` | Commit type (`feat`, `fix`, `chore`, etc.) |
| `--scope <SCOPE>` | `-s` | Commit scope |
| `--message <MSG>` | `-m` | Short summary (subject line, max 72 chars by default) |
| `--breaking` | `-B` | Mark as a breaking change |
| `--breaking-message <MSG>` | `-d` | Breaking change description (goes in footer) |
| `--body <BODY>` | `-b` | Commit message body (freeform, after a blank line) |
| `--footer <FOOTER>` | `-f` | Footer token(s); may be repeated (e.g. `"Refs: #42"`) |
| `--ticket <TICKET>` | | Ticket reference, written at the start of the subject; see [Tickets](#tickets) |
| `--tracked` | `-a` | Stage changes to tracked files first, like `git commit -a` (new files are left out) |
| `--all` | `-A` | Stage every change first, new files included. Cannot be combined with `-a` |
| `--allow-empty` | | Allow committing with nothing staged |

Staging happens just before the commit is created, so cancelling at the final question, or using `--dry-run`, stages nothing.

## Commit message anatomy

```sh
<type>(<scope>): <subject>

<body>

BREAKING CHANGE: <breaking-message>
<footer-token>: <footer-value>
```

Example:

```sh
feat(api): add pagination support

Adds cursor-based pagination to the /users endpoint.
Clients should pass `cursor` and `limit` query params.

Refs: #88
```

## Tickets

`cw` assumes no ticketing system. What a ticket looks like is up to your `commit.ticket.pattern` (a Jira key, `#123`, `Ticket-012`, anything), and the ticket goes at the start of the subject, after the type and scope:

```text
feat: Ticket-012: add ticketing
fix(api): PROJ-123: handle nil pointer
```

Where the ticket comes from is set by `commit.ticket.source`, and `--ticket` always wins:

| `source` | What happens |
| --- | --- |
| `branch` | Taken from the branch name using `pattern` (`feature/PROJ-123-login` gives `PROJ-123`) |
| `prompt` | You are asked on a terminal |
| `branch_or_prompt` | The branch if it matches, otherwise you are asked |
| `disabled` (default) | Nothing is looked for, unless `commit.ticket.required` is on, in which case you are asked |

With `required = true`, a commit with no ticket is refused, and without a terminal the error names `--ticket`. A ticket that does not match `pattern` is refused. Without a `pattern`, any text counts as a ticket, but none can be found in a branch name. `commit.ticket.header_format` changes how the ticket is written (default `"{ticket}: "`; `"[{ticket}] "` gives `feat: [PROJ-123] add parser`), and the ticket counts towards the subject length limit. See [`commit.ticket`](../guides/public-api.md#committicket).

## Scope behaviour

Scope behaviour is controlled by `commit.scopes.mode` in config:

| Mode | Effect |
| --- | --- |
| `disabled` | Scopes are not used: the question is never asked, and `--scope` is refused |
| `optional` | On a terminal you are asked whether to add a scope (the default answer is no); with flags, a scope is added only if you pass `--scope` |
| `required` | A scope must be provided; without a terminal, a missing `--scope` is an error |

When `commit.scopes.restrict_to_defined = true`, only scopes defined in `[commit.scopes.definitions]` are accepted.

## Global flags

All [global flags](global-flags.md) are supported. `--dry-run` prints the final commit message without running `git commit`.
