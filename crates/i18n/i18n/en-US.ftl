# WoodSketch in English. This is the catalog every translation is filled in from, so a key here
# is the one the code asks for, and a key missing from a translation shows in English rather
# than showing nothing.
#
# A key is `area-thing`, and a tooltip is the same key with `-hint`. Keep a key when its wording
# changes and the translations keep up by themselves; change a key only when the thing it names
# has become something else, so that a stale translation is noticed rather than silently kept.
#
# Punctuation belongs to the translation. A trailing `...` means "this opens something that will
# ask you more", which is a Windows convention a translator should be free to drop.

## The menu bar

menu-file = File
menu-edit = Edit
menu-view = View
menu-camera = Camera
menu-draw = Draw
menu-tools = Tools
menu-window = Window
menu-help = Help

## File

file-start-page = Start Page
file-start-page-hint = New, Open, Recent and the samples; Esc comes back here
file-new = New
file-open = Open...
file-open-recent = Open Recent
file-save = Save
file-save-as = Save As...
file-import-skp = Import SketchUp Model...
file-import-skp-hint = Adds a .skp (SketchUp 2021 or later) to this drawing
file-print-build-sheets = Print Build Sheets...
file-print-build-sheets-hint = A page per part with its steps, then how it all goes together, with exploded drawings; opens in your browser to print
file-export = Export
file-export-obj = OBJ...
file-export-obj-hint = The 3D model with its material colors, for renderers and other modelers
file-export-stl = STL...
file-export-stl-hint = The 3D model, for 3D printing
file-export-dxf = DXF for CNC...
file-export-dxf-hint = Each part flat, seen from its face: the outline, through holes, and pockets on layers by depth
file-export-svg = SVG for CNC...
file-export-svg-hint = Each part flat, seen from its face: the outline black, holes white, pockets grey
file-export-units = Units
file-export-inches = Inches
file-export-millimeters = Millimeters
file-exit = Exit

## Edit

edit-undo = Undo
edit-redo = Redo
edit-cut = Cut
edit-copy = Copy
edit-paste = Paste
edit-paste-hint = Follows the mouse; click to put it down
edit-paste-in-place = Paste in Place
edit-duplicate = Duplicate
edit-duplicate-hint = A copy right beside it
edit-delete = Delete
edit-delete-guides = Delete Guides
edit-select-all = Select All
edit-select-none = Select None
edit-make-group = Make Group
edit-make-component = Make Component...
edit-explode = Explode

## View

