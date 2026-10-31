#!/usr/bin/env python3
"""Drive `woodsketch-cli mcp` over stdio and check it behaves, by hand.

The MCP server has unit tests (`cargo test -p woodsketch-cli`), but those drive it in-process.
This talks to the real program the way an assistant does — one JSON-RPC message per line on its
stdin, answers on its stdout — so it catches anything that only goes wrong across the pipe, such
as something printing to stdout and corrupting the protocol.

What it walks through, in the order it matters:

  build -> save -> edit (saved by itself) -> the file changed behind us (refused)
        -> open (take those changes) -> edit again

    python -I scripts/try-mcp.py                      # finds the built CLI itself
    python -I scripts/try-mcp.py --cli path/to/woodsketch-cli.exe
    python -I scripts/try-mcp.py --keep               # leave the drawing to open in the app

`-I` keeps Python from loading anything out of the working directory. Exits non-zero on the
first thing that doesn't hold, so it can run in a shell loop or a pre-release check.
"""

import argparse
import json
import os
import platform
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def find_cli():
    """The CLI to talk to: one built here, else the one installed beside the app."""
    exe = "woodsketch-cli.exe" if platform.system() == "Windows" else "woodsketch-cli"
    guesses = [
        os.path.join(HERE, "target", "release", exe),
        os.path.join(HERE, "target", "debug", exe),
    ]
    if platform.system() == "Windows":
        local = os.environ.get("LOCALAPPDATA", "")
        guesses.append(os.path.join(local, "WoodSketch", "current", exe))
    elif platform.system() == "Darwin":
        guesses.append("/Applications/WoodSketch.app/Contents/MacOS/" + exe)
    for guess in guesses:
        if os.path.isfile(guess):
            return guess
    sys.exit(
        "Couldn't find woodsketch-cli. Build it with `cargo build --release -p woodsketch-cli`, "
        "or point at one with --cli."
    )


class Server:
    """The MCP server, one request at a time."""

    def __init__(self, cli):
        self.process = subprocess.Popen(
            [cli, "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            text=True, encoding="utf-8", bufsize=1,
        )
        self.id = 0
        self.stray = []

    def send(self, method, params=None):
        """One request in, its answer out. Anything that isn't JSON-RPC is kept for the check
        at the end: this program's stdout is the protocol, so nothing else belongs there."""
        self.id += 1
        request = {"jsonrpc": "2.0", "id": self.id, "method": method, "params": params or {}}
        self.process.stdin.write(json.dumps(request) + "\n")
        self.process.stdin.flush()
        while True:
            line = self.process.stdout.readline()
            if not line:
                raise SystemExit("the server stopped answering")
            try:
                message = json.loads(line)
            except json.JSONDecodeError:
                self.stray.append(line.rstrip())
                continue
            if message.get("jsonrpc") != "2.0":
                self.stray.append(line.rstrip())
            elif message.get("id") == self.id:
                return message

    def call(self, tool, **arguments):
        """A tool call: whether it refused, and what it said."""
        answer = self.send("tools/call", {"name": tool, "arguments": arguments})
        result = answer.get("result", {})
        said = "".join(part.get("text", "") for part in result.get("content", []))
        return result.get("isError", False), said

    def close(self):
        self.process.stdin.close()
        self.process.wait(timeout=10)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", help="the woodsketch-cli to run (found automatically otherwise)")
    parser.add_argument("--keep", action="store_true", help="leave the drawing behind, to open in the app")
    args = parser.parse_args()

    cli = args.cli or find_cli()
    work = tempfile.mkdtemp(prefix="woodsketch-try-mcp-")
    drawing = os.path.join(work, "bench.woodsketch")
    print(f"Using {cli}\nWorking in {work}\n")

    failures = []

    def check(label, condition, detail=""):
        print(f"{'ok  ' if condition else 'FAIL'}  {label}")
        if not condition:
            failures.append(f"{label}{': ' + detail.strip()[:300] if detail else ''}")

    server = Server(cli)
    try:
        hello = server.send("initialize", {"protocolVersion": "2025-06-18"})
        check("the server introduces itself", "result" in hello, json.dumps(hello))

        recipe = {
            "materials": [{"id": "oak", "library": "Red Oak"}],
            "parts": [{"name": "Top", "material": "oak", "size": [48, 20, 1.5]}],
        }
        errored, said = server.call("build", recipe=recipe)
        check("a recipe builds", not errored, said)

        check("nothing is written before a file is named", not os.path.exists(drawing))

        errored, said = server.call("save", path=drawing)
        check("save names the file", not errored and os.path.exists(drawing), said)
        check("and says edits will follow it", "as they're made" in said, said)
        size = os.path.getsize(drawing)

        leg = {"name": "Leg", "material": "oak", "size": [28, 3, 3], "length": "z"}
        errored, said = server.call("add_part", part=leg)
        check("an edit works", not errored, said)
        check("and reaches the file on its own", os.path.getsize(drawing) != size, said)

        # An edit that doesn't build must leave both the drawing and the file alone.
        before = open(drawing, encoding="utf-8").read()
        errored, said = server.call("add_joint", joint={"kind": "dado", "part": "Leg", "into": "Nothing"})
        check("an edit that doesn't build is refused", errored, said)
        check("and writes nothing", open(drawing, encoding="utf-8").read() == before, said)

        # Stand in for someone moving a part in WoodSketch and saving.
        print("\n   (editing the file behind its back, as the app would)\n")
        with open(drawing, "w", encoding="utf-8") as f:
            f.write(before.replace('"name": "Top"', '"name": "Top (moved by hand)"', 1))
        theirs = open(drawing, encoding="utf-8").read()

        shelf = {"name": "Shelf", "material": "oak", "size": [40, 18, 0.75]}
        errored, said = server.call("add_part", part=shelf)
        check("an edit won't write over that", errored, said)
        check("and says how to go on", "`open`" in said and "`save`" in said, said)
        check("their work is untouched", open(drawing, encoding="utf-8").read() == theirs)

        errored, said = server.call("open", path=drawing)
        check("open takes their changes", not errored and "moved by hand" in said, said)

        # Reading a drawing back names materials after themselves, so the id has changed.
        _, text = server.call("recipe")
        material = json.loads(text)["materials"][0]["id"]
        errored, said = server.call("add_part", part={**shelf, "material": material})
        check(f"editing works again (material is now {material!r})", not errored, said)

        # A recipe or a SketchUp model is somewhere to read, never somewhere to write.
        source = os.path.join(work, "bench.json")
        with open(source, "w", encoding="utf-8") as f:
            json.dump(recipe, f)
        untouched = open(source, encoding="utf-8").read()
        errored, said = server.call("open", path=source)
        check("a recipe opens", not errored, said)
        check("and isn't a place to write", "save" in said.lower(), said)
        _, text = server.call("recipe")
        errored, said = server.call(
            "add_part", part={"name": "Rail", "material": json.loads(text)["materials"][0]["id"], "size": [40, 3, 1]}
        )
        check("editing after it works", not errored, said)
        check("and the recipe file is untouched", open(source, encoding="utf-8").read() == untouched)
    finally:
        server.close()

    check("nothing but JSON-RPC came back on stdout", not server.stray, " | ".join(server.stray))

    print()
    if failures:
        print(f"{len(failures)} problem(s):")
        for failure in failures:
            print(f"  - {failure}")
        sys.exit(1)
    print("All good.")
    if args.keep:
        print(f"\nThe drawing is at {drawing}\nOpen it in WoodSketch; edits from a live session show up there.")
    else:
        import shutil

        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    main()
