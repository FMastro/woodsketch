# What's new in WoodSketch

Each version's notes, newest first. The release workflow puts a version's section into its
GitHub release; keep a `## <version>` heading for every release.

## 0.1.47

- If WoodSketch crashes, the next start says so and offers to report it: Report a Problem opens
  filled in with what went wrong and where in WoodSketch's code (paths and your name taken out),
  for you to read before sending.
- Problem reports name the OS version (Windows 11 Pro 10.0.26100, not just "windows").

## 0.1.46

- Pocket screws drill real pocket holes: a 3/8" bore at 15° from the hidden face, stepping down
  to a pilot that comes out of the middle of the end, shown in the drawing and on the build sheet
  (they were listed but not drawn). Remove takes them out again.

## 0.1.45

- Angled joints on copies of a shared component: cut every copy alike (Joinery panel's Cut every
  copy, `every_copy` in recipes), or only the one picked, which becomes a component of its own.
- Remove (and Change) now work on a joint whose part was changed by hand elsewhere since (a board
  lengthened after its dado was cut); a change right where the joint is still asks for Undo. An
  angled joint whose parts were changed by hand since is no longer taken out (that lost the change).
- Push/Pull: pushing a pocket's or dado's floor back up flush closes it, as in SketchUp (it was
  refused).
- For AI assistants: angled housings read back from a drawing as housings.

## 0.1.44

- Fit allowances for biscuit and loose tenon slots, haunched tenons and angled tenons (Joinery
  panel's Fit, `fit` in recipes).
- Dovetails: set the pin and half-pin widths, for narrow hand-cut pins or a machine-cut look.
- Entity Info lists each joint cut once, and says which end it's at when two read alike (a
  drawer front made as copies).
- Build sheets: a brace or other part at an angle slides out along its own length in the step
  pictures.

## 0.1.43

- For AI assistants: recipes take tags (folders, hidden ones) on parts, copies, groups and
  templates, hardware parts, drawer and door groups, and profiles on only some edges; drawings
  read back with all of them, hidden things included (they come back hidden).
- For AI assistants: renders pick out parts inside groups and templates by name, and builds
  report their progress step by step.

## 0.1.42

- For AI assistants: recipes can draw shaped parts as an outline cut through the board's
  thickness (angled ends, tapers, curved aprons, brackets), with curves and cut-outs.
- For AI assistants: parts can be turned at compound angles (a leg splayed two ways), and such
  parts read back from a drawing keep their own length, width and thickness in the cut list.

## 0.1.41

- For AI assistants: parts that aren't plain boxes (angled ends, tapers, curves, cuts made by
  hand, parts turned at compound angles) now keep their exact shape when a drawing is read back as
  a recipe, so an assistant changing one part of a SketchUp model leaves the rest as drawn.

## 0.1.40

For AI assistants and scripts (`woodsketch-cli`, its MCP server):

