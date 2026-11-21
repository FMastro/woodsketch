#!/usr/bin/env bash
# A .deb and a .rpm of WoodSketch, for people who would rather install from their own package
# manager than run an AppImage.
#
#   scripts/linux-packages.sh <version> <arch> <bindir> <outdir>
#
# `arch` is the Debian name (amd64, arm64); the RPM name is worked out from it. `bindir` holds the
# built `woodsketch` and `woodsketch-cli`. Both packages are written into `outdir`.
#
# The files are named `woodsketch-linux-x64.deb` and so on, without the version, which is not the
# Debian convention but is how every other file in a release here is named: the website links
# `/releases/latest/download/<name>`, and that only works for a name that doesn't move. The
# version is inside both packages, where a package manager reads it from (`dpkg -I`, `rpm -qpi`).
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
  # `rpm_arch` is what rpm calls it; `name` is what the release's files are named by, the same
  # word the AppImage and the installers use.
  amd64) rpm_arch=x86_64; name=linux-x64; rpm_bits=64bit ;;
  arm64) rpm_arch=aarch64; name=linux-arm64; rpm_bits=64bit ;;
  *) echo "unknown architecture '$arch' (expected amd64 or arm64)" >&2; exit 2 ;;
esac
deb_file="$outdir/woodsketch-$name.deb"
rpm_file="$outdir/woodsketch-$name.rpm"

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
# --- what the program loads at runtime, which no scanner can see -----------------------------
# `dpkg-shlibdeps` above reads the binary's DT_NEEDED, and WoodSketch's is only libc, libm and
# libgcc: the windowing, the keyboard, the graphics and the file dialogs are all opened by name
# while the program runs (winit, wgpu and rfd each do their own `dlopen`), which a link table
# cannot show. Debian policy says such a dependency has to be declared by hand, so here it is,
# and `missing_sonames` below makes sure the hand doesn't fall behind the program.
#
# Each line is: soname package where-it-goes how-we-know. `need` is a dependency without which
# there is
# nothing to run at all; `want` is a Recommends, which apt installs by default and a minimal or
# unusual machine can go without. The pairs under `need` are alternatives: some display stack,
# and some way onto the GPU.
#
# The last column says how the entry is known, and it matters. `string` means the soname is a
# literal in the binary, so the two checks below hold the table and the program to each other in
# both directions. `traced` means the name is **assembled while the program runs** and appears
# nowhere in the file -- Vulkan's does, which is why removing it from this table once went
# unnoticed. Those entries can only be confirmed by watching the program:
#
#   strace -f -e trace=openat woodsketch-cli render table.json -o /tmp/t.png | grep -i vulkan
#
# which on a real Linux shows `openat(... "libvulkan.so.1") = 3`. So a `traced` line is a claim
# about behaviour, and the container checks in release.yml, which install the package and run the
# program, are what stand behind it.
#
# None of these packages was touched by the time64 renaming (libgtk-3-0t64 and friends), because
# rfd resolved to the XDG portal over zbus and WoodSketch links no GTK at all.
runtime_table="
libxkbcommon.so.0      libxkbcommon0        need         string
libX11.so.6            libx11-6             pick-display string
libwayland-client.so.0 libwayland-client0   pick-display string
libEGL.so.1            libegl1              pick-gpu     string
libvulkan.so.1         libvulkan1           pick-gpu     traced
libX11-xcb.so.1        libx11-xcb1          want         string
libXcursor.so.1        libxcursor1          want         string
libXi.so.6             libxi6               want         string
libxkbcommon-x11.so.0  libxkbcommon-x11-0   want         string
libwayland-egl.so.1    libwayland-egl1      want         string
libdbus-1.so.3         libdbus-1-3          want         string
"
# Covered by DT_NEEDED already, so shlibdeps names their packages itself.
linked_in="libc.so.6 libm.so.6 libgcc_s.so.1 ld-linux-x86-64.so.2 ld-linux-aarch64.so.1"

# Every soname either binary can ask for. Rust packs its string literals end to end, so these
# don't sit on lines of their own and the match can't be anchored.
found_sonames() {
  strings -a "$bindir/woodsketch" "$bindir/woodsketch-cli" | grep -oE 'lib[A-Za-z0-9_+-]+\.so\.[0-9]+' | sort -u
}

