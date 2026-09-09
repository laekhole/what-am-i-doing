#!/bin/bash
# Unsigned development bundle. Does not publish, notarize or change Gatekeeper.
set -euo pipefail
cd "$(dirname "$0")/.."
arch="$(uname -m)"
case "$arch" in arm64|x86_64) ;; *) echo "Unsupported Mac architecture: $arch" >&2; exit 1;; esac
output="${1:-desktop/target/macos-bundle}"
if [[ -e "$output" ]]; then echo "Output already exists: $output" >&2; exit 1; fi
export MACOSX_DEPLOYMENT_TARGET=13.0
cargo build --manifest-path desktop/Cargo.toml --release --locked
app="$output/waid.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp desktop/target/release/waid-desktop "$app/Contents/MacOS/waid"
cp desktop/assets/fonts/LICENSE.txt "$app/Contents/Resources/FONT-LICENSE.txt"
cat > "$app/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>waid</string>
<key>CFBundleIdentifier</key><string>io.github.laekhole.waid</string>
<key>CFBundleName</key><string>waid</string>
<key>CFBundleDisplayName</key><string>waid</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.3.0</string>
<key>CFBundleVersion</key><string>0.3.0</string>
<key>LSMinimumSystemVersion</key><string>13.0</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
plutil -lint "$app/Contents/Info.plist"
codesign --force --sign - "$app"
codesign --verify --strict "$app"
ditto -c -k --sequesterRsrc --keepParent "$app" "$output/waid-macos-$arch.zip"
(cd "$output" && shasum -a 256 "waid-macos-$arch.zip" > SHA256SUMS.txt)
echo "Development bundle: $app (ad-hoc signed; not notarized)"
