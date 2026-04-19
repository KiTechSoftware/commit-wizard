# Configuration: Recommended Starting Points

This guide shows recommended configurations for different team types and sizes.

## Solo developer

**Goal:** Quick setup, minimal overhead, personal project standards.

```toml
version = 1

[commit.subject_max_length]
subject_max_length = 72

# Use all built-in commit types and defaults
# No scope requirements
```

**Why this works:**

- Zero scope overhead
- Sensible type defaults (`feat`, `fix`, `chore`, etc.)
- No ticket requirements
- Quick interactive commits with `cw commit`

**Get started:**

```sh
cw init --profile minimal
```

---

## Small team (5–20 people)

**Goal:** Consistent commits across the team, standardized scopes, clear changelog.

```toml
version = 1

[commit.subject_max_length]
subject_max_length = 72

[commit.scopes]
mode = "required"  # Every commit needs a scope
restrict_to_defined = true  # Only defined scopes allowed

[commit.scopes.definitions.api]
title = "API"
description = "REST endpoints, services"
types = ["feat", "fix", "perf"]

[commit.scopes.definitions.ui]
title = "User Interface"
description = "Frontend, styling, components"
types = ["feat", "fix"]

[commit.scopes.definitions.db]
title = "Database"
description = "Schema, migrations"
types = ["feat", "fix"]

[commit.scopes.definitions.infra]
title = "Infrastructure"
description = "CI/CD, deployment, config"
types = ["chore", "fix"]

[commit.scopes.definitions.docs]
title = "Documentation"
description = "README, guides, comments"
types = ["docs", "chore"]

[commit.breaking]
require_footer = true  # Document breaking changes

[branch]
protected = ["main", "master", "staging"]

[check.commits]
enforce_on = "protected_branches"  # Check PRs going to main/staging

[push.allow]
protected = false  # Prevent direct push to main
force = false

[versioning]
tag_prefix = "v"

[changelog]
output = "CHANGELOG.md"
format = "markdown"

[changelog.layout]
group_by = ["type"]
section_order = ["feat", "fix", "perf", "docs", "refactor", "test", "chore"]
```

**Why this works:**

- Scopes provide team visibility (who worked on what)
- Changelog is automatically generated and organized
- Protected branches prevent accidental commits to production
- Breaking changes are documented

**Get started:**

```sh
cw init --profile standard
# Then edit .cwizard.toml to add your scopes
```

**Add to CI (GitHub Actions example):**

```yaml
name: Validate commits
on: pull_request
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0
      - uses: taiki-e/install-action@v2
        with:
          tool: commit-wizard@0.0.2
      - run: cw check --ci --from origin/${{ github.base_ref }} --to HEAD
```

---

## Mid-size team (20–100 people)

**Goal:** Centralized standards, ticket integration, section-based registry.

```toml
version = 1

# Use registry for shared team standards
[registry]
use = "team-standards"
section = "backend"  # or "frontend", "platform"

[registries.team-standards]
url = "https://github.com/my-team/cw-registry.git"
ref = "main"
section = "backend"

# Local overrides (if needed)
[commit.breaking]
require_footer = true

[branch]
protected = ["main", "release/*", "staging"]

[push.allow]
protected = false

[check.commits]
enforce_on = "protected_branches"

[versioning]
tag_prefix = "v"

[changelog]
output = "CHANGELOG.md"
format = "markdown"

[changelog.layout]
section_order = ["feat", "fix", "perf", "docs", "refactor", "test", "chore"]
```

**Registry structure:**

```sh
team-registry/
├── shared/
│   ├── config.toml      # Common to all teams
│   └── rules.toml
├── backend/
│   ├── config.toml      # Backend-specific overrides
│   └── rules.toml
└── frontend/
    ├── config.toml
    └── rules.toml
```

**Why this works:**

- Single source of truth for team standards
- Each team (backend, frontend, platform) has customizable overrides
- Version sync across projects
- Easy onboarding: `cw init` pulls registry config automatically

**Set up registry (one-time):**

```sh
mkdir team-registry && cd team-registry
git init

cw init registry . --rules --section shared --section backend --section frontend

# Customize each section's config.toml as needed
git add .
git commit -m "chore: init cw registry"
git push -u origin main
```

---

## Enterprise (100+ people, multi-team)

**Goal:** Maximum governance, ticket integration, release workflows.

```toml
version = 1

# Central registry + local overrides
[registry]
use = "enterprise-standards"
section = "platform"

[registries.enterprise-standards]
url = "https://git-internal.company.com/standards/cw.git"
ref = "main"
sections = ["platform"]

# Local project overrides
[commit.scopes]
mode = "required"
restrict_to_defined = true

[commit.scopes.definitions.api]
title = "API"
types = ["feat", "fix", "perf"]

[commit.scopes.definitions.auth]
title = "Authentication"
types = ["feat", "fix", "security"]

# Ticket integration
[commit.ticket]
required = true
pattern = "PROJ-\\d+"
source = "branch_or_prompt"
header_format = "[{ticket}] "

[commit.breaking]
require_header = true
require_footer = true

# Strict branch protection
[branch]
protected = ["main", "release/*", "hotfix/*"]
remote = "origin"

[push.allow]
protected = false  # No direct push to protected branches
force = false

[push.check]
commits = true  # Validate before push
branch_policy = true

[check.commits]
enforce_on = "all_branches"  # Strict checking everywhere

# Release workflows (v0.0.3+)
[release]
enabled = true
source_branch = "develop"
target_branch = "main"
branch_format = "release/{version}"

[release.validation]
require_clean_worktree = true
fail_if_tag_exists = true

[versioning]
tag_prefix = "v"

[changelog]
output = "CHANGELOG.md"
format = "markdown"

[changelog.layout]
group_by = ["type"]
show_scope = true
section_order = ["feat", "fix", "security", "perf", "refactor", "docs", "test", "chore"]
```

**Enterprise registry structure:**

```sh
enterprise-registry/
├── platform/
│   ├── config.toml          # Platform standards
│   └── rules.toml
├── frontend/
├── backend/
├── data/
└── devops/
```

**Why this works:**

- Tickets provide audit trail and traceability
- Strict validation on all branches prevents regressions
- Release workflows automate version bumps and changelogs
- Central governance via registry

**CI setup example (GitLab):**

```yaml
validate:commits:
  stage: test
  script:
    - cw check --ci --from $CI_MERGE_REQUEST_DIFF_BASE_SHA --to HEAD

release:tag:
  stage: release
  only:
    - main
  script:
    - cw bump --ci
    - cw tag --sign --push
```

---

## Comparison table

| Aspect | Solo | Small | Mid | Enterprise |
| --- | --- | --- | --- | --- |
| Scopes | Optional | Required | Via registry | Required |
| Tickets | No | Optional | No | Required |
| Registry | No | No | Yes | Yes |
| Branch protection | No | Yes | Yes | Yes |
| Release workflows | No | No | Basic | Full |
| Changelog | Auto | Auto | Auto | Auto |
| Hook integration | Maybe | Yes | Yes | Yes |

---

## Next steps

1. **Start with your team size** — Use the config above as a template
2. **Customize scopes** — Add scope definitions that match your codebase
3. **Set up CI** — Run `cw check` in your pull request workflow
4. **Share with team** — Commit `.cwizard.toml` to your repo
5. **Iterate** — Adjust settings as your team's workflow evolves

For more details on each configuration field, see [Public API Reference](../public-api.md).