- Change a design a piece at a time: add, change or remove a part, a joint or a profile, and the
  drawing is rebuilt (an edit that wouldn't build changes nothing and says why).
- Any drawing, including ones made by hand or opened from SketchUp, reads back as a recipe
  (`woodsketch-cli recipe`), so an assistant can change it the same way.
- Recipes can now place copies of a part (legs turned to each corner), turn parts (braces), set
  grain, cut and reference faces, add edge profiles, and make angled joints.
- A JSON Schema for recipes (`woodsketch-cli schema`), renders in the dark theme, from any angle,
  with parts highlighted, and the command line on its own as a download on each release.
- Joints now keep the sizes they were made with, however they were made, so Change in the Joinery
  panel starts from them.

## 0.1.39

- Center Arc and Pie: turning all the way round now closes a full circle (it snaps shut within a few
  degrees) instead of starting over, so it makes a face ready to push into a hole. Backing off opens
  it again; typing 360 works too.

## 0.1.38

- Assembly step pictures pull each part out the way it goes in: a shelf slides off its dados, a
  back pulls away behind.
- Butt joints say where the screws go ("Screw through the bottom into the side's end"), or to
  clamp or pocket-screw when that fits.
- Leaders bend around parts that would be in their way.
- A part with more than four places cut gets a table of where the cuts are, instead of crowding
  its views with dimensions.

## 0.1.37

- Assembly pages: a picture for each step, with what's together already in gray and the parts
  going on in color, labeled.
- Parts with no joints made get a glue-up order too, taken as butt joints (glued and screwed), and
  the page says so.
- Parts at an angle (braces, splayed legs) count as touching only where their faces actually meet.

## 0.1.36

- Build sheets: each cut's place along the part, its length and its depth are dimensioned on the
  views; edge profiles are drawn in orange and named; cuts at the same spot share a row.
- Reference end: Entity Info > Pick, then click an end, and lengths are measured from it.
- The traditional face mark (a loop running to the reference edge) and edge mark (a caret pointing
  to the reference face) on the build sheet and on the model.
- Assembly pages: labels the right size again, leaders that keep clear of other parts, slides
  mounted "on both" box sides, and which copies of a part take which joint ("Side apron on 2").
- Parts alike but drawn a little differently (an extra seam) share one build sheet.

## 0.1.35

- SketchUp files: dimensions and text come in on the geometry they're attached to, construction
  lines come in as short dashed guides, and softened or hidden edges stay out of sight (curves look
  smooth again).
- SketchUp files: wood parts sized like lumber (2x4s, 2x10s, 5/4 decking) come in as lumber, so
  they're bought as lumber in the cut list. When a material has both (a 2x4 frame and 3/4" slats),
  the lumber parts get a lumber copy of it.

## 0.1.34

- SketchUp files open with everything in them: groups were being left out, so walls, rooms and
  anything you had grouped in SketchUp was missing. Groups now come in as groups, named as in
  SketchUp.
- Things hidden in SketchUp (Edit > Hide) stay hidden: they go on a "Hidden in SketchUp" tag,
  which you can show from the Tags panel.
- SketchUp guide lines come in too.

## 0.1.33

- Directions are sticky too: once a line follows red, green, blue or an edge, it holds a little
  farther out unless another direction is clearly closer.
- Circle and Polygon: near the circle through a point you passed over, the rim snaps through it
  ("Through point").

## 0.1.32

- Rotated Rectangle (Draw menu): a rectangle at any angle. Click two corners for the first side,
  then move across.
- Rectangle: the far corner lines up with points you passed over (dotted line, as Line does).
- Arc: the end point infers like a line: along red, green or blue, parallel or perpendicular to
  an edge, lined up with a point.
- Snapping is "sticky": a point you snapped to holds until another is clearly closer, so it no
  longer flickers between two points near each other.

## 0.1.31

- A face you close by drawing lines now faces the same way as the faces around it (a box closed
  by hand faces out all round).
- Guide lines snap where they pass through a face.
- Offset snaps its distance to points you pass over.
- Tape Measure: measure (tap Ctrl to Measure), then type the length it should be to resize the
  whole model (one undo step).
- Scale: hold Shift to switch between even scaling and stretching.
- Move: rotation grips are hidden on faces too small on screen to use.
- New Center Arc, 3-Point Arc and Pie tools (Draw menu and the tool set). Pie closes into a face.
- Rotate and Protractor: off a face, the protractor stands in the plane you see most squarely
  (through the edge under the cursor), never edge-on.

## 0.1.30

- Double-click a text note to change its words in Entity Info; a selected dimension shows its length
  there.
- Dimension: click along an edge to dimension the whole edge in one click.
- View > Dimensions and View > Text show or hide them on their own.
- Type `[x,y,z]` before the first click to start Move, Tape Measure, Circle or Arc at that point.
- Walk: E and Q (or Page Up and Page Down) go up and down, the mouse wheel sets the speed, walls
  stop you, and typing a length sets the eye height.

## 0.1.29

- Dimensions tool (Tools > Dimension): click two points, then move out and click (or type how far
  out) to place a dimension. It always shows the current length, so it stays right as the
  drawing changes.
- Text tool (Tools > Text): click a point, then where the words go. It starts with the part's name
  or the edge's length; type your own words and press Enter.
- Dimensions and notes are saved, selected, moved, copied and erased like guides, and stay when
  guides are hidden or deleted.

## 0.1.28

- Walk (Camera > Walk): look around with the mouse and walk with W A S D or the arrow keys, Shift
  for faster, Esc to stop.
- Line: type a length, then another right after (before moving the mouse) to change that line.
- Rectangle: snaps to a square or a golden rectangle when it's close, and says so.
- Circle and Polygon: arrow keys stand them along an axis (Right red, Left green, Up blue).
- Polygon: tap Ctrl to measure the radius to a side's middle instead of a corner.
- A guide made while guides are hidden shows them again.

## 0.1.27

- Polygon tool (Draw > Polygon): a hexagon to start; type `8s` for an octagon.
- Protractor tool (Tools > Protractor): pick a center and a starting direction, then an angle
  (15° snaps, or type one) for an angled guide line.
- Push/Pull: tap Ctrl to keep the face, making a new one and walls from it (a board's end
  pulled out keeps a seam where it was).
- Offset: double-click a face to offset it by the last distance again.
- Line and Rectangle: type `[x,y,z]` before the first click to start at that point.

## 0.1.26

- Works with your AI assistant: `woodsketch-cli`, installed with the app, builds a drawing from
  a project recipe (JSON) and writes the cut list, cut sheet, build sheets, wood movement check,
  OBJ/STL and pictures. `woodsketch-cli mcp` lets Claude and other assistants design with
  WoodSketch directly. See docs/AGENTS.md.

## 0.1.25

- Dark mode: View > Theme > Dark (or Follow System to match your computer). The panels go
  charcoal, the drawing gets a dusk sky and a dark ground, and the labels on the model follow.
  It's remembered next time. Printed sheets stay light.
- The sample drawing has a door with a floating solid panel and a bench top with glued
  breadboard ends, to try View > Wood Movement on.

## 0.1.24

- Wood movement: a solid panel let into grooves floats there and isn't flagged; tick "Fastened so
  it can move" in Entity Info for a part on tabletop buttons, figure-8s or slotted screws. Both are
  listed under "Allowed for" in the Wood movement tab.
- View > Wood Movement: an arrow across each solid-wood part with how much it moves, red where a
  part it meets holds it, green where it's free.
- Materials: "Moves like" picks the species a custom wood moves like; the Wood movement tab names
  woods it has no data for.

## 0.1.23

- Report a Problem and Send Feedback no longer need a GitHub account: Send posts it, and you get
  the link to the issue it made. You can leave your email for questions; only the developer sees
  it, it's never posted. Open on GitHub instead is still there.
- In that form the title has focus, and Tab and Shift+Tab move between the title and description.

## 0.1.22

- Fix: a selected part with joinery (dovetails, a notched outline) no longer shows a flickering
  striped pattern over its faces.

## 0.1.21

- Help > Report a Problem... and Help > Send Feedback...: write what happened (or your idea), see
  exactly what goes with it (version, system, graphics card, units, the last error with file paths
  taken out, and counts from the drawing; no names, no files), then open it as a GitHub issue to
  submit. When a reported problem is fixed, the issue closes with the version that has the fix.

## 0.1.20

- Reference face and edge: Entity Info > Reference > Pick, then click the part's wide face or long
  edge. They're marked in blue on the selected part (shared by copies of a component).
- Build sheets place each cut from them: the reference face and edge marked on the views, each
  cut numbered with its distance from the reference edge, and a "Where the cuts are" list (from
  the left end, from the reference edge, how deep from the reference face).

## 0.1.19

- Build sheets: a part made as copies (four legs from one component) lists each of its cuts once,
  with how many ("In 2 places", "At both ends"), instead of once per copy; the assembly pages
  count a joint only where its cuts are.
- Parts alike, or mirror images of each other (a left and a right side), share one page: "Make 2,
  a mirrored pair".
- A glued-up part's views show its boards (named as in the cutting diagrams), their seams, and
  the growth rings alternating across the end.

## 0.1.18

- Fix: two parts that are copies of components joined somewhere else (a drawer whose sides and
  front and back are copies, dovetailed at one corner) no longer show as "Already joined" at a
  corner that only butts; the Joinery panel offers the joint there and says the copies are
  joined elsewhere.

## 0.1.17

- Build sheets: the exploded drawings label each part beside the drawing with a leader line to it
  (drawers and doors by name); hardware is said at the step it's used (hinges and pulls as a door
  is hung, slides on the drawer box's sides, a pull once its front is on); the part views dash the
  edges hidden behind the part (a dado on the far face, a mortise's bottom).

## 0.1.16

- Updating reopens the drawing you had open (after "Save changes?").
- Wood movement: a board held the same way by several parts gets one warning naming them all
  ("where it meets Panel and 2 × Side"), not one per part.

## 0.1.15

- Assembly instructions: the build sheets end with a page for each assembly (a table, a drawer, a
  cabinet), sub-assemblies first, with an exploded drawing labeled with the cut-list letters and the
  glue-up order: frames first (two aprons between their legs), then what joins them; panels in
  grooves slid in without glue; tops and backs fastened after (a solid-wood top with fasteners
  that let it move); drawers fitted and doors hung last; the hardware to install.
- File > Print Build Sheets..., the same sheets as the Cut List window's button.

## 0.1.14

- Fix: Help > Release Notes and the "What's new" links now open in your browser.
- Linux: the app is built with its X11 and Wayland window support.

## 0.1.13

- **Millimeters**: View > Units shows every length in millimeters (to 0.1) and reads a number typed
  without a unit as millimeters; inches stay the default.
- Merging faces (erasing a seam, angled joints) no longer leaves stray corners on straight edges,
  and faces painted on their own keep their paint through an angled joint.
- **Right-click menu** in the drawing: right-clicking selects what's under the pointer (unless
  it's already selected) and offers Cut, Copy, Paste, Delete, Make Group, Make Component, Explode,
  Make Unique (for a copy of a shared component) and Reverse Faces.
- **View > Guides** shows or hides every guide; hidden guides aren't drawn or snapped to.

## 0.1.12

- **Templates, more choices**: the wood (change it later with Change size), the stock thickness
  (cabinet case, drawer box, face frame), adjustable cabinet shelves on shelf pins (counted in
  the shopping list), which side a single door hinges on, middle rails in a face frame, and a
  table's leg size, apron width and top thickness. Drawer fronts over 24" wide get two pulls.
- **Change starts from the joint's own sizes**: every joint keeps the numbers it was made with,
  and Change fills the Joinery panel in with them (angled joints too, which now have Change).
- **Fit allowance** for mortise and tenon and for dowels: a "Fit" size cuts the mortise (or the
  holes) that much bigger than the tenon (or dowels), and the mortise that much deeper.

## 0.1.11

- Release notes: Help > Release Notes, a "What's new" link in Help > About, and a note in the
  status bar after an update.

## 0.1.10

- **Angled joints.** Parts meeting at an angle (a knee brace into a post, a rafter into a ridge)
  can be joined: an angled mortise and tenon (shoulders trimmed to the post's face, a tenon
  through the middle into a mortise of its exact shape) or a housing. Draw one part's end into
  the other; the overlap sets the depth. The Joinery panel offers them when parts aren't square.
- Remove puts angled joints back as drawn. Joining one copy of a shared component makes that copy
  its own, so a mirrored copy isn't cut on the wrong side.

## 0.1.9

- Tags drag by the whole row (not just the name), with an open hand over a row and a closed hand
  while dragging.

## 0.1.8

- **Drawers and doors fitted to a cabinet's openings.** Each opening of a Cabinet template can be a
  drawer (a dovetailed box on side-mount slides, a front and a pull) or doors (a pair over 24",
  slab or frame-and-panel, with concealed hinges). Fronts overlay or inset; Change size refits them.
- **Hardware as parts:** slides, hinges, pulls and knobs, counted in the shopping list and left out
  of the cut list and build sheets.
- Drawers and doors no longer raise wood-movement warnings against their case.

## 0.1.7

- Drag tags into folders (onto Untagged for the top level).
- Cedar boards (solid wood) in the library.

## 0.1.6

- **Tag folders**, like SketchUp's: a tree in the Tags panel, folders that hide everything inside,
  "Move to" on right-click, and `Folder / Tag` names in Entity Info.
- SketchUp import reads which tags are hidden and their folder tree.
- Cedar Lumber in the library; standard lumber now includes 5/4 decking and 6x posts.

## 0.1.5

- **Big models draw fast:** each component is drawn once and placed by the graphics card. A
  729,000-face deck's first frame went from 1.2 s to 0.06 s, and a redraw after an edit from 1.2 s
  to 0.03 s.
- SketchUp materials that aren't wood (metal, glass, colors) come in as Other: listed with the
  parts, left out of cutting diagrams and build sheets.

## 0.1.4

- SketchUp import about 150 times faster on big models, and faces it used to skip now come in.
- Updates: the status bar shows the download, and "Restart to update" is green.

## 0.1.3

- **Open and import SketchUp files** (SketchUp 2021 and later): components, faces, materials and
  layers, read straight from the file (no SketchUp needed). File > Open opens one as a new
  drawing; File > Import SketchUp Model adds one to yours.
- Flat things (2D people, outlines) aren't cut-list parts.

## 0.1.2

- **Templates:** a cabinet, drawer, face frame or table from a few sizes, with its joinery cut;
  Change size rebuilds it.
- **Copy and paste:** Ctrl+C / X / V (the paste follows the mouse until you click), Paste in Place,
  Duplicate (Ctrl+D); pasting into another drawing brings materials, joints and profiles along.

## 0.1.1

- The WoodSketch icon on the app, the window and the installers.

## 0.1.0

- The first downloadable beta: installers for Windows, macOS (Apple silicon and Intel) and Linux,
  with updates from inside the app.
