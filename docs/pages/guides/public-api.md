# Public API Reference

This page documents the complete configuration schema for Commit Wizard v0.0.2. All fields listed here are supported and stable.

## Top-level structure

```toml
version = 1  # Optional; currently always 1

[commit]
[branch]
[pr]
[check]
[push]
[versioning]
[changelog]
[registry]
[registries]
[rules]
```

---

## `[commit]` — Commit message policy

Controls commit type definitions, scope requirements, subject length, and breaking change handling.

### `commit.subject_max_length`

**Type:** `u32`  
**Default:** `72`  
**Description:** Maximum length for the commit subject line.

```toml
[commit]
subject_max_length = 72
```

### `commit.use_emojis`

**Type:** `bool`  
**Default:** `false`  
**Description:** Include emoji in commit types when displaying options.

```toml
[commit]
use_emojis = true  # Shows ✨ next to feat, 🐛 next to fix, etc.
```

### `commit.types.<type>`

**Description:** Define allowed commit types and their versioning impact.

**Per-type fields:**

- `emoji` (string) — Emoji representation
- `description` (string) — Human-readable description
- `bump` (enum: `"major"`, `"minor"`, `"patch"`) — Semantic version impact
- `section` (string) — Changelog section name

```toml
[commit.types.feat]
emoji = "✨"
description = "A new feature"
bump = "minor"
section = "Features"

[commit.types.fix]
emoji = "🐛"
description = "A bug fix"
bump = "patch"
section = "Bug Fixes"

[commit.types.docs]
emoji = "📚"
description = "Documentation changes"
bump = "patch"
section = "Documentation"

[commit.types.chore]
emoji = "🔧"
description = "Maintenance tasks"
bump = "patch"
section = "Chores"
```

**Built-in defaults:** `feat`, `fix`, `chore`, `docs`, `style`, `refactor`, `perf`, `test`

You can add custom types:

```toml
[commit.types.release]
emoji = "🚀"
description = "Release preparation"
bump = "patch"
section = "Release"
```

### `commit.scopes`

Controls scope availability and constraints.

#### `commit.scopes.mode`

**Type:** enum (`"disabled"`, `"optional"`, `"required"`)  
**Default:** `"optional"`

```toml
[commit.scopes]
mode = "optional"  # Users may include a scope but aren't required
mode = "required"  # Every commit needs a scope
mode = "disabled"  # Hide scope field entirely
```

#### `commit.scopes.restrict_to_defined`

**Type:** `bool`  
**Default:** `false`  
**Description:** If `true`, users can only use scopes defined in `[commit.scopes.definitions]`.

```toml
[commit.scopes]
restrict_to_defined = true
```

#### `commit.scopes.definitions.<scope>`

**Description:** Define allowed scopes with metadata.

**Per-scope fields:**

- `title` (string) — Display name
- `description` (string) — Scope purpose
- `types` (array of strings, optional) — Limit this scope to specific commit types

```toml
[commit.scopes.definitions.api]
title = "API"
description = "REST API endpoints and services"
types = ["feat", "fix"]  # Only these types can use this scope

[commit.scopes.definitions.ui]
title = "User Interface"
description = "Frontend components and styling"

[commit.scopes.definitions.db]
title = "Database"
description = "Schema changes, migrations"
```

### `commit.breaking`

Controls breaking change handling.

#### `commit.breaking.require_header`

**Type:** `bool`  
**Default:** `false`  
**Description:** If `true`, breaking changes must use the `!` suffix after scope.

```toml
[commit.breaking]
require_header = true
# Users must write: feat(api)!: remove old endpoint
```

#### `commit.breaking.require_footer`

**Type:** `bool`  
**Default:** `false`  
**Description:** If `true`, breaking changes must include a `BREAKING CHANGE:` footer.

```toml
[commit.breaking]
require_footer = true
# Users must add: BREAKING CHANGE: The old endpoint is removed
```

#### `commit.breaking.footer_key`

**Type:** `string`  
**Default:** `"BREAKING CHANGE"`  
**Description:** The footer token to use for breaking change description.

```toml
[commit.breaking]
footer_key = "BREAKING CHANGE"
```

#### `commit.breaking.footer_keys`

**Type:** `array of strings`  
**Default:** `[]`  
**Description:** Additional footer tokens to recognize as breaking changes.

