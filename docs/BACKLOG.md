# Backlog

Things we deliberately skipped or know are incomplete, to come back to. Grouped by category, the
categories in the recommended order, and the items in each in the order to do them. Remove items
when they're done (mention them in the commit); add what a feature leaves out.

Each item has a relative effort in points (modified Fibonacci: 1, 2, 3, 5, 8, 13, 21, 40, 100).
A 1 is a quick tweak; a 3 is about a focused hour or two; an 8 is a feature-sized change like a
new joint; a 21 is a milestone like the SketchUp reader; 40 and 100 are projects of their own.

## Summary (recommended order)
| # | Category | Points | Why here |
|---|---|---|---|
| 1 | Build sheets and assembly | 3 | Finishes what just shipped; shop output is what woodworkers keep |
| 2 | Joinery | 92 | Closes gaps in the core woodworking feature |
| 3 | Templates | 139 | Fast wins that make templates usable as-is |
| 4 | Drawers, doors and hardware | 84 | Rounds out cabinets |
| 5 | Agent-ready | 11 | Shapes beyond boxes, the last read-back gaps |
| 6 | Release polish | 79 | Signing before announcing widely |
| 7 | Importing | 222 | Gets people's existing models in |
| 8 | Cut list and materials | 171 | Better buying and cutting |
| 9 | Wood movement | 23 | Fewer false warnings, clearer results |
| 10 | Edge profiles | 76 | More bits, profiles on any board |
| 11 | Parts and boards | 11 | Small drawing conveniences |
| 12 | Modeling tools | 108 | SketchUp parity for general modeling |
| 13 | Groups, components and tags | 63 | Organizing bigger projects |
| 14 | UI | 87 | Comfort and polish |
| 15 | Rendering and performance | 66 | Only pressing for very big models |
| 16 | Export | 48 | More ways out (CNC, other apps) |
| 17 | Later | 100 | Separate projects |
| | **All** | **1383** | |

