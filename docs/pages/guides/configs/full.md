# Configuration: Full

Reference configuration with every supported field in v0.0.2. This is not meant to be used as-is; it's a reference for what's available.

```toml
version = 1

# ── Commit configuration ────────────────────────────────────────────

[commit]
subject_max_length = 72
use_emojis = true

# Define all commit types with full metadata
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

[commit.types.security]
emoji = "🔐"
description = "Security fix"
bump = "patch"
section = "Security"

[commit.types.perf]
emoji = "⚡"
description = "Performance improvement"
bump = "patch"
section = "Performance"

[commit.types.docs]
emoji = "📚"
description = "Documentation changes"
bump = "patch"
section = "Documentation"

[commit.types.style]
emoji = "🎨"
description = "Code style changes (no logic change)"
bump = "patch"
section = "Styling"

[commit.types.refactor]
emoji = "♻️"
description = "Code refactoring (no feature or fix)"
bump = "patch"
section = "Refactoring"

[commit.types.test]
emoji = "✅"
description = "Test additions or changes"
bump = "patch"
section = "Tests"

[commit.types.chore]
emoji = "🔧"
description = "Maintenance, dependencies, config"
bump = "patch"
section = "Chores"

[commit.types.ci]
emoji = "⚙️"
description = "CI/CD configuration"
bump = "patch"
section = "CI"

[commit.types.release]
emoji = "🚀"
description = "Release preparation"
bump = "patch"
section = "Release"

# Scope configuration
[commit.scopes]
mode = "required"  # "disabled", "optional", "required"
restrict_to_defined = true

[commit.scopes.definitions.api]
title = "API"
description = "REST API endpoints and services"
types = ["feat", "fix", "perf", "docs"]

[commit.scopes.definitions.auth]
title = "Authentication"
description = "User authentication and authorization"
types = ["feat", "fix", "security"]

[commit.scopes.definitions.ui]
title = "User Interface"
description = "Frontend, components, styling"
types = ["feat", "fix", "style", "docs"]

[commit.scopes.definitions.database]
title = "Database"
description = "Database schema, migrations, queries"
types = ["feat", "fix", "perf"]

[commit.scopes.definitions.config]
title = "Configuration"
description = "Application configuration"
types = ["chore", "docs"]

[commit.scopes.definitions.infra]
title = "Infrastructure"
description = "CI/CD, deployment, docker"
types = ["chore", "ci"]

[commit.scopes.definitions.deps]
title = "Dependencies"
description = "Dependency updates"
types = ["chore"]

# Breaking change configuration
[commit.breaking]
require_header = true  # Require !: in type(scope)
require_footer = true  # Require BREAKING CHANGE: footer
footer_key = "BREAKING CHANGE"
footer_keys = ["BREAKING", "BREAKING-CHANGE"]
emoji = "💥"
emoji_mode = "prefix"  # "prefix", "suffix", "replace"

# Commit protection on protected branches
[commit.protected]
allow = false        # Allow commits to protected branches
force = false        # Allow amend on protected branches
warn = true          # Warn when attempting

# Ticket reference configuration
[commit.ticket]
required = true
pattern = "([A-Z]+-\\d+|#\\d+)"  # PROJ-123 or #123
source = "branch_or_prompt"  # "branch", "prompt", "branch_or_prompt", "disabled"
header_format = "[{ticket}] "  # Prefix subject with ticket


# ── Branch configuration ────────────────────────────────────────────

[branch]
remote = "origin"
protected = ["main", "master", "staging", "release/*", "hotfix/*"]

[branch.naming]
pattern = "^(feature|bugfix|hotfix|release)/[a-z0-9-]+(/[a-z0-9-]+)*$"


# ── PR configuration ────────────────────────────────────────────────
# (Basic structure defined; full support in v0.0.3+)

[pr]
enabled = true

[pr.title]
require_conventional = true
require_ticket = true
scope_mode = "optional"  # "disabled", "optional", "required"

[pr.branch]
check_source = true
check_target = true
source_pattern = "^(feature|bugfix|hotfix)/.*"
target_allowed = ["main", "develop"]


# ── Check configuration ─────────────────────────────────────────────

[check]
require_conventional = true

[check.commits]
enabled = true
enforce_on = "protected_branches"  # "all_branches", "protected_branches", "none"


# ── Push configuration ──────────────────────────────────────────────

[push.allow]
protected = false  # Allow push to protected branches
force = false      # Allow force push

[push.check]
commits = true         # Validate commits before push
branch_policy = true   # Check branch naming/protection


# ── Versioning configuration ────────────────────────────────────────

[versioning]
tag_prefix = "v"


# ── Changelog configuration ─────────────────────────────────────────

[changelog]
output = "CHANGELOG.md"
format = "markdown"  # "markdown" or "json"

[changelog.header]
use = true
title = "Changelog"
description = "All notable changes to this project"

[changelog.layout]
group_by = ["type"]  # "type", "scope"
section_order = ["feat", "fix", "security", "perf", "docs", "refactor", "test", "chore", "ci"]
scope_order = ["api", "auth", "ui", "database", "config", "infra"]
show_scope = true
show_empty_sections = false
show_empty_scopes = false
misc_section = "Miscellaneous"
unreleased_label = "Unreleased"
date_format = "2006-01-02"

[changelog.sections.feat]
title = "Features"
order = 0

[changelog.sections.fix]
title = "Bug Fixes"
order = 1

[changelog.sections.security]
title = "Security"
order = 2

[changelog.sections.perf]
title = "Performance"
order = 3

[changelog.sections.docs]
title = "Documentation"
order = 10

[changelog.sections.refactor]
title = "Refactoring"
order = 11

[changelog.sections.test]
title = "Tests"
order = 12

[changelog.sections.chore]
title = "Chores"
order = 13


# ── Release configuration (v0.0.3+) ────────────────────────────────

[release]
enabled = false
source_branch = "main"
target_branch = "main"
branch_format = "release/{version}"
hotfix_pattern = "hotfix/*"

[release.validation]
require_clean_worktree = true
fail_if_tag_exists = true
fail_if_release_branch_exists = true

[release.finish]
tag = true
push = true
backmerge_branch = "main"


# ── Registry configuration ──────────────────────────────────────────

[registry]
use = "cw"
# no section = root-level config
section = "angular"  # Optional: specify a section to use from the registry
[registries.cw]
url = "https://github.com/KiTechSoftware/commit-wizard-registry.git"
ref = "main"
sections = ["minimal", "angular", "gitmoji"]

[registries.team-config]
url = "/opt/company/team-config"  # Local registry
section = "shared"


# ── Rules (reusable values) ─────────────────────────────────────────

[rules]
# Define reusable values referenced as @rules.<key>
allowed_scopes = ["api", "auth", "ui", "database", "config", "infra"]
ticket_formats = ["PROJ-\\d+", "#\\d+"]
release_checklist = ["tests_pass", "changelog_updated", "docs_updated"]
team_on_call = "security-team@company.com"
```

