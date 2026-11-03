#!/usr/bin/env python3
"""Refuse shipped code that sits on main at a version already released.

The release workflow writes a release's notes from the CHANGELOG section named by its tag.
So a note written into a section that has already shipped is never read again: the change
goes out in the next release with nothing said about it. Nothing catches that on its own.
The app's own test only asks that the newest `## ` section matches Cargo.toml, which is
just as true before a release as after it.

This asks the question that tells the two apart: is there source on main that the newest
tag doesn't have? If there is, the version must have moved on, and the note for it belongs
under that new heading.

Run it from anywhere in the checkout; `--verbose` prints what it found either way.
"""

import re
import subprocess
import sys

# Shipped code. Tests under `crates/*/tests` build the website's screenshots and never
# reach a user, so changing one is not a reason to cut a release.
SOURCE = ["crates", "apps", ":(exclude)crates/*/tests/*"]


def git(*args):
    """Run git in the repository holding this script, or die saying why it couldn't."""
    try:
        done = subprocess.run(
            ["git", *args], capture_output=True, text=True, check=True, cwd=repo_root()
        )
    except subprocess.CalledProcessError as e:
        sys.exit(f"git {' '.join(args)} failed: {e.stderr.strip()}")
    return done.stdout.strip()


def repo_root():
    out = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True
    )
    if out.returncode != 0:
        sys.exit("not inside a git checkout")
    return out.stdout.strip()


def version_key(tag):
    """`v0.1.9` sorts below `v0.1.10`, which a plain string sort gets backwards."""
    return tuple(int(n) for n in re.findall(r"\d+", tag))


def newest_tag():
    tags = [t for t in git("tag", "--list", "v*").splitlines() if re.fullmatch(r"v\d+(\.\d+)*", t)]
    return max(tags, key=version_key) if tags else None


def cargo_version():
    text = open(f"{repo_root()}/Cargo.toml", encoding="utf-8").read()
    found = re.search(r'(?m)^version\s*=\s*"([^"]+)"', text)
    if not found:
        sys.exit("no version in Cargo.toml")
    return found.group(1)


def main():
    verbose = "--verbose" in sys.argv
    tag = newest_tag()
    if tag is None:
        # A checkout without tags can't answer the question; say so rather than pass quietly.
        print("no v* tags found — skipping (fetch tags with `git fetch --tags`)")
        return 0

    version = cargo_version()
    commits = git("log", "--oneline", f"{tag}..HEAD", "--", *SOURCE).splitlines()

    if verbose:
        print(f"newest tag: {tag}\nCargo.toml: {version}\nunreleased source commits: {len(commits)}")

    if version_key(f"v{version}") < version_key(tag):
        print(
            f"Cargo.toml is {version}, behind the newest tag {tag}.\n"
            "A released version should never be ahead of the tree it was built from.",
            file=sys.stderr,
        )
        return 1

    if commits and f"v{version}" == tag:
        listing = "\n".join(f"  {c}" for c in commits)
        print(
            f"{len(commits)} commit(s) change shipped code since {tag}, but Cargo.toml is still"
            f" {version}, which {tag} already released:\n{listing}\n\n"
            f"Their notes would go under `## {version}` in CHANGELOG.md, a section the release"
            f" workflow has already used, so no release would ever mention them.\n"
            f"Raise the version in Cargo.toml, run `cargo update --workspace`, and start a new"
            f" `## <version>` section for these changes.",
            file=sys.stderr,
        )
        return 1

    if verbose:
        print("ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
