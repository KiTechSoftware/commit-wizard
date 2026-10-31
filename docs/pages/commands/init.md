# cw init

Bootstrap Commit Wizard config assets for a project, user, or registry. When run with no subcommand it initialises a hidden project config (`.cwizard.toml`) using the `standard` profile.

## Usage

```sh
cw init [OPTIONS] [SUBCOMMAND]
```

## Guided setup

Run `cw init` on a terminal and it asks how your project works, then writes a config to match. A first run takes under a minute: press Enter to accept the recommended answer to each question.

1. **Which profile fits your project?** — *Standard* (recommended), *Minimal* or *Full*.
2. **Which file format do you prefer?** — TOML, YAML or JSON.
3. **Which commit types will you use?** — all pre-selected; `feat` and `fix` are always kept because versioning and changelogs depend on them. (Skipped for *Minimal*, which lists no types.)
4. **How should commit scopes work?** — *Optional*, *Required* or *Not used*.
5. **Which branches are protected?** — comma-separated, patterns like `release/*` work.
6. **Where should commit messages be checked?** — *Protected branches only* (recommended), *Every branch* or *Nowhere*.
7. **Allow pushing straight to a protected branch?**
8. **Customize more options?** — answer yes for the subject length, emojis before commit types, whether breaking changes need a description, whether a ticket reference is required, the tag prefix, and the changelog file.
9. A last **Create `.cwizard.toml`?**

If the file already exists you are asked whether to replace it; answering no leaves it untouched. Declining the final question writes nothing.

Flags answer their own questions: `cw init --minimal --file-format yaml` skips the profile and format questions. Scripts, CI and `--yes`, `--ci` or `--non-interactive` skip the wizard and write the chosen profile exactly as before, so `cw init --yes` is the quick way to take every default. The wizard also runs for `cw init project`, `cw init config` and `cw init config --global`; `rules` and `registry` are not guided.

## Subcommands

| Subcommand | Description |
| --- | --- |
| `project` | Create a project config (`.cwizard.toml`) in the current directory |
| `config` | Create a config file: the project config, or with `--global` your user-wide one |
| `rules` | Create your user-wide rules file (`--global`). A project keeps its rules in the `[rules]` table of its config, so there is no separate project rules file |
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
| `--global` | `-g` | Write to the global (user-wide) location. `rules` requires it; without it, `config` creates the project config exactly like bare `cw init` |

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
