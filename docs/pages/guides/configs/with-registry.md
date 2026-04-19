# Configuration: With Registry

Registries allow you to centralize Commit Wizard configuration across multiple projects and teams.

## What is a registry?

A registry is a **Git repository** containing shared configuration that multiple projects can reference:

```sh
my-org-standards/
├── config.toml        # Shared config for all teams
├── rules.toml         # Shared rules
├── backend/
│   ├── config.toml        # Backend-specific config
│   └── rules.toml         # Backend-specific rules
├── frontend/
│   ├── config.toml        # Frontend-specific config
│   └── rules.toml         # Frontend-specific rules
└── platform/
    ├── config.toml        # Platform-specific config
    └── rules.toml         # Platform-specific rules
```

## Benefits

- **Single source of truth** — Update once, all projects use the new config
- **Consistency** — All teams follow the same commit standards
- **Faster onboarding** — New projects inherit standards automatically
- **Flexibility** — Each section (team) can customize overrides

## Setting up a registry

### 1. Create a new repository

```bash
mkdir my-org-standards
cd my-org-standards
git init
git remote add origin https://github.com/my-org/my-org-standards.git
```

### 2. Bootstrap with `cw init registry`

```bash
cw init registry . --rules --section shared --section backend --section frontend
```

This creates:

```sh
config.toml          # Root-level config
rules.toml           # Root-level rules
shared/config.toml
shared/rules.toml
backend/config.toml
backend/rules.toml
frontend/config.toml
frontend/rules.toml
```

### 3. Customize each section

Edit `shared/config.toml` with settings every project should use:

```toml
version = 1

[commit]
subject_max_length = 72

[commit.breaking]
require_footer = true

[branch]
protected = ["main", "staging"]

[versioning]
tag_prefix = "v"

[changelog]
output = "CHANGELOG.md"
format = "markdown"
```

Edit `backend/config.toml` with backend-specific overrides:

```toml
version = 1

# Inherit shared config, then add backend-specific scopes
[commit.scopes]
mode = "required"
restrict_to_defined = true

[commit.scopes.definitions.api]
title = "API"
types = ["feat", "fix", "perf"]

[commit.scopes.definitions.auth]
title = "Authentication"
types = ["feat", "fix", "security"]

[commit.scopes.definitions.db]
title = "Database"
types = ["feat", "fix"]
```

Edit `frontend/config.toml` for frontend:

```toml
version = 1

[commit.scopes]
mode = "required"
restrict_to_defined = true

[commit.scopes.definitions.ui]
title = "User Interface"
types = ["feat", "fix"]

[commit.scopes.definitions.state]
title = "State Management"
types = ["feat", "fix", "refactor"]

[commit.scopes.definitions.routing]
title = "Routing"
types = ["feat", "fix"]
```

### 4. Define shared rules

Edit `shared/rules.toml`:

```toml
# Shared rules all teams can reference
ticket_pattern = "([A-Z]+-\\d+|#\\d+)"
protected_branches = ["main", "staging", "release/*", "hotfix/*"]
```

### 5. Commit and push

```bash
git add -A
git commit -m "chore: init cw registry"
git push -u origin main
```

## Using a registry in a project

### 1. Add registry definition to project `.cwizard.toml`

```toml
version = 1

[registries.my-org]
url = "https://github.com/my-org/my-org-standards.git"
ref = "main"
section = "backend"

[registry]
use = "my-org"
```

### 2. Project config inherits registry settings

When you run `cw commit`, `cw check`, etc., your project uses:

1. Registry config from `backend/config.toml`
2. Project overrides from `.cwizard.toml`

**Resolution precedence:**

```sh
Project .cwizard.toml > Registry config > Defaults
```

### 3. Override registry settings locally

```toml
version = 1

[registries.my-org]
url = "https://github.com/my-org/my-org-standards.git"
ref = "main"
section = "backend"

[registry]
use = "my-org"

# Local override: stricter than registry
[commit.ticket]
required = true
```

