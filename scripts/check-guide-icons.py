#!/usr/bin/env python3
"""Refuse a built User Guide whose toolbar icons resolve to nothing.

The guide shows each tool's own Phosphor icon, drawn as a CSS mask so it takes the app's
colours and follows mdBook's light and dark themes. A mask whose url points at a file that
isn't there fails silently: no error from mdBook, no error from the browser, just an empty
square where the icon should be. That is exactly what happened once, when the stylesheet was
written a folder deeper than its urls assumed.

So this resolves every url() in the generated stylesheet against the built book, the same way
a browser would, and says which ones miss. Run it after `mdbook build docs/guide`.

`crates/ui-egui/tests/guide.rs` checks the other half -- that every tool has artwork vendored
and that nothing is vendored which no tool uses -- but it cannot check the built layout,
because the layout is mdBook's doing.
"""

import re
import sys
from pathlib import Path

STYLESHEET = "docs/guide/book/assets/icons.css"


def main():
    root = Path(__file__).resolve().parent.parent
    css = root / STYLESHEET
    if not css.exists():
        sys.exit(f"{STYLESHEET} isn't there; run `mdbook build docs/guide` first")

    urls = sorted(set(re.findall(r"url\(([^)]+)\)", css.read_text(encoding="utf-8"))))
    if not urls:
        sys.exit(f"{STYLESHEET} asks for no icons at all, which means the guide shows none")

    missing = [u for u in urls if not (css.parent / u).resolve().exists()]
    print(f"{len(urls)} icon masks, {len(missing)} missing")
    if missing:
        listing = "\n  ".join(missing)
        sys.exit(
            "a mask that resolves to nothing is an empty square on the page, which nothing"
            f" else would notice:\n  {listing}"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())
