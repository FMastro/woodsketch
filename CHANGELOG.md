# What's new in WoodSketch

Each version's notes, newest first. The release workflow puts a version's section into its
GitHub release; keep a `## <version>` heading for every release.

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
