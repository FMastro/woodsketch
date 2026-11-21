# Stock, the cut list and what to buy

A cut list is the model turned back into a shopping trip and a morning at the saw. Getting it
right is mostly a matter of telling the drawing what you can actually buy.

## What counts as a part

A part is a **group or component that is visible and has faces**. That's the whole rule.

Which means: loose geometry isn't a part, a hidden part isn't a part, and anything you want
counted needs to be grouped. It also means the cut list is a good check on the model — if
something is missing from it, it isn't a part yet.

Two things are deliberately excluded:

- **Hardware.** A part whose material is Hardware, or a component marked as a placed piece of
  hardware — hinges, knobs, slides — is counted and bought, never cut. It appears in the
  hardware list instead of the parts list.
- **Flat things under 1/64" thick**, which is how imported SketchUp models avoid listing every
  decorative surface as a board.

Parts are grouped by **material, size and name**, so a pair of identical sides is one row with
a count of 2. Each row gets a letter — A, B, C — and that letter follows the part through the
cutting diagrams, the build sheets and the assembly views.

## Stock: what you can actually buy

Wood isn't sold in the sizes your parts are. It's sold in a handful of sizes per species, and
the cut list's job is to get your parts out of those.

Each material carries the standard stock it comes in, which differs by type:

- **Sheet goods** — the thicknesses it's made in, and the sheet sizes. 4' × 8' usually; Baltic
  birch is 5' × 5' and comes thinner, which the library already knows.
- **Dimensional lumber** — sections (a 2x4 is actually 1 1/2" × 3 1/2") and the lengths it's
  sold in. 5/4 decking and 6x material are in there too.
- **Solid wood** — the thicknesses it's milled to, and the lengths and widths boards come in.

The defaults are US-typical. Change them for the drawing in **Cut List > Stock & kerf**, or
per material in the **Materials** panel — which is the right place when one species comes in
different sizes from the rest. Stock settings are saved *with the drawing*, so a project keeps
the sizes you planned it around.

**Kerf** is the width your saw blade removes, and it belongs here because the diagrams can't
pack honestly without it. It's saved with the drawing too.

### Picking stock for a part

WoodSketch works out each part's stock from the part's own size. A 1 1/2" × 3 1/2" × 84" part in
Pine Lumber is a stud, so it's listed as a **2x4, 8'**, not as a mysterious rectangle. A 3/4"
part in Red Oak is cut from 3/4" boards.

That's why getting the thickness right matters: a part drawn at 0.72" isn't 3/4" stock and won't
be recognised as any.

## The cutting diagrams

**Cut List > Cutting diagrams** packs your parts onto real stock and draws the result.

**Sheet goods and solid wood** are packed in two dimensions as **guillotine cuts** — every cut
goes right across the piece it's cutting, because that's what a table saw and a track saw
actually do. A packing that needs a cut to stop halfway is no use in a shop.

**The grain is respected.** A part's length runs along its grain, and a sheet's grain runs along
its length, so parts are laid out the right way round rather than rotated for a tighter fit.
A tighter diagram that turns a cabinet side cross-grain is a worse diagram.

**Lumber** is packed in one dimension along each standard length, and boards are split where
shorter ones cost no more — there's no point buying a 12-footer for parts that come out of two
8-footers.

**Materials that aren't wood** — glass, steel, concrete, anything marked Other — are listed in
the parts but left out of the diagrams, with one line naming them. They're bought, not cut from
stock.

### Glue-ups

A solid-wood part wider than the widest board you can buy has to be glued up from narrower
ones. WoodSketch does this automatically when it must, aiming for boards around 7 1/4" or
narrower — narrow enough to be stable and to be stock you can actually find.

The strips appear in the diagrams as B-1, B-2 and so on, and a glue-up step appears on the
part's build sheet. You can override it in **Entity Info**: Auto, or a specific number of
boards from 1 to 5. Choosing it yourself is worth doing when the grain match matters, which on
a tabletop it does.

## The shopping list

Beside the diagrams is what to buy: how many sheets, how many boards of each section and
length, and the hardware count — every dowel, biscuit, screw, hinge and slide your joints and
templates called for.

This is the number that decides whether the design is affordable, so it's worth looking at
before cutting anything rather than after.

## Taking it to the shop

- **CSV** from the Parts tab, for a spreadsheet.
- **Print cut sheet** — one HTML page with the parts, what to buy and the diagrams, which opens
  in your browser and prints.
- **Print build sheets** — a page per part with its views and every cut placed from its
  [reference face and edge](reference-face-and-edge.md), then the assemblies in glue-up order.
- **Export** the geometry as OBJ or STL, or each part flattened as DXF or SVG for a CNC.

## Two habits worth having

**Check the cut list early.** It is the fastest way to find out that a part isn't grouped, a
material isn't set, or a thickness is 0.72". Look at it when the carcase is drawn, not when the
design is finished.

**Check wood movement before you buy.** It's the fourth tab, it takes a second, and it is far
cheaper to widen a groove in the model than to re-make a panel. See
[Grain, and why wood moves](grain-and-movement.md).

## Next

- [Materials and stock](../reference/materials.md) — what's in the library and the sizes each
  comes in.
- [Wood movement by species](../reference/wood.md)
