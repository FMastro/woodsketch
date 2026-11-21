# Choosing a joint

There are a dozen ways to join two boards and most of them will hold. Choosing between them is
a matter of what the joint has to do, what it has to look like, and what machines you own —
roughly in that order.

This page is about that choice. The complete list of what WoodSketch can cut, and what each one
needs, is in [Joints](../reference/joints.md).

## First: how are the parts sitting?

WoodSketch reads how the two selected parts actually sit in the model and offers only the
joints that suit it. So the first thing to get right is the arrangement, not the joint.

| How they sit | What it means | What's offered |
|---|---|---|
| **Butted** | One part's end or edge meets the other's face, flush | Let in, mortise and tenon, dowels, biscuits, loose tenons, splines, pocket screws |
| **Butted at a corner** | End meets face at the end of the other part | the above, plus dovetails |
| **Crossing at a corner** | Two ends meet at a corner | box joint |
| **Crossing, overlapping** | They cross each other, each one's face overlapping | half-lap |
| **Overlapping** | One part sits into the volume of the other | notch, mortise |
| **Not square to each other** | Turned about a shared axis | angled mortise and tenon, housing |

If a joint you want isn't offered, the parts aren't where that joint needs them. Move them
together until they are and the panel will offer it. For a let-in joint, that means pushing the
shelf **up against** the inside of the side, not overlapping it — WoodSketch grows the shelf
into the dado it cuts.

## Second: what does the joint have to do?

Four questions, and they're usually enough.

**Is it carrying load, and which way?** A shelf carries weight downward, which a dado resists
beautifully — the load pushes the shelf into solid wood. A chair rail is pulled and racked in
every direction, and nothing beats a mortise and tenon. A drawer front is pulled, repeatedly,
by the one person who yanks it, which is what dovetails are for and why they survived.

**Is it cross-grain?** If one part's grain runs across the other's, the joint has to allow
movement or something will split. A panel goes in a **groove** and floats, glued nowhere. A
wide shelf glued into a dado along its whole length is cross-grain and will fight; glue the
front few inches and let the rest float. See
[Grain, and why wood moves](grain-and-movement.md).

**Will it be seen?** Dovetails are shown off. Pocket screws are hidden — inside a cabinet,
under a tabletop, behind a face frame — and there's no shame in them there. Through tenons and
splines are a deliberate statement. Biscuits and dominoes are invisible by design.

**What do you own?** This decides more than anybody admits. A joint you can't cut accurately is
weaker than a simpler one you can. A domino machine makes loose tenons trivial; without one,
dowels do the same job for the price of a jig. A dovetail jig changes which drawer joint is
reasonable. There is no virtue in the harder joint.

## The joints, and when each one earns its place

**Dado, groove, rabbet, notch — "let in".** A recess in one part that the other sits into.
Shelves into case sides, backs into rabbets, panels into grooves. Strong in the direction that
matters, self-aligning during glue-up, and dead simple. The default for casework, and WoodSketch
grows the let-in part to fill the recess so the outside dimension doesn't change.

**Mortise and tenon.** The frame joint. Rails into legs, door frames, anything that gets racked.
Enormous glue area on long grain and a mechanical interlock. Haunched when it meets a grooved
frame member, through when you want it seen, angled when the parts aren't square.

**Dovetails.** Mechanically locked against being pulled apart, so: drawer fronts, carcase
corners. At 1:8 for hardwood. The joint people judge your work by, which is both why it's worth
cutting and why it's worth cutting on a jig if hand-cutting isn't your hobby.

**Box joint.** Dovetails' honest cousin — square fingers, no slope, cut on a table saw with a
simple jig. Nearly as strong because the glue area is nearly the same. Drawer boxes, shop
fixtures, anywhere it doesn't need to be a heirloom.

**Half-lap.** Two crossing parts each cut to half their thickness so they finish flush. Frames,
web frames, shop-built jigs, anything where two members cross in a plane.

**Loose tenons and biscuits.** A slot in both parts and a separate piece glued into both. All
the alignment and most of the strength of a mortise and tenon, in a fraction of the time, if you
own the machine. WoodSketch cuts the slot shallower into a face than into an end — two thirds of
the way into the face, the rest in the end — and rounds the depths to whole millimetres, because
2/3 of 3/4" is 12.7mm and no joiner has a stop there. Give it your machine's actual depth stops
and it will pick a pair that holds the tenon.

**Dowels.** The same idea with round pegs. Cheaper to get into, fussier to align, perfectly
sound. Still the right answer for a lot of face-frame and carcase work.

**Pocket screws.** A screw driven at a shallow angle through a bored pocket. Fast, strong
enough for face frames and cabinet carcases, needs no clamps and no glue-up time. Hide them.
A joint you'd use on twelve kitchen cabinets and never on a jewellery box.

**Splines.** A thin strip in a groove in both parts. Mitres, long-grain edge joints, and
anywhere you want alignment plus a decorative line.

## A rough default

If you're stuck, these are rarely wrong:

| Situation | Start with |
|---|---|
| Shelf into a case side | Dado |
| Case back, cabinet bottom | Rabbet |
| Door or frame corner | Mortise and tenon |
| Panel in a frame | Groove, floating |
| Drawer front to side | Dovetails, or box joint |
| Drawer back to side | Dado, or box joint |
| Face frame | Pocket screws, or dowels |
| Tabletop to apron | Buttons or slotted screws — *not* a glued joint |
| Edge-joining boards for a panel | Plain glue line; splines or biscuits for alignment only |
| Two parts crossing in a plane | Half-lap |

## What WoodSketch does with it

Making a joint cuts **both** parts, in one undo step, and remembers it. That matters more than
it sounds:

- It's recorded, so **Change** re-opens the sizes and **Remove** rebuilds the parts without it.
- The hardware adds up. Dowels, biscuits, screws and hinges land in the cut list's hardware
  list, so the shopping list includes them.
- Copies of a shared component are recognised: cut a mortise on one leg of a four-leg component
  and the design is repeated rather than applied twice.
- The cuts appear on the build sheets, placed from each part's
  [reference face and edge](reference-face-and-edge.md), and in the assembly sequence in
  glue-up order.
- Fit allowances are yours to set — a tenon, a dowel or a loose tenon can be cut to a
  deliberate fit rather than nominal size.

If you change a part by hand after making a joint on it, **Remove** will refuse rather than
guess, because it can no longer be sure what it would be undoing.

## Next

- [Joints](../reference/joints.md) — the generated list of everything available.
- [Stock, the cut list and what to buy](cut-list-and-stock.md)