## Field categories

### Behavior fields (affect how `cw` executes)

- `commit.scopes.mode` — How scopes are handled
- `commit.breaking.require_*` — Breaking change requirements
- `branch.protected` — Which branches are protected
- `check.commits.enforce_on` — When validation is required
- `push.allow.protected` — Push restrictions

### Appearance fields (affect output and prompts)

- `commit.types.<type>.emoji` — Emoji display
- `commit.use_emojis` — Enable emoji in prompts
- `changelog.layout.section_order` — Changelog section ordering

### Integration fields (connect to external systems)

- `registry` — Use a shared registry
- `commit.ticket.source` — Extract tickets from branch names
- `commit.ticket.pattern` — Validate ticket IDs

### Descriptive fields (metadata)

- `commit.types.<type>.description` — Type description
- `commit.scopes.definitions.<scope>.title` — Scope label
- `changelog.sections.<type>.title` — Changelog section title

## Minimal activation

Don't feel obligated to set everything. Start with:

```toml
version = 1

[commit.scopes]
mode = "required"
restrict_to_defined = true

[commit.scopes.definitions.api]
title = "API"

[push.allow]
protected = false
```

Then add more as your team evolves.

## Field constraints

- **`pattern` fields** — Must be valid regex
- **`emoji` fields** — Single character or emoji
- **`topic` fields** — Lowercase alphanumeric + hyphens
- **`url` fields** — Git URL or local filesystem path

## Defaults for fields not specified

Any field not in your `.cwizard.toml` uses the built-in default. See [Public API Reference](../public-api.md) for all defaults.

## Override precedence

```sh
CLI > ENV > .cwizard.toml > registry > global config > defaults
```

Example: Set default via config, override in CI:

```toml
[commit.scopes]
mode = "optional"
```

Then in CI:

```sh
cw commit --ci --scope api  # Explicit override
CW_COMMIT_SCOPES_MODE=required cw commit  # ENV override
```

## To generate this

```sh
cw init --full
```

This creates a `.cwizard.toml` with all available fields (commented out as defaults).

## Next steps

- **For your team:** See [Recommended Configs](./recommended.md)
- **To use registries:** See [With Registry](./with-registry.md)
- **To define rules:** See [With Rules](./with-rules.md)
- **For field reference:** See [Public API Reference](../public-api.md)
