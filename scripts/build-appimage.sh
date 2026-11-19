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

# hicolor icons (source file -> destination size)
install_icon() {
  local size="$1" src="$2"
  install -Dm644 "$ROOT_DIR/resources/icons/$src" \
    "$APP_DIR/usr/share/icons/hicolor/$size/apps/${APP_NAME}.png"
}
install_icon 32x32 32x32.png
install_icon 64x64 64x64.png
install_icon 256x256 128x128@2x.png
install_icon 512x512 512x512.png
# Root icon referenced by the desktop entry (Icon=tayf); linuxdeploy turns it into .DirIcon
install -Dm644 "$ROOT_DIR/resources/icons/512x512.png" \
  "$APP_DIR/${APP_NAME}.png"

# AppStream metadata (optional but recommended for stores)
mkdir -p "$APP_DIR/usr/share/metainfo"
cat > "$APP_DIR/usr/share/metainfo/me.a3k.tayf.appdata.xml" <<EOF
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
# linuxdeploy names hicolor entries after the source file, so hand it a copy
# already named tayf.png to avoid a stray "512x512.png" icon in the AppImage
cp "$ROOT_DIR/resources/icons/512x512.png" "$BUILD_DIR/$APP_NAME.png"
"$LINUXDEPLOY" \
  --appdir="$APP_DIR" \
  --executable="$APP_DIR/usr/bin/$APP_NAME" \
  --desktop-file="$APP_DIR/usr/share/applications/$APP_NAME.desktop" \
  --icon-file="$BUILD_DIR/$APP_NAME.png"

echo "==> Bundling lavapipe (software Vulkan) fallback"
# GPUI's Linux backend renders through blade_graphics, which requires Vulkan
# and the VK_KHR_surface instance extension. Headless/GPU-less systems (VMs,
# containers, the AppImage catalog test runner) ship a Vulkan loader but no
# ICD, so GPUI aborts before creating a window. Bundle Mesa's lavapipe as a
# fallback ICD; the launcher below only selects it when the host has no ICD,
# so systems with a real GPU keep using their hardware driver.
if ! dpkg-query -W -f='${Status}' mesa-vulkan-drivers 2>/dev/null \
    | grep -q "install ok installed"; then
  echo "mesa-vulkan-drivers is required for the software Vulkan fallback" >&2
  exit 1
fi

LVP_LIB="$(dpkg -L mesa-vulkan-drivers | grep -E '/libvulkan_lvp\.so$' | head -n1)"
LVP_ICD="$(dpkg -L mesa-vulkan-drivers | grep -E '/lvp_icd\..*\.json$' | head -n1)"
if [[ -z "$LVP_LIB" || -z "$LVP_ICD" ]]; then
  echo "Could not locate lavapipe files in mesa-vulkan-drivers" >&2
  exit 1
fi

ICD_DIR="$APP_DIR/usr/share/vulkan/icd.d"
install -Dm644 "$LVP_LIB" "$ICD_DIR/libvulkan_lvp.so"
# The distro manifest uses an absolute library_path under /usr/lib, which does
# not exist once the AppImage is mounted. Rewrite it to a relative path so the
# loader resolves the bundled driver next to the manifest.
sed -E 's#"library_path"[[:space:]]*:[[:space:]]*"[^"]*"#"library_path": "libvulkan_lvp.so"#' \
  "$LVP_ICD" > "$ICD_DIR/$(basename "$LVP_ICD")"

echo "==> Installing AppRun launcher"
# linuxdeploy leaves AppRun as a symlink to the binary. Replace it with a
# launcher that exposes the bundled libs and selects lavapipe only as a
# fallback when the host has no Vulkan ICD of its own.
rm -f "$APP_DIR/AppRun"
cat > "$APP_DIR/AppRun" <<'APPRUN_EOF'
#!/bin/sh
# Launcher for the Tayf AppImage.
# GPUI requires Vulkan; without any ICD it panics before showing a window.
# Prefer a host driver and fall back to the bundled Mesa lavapipe driver.
HERE="$(CDPATH= cd -- "$(dirname -- "$(readlink -f -- "$0")")" && pwd)"
export LD_LIBRARY_PATH="$HERE/usr/lib:$HERE/usr/share/vulkan/icd.d${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

if [ -z "${VK_ICD_FILENAMES:-}" ] && [ -z "${VK_DRIVER_FILES:-}" ]; then
    host_icd=""
    for dir in \
        /usr/share/vulkan/icd.d \
        /etc/vulkan/icd.d \
        "${XDG_CONFIG_HOME:-$HOME/.config}/vulkan/icd.d" \
        "${XDG_DATA_HOME:-$HOME/.local/share}/vulkan/icd.d"
    do
        if [ -d "$dir" ]; then
            host_icd="$(find "$dir" -maxdepth 1 -type f -name '*.json' -print -quit 2>/dev/null)"
            [ -n "$host_icd" ] && break
        fi
    done
    if [ -z "$host_icd" ]; then
        for icd in "$HERE"/usr/share/vulkan/icd.d/lvp_icd.*.json; do
            if [ -f "$icd" ]; then
                export VK_ICD_FILENAMES="$icd"
                break
            fi
        done
    fi
fi

exec "$HERE/usr/bin/tayf" "$@"
APPRUN_EOF
chmod +x "$APP_DIR/AppRun"

OUT_NAME="${APP_NAME}-${VERSION}-${ARCH}.AppImage"
OUT_PATH="$BUILD_DIR/$OUT_NAME"

echo "==> Building AppImage: $OUT_NAME"
"$APPIMAGETOOL" "$APP_DIR" "$OUT_PATH"

echo "==> Checksums"
sha256sum "$OUT_PATH" > "$OUT_PATH.sha256"

echo "==> Done: $OUT_PATH"
