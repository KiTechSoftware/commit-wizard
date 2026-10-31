# cw init

Bootstrap Commit Wizard config assets for a project, user, or registry. When run with no subcommand it initialises a hidden project config (`.cwizard.toml`) using the `standard` profile.

## Usage

```sh
cw init [OPTIONS] [SUBCOMMAND]
```

## Subcommands

| Subcommand | Description |
| --- | --- |
| `project` | Create a project config (`.cwizard.toml`) in the current directory |
| `config` | Create a config file (project or global) |
| `rules` | Create a rules file (project or global) |
| `registry <PATH>` | Scaffold a new registry at the given path |

## Examples

**Quickstart** — create `.cwizard.toml` with the standard profile:

```sh
cw init
```

**Minimal config** (fewest required fields):

```sh
cw init --minimal
```

**Full config** (every available field with defaults):

```sh
cw init --full
```

**Use a named profile:**

```sh
cw init --profile standard
```

**Set values inline during init:**

```sh
cw init --set commit.scopes.mode=required --set versioning.tag_prefix=v
```

**Init a global config:**

```sh
cw init config --global
```

**Init a global rules file:**

```sh
cw init rules --global
```

**Scaffold a registry with rules support and two sections:**

```sh
cw init registry ./path/to/registry --rules --section frontend --section backend
```

**Init a visible project config** (`cwizard.toml` instead of `.cwizard.toml`):

```sh
cw init project
```

## Options

| Flag | Conflicts with | Description |
| --- | --- | --- |
| `--minimal` | `--full`, `--profile` | Generate a minimal config with only required fields |
| `--full` | `--minimal`, `--profile` | Generate a full config with all fields |
| `--profile <PROFILE>` | `--minimal`, `--full` | Profile: `minimal`, `standard` (default), `full` |
| `--set <KEY=VALUE>` | | Inline key-value overrides; may be repeated |

### Choosing the file format

Config files can be TOML, YAML or JSON. `cw init` writes TOML unless you ask otherwise:

```sh
cw init --file-format yaml                 # .cwizard.yaml
cw init --file-format json                 # .cwizard.json
cw init config --global --file-format yaml # ~/.config/cwizard/config.yaml
cw init --config team.yaml                 # the extension picks the format
```

`--file-format` applies to the project and global files `init` creates; registries are always written as TOML. When `--config <PATH>` names a file, its extension decides the format instead. Generated YAML and JSON files carry the same settings as the TOML profile but no comments.

### Subcommand-specific flags

**`project`:**

| Flag | Description |
| --- | --- |
| `--hidden` | Write to `.cwizard.toml` (default for `cw init` with no subcommand) |

**`config`, `rules`:**

| Flag | Short | Description |
| --- | --- | --- |
| `--global` | `-g` | Write to the global config/rules location |

**`registry <PATH>`:**

| Flag | Description |
| --- | --- |
| `--rules` | Include a `rules.toml` file in the registry |
| `--section <NAME>` | Create a named subdirectory section; may be repeated |

## Profiles

| Profile | Description |
| --- | --- |
| `minimal` | Only the fields you're most likely to customise |
| `standard` | Sensible defaults for most teams (default) |
| `full` | Every supported field, useful as a reference |

## Global flags

All [global flags](../global-flags.md) are supported. Use `--dry-run` to preview what would be written without creating any files.