## Registry caching

Registries are cached locally to avoid repeated Git clones:

```sh
~/.cache/cwizard/registries/  (Linux/macOS)
%APPDATA%\cwizard\cache\      (Windows)
```

**Cache behavior:**

- Version tags (e.g., `v1.0.0`) — immutable, cached forever
- Branches (e.g., `main`, `develop`) — checked for updates on each `cw` invocation

**Clear cache:**

```bash
rm -rf ~/.cache/cwizard/registries
# Cache will rebuild on next run
```

## CLI override

Override registry at invocation time:

```bash
# Use a different section
cw commit --registry-section frontend

# Use a different registry
cw commit --registry https://github.com/other-org/standards.git

# Use a different ref
cw commit --registry-ref develop
```

## Local registry (for testing)

Registries can be local paths:

```toml
[registries.local-test]
url = "/opt/cw-registry"
section = "test"

[registry]
use = "local-test"
```

## Multi-project example

**Team structure:**

- API team: 5 projects
- Frontend team: 8 projects
- Platform team: 3 projects

**Registry structure:**

```sh
my-org-standards/
├── config.toml            # Default types, subject length, etc.
├── rules.toml             # Team email, Slack channel, etc.
├── api/
│   ├── config.toml        # API scopes: api, auth, db, cache
│   └── rules.toml         # API-specific rules
├── frontend/
│   ├── config.toml        # Frontend scopes: ui, routing, state
│   └── rules.toml         # Frontend-specific rules
└── platform/
    ├── config.toml        # Platform scopes: ci, infra, config
    └── rules.toml         # Platform-specific rules
```

**In `my-api-project/.cwizard.toml`:**

```toml
[registries.company]
url = "https://github.com/my-org/my-org-standards.git"
ref = "main"
section = "api"

[registry]
use = "company"

# Optional local override
[commit.ticket]
required = true
pattern = "PROJ-\\d+"
```

**In `my-frontend-project/.cwizard.toml`:**

```toml
[registries.company]
url = "https://github.com/my-org/my-org-standards.git"
ref = "main"
section = "frontend"

[registry]
use = "company"
```

## Versioning the registry

Use Git tags to version your registry:

```bash
git tag -a v1.0.0 -m "Commit Wizard standards v1.0.0"
git push origin v1.0.0
```

Then pin projects to a specific version:

```toml
[registries.company]
url = "https://github.com/my-org/my-org-standards.git"
ref = "v1.0.0"  # Pin to version, not main
section = "api"
```

## Registry as a shared library

Treat your registry like shared documentation and tooling:

```sh
my-org-standards/
├── README.md              # How to use this registry
├── CONTRIBUTING.md        # How to update standards
├── config.toml
├── rules.toml
├── api/
│   ├── config.toml
│   ├── rules.toml
│   └── README.md          # API team guidelines
└── frontend/
    ├── config.toml
    ├── rules.toml
    └── README.md          # Frontend team guidelines
```

## Migration: From ad-hoc to registry

### Before (every project has its own config)

```sh
my-api-project/.cwizard.toml
my-frontend-project/.cwizard.toml
my-cli-project/.cwizard.toml
# All duplicated, hard to sync
```

### After (single registry, projects reference it)

```sh
# Single registry
my-org-standards/
  ├── api/config.toml
  ├── frontend/config.toml
  └── cli/config.toml

# Projects reference registry
my-api-project/.cwizard.toml          (tiny, just registry ref)
my-frontend-project/.cwizard.toml    (tiny, just registry ref)
my-cli-project/.cwizard.toml         (tiny, just registry ref)
```

## Next steps

- **Define shared rules:** See [With Rules](./with-rules.md)
- **For field reference:** See [Public API Reference](../public-api.md)
- **For team configs:** See [Recommended Configs](./recommended.md)
