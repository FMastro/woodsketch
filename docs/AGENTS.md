# Designing with WoodSketch (for AI assistants and scripts)

WoodSketch is a woodworking design app. You describe a piece of furniture as a **project recipe**
(JSON); WoodSketch builds a real drawing from it: boards cut from stock, real joinery cut into
them, a cut list with what to buy, cutting diagrams, build sheets for the shop and a wood movement
check. The person can then open the drawing in the WoodSketch app and keep working on it.

Two ways in:

- **MCP** (Claude and other agents): `woodsketch-cli mcp` serves tools over stdin/stdout. Call
  `guide` (this text), `build` with a recipe, check with `summary`, `movement` and `render`, then
  change it a piece at a time (`add_part`, `change_part`, `remove`, `add_joint`, `remove_joint`,
  `add_profile`; `recipe` shows the whole recipe now), and `save` (a `.woodsketch` file) and
  `cut_sheet` / `build_sheets` (HTML). `open` reads a drawing (`.woodsketch`, or a SketchUp `.skp`)
  back as a recipe, so a design made by hand can be changed the same way. The guide and the
  recipe's JSON Schema are resources (`woodsketch://guide`, `woodsketch://recipe-schema`), and the
  prompt `design` starts a project.
- **Command line**: `woodsketch-cli build recipe.json -o piece.woodsketch`, then `summary`,
  `cutlist`, `movement`, `render -o view.png --view iso`, `cutsheet -o cut.html`,
  `buildsheets -o build.html`, `export -o piece.obj`; `recipe piece.woodsketch` writes a drawing
  as a recipe, `schema` the recipe's JSON Schema. Run `woodsketch-cli help` for all of them.

## Coordinates and sizes

- Inches unless the recipe says `"units": "mm"`. A length is a number in those units, or text
  with its own: `"3/4\""`, `"19mm"`, `"1 1/2"`, `"2'6\""`.
- **Z is up.** X runs left to right, Y front to back (the front of a piece faces -Y).
- A **part is a box.** `at` is its lowest corner (smallest x, y and z). `size` is
  `[length, width, thickness]`; `length` names the axis the length (and the grain) runs along,
  `width` the axis the width runs along, and the thickness takes the third axis.
  - A shelf lying flat, 30" wide: `"size": [30, 11.25, 0.75], "length": "x", "width": "y"`.
  - A cabinet side standing up: `"size": [30, 11.25, 0.75], "length": "z", "width": "y"`.
  - A table leg: `"size": [29.25, 1.75, 1.75], "length": "z", "width": "x"`.
  - An apron along the front: `"size": [30, 4, 0.75], "length": "x", "width": "z"`.
- **Parts that are joined must touch** (a face against a face) or overlap. Place them exactly:
  a shelf between two sides starts where the left side ends.

## The recipe

```json
{
  "name": "Nightstand",
  "units": "in",
  "materials": [
    { "id": "cherry", "library": "Cherry" },
    { "id": "ply", "library": "Plywood" }
  ],
  "parts": [
    { "name": "Left side", "material": "cherry", "size": [24, 14, 0.75], "at": [0, 0, 0], "length": "z", "width": "y" },
    { "name": "Right side", "material": "cherry", "size": [24, 14, 0.75], "at": [17.25, 0, 0], "length": "z", "width": "y" },
    { "name": "Bottom", "material": "cherry", "size": [16.5, 14, 0.75], "at": [0.75, 0, 3], "length": "x", "width": "y" },
    { "name": "Shelf", "material": "cherry", "size": [16.5, 13.75, 0.75], "at": [0.75, 0, 12], "length": "x", "width": "y" },
    { "name": "Back", "material": "ply", "size": [21, 16.5, 0.25], "at": [0.75, 14, 3], "length": "z", "width": "x" },
    { "name": "Top", "material": "cherry", "size": [20, 16, 0.75], "at": [-1, -1, 24], "free_to_move": true }
  ],
  "joints": [
    { "kind": "let_in", "part": "Bottom", "into": "Left side", "options": { "depth": 0.25 } },
    { "kind": "let_in", "part": "Bottom", "into": "Right side" },
    { "kind": "let_in", "part": "Shelf", "into": "Left side" },
    { "kind": "let_in", "part": "Shelf", "into": "Right side" }
  ],
  "groups": [{ "name": "Nightstand", "parts": ["Left side", "Right side", "Bottom", "Shelf", "Back", "Top"] }]
}
```

### materials

Each has an `id` the parts use. From the library: `{ "id": "oak", "library": "Red Oak" }`.
The library: Red Oak, White Oak, Hard Maple, Cherry, Walnut, Poplar, Pine, Cedar (solid wood);
Plywood, Pine Plywood, Baltic Birch Plywood, MDF, Hardboard (sheets); Pine Lumber, Douglas Fir
Lumber, Cedar Lumber, Pressure-Treated Lumber (dimensional lumber: 2x4s and such). Your own:
`{ "id": "teak", "name": "Teak", "kind": "solid wood", "color": [150, 100, 60] }` (`kind` is
`solid wood`, `lumber`, `sheet`, `hardware` or `other`).

### parts

`name` (unique), `material`, `size`, `at`, `length`, `width`; optional:

- `tag` (made if new); `called` (the name shown, when two parts share one: names `Stile` and
  `Stile 2`, both `called` "Stile").
- `free_to_move`: true for a solid top held by tabletop buttons or slotted screws, so the wood
  movement check knows it can move.
