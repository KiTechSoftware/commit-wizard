# cw release

Open and finish a release branch using the `[release]` settings. Requires `[release] enabled = true`.

## Usage

```sh
cw release start [--set-version SEMVER]
cw release finish [--set-version SEMVER] [--remote origin]
```

- `start` creates the release branch (`branch_format`, default `release/{version}`) from `source_branch` and switches to it. The version defaults to the next one calculated from the commits. It refuses a dirty working tree, an existing tag or an existing branch, as `[release.validation]` says.
- `finish` merges the release branch into `target_branch`, tags it (`finish.tag`), merges it into `backmerge_branch` when that differs, and pushes (`finish.push`). The version comes from the current release branch name, or `--set-version`. On a terminal it asks before changing anything.

`--dry-run` shows what would happen. The release branch is left in place after `finish`.
