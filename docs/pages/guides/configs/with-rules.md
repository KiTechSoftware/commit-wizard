# Configuration: With Rules

Rules are reusable values defined in a `[rules]` section that can be referenced elsewhere in your config.

## What are rules?

Rules are free-form TOML values that you define once and reference via `@rules.<key>`.

**Benefits:**

- Define values once, use them multiple places
- Easy to update a value that's referenced in many places
- Share rules across projects via registries

## Basic example

```toml
version = 1

# Define rules under [rules]
[rules]
allowed_scopes = ["api", "ui", "db", "auth", "infra"]
ticket_pattern = "PROJ-\\d+"
team_slack_channel = "#dev-team"
```

## Using rules with patterns

**Current status (v0.0.2):** Rules are stored but cross-field references are coming in v0.0.3+.

In v0.0.2, use rules as a documentation and reference point. In v0.0.3+, you'll be able to do:

```toml
[commit.ticket]
pattern = "@rules.ticket_pattern"
```

But for now, manually reference the rule value:

```toml
[commit.ticket]
pattern = "PROJ-\\d+"  # From @rules.ticket_pattern
```

## Rules for different aspects

### Ticket and issue tracking

```toml
[rules]
# Ticket ID patterns
ticket_patterns = {
  jira = "PROJ-\\d+",
  github = "#\\d+",
  gitlab = "#\\d+"
}

# Ticket source mapping
ticket_required_on_branches = ["main", "release/*"]
```

### Scope definitions

```toml
[rules]
backend_scopes = ["api", "auth", "db", "cache"]
frontend_scopes = ["ui", "routing", "state", "styling"]
shared_scopes = ["config", "deps", "ci", "docs"]
all_scopes = ["api", "auth", "ui", "db", "routing", "state", "styling", "config", "deps", "ci", "docs", "cache"]
```

### Branch policies

```toml
[rules]
protected_branch_patterns = ["main", "master", "release/*", "hotfix/*", "staging"]
```

### Release checklist

```toml
[rules]
release_checks = [
  "tests_passing",
  "changelog_updated",
  "docs_updated",
  "security_review",
  "performance_baseline"
]
```

### Team info

```toml
[rules]
team_name = "platform"
team_members = ["alice", "bob", "charlie"]
team_lead = "alice"
on_call_rotation = "https://oncall.company.com/platform"
```

### Environment mappings

```toml
[rules]
environments = {
  dev = "https://dev.company.com",
  staging = "https://staging.company.com",
  prod = "https://company.com"
}
```

## Full example with rules

```toml
version = 1

# ── Centralized rule definitions ────────────────────────────────

[rules]
# Scopes
backend_scopes = ["api", "auth", "db", "queue"]
frontend_scopes = ["ui", "router", "state"]
shared_scopes = ["infra", "deps", "ci", "docs"]

# Tickets
ticket_jira = "PROJ-\\d+"
ticket_github = "#\\d+"

# Branches
protected_branches = ["main", "staging", "release/*"]

# Changelog
changelog_section_order = ["feat", "fix", "security", "perf", "docs", "refactor"]

# Team info
team = "backend"
team_lead_email = "alice@company.com"


# ── Usage in actual config ─────────────────────────────────────────

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
description = "Documentation"
bump = "patch"
section = "Documentation"

[commit.scopes]
mode = "required"
restrict_to_defined = true

# Define scopes (manually reference the rule)
[commit.scopes.definitions.api]
title = "API"
types = ["feat", "fix", "security", "perf"]

[commit.scopes.definitions.auth]
title = "Authentication"
types = ["feat", "fix", "security"]

[commit.scopes.definitions.db]
title = "Database"
types = ["feat", "fix"]

[commit.scopes.definitions.queue]
title = "Queue"
types = ["feat", "fix", "perf"]

[commit.scopes.definitions.infra]
title = "Infrastructure"
types = ["chore", "ci"]

[commit.scopes.definitions.deps]
title = "Dependencies"
types = ["chore"]

[commit.scopes.definitions.ci]
title = "CI/CD"
types = ["chore", "ci"]

[commit.scopes.definitions.docs]
title = "Documentation"
types = ["docs"]

# Ticket config (manually reference ticket_jira)
[commit.ticket]
required = true
pattern = "PROJ-\\d+"
source = "branch_or_prompt"
header_format = "[{ticket}] "

# Branch config (manually reference protected_branches)
[branch]
protected = ["main", "staging", "release/*"]

[branch.naming]
pattern = "^(feature|bugfix|hotfix|release)/[a-z0-9-]+(/[a-z0-9-]+)*$"

# Changelog (manually reference changelog_section_order)
[changelog]
output = "CHANGELOG.md"
format = "markdown"

[changelog.layout]
section_order = ["feat", "fix", "security", "perf", "docs", "refactor", "test", "chore"]
```

## Sharing rules across projects

Rules are especially useful when shared via a registry. All projects in your organization can use the same rules:

```toml
version = 1

[registry]
use = "company-standards"

# Now the registry's [rules] section is available
# Reference them as @rules.* (v0.0.3+ with cross-field refs)
```

## Rules inheritance

Rules resolution follows this precedence:

```sh
project [rules] > registry [rules] > global [rules] > defaults
```

Higher-priority `[rules]` sections override lower-priority ones.

## Documentation rules

Rules are also useful for documenting team conventions even before cross-field references are fully supported:

```toml
[rules]
# Documentation: commit types we support
supported_types = ["feat", "fix", "chore", "docs", "style", "refactor", "perf", "test"]

# Documentation: acceptable scopes by team
api_team_scopes = ["api", "auth", "logging"]
frontend_team_scopes = ["ui", "routing", "state-management"]

# Documentation: release process
release_process = [
  "1. Ensure all tests pass",
  "2. Update CHANGELOG.md",
  "3. Run 'cw bump' to get new version",
  "4. Run 'cw tag --sign --push'",
  "5. Notify team in Slack"
]

# Documentation: on-call contacts
on_call_schedule = "https://oncall.company.com/api-team"
security_contact = "security@company.com"
```

## Next steps

- **Use with registry:** See [With Registry](./with-registry.md)
- **For field reference:** See [Public API Reference](../public-api.md)
- **For team configs:** See [Recommended Configs](./recommended.md)