# A soname the program can load that nothing above accounts for. That is the whole point of this
# check: a dependency added upstream can start dlopening something new, and the package would
# otherwise go out quietly missing it.
missing_sonames() {
  local so
  for so in $(found_sonames); do
    case " $linked_in " in *" $so "*) continue ;; esac
    echo "$runtime_table" | grep -qE "^$so[[:space:]]" || echo "$so"
  done
}

# The other direction: a line claiming to be visible in the binary, which isn't there any more.
# Left alone, the package would keep asking for a library nothing loads, and on a distribution
# that renamed it nobody could install at all.
stale_sonames() {
  local so
  for so in $(echo "$runtime_table" | awk '$4 == "string" {print $1}'); do
    found_sonames | grep -qx "$so" || echo "$so"
  done
}

stale=$(stale_sonames)
if [ -n "$stale" ]; then
  echo "These are declared as literals in the binary but are not in it any more:" >&2
  for s in $stale; do echo "  $s" >&2; done
  echo "Take each out of runtime_table in $0, or mark it 'traced' and say how it was confirmed." >&2
  exit 1
fi

missing=$(missing_sonames)
if [ -n "$missing" ]; then
  echo "These libraries are loaded at run time and declared nowhere:" >&2
  for m in $missing; do echo "  $m" >&2; done
  echo "Add each to runtime_table in $0 (need, pick-display, pick-gpu or want), with the Debian" >&2
  echo "package that provides it, or to linked_in if the linker records it after all." >&2
  exit 1
fi

# The table, as dependency fields. The `pick-` groups become alternatives: one of them will do.
column() { echo "$runtime_table" | awk -v want="$2" '$3 == want {print $2}' | paste -sd "$1" -; }
needs=$(column ',' need)
display=$(column '|' pick-display)
gpu=$(column '|' pick-gpu)
wants=$(column ',' want)
# Every alternative is also a Recommends, so an ordinary install pulls the lot and only an
# unusual one is left choosing.
wants="$wants,$(column ',' pick-display),$(column ',' pick-gpu)"
depends="$depends, $needs, $display, $gpu"
echo "Depends: $depends"
echo "Recommends: $wants"
cat > "$deb/DEBIAN/control" <<CONTROL
Package: woodsketch
Version: $version
Section: graphics
Priority: optional
Architecture: $arch
Maintainer: Fred Mastropasqua <noreply@woodsketch.net>
Homepage: https://woodsketch.net
Depends: $depends
Recommends: $wants
Description: 3D modeler for woodworkers
$(printf '%s\n' "$description" | sed 's/^/ /')
CONTROL
# Reproducible-ish: one timestamp for everything, rather than whenever each file was copied.
dpkg-deb --root-owner-group -Zxz --build "$deb" "$deb_file" >/dev/null
echo "wrote $deb_file"

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
$(echo "$runtime_table" | awk -v bits="$rpm_bits" '$3 == "need" {print "Requires:       " $1 "()(" bits ")"}')
$(echo "$runtime_table" | awk -v bits="$rpm_bits" '$3 == "pick-display" { s = s (s ? " or " : "") $1 "()(" bits ")" } END { if (s) print "Requires:       (" s ")" }')
$(echo "$runtime_table" | awk -v bits="$rpm_bits" '$3 == "pick-gpu" { s = s (s ? " or " : "") $1 "()(" bits ")" } END { if (s) print "Requires:       (" s ")" }')
$(echo "$runtime_table" | awk -v bits="$rpm_bits" '$3 == "want" {print "Recommends:     " $1 "()(" bits ")"}')
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
# `--target` as well as the spec's BuildArch, to say plainly what is being built. Note that rpm
# builds for the machine it is on and no other: Ubuntu's refuses a foreign architecture outright
# ("No compatible architectures found for build"), which is fine here because each release job
# runs on the architecture it packages, but means the .rpm for the *other* one can't be checked
# from this machine. The .deb can: dpkg doesn't care what it is running on.
rpmbuild --define "_topdir $rpm_top" --target "$rpm_arch" --buildroot "$root" -bb "$rpm_top/SPECS/woodsketch.spec" >/dev/null
built=$(find "$rpm_top/RPMS" -name '*.rpm' -print -quit)
[ -n "$built" ] || { echo "rpmbuild made no package" >&2; exit 1; }
mv "$built" "$rpm_file"
echo "wrote $rpm_file"

# What each one says it needs, in the log, so a dependency that goes missing is visible in the
# build that made it rather than in somebody's failed install.
echo "--- deb depends ---"
dpkg-deb -f "$deb_file" Depends
echo "--- rpm requires ---"
rpm -qp --requires "$rpm_file" 2>/dev/null || true
