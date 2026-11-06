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
