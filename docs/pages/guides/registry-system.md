# Registry System

The registry system allows you to share Commit Wizard configuration across multiple projects and teams via Git repositories.

## Concepts

### What is a registry?

A registry is a **Git-hosted repository** containing:

- `config.toml` — Shared configuration (or per-section: `<section>/config.toml`)
- `rules.toml` — Shared rules (or per-section: `<section>/rules.toml`)

Projects reference a registry and inherit its configuration.

### Why use a registry?

**Without a registry:**

- Every project has its own `.cwizard.toml`
- Commit standards drift across projects
- Updating a rule in 20 projects takes 20 commits

**With a registry:**

- One source of truth for team standards
- All projects update automatically when the registry changes
- Consistency across the organization

### Local vs. Git registries

| Type | Example | Use case |
| --- | --- | --- |
| **Git** | `https://github.com/my-org/standards.git` | Shared org standards |
| **Local path** | `/opt/cw-registry` or `./registry` | Testing, private standards |

## How registries work

### 1. Project declares a registry

```toml
# .cwizard.toml
[registries.company]
url = "https://github.com/my-org/cw-standards.git"
ref = "main"
section = "backend"

[registry]
use = "company"
```

### 2. `cw` fetches the registry

When you run `cw commit`, `cw check`, etc.:

- For Git URLs: Clone/fetch into local cache: `~/.cache/cwizard/registries/`
- For local paths: Read directly

### 3. Configuration is merged

Settings are resolved in this order (highest priority first):

```sh
Project .cwizard.toml > Registry config > Defaults
```

Example:

```sh
Registry [commit.scopes]:
  mode = "optional"

Project .cwizard.toml:
  [commit.scopes]
  mode = "required"

Result: mode = "required" (project wins)
```

### 4. Rules are merged

Similarly for `[rules]`:

```sh
Project rules > Registry rules > Defaults
```

## Registry structure

### Single-source registry

Simple structure: config and rules at the root.

```sh
my-registry/
├── config.toml
└── rules.toml
```

Use:

```toml
[registries.company]
url = "https://github.com/my-org/cw-standards.git"
# No section specified; uses root

[registry]
use = "company"
```

### Sectioned registry

Organize by team or domain.

```sh
my-registry/
├── config.toml      # Base settings
├── rules.toml
├── backend/
│   ├── config.toml      # Backend overrides
│   └── rules.toml
├── frontend/
│   ├── config.toml
│   └── rules.toml
└── platform/
    ├── config.toml
    └── rules.toml
```

Use:

```toml
[registries.company]
url = "https://github.com/my-org/cw-standards.git"

[registry]
use = "company"
section = "backend"  # Load backend/config.toml and backend/rules.toml
```

## Caching

Git registries are cached locally to avoid repeated clones.

### Cache location

```sh
~/.cache/cwizard/registries/   (Linux/macOS)
%APPDATA%\cwizard\cache\       (Windows)
```

### Cache invalidation

**Version tags** (e.g., `v1.0.0`):

- Treated as immutable
- Cached indefinitely
- No network calls after first fetch

**Branches** (e.g., `main`, `develop`):

- Checked for updates on each `cw` invocation
- Automatically synced if remote changes

**Force refresh:**

```bash
rm -rf ~/.cache/cwizard/registries/<registry-id>
# Cache rebuilds on next run
```

## Resolution precedence

Behavior and rules follow separate, well-defined precedence chains:

### Behavior precedence (what settings actually apply)

```sh
1. CLI arguments (e.g., cw commit --scope api)
2. Environment variables (e.g., CW_COMMIT_SCOPES_MODE=required)
3. Project config (.cwizard.toml)
4. Registry config (selected via [registry])
5. Built-in defaults
```

**Example:**

```sh
# These override anything:
cw commit --scope api

# ENV overrides project config:
export CW_COMMIT_SCOPES_MODE=required

# Project config overrides registry:
# .cwizard.toml: [commit.scopes] mode = "required"

# Registry config overrides defaults:
# registry/config.toml: [commit.scopes] mode = "optional"

# If nothing is set, defaults apply
# Result: required (from project config)
```

### Rules precedence (reusable values)

```sh
1. Project rules (in project/.cwizard.toml)
2. Registry rules (in registry/rules.toml)
3. Global rules (in ~/.config/cwizard/rules.toml)
4. Built-in rules
```

## Working with multiple registries

You can define multiple registries and switch between them:

```toml
[registries.org-standards]
url = "https://github.com/my-org/standards.git"
ref = "main"

[registries.team-overrides]
url = "https://github.com/my-team/overrides.git"
ref = "v1.0.0"

[registries.local-test]
url = "/opt/test-registry"

# Use one
[registry]
use = "org-standards"

# Switch at runtime:
# cw commit --registry-section team-overrides
```

## Versioning registries

Use Git tags to version your registry for stability:

```bash
cd my-registry
git tag -a v1.0.0 -m "Freeze for production"
git push origin v1.0.0
```

Pin projects to a version:

```toml
[registries.company]
url = "https://github.com/my-org/cw-standards.git"
ref = "v1.0.0"  # Pin to version, not main
section = "backend"

[registry]
use = "company"
```

Benefits:

- Prevents unexpected changes from registry updates
- Easy rollback if a registry change breaks things
- Clear audit trail

## Registry discovery

If a project doesn't declare a registry, `cw` looks for:

1. **CLI:** `--registry <URL>` or `--registry-ref <REF>`
2. **Environment:** `CW_REGISTRY_URL`, `CW_REGISTRY_REF`
3. **Config:** `[registry]` section in `.cwizard.toml`

## Common patterns

### Pattern 1: Org-wide standards + team overrides

```sh
my-org-standards/
├── shared/
│   ├── config.toml        # Commit types, basic settings
│   └── rules.toml
└── backend/
    ├── config.toml        # Backend-specific scopes
    └── rules.toml
```

Every backend project:

```toml
[registries.org]
url = "..."
section = "backend"

[registry]
use = "org"
```

### Pattern 2: Versioned standards

Release registry versions quarterly:

```bash
git tag -a v2025-Q1 -m "Standards for Q1 2025"
git push origin v2025-Q1
```

Pin projects:

```toml
[registries.company]
url = "..."
ref = "v2025-Q1"

[registry]
use = "company"
```

### Pattern 3: Local registry for CI/CD

For CI systems without external network access, use a local path:

```toml
[registries.ci-local]
url = "/mnt/shared/cw-registry"

[registry]
use = "ci-local"
```

## Troubleshooting

### Registry not found

```sh
Error: Registry URL not accessible
```

Check:

- Network connectivity
- Git URL correctness
- SSH keys (for private repos)

### Config not applying

Add verbosity to see what's being loaded:

```bash
cw commit -vv
# Shows which registry section is loaded and what config is merged
```

### Cache issues

Clear the cache:

```bash
rm -rf ~/.cache/cwizard/registries
cw doctor  # Re-init cache
```

## Best practices

1. **Use sections** — Organize by team (backend, frontend, platform)
2. **Document standards** — Add README.md to each section
3. **Version releases** — Use Git tags for stability
4. **Override strategically** — Only override in projects when necessary
5. **Keep rules simple** — Use rules for documentation and future cross-field references

## Next steps

- **Set up a registry:** See [Configuration: With Registry](./configs/with-registry.md)
- **Define shared rules:** See [Configuration: With Rules](./configs/with-rules.md)
- **For team recommendations:** See [Recommended Configs](./configs/recommended.md)
- **For field reference:** See [Public API Reference](./public-api.md)
