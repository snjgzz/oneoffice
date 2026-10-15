#!/bin/sh
# Apply the OneOffice embed patch onto the gridcraft submodule. Safe to run twice.
set -eu
root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
patch="$root/patches/gridcraft-embed.patch"
repo="$root/vendor/gridcraft"
if [ ! -e "$repo/Cargo.toml" ]; then
  echo "vendor/gridcraft is missing. Run: git submodule update --init --recursive" >&2
  exit 1
fi
if [ ! -f "$patch" ]; then
  echo "missing $patch" >&2
  exit 1
fi
if git -C "$repo" apply --reverse --check "$patch" >/dev/null 2>&1; then
  echo "gridcraft embed patch already applied"
  exit 0
fi
git -C "$repo" apply --check "$patch"
git -C "$repo" apply "$patch"
echo "applied patches/gridcraft-embed.patch"