view-grid = Grid
view-wood-movement = Wood Movement
view-wood-movement-hint = How much each solid-wood part moves across its grain: red where a part it meets holds it, green where it's free
view-theme = Theme
view-theme-light = Light
view-theme-dark = Dark
view-theme-system = Follow System
view-theme-hint = Dark is easier on the eyes at night; Follow System matches your computer
view-units = Units
view-units-hint = How lengths are shown, and what a number typed without a unit means; saved with the drawing
view-guides = Guides
view-guides-hint = Show or hide every guide (hidden ones aren't snapped to)
view-dimensions = Dimensions
view-dimensions-hint = Show or hide every dimension
view-text = Text
view-text-hint = Show or hide every text note
view-default-tray = Default Tray
view-language = Language
view-language-hint = What WoodSketch itself is written in; your drawing is untouched

## Camera

camera-standard-views = Standard Views
camera-view-iso = Iso
camera-view-top = Top
camera-view-front = Front
camera-view-right = Right
# The toolbar's view buttons, whose tooltip names the view: "Top view".
camera-view-hint = { $view } view
camera-zoom-extents = Zoom Extents

## Tools

tools-cut-list = Cut List...

## Window

window-default-tray = Default Tray
window-cut-list = Cut List

## Help

help-check-for-updates = Check for Updates...
help-check-at-startup = Check for updates at startup
help-check-at-startup-hint = A quick look at WoodSketch's releases on GitHub when it opens; new versions download in the background
help-whats-new = What's New...
help-whats-new-hint = What the last few updates brought, newest first
help-release-notes = Release Notes
help-release-notes-hint = Every version's notes, on GitHub
help-report-problem = Report a Problem...
help-report-problem-hint = Tell us what went wrong: opens a GitHub issue with details that help, nothing private
help-send-feedback = Send Feedback...
help-send-feedback-hint = An idea or anything else: opens a GitHub issue
help-problems-and-ideas = Problems and Ideas
help-problems-and-ideas-hint = Everything people have reported or suggested, and what's been fixed, on GitHub
help-discussions = Discussions
help-discussions-hint = Ask how to do something, say what WoodSketch should handle next, or show a piece you've made
help-about = About WoodSketch

## The About window

about-title = About WoodSketch
about-version = WoodSketch { $version }
about-whats-new = What's new
about-what-it-is = A lightweight 3D modeler for woodworking, in the spirit of SketchUp.
about-not-affiliated = Not affiliated with or endorsed by Trimble.

## The status bar

status-reloaded = Reloaded: the file changed outside WoodSketch
status-reloaded-hint = Ctrl+Z goes back to the drawing as it was before the reload
status-dismiss = Dismiss
status-whats-new = What's new
status-whats-new-hint = What came in this update, and in any you skipped
status-updated = Updated to { $version }
status-downloading = Downloading WoodSketch { $version }...
status-restart-to-update = Restart to update to { $version }
status-restart-to-update-hint = Installs the new version and reopens WoodSketch (you'll be asked to save first)

## The tools' own names, shown in the Tools menu and in each toolbar button's tooltip.
##
## These are SketchUp's names, because most people arriving here already know them from there.
## SketchUp has its own translation, so if it names a tool in your language, that name is the one
## your readers will be looking for -- use it rather than a fresh translation of the English.
## Leaving one in English is a reasonable choice too, if that is what people in your language
## actually say.

tool-name-select = Select
tool-name-eraser = Eraser
tool-name-paint-bucket = Paint Bucket
tool-name-line = Line
tool-name-arc = Arc
tool-name-rectangle = Rectangle
tool-name-circle = Circle
tool-name-polygon = Polygon
tool-name-center-arc = Center Arc
tool-name-three-point-arc = 3-Point Arc
tool-name-pie = Pie
tool-name-rotated-rectangle = Rotated Rectangle
tool-name-board = Board
tool-name-push-pull = Push/Pull
tool-name-move = Move
tool-name-rotate = Rotate
tool-name-scale = Scale
tool-name-offset = Offset
tool-name-tape-measure = Tape Measure
tool-name-protractor = Protractor
tool-name-dimension = Dimension
tool-name-text = Text
tool-name-orbit = Orbit
tool-name-pan = Pan
tool-name-zoom = Zoom
tool-name-walk = Walk

# A toolbar button's tooltip: the tool's name and the key that picks it. $key is the same on
# every keyboard, so keep it as it is.
tool-hint = { $tool } ({ $key })
# The same, for a tool that isn't built yet.
tool-hint-coming-soon = { $tool } ({ $key }): coming soon
# Said at the bottom of the window when someone picks a tool that isn't built yet.
tool-coming-soon = { $tool } ({ $key }) is coming soon.
# Said when a length or an angle is typed while holding a tool that has nothing to do with one.
tool-takes-no-typed-values = { $tool } doesn't take typed values.

## The toolbar. Each of these is a tooltip, and carries the keyboard shortcut in brackets; the
## shortcut itself is the same on every keyboard, so leave it as it is and translate around it.

toolbar-new-hint = New (Ctrl+N)
toolbar-open-hint = Open (Ctrl+O)
toolbar-save-hint = Save (Ctrl+S)
toolbar-undo-hint = Undo (Ctrl+Z)
toolbar-redo-hint = Redo (Ctrl+Y)
toolbar-zoom-extents-hint = Zoom Extents (Shift+Z)
toolbar-cut-list-hint = Cut List: parts and cutting diagrams

## The Default Tray, down the right-hand side, and the name of each panel in it

tray-title = Default Tray
tray-hide-hint = Hide the tray (Window > Default Tray)
tray-board = Board
tray-entity-info = Entity Info
tray-joinery = Joinery
tray-edge-profile = Edge profile
tray-templates = Templates
tray-materials = Materials
tray-tags = Tags
tray-outliner = Outliner
# The sizes of whichever template is selected: "Cabinet sizes", "Drawer sizes". The template's own
# name is not translated yet, so it arrives in English.
tray-template-sizes = { $template } sizes

## The What's New window (Help > What's New), shown once after an update

whats-new-title = What's New in WoodSketch
# Marked beside the version that is running.
whats-new-this-one = (this one)
# How many versions have arrived since the one that ran last. Two messages rather than one with a
# plural, because the singular drops the number altogether: "One update since 0.1.64:".
whats-new-one-since = One update since { $from }:
whats-new-many-since = { $count } updates since { $from }:
whats-new-recent = The most recent updates:
# "Done" rather than "Close", because clicking it marks the notes as read.
whats-new-done = Done
whats-new-done-hint = Marks these as read
whats-new-every-version = Every version on GitHub

## The Outliner panel: where you are in the drawing, and what is at this level

outliner-empty = No groups or components yet.
outliner-group = Group
outliner-group-hint = Make Group (Ctrl+G)
outliner-component = Component
outliner-component-hint = Make Component (G)
outliner-explode-hint = Turn the selected groups back into loose geometry
outliner-auto-group = Auto-group new boards
outliner-auto-group-hint = When Push/Pull turns a free-standing face into a solid, make it a group
# The start of the trail showing which group is open: "Model > Case > Leg".
outliner-model = Model
outliner-close-group = Close group (Esc)
# Marks a row whose tag is hidden, after the name.
outliner-hidden = (hidden)
outliner-open-hint = Open: you're working inside it
outliner-needs-material-hint = Needs a material for the cut list

## The Board panel: the stock a new board is drawn from

board-material = Material
board-pick-material = Pick a material
# A material with what it comes as, when its name doesn't already say: "Pine (boards)".
board-with-noun = { $name } ({ $noun })
board-noun-lumber = lumber
board-noun-boards = boards
board-noun-sheets = sheets
# The headings the material dropdown groups by.
board-kind-lumber = Lumber: 2x4, 2x6...
board-kind-boards = Boards: 3/4", 1"... thick
board-kind-sheets = Sheets: plywood, MDF
board-from-library-hint = From the library: adds it to the drawing
board-stock = Stock
board-custom-size-open = Custom size...
board-custom-size-open-hint = Add a size this material comes in (or type it in the Measurements box)
board-lay-it = Lay it
board-flat = Flat
board-on-edge = On edge
board-on-edge-hint = Ctrl switches while drawing
board-lumber-note = Lumber is named by its nominal size; the actual size is in brackets. Draw any length: the cut list picks the board to buy (8', 10'...). For a one-off size, type it, like 3,6 1/2; to keep a size in the list, use Custom size.
board-sheet-note = Drawn as a rectangle at this thickness. For 2x4s and other framing sizes, pick a Lumber material such as Pine Lumber.
board-custom-size = Custom size
board-thickness = Thickness
board-width = Width
board-add-to = Add to { $material }
board-cancel = Cancel
board-custom-note = It's added to the material's sizes, so the cut list knows it.

## The Tags panel

tags-add-hint = Add a tag
tags-add-folder-hint = Add a folder (right-click a tag to move it into one)
tags-untagged = Untagged
tags-fold = Fold
tags-unfold = Unfold
# Two whole sentences rather than one built from pieces: a sentence assembled out of fragments
# only ever comes out right in the language it was assembled for.
tags-hide-folder = Hide everything in this folder
tags-hide-tag = Hide everything with this tag
tags-show = Show it again
tags-rename-hint = Double-click to rename; drag the row onto a folder (or right-click) to move it
tags-delete-folder-hint = Delete folder (what's in it moves up)
tags-delete-tag-hint = Delete tag (its groups become Untagged)
tags-move-to = Move to
tags-top-level = Top level
tags-no-folders = No folders yet: add one with the folder button.

## Entity Info: what is selected, its name, tag and material

entity-nothing-selected = Nothing selected.
entity-name = Name
entity-text = Text
entity-tag = Tag
entity-dimension = Dimension  { $length }
# Loose geometry — edges, faces, guides — rather than groups.
entity-loose =
    { $count ->
        [one] { $count } loose entity
       *[other] { $count } loose entities
    }
entity-selected = { $count } selected ({ $groups } groups/components)
entity-tags-note = Tags go on groups and components. Group this first (Ctrl+G).
entity-material = Material
entity-material-default = Default

## The Materials panel, and the editor behind the pencil

materials-add-hint = Add a material
materials-none = No material (default)
materials-none-hint = The plain default look. Painting with it removes paint. It isn't a material you can edit: add one with +.
# $kind is the sort of material (Solid wood, Sheet good); $pencil is the pencil icon.
materials-paint-hint = { $kind }. Click to paint with it; { $pencil } to edit it.
materials-edit-hint = Edit this material
materials-empty = No materials yet. Click { $plus } to add one, like Red Oak or Plywood.
materials-none-selected = No material is selected: painting removes paint. Pick a material above to paint with.
materials-editing = Editing { $name }
materials-close-editor = Close the editor
materials-library = Library
materials-new-blank = New blank material
materials-name = Name
materials-color = Color
materials-type = Type
materials-grain = Grain
materials-has-grain = Has grain
materials-has-grain-hint = Parts must follow the grain when cut
materials-thicknesses = Thicknesses
materials-sheet-sizes = Sheet sizes
materials-board-lengths = Board lengths
materials-board-widths = Board widths
materials-lengths = Lengths
materials-not-cut = Not cut from stock; the cut list counts the parts.
materials-moves-like = Moves like
materials-moves-like-hint = For the wood movement check: the species whose seasonal movement it shares
materials-movement-custom = Custom
materials-movement-unknown = Not known
materials-sections = Sections ({ $shown } of { $total })
materials-sections-note = Nominal and actual sizes it comes in; each part gets the smallest that fits.
materials-delete = Delete material
materials-delete-hint = Painted things go back to no material

## Which way a face or an edge points in the drawing.
##
## Two forms of each, because English puts them in sentences two ways: "Top-left" starts a name,
## "around the top" sits inside one. A language that doesn't change the word can give both the
## same text; one that changes it more than English does has somewhere to put each form.

side-top = Top
side-bottom = Bottom
side-left = Left
side-right = Right
side-front = Front
side-back = Back
side-top-lower = top
side-bottom-lower = bottom
side-left-lower = left
side-right-lower = right
side-front-lower = front
side-back-lower = back

## The Edge profile panel

profile-assembly = An assembly: profile the parts inside it.
profile-shape = Shape
profile-size = Size
profile-around-face = Around a face
profile-along-length = Along the length
# A corner named by the two faces meeting at it: "Top-left".
profile-corner = { $first }-{ $second }
# Which face the edges are taken around: "Around the top".
profile-around = Around the { $side }
# One edge of that face: "Left edge".
profile-edge = { $edge } edge
# $profile is the router bit's name (Roundover, Chamfer), which is not translated yet.
profile-rout = Rout the { $profile }
profile-routed = { $size } { $profile } routed.
profile-edge-profiles = Edge profiles

## Entity Info's part section: size, stock, grain, reference marks, cut and movement

part-assembly = An assembly: its parts are the groups and components inside.
part-size = Size
part-size-hint = Length (along the grain) x width x thickness
part-stock = Stock
part-needs-material = Needs a material
part-stock-line = { $material }, { $stock }
part-stock-glued-up = { $material }, { $stock }, glued up from { $boards }
part-grain = Grain
part-grain-along-side = Along the { $length } side
part-grain-along-length = Along its length
part-grain-along-length-hint = The longest side (the usual case)
part-reference = Reference
part-reference-hint = The face, edge and end the build sheet measures from (a woodworker's face and edge marks)
part-reference-face-edge = Face { $face }, edge { $edge }
part-reference-end = End { $end }
part-reference-not-set = not set
# The end is the one mark with no default, so saying it isn't set would be misleading.
part-reference-as-drawn = left as drawn
part-pick = Pick
part-pick-hint = Then click the part: a wide face sets the reference face, a long edge the reference edge, an end the reference end
part-clear = Clear
part-cut = Cut
part-cut-unknown-hint = Movement checks will assume flat-sawn, which moves the most
part-cut-flat-hint = Plain-sawn: growth rings run across the end grain; moves the most
part-cut-quarter-hint = Growth rings run up and down the end grain; moves about half as much
part-movement = Movement
part-free-to-move = Fastened so it can move
part-free-to-move-hint = Tabletop buttons, figure-8s or screws in slotted holes, not glued or screwed solid: it's free to move across its grain
part-glue-up = Glue up
part-board-one = one board
part-boards-n = { $count } boards
part-glue-auto = Auto ({ $boards })
part-glue-one = One board
part-glue-from = From { $count } boards
part-edge-joint = Edge joint
# $piece is a biscuit or a loose tenon, named by the joinery; English makes it plural with an s.
part-edge-joint-hint = { $piece }s along each seam, about every { $spacing }
part-edge-glued-hint = Edge-glued: a well-jointed glue line is stronger than the wood

## The Templates panel: a cabinet, drawer, face frame or table from a few sizes

template-wood = Wood
# $template is the template's own name (Cabinet, Drawer), not translated yet.
template-add = Add the { $template }
template-add-hint = Built beside the drawing, as one group; one Undo takes it out
template-change-size = Change size
template-change-size-hint = Builds it again with these sizes and wood, where it is. Changes made inside it by hand are lost.
template-width = Width
template-height = Height
template-depth = Depth
template-stock = Stock
template-top = Top
template-shelves = Shelves
template-adjustable = Adjustable (on shelf pins)
template-back = Back
template-back-ply = 1/4" plywood
template-joints = Joints
template-corners = Corners
template-stiles = Stiles
template-rails = Rails
template-legs = Legs
template-aprons = Aprons
template-middle-rails = Middle rails
template-fronts = Fronts
template-pulls = Pulls
template-doors = Doors
template-hinge = Single doors hinge
template-drawer-corners = Drawer corners
# The openings in a cabinet, named top-down as you look at it. A cabinet with no shelves has
# just the one, so it needs no number.
template-opening = Opening
template-opening-bottom = Bottom opening
template-opening-top = Top opening
template-opening-n = Opening { $n }
template-note-cabinet = Sides, top, bottom and shelves of the stock; outside sizes. Drawers ride on side-mount slides; doors wider than 24" come in pairs.
template-note-drawer = A box with a 1/4" plywood bottom in grooves; outside sizes.
template-note-face-frame = Stiles full height, rails between them.
template-note-table = Square legs, 3/4" thick aprons, a top overhanging 1"; height to the top of the top.

## The Joinery panel: the joint between two selected parts
##
## $part and $into are the parts' own names, as the drawing calls them, and $kind and $detail
## are the joinery's own words. They arrive as they are.

joinery-kind-let-in = Dado, groove or rabbet
joinery-kind-overlap = Cut where they overlap
joinery-kind-mortise-tenon = Mortise and tenon
joinery-kind-dowels = Dowels
joinery-kind-biscuits = Biscuits
joinery-kind-loose-tenons = Loose tenons (Domino style)
joinery-kind-pocket-screws = Pocket screws
joinery-kind-dovetails = Dovetails
joinery-kind-splines = Spline
joinery-kind-box-joint = Box joint
joinery-kind-half-lap = Half-lap

# How the two parts have to be sitting before each joint can be made.
joinery-how-let-in = Butt one part's end or edge against the other (a shelf against a side, a panel against a post). It's let in by the depth: a dado across the grain, a groove along it, a rabbet at an edge.
joinery-how-mortise-tenon = Butt the rail's end against the other part (a rail against a leg or stile).
joinery-how-butt = Butt one part's end or edge against the other.
joinery-how-dovetails = Butt one board's end against the inside of the other at its end, like a drawer's side against its front. The butting board gets the tails.
joinery-how-box-joint = Put the two parts' ends together at a corner (a drawer's front and side), butted or overlapping.
joinery-how-half-lap = Cross the two boards so they overlap through their whole thickness.
joinery-how-overlap = Move one part into the other by the joint's depth (a shelf 1/4" into the side).

joinery-elsewhere = Other copies of these parts are joined ({ $joints }) where they meet elsewhere; not here yet.
joinery-already = Already joined:
joinery-change = Change
joinery-change-hint = Take it out and set it up again from its sizes
joinery-remove = Remove
joinery-remove-hint = Take this joint out of both parts
joinery-removed = Joint removed.
joinery-set-up-again = Set it up again, then make it.
joinery-add-another = Add another joint
# What an unnamed part is called while describing a joint.
joinery-part-default = Part
joinery-joint = Joint
joinery-place-first = { $joint } (place the parts first)
joinery-joints = Joints
joinery-at = (at the { $at })
joinery-row = { $kind } with { $other }{ $at }, { $detail }

joinery-cut-into = Cut { $cut } into { $into }
joinery-cut-into-hint = { $cut } keeps its size; { $into } gets the cut
joinery-cut-in = { $kind } cut in { $into }.
joinery-make-half-lap = Make the half-lap
joinery-half-lap-made = Half-lap made.
joinery-finger-width = Finger width
joinery-finger-width-hint = Rounded so there's an odd number of fingers
joinery-make-box-joint = Make the box joint
joinery-box-joint-made = Box joint made.
joinery-end-meets = { $part }'s end meets { $into }.

joinery-depth = Depth
joinery-depth-hint = How far it's let in; it grows by this
joinery-let-into = Let { $part } into { $into }
joinery-let-in-done = { $kind } cut in { $into }; { $part } is { $length } longer.

joinery-through = Through
joinery-through-hint = The tails show on both faces of the corner
joinery-half-blind = Half-blind
joinery-half-blind-hint = Hidden from the front, as on a drawer front
joinery-tails = Tails
joinery-tails-hint = How many tails across the width
joinery-pins = Pins
joinery-pins-hint = Each pin's width where it shows: narrow (1/8") for a hand-cut look, wider for a machine-cut one
joinery-half-pins = Half pins
joinery-half-pins-hint = The pins at each edge; the tails share the rest
joinery-dovetail-note = { $part } gets the tails, { $into } the pins (1:8 slope).
joinery-cut-dovetails = Cut the dovetails
joinery-dovetails-done = Dovetails cut; { $part } is longer by the tails.

joinery-spline-note = A { $piece } along the whole joint, glued into a groove in each part.
joinery-cut-spline = Cut the spline groove
joinery-spline-done = Spline groove cut in both parts.

joinery-size = Size
joinery-count = Count
joinery-count-hint-slots = Spread evenly across the joint
joinery-fit = Fit
joinery-fit-hint-slots = How much roomier the slots are than the pieces (thicker, longer, deeper); 0 for a tight fit
joinery-cut-biscuit-slots = Cut biscuit slots
joinery-cut-loose-tenon-mortises = Cut loose tenon mortises
joinery-slots-done = { $count } slots cut in each part, for { $count } x { $piece }.

joinery-count-hint-pocket = About one every 4"
joinery-screw = Screw
joinery-screw-hint = Length, for the thickness of the part they go through
joinery-pocket-note = Drilled from the hidden face of { $part }; the holes aren't drawn.
joinery-add-pocket-screws = Add pocket screws
joinery-pocket-done = { $count } pocket screws added.

joinery-through-tenon = Through tenon ({ $length } long, shows on the far side)
joinery-haunch = Haunch (fills a panel groove at the end)
joinery-length = Length
joinery-length-hint = How far the tenon goes into the mortise
joinery-thickness = Thickness
joinery-thickness-hint = About a third of the rail is usual
joinery-shoulders = Shoulders
joinery-shoulders-hint = Set in from the rail's edges, each side
joinery-fit-hint-mortise = How much bigger the mortise is than the tenon (wider, thicker, deeper); 0 for a tight fit
joinery-tenon-mortise = Tenon on { $part }, mortise in { $into }
joinery-mortise-tenon-done = Mortise and tenon made; { $part } is { $length } longer in the cut list.

joinery-count-hint-dowels = Spread evenly across the width
joinery-diameter = Diameter
joinery-diameter-hint = 3/8" for 3/4" stock is usual
joinery-depth-hint-dowels = Hole depth in each part
joinery-fit-hint-dowels = How much bigger the holes are than the dowels; 0 for a tight fit
joinery-drill-dowels = Drill for dowels
joinery-dowels-done = { $count } dowel holes drilled in each part.

## Angled joints, where two parts meet at something other than a right angle

joinery-meet-angle = They meet at { $angle }°.
joinery-draw-into = Draw one part's end into the other where the joint goes: the overlap sets how deep it is.
joinery-angled-mortise-tenon-hint = A tenon through the middle, into a mortise of its shape
joinery-housing = Housing
joinery-housing-hint = The whole end let into the other part
joinery-every-copy = Cut every copy
joinery-every-copy-hint = These are copies of a shared component: cut every copy alike, or only this one (it becomes a component of its own)
joinery-let-into-done = { $part } let into { $into }.
joinery-angled-thickness-hint = Through the middle of the tenoned part; about a third
joinery-angled-shoulders-hint = Set in from its edges, each side
joinery-angled-fit-hint = How much smaller the tenon is than its mortise (thinner, narrower, shorter); 0 for a tight fit
joinery-angled-done = Angled mortise and tenon made: { $part }'s end is trimmed to { $into }'s face, with its tenon in.

## The Cut List window

cutlist-title = Cut List
cutlist-tab-parts = Parts
cutlist-tab-diagrams = Cutting diagrams
cutlist-tab-movement = Wood movement
cutlist-tab-settings = Saw kerf
cutlist-print-cut-sheet = Print cut sheet...
cutlist-print-cut-sheet-hint = Save the parts and cutting diagrams as a page to print, or save as PDF, from your browser
cutlist-print-build-sheets = Print build sheets...
cutlist-print-build-sheets-hint = A page per part for the shop: its size, drawings, joints and profiles, in order
cutlist-export-csv = Export CSV...
cutlist-save-failed = Couldn't save { $path }: { $error }
cutlist-saved-not-opened = Saved, but couldn't open it: { $error }
cutlist-kerf-note = The saw kerf every cut takes, saved with the drawing.
cutlist-stock-note = Stock sizes (thicknesses, sheet sizes, lumber sections and lengths) are set for each material in the Materials panel.
cutlist-saw-kerf = Saw kerf
cutlist-kerf-unreadable = The kerf doesn't read as a length; the last good one is used.

cutlist-no-parts = No parts yet. Each group or component is a part: boards made with Push/Pull count automatically.
# How many parts still have no material. The verb agrees with the number in English, which is
# why this is one message and not a sentence glued together from three.
cutlist-needs-material =
    { $count ->
        [one] { $warning } { $count } part needs a material (listed first). Paint each one, or paint the group around them.
       *[other] { $warning } { $count } parts need a material (listed first). Paint each one, or paint the group around them.
    }
cutlist-loose-faces = { $count } loose faces aren't in any group, so they aren't parts. Group them (Ctrl+G) to list them.
cutlist-needs-a-material = Needs a material
cutlist-parts-count =
    { $count ->
        [one] { $count } part
       *[other] { $count } parts
    }
cutlist-material-line = { $kind }  ·  { $parts }  ·  { $amount }
cutlist-edit-material = Edit material
cutlist-square-feet = { $amount } sq ft
cutlist-linear-feet = { $amount } linear ft
cutlist-board-feet = { $amount } board ft
cutlist-pieces = { $count } pieces

# The parts table's column headings.
cutlist-column-name = Name
cutlist-column-qty = Qty
cutlist-column-length = Length
cutlist-column-width = Width
cutlist-column-thickness = Thickness
cutlist-column-stock = Stock
cutlist-glued-up = { $stock }, glued up from { $boards }
cutlist-glued-up-hint = { $boards } boards about { $width } x { $length }, edges jointed and glued, then cut to size
cutlist-no-stock = none
cutlist-hardware = Hardware

cutlist-nothing-to-cut = No parts to cut yet.
cutlist-hardware-not-cut = Hardware isn't cut from stock.
cutlist-paint-to-plan = Paint these parts with a material (and set its type) to plan the cuts.
cutlist-type-not-cut = This material type isn't cut from stock. Set it to sheet good, solid wood or dimensional lumber.
cutlist-buy = Buy:
cutlist-buy-line = { $count } × { $what }
cutlist-used = ({ $percent }% used)
cutlist-part-problem = Part { $label }: { $reason }
cutlist-stock-line = { $n }. { $description }  ·  { $percent }% used
cutlist-not-from-stock = Not cut from stock (material type Other): { $materials }.

## The Wood movement tab

movement-intro = Solid wood swells and shrinks across its grain as the seasons change. Plywood and MDF don't.
movement-swing = Moisture swing
movement-swing-hint = How much the wood's moisture content changes over a year: about 4% indoors (6% in a dry winter to 10% in a humid summer); more in a shed or an unheated room
movement-unknown = { $info } No movement data for { $materials }: set "Moves like" on it in the Materials panel to check its parts.
movement-none-known = No solid-wood parts with a known species yet. Paint parts with a wood from the library, like Cherry or Red Oak.
movement-no-conflicts = { $check } No parts fight each other's movement.
movement-watch = Watch these joints
movement-fastened-note = If a part is fastened so it can move (tabletop buttons, figure-8s, slotted screw holes), tick "Fastened so it can move" for it in Entity Info.
movement-allowed = Allowed for
movement-how-much = How much each part moves across its grain
movement-column-part = Part
movement-column-wood = Wood
movement-column-width = Width
movement-column-moves = Moves about
movement-assumed-flat-hint = Its cut is "Not sure", so it's figured as flat-sawn, which moves the most. Set the cut in Entity Info.

## The Start page

start-back-to = ‹  Back to { $drawing }
start-back-to-hint = Esc
start-title = Start
start-intro = Make a drawing, open one you were working on, or take a sample apart.
start-new-drawing = New drawing
start-inches = Inches
start-millimeters = Millimeters
start-open = Open...
start-open-under = A drawing or a SketchUp model (Ctrl+O)
start-import = Import...
start-import-under = A SketchUp model, into the drawing that's open
start-recent = Recent
start-recent-empty = Drawings you open or save appear here.
start-new = New
start-samples = Samples
start-samples-note = Finished pieces, built fresh each time you open one. Change anything: they aren't files, so nothing here can be written over.
# How a card reads out to a screen reader: its title, then the line under it.
start-card-label = { $title }. { $under }

## Opening, saving and the title bar

files-untitled = Untitled
files-save-failed = Couldn't save { $path }: { $error }
files-open-failed = Couldn't open { $path }: { $error }
files-export-failed = Couldn't export { $path }: { $error }
# The window's title: the drawing's name, a star if there are unsaved changes, then the app.
files-window-title = { $name }{ $dirty } - WoodSketch
# Dismisses a message about a file that couldn't be opened or saved.
files-ok = OK
files-save-changes = Save changes?
files-save-changes-question = Do you want to save the changes to { $name }?
files-save = Save
files-dont-save = Don't Save
files-cancel = Cancel
# The named kinds of file in the system Open and Save dialogs.
files-filter-drawing = WoodSketch drawing
files-filter-drawings-and-skp = Drawings and SketchUp models
files-filter-sketchup = SketchUp model (2021 and later)
files-filter-web-page = Web page

## Report a Problem and Send Feedback

feedback-problem-title = Report a Problem
feedback-idea-title = Send Feedback
feedback-unreachable = Couldn't reach woodsketch.net ({ $error }). Check your connection, or use Open on GitHub.
feedback-after-crash = WoodSketch closed unexpectedly. What I was doing:
feedback-thanks = { $check } Thank you! It's issue #{ $number } on GitHub:
feedback-follow-email = Follow it there to see replies. If there are questions, you'll get an email.
feedback-follow = Follow it there to see replies (no account needed to read it).
feedback-close = Close
feedback-kind-problem = Something went wrong
feedback-kind-idea = An idea or other feedback
feedback-title-label = Title
feedback-what-happened = What happened? (what you did, what you expected)
feedback-your-idea = Your idea
feedback-email = Your email (optional)
feedback-email-hint = Only if you'd like to be asked about it
feedback-email-note = Only the developer sees it, to ask about this report. It isn't posted anywhere.
feedback-include-counts = Include counts from the open drawing (no names)
feedback-include-counts-hint = How many parts, faces, joints and materials of each kind: helps reproduce a problem
feedback-sent-with = Sent with it:
feedback-public-note = It becomes a public issue on GitHub, where you can follow it. No account needed to send it.
feedback-bad-email = That email address doesn't look right.
feedback-send = Send
feedback-sending = Sending...
feedback-open-github = Open on GitHub instead
feedback-open-github-hint = Submit it yourself on GitHub (needs a free account); your email isn't included
feedback-cancel = Cancel

## Checking for updates (Help > Check for Updates)

updates-checking = Checking for updates...
updates-not-installed = This copy wasn't installed with the WoodSketch installer, so it can't update itself. Download the latest from woodsketch.net.
updates-up-to-date = WoodSketch { $version } is up to date.
updates-downloading = WoodSketch { $version } is downloading in the background (shown at the bottom right). When it's ready, a green "Restart to update" button appears there.
updates-ready = WoodSketch { $version } is ready: restart to update.
updates-failed = Couldn't check for updates: { $error }

## While a sample is being built

building-title = Building
building-what = Building the { $sample }
building-note = It's built from a recipe each time, by the joinery the app has now.

## Opening and importing, crash reports, recovery, and the file-changed question

app-opened-skipped = Opened, but { $count } faces couldn't be read and were left out.
app-imported-skipped = Imported, but { $count } faces couldn't be read and were left out.
app-skp-empty = That SketchUp model is empty.
app-crash-title = WoodSketch ran into a problem
app-crash-note = It closed unexpectedly last time ({ $version }). A report helps get it fixed: it says what went wrong and where in WoodSketch's code, and you'll see all of it before it's sent.
app-crash-report = Report it...
app-crash-not-now = Not now
# What a drawing with no file of its own is called in the question below.
app-the-drawing = The drawing
app-file-changed-title = The file changed outside WoodSketch
app-file-changed = { $name } was changed by something else, and this drawing has changes that aren't saved. Only one of the two can be kept.
app-reload = Reload from the file
app-reload-hint = Take the file as it is now, losing the unsaved changes here (Ctrl+Z brings them back)
app-keep-mine = Keep mine
app-keep-mine-hint = Leave this drawing as it is; saving will write over the file
app-recover-title = Recover unsaved work?
app-recover-note = WoodSketch didn't close properly last time. It kept a copy of your unsaved changes:
app-recover-minutes = , { $minutes } minutes ago
app-recover-hours = , { $hours } hours ago
app-recover-line = { $name }{ $when }
app-recover = Recover
app-discard = Discard
app-discard-all = Discard all
app-build-sheets-title = { $name } build sheets
app-autosave-failed = Couldn't autosave: { $error }
# The headings on the printable pages, which open in a browser.
cutlist-sheet-title = Cut list
cutlist-build-sheets-title = Build sheets

## The status bar's left-hand side: what the tool in your hand is waiting for.
##
## These are read while working, so they are terse on purpose, and they name keys (Ctrl, Esc,
## W A S D) that are the same on every keyboard. Keep the key, translate around it.

status-pick-reference = Click the part's reference face (a wide face), edge (a long edge) or end. Esc stops.
status-select = Select objects. Ctrl adds, Shift toggles, Ctrl+Shift removes. Drag right for window, left for crossing selection.
status-orbit = Drag to orbit.
status-pan = Drag to pan.
status-zoom = Drag up to zoom in, down to zoom out.
status-walk = Walking: move the mouse to look; W A S D or the arrows walk (Shift: faster), E and Q go up and down, F flies, the wheel sets the speed; type an eye height. Walls don't stop you; the floor holds you up. Esc stops.

# What the Measurements box at the bottom right is asking for.
measure-dimensions = Dimensions
measure-angle = Angle
measure-length = Length
measure-radius = Radius
measure-bulge = Bulge
measure-scale = Scale
measure-distance = Distance
measure-text = Text
measure-eye-height = Eye height
measure-measurements = Measurements

# Inference: which axis the cursor has lined up with. The colors are the drawing's own -- red is
# X, green is Y, blue is Z, as in SketchUp -- so they are names here, not just colors.
infer-on-red = On Red Axis
infer-on-green = On Green Axis
infer-on-blue = On Blue Axis
infer-parallel = Parallel to Edge
infer-perpendicular = Perpendicular to Edge

tools-part-gone = That part is gone.
tools-click-part = Click on the part itself.
tools-click-part-reference = Click on the part itself to pick its reference face or edge.
tools-opened-group = Opened { $name } to work on it. Esc or click outside closes it.
tools-select-to-group = Select something to make a { $noun }.
tools-noun-group = group
tools-noun-component = component
tools-no-guides = There are no guides to delete.
tools-select-to-explode = Select a group or component to explode.
tools-select-to-copy = Select something to copy
tools-nothing-to-paste = Nothing to paste: copy something first

## What each tool says while you use it, and what it says when it can't do what was asked.
##
## `[x,y,z]`, `3x`, `/3`, `24s` and `5r` are typed into the Measurements box and are the same in
## every language: keep them exactly, and translate the words around them.

# Snapping: what the point under the cursor is.
snap-endpoint = Endpoint
snap-origin = Origin
snap-guide-point = Guide Point
snap-intersection = Intersection
snap-midpoint = Midpoint
snap-on-edge = On Edge
snap-on-guide = On Guide
snap-on-face = On Face

# Arc (two points and a bulge)
tool-arc-start = Select start point (or type [x,y,z]). Sides: { $sides } (type 24s to change).
tool-arc-end = Select end point or enter the chord length.
tool-arc-bulge = Select bulge, or enter it (5r for a radius).
tool-arc-click-start-end = Click the arc's start, then its end
tool-arc-radius-min = The radius must be at least half the chord ({ $half })
tool-arc-end-away = Pick an end point away from the start

# Center arc, and the three-point arc
tool-arc-center = Select the center. Sides: { $sides } (type 24s to change). Arrow keys lay it across an axis.
tool-arc-where-starts = Select where the arc starts, or type the radius.
tool-arc-turn-and-click = Turn and click where it ends, or type the angle.
tool-arc-start-away = Pick a start point away from the center
tool-arc-click-center-first = Click the center first
tool-arc-turn-away = Turn away from the start to sweep the arc
tool-arc-three-first = Select the arc's start. Sides: { $sides } (type 24s to change).
tool-arc-three-second = Select a point the arc passes through.
tool-arc-three-third = Select the arc's end.
tool-arc-second-away = Pick a second point away from the first
tool-arc-in-a-line = The three points are in a line: pick an end off it
tool-arc-three-points = Click three points: the start, one on the arc, the end. Type 24s for 24 sides.

# Circle and Polygon
tool-circle-sides = Sides must be a whole number from 3 to 999, like 24s
tool-circle-center-then-radius = Click the center, then type the radius
tool-circle-corner = Select a corner of the circle or enter its radius. Sides: { $sides } (type 8s to change).
tool-polygon-corner = Select a corner of the polygon or enter its radius. Sides: { $sides } (type 8s to change).
tool-circle-center = Select center point (or type [x,y,z]). Sides: { $sides } (type 32s to change). Arrow keys face it along an axis.
tool-polygon-center-to-corner = Select center point (or type [x,y,z]). Sides: { $sides } (type 8s to change). Arrow keys face it along an axis. Radius to a corner (tap Ctrl: to a side).
tool-polygon-center-to-side = Select center point (or type [x,y,z]). Sides: { $sides } (type 8s to change). Arrow keys face it along an axis. Radius to a side's middle (tap Ctrl: to a corner).
tool-circle-through-point = Through point
tool-circle-move-away = Move away from the center, or type the radius
tool-circle-radius-positive = Radius must be greater than zero

# Eraser, Line, Offset, Paint Bucket
tool-eraser-status = Click or drag over edges, guides or groups to erase them.
tool-eraser-click = Click an edge, guide, group or component to erase it
tool-line-start = Select start point, or type [x,y,z].
tool-line-end = Select end point or enter value. Hold Shift or press an arrow key to lock a direction; Esc ends the line.
tool-line-length-positive = Length must be greater than zero
tool-line-click-start = Click the start point (or type [x,y,z]), then type the length
tool-offset-pick = Click a face to offset its outline (or select a face first).
tool-offset-move = Move in or out and click, or enter the offset distance.
tool-offset-click-face = Click a face to offset its outline
tool-offset-click-then-type = Click a face, then type the offset
tool-offset-move-first = Move the mouse in or out of the face first
tool-paint-status = Click a board to paint all of it with the material picked in the Materials panel. Ctrl-click: one face; Alt-click: sample.
tool-paint-click = Click a group, component or face to paint it.

# Push/Pull
tool-pushpull-pick = Pick a face to push or pull. Ctrl keeps the face. Double-click a face to repeat the last distance.
tool-pushpull-keeping-pick = Keeping the face (Ctrl): pick a face; a new face and walls go from it.
tool-pushpull-move = Move to set the distance, then click, or enter a value. Ctrl keeps the face.
tool-pushpull-keeping = Keeping the face: a new face and walls. Move to set the distance, then click, or enter a value.
tool-pushpull-click-then-type = Click a face, then type the distance
tool-pushpull-standard-thickness = Standard thickness
tool-pushpull-in-group = That face is part of a group. Double-click the group to edit inside it, then push/pull.

# Rectangle, and the rotated rectangle
tool-rect-first = Select first corner, or type [x,y,z].
tool-rect-opposite = Select opposite corner or enter value.
tool-rect-square = Square
tool-rect-golden = Golden section
tool-rect-enter-wh = Enter width and height, like 1.5",3.5"
tool-rect-dims-positive = Dimensions must be greater than zero
tool-rect-click-first = Click the first corner, then type the size
tool-rotrect-first = Select the first corner.
tool-rotrect-side = Select the second corner (the first side), or type its length.
tool-rotrect-across = Move across and click, or type the other side's length.
tool-rotrect-second-away = Pick the second corner away from the first
tool-rotrect-click-first = Click the first corner
tool-rotrect-move-across = Move across the first side to give the rectangle its width

# Scale
tool-scale-pick-grip = Click a grip: corners scale evenly, edges two ways, faces one way.
tool-scale-move-center = Move to scale, or type a factor (2, 0.5) or a size (36"). About the center (tap Ctrl to anchor the opposite side). Hold Shift to switch even and stretched.
tool-scale-move-opposite = Move to scale, or type a factor (2, 0.5) or a size (36"). Tap Ctrl to scale about the center. Hold Shift to switch even and stretched.
tool-scale-click-grip = Click a scale grip, then type the scale
tool-scale-hover-grips = Click one of the scale grips (hover something to see them)
tool-scale-positive = Scale must be greater than zero (mirroring isn't supported yet)
# $value is what was typed, kept as it was typed.
tool-scale-not-a-factor = "{ $value }" isn't a scale factor
tool-scale-one-or-n = Enter one value, or { $count } separated by commas

# Ctrl+D with nothing picked.
tool-nothing-to-duplicate = Select something to duplicate

# Tape Measure
tape-note-guide-line = + Guide Line
tape-note-guide-point = + Guide Point
tape-note-measure = Measure
tool-tape-guide-lines = Create Guide Lines: click an edge to offset a guide from it, or a point. Tap Ctrl to change mode.
tool-tape-guide-points = Create Guide Points: click where to measure from. Tap Ctrl to change mode.
tool-tape-measure-mode = Measure: click where to measure from, or hover an edge. Type a length after measuring to resize the model. Tap Ctrl to change mode.
tool-tape-move-away = Move away from the edge; click or type the guide's offset.
tool-tape-click-to = Click the point to measure to.
tool-tape-click-to-or-type = Click the point to measure to, or type the distance.
tool-tape-show-direction = Move the mouse to show the direction, then type the distance
tool-tape-measure-first = Measure a distance, then type the length it should be
tool-tape-distance-positive = Distance must be greater than zero
tool-tape-click-from = Click where to measure from, then type the distance

# Dimensions and text notes
tool-dim-first = Pick the first point to measure from, or click an edge to dimension it.
tool-dim-second = Pick the second point.
tool-dim-place = Move out to place the dimension, then click, or type how far out.
tool-dim-second-away = Pick a second point away from the first
tool-dim-two-points = Pick two points, then type how far out the dimension goes
tool-note-pick-point = Pick the point the note is about.
tool-note-where-words = Click where the words go (the same spot: no leader line).
tool-note-type-words = Type the note's words and press Enter, or pick a point for another note.
tool-note-place-first = Place a note (click a point, then where the words go), then type its words

# Move. The copying ones are whole sentences rather than a sentence with "(copying)" dropped
# into it, because where that note belongs is a question about the language.
tool-move-base = Pick a base point to move (or type [x,y,z]). Tap Ctrl to copy; type 3x or /3 after a copy for an array.
tool-move-base-copying = Pick a base point to move (copying) (or type [x,y,z]). Tap Ctrl to copy; type 3x or /3 after a copy for an array.
tool-move-destination = Pick the destination, or enter a distance. Arrow keys lock an axis; tap Ctrl to copy.
tool-move-destination-copying = Pick the destination, or enter a distance (copying). Arrow keys lock an axis; tap Ctrl to copy.
tool-move-pasting = Click to put down what was pasted (it snaps to points), or type a distance. Esc takes it out.
tool-move-copy-first = Copy something first (tap Ctrl while moving), then type 3x or /3
tool-move-arrays-need-copy = Arrays need a copy: tap Ctrl while moving
tool-move-click-base = Click a base point, then type the distance
tool-move-select = Select something to move, or start on it
tool-move-away-from-base = Move the mouse away from the base point first
tool-move-vector = Enter a vector as x,y,z
tool-move-show-direction = Move the mouse to show the direction, then type the distance

# Rotate, and the Protractor
tool-rotate-center = Pick the center of rotation. Arrow keys lock the protractor (Right red, Left green, Up blue); tap Ctrl to copy.
tool-rotate-center-copying = Pick the center of rotation (copying). Arrow keys lock the protractor (Right red, Left green, Up blue); tap Ctrl to copy.
tool-rotate-start = Pick a point to start the rotation.
tool-rotate-start-copying = Pick a point to start the rotation (copying).
tool-rotate-end = Pick the end of the rotation, or enter an angle.
tool-rotate-end-copying = Pick the end of the rotation, or enter an angle (copying).
tool-protractor-center = Pick the protractor's center (a guide line goes through it). Arrow keys lock it (Right red, Left green, Up blue).
tool-protractor-start = Pick a point for the starting direction.
tool-protractor-direction = Pick the guide's direction, or enter an angle.
tool-rotate-away-from-center = Pick a point away from the center
tool-rotate-copy-first = Rotate a copy first (tap Ctrl while rotating), then type 3x or /3
tool-rotate-arrays-need-copy = Arrays need a copy: tap Ctrl while rotating
tool-rotate-pick-start-dir = Pick the starting direction, then type the angle
tool-rotate-pick-center-dir = Pick a center and a starting direction, then type the angle
tool-rotate-select = Select something to rotate, or start on it
tool-rotate-turn-away = Turn away from the starting direction first
tool-rotate-invalid-angle = Invalid angle: "{ $typed }"

# The Board tool. $stock is the material and size picked in the Board panel ("Pine Lumber 2x4").
tool-board-no-material = Board: add a wood material (Materials panel, +) to draw boards from its stock.
tool-board-lumber-start = { $stock }: click where the board starts. Type 2x4x96, or 3,6 1/2 for a custom size. Ctrl stands it on edge.
tool-board-sheet-start = { $stock }: click a corner of the board.
tool-board-lumber-end = Click where it ends, or type the length. Arrows lock an axis. W mirrors the board.
tool-board-sheet-end = Click the opposite corner, or type length,width. W mirrors the board.
tool-board-add-material = Add a wood material first (Materials panel, +), then pick its stock in the Board panel.
tool-board-length-first = Pick a length first: move away from the start.
tool-board-no-size = That board has no size.
tool-board-pick-stock = Pick stock in the Board panel first.
tool-board-click-start = Click where the board starts, then type its length (or type 2x4x96).
tool-board-length-positive = The length must be greater than zero.
tool-board-show-direction = Move the mouse the way the board should run, then type its length.
tool-board-length-width = Type length and width, like 30,11 1/4
tool-board-no-thickness = That size has no thickness or width.
tool-board-pick-sheet = Pick a sheet or solid-wood material in the Board panel first.

## The viewport: what is drawn over the drawing, and the right-click menu

viewport-editing = Editing  { $path }    Esc or click outside to close
viewport-walk-speed = Walking speed: { $speed } a second
viewport-flying = Flying: nothing holds you up. E and Q go up and down; F walks again.
viewport-walking = Walking: back on the floor.
viewport-grain = Grain
viewport-reference-face = Reference face
viewport-reference-edge = Reference edge
viewport-from-point = From Point
viewport-make-unique = Make Unique
viewport-make-unique-hint = Its own copy, so changing it leaves the other copies alone
viewport-reverse-faces = Reverse Faces
viewport-reverse-faces-hint = Turn the selected faces over, front to back
