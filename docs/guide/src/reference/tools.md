# Tools and keys

<!-- Written by crates/ui-egui/tests/guide.rs from the app itself. Don't edit it by hand:
     change the app and run `cargo test -p woodsketch-ui-egui --test guide`, which rewrites it. -->

Every tool, its key and what it does. The keys are SketchUp's, so they should already be in
your fingers; the ones with no key are reached from the toolbar or the Tools menu.

**You never have to click into a box to type a number.** Start typing and it goes to the
Measurements box at the bottom of the window: `Enter` commits it, `Esc` clears it. Type
*after* finishing a shape to correct it. A comma is a decimal point (`38,5`), so a list of
lengths is separated by a semicolon (`5,5';6'`). `Space` is Select and `Esc` cancels whatever
is in hand, which between them get you out of anything.

## Principal

| | Tool | Key | What it does |
|---|---|---|---|
| <span class="ws-icon ws-icon-cursor"></span> | **Select** | `Space` | Pick things. Click, shift-click to add or remove, drag a box. Double-click a group to edit inside it. |
| <span class="ws-icon ws-icon-eraser"></span> | **Eraser** | `E` | Erase the edge, guide, group or component you click, or everything you drag over, as one undo step. |
| <span class="ws-icon ws-icon-paint-bucket"></span> | **Paint Bucket** | `B` | Paint a material onto a whole object; Ctrl paints one face, Alt samples the one under the cursor. |

## Drawing

| | Tool | Key | What it does |
|---|---|---|---|
| <span class="ws-icon ws-icon-line-segment"></span> | **Line** | `L` | Draw edges point to point. A closed loop makes a face; a line across a face splits it. Type each length. |
| <span class="ws-icon ws-icon-bezier-curve"></span> | **Arc** | `A` | An arc through two endpoints and a bulge. Type `12s` for the number of segments, `4r` for a radius. |
| <span class="ws-icon ws-icon-circle-half"></span> | **Center Arc** | -- | An arc from its center: the center, then the start, then the sweep. |
| <span class="ws-icon ws-icon-arrow-arc-right"></span> | **3-Point Arc** | -- | An arc through three points you click. |
| <span class="ws-icon ws-icon-chart-pie-slice"></span> | **Pie** | -- | A filled wedge: center, radius, then the angle. |

## Shapes

| | Tool | Key | What it does |
|---|---|---|---|
| <span class="ws-icon ws-icon-rectangle"></span> | **Rectangle** | `R` | A rectangle on a face or in free space. Type `36x11 1/4` to size it, before or after drawing. |
| <span class="ws-icon ws-icon-circle"></span> | **Circle** | `C` | A circle, drawn as edges so it splits the face under it. Type `24s` to change the number of sides. |
| <span class="ws-icon ws-icon-hexagon"></span> | **Polygon** | -- | A regular polygon from its center. Type the number of sides. |
| <span class="ws-icon ws-icon-diamond"></span> | **Rotated Rectangle** | -- | A rectangle turned off the axes: two corners set the baseline, the third the width. |
| <span class="ws-icon ws-icon-log"></span> | **Board** | `W` | Draw in real stock: pick a material and one of its standard sizes in the Board panel, then drag or type the size. Lumber drags out its length; sheets and solid wood drag a rectangle at their thickness. |

## Modifying

| | Tool | Key | What it does |
|---|---|---|---|
| <span class="ws-icon ws-icon-arrow-fat-lines-up"></span> | **Push/Pull** | `P` | Push or pull a face. Makes a solid from a free face, cuts a rabbet or notch where a wall is flush, and cuts through when pushed to the far face. Double-click repeats the last distance. |
| <span class="ws-icon ws-icon-arrows-out-cardinal"></span> | **Move** | `M` | Move or copy. Locks to an axis as you push along one, or with the arrow keys. Ctrl-tap to copy; `3x` makes an array, `/3` divides the move. Type a distance, or `x,y,z`. |
| <span class="ws-icon ws-icon-arrow-clockwise"></span> | **Rotate** | `Q` | Turn things about a point: the protractor sets the axis, snapping at 15 degrees and onto red, green or blue. Type the angle; Ctrl-tap to copy, `3x` for a rotational array. |
| <span class="ws-icon ws-icon-resize"></span> | **Scale** | `S` | Resize with 26 grips on the bounding box. The opposite grip stays put, or Ctrl holds the center. Type factors, or a finished size. |
| <span class="ws-icon ws-icon-frame-corners"></span> | **Offset** | `F` | Offset a face's outline in or out, mitering the corners -- an inward offset makes a frame and a panel. Type the distance. |

## Measuring and marking

| | Tool | Key | What it does |
|---|---|---|---|
| <span class="ws-icon ws-icon-ruler"></span> | **Tape Measure** | `T` | Measure, and leave a guide behind: a parallel guide line from an edge, a guide point from a point. Ctrl measures without leaving anything. |
| <span class="ws-icon ws-icon-angle"></span> | **Protractor** | -- | Measure an angle, and leave a guide line at it. |
| <span class="ws-icon ws-icon-arrows-horizontal"></span> | **Dimension** | -- | A dimension between two points, placed by moving out and clicking, or by typing how far out. |
| <span class="ws-icon ws-icon-text-t"></span> | **Text** | -- | A note on the drawing. It starts with something useful -- a part's name, an edge's length -- and typing replaces it. |

## Looking around

| | Tool | Key | What it does |
|---|---|---|---|
| <span class="ws-icon ws-icon-arrows-clockwise"></span> | **Orbit** | `O` | Turn the view around the drawing. |
| <span class="ws-icon ws-icon-hand"></span> | **Pan** | `H` | Slide the view. |
| <span class="ws-icon ws-icon-magnifying-glass"></span> | **Zoom** | `Z` | Zoom in and out. Shift+Z zooms to fit everything. |
| <span class="ws-icon ws-icon-person-simple-walk"></span> | **Walk** | -- | Walk through the drawing like a game: the mouse looks, W A S D or the arrows walk, Shift is faster, E and Q go up and down, F flies, the wheel sets the speed. Type a length to change eye height. |

## Keys that aren't tools

| Key | What it does |
|---|---|
| `Space` | Select |
| `Shift+Z` | Zoom to fit everything |
| `G` | Group the selection |
| `Ctrl+G` | Make a component of the selection |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Copy, cut, paste -- a paste is carried on the cursor until you click |
| `Ctrl+D` | Duplicate |
| `Ctrl+Z` / `Ctrl+Y` | Undo, redo |
| `Ctrl+N` / `Ctrl+O` / `Ctrl+S` | New, open, save (`Ctrl+Shift+S` saves as) |
| `Delete` | Erase the selection |
| Arrow keys | Lock the tool in hand to red, green or blue |
| `Esc` | Clear what you typed, cancel what's in hand, or deselect |
| Double-click | Edit inside a group; with Push/Pull, repeat the last distance |
