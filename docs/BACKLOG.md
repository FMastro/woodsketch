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
| 1 | Build sheets and assembly | 53 | Finishes what just shipped; shop output is what woodworkers keep |
| 2 | Joinery | 95 | Closes gaps in the core woodworking feature |
| 3 | Templates | 139 | Fast wins that make templates usable as-is |
| 4 | Drawers, doors and hardware | 84 | Rounds out cabinets |
| 5 | Agent-ready | 41 | The next big feature; builds on templates and joinery |
| 6 | Release polish | 79 | Signing before announcing widely |
| 7 | Importing | 238 | Gets people's existing models in |
| 8 | Cut list and materials | 171 | Better buying and cutting |
| 9 | Wood movement | 23 | Fewer false warnings, clearer results |
| 10 | Edge profiles | 76 | More bits, profiles on any board |
| 11 | Parts and boards | 11 | Small drawing conveniences |
| 12 | Modeling tools | 255 | SketchUp parity for general modeling |
| 13 | Groups, components and tags | 63 | Organizing bigger projects |
| 14 | UI | 87 | Comfort and polish |
| 15 | Rendering and performance | 66 | Only pressing for very big models |
| 16 | Export | 48 | More ways out (CNC, other apps) |
| 17 | Later | 100 | Separate projects |
| | **All** | **1629** | |

## 1. Build sheets and assembly
Done: a page per part (size, stock, three dimensioned views, steps in shop order); assembly pages
(exploded drawing with leader lines and glue-up order per assembly, sub-assemblies first, hardware
at the step it's used); hidden lines dashed in the part views; each cut once with how many; alike
and mirrored parts on one page; glue-up boards drawn; reference face and edge picked on the model
(Entity Info > Pick) and each cut placed from them on the sheet; File > Print Build Sheets.
1. [5] Where each profile sits on the build sheet: its edges drawn and named on the views (joints
   are placed; profiles are only listed as steps).
2. [5] Dimension lines for a cut's length position and depth on the views (only its distance from
   the reference edge is drawn; the rest is in the "Where the cuts are" list).
3. [3] A reference end: lengths are measured from the left end as the views draw it.
4. [3] The traditional face and edge mark symbols (the loop and the caret) on the views and
   the model, not only blue lines.