```toml
[commit.breaking]
footer_keys = ["BREAKING", "BREAKING-CHANGE"]
```

#### `commit.breaking.emoji`

**Type:** `string`  
**Default:** `"💥"`  
**Description:** Emoji to display for breaking changes.

```toml
[commit.breaking]
emoji = "💥"
```

#### `commit.breaking.emoji_mode`

**Type:** enum (`"prefix"`, `"suffix"`, `"replace"`)  
**Default:** `"prefix"`  
**Description:** Where to place the breaking change emoji in output.

```toml
[commit.breaking]
emoji_mode = "prefix"   # 💥 feat(api): ...
emoji_mode = "suffix"   # feat(api)! 💥: ...
emoji_mode = "replace"  # 💥(api): ...
```

### `commit.protected`

Controls commit protection on protected branches.

#### `commit.protected.allow`

**Type:** `bool`  
**Default:** `true`  
**Description:** Allow committing to protected branches.

#### `commit.protected.force`

**Type:** `bool`  
**Default:** `false`  
**Description:** Allow `git commit --amend` on protected branches.

#### `commit.protected.warn`

**Type:** `bool`  
**Default:** `true`  
**Description:** Warn when committing to a protected branch.

```toml
[commit.protected]
allow = false       # Prevent commits to protected branches
force = false       # No amend on protected branches
warn = true         # Show warning if attempting
```

### `commit.ticket`

Ticket reference requirements (e.g., Jira ticket, GitHub issue number).

#### `commit.ticket.required`

**Type:** `bool`  
**Default:** `false`

#### `commit.ticket.pattern`

**Type:** `string` (regex)  
**Default:** none  
**Description:** Regex pattern for valid ticket IDs.

```toml
[commit.ticket]
required = true
pattern = "PROJ-\\d+"    # Matches: PROJ-123, PROJ-999
pattern = "#\\d+"        # Matches: #123, #999
pattern = "([A-Z]+-\\d+|#\\d+)"  # Multiple formats
```

#### `commit.ticket.source`

**Type:** enum (`"branch"`, `"prompt"`, `"branch_or_prompt"`, `"disabled"`)  
**Default:** `"disabled"`  
**Description:** Where to extract ticket references from.

```toml
[commit.ticket]
source = "branch"           # Extract from branch name: feature/PROJ-123/my-feature
source = "prompt"           # Ask user
source = "branch_or_prompt" # Try branch first, prompt if not found
source = "disabled"         # Don't look for tickets
```

#### `commit.ticket.header_format`

**Type:** `string`  
**Default:** `"[{ticket}] "`  
**Description:** Format string for the ticket reference in the subject. Use `{ticket}` as placeholder.

```toml
[commit.ticket]
header_format = "[{ticket}] "     # [PROJ-123] my message
header_format = "({ticket}) "     # (PROJ-123) my message
header_format = "{ticket}: "      # PROJ-123: my message
```

---

## `[branch]` — Branch policy

### `branch.remote`

**Type:** `string`  
**Default:** `"origin"`  
**Description:** Default remote name for operations.

### `branch.protected`

**Type:** `array of strings`  
**Default:** `["main", "master"]`  
**Description:** Branch name patterns (glob-style) that are protected.

```toml
[branch]
protected = ["main", "master", "develop", "release/*", "hotfix/*"]
```

### `branch.naming.pattern`

**Type:** `string`  
**Default:** none  
**Description:** Required branch naming pattern (regex).

```toml
[branch.naming]
pattern = "^(feature|bugfix|hotfix)/[a-z0-9-]+$"
# Enforces: feature/my-feature, bugfix/issue-123, hotfix/critical
```

---

## `[pr]` — Pull Request policy

**Note:** Basic structure defined; full PR integration in v0.0.3+.

### `pr.enabled`

**Type:** `bool`  
**Default:** `false`

### `pr.title.require_conventional`

**Type:** `bool`  
**Default:** `false`  
**Description:** PR title must follow Conventional Commit format.

### `pr.title.require_ticket`

**Type:** `bool`  
**Default:** `false`  
**Description:** PR title must include a ticket reference.

### `pr.title.scope_mode`

**Type:** enum (`"disabled"`, `"optional"`, `"required"`)  
**Default:** `"optional"`

### `pr.branch.check_source`

