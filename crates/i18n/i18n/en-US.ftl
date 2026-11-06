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
