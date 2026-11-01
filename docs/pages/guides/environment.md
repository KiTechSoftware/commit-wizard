# Environment variables

Any setting in the table below can be set from the environment, which is handy in CI and for one-off runs. Environment variables sit above the project config and below command-line flags:

```text
CLI flags > environment variables > project config > registry config > defaults
```

```sh
CW_COMMIT_SCOPES_MODE=required cw commit
```

## Value formats

- **Booleans:** `true`, `1`, `yes`, `on` or `false`, `0`, `no`, `off` (any case).
- **Lists:** comma-separated, such as `main,develop`; spaces around items are trimmed.
- **Numbers:** whole numbers.
- **Scope modes:** `disabled`, `optional`, `required`.

A value that cannot be read is ignored with a `[warn] CW: ...` message on stderr, and the lower layers apply. An empty variable counts as unset.

To turn the environment layer off entirely, set `CW_ALLOW_ENV_OVERRIDE=false`.

## Commit and check

| Variable | Config key | Values |
| --- | --- | --- |
| `CW_COMMIT_SUBJECT_MAX_LENGTH` | `commit.subject_max_length` | number |
| `CW_COMMIT_SCOPES_MODE` | `commit.scopes.mode` | scope mode |
| `CW_COMMIT_SCOPES_RESTRICT_TO_DEFINED` | `commit.scopes.restrict_to_defined` | boolean |
| `CW_COMMIT_TICKET_REQUIRED` | `commit.ticket.required` | boolean |
| `CW_COMMIT_TICKET_PATTERN` | `commit.ticket.pattern` | regular expression |
| `CW_COMMIT_TICKET_SOURCE` | `commit.ticket.source` | `branch`, `prompt`, `branch_or_prompt`, `disabled` |
| `CW_CHECK_REQUIRE_CONVENTIONAL` | `check.require_conventional` | boolean |
| `CW_CHECK_COMMITS_ENABLED` | `check.commits.enabled` | boolean |
| `CW_CHECK_COMMITS_ENFORCE_ON` | `check.commits.enforce_on` | `all_branches` (or `all`), `protected_branches` (or `protected`), `none` |

## Branches, push and versioning

| Variable | Config key | Values |
| --- | --- | --- |
| `CW_BRANCH_REMOTE` | `branch.remote` | remote name |
| `CW_BRANCH_PROTECTED` | `branch.protected` | list of branch names or patterns |
| `CW_BRANCH_NAMING_PATTERN` | `branch.naming.pattern` | template such as `{type}/{ticket}-{slug}` |
| `CW_PUSH_ALLOW_PROTECTED` | `push.allow.protected` | boolean |
| `CW_PUSH_ALLOW_FORCE` | `push.allow.force` | boolean |
| `CW_PUSH_CHECK_COMMITS` | `push.check.commits` | boolean |
| `CW_PUSH_CHECK_BRANCH_POLICY` | `push.check.branch_policy` | boolean |
| `CW_VERSIONING_TAG_PREFIX` | `versioning.tag_prefix` | text, such as `v` |

## Registry

| Variable | Meaning |
| --- | --- |
| `CW_REGISTRY_URL` | Git URL (or local path) of the registry to use |
| `CW_REGISTRY_REF` | Branch, tag or commit to use |
| `CW_REGISTRY_SECTION` | Section of the registry to use |

See the [registry guide](registry-system.md) for how these combine with the config and the `--registry` flags.

## Reserved

`CW_PR_TITLE_REQUIRE_CONVENTIONAL`, `CW_PR_TITLE_REQUIRE_TICKET`, `CW_PR_TITLE_SCOPE_MODE`, `CW_PR_BRANCH_SOURCE_PATTERN` and `CW_PR_BRANCH_TARGET_ALLOWED` set the reserved `[pr]` settings. They are read and validated, but no command uses them yet.