**Type:** `bool`  
**Default:** `true`  
**Description:** Validate source branch naming.

### `pr.branch.check_target`

**Type:** `bool`  
**Default:** `true`  
**Description:** Validate target branch is allowed.

### `pr.branch.source_pattern`

**Type:** `string`  
**Default:** none

### `pr.branch.target_allowed`

**Type:** `array of strings`  
**Default:** `["main"]`

---

## `[check]` — Commit validation

### `check.require_conventional`

**Type:** `bool`  
**Default:** `true`  
**Description:** Checked commits must follow Conventional Commit format.

### `check.commits.enabled`

**Type:** `bool`  
**Default:** `true`

### `check.commits.enforce_on`

**Type:** enum (`"all_branches"`, `"protected_branches"`, `"none"`)  
**Default:** `"protected_branches"`  
**Description:** When to enforce strict checking.

```toml
[check.commits]
enforce_on = "protected_branches"  # Only enforce on main, master, etc.
enforce_on = "all_branches"        # Check every commit everywhere
enforce_on = "none"                # Disabled (but can be run manually)
```

---

## `[push]` — Push policy

### `push.allow.protected`

**Type:** `bool`  
**Default:** `false`  
**Description:** Allow pushing to protected branches.

### `push.allow.force`

**Type:** `bool`  
**Default:** `false`  
**Description:** Allow force push.

### `push.check.commits`

**Type:** `bool`  
**Default:** `true`  
**Description:** Run commit validation before push.

### `push.check.branch_policy`

**Type:** `bool`  
**Default:** `true`  
**Description:** Check branch naming and protection rules.

```toml
[push]
[push.allow]
protected = false
force = false

[push.check]
commits = true
branch_policy = true
```

---

## `[versioning]` — Semantic versioning

### `versioning.tag_prefix`

**Type:** `string`  
**Default:** `"v"`  
**Description:** Prefix for version tags.

```toml
[versioning]
tag_prefix = "v"      # Tags: v1.0.0, v1.1.0
tag_prefix = ""       # Tags: 1.0.0, 1.1.0
tag_prefix = "release-"  # Tags: release-1.0.0
```

---

## `[changelog]` — Changelog generation

### `changelog.output`

**Type:** `string` (file path)  
**Default:** `"CHANGELOG.md"`  
**Description:** Where to write the changelog.

### `changelog.format`

**Type:** enum (`"markdown"`, `"json"`)  
**Default:** `"markdown"`

### `changelog.header.use`

**Type:** `bool`  
**Default:** `true`  
**Description:** Include a header section in the changelog.

### `changelog.header.title`

**Type:** `string`  
**Default:** `"Changelog"`

### `changelog.header.description`

**Type:** `string`  
**Default:** `""`

### `changelog.layout.group_by`

**Type:** `array of strings`  
**Default:** `["type"]`  
**Description:** How to organize entries. Options: `"type"`, `"scope"`.

### `changelog.layout.section_order`

**Type:** `array of strings`  
**Description:** Order of sections (commit types) in changelog.

```toml
[changelog.layout]
section_order = ["feat", "fix", "perf", "docs", "refactor", "test", "chore"]
```

### `changelog.layout.scope_order`

**Type:** `array of strings`  
**Default:** `[]`  
**Description:** Order of scopes if grouping by scope.

### `changelog.layout.show_scope`

**Type:** `bool`  
**Default:** `true`

### `changelog.layout.show_empty_sections`

**Type:** `bool`  
**Default:** `false`  
**Description:** Include section headers even if no commits in that section.

### `changelog.layout.show_empty_scopes`

**Type:** `bool`  
**Default:** `false`

### `changelog.layout.misc_section`

**Type:** `string`  
**Default:** `"Miscellaneous"`  
**Description:** Section name for commits without a type.

### `changelog.layout.unreleased_label`

**Type:** `string`  
**Default:** `"Unreleased"`  
**Description:** Section name for commits not yet released.

### `changelog.layout.date_format`

**Type:** `string`  
**Default:** `"%Y-%m-%d"`  
**Description:** Date format string (strftime-compatible).

### `changelog.sections.<type>`

**Description:** Customize how sections appear in changelog.

```toml
[changelog.sections.feat]
title = "Features"
order = 0  # Display first

[changelog.sections.fix]
title = "Bug Fixes"
order = 1

[changelog.sections.docs]
title = "Documentation"
order = 10
```

