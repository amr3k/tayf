#!/usr/bin/env bash
set -euo pipefail

# Builds an AppImage for Tayf and produces a SHA256 checksum file.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="tayf"
APP_ID="Tayf"
VERSION="$(grep -m1 '^version' "$ROOT_DIR/Cargo.toml" | cut -d'"' -f2)"
ARCH="$(uname -m)"

BUILD_DIR="$ROOT_DIR/target/appimage"
APP_DIR="$BUILD_DIR/AppDir"
TOOL_DIR="$BUILD_DIR/tools"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR" "$TOOL_DIR"

echo "==> Building release binary ($ARCH)"
cargo build --release

echo "==> Assembling AppDir"
install -Dm755 "$ROOT_DIR/target/release/$APP_NAME" \
  "$APP_DIR/usr/bin/$APP_NAME"

install -Dm644 "$ROOT_DIR/resources/linux/$APP_NAME.desktop" \
  "$APP_DIR/usr/share/applications/$APP_NAME.desktop"
# linuxdeploy creates the root-level desktop symlink from this file

for size in 32x32 64x64 128x128@2x; do
  target_size="${size%@*}"
  install -Dm644 "$ROOT_DIR/resources/icons/${size}.png" \
    "$APP_DIR/usr/share/icons/hicolor/${target_size}/apps/${APP_NAME}.png"
done
install -Dm644 "$ROOT_DIR/resources/icons/icon.png" \
  "$APP_DIR/${APP_NAME}.png"

# AppStream metadata (optional but recommended for stores)
mkdir -p "$APP_DIR/usr/share/metainfo"
cat > "$APP_DIR/usr/share/metainfo/${APP_ID}.appdata.xml" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>me.a3k.tayf</id>
  <name>Tayf</name>
  <summary>A native Lottie animation viewer</summary>
  <metadata_license>CC0-1.0</metadata_license>
  <project_license>GPL-3.0-or-later</project_license>
  <launchable type="desktop-id">tayf.desktop</launchable>
  <releases>
    <release version="${VERSION}" date="$(date +%Y-%m-%d)"/>
  </releases>
  <content_rating type="oars-1.1"/>
</component>
EOF

echo "==> Fetching packaging tools"
# Run AppImage tools in extraction mode so no FUSE is required.
export APPIMAGE_EXTRACT_AND_RUN=1
fetch_tool() {
  local name="$1" url="$2"
  local path="$TOOL_DIR/$name"
  if [[ ! -x "$path" ]]; then
    echo "    downloading $name" >&2
    curl -fsSL -o "$path" "$url"
    chmod +x "$path"
  fi
  echo "$path"
}

case "$ARCH" in
  x86_64) TOOL_ARCH="x86_64" ;;
  aarch64|arm64) TOOL_ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

LINUXDEPLOY="$(fetch_tool "linuxdeploy-$TOOL_ARCH.AppImage" \
  "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-$TOOL_ARCH.AppImage")"
APPIMAGETOOL="$(fetch_tool "appimagetool-$TOOL_ARCH.AppImage" \
  "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-$TOOL_ARCH.AppImage")"

echo "==> Running linuxdeploy (bundling shared libraries)"
"$LINUXDEPLOY" \
  --appdir="$APP_DIR" \
  --executable="$APP_DIR/usr/bin/$APP_NAME" \
  --desktop-file="$APP_DIR/usr/share/applications/$APP_NAME.desktop" \
  --icon-file="$ROOT_DIR/resources/icons/icon.png"

OUT_NAME="${APP_NAME}-${VERSION}-${ARCH}.AppImage"
OUT_PATH="$BUILD_DIR/$OUT_NAME"

echo "==> Building AppImage: $OUT_NAME"
"$APPIMAGETOOL" "$APP_DIR" "$OUT_PATH"

echo "==> Checksums"
sha256sum "$OUT_PATH" > "$OUT_PATH.sha256"

echo "==> Done: $OUT_PATH"
