#!/usr/bin/env bash
# A .deb and a .rpm of WoodSketch, for people who would rather install from their own package
# manager than run an AppImage.
#
#   scripts/linux-packages.sh <version> <arch> <bindir> <outdir>
#
# `arch` is the Debian name (amd64, arm64); the RPM name is worked out from it. `bindir` holds the
# built `woodsketch` and `woodsketch-cli`. Both packages are written into `outdir`.
#
# These installs are NOT updated by WoodSketch itself, on purpose: apt and dnf own what they
# installed, and `/usr/bin` isn't somewhere an app should be rewriting itself. Velopack agrees
# without being told -- its Linux side resolves everything from `$APPIMAGE`, which isn't set here,
# so the updater reports "not installed" and the app never offers to update itself. The AppImage
# beside these packages is the self-updating one.
#
# Dependencies are not written by hand. `dpkg-shlibdeps` reads the binary and names the packages
# that provide the libraries it actually asks for, and rpm's own find-requires does the same as
# sonames, which is what makes one .rpm work on Fedora and openSUSE alike. A hand-kept list would
# be a guess that rots with every dependency added.
#
# Nothing here installs a desktop or MIME database: Debian and Fedora both run those from triggers
# when a package drops files in /usr/share/applications and /usr/share/mime/packages, so a
# maintainer script would only be doing it twice.
set -euo pipefail

if [ "$#" -ne 4 ]; then
  echo "usage: $0 <version> <deb-arch: amd64|arm64> <bindir> <outdir>" >&2
  exit 2
fi
version="$1"
arch="$2"
bindir="$3"
outdir="$4"
here=$(cd "$(dirname "$0")/.." && pwd)

case "$arch" in
  amd64) rpm_arch=x86_64 ;;
  arm64) rpm_arch=aarch64 ;;
  *) echo "unknown architecture '$arch' (expected amd64 or arm64)" >&2; exit 2 ;;
esac

for tool in dpkg-deb dpkg-shlibdeps rpmbuild; do
  command -v "$tool" >/dev/null || { echo "$tool is needed (apt-get install dpkg-dev rpm)" >&2; exit 1; }
done
for exe in woodsketch woodsketch-cli; do
  [ -x "$bindir/$exe" ] || { echo "$bindir/$exe isn't there" >&2; exit 1; }
done

mkdir -p "$outdir"
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
root="$stage/root"

# What both packages install, laid out as it lands on the machine.
install -Dm755 "$bindir/woodsketch" "$root/usr/bin/woodsketch"
install -Dm755 "$bindir/woodsketch-cli" "$root/usr/bin/woodsketch-cli"
install -Dm644 "$here/packaging/linux/woodsketch.desktop" "$root/usr/share/applications/woodsketch.desktop"
install -Dm644 "$here/packaging/linux/woodsketch.xml" "$root/usr/share/mime/packages/woodsketch.xml"
install -Dm644 "$here/THIRD-PARTY-NOTICES.md" "$root/usr/share/doc/woodsketch/THIRD-PARTY-NOTICES.md"
install -Dm644 "$here/LICENSE" "$root/usr/share/doc/woodsketch/LICENSE"
# Every icon size there is, so the taskbar, the menu and the file manager each find one they can
# use without scaling. 1024 isn't a hicolor size; the SVG covers anything bigger.
for size in 16 32 48 64 128 256 512; do
  install -Dm644 "$here/assets/icon/woodsketch-$size.png" "$root/usr/share/icons/hicolor/${size}x${size}/apps/woodsketch.png"
done
install -Dm644 "$here/assets/icon/woodsketch.svg" "$root/usr/share/icons/hicolor/scalable/apps/woodsketch.svg"

description="A SketchUp-style 3D modeler for woodworkers: design furniture, work out the joinery,
 and print a cut list, cutting diagrams and build sheets."

# --- the .deb ---------------------------------------------------------------------------------
deb="$stage/deb"
cp -a "$root" "$deb"
mkdir -p "$deb/DEBIAN"
# dpkg-shlibdeps wants a debian/control to exist before it will say anything, and reads the
# binaries from the package root it is run in.
mkdir -p "$deb/debian"
printf 'Source: woodsketch\nMaintainer: Fred Mastropasqua <noreply@woodsketch.net>\n\nPackage: woodsketch\nArchitecture: any\nDescription: placeholder\n' \
  > "$deb/debian/control"
depends=$(cd "$deb" && dpkg-shlibdeps -O --ignore-missing-info usr/bin/woodsketch usr/bin/woodsketch-cli 2>/dev/null | sed 's/^shlibs:Depends=//')
rm -rf "$deb/debian"
if [ -z "$depends" ]; then
  echo "dpkg-shlibdeps named no dependencies, which can't be right for a GUI app" >&2
  exit 1
fi
echo "Depends: $depends"
cat > "$deb/DEBIAN/control" <<CONTROL
Package: woodsketch
Version: $version
Section: graphics
Priority: optional
Architecture: $arch
Maintainer: Fred Mastropasqua <noreply@woodsketch.net>
Homepage: https://woodsketch.net
Depends: $depends
Description: 3D modeler for woodworkers
$(printf '%s\n' "$description" | sed 's/^/ /')
CONTROL
# Reproducible-ish: one timestamp for everything, rather than whenever each file was copied.
dpkg-deb --root-owner-group -Zxz --build "$deb" "$outdir/woodsketch_${version}_${arch}.deb" >/dev/null
echo "wrote $outdir/woodsketch_${version}_${arch}.deb"

# --- the .rpm ---------------------------------------------------------------------------------
# rpmbuild insists on building from its own tree, so the staged files are handed to it as the
# buildroot and the spec installs nothing itself.
rpm_top="$stage/rpm"
mkdir -p "$rpm_top"/{BUILD,RPMS,SPECS}
cat > "$rpm_top/SPECS/woodsketch.spec" <<SPEC
Name:           woodsketch
Version:        $version
Release:        1
Summary:        3D modeler for woodworkers
License:        MIT OR Apache-2.0
URL:            https://woodsketch.net
BuildArch:      $rpm_arch
# The binaries are already built; this spec only packs them.
%global __os_install_post %{nil}
%global debug_package %{nil}

%description
$description

%files
/usr/bin/woodsketch
/usr/bin/woodsketch-cli
/usr/share/applications/woodsketch.desktop
/usr/share/mime/packages/woodsketch.xml
/usr/share/icons/hicolor/*/apps/woodsketch.*
%dir /usr/share/doc/woodsketch
/usr/share/doc/woodsketch/*

%changelog
SPEC
rpmbuild --define "_topdir $rpm_top" --buildroot "$root" -bb "$rpm_top/SPECS/woodsketch.spec" >/dev/null
built=$(find "$rpm_top/RPMS" -name '*.rpm' -print -quit)
[ -n "$built" ] || { echo "rpmbuild made no package" >&2; exit 1; }
mv "$built" "$outdir/woodsketch-$version-1.$rpm_arch.rpm"
echo "wrote $outdir/woodsketch-$version-1.$rpm_arch.rpm"

# What each one says it needs, in the log, so a dependency that goes missing is visible in the
# build that made it rather than in somebody's failed install.
echo "--- deb depends ---"
dpkg-deb -f "$outdir/woodsketch_${version}_${arch}.deb" Depends
echo "--- rpm requires ---"
rpm -qp --requires "$outdir/woodsketch-$version-1.$rpm_arch.rpm" 2>/dev/null || true