## 1. Build sheets and assembly
Done: a page per part (size, stock, three dimensioned views, steps in shop order); assembly pages
(exploded drawing with leader lines and glue-up order per assembly, sub-assemblies first, hardware
at the step it's used); hidden lines dashed in the part views; each cut once with how many; alike
and mirrored parts on one page; glue-up boards drawn; reference face and edge picked on the model
(Entity Info > Pick) and each cut placed from them on the sheet; File > Print Build Sheets. Profiles
drawn and named on the views; each cut's place, length and depth dimensioned; a reference end;
the face mark's loop and the edge mark's caret on the views and the model; leaders that keep clear
of other parts; hardware on two parts names both; parts alike but drawn differently share a
sheet; which copies take which part. A glue-up order for parts with no joints
made (taken as butt joints); a picture for each step; parts at an angle touch only where their faces
meet. Step pictures pull each part out the way it goes in; butt joints say where the
screws go; leaders bend around parts in their way; past four places, a table of where the cuts are.
1. [3] Step pictures slide a part out along red, green or blue only (by the parts' boxes); a part
   at an angle comes straight out from what's together.

## 2. Joinery
Done: dado, groove, rabbet, mortise and tenon (through, haunched), dovetails (through,
half-blind), dowels, biscuits, loose tenons, splines, pocket screws, box joint, half-lap, cut
where parts overlap; angled mortise and tenon and housings; Change and Remove; fit allowances on
mortises and dowel holes.
1. [3] Fit allowance on slots, haunched tenons and angled joints (angled tenons go as deep as
   the overlap drawn).
2. [5] Variable dovetail spacing.
3. [5] An angled joint on one copy of a shared component makes it its own component (the
   others stay); square joints change every copy. Offer to change all of them.
4. [8] Removing a joint after a part was changed by hand (refused now; use Undo).
5. [8] Pocket screws: draw the angled bore, not only the pocket on the hidden face.
6. [13] Sliding and lapped dovetails.
7. [13] Joining parts in different groups (both must be side by side now).
8. [13] Angled dowels, half-laps and dovetails.
9. [21] Compound angles, and a second angled joint on a part already cut at an angle in
    another plane.
10. [3] Entity Info lists a part's joints once per copy's meeting ("Dovetails with Side" twice on
    a drawer front made as copies); list each cut once, as the build sheets do
    (`buildsheet::cuts`), and say which corners it's at.

## 3. Templates
Done: cabinet (drawers and doors in its openings, adjustable shelves, hinge side), drawer, face
frame (middle rails), table (leg, apron and top sizes); wood and stock choices; Change size.
1. [5] A new template goes where you click, not a foot to the right of everything.
2. [3] Drill the rows of shelf-pin holes in the sides for adjustable shelves.
3. [3] Drawer: notch the bottom's corners around the corner joints.
4. [5] Choose the still-fixed sizes: 1/4" backs, bottoms and door panels; 3/4" fronts and
   doors; 3/4" aprons; 1/2" drawer boxes.
5. [5] Cabinet: back set in a rabbet, not applied over the back edges.
6. [5] Drawer: an applied false front.
7. [3] Face frame: mullions (upright dividers).
8. [5] Table: set the tenon's size by hand (it follows the leg now).
9. [5] Cabinet: a toe kick.
10. [8] Cabinet: dividers.
11. [8] Cabinet: a face frame.
12. [8] Table: stretchers.
13. [8] Table: a drawer.
14. [13] A bookcase template.
15. [13] A workbench template.
16. [21] A bed template.
17. [21] Change size keeps edits made inside the group by hand (it rebuilds from scratch now; the
    button warns).

## 4. Drawers, doors and hardware
Done: drawers on side-mount slides, doors (slab or frame-and-panel, overlay or inset, concealed
hinges), pulls and knobs, shelf pins, all counted in the cut list's hardware.
1. [3] Pairs of doors: choose where they hinge (always at the outsides now).
2. [5] Place and size pulls by hand (5" bars or knobs, centered now; two over 24").
3. [5] Hardware prices.
4. [8] Hinges drawn properly: the cup bored in the door and a mounting plate; more hinge kinds.
5. [8] Undermount slides; choosing a slide and its clearances.
6. [8] Screws.
7. [8] Placing hardware on parts drawn by hand.
8. [13] Solid raised panels in frame-and-panel doors, sized for wood movement (1/4" plywood now).
9. [13] Drawers and doors in a hand-drawn cabinet's openings (click an opening).
10. [13] Face-frame cabinets (fronts set in a face frame).

## 5. Agent-ready
Done: project recipes (`doc::recipe`: materials, parts as boxes, templates, joints, tags, groups;
every problem named by item), `woodsketch-cli` (build, summary, cutlist, cutsheet, buildsheets,
movement, export, render to PNG headless) installed beside the app, an MCP server
(`woodsketch-cli mcp`), and the guide for agents (`docs/AGENTS.md`, llms.txt on the site). Recipes with copies,
turns, grain and cut, reference faces, profiles and angled joints; any drawing read back as a
recipe (`recipe_of`, `woodsketch-cli recipe`, MCP `open`) that builds it again; editing piece by
piece over MCP (`add_part`, `change_part`, `remove`, `add_joint`, `remove_joint`, `add_profile`,
`recipe`), each rebuilt and refused whole if it doesn't build; a JSON Schema (`schema`, and an MCP
resource with the guide); a `design` prompt and progress notices; renders in the dark theme, from
any angle, with parts highlighted; the CLI alone as a zip per platform on each release. Every
joint keeps the sizes it was made with, wherever it was made.
1. [5] Parts that aren't plain boxes (angled ends, curves, cuts made by hand) read back as their
   box; a recipe part drawn as an outline and pulled to its thickness would keep them.
2. [2] Read back too: tag folders and hidden tags, parts turned about two axes, profiles on some
   of a face's edges, hardware placed by hand (each named in the notes now).
3. [2] Highlight parts inside groups in renders (top-level parts and groups only now).
4. [2] Progress for long builds that says how far along it is (only started and done now).

## 6. Release polish
Done: installers, auto-update, icons, release notes, the site's download links; Help > Report a
Problem / Send Feedback (GitHub issue forms, details with nothing private; fixed issues close
when their release is published).
1. [5] Windows signing with Azure Artifact Signing (~$9.99/month; individuals in the US/Canada; a
   paid Azure subscription; identity validation takes days; CI signs via OIDC, no stored key).
   Until then SmartScreen warns. Do it before announcing WoodSketch widely.
2. [8] macOS signing with an Apple Developer ID ($99/year) plus notarization, or Gatekeeper blocks it.
3. [2] SHA-256 checksums (Linux needs no signing; optionally GPG-signed checksums).
4. [2] A Download button on woodsketch.net that picks the visitor's OS.
5. [3] Delta updates (download the previous release before packing, so updates are small).
6. [5] `.woodsketch` file association on all three (double-click opens it).
7. [3] A macOS DMG.
8. [5] Linux .deb and .rpm.
9. [5] Windows ARM64 and Linux ARM64 builds.
10. [5] After a crash, offer to report it at the next start: a panic hook saves the message and
    where it happened (paths scrubbed), and Report a Problem opens with it filled in.
11. [3] The OS version in a report's details (only the OS and processor now: Rust's standard library
    doesn't say which Windows or macOS version).
12. [2] A scheduled check of new reports (a daily routine that reads `from-app` issues and drafts
    fixes for review); now they're checked at the start of each session (CLAUDE.md).
13. [21] End-user help for every feature: a User Guide on woodsketch.net (/help), a page per
    feature or tool (what it's for, step by step, the keys and typed values, tips, what it doesn't
    do yet), written for woodworkers new to SketchUp as well as SketchUp users. Getting started
    (draw a board, join two parts, print a cut list), each drawing tool, groups and components,
    tags, materials and stock, the Board tool, joinery (each joint), edge profiles, templates,
    drawers and doors, cut list and cutting diagrams, build sheets, wood movement, reference
    face and edge, units, SketchUp import, export, updates and reporting a problem.
14. [5] Screenshots for the guide made by the UI tests (egui_kittest snapshots of each step), so
    they're redone when the app changes instead of going stale.
15. [3] Help > User Guide, and F1 opening the page for the tool or panel in use; a "?" link in
    each tray panel and window to its page.
16. [2] Keep it current: every feature ships with its help page updated (a rule in CLAUDE.md
    next to the CHANGELOG one), and the release notes link to the pages.

## 7. Importing
Done: SketchUp 2021+ `.skp` (geometry, components, groups, materials, tags and tag folders, hidden
tags, hidden objects on a "Hidden in SketchUp" tag, guide lines and construction lines, hidden and
softened edges, dimensions and text attached to geometry, lumber told from boards by the parts'
sizes); checked against 34 real files.
1. [2] Lumber told by size only moves parts painted as a whole; a part painted face by face stays
   with the boards (and 1x lumber, 3/4" thick, always reads as boards).
2. [3] Painted wood parts (a material named like `[Color_009]`) come in as Other; materials are
   sorted into wood, sheet goods and Other by name only, and their stock may need setting.
3. [3] A sample saved by SketchUp 2021 or 2022 (only 2023-2026 files tested so far).
4. [3] SketchUp smooth edges shade smooth (soft and hidden edges aren't drawn now; faces are shaded
   flat).
5. [5] SketchUp back-face materials and transparency.
6. [2] SketchUp guide points (not found in the sample files yet).
7. [8] OBJ import (named groups, an MTL of colors).
8. [21] COLLADA (.dae) import: groups, components, names and materials. Most 3D programs and
   SketchUp (including the free SketchUp Make 2017) export it, so it's also a way in for
   SketchUp 2020 and older.
9. [8] SketchUp scenes and cameras.
10. [3] SketchUp dimension and text details: custom dimension text, screen text (pinned to the view),
   angular and radial dimensions, and attachment types other than a vertex (2) or a point (12) are
   taken from their stored point.
11. [8] SketchUp images (`1390` > `1f40`).
12. [13] SketchUp textures (needs textured materials).
13. [40] SketchUp 2020 and older (a different format; refused with a message now: open and save
    them in SketchUp 2021+ first, or use DAE).
14. [100] Saving back to `.skp`.
15. [2] A hidden SketchUp object loses its own tag: it goes on "Hidden in SketchUp" instead (no
    per-object hiding here).
16. [1] A hidden (softened) edge split by a new line comes back as two drawn edges.

## 8. Cut list and materials
1. [2] Warn when a part is thinner than the thinnest stock (it comes from it, silently, now).
2. [3] Drawings saved before 5/4 and 6x lumber sizes keep their old size lists (add the sizes in
   the Materials panel, or re-add from the library); offer to update them.
3. [3] Length, width and thickness oversize allowances per material.
4. [5] Glue-ups: mix a wide and a narrow board (strips are equal now); set the 1/4" jointing
   allowance and 1" trim.
5. [8] Several sheet sizes per material to choose from (one now; US defaults).
6. [8] Labels or stickers per part.
7. [8] Cost estimates: price per board foot or sheet, density, a total.
8. [8] Use my leftover pieces first; offcut reuse across materials.
9. [8] Face-glued thick parts (a laminated leg); edge-glued panels only now.
10. [3] Paint Bucket: Shift replaces a material everywhere.
11. [8] Front and back faces painted separately (paint shows on both sides now).
12. [13] Random-width hardwood (one board width per material is assumed now).
13. [13] Offcuts inventory.
14. [13] Edge banding and veneer (material types and in the cut list).
15. [13] Material collections like SketchUp's: save materials into named collections, browse them
    across drawings, import and export them (the library is built in and fixed now).
16. [13] A PDF written directly (the cut sheet is a web page printed from the browser now).
17. [21] Textures: wood grain shown in the wood (color only now; grain shows as an arrow).
18. [21] Packing that's closer to optimal (heuristic now: several orders and rules, best kept).

## 9. Wood movement
Done: Wood Handbook coefficients per species and cut, a moisture swing setting, movement per part,
warnings where touching parts fight it (Cut List window); panels in grooves float and parts can be
"Fastened so it can move" (listed as allowed for); View > Wood Movement draws arrows on the parts;
"Moves like" a species for custom woods, and woods without data are named.
1. [5] Check a floating panel's grooves leave it room: groove depth against half its movement
   each side (a panel can float yet still bind or fall out of a shallow groove).
2. [3] "Fastened so it can move" is for the whole part; let each meeting choose (a top glued at
   its middle and buttoned at its ends, a breadboard end pinned at the middle only).
3. [2] Remember View > Wood Movement between runs.
4. [5] Movement in thickness and along the grain.
5. [8] Parts at an angle (only parts square to the axes are checked or get arrows).

## 10. Edge profiles
Done: roundover, chamfer, cove, ogee, around a face (mitered corners) or along a length.
1. [5] Bead and other bits.
2. [8] Click edges in the model to pick them (named by direction in the panel now).
3. [8] Edges around a face and along the length on one board.
4. [13] Change a profile later (it's listed, not yet parametric).
5. [21] Profile a board that already has joints or other cuts (profile first for now).
6. [21] Curved and angled edges.

## 11. Parts and boards
1. [3] Choose which side of the line a lumber board's width goes (always to the left of the drag).
2. [3] Pick stock straight from the Materials panel.
3. [5] Tab cycles a board's faces against a surface.

## 12. Modeling tools
Done: Polygon (Ctrl: to a side or a corner), Protractor (angled guide lines), Push/Pull with Ctrl
(keeps the face), Offset repeats on a double-click, `[x,y,z]` typed before the first click (Line,
Rectangle), Walk (mouse looks, W A S D walk), retyping a line's length, Rectangle's square and golden
hints, arrow keys facing a circle along an axis, new guides showing hidden guides. Dimensions
and Text tools (kept with the guides: saved, moved, copied, erased; drawn over the view). Notes
edited in Entity Info (double-click one), an edge dimensioned in one click, View > Dimensions and
View > Text, `[x,y,z]` starts for Move, Tape, Circle and Arc, Walk's eye height, E / Q, speed and walls.
Faces closed from loose lines face like the faces they meet, guide lines snap where they pass
through faces, Offset snaps its distance to points, Tape Measure resizes the model to a typed
length, Shift switches Scale between even and stretched, and Move hides rotation grips on faces
too small on screen. Off a face, Rotate and Protractor lie in the red, green or blue plane seen most
squarely (through the edge under the cursor). Center Arc, 3-Point Arc and Pie. Rectangle's far corner lines up with points passed over;
Arc's end infers like a line; Rotated Rectangle (any angle); sticky snapping (the last point holds
until another is clearly closer). Inference and snapping is finished: directions stick too, and a
circle or polygon's rim goes through a point passed over.

### Tools
1. [2] Notes: a text size, and leader-less text pinned to the screen (SketchUp's screen text).
2. [5] Walk: following the floor (steps up stairs, falls off ledges); it keeps its height now.

### Circles and arcs
3. [5] Arc: tangent inference.
4. [8] Soften/Smooth per edge, and smooth shading (soft edges are worked out when drawing now,
    facets are shaded flat).
5. [13] Curves as one entity (a circle is 24 edges now; SketchUp selects it with one click and
    shows its radius and sides).

### Offset, Tape Measure, Scale, Rotate
6. [3] Offset: rounded outward corners (always mitered now).
7. [5] Offset holes too (only a face's outer outline now).
8. [5] Offset a selection of connected edges (only faces now).
9. [1] Tape Measure resize: ask first, as SketchUp does (it's one undo step now).
10. [3] Scale grips as green boxes, hiding the ones facing away (red crosses now).
11. [8] Scale: mirroring (negative factors are refused).
12. [8] Scale: a rotated group among several scaled together gets sheared (they use the
    red/green/blue axes), and the cut list measures it only approximately.

### Geometry
13. [5] Collinear leftovers: other merges (pulling part of a top up flush with a side) can still
    leave a corner on a straight edge (erasing a seam heals them).
14. [8] Coplanar faces sharing an edge merge on their own (boards glued end to end keep a seam).
15. [8] Move folds a face that would bend (adds a diagonal, as SketchUp does); refused now.
16. [21] Auto-joining: moved or copied geometry welds to geometry it lands on (also pasted loose
    geometry). Fine while boards should stay separate.

## 13. Groups, components and tags
1. [1] Opening one copy of a shared group gives it its own definition; make that an undo step by itself.
2. [2] Hidden groups can be selected in the Outliner (listed, dimmed, now).
3. [3] Rename in the Outliner (Entity Info renames now).
4. [3] Selected groups drawn with a blue bounding box like SketchUp (tinted now).
5. [3] An active tag that new groups get automatically.
6. [5] Tag colors and "color by tag"; dashed tag lines.
7. [8] Drag to re-parent in the Outliner.
8. [8] Move stretching reaches into groups from outside (Move moves the whole group now).
9. [3] Copy and paste: guides are copied too.
10. [3] Paste: pick a grab point (carried by its box's lowest corner now).
11. [3] Duplicate repeats (always 2" to the right now; Move with Ctrl and `3x` does arrays).
12. [8] Copy to the system clipboard, so a paste reaches a second WoodSketch or another app (the
    clipboard only gets a note now).
13. [13] Component browser (copy and paste or Move with Ctrl place more instances now).

## 14. UI
Done: View > Theme (Light, Dark, Follow System), remembered; the viewport's sky and overlays follow it.
1. [2] Eraser: erase a face by itself (select it and press Delete now).
2. [3] Standard views: Back, Left, Bottom; a Parallel Projection toggle.
3. [3] Units: centimeters, decimal inches, feet and inches.
4. [3] Units saved with the drawing (an app preference now); joint descriptions follow the units
   (they keep the ones they were made in).
5. [3] Sky and ground colors of your own (light and dark presets now).
6. [5] Eraser: Shift hides, Ctrl softens/smooths.
7. [5] Right-click menu: Hide, Entity Info, Intersect Faces and per-tool menus.
8. [5] Per-tool cursors (the system arrow now).
9. [5] File dialogs that don't block the window (rfd's synchronous API).
10. [8] Tab moves between widgets (focus never stays on buttons now, so shortcuts and
    Space/Enter aren't swallowed).
11. [21] Components, Styles and Scenes panels.
12. [21] Movable, customizable toolbars; tray panels reordered and undocked.
13. [3] Updating reopens the drawing only if it has a file: a never-saved drawing (or a `.skp`
    opened and not saved) isn't brought back after the restart (carry it over through autosave).

## 15. Rendering and performance
At 1500 boards (12,900 faces): hover 1-3 ms, click 9 ms, selection highlight 2 ms, an edit 14-17
ms. A 729,000-face SketchUp deck: opens in 0.3 s, first frame 0.06 s, a frame after an edit 0.03 s.
Measured with `tests/perf.rs` (`cargo test --release -p woodsketch-ui-egui --test perf --
--ignored --nocapture`, `PERF_BOARDS=1500` for a big drawing).
1. [3] No hover pre-selection highlight in Select.
2. [3] Copies scaled unevenly are shaded with slightly off normals.
3. [5] Edges thicker than 1px.
4. [5] Tool previews and highlights depth-tested (drawn on top, x-ray, now).
5. [5] Selecting a huge group builds its highlight face by face.
6. [5] Vertex dedupe (`add_vertex`) and edge lookup are linear scans per call (file import
   avoids them).
7. [13] A spatial index (BVH) for picking (every face's plane and box is tested now, cached per model).
8. [21] Each edit clones the scene for undo and re-flattens the whole view for picking (about
   0.1 s on the deck); sharing unchanged definitions and per-definition caches would cut both.
9. [3] The selection tint is built on the CPU from the flattened view, a second copy of the
   geometry the GPU places; it's drawn unbiased so it doesn't z-fight (0.1.22). Draw it from the
   placed mesh instead (the selected placements again, tinted), so the two always match.
10. [3] The viewport image is rendered to an sRGB target but egui shows its linear values as they
    are, so material colors look darker in the drawing than their swatches (the dark sky is given
    in shown values to match). Convert once, so a material looks the color you picked.

## 16. Export
Done: OBJ (+ MTL colors) and ASCII STL, inches or millimeters.
1. [3] Export the selection only.
2. [3] Each part as its own STL file.
3. [8] glTF.
4. [8] Textures in exports (needs textured materials).
5. [13] DAE.
6. [13] DXF / SVG (2D, for CNC).

## 17. Later
1. [100] A community model library (a separate service: accounts, moderation, licensing; 3D
   Warehouse's terms forbid aggregating its models).

## Not planned
- Clamping notes and dry-fit steps in the assembly pages: basic practice every woodworker knows.
- Follow Me (sweeping a profile along a path): edge profiles cover woodworking's cases. [21]
- Section Plane (slicing the model to see inside): build sheets and hidden lines show joinery;
  the one to revisit if seeing joints inside an assembly is wanted. [21]

## Deliberate differences from SketchUp (not planned)
- Starting Line, Rectangle, Circle, Arc, Push/Pull or Offset on a closed group's face opens the
  group, so the work goes into the board. (Paint still paints the whole group.)
- Paint Bucket paints the whole object; Ctrl paints one face.
- Faces drawn on the ground face up, where SketchUp shows the blue back face.
- Only groups and components can be tagged; loose geometry is always Untagged.
- Toggling a tag's visibility is an undo step (SketchUp does the same).
- Rotate grips also appear around loose connected geometry (SketchUp shows them only on groups
  and components). Kept since it helps; revisit if it gets in the way.

## Testing and infra
- Two full test runs once died without a failing assertion, likely a GPU driver crash with
  several wgpu devices at once. UI tests now take turns on the GPU; watch for recurrences.