---

## `[release]` — Release workflow (structural in v0.0.2, full support in v0.0.3+)

### `release.enabled`

**Type:** `bool`  
**Default:** `false`

### `release.source_branch`

**Type:** `string`  
**Default:** `"main"`

### `release.target_branch`

**Type:** `string`  
**Default:** `"main"`

### `release.branch_format`

**Type:** `string`  
**Default:** `"release/{version}"`

### `release.hotfix_pattern`

**Type:** `string`  
**Default:** `"hotfix/*"`

### `release.validation.require_clean_worktree`

**Type:** `bool`  
**Default:** `true`

### `release.validation.fail_if_tag_exists`

**Type:** `bool`  
**Default:** `true`

### `release.validation.fail_if_release_branch_exists`

**Type:** `bool`  
**Default:** `true`

### `release.finish.tag`

**Type:** `bool`  
**Default:** `true`

### `release.finish.push`

**Type:** `bool`  
**Default:** `true`

### `release.finish.backmerge_branch`

**Type:** `string`  
**Default:** `"main"`

---

## `[registry]` — Active registry selection

### `registry.use`

**Type:** `string`  
**Description:** Key name under `[registries]` to use.

### `registry.section`

**Type:** `string`  
**Description:** Section within the selected registry.

```toml
[registry]
use = "org-standards"
section = "frontend"
```

---

## `[registries]` — Available registries

Define multiple registries; select one via `[registry].use`.

```toml
[registries.org-standards]
url = "https://github.com/my-org/cw-standards.git"
ref = "main"
section = "shared"
sections = ["shared", "frontend", "backend"]

[registries.team-config]
url = "/opt/shared/team-registry"  # Local path
```

**Per-registry fields:**

- `url` (string) — Git URL or local path
- `ref` (string) — Git reference (branch, tag, commit)
- `section` (string) — Default section to use
- `sections` (array of strings, optional) — Available sections (informational)

---

## `[rules]` — Reusable values

Free-form TOML that defines values referenced elsewhere as `@rules.<key>`.

```toml
[rules]
allowed_scopes = ["api", "ui", "db", "auth"]
ticket_pattern = "PROJ-\\d+"
protected_branches = ["main", "staging"]
release_checklist = ["tests", "docs", "changelog"]
```

Then reference them:

```toml
[commit.scopes.definitions]
# Could reference @rules.allowed_scopes in future versions

[commit.ticket]
pattern = "@rules.ticket_pattern"
```

**Current status:** Rules are stored but full cross-field referencing coming in v0.0.3+.

---

## Summary of defaults

Here's a minimal `.cwizard.toml` that activates all defaults:

```toml
version = 1
```

And here's what's actually applied:

```toml
[commit]
subject_max_length = 72
use_emojis = false

[commit.types.feat]
emoji = "✨"
description = "A new feature"
bump = "minor"
section = "Features"

[commit.types.fix]
emoji = "🐛"
description = "A bug fix"
bump = "patch"
section = "Bug Fixes"

# ... (more built-in types)

[commit.scopes]
mode = "optional"
restrict_to_defined = false

[commit.breaking]
require_header = false
require_footer = false
footer_key = "BREAKING CHANGE"
emoji = "💥"
emoji_mode = "prefix"

[branch]
remote = "origin"
protected = ["main", "master"]

[check.commits]
enforce_on = "protected_branches"

[push.allow]
protected = false
force = false

[push.check]
commits = true
branch_policy = true

[versioning]
tag_prefix = "v"

[changelog]
output = "CHANGELOG.md"
format = "markdown"

[changelog.layout]
group_by = ["type"]
show_scope = true
misc_section = "Miscellaneous"
unreleased_label = "Unreleased"
date_format = "%Y-%m-%d"
```

---

## Customization paths

| Goal | Start here |
| --- | --- |
| Require scopes on all commits | `[commit.scopes]` |
| Add team-specific scope definitions | `[commit.scopes.definitions.<scope>]` |
| Enforce ticket references | `[commit.ticket]` |
| Restrict push to protected branches | `[push.allow]` |
| Customize changelog layout | `[changelog.layout]` |
| Use shared registry config | `[registry]` + `[registries]` |
| Define reusable values | `[rules]` |

See [Configuration Guides](./configs/) for practical examples.
