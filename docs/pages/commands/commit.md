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
| `--allow-empty` | | Allow committing with nothing staged |

## Commit message anatomy

```sh
<type>(<scope>): <subject>

<body>

<footer-token>: <footer-value>
BREAKING CHANGE: <breaking-message>
```

Example:

```sh
feat(api): add pagination support

Adds cursor-based pagination to the /users endpoint.
Clients should pass `cursor` and `limit` query params.

Refs: #88
```

## Scope behaviour

Scope behaviour is controlled by `commit.scopes.mode` in config:

| Mode | Effect |
| --- | --- |
| `disabled` | Scope field is hidden entirely |
| `optional` | Scope is prompted but can be skipped |
| `required` | A scope must be provided |

When `commit.scopes.restrict_to_defined = true`, only scopes defined in `[commit.scopes.definitions]` are accepted.

## Global flags

All [global flags](../global-flags.md) are supported. `--dry-run` prints the final commit message without running `git commit`.
