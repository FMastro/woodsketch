# Materials and stock

<!-- Written by crates/ui-egui/tests/guide.rs from the app itself. Don't edit it by hand:
     change the app and run `cargo test -p woodsketch-ui-egui --test guide`, which rewrites it. -->

A new drawing starts with no materials. You add them from the library -- the **Materials**
panel, or straight from the Board tool's dropdown, which adds whatever you pick.

A material is a **kind of stock, not one size**. Plywood comes in several thicknesses; Pine
Lumber comes in every standard section; the cut list works out which one each part needs from
the part's own size. The sizes below are the US defaults, and they are yours to change: for
the whole drawing in **Cut List > Stock & kerf**, or per material in the Materials panel.

Solid wood and lumber also carry how much the species moves -- see
[Wood movement by species](wood.md). A material you make yourself can borrow a species'
figures with *Moves like*.

| Material | Type | Grain | Sold as |
|---|---|---|---|
| <span class="ws-swatch" style="background:#c49268" title="#c49268"></span> | **Red Oak** | Solid wood | yes | 3/4", 1", 1 1/4", 1 3/4" thick; boards up to 11 1/4" wide, lengths 72", 96", 120" |
| <span class="ws-swatch" style="background:#cdac7a" title="#cdac7a"></span> | **White Oak** | Solid wood | yes | 3/4", 1", 1 1/4", 1 3/4" thick; boards up to 11 1/4" wide, lengths 72", 96", 120" |
| <span class="ws-swatch" style="background:#ecd6aa" title="#ecd6aa"></span> | **Hard Maple** | Solid wood | yes | 3/4", 1", 1 1/4", 1 3/4" thick; boards up to 11 1/4" wide, lengths 72", 96", 120" |
| <span class="ws-swatch" style="background:#b26642" title="#b26642"></span> | **Cherry** | Solid wood | yes | 3/4", 1", 1 1/4", 1 3/4" thick; boards up to 11 1/4" wide, lengths 72", 96", 120" |
| <span class="ws-swatch" style="background:#68462e" title="#68462e"></span> | **Walnut** | Solid wood | yes | 3/4", 1", 1 1/4", 1 3/4" thick; boards up to 11 1/4" wide, lengths 72", 96", 120" |
| <span class="ws-swatch" style="background:#cec896" title="#cec896"></span> | **Poplar** | Solid wood | yes | 3/4", 1", 1 1/4", 1 3/4" thick; boards up to 11 1/4" wide, lengths 72", 96", 120" |
| <span class="ws-swatch" style="background:#e4be88" title="#e4be88"></span> | **Pine** | Solid wood | yes | 3/4", 1", 1 1/4", 1 3/4" thick; boards up to 11 1/4" wide, lengths 72", 96", 120" |
| <span class="ws-swatch" style="background:#c4805c" title="#c4805c"></span> | **Cedar** | Solid wood | yes | 3/4", 1", 1 1/4", 1 3/4" thick; boards up to 11 1/4" wide, lengths 72", 96", 120" |
| <span class="ws-swatch" style="background:#d6bc8c" title="#d6bc8c"></span> | **Plywood** | Sheet good | yes | 1/4", 1/2", 3/4" thick; sheets 96" x 48" |
| <span class="ws-swatch" style="background:#dec28c" title="#dec28c"></span> | **Pine Plywood** | Sheet good | yes | 1/4", 1/2", 3/4" thick; sheets 96" x 48" |
| <span class="ws-swatch" style="background:#e8d6b2" title="#e8d6b2"></span> | **Baltic Birch Plywood** | Sheet good | yes | 1/8", 1/4", 1/2", 3/4" thick; sheets 60" x 60" |
| <span class="ws-swatch" style="background:#aa8c6c" title="#aa8c6c"></span> | **MDF** | Sheet good | no | 1/4", 1/2", 3/4" thick; sheets 96" x 48" |
| <span class="ws-swatch" style="background:#785a40" title="#785a40"></span> | **Hardboard** | Sheet good | no | 1/8", 1/4" thick; sheets 96" x 48" |
| <span class="ws-swatch" style="background:#e2c48c" title="#e2c48c"></span> | **Pine Lumber** | Dimensional lumber | yes | 1x2, 1x3, 1x4, 1x6, 1x8, 1x10, 1x12, 5/4x2, 5/4x3, 5/4x4, 5/4x6, 5/4x8, 5/4x10, 5/4x12, 2x2, 2x3, 2x4, 2x6, 2x8, 2x10, 2x12, 4x4, 4x6, 4x8, 4x10, 4x12, 6x6, 6x8, 6x10, 6x12; lengths 96", 120", 144", 192" |
| <span class="ws-swatch" style="background:#d6aa78" title="#d6aa78"></span> | **Douglas Fir Lumber** | Dimensional lumber | yes | 1x2, 1x3, 1x4, 1x6, 1x8, 1x10, 1x12, 5/4x2, 5/4x3, 5/4x4, 5/4x6, 5/4x8, 5/4x10, 5/4x12, 2x2, 2x3, 2x4, 2x6, 2x8, 2x10, 2x12, 4x4, 4x6, 4x8, 4x10, 4x12, 6x6, 6x8, 6x10, 6x12; lengths 96", 120", 144", 192" |
| <span class="ws-swatch" style="background:#c4805c" title="#c4805c"></span> | **Cedar Lumber** | Dimensional lumber | yes | 1x2, 1x3, 1x4, 1x6, 1x8, 1x10, 1x12, 5/4x2, 5/4x3, 5/4x4, 5/4x6, 5/4x8, 5/4x10, 5/4x12, 2x2, 2x3, 2x4, 2x6, 2x8, 2x10, 2x12, 4x4, 4x6, 4x8, 4x10, 4x12, 6x6, 6x8, 6x10, 6x12; lengths 96", 120", 144", 192" |
| <span class="ws-swatch" style="background:#beb478" title="#beb478"></span> | **Pressure-Treated Lumber** | Dimensional lumber | yes | 1x2, 1x3, 1x4, 1x6, 1x8, 1x10, 1x12, 5/4x2, 5/4x3, 5/4x4, 5/4x6, 5/4x8, 5/4x10, 5/4x12, 2x2, 2x3, 2x4, 2x6, 2x8, 2x10, 2x12, 4x4, 4x6, 4x8, 4x10, 4x12, 6x6, 6x8, 6x10, 6x12; lengths 96", 120", 144", 192" |

## The types, and what each one changes

| Type | How it is bought, cut and listed |
|---|---|
| **Solid wood** | Boards milled from rough stock. Has grain, moves with the seasons, listed with its thickness, packed two-dimensionally onto boards, and glued up from narrower ones when a part is wider than anything you can buy. |
| **Sheet good** | Plywood, MDF and the like: a fixed thickness, bought as sheets, packed as guillotine cuts. Doesn't move. |
| **Dimensional lumber** | Ready-milled stock like a 2x4, bought by section and length, packed along standard lengths. |
| **Hardware** | Hinges, screws, knobs, slides: counted and bought, never cut, and left out of the diagrams and the movement check. |
| **Other** | Glass, steel, concrete. Listed in the parts so nothing is forgotten, named in one line beside the diagrams, and not packed onto stock. |
