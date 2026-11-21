#!/usr/bin/env python3
"""Render the User Guide's joint pictures from docs/guide/recipes/.

Each recipe is two boards and one joint -- the smallest drawing that shows what the joint looks
like. They are recipes rather than saved drawings on purpose: every picture is built by today's
joinery code, so changing how a dovetail is cut changes the picture rather than leaving a
rendering of how it used to be cut.

A recipe's file name is the `JointKind` it is meant to produce, and this checks the built
drawing really did come out with that joint before rendering it. A recipe that stops working
says so here rather than quietly picturing the wrong joint.

Two PNGs per joint, light and dark, because mdBook has themes of both kinds and a light
picture on a dark page is a glaring rectangle; `assets/diagrams.css` shows whichever matches.

    cargo build --release -p woodsketch-cli
    python -I scripts/make-guide-pictures.py          # into docs/guide/src/images/joints/
    python -I scripts/make-guide-pictures.py --check  # fail if any are missing

Rendering needs a GPU, so CI cannot do it: the pictures are made here and committed, and
`crates/ui-egui/tests/guide.rs` checks that every joint the app can make has one.
"""

import argparse
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RECIPES = ROOT / "docs/guide/recipes"
PICTURES = ROOT / "docs/guide/src/images/joints"
SIZE = "900x620"
FIT = "0.82"
VIEW_ANGLE = "28,22"


def cli():
    for name in ("woodsketch-cli.exe", "woodsketch-cli"):
        exe = ROOT / "target/release" / name
        if exe.exists():
            return exe
    sys.exit("build it first: cargo build --release -p woodsketch-cli")


def run(*args):
    done = subprocess.run([str(a) for a in args], capture_output=True, text=True)
    if done.returncode != 0:
        sys.exit(f"{' '.join(str(a) for a in args[1:3])} failed:\n{done.stderr.strip()}")
    return done.stdout


def names_the_same(stem, made):
    """Whether the joint built is the one the recipe's file name promised.

    Compared loosely on purpose: the app writes "half-lap" with a hyphen where the file name
    has one anyway, and `JointKind::Splines` is called "Spline" in the singular. Hyphens
    become spaces on both sides and a trailing s is optional, which is enough to tell a dado
    from a rabbet without being brittle about wording.
    """
    plain = lambda s: s.lower().replace("-", " ").strip()
    want, made = plain(stem), plain(made)
    return want in made or want.rstrip("s") in made


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="only report what is missing")
    chosen = parser.parse_args()

    recipes = sorted(RECIPES.glob("*.json"))
    if not recipes:
        sys.exit(f"no recipes in {RECIPES}")

    if chosen.check:
        missing = [
            f"{r.stem} ({theme})"
            for r in recipes
            for theme in ("light", "dark")
            if not (PICTURES / f"{r.stem}-{theme}.png").exists()
        ]
        print(f"{len(recipes) * 2} pictures wanted, {len(missing)} missing")
        if missing:
            sys.exit("missing:\n  " + "\n  ".join(missing))
        return 0

    exe = cli()
    PICTURES.mkdir(parents=True, exist_ok=True)
    scratch = Path(tempfile.mkdtemp())
    total = 0
    for recipe in recipes:
        drawing = scratch / f"{recipe.stem}.woodsketch"
        said = run(exe, "build", recipe, "-o", drawing)

        # The recipe's name is a promise about what it builds; hold it to that.
        line = next((l for l in said.splitlines() if l.startswith("Joints:")), "")
        made = re.sub(r"^Joints:\s*|\.$", "", line).strip()
        if not names_the_same(recipe.stem, made):
            want = recipe.stem.replace("-", " ")
            sys.exit(f"{recipe.name} should build a {want} but built: {made or '(no joint at all)'}")

        for theme, flag in (("light", []), ("dark", ["--dark"])):
            out = PICTURES / f"{recipe.stem}-{theme}.png"
            run(exe, "render", drawing, "-o", out, "--angle", VIEW_ANGLE,
                "--size", SIZE, "--fit", FIT, "--no-axes", *flag)
            total += 1
        print(f"  {recipe.stem:20} {made}")

    weight = sum(p.stat().st_size for p in PICTURES.glob("*.png")) // 1024
    print(f"\n{total} pictures into {PICTURES.relative_to(ROOT)} ({weight} KB)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
