# cw tag

Create a SemVer Git tag derived from Conventional Commit history. Optionally sign and push the tag in one step.

## Usage

```sh
cw tag [OPTIONS]
```

## Guided mode

Run `cw tag` on a terminal and it walks you through the tag, asking only about what you did not already give as a flag:

1. **Version** — "Use the next version, v1.4.0?" (worked out from the commits since the last tag). Answer no to type another one. If the proposed tag already exists because nothing new needs releasing, it goes straight to asking for a version.
2. **Tag prefix** — pre-filled with the project's prefix (usually `v`); press Enter to keep it.
3. **Prerelease label** — for example `rc` or `beta` to get `v1.4.0-rc.1`; leave empty for a normal release. Skipped when the version is already a prerelease.
4. **Sign the tag with GPG?**
5. **Push the tag after creating it?** — and, if yes, **which remote** (default `origin`).
6. **Tag message** — pre-filled with `Release 1.4.0`.
7. A final **"Create tag v1.4.0, and push it to origin?"**. Answering no cancels cleanly and creates nothing.

Each flag answers its question up front, so `cw tag --set-version 2.0.0 --sign` asks only about the prefix, label, push and message. In scripts and CI, and with `--yes`, `--ci` or `--non-interactive`, there are no questions: the flags and the commit history decide everything, as before.

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

All [global flags](global-flags.md) are supported. `--dry-run` prints the tag that would be created without writing to Git.
