#!/usr/bin/env python3
"""Render the User Guide's pictures from docs/guide/recipes/.

Three groups, each the smallest drawing that shows what the guide is naming:

    joints/     two boards and one joint
    profiles/   one chunky block with a profile run round its top face
    templates/  one template at its own default sizes

They are recipes (docs/AGENTS.md) rather than saved drawings on purpose: every picture is
built by today's code, so changing how a dovetail is cut, or what a cabinet template builds,
changes the picture instead of leaving a rendering of how it used to be.

A recipe's file name is a promise, and this holds it to it. A joint recipe has to report that
joint when it builds. A profile or template recipe has to still describe it when the built
drawing is read back as a recipe -- which is `recipe_of` doing the checking, so a read-back
that quietly stopped keeping profiles would fail here too.

Two PNGs each, light and dark, because mdBook has themes of both kinds and a light picture on
a dark page is a glaring rectangle; `assets/diagrams.css` shows whichever matches.

    cargo build --release -p woodsketch-cli
    python -I scripts/make-guide-pictures.py          # into docs/guide/src/images/
    python -I scripts/make-guide-pictures.py --check  # fail if any are missing

Rendering needs a GPU, so CI cannot do it: the pictures are made here and committed, and
`crates/ui-egui/tests/guide.rs` checks that every joint, profile and template has one.
"""

import argparse
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RECIPES = ROOT / "docs/guide/recipes"
PICTURES = ROOT / "docs/guide/src/images"

# How each group is framed. A profile is half an inch of shape on one edge, so it is shot
# closer and tighter than a cabinet.
GROUPS = {
    "joints": {"angle": "28,22", "size": "900x620", "fit": "0.82"},
    "profiles": {"angle": "32,24", "size": "760x520", "fit": "0.82"},
    "templates": {"angle": "26,20", "size": "900x700", "fit": "0.86"},
}


def cli():
    for name in ("woodsketch-cli.exe", "woodsketch-cli"):
        exe = ROOT / "target/release" / name
        if exe.exists():
            return exe
    sys.exit("build it first: cargo build --release -p woodsketch-cli")


def run(exe, *args):
    done = subprocess.run([str(exe), *[str(a) for a in args]], capture_output=True, text=True)
    if done.returncode != 0:
        sys.exit(f"`{args[0]}` failed on {args[1]}:\n{done.stderr.strip()}")
    return done.stdout


def plain(text):
    """Hyphens and case are not the point: the app writes "half-lap" and calls `Splines`
    "Spline", and a file name cannot carry an underscore and a hyphen at once."""
    return text.lower().replace("-", " ").replace("_", " ").strip()


def promised(group, stem, exe, drawing, built):
    """Whether the drawing is the thing the recipe's file name said it would be."""
    if group == "joints":
        line = next((l for l in built.splitlines() if l.startswith("Joints:")), "")
        made = re.sub(r"^Joints:\s*|\.$", "", line).strip()
        want = plain(stem)
        ok = want in plain(made) or want.rstrip("s") in plain(made)
        return ok, made or "no joint at all"
    # Read the built drawing back as a recipe: whatever `recipe_of` can still say about it.
    back = json.loads(run(exe, "recipe", drawing))
    said = [p.get("profile", "") for p in back.get("profiles", [])] if group == "profiles" else [t.get("kind", "") for t in back.get("templates", [])]
    return any(plain(stem) == plain(s) for s in said), ", ".join(said) or f"no {group[:-1]} at all"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="only report what is missing")
    chosen = parser.parse_args()

    wanted = {group: sorted((RECIPES / group).glob("*.json")) for group in GROUPS}
    for group, recipes in wanted.items():
        if not recipes:
            sys.exit(f"no recipes in {RECIPES / group}")

    if chosen.check:
        missing = [
            f"{group}/{r.stem} ({theme})"
            for group, recipes in wanted.items()
            for r in recipes
            for theme in ("light", "dark")
            if not (PICTURES / group / f"{r.stem}-{theme}.png").exists()
        ]
        total = sum(len(r) for r in wanted.values()) * 2
        print(f"{total} pictures wanted, {len(missing)} missing")
        if missing:
            sys.exit("missing:\n  " + "\n  ".join(missing))
        return 0

    exe = cli()
    scratch = Path(tempfile.mkdtemp())
    made = 0
    for group, recipes in wanted.items():
        shot = GROUPS[group]
        (PICTURES / group).mkdir(parents=True, exist_ok=True)
        print(f"{group}:")
        for recipe in recipes:
            drawing = scratch / f"{group}-{recipe.stem}.woodsketch"
            built = run(exe, "build", recipe, "-o", drawing)
            ok, said = promised(group, recipe.stem, exe, drawing, built)
            if not ok:
                sys.exit(f"{group}/{recipe.name} should build a {plain(recipe.stem)} but built: {said}")
            for theme, flag in (("light", []), ("dark", ["--dark"])):
                out = PICTURES / group / f"{recipe.stem}-{theme}.png"
                run(exe, "render", drawing, "-o", out, "--angle", shot["angle"],
                    "--size", shot["size"], "--fit", shot["fit"], "--no-axes", *flag)
                made += 1
            print(f"  {recipe.stem:20} {said}")

    weight = sum(p.stat().st_size for p in PICTURES.rglob("*.png")) // 1024
    print(f"\n{made} pictures into {PICTURES.relative_to(ROOT)} ({weight} KB)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
