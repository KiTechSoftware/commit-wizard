# cw tag

Create a SemVer Git tag derived from Conventional Commit history. Optionally sign and push the tag in one step.

## Usage

```sh
cw tag [OPTIONS]
```

## Examples

**Auto-calculate the next version and tag:**

```sh
cw tag
```

**Tag with an explicit version:**

```sh
cw tag --set-version 2.0.0
```

**Custom prefix and suffix:**

```sh
cw tag --prefix v --suffix -rc1
# → v2.0.0-rc1
```

**No prefix:**

```sh
cw tag --prefix ""
# → 2.0.0
```

**Sign and push in one step:**

```sh
cw tag --sign --push
```

**Tag a specific branch:**

```sh
cw tag --branch release/2.0 --push --remote upstream
```

**Dry-run — show what tag would be created:**

```sh
cw tag --dry-run
```

## Options

| Flag | Description |
| --- | --- |
| `--set-version <SEMVER>` / `--use-version` | Use this exact version instead of calculating from commits (e.g. `1.2.3` or `v1.2.3`) |
| `--prefix <PREFIX>` | Tag prefix (default: `v` from `versioning.tag_prefix` in config) |
| `--suffix <SUFFIX>` | Tag suffix appended after the version (e.g. `-rc1`) |
| `--sign` | Create a GPG-signed annotated tag |
| `--push` | Push the tag to the remote after creation |
| `--remote <NAME>` | Remote to push to (default: `origin`) |
| `--branch <BRANCH>` | Branch to tag (default: `HEAD`) |
| `--message <MSG>` | Annotated tag message/body |

## Version calculation

`cw tag` uses the same version calculation logic as [`cw bump`](./bump.md). The starting version is derived from the most recent SemVer-compatible tag on the current branch.

| Commit trigger | Bump |
| --- | --- |
| Breaking change (`!` or `BREAKING CHANGE` footer) | **major** |
| `feat` | **minor** |
| `fix`, `perf`, and patch-level types | **patch** |

## Configuration

The default tag prefix is controlled by `versioning.tag_prefix` in your config:

```toml
[versioning]
tag_prefix = "v"
```

Or via the environment variable `CW_VERSIONING_TAG_PREFIX`.

## Global flags

All [global flags](../global-flags.md) are supported. `--dry-run` prints the tag that would be created without writing to Git.
