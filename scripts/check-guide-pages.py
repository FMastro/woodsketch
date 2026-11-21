#!/usr/bin/env python3
"""Refuse a built User Guide whose drawings came out as text.

The diagrams are inlined into a page with mdBook's `{{#include}}` so their strokes can follow
the reader's theme. Markdown ends an HTML block at the first blank line, so a blank line inside
an SVG leaves everything after it to be parsed as Markdown -- and because SVG is indented, that
means an indented code block. The rest of the drawing then appears on the page as escaped XML
in a grey box. Nothing reports it: mdBook is happy, the browser is happy, and the page still
contains an `<svg`, which is why counting those was not enough to notice.

So this looks for the symptom instead: SVG markup that reached the page escaped. It also
catches an include that never resolved, which leaves its `{{#include ...}}` sitting in the text.

Run it after `mdbook build docs/guide`. `crates/ui-egui/tests/guide.rs` checks the source side
(no blank lines in a drawing); this checks what was actually rendered.
"""

import sys
from pathlib import Path

BOOK = "docs/guide/book"
ESCAPED = ["&lt;svg", "&lt;path", "&lt;rect", "&lt;circle", "&lt;g ", "&lt;text", "&lt;marker", "&lt;defs"]


def main():
    root = Path(__file__).resolve().parent.parent
    book = root / BOOK
    if not book.exists():
        sys.exit(f"{BOOK} isn't there; run `mdbook build docs/guide` first")

    pages = sorted(book.rglob("*.html"))
    if not pages:
        sys.exit(f"no pages in {BOOK}")

    problems = []
    for page in pages:
        text = page.read_text(encoding="utf-8", errors="replace")
        where = page.relative_to(book)
        found = [tag for tag in ESCAPED if tag in text]
        if found:
            problems.append(
                f"{where}: drawing markup reached the page as text ({', '.join(found)})."
                " A blank line inside an included SVG does this."
            )
        if "{{#include" in text:
            problems.append(f"{where}: an {{{{#include}}}} was never resolved.")

    print(f"{len(pages)} pages checked, {len(problems)} with a drawing that did not draw")
    if problems:
        sys.exit("\n  " + "\n  ".join(problems))
    return 0


if __name__ == "__main__":
    sys.exit(main())
