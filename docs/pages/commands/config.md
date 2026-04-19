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
| `show` | Print the active (merged) configuration |
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
CLI flags > ENV vars > project config > registry config > global config > defaults
```

`cw config show` displays the fully merged result after all layers are applied.

## Config file locations

| Scope | Path |
| --- | --- |
| Project (preferred) | `<cwd>/.cwizard.toml` |
| Project (alternate) | `<cwd>/cwizard.toml` |
| Global (Linux/macOS) | `~/.config/cwizard/config.toml` |
| Global (Windows) | `%APPDATA%\cwizard\config.toml` |

Use `cw init config` or `cw init project` to create these files. See [`cw init`](./init.md).

## Global flags

All [global flags](../global-flags.md) are supported. Use `--config <PATH>` to target a specific config file.
