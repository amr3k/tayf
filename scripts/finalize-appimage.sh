#!/usr/bin/env sh
set -eu

BUNDLE_DIR="src-tauri/target/release/bundle/appimage"
APPIMAGE="${1:-}"
APPDIR="${APPDIR:-}"

if [ -z "$APPIMAGE" ]; then
  APPIMAGE="$(find "$BUNDLE_DIR" -maxdepth 1 -type f -name '*.AppImage' -printf '%T@ %p\n' 2>/dev/null | sort -nr | awk 'NR == 1 { $1 = ""; sub(/^ /, ""); print }')"
fi

if [ -z "$APPIMAGE" ] || [ ! -f "$APPIMAGE" ]; then
  echo "No AppImage found. Build one with: pnpm tauri build --bundles appimage" >&2
  exit 1
fi

if [ -z "$APPDIR" ]; then
  APPDIR="$(find "$BUNDLE_DIR" -maxdepth 1 -type d -name '*.AppDir' -printf '%T@ %p\n' 2>/dev/null | sort -nr | awk 'NR == 1 { $1 = ""; sub(/^ /, ""); print }')"
fi

if [ -z "$APPDIR" ] || [ ! -d "$APPDIR" ]; then
  echo "No AppDir found beside $APPIMAGE" >&2
  exit 1
fi

TMPDIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TMPDIR"
}
trap cleanup EXIT INT TERM

set_desktop_key() {
  file="$1"
  key="$2"
  value="$3"
  tmp="$TMPDIR/$(basename "$file").$key"

  awk -v key="$key" -v line="$key=$value" '
    BEGIN { done = 0 }
    index($0, key "=") == 1 {
      if (!done) {
        print line
        done = 1
      }
      next
    }
    { print }
    END {
      if (!done) {
        print line
      }
    }
  ' "$file" > "$tmp"

  cat "$tmp" > "$file"
}

patch_desktop_entries() {
  desktop_count=0

  while IFS= read -r desktop_file; do
    [ -f "$desktop_file" ] || continue
    desktop_count=$((desktop_count + 1))
    set_desktop_key "$desktop_file" "Exec" "animaview %F"
    set_desktop_key "$desktop_file" "Categories" "Graphics;2DGraphics;Viewer;"
    set_desktop_key "$desktop_file" "MimeType" "application/json;video/lottie+json;application/zip+dotlottie;"
    set_desktop_key "$desktop_file" "Icon" "animaview"
  done <<EOF
$(find "$APPDIR" \( -type f -o -type l \) -name '*.desktop' | sort)
EOF

  if [ "$desktop_count" -eq 0 ]; then
    echo "No desktop entry found in $APPDIR" >&2
    exit 1
  fi

  if command -v desktop-file-validate >/dev/null 2>&1; then
    while IFS= read -r desktop_file; do
      [ -f "$desktop_file" ] || continue
      desktop-file-validate "$desktop_file"
    done <<EOF
$(find "$APPDIR" \( -type f -o -type l \) -name '*.desktop' | sort)
EOF
  fi
}

strip_elf_files() {
  if [ "${APPIMAGE_SKIP_HOST_STRIP:-0}" = "1" ]; then
    echo "Skipping host strip because APPIMAGE_SKIP_HOST_STRIP=1"
    return
  fi

  strip_bin="${STRIP:-}"
  if [ -z "$strip_bin" ]; then
    strip_bin="$(command -v strip || true)"
  fi

  file_bin="$(command -v file || true)"
  if [ -z "$strip_bin" ] || [ -z "$file_bin" ]; then
    echo "Skipping host strip because strip or file was not found"
    return
  fi

  stripped_count_file="$TMPDIR/stripped-count"
  failed_count_file="$TMPDIR/strip-failed-count"
  echo 0 > "$stripped_count_file"
  echo 0 > "$failed_count_file"

  while IFS= read -r appdir_file; do
    if "$file_bin" -b "$appdir_file" | grep -q 'ELF'; then
      if "$strip_bin" --strip-unneeded "$appdir_file" >/dev/null 2>&1; then
        count="$(cat "$stripped_count_file")"
        echo $((count + 1)) > "$stripped_count_file"
      else
        count="$(cat "$failed_count_file")"
        echo $((count + 1)) > "$failed_count_file"
      fi
    fi
  done <<EOF
$(find "$APPDIR" -type f | sort)
EOF

  echo "Host-stripped $(cat "$stripped_count_file") ELF files ($(cat "$failed_count_file") skipped/failed)."
}

extract_appimagetool() {
  linuxdeploy_appimage="${LINUXDEPLOY_APPIMAGE:-$HOME/.cache/tauri/linuxdeploy-x86_64.AppImage}"
  if [ ! -f "$linuxdeploy_appimage" ]; then
    echo "Could not find linuxdeploy AppImage at $linuxdeploy_appimage" >&2
    echo "Set APPIMAGETOOL=/path/to/appimagetool or LINUXDEPLOY_APPIMAGE=/path/to/linuxdeploy-x86_64.AppImage." >&2
    exit 1
  fi

  (
    cd "$TMPDIR"
    "$linuxdeploy_appimage" --appimage-extract >/dev/null
  )

  appimagetool_prefix="$TMPDIR/squashfs-root/plugins/linuxdeploy-plugin-appimage/appimagetool-prefix"
  appimagetool="$appimagetool_prefix/usr/bin/appimagetool"

  if [ ! -x "$appimagetool" ]; then
    echo "Could not find appimagetool inside $linuxdeploy_appimage" >&2
    exit 1
  fi

  APPIMAGETOOL="$appimagetool"
  APPIMAGETOOL_LD_LIBRARY_PATH="$appimagetool_prefix/usr/lib"
}

repack_appimage() {
  if [ -n "${APPIMAGETOOL:-}" ]; then
    APPIMAGETOOL_LD_LIBRARY_PATH="${APPIMAGETOOL_LD_LIBRARY_PATH:-}"
  elif command -v appimagetool >/dev/null 2>&1; then
    APPIMAGETOOL="$(command -v appimagetool)"
    APPIMAGETOOL_LD_LIBRARY_PATH=""
  else
    extract_appimagetool
  fi

  output="$TMPDIR/$(basename "$APPIMAGE")"
  old_ld_library_path="${LD_LIBRARY_PATH:-}"
  if [ -n "$APPIMAGETOOL_LD_LIBRARY_PATH" ]; then
    export LD_LIBRARY_PATH="$APPIMAGETOOL_LD_LIBRARY_PATH${old_ld_library_path:+:$old_ld_library_path}"
  fi

  appimage_comp="${APPIMAGETOOL_COMP:-xz}"
  ARCH="${ARCH:-x86_64}" "$APPIMAGETOOL" -n --comp "$appimage_comp" "$APPDIR" "$output" >/dev/null
  mv "$output" "$APPIMAGE"
  chmod +x "$APPIMAGE"
}

patch_desktop_entries
strip_elf_files
repack_appimage

echo "Finalized AppImage: $APPIMAGE"
