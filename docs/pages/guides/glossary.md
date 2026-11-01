# Glossary

| Term | Meaning |
| --- | --- |
| **Conventional Commits** | The `type(scope): description` format for commit messages that `cw` checks and builds from |
| **Type** | The kind of change at the start of a message: `feat`, `fix`, `docs` and so on. `feat` and `fix` are always known |
| **Scope** | An optional area in brackets after the type, such as `feat(api):`. `commit.scopes.mode` decides whether one is `disabled`, `optional` or `required` |
| **Breaking change** | A change that needs a major version. Marked with `!` after the type or a `BREAKING CHANGE:` footer (uppercase) |
| **Ticket** | A reference to an issue, such as `PROJ-123`, checked against `commit.ticket.pattern` |
| **Tag prefix** | The text before a version in a tag, usually `v`. Set by `versioning.tag_prefix` |
| **Prerelease** | A version with a label, such as `1.4.0-rc.1` |
| **Protected branch** | A branch `cw` never deletes and treats carefully when pushing, from `branch.protected` |
| **Prune** | Delete local branches whose remote branch is gone |
| **Guided mode** | `cw` asks for what is missing when on a terminal |
| **Non-interactive** | `cw` never asks. Used for scripts and CI (`--ci`, `--non-interactive`) |
| **Dry run** | `--dry-run` shows what would change and changes nothing |
| **Project config** | The settings file in a repository |
| **Global config** | The settings file for your user account, edited with `cw config -g` |
| **Registry** | A shared Git repository of config sections that many projects can use. See the [registry guide](registry-system.md) |
| **Reserved setting** | A section that is accepted but not read by any command yet: `[pr]`, `[release]`, `[hooks]`, `[ai]` |
| **Error id** | The `E`-number on an error. See [errors and exit codes](exit-codes.md) |
