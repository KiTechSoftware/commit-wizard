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

## Interactive mode

Run `cw changelog` on a terminal with no options and it asks what to generate:

1. **What would you like to generate?** — *Changes since the last tag*, *Full history*, or a *Custom range*.
2. For a range, the **From** and **To** revisions (From is required, To defaults to `HEAD`).
3. For anything but full history, a **version label**; leave it empty to call the section `Unreleased`.
4. Whether to **group entries by scope**, and whether to **include commits that are not Conventional Commits**.

Giving any option on the command line skips the questions, and so does running without a terminal (in a script or CI), where `cw changelog` simply uses the defaults. The output format, file and link settings always come from config or flags.

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

**Dates.** A tagged release is dated by when it was tagged: the tagger date for an annotated tag, or the commit date for a lightweight one. A `--next-tag` section is dated today. The plain `Unreleased` section has no date on purpose, so running the command again does not change the file.

Releases are matched by their heading label, so `## [v1.2.0] - 2026-01-01` and `## v1.2.0` are the same release.

## Links

When the repository has a web address, the changelog links to it:

- each release heading links to the compare view (`## [v1.2.0](…/compare/v1.1.0...v1.2.0) - 2026-01-01`), or to the tag page for the first release;
- each short hash links to its commit: `([abc1234](…/commit/abc1234…))`;
- `Unreleased` compares the newest tag to `HEAD`, and a `--next-tag` section compares to the version you are about to tag.

The address comes from the `origin` remote (`git@github.com:acme/app.git` becomes `https://github.com/acme/app`; GitLab remotes use its `/-/` paths). A remote with no web address, such as a local path, gives no links. Override or switch it off in config:

```toml
[changelog.links]
enabled = true                              # false removes all links
url = "https://code.example.com/team/app"   # instead of the origin remote
```

The JSON output carries the same addresses as `url` fields, and each entry's full commit id as `commit`. Existing releases are matched by label, so turning links on or off rewrites those headings in place rather than adding duplicates.

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

`show_empty_sections = true` lists every configured section (from `section_order` and `[changelog.sections.*]`) in each release, with `_No changes._` under the ones that have no commits. `show_empty_scopes = true` does the same for the scopes in `scope_order` when entries are grouped by scope. Both are off by default.

## Global flags

All [global flags](./global-flags.md) are supported. `--dry-run` shows the result without writing, and global `--force` recreates the file instead of updating it. `--json` wraps the status report in the JSON envelope; use `--output-format json` for the changelog itself.
