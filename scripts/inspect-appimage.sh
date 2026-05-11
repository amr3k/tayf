#!/usr/bin/env sh
set -eu

APPIMAGE="${1:-}"

if [ -z "$APPIMAGE" ]; then
  APPIMAGE="$(find src-tauri/target/release/bundle/appimage -maxdepth 1 -type f -name '*.AppImage' -printf '%T@ %p\n' 2>/dev/null | sort -nr | awk 'NR == 1 { $1 = ""; sub(/^ /, ""); print }')"
fi

if [ -z "$APPIMAGE" ] || [ ! -f "$APPIMAGE" ]; then
  echo "No AppImage found. Build one with: pnpm build:appimage" >&2
  exit 1
fi

APPIMAGE_ABS="$(cd "$(dirname "$APPIMAGE")" && pwd)/$(basename "$APPIMAGE")"
TMPDIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TMPDIR"
}
trap cleanup EXIT INT TERM

chmod +x "$APPIMAGE" 2>/dev/null || true

(
  cd "$TMPDIR"
  "$APPIMAGE_ABS" --appimage-extract >/dev/null
)

APPDIR="$TMPDIR/squashfs-root"
DESKTOP_FILE="$(find "$APPDIR" \( -type f -o -type l \) -name '*.desktop' | head -n 1)"
ICON_COUNT="$(find "$APPDIR" -type f \( -path '*/icons/*' -o -name '*.png' -o -name '*.svg' \) | wc -l | tr -d ' ')"

echo "AppImage: $APPIMAGE"
echo "Size: $(du -h "$APPIMAGE" | awk '{ print $1 }')"

if [ -n "$DESKTOP_FILE" ]; then
  echo
  echo "Desktop entry: ${DESKTOP_FILE#$APPDIR/}"
  grep -E '^(Name|Comment|Exec|Icon|MimeType|Categories)=' "$DESKTOP_FILE" || true

  if command -v desktop-file-validate >/dev/null 2>&1; then
    desktop-file-validate "$DESKTOP_FILE"
  fi

  grep -q '^Name=AnimaView$' "$DESKTOP_FILE"
  grep -q '^Icon=animaview$' "$DESKTOP_FILE"
  grep -q '^Exec=.*%F' "$DESKTOP_FILE"
  grep -q '^Categories=.*Graphics' "$DESKTOP_FILE"
  grep -q '^MimeType=.*application/json' "$DESKTOP_FILE"
  grep -q '^MimeType=.*video/lottie+json' "$DESKTOP_FILE"
  grep -q '^MimeType=.*application/zip+dotlottie' "$DESKTOP_FILE"
else
  echo "No desktop entry found" >&2
  exit 1
fi

echo
echo "Icon files:"
find "$APPDIR" -type f \( -path '*/icons/*' -o -name '*.png' -o -name '*.svg' \) | sed "s#^$APPDIR/##" | sort | head -n 40

if [ "$ICON_COUNT" -eq 0 ]; then
  echo "No icons found" >&2
  exit 1
fi

if [ ! -f "$APPDIR/.DirIcon" ]; then
  echo "No root .DirIcon found" >&2
  exit 1
fi

if [ -L "$APPDIR/.DirIcon" ]; then
  echo "Root .DirIcon is a symlink; expected a portable PNG file" >&2
  exit 1
fi

if [ ! -f "$APPDIR/animaview.png" ]; then
  echo "No root animaview.png icon found" >&2
  exit 1
fi

if [ -L "$APPDIR/animaview.png" ]; then
  echo "Root animaview.png is a symlink; expected a portable PNG file" >&2
  exit 1
fi

if command -v file >/dev/null 2>&1; then
  file -b "$APPDIR/.DirIcon" | grep -q 'PNG image data'
  file -b "$APPDIR/animaview.png" | grep -q 'PNG image data'
fi

GTK_HOOK="$APPDIR/apprun-hooks/linuxdeploy-plugin-gtk.sh"
if [ -f "$GTK_HOOK" ]; then
  echo
  echo "GTK backend hook:"
  grep -n 'GDK_BACKEND' "$GTK_HOOK" || true

  if grep -Eq '^[[:space:]]*export GDK_BACKEND=x11([[:space:]]|#|$)' "$GTK_HOOK"; then
    echo "GTK hook still hard-codes the X11 backend" >&2
    exit 1
  fi

  grep -q 'GDK_BACKEND=x11,wayland' "$GTK_HOOK"
  grep -q 'GDK_BACKEND=wayland,x11' "$GTK_HOOK"

  wayland_line="$(grep -n 'GDK_BACKEND=wayland,x11' "$GTK_HOOK" | head -n 1 | cut -d: -f1)"
  x11_line="$(grep -n 'GDK_BACKEND=x11,wayland' "$GTK_HOOK" | head -n 1 | cut -d: -f1)"
  if [ "$wayland_line" -gt "$x11_line" ]; then
    echo "GTK hook should prefer Wayland before X11" >&2
    exit 1
  fi
fi

echo
echo "MIME references:"
grep -R "lottie\\|dotlottie\\|video/lottie\\|application/zip+dotlottie" "$APPDIR" 2>/dev/null | head -n 40 || true

if ! find "$APPDIR" -path '*/mime/packages/*.xml' -type f | grep -q .; then
  echo "No bundled shared-mime-info XML found" >&2
  exit 1
fi

if command -v update-mime-database >/dev/null 2>&1 && [ -d "$APPDIR/usr/share/mime" ]; then
  update-mime-database -n "$APPDIR/usr/share/mime" >/dev/null 2>&1
fi

echo
echo "Inspection passed."
