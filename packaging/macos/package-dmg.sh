#!/bin/sh
# Assemble an unsigned OneOffice.app and wrap it in a .dmg.
# Run from anywhere, after `cargo build --release -p oneoffice`.
set -eu
root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
label=${1:-}
if [ -z "$label" ]; then
  echo "usage: packaging/macos/package-dmg.sh v0.1.0" >&2
  exit 1
fi
version=${label#v}
case "$version" in
  *[!0-9A-Za-z._-]*)
    echo "refusing to substitute version: $version" >&2
    exit 1
    ;;
esac
bin="$root/target/release/oneoffice"
if [ ! -f "$bin" ]; then
  echo "missing $bin (build with: cargo build --release -p oneoffice)" >&2
  exit 1
fi
case $(uname -m) in
  arm64) arch=aarch64 ;;
  x86_64) arch=x86_64 ;;
  *) arch=$(uname -m) ;;
esac
dist="$root/dist"
app="$dist/OneOffice.app"
stage="$dist/dmg-root"
rm -rf "$app" "$stage"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources" "$stage"
cp "$bin" "$app/Contents/MacOS/oneoffice"
chmod +x "$app/Contents/MacOS/oneoffice"
cp "$root/packaging/macos/OneOffice.icns" "$app/Contents/Resources/OneOffice.icns"
sed "s/__VERSION__/$version/g" "$root/packaging/macos/Info.plist" > "$app/Contents/Info.plist"
cp -R "$app" "$stage/OneOffice.app"
ln -s /Applications "$stage/Applications"
dmg="$dist/OneOffice-${version}-macos-${arch}.dmg"
rm -f "$dmg"
hdiutil create -volname "OneOffice" -srcfolder "$stage" -ov -format UDZO "$dmg"
rm -rf "$stage"
echo "wrote $dmg"
