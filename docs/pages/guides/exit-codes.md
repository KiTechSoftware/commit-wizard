# Errors and exit codes

Every failure in `cw` has an **error id** such as `E4003` and ends the process with an **exit code** that depends on the kind of failure. Scripts and CI should branch on the exit code; people can search for the id.

```text
E4003: Git command failed
  command: git ...
  reason: ...
```

`0` always means success. A "nothing to do" outcome, such as `cw branch prune` finding no branches, is a success.

## Exit codes

| Exit | Kind | Meaning |
| --- | --- | --- |
| `0` | | Success |
| `2` | User | Bad flag, missing input, cancelled prompt, or a question that could not be asked |
| `10` | Config | The configuration is invalid, unreadable or points at something that does not exist |
| `11` | State | The repository or project is not in a state the command can work with |
| `20` | Registry | The registry is unknown, invalid, unreachable or offline |
| `21` | Git | Git failed, no repository was found, or a rule about history blocked the command |
| `25` | I/O | Reading or writing a file failed, or a prompt or output could not be rendered |
| `26` | Process | A program that `cw` ran failed |
| `30` | Validation | Commits, pull requests or branches break the project's rules |
| `31` | Release | A release or tag cannot be made, for example the tag already exists |
| `32` | Hook | A hook could not be installed or ran and failed |
| `33` | AI | An AI step is not supported or its provider failed |
| `50` | Runtime | An unexpected internal failure |

Exit codes are stable: new error ids may appear, but an id never moves to a different exit code.

## Error ids

The id is `E` followed by the kind's number and a counter, as in `E` + `4` + `003`.

| Kind | Prefix | Ids |
| --- | --- | --- |
| User | `E0xxx` | `E0001` operation cancelled · `E0002` interactive input required · `E0003` invalid argument · `E0004` format not supported for this command · `E0005` input is invalid |
| Config | `E1xxx` | `E1001` configuration invalid · `E1002` unreadable · `E1003` reference invalid |
| State | `E2xxx` | `E2001` state invalid · `E2002` state unreadable |
| Registry | `E3xxx` | `E3001` unknown · `E3002` invalid · `E3003` unreachable · `E3004` sync failed · `E3005` ref not found · `E3006` section missing · `E3007` unavailable in offline mode |
| Git | `E4xxx` | `E4001` git is missing · `E4002` repository not found · `E4003` git command failed · `E4004` push blocked by policy · `E4005` commits are already published · `E4006` history cannot be rewritten safely |
| Validation | `E5xxx` | `E5001` validation failed · `E5002` commit invalid · `E5003` pull request invalid · `E5004` branch invalid |
| Release | `E6xxx` | `E6001` release invalid · `E6002` tag already exists · `E6003` release branch already exists |
| Hook | `E7xxx` | `E7001` installation failed · `E7002` execution failed |
| AI | `E8xxx` | `E8001` not supported · `E8002` provider failed |
| I/O | `E9xxx` | `E9001` I/O failure · `E9002` serialization failure · `E9003` prompt failed · `E9004` output render failure |
| Process | `E10xxx` | `E10001` process failure |
| Runtime | `E11xxx` | `E11001` runtime failure |

The `[hooks]` and `[ai]` settings are reserved, so the Hook and AI errors are not raised by any command yet.

## Using them in a script

```sh
cw check --ci --from origin/main --to HEAD
case $? in
  0)  echo "all commits pass" ;;
  30) echo "a commit breaks the rules" ;;
  21) echo "git failed" ;;
  *)  echo "cw failed for another reason" ;;
esac
```

Branch on the exit code, not on the message text, which can be reworded. Add `-v` to see more detail, such as the git command that failed.

## Common cases

| You see | Exit | Cause and fix |
| --- | --- | --- |
| `Interactive input required` | `2` | A question had no answer: pass the flag the message names, or run on a terminal. See [global flags](../commands/global-flags.md) |
| `Commit is invalid` | `30` | `cw check` or `cw commit` found a message that breaks the rules; `cw fix` repairs history |
| `Push blocked by policy` | `21` | `push.allow.protected` or `push.allow.force` forbids it; `--force` overrides |
| `Commits are already published` | `21` | `cw fix` refuses to rewrite commits that are on a remote |
| `Release tag already exists` | `31` | Pick another version, or delete the tag first |
| `Git repository not found` | `21` | Run `cw` inside a repository, or pass `--cwd` |
