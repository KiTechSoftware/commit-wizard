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

### 4. Rules come from one place

Unlike settings, `[rules]` tables are **not merged**: `cw` uses the first rules table it finds, so a project that defines `[rules]` ignores the registry's rules (see [Rules precedence](#rules-precedence-reusable-values)).

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
~/.cache/cwizard/registries/        (Linux/macOS; $XDG_CACHE_HOME/cwizard/registries if set)
%LOCALAPPDATA%\cwizard\registries\   (Windows)
```

Each registry gets its own folder, named by a hash of its URL, so the names are not readable.

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
rm -rf ~/.cache/cwizard/registries
# The cache rebuilds on the next run
```

### Disabling the cache

Set `cache = false` to clone the registry fresh on every run. Nothing is kept
between runs, so the registry must be reachable each time.

```toml
[registry]
use = "my-org"
cache = false   # default: true
```

When caching is on and a branch registry cannot be synced (for example you are
offline), `cw` warns and keeps using the cached copy.

## Failure behavior

By default a registry that cannot be loaded (unreachable, missing `config.toml`,
unknown section) is reported as a warning and `cw` continues with the rest of
your configuration. `cw doctor` lists each failure as a warning.

To fail hard instead, mark the registry as required:

```toml
[registry]
use = "my-org"
required = true   # default: false
```

With `required = true`, `cw` stops with the registry error before running the
command, including when `use` names a registry that is not configured. Only
the active registry is affected; other registries in `[registries.*]` that
fail to load are always warnings.

### Layering across config files

The `[registry]` table is merged field by field across layers (CLI > env > repo
> global), so a layer only overrides the fields it sets. A global
`required = true` still applies when a repo config only sets `use` and
`section`; the repo can opt out with `required = false`.

```toml
# ~/.config/cwizard/config.toml
[registry]
required = true

# .cwizard.toml  (use/section from here, required from the global file)
[registry]
use = "my-org"
section = "backend"
```

### Running without any config

When there is no project config, no global config and no usable registry (for
example a registry that failed to load and is not `required`), `cw` runs on the
built-in defaults instead of failing.

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

### Replacing part or all of a registry

Local config takes precedence, and layers are merged recursively: tables are
combined key by key, and a higher layer wins only for the values it sets.
Anything the local config does not mention keeps coming from the registry, so a
project can change one setting, one commit type, or one field of a type.

```toml
# registry: [commit.types.fix] description = "A bug fix", bump = "patch"

# .cwizard.toml
[commit]
subject_max_length = 72        # changes one value; types and scopes still come from the registry

[commit.types.fix]
bump = "minor"                 # changes one field; description etc. stay from the registry

[commit.types.docs]            # adds a type the registry does not have
description = "Documentation"

[commit.types.chore]
enabled = false                # drops a registry type
```

Rules of the merge:

- **Tables** (`[commit]`, `[commit.types.fix]`, `[push.allow]`, `[versioning]`, ...)
  are merged field by field.
- **Arrays** (such as `[branch] protected = [...]`) are replaced by the higher
  layer, not concatenated.
- **Dropping a commit type** from a lower layer is done with
  `enabled = false` on that type.
- **Replacing everything** is just defining all of it locally; a registry is
  only a base.

### Rules precedence (reusable values)

```sh
1. Project rules (the [rules] table in .cwizard.toml)
2. Registry rules (in registry/rules.toml)
3. Global rules (in ~/.config/cwizard/rules.toml), only when the registry has no rules
```

The **first** of these that exists is used whole; rules from different places are never combined. A `@rules.<name>` reference that the chosen rules do not define is an error.

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
```

To use a different registry for one run, point at it directly with `--registry <URL>` (and `--registry-ref` / `--registry-section`), or change `use`.

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