5. [8] A glue-up order for parts with no joints made (now they're only "fastened").
6. [8] An exploded drawing for each step instead of one per assembly.
7. [8] Assembly order for parts at an angle (touching is judged on boxes square to the axes).
8. [5] Leader lines can cross other parts on their way to the label column; route them around,
   or put the label on the part where there's room.
9. [3] Hardware lying on two parts (a pair of slides on both box sides) is said to go on one
   ("on J (Drawer box side)"); name both.
10. [3] Parts alike but drawn a little differently (an extra seam line, a leftover point on an
    edge) aren't recognized as the same part, so they keep a page each; compare their shapes, not
    their points.
11. [2] A cut used by different parts on different copies reads "Cut the mortise for Front and
    back apron or Side apron"; say which corner takes which.

## 2. Joinery
Done: dado, groove, rabbet, mortise and tenon (through, haunched), dovetails (through,
half-blind), dowels, biscuits, loose tenons, splines, pocket screws, box joint, half-lap, cut
where parts overlap; angled mortise and tenon and housings; Change and Remove; fit allowances on
mortises and dowel holes.
1. [3] Fit allowance on slots, haunched tenons and angled joints (angled tenons go as deep as
   the overlap drawn).
2. [3] Joints made before sizes were kept: Change starts from suggestions; fill in their sizes.
3. [5] Variable dovetail spacing.
4. [5] An angled joint on one copy of a shared component makes it its own component (the
   others stay); square joints change every copy. Offer to change all of them.
5. [8] Removing a joint after a part was changed by hand (refused now; use Undo).
6. [8] Pocket screws: draw the angled bore, not only the pocket on the hidden face.
7. [13] Sliding and lapped dovetails.
8. [13] Joining parts in different groups (both must be side by side now).
9. [13] Angled dowels, half-laps and dovetails.
10. [21] Compound angles, and a second angled joint on a part already cut at an angle in
    another plane.
11. [3] Entity Info lists a part's joints once per copy's meeting ("Dovetails with Side" twice on
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
Let AI agents design with WoodSketch, so a woodworker can ask their assistant "build me a
nightstand, 24" tall, from 3/4" cherry" and get a real drawing, cut list, cutting diagrams and
shopping list. The engine (`doc`) already runs without the UI; templates (with their drawers,
doors and hardware) are the building blocks a recipe calls.
1. [13] A project recipe (JSON): parts as boards from stock (material, section or thickness,
   length, place, orientation), templates, joints between them, tags; validated with clear
   errors an agent can fix ("Shelf doesn't touch Side"). Builds a `.woodsketch`.
2. [8] A command line: `woodsketch build recipe.json`, `cutlist`, `cutsheet` (HTML), `export`,
   `render` (PNG views), headless.
3. [13] An MCP server (`woodsketch mcp`) so Claude and other agents can call it directly: add
   boards, make joints, read the cut list and diagrams, render views to check their work.
4. [5] Docs for agents (AGENTS.md / llms.txt on woodsketch.net, with recipes for common
   furniture).
5. [2] The app opening what an agent made for editing by hand (it opens `.woodsketch` files;
   check the round trip).

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
Done: SketchUp 2021+ `.skp` (geometry, components, materials, tags and tag folders, hidden tags).
1. [3] A SketchUp material named for a species (`cedar`) comes in as that species' boards (Cedar),
   not its lumber (Cedar Lumber), so a deck's 2x10s are matched by thickness, not as 2x10
   sections. Tell lumber from boards by the parts' sizes.
2. [3] Painted wood parts (a material named like `[Color_009]`) come in as Other; materials are
   sorted into wood, sheet goods and Other by name only, and their stock may need setting.
3. [5] More sample files: a model with groups, one saved by SketchUp 2021 or 2022, a detailed
   piece of furniture (drawers, joinery). Each will turn up records that aren't read yet (tested
   on SketchUp 2026 files only so far).
4. [8] SketchUp groups (they come in as components; the flag that tells them apart isn't worked out).
5. [5] SketchUp hidden and soft edges (edges between faces under 30° still draw soft).
6. [5] SketchUp back-face materials and transparency.
7. [3] SketchUp guides.
8. [8] OBJ import (named groups, an MTL of colors).
9. [21] COLLADA (.dae) import: groups, components, names and materials. Most 3D programs and
   SketchUp (including the free SketchUp Make 2017) export it, so it's also a way in for
   SketchUp 2020 and older.
10. [8] SketchUp scenes and cameras.
11. [8] SketchUp dimensions and text (needs the Dimensions and Text tools).
12. [8] SketchUp images.
13. [13] SketchUp textures (needs textured materials).
14. [40] SketchUp 2020 and older (a different format; refused with a message now: open and save
    them in SketchUp 2021+ first, or use DAE).
15. [100] Saving back to `.skp`.

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
### Tools
1. [3] Polygon tool.
2. [5] Push/Pull: Ctrl keeps the original face (push/pull copy).
3. [3] Double-click repeat across tools (Push/Pull repeats only its own).
4. [5] Typing before the first click (absolute `[x,y,z]` start points).
5. [5] Protractor tool (angled guide lines).
6. [8] Text tool.
7. [8] Walk and Look Around, like a 3D game: the mouse looks around, W A S D or the arrow keys
   walk (forward, left, back, right). While walking those keys move you instead of picking tools
   (W, S, A are Board, Scale and Arc); Esc stops walking.
8. [13] Dimensions tool.
9. [21] Follow Me.
10. [21] Section Plane.

### Inference and snapping
11. [3] Line: change a typed length after the line is drawn.
12. [3] Rectangle: square and golden-ratio diagonal hints.
13. [5] Faces closed from loose lines face to match connected faces (up, front, then right now;
    Reverse Faces turns them over).
14. [5] A "sticky" preference for the last inference (snapping only weighs screen distance).
15. [8] Rectangle, Arc and Circle use parallel/perpendicular and From Point inference (Line,
    Move and Tape do).
16. [8] Rectangles off the axes.

### Circles and arcs
17. [3] Circles off any face lie flat on the ground; follow the axis or a face under the cursor.
18. [5] Arc: tangent inference.
19. [5] 3-point Arc and Pie.
20. [8] Soften/Smooth per edge, and smooth shading (soft edges are worked out when drawing now,
    facets are shaded flat).
21. [13] Curves as one entity (a circle is 24 edges now; SketchUp selects it with one click and
    shows its radius and sides).

### Offset, Tape Measure, Scale, Rotate
22. [1] New guides made while guides are hidden stay hidden; show them.
23. [3] Offset: snap the distance to points the cursor passes.
24. [3] Offset: rounded outward corners (always mitered now).
25. [5] Offset holes too (only a face's outer outline now).
26. [5] Offset a selection of connected edges (only faces now).
27. [3] Guide lines snap where they cross faces (edges and other guides only now).
28. [5] Tape Measure: resize the model after measuring (scale everything so the measured distance
    becomes the typed one).
29. [3] Scale: Shift switches corner grips to non-uniform and edge grips to uniform.
30. [3] Scale grips as green boxes, hiding the ones facing away (red crosses now).
31. [3] Rotate: hide grips on faces that are tiny on screen (they crowd on thin boards).
32. [5] Protractor auto-orients to the hovered edge's plane and flips when seen edge-on.
33. [8] Scale: mirroring (negative factors are refused).
34. [8] Scale: a rotated group among several scaled together gets sheared (they use the
    red/green/blue axes), and the cut list measures it only approximately.

### Geometry
35. [5] Collinear leftovers: other merges (pulling part of a top up flush with a side) can still
    leave a corner on a straight edge (erasing a seam heals them).
36. [8] Coplanar faces sharing an edge merge on their own (boards glued end to end keep a seam).
37. [8] Move folds a face that would bend (adds a diagonal, as SketchUp does); refused now.
38. [21] Auto-joining: moved or copied geometry welds to geometry it lands on (also pasted loose
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
