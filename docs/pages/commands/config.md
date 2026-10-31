# cw config

View and edit Commit Wizard configuration. Works with both your project config (`.cwizard.toml`) and your global config (`~/.config/cwizard/config.toml`).

## Usage

```sh
cw config [OPTIONS] <SUBCOMMAND>
```

## Subcommands

| Subcommand | Description |
| --- | --- |
| `path` | Show the path of the config file that would be read/written |
| `show` | Print the contents of the config file |
| `list [--effective] [PREFIX]` | List settings; `--effective` shows the resolved configuration and where each value came from |
| `get <KEY>` | Get a specific value by dot-path key |
| `set <KEY> <VALUE>` | Set a config value |
| `unset <KEY>` | Remove a config value |

## Options

| Flag | Short | Description |
| --- | --- | --- |
| `--global` | `-g` | Target the global config file instead of the project config |

## Examples

**Show which config file is active:**

```sh
cw config path
```

**Dump the full active config:**

```sh
cw config show
```

**Read a specific value:**

```sh
cw config get commit.scopes.mode
```

**Set a value in the project config:**

```sh
cw config set commit.scopes.mode required
```

**Set a value in the global config:**

```sh
cw config set --global versioning.tag_prefix v
```

**Remove a value (revert to default/inherited):**

```sh
cw config unset commit.ticket.required
```

## Configuration precedence

Values are resolved in this order (highest wins):

```sh
CLI flags > ENV vars > project config > registry config > global config (opt-in) > defaults
```

Layers are merged setting by setting, so a higher layer only overrides the values it sets.

**The global config** is used when no project config is found. Once a project, registry, env or cli config exists, the global config is ignored, unless the project opts in to using it as its lowest layer:

```toml
# .cwizard.toml
[global]
inherit = true   # global config sits under this project's settings
```

(or `cw config set global.inherit true`).

**Built-in defaults** are not a layer you can see through. They are used only when there is no external config at all — no project, registry, global, env or cli config. As soon as any external config exists, it is the whole configuration, and a setting it leaves unset falls back to that setting's own default when it is read.

`cw config list --effective` shows which layers applied and which were found but not applied.

`cw config show` prints the raw contents of one config file. To see the resolved result, use `cw config list --effective`.

## Seeing the effective configuration

```sh
cw config list --effective            # every resolved setting, with where it came from
cw config list --effective commit     # only keys starting with "commit"
cw config list                        # settings in the project config file
cw config -g list                     # settings in the global config file
```

`--effective` prints two tables:

- **Layers** (highest priority first): each layer that took part, where it was read from (file, registry URL, environment or flags), and layers that exist but were *not applied*, with the reason.
- **Settings**: each resolved key, its value, and the layer that supplied it (`cli`, `env`, `project`, `registry:<name>`, `global`). Settings no layer sets are not listed; they use their own defaults when read. When there is no external config at all, the built-in defaults are listed with the source `default`.

With `--json` the same tables are returned as structured data. `--effective` cannot be combined with `-g`.

## Config file locations

| Scope | Path |
| --- | --- |
| Project (preferred) | `<cwd>/.cwizard.toml` |
| Project (alternate) | `<cwd>/cwizard.toml` |
| Global (Linux/macOS) | `~/.config/cwizard/config.toml` |
| Global (Windows) | `%APPDATA%\cwizard\config.toml` |

Every one of these can also be written as YAML (`.yaml` or `.yml`) or JSON (`.json`) — for example `.cwizard.yaml` or `~/.config/cwizard/config.json`. See [File formats](../guides/public-api.md#file-formats).

`cw config path`, `show`, `get`, `set` and `unset` work on whichever format the file uses, and `set`/`unset` write the file back in that same format. Note that `set` and `unset` rewrite the whole file, so comments are not kept.

Use `cw init config` or `cw init project` to create these files. See [`cw init`](./init.md).

## Global flags

All [global flags](../global-flags.md) are supported. Use `--config <PATH>` to target a specific config file.
