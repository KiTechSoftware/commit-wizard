# Vision and scope

Commit Wizard exists so that a team's history stays readable and its releases stay predictable, without the tool being in the way.

## Principles

- **Guided, not restrictive.** On a terminal `cw` walks you through a step and explains it. It does not block a person who knows what they want: every question has a flag.
- **Strict where it must be.** In CI there are no questions and no guessing. The same rules fail the build with an exit code.
- **One config, explicit overrides.** Flags beat environment variables, which beat the project config, the registry and the defaults.
- **Safe by default.** Anything outward-facing (pushing, deleting remote branches, rewriting history) asks first, previews with `--dry-run`, and prints how to undo it.
- **Portable.** The same binary behaves the same on Linux, macOS and Windows.

## In scope

Writing and checking Conventional Commits, repairing history, working out the next version, tagging, changelogs, branch naming and cleanup, push safety, and sharing config between projects.

## Out of scope

`cw` is not a git client, a code host, or a replacement for your CI. It does not publish packages. It leaves hosting, review and deployment to the tools built for them.

## Not yet built

`[pr]`, `[release]`, `[hooks]` and `[ai]` are reserved settings. They are accepted so configs written today keep working, and no command reads them yet. Installing git hooks and generating messages with AI are the intended uses.
