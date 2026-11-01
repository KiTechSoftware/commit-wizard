#!/usr/bin/env bash
# Install the prebuilt `cw` binary from GitHub Releases (Linux and macOS).
#
#   curl -fsSL https://raw.githubusercontent.com/KiTechSoftware/commit-wizard/main/scripts/install.sh | bash
#   VERSION=0.0.5 ./scripts/install.sh           # pin a version
#   INSTALL_DIR=/usr/local/bin ./scripts/install.sh
set -euo pipefail

REPO="KiTechSoftware/commit-wizard"
VERSION="${VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

case "$(uname -s)" in
  Linux) os=linux ;;
  Darwin) os=macos ;;
  *) echo "unsupported OS: $(uname -s) (use 'cargo install commit-wizard --locked')" >&2; exit 1 ;;
esac
case "$(uname -m)" in
  x86_64 | amd64) arch=x86_64 ;;
  arm64 | aarch64) arch=aarch64 ;;
  *) echo "unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac
platform="$os-$arch"

if [ "$VERSION" = "latest" ]; then
  tag=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n1)
  [ -n "$tag" ] || { echo "could not find the latest release" >&2; exit 1; }
else
  tag="v${VERSION#v}"
fi

name="cw-$tag-$platform"
base="https://github.com/$REPO/releases/download/$tag"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

echo "Downloading $name.tar.gz"
curl -fsSL "$base/$name.tar.gz" -o "$tmp/$name.tar.gz"

if curl -fsSL "$base/SHASUMS256.txt" -o "$tmp/SHASUMS256.txt" 2> /dev/null; then
  expected=$(grep " $name.tar.gz\$" "$tmp/SHASUMS256.txt" | awk '{print $1}')
  if command -v sha256sum > /dev/null; then actual=$(sha256sum "$tmp/$name.tar.gz" | awk '{print $1}')
  else actual=$(shasum -a 256 "$tmp/$name.tar.gz" | awk '{print $1}'); fi
  [ -n "$expected" ] && [ "$expected" = "$actual" ] || { echo "checksum mismatch for $name.tar.gz" >&2; exit 1; }
else
  echo "warning: SHASUMS256.txt not found; skipping checksum verification" >&2
fi

tar -xzf "$tmp/$name.tar.gz" -C "$tmp"
mkdir -p "$INSTALL_DIR"
install -m 755 "$tmp/$name/cw" "$INSTALL_DIR/cw"

echo "Installed cw $tag to $INSTALL_DIR/cw"
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "Add $INSTALL_DIR to your PATH, then run: cw --version" ;;
esac
