# Wood movement by species

<!-- Written by crates/ui-egui/tests/guide.rs from the app itself. Don't edit it by hand:
     change the app and run `cargo test -p woodsketch-ui-egui --test guide`, which rewrites it. -->

How much each species swells and shrinks **across** the grain, from the USDA Wood Handbook.
Along the grain the movement is small enough to design as though it were zero.

The two figures are the fraction of its width a board gains or loses per 1% change in moisture
content: the **flat-sawn** figure where the growth rings run roughly parallel to the face, the
**quartersawn** figure where they run across it. Quartersawn always moves less -- across the
species below it runs from 33% of the flat-sawn figure to 76%, about 55% for the middle
species -- and that stability is what you are paying for.

> movement = width x coefficient x moisture swing

The last two columns work that out for a 24" board over a 4% swing in moisture content, which
is the default for a drawing and about what a heated house does across a year. Change it in
**Cut List > Stock & kerf**.

What to do about all this is [Grain, and why wood moves](../concepts/grain-and-movement.md).

| Species | Flat-sawn | Quartersawn | A 24" board moves, flat-sawn | ...and quartersawn |
|---|---|---|---|---|
| **Red Oak** | 0.00369 | 0.00158 | ~ 3/8" | ~ 1/8" |
| **White Oak** | 0.00365 | 0.00180 | ~ 3/8" | ~ 3/16" |
| **Hard Maple** | 0.00353 | 0.00165 | ~ 5/16" | ~ 3/16" |
| **Soft Maple** | 0.00289 | 0.00137 | ~ 1/4" | ~ 1/8" |
| **Cherry** | 0.00248 | 0.00126 | ~ 1/4" | ~ 1/8" |
| **Walnut** | 0.00274 | 0.00190 | ~ 1/4" | ~ 3/16" |
| **Poplar** | 0.00289 | 0.00158 | ~ 1/4" | ~ 1/8" |
| **Ash** | 0.00274 | 0.00169 | ~ 1/4" | ~ 3/16" |
| **Birch** | 0.00338 | 0.00256 | ~ 5/16" | ~ 1/4" |
| **Hickory** | 0.00411 | 0.00259 | ~ 3/8" | ~ 1/4" |
| **Mahogany** | 0.00172 | 0.00125 | ~ 3/16" | ~ 1/8" |
| **Cedar** | 0.00234 | 0.00111 | ~ 1/4" | ~ 1/8" |
| **Douglas Fir** | 0.00267 | 0.00165 | ~ 1/4" | ~ 3/16" |
| **Pressure-treated** | 0.00259 | 0.00165 | ~ 1/4" | ~ 3/16" |
| **Pine** | 0.00212 | 0.00071 | ~ 3/16" | ~ 1/16" |

## Where these come from, and where they go

A material made from the library carries its species' figures. One you make yourself can
borrow them in the Materials panel with *Moves like*; without them, it isn't checked.

Each part also carries which way its grain runs and how it was sawn (**Entity Info**), and the
sawing is what picks between the two columns above. A part whose sawing you haven't said is
assumed flat-sawn, which is the worse case.

**Cut List > Wood movement** then lists every solid-wood part's movement and every place two
touching parts disagree about it. **View > Wood Movement** draws the same thing as arrows in
the viewport. A panel floating in a groove, and a part you have marked *free to move*, are
understood to be allowed for and aren't reported.
