# Configuration: Minimal

The absolute minimum configuration needed to use Commit Wizard.

```toml
version = 1
```

That's it. All other settings use built-in defaults.

## What you get with this

- Commit types: `feat`, `fix`, `chore`, `docs`, `style`, `refactor`, `perf`, `test`
- Scope: optional, unrestricted
- Subject length: 72 characters max
- No ticket requirements
- Protected branches: `main`, `master`
- Changelog: `CHANGELOG.md` in Markdown format
- Versioning: tags with `v` prefix

## To generate this

```sh
cw init --minimal
```

Or manually create `.cwizard.toml` with just `version = 1`.

## When to use this

- **Personal projects** where you just want consistent commit messages
- **Experimentation** before committing to more structure
- **CI gates** that just need to check compliance, not enforce structure

## To extend

Add sections as needed. For example, to require scopes:

```toml
version = 1

[commit.scopes]
mode = "required"

[commit.scopes.definitions.api]
title = "API"

[commit.scopes.definitions.ui]
title = "UI"
```

See [Recommended Configs](./recommended.md) for team-specific starting points.
