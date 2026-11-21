# Coming from SketchUp

WoodSketch is built to be familiar on purpose. If SketchUp is in your fingers, most of it will
transfer without you thinking about it. This page is the short list of what's the same, what's
different and what isn't there.

It is worth reading once. The differences are few, but two of them will trip you up on the
first afternoon if nobody tells you.

## The same

- **The tools and their keys.** `L` line, `R` rectangle, `C` circle, `A` arc, `P` push/pull,
  `M` move, `Q` rotate, `S` scale, `F` offset, `T` tape measure, `E` eraser, `B` paint bucket,
  `O` orbit, `H` pan, `Z` zoom, Space to select. The full table is in
  [Tools and keys](reference/tools.md).
- **Axes and units.** Z up, red = X, green = Y, blue = Z.
- **Just type.** No clicking into a field: start typing and it goes to the Measurements box.
  `Enter` commits, `Esc` clears. Type after you finish a shape to correct it.
- **Inference.** Endpoints, midpoints, on-edge, intersections, axis locks with the arrow keys,
  "from point" alignment.
- **Groups and components.** `G` groups, `Ctrl+G` makes a component. Double-click to edit
  inside, `Esc` or a click outside to come back out. The Outliner shows the whole tree.
- **Entity Info** and the tray of panels down the right-hand side.
- **Push/Pull**, including double-click to repeat the last distance.
- **Orbit, pan, zoom** on the same mouse buttons and the wheel. `Shift+Z` zooms to extents.

## Different

**Tags go on groups and components only.** In SketchUp you *can* put a layer on raw geometry,
and every SketchUp guide published in the last decade tells you not to. WoodSketch simply
doesn't let you, so the advice is the behaviour. Hiding a tag drops its instances out of the
drawing entirely — not drawn, not picked, not snapped to, not in the cut list.

**A comma is a decimal point.** Half the world writes `38,5`, so `38,5` is thirty-eight and a
half. Which means a comma can't quietly be a separator: to type a list of lengths, use a
semicolon — `5,5';6'` is five and a half feet by six feet. Type a semicolon anywhere in the
entry and every comma in it is a decimal point.

**Units belong to the drawing.** Not to the program and not to a template. A drawing made in
millimetres opens in millimetres on anyone's machine. `View > Units` changes it.

**Parts know they're wood.** This is the real difference and it's why the app exists. A group
can have a species, a grain direction, a sawing (flat-sawn or quartersawn), a reference face
and edge, stock it is cut from, joints to its neighbours and an edge profile. From that you get
a cut list, cutting diagrams, a shopping list, build sheets and a wood-movement check. There is
no equivalent in SketchUp without an extension.

**The Board tool (`W`) has no SketchUp counterpart.** You pick real stock — Red Oak, 3/4" — and
draw a board in it, rather than drawing a box and telling it what it is afterwards.

**Joints are made, not drawn.** Select two parts and the Joinery panel offers the joints that
suit how they sit, then cuts both parts. The joint is remembered, so it can be changed or
removed and the parts rebuilt without it. See [Joints](reference/joints.md).

**Scale behaves itself on a single group.** Scaling one group scales along its own axes into its
transform, so the cut list still measures the part correctly through it.

**Dynamic components are not imported.** WoodSketch has its own
[parametric components](reference/templates.md) — a group that remembers the recipe it was
built from, so changing a number rebuilds the piece rather than stretching it. SketchUp's
dynamic-component formulas speak to SketchUp's attribute model and are deliberately left alone.

## Not there

Some of this will come; some of it is out of scope for a woodworking tool. Either way, don't go
looking:

- **Follow Me**, **Solid Tools**, **Sandbox** and terrain.
- **Styles**, **Scenes** and animation, **section planes**.
- **LayOut**. Build sheets and the cut sheet are the shop-drawing answer here, and they are
  generated rather than drawn.
- **Extensions / the Ruby API.** There is instead a command line and an
  [MCP server](https://github.com/FMastro/woodsketch/blob/main/docs/AGENTS.md), so an AI
  assistant can build and edit drawings and the app follows along live.
- **Rendering.** The viewport is the view; there is no photorealistic output.

## Bringing your models over

**File > Open** reads a `.skp` saved by SketchUp 2021 or later, as a new unsaved drawing.
**File > Import SketchUp Model...** pastes one into the drawing you already have.

What comes across: component definitions and instances, top-level geometry, faces with holes,
materials (matched to WoodSketch's library by name), layers as tags and their folders, hidden
layers, hidden and softened edges, guides, construction lines, dimensions and text.

What doesn't: anything that was only ever a SketchUp concept — styles, scenes, dynamic-component
formulas. Materials whose names don't say what they are come in as "Other", which keeps them in
the parts list but out of the cutting diagrams; set the species yourself in the Materials panel
and they join in.

Lumber-sized parts are recognised as lumber on the way in, so a 2x4 wall arrives as 2x4s.