- `grain`: `length` (the default) or `width`; `cut`: `flat_sawn` or `quartersawn`.
- `turn`: `{ "axis": "y", "degrees": -45 }` turns it about that axis through `at` (its lowest
  corner before turning): a brace, a splayed leg. Positive is counter-clockwise looking down the axis.
- `reference`: `{ "face": "front", "edge": "bottom", "end": "left" }`, which way its reference
  face, edge and end face once placed (`top`, `bottom`, `front`, `back`, `left`, `right`); the
  build sheets measure from them.
- `copies`: `[{ "name": "Leg 2", "at": [20, 0, 0], "turn": 90 }]` places more of the same part
  (a component: a joint or profile made on one is on all), each turned about z, then put at
  its lowest corner. Joints name each copy.

### profiles

`{ "part": "Top", "profile": "roundover", "size": 0.25, "around": "top" }` rounds over every edge
of the face that faces up; `"along": "length"` instead profiles the four edges running the length
(a chamfered leg). Profiles are `roundover`, `chamfer`, `cove` and `ogee`, made before the joints.

### joints

`{ "kind": ..., "part": ..., "into": ..., "options": { ... } }`. Sizes left out are suggested from
the parts, as the app's Joinery panel does.

| kind | part / into | options |
|---|---|---|
| `let_in` (dado, groove or rabbet) | the part let in / the part cut | `depth` (it grows by this: size it to butt) |
| `cut_overlap` | the part kept whole / the part cut where they overlap | |
| `mortise_and_tenon` | the rail (tenon) / the leg or stile (mortise) | `length`, `thickness`, `shoulder` |
| `dowels` | either / either | `count`, `diameter`, `depth` |
| `biscuits`, `loose_tenons` | either / either | `count` |
| `splines` | either / either | |
| `pocket_screws` | the part drilled / the part screwed into | `count`, `screw` (length) |
| `dovetails` | the tails board (a drawer side) / the pins board (the front) | `tails`, `half_blind` |
| `box_joint` | either / either (ends meeting at a corner) | `finger` |
| `half_lap` | the top board / the bottom board (crossing) | |
| `angled_mortise_and_tenon` | the turned part (a brace) / the part it meets | `thickness`, `shoulder` |
| `housing` | the turned part / the part it's let into | |

Butt the parts as they'd sit without the joint: a shelf ends at the side's face, a rail at the
leg's face; WoodSketch grows the shelf into the dado and the rail's tenon into the leg.

### templates

Ready-made assemblies, with only the sizes you change:
`{ "kind": "table", "name": "Hall table", "at": [0, 0, 0], "wood": "oak", "sizes": { "width": 48, "depth": 16, "height": 30 } }`.

- `table`: `width`, `depth`, `height`, `leg`, `apron`, `top`, `joints` (`MortiseAndTenon`,
  `PocketScrews`, `Dowels`).
- `cabinet`: `width`, `height`, `depth`, `shelves`, `back` (true/false), `stock`, `adjustable`,
  `joints` (`Dados`, `Biscuits`, `PocketScrews`, `Butt`), `fills` (each opening from the bottom:
  `Open`, `Drawer`, `Doors`, e.g. `["Drawer", "Doors"]`), `fronts` (`Overlay`, `Inset`), `doors`
  (`Slab`, `FrameAndPanel`), `pulls` (`Bar`, `Knob`, `None`), `drawer_joints` (`Dovetails`,
  `BoxJoints`, `Rabbets`), `hinge` (`Left`, `Right`).
- `drawer`: `width`, `height`, `depth`, `joints` (drawer joints), `stock`.
- `face_frame`: `width`, `height`, `stile`, `rail`, `stock`, `middle_rails`, `joints`.

### groups

`{ "name": "Case", "parts": ["Left side", "Right side"], "component": false }`, made last (joints
are made between parts side by side first). A group's parts can be templates' names too.

## Checking your work

- `build` lists every problem, each naming the item: `joints[2] (mortise_and_tenon, Apron into
  Leg): Apron doesn't touch Leg (move one so they meet...)`. Fix those items and build again.
- `summary` gives the overall size, every part with its finished size and the joints. Check the
  overall size is what was asked for and the parts make sense.
- `movement`: solid wood moves across its grain with the seasons (a 16" cherry top about 3/16").
  A warning means a part is held where it needs to move: let a panel float in grooves (`let_in`
  into a frame's grooves), mark a top `free_to_move` (fastened with buttons), or turn a part.
- `render` shows a picture (`iso`, `top`, `front`, `right`, or any `angle`: `[around, up]` in
  degrees), light or `dark`, with parts picked out by name (`highlight`), to see that it looks right.
- `save` the drawing for the person to open in WoodSketch; `cut_sheet` and `build_sheets` write
  what they take to the shop.

## Setting it up

- WoodSketch installs `woodsketch-cli` beside the app: on Windows
  `%LocalAppData%\WoodSketch\current\woodsketch-cli.exe`; on a Mac inside `WoodSketch.app/Contents/MacOS/`;
  on Linux next to the AppImage's app (or build it: `cargo build --release -p woodsketch-cli`).
- Without the app (a server, CI): each release also has `woodsketch-cli-<version>-<platform>.zip`
  (`win-x64`, `osx-arm64`, `osx-x64`, `linux-x64`) with just the command line, on GitHub
  (github.com/FMastro/woodsketch/releases). Rendering pictures needs a graphics adapter.
- Claude Code: `claude mcp add woodsketch -- "<path to woodsketch-cli>" mcp`.
- Claude Desktop and other MCP clients: a stdio server with the command `<path to woodsketch-cli>`
  and the argument `mcp`.
