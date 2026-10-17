# What's new in WoodSketch

Each version's notes, newest first. The release workflow puts a version's section into its
GitHub release; keep a `## <version>` heading for every release.

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
