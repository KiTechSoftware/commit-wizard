# Installation

## Prebuilt binary (recommended for CI)

Each release publishes archives for these platforms, with a `SHASUMS256.txt` of their checksums:

| Platform | Archive |
| --- | --- |
| Linux x86_64 | `cw-v<version>-linux-x86_64.tar.gz` |
| macOS Intel | `cw-v<version>-macos-x86_64.tar.gz` |
| macOS Apple Silicon | `cw-v<version>-macos-aarch64.tar.gz` |
| Windows x86_64 | `cw-v<version>-windows-x86_64.zip` |

The installer picks the right one, verifies its checksum and installs to your user directory without needing administrator rights.

**Linux and macOS** (installs to `~/.local/bin`):

```sh
curl -fsSL https://raw.githubusercontent.com/KiTechSoftware/commit-wizard/main/scripts/install.sh | bash
VERSION=0.0.4 INSTALL_DIR=/usr/local/bin bash scripts/install.sh   # pin a version, choose the folder
```

**Windows** (installs to `%USERPROFILE%\AppData\Local\Programs\cw`):

```powershell
iwr -useb https://raw.githubusercontent.com/KiTechSoftware/commit-wizard/main/scripts/install.ps1 | iex
.\scripts\install.ps1 -Version 0.0.4
```

Both scripts install the latest release unless you pin one, warn (and continue) if the checksum file is missing, and tell you if the folder is not on your `PATH`. You can also download an archive from the [Releases](https://github.com/KiTechSoftware/commit-wizard/releases) page and put `cw` anywhere on your `PATH`.

## From crates.io

```sh
cargo install commit-wizard --locked
```

This needs Rust **1.89 or newer**. It works on every platform Rust supports, including those without a prebuilt binary (Linux on ARM, Windows on ARM).

## Requirements

`git` must be on your `PATH`. Some commands, such as `push` and the branch cleanup, run it directly.

## Check it worked

```sh
cw --version
cw doctor
```

## Updating

Run the installer again, or `cargo install commit-wizard --locked --force`.

## For maintainers

Pushing a `v*` tag runs two workflows: *Publish crate* releases to crates.io, and *Release binaries* builds the archives, writes `SHASUMS256.txt` and creates the GitHub Release (or adds the files to one that exists). Run *Release binaries* by hand with an existing tag to rebuild its archives.
