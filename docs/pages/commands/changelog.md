# cw changelog

Generate or update a changelog from Conventional Commit history. Commits are grouped into releases by version tag, and into sections by commit type.

## Usage

```sh
cw changelog [OPTIONS]
```

## Examples

**Add the changes since the last tag as an "Unreleased" section:**

```sh
cw changelog
```

**Label those changes with the version you are about to release:**

```sh
cw changelog --next-tag 1.4.0
```

**Rebuild every release from the tag history:**

```sh
cw changelog --all
```

**Preview without writing anything:**

```sh
cw changelog --dry-run
```

**Print to stdout, for piping:**

```sh
cw changelog --output - --output-format json | jq '.releases[0].sections'
```

**One section for an arbitrary range:**

```sh
cw changelog --from v1.2.0 --to v1.3.0
```

## How it works

1. **Releases.** Version tags reachable from `--to` (default `HEAD`) define the releases, newest version first. Each release holds the commits since the previous tag. Tags on other branches are ignored.
2. **Unreleased.** Commits after the newest tag form the pending section, called `Unreleased` (`changelog.layout.unreleased_label`) or the version you pass to `--next-tag`.
3. **Sections.** Commits are grouped under a heading per type. The heading is `[changelog.sections.<type>].title` if set, otherwise the commit type's own `section` (for example `Features`). Sections are ordered by `[changelog.sections.<type>].order`, then by `changelog.layout.section_order`.
4. **Breaking changes.** Commits marked with `!` or a `BREAKING CHANGE:` footer are also listed in a `Breaking Changes` section at the top of their release, with the footer text.
5. **Skipped commits.** Commits that are not Conventional Commits and merge commits are left out. `--include-non-cc` adds the non-conventional ones under the miscellaneous section. A conventional commit with a type your policy does not know is listed under the miscellaneous section instead of being dropped.

Prerelease tags such as `v1.1.0-rc.1` do not get their own section: their commits are part of the stable release that follows. Use `--prereleases` to list them separately.

## Updating an existing file

The command is safe to run repeatedly:

- A release that is already in the file is **replaced in place**, never duplicated.
- Releases you did not regenerate, and any text above the first release (your header or intro), are **kept**.
- When a real version is generated (`--next-tag`, `--all`), a stale `Unreleased` section is removed, since those changes now belong to the version.
- Global `--force` ignores the existing file and writes a fresh one.

Releases are matched by their heading label, so `## [v1.2.0] - 2026-01-01` and `## v1.2.0` are the same release.

## Output formats

**Markdown** (default) is written to `changelog.output`, `CHANGELOG.md` unless configured. Each entry looks like `- **scope:** subject (abc1234)`; use `--group-by-scope` (or `changelog.layout.group_by = ["type", "scope"]`) to group entries under `####` scope headings instead.

**JSON** (`--output-format json`, or `changelog.format = "json"`) writes `{ "releases": [...] }` with typed entries (`type`, `scope`, `subject`, `hash`, `breaking`, `breaking_note`). When no output path is given, `CHANGELOG.md` becomes `CHANGELOG.json` so the Markdown file is not overwritten. JSON is always regenerated from the releases selected in that run.

Use `--output -` to print to stdout. Only the changelog is printed, so the output can be piped or redirected.

## Options

| Flag | Description |
| --- | --- |
| `--all` | Generate every release found in the tag history |
| `--from <REF>` | Start of a custom range (exclusive). Produces a single section labelled `FROM..TO`, or the `--next-tag` value |
| `--to <REF>` | End of the range (default: `HEAD`) |
| `--output <PATH>` | File to write (default: `changelog.output`); `-` prints to stdout |
| `--output-format <markdown\|json>` | File format (default: `changelog.format`) |
| `--group-by-scope` | Group entries by scope under each section |
| `--next-tag <VERSION>` | Label the pending changes with a version instead of `Unreleased` (`1.4.0` and `v1.4.0` are equivalent) |
| `--include-non-cc` | Also list non-conventional commits under the miscellaneous section |
| `--prereleases` | Give prerelease tags their own sections |

`--all` cannot be combined with `--from`.

## Configuration

```toml
[changelog]
output = "CHANGELOG.md"
format = "markdown"            # or "json"

[changelog.header]
use = true
title = "Changelog"
description = "All notable changes to this project"

[changelog.layout]
group_by = ["type"]            # add "scope" to group by scope as well
section_order = ["feat", "fix", "docs", "refactor", "test", "chore"]
scope_order = ["api", "ui"]    # order of scope groups
show_scope = true              # prefix entries with **scope:**
misc_section = "Miscellaneous"
unreleased_label = "Unreleased"
date_format = "2006-01-02"     # Go-style layout or strftime ("%Y-%m-%d")

[changelog.sections.fix]
title = "Bug Fixes"
order = 1                      # lower sorts first
```

`show_empty_sections` and `show_empty_scopes` are accepted but not used yet: sections with no entries are never printed.

## Global flags

All [global flags](./global-flags.md) are supported. `--dry-run` shows the result without writing, and global `--force` recreates the file instead of updating it. `--json` wraps the status report in the JSON envelope; use `--output-format json` for the changelog itself.
