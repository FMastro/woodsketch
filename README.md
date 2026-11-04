# WoodSketch

**Design it the way you already know.** WoodSketch is a free, open-source 3D modeler for
woodworkers. It works the way SketchUp does, but the parts know they're wood: species, grain,
real stock sizes, joinery, cut lists and build sheets.

**Website and downloads: [woodsketch.net](https://woodsketch.net)**

![Cutting diagrams and a shopping list](https://woodsketch.net/images/cutting-diagrams-shopping-list.webp)

## Download

Installers for Windows, macOS (Apple silicon and Intel) and Linux are
on [woodsketch.net](https://woodsketch.net/#get-started) and in
[GitHub Releases](https://github.com/FMastro/woodsketch/releases/latest). Installed copies check
for updates when they start and install them when you restart.

The Windows installer is signed, though SmartScreen may still warn until the signature earns a
reputation: click *More info*, then *Run anyway*. The macOS installer isn't signed yet, so
right-click it and choose *Open*.

Would rather not run someone else's binary? [Build it yourself](#build-it-yourself) — it's three
commands once Rust is installed.

## What it does

- **Feels like SketchUp.** Select, Line, Rectangle, Circle, Arc, Push/Pull, Offset, Move,
  Rotate, Scale, Tape Measure, Eraser and Paint Bucket, with the same shortcuts. Type sizes
  into the Measurements box as you go (`3/4"`, `2'6`, `38mm`). Inference and snapping work the
  same way.
- **Groups, components and tags**, an Outliner, copy and paste, and arrays (`3x`, `/3`).
- **Boards from real stock.** Pick a material and a standard size (a 2x4, 3/4" plywood, 4/4
  cherry) and drag it out.
- **Templates.** A cabinet, drawer, face frame or table from a few sizes, with its joinery done.
- **Joinery.** Dados and rabbets, mortise and tenon (haunched, through), dowels, biscuits,
  loose tenons, pocket screws, half-laps, box joints, dovetails and splines, cut into the parts
  and listed with them.
- **Edge profiles.** Roundover, chamfer, cove and ogee.
- **Cut list and cutting diagrams.** Parts grouped by material and size, packed onto sheets
  and boards with kerf and grain direction, a shopping list, and glue-ups for wide panels.
- **Build sheets.** One printable page per part with views to scale and the steps in shop
  order.
- **Wood movement checks.** Warns where a part will swell or shrink across its grain against
  a part that won't move with it.
- **Works with your AI assistant.** `woodsketch-cli` (installed with the app) builds a drawing
  from a project recipe (JSON) and writes the cut list, cut sheet, build sheets, movement check
  and pictures; `woodsketch-cli mcp` is an MCP server for Claude and other agents. See
  [docs/AGENTS.md](docs/AGENTS.md).
- **Save, autosave and export.** `.woodsketch` files, crash recovery, OBJ and STL export.

![A part showing its grain direction](https://woodsketch.net/images/part-grain-direction.webp)

## Build it yourself

You don't have to download an installer. WoodSketch is written in Rust (egui and wgpu), and
building it from the code gives you exactly the same program.

### What you need

- **Rust 1.95 or newer.** Install it from [rustup.rs](https://rustup.rs), which is the usual way
  on every platform and sets up `cargo` for you. If you already have Rust, `rustup update` will
  do. (The code uses the 2024 edition, so an older Rust won't compile it.)
- **Git**, to fetch the code — or download the ZIP from the repository's green *Code* button.
- **A C/C++ toolchain**, which some dependencies need to build:
  - **Windows:** the [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
    with the *Desktop development with C++* workload. The rustup installer offers to get these
    for you if they're missing.
  - **macOS:** the Xcode command line tools — run `xcode-select --install`.
  - **Linux:** a compiler plus the windowing and file-dialog libraries. On Ubuntu or Debian:
    ```sh
    sudo apt install build-essential libgtk-3-dev libxkbcommon-dev libwayland-dev libxcursor-dev libxrandr-dev libxi-dev libx11-xcb-dev
    ```
    These are exactly what CI installs, so if the list ever goes stale, `.github/workflows/ci.yml`
    is the one that's right.
- **A GPU with Vulkan, Metal, DirectX 12 or OpenGL.** The viewport is drawn on the graphics card.
  Anything that runs SketchUp will do.

### Build and run it

```sh
git clone https://github.com/FMastro/woodsketch
cd woodsketch
cargo run --release -p woodsketch
```

That's the whole thing. The first build compiles about 500 crates, including the entire graphics
stack, so put the kettle on; everything after that is quick, because only what you changed is
rebuilt. Leave out `--release` for a faster build of a slower program — handy if you're changing
the code, not if you're using it.

To build without running, and to get the command line tool at the same time:

```sh
cargo build --release -p woodsketch -p woodsketch-cli
```

Both programs land in `target/release/` as `woodsketch` and `woodsketch-cli` (with `.exe` on
Windows). They're self-contained, so you can copy them wherever you like.

### Good to know

- **A copy you built doesn't update itself.** The updater only works in installed copies, and a
  self-built one simply reports that there's nothing to update. To move to a newer version, run
  `git pull` and build again.
- **Your drawings are unaffected either way.** `.woodsketch` files from an installed copy open in
  a self-built one and the other way round.
- Check your build with `cargo test --workspace` (the UI tests want a working GPU) and
  `cargo clippy --workspace --all-targets`.
- `CLAUDE.md` describes how the code is laid out, and `docs/BACKLOG.md` lists what's planned.

## Feedback

WoodSketch is new, and what gets built next depends on what people tell me they need. If a joint
you use isn't there, if the cut list doesn't match how your shop works, or if something simply
broke, I'd like to hear it.

- **In the app:** *Help > Report a Problem* and *Help > Send Feedback*. These need no GitHub
  account, and you can leave an email if you'd like a reply.
- **On GitHub:** [open an issue](https://github.com/FMastro/woodsketch/issues/new/choose) if you
  have an account.
- **Questions, ideas and what you've built:**
  [Discussions](https://github.com/FMastro/woodsketch/discussions) — ask how to do something, say
  what the app should handle next, or show a piece you've made.

Rough edges and half-finished corners are expected at this stage, so don't assume a problem is
already known. Nothing is too small to mention.

## Independent of SketchUp

WoodSketch reimplements SketchUp's *behavior* from its public documentation and from using the
app. It is a clean-room implementation: none of SketchUp's code, SDK or artwork was decompiled,
disassembled, read or copied, and none of it is present here. The `.skp` reader was worked out by
examining `.skp` files alone, without Trimble's SDK (see the notes at the top of
`crates/doc/src/skp.rs`). WoodSketch isn't affiliated with, endorsed by or connected to Trimble or
SketchUp; SketchUp is a trademark of Trimble Inc.

## License

See [LICENSE](LICENSE) (MIT or Apache-2.0, your choice). The open-source packages WoodSketch is
built with, and their licenses, are in
[THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md), which ships beside the app.
