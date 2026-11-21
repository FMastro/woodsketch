# Grain, and why wood moves

If you read one page of this guide, read this one. Almost every piece of furniture that fails
in its first year fails here, and it fails for a reason you can calculate in advance.

## Wood is not a material, it's a bundle of straws

A tree moves water up its trunk through long cells running the length of the trunk. Dried and
sawn into boards, those cells are still there, still running the length of the board. That's
the grain.

It has one consequence that drives everything else: **wood barely changes length along the
grain, and changes a great deal across it.** Along the grain the movement is so small you can
design as though it were zero. Across the grain a wide board in a centrally heated house will
shrink and swell by enough to split itself, or whatever it's screwed to, every single year.

Not once. Every year, in both directions, forever.

{{#include ../diagrams/across-not-along.svg}}

*Along the grain a board barely changes. Across it, a wide board in a heated house moves*
*enough to split itself or whatever it is fastened to — every year, in both directions.*

## Why it moves: moisture, not weather

Wood is hygroscopic — it takes up and gives off water vapour until it's in balance with the air
around it. That balance point is its **equilibrium moisture content**, and it tracks the
*relative humidity* of the room, not the temperature and not the rain.

In a heated house the air is dry in winter and damp in summer. Most shops and most homes swing
through something like **4 percentage points of moisture content** across the year — drier in a
cold climate with hard winter heating, less in a mild damp one. That 4% is WoodSketch's
default, and you can change it per drawing in **Cut List > Stock & kerf**.

Kiln-dried lumber arrives at 6–8%. It has not finished moving. It will come to terms with
*your* shop, which is why you let boards sit a week before milling them.

## How much, exactly

Each species has two coefficients — the fraction of its width it gains or loses per 1% change
in moisture content. There are two because wood doesn't move equally in both directions across
the grain:

- **Tangential**, along the growth rings. This is the figure for a **flat-sawn** board, the
  ordinary kind, where the rings run roughly parallel to the face.
- **Radial**, across the rings. This is the figure for a **quartersawn** board, where the rings
  run roughly perpendicular to the face.

**Radial movement is always the smaller of the two**, and for most species it's somewhere
around half the tangential figure — a third of it in pine, but nearer three quarters in birch
and walnut, so it is worth looking up rather than assuming. That difference is the real reason
quartersawn stock is prized for wide panels and door frames, and why it's worth paying for when
a part has to stay flat and stay put. The figures for every species are in
[Wood movement by species](../reference/wood.md).

{{#include ../diagrams/flat-sawn-and-quartersawn.svg}}

*The same log, two ways to saw it. Flat-sawn is the ordinary board and the one that moves;*
*quartersawn costs more and stays put, which is why wide panels and door frames are worth*
*buying it for.*

The arithmetic is just multiplication:

> **movement = width × coefficient × moisture swing**

Take a 24" wide red oak panel. Flat-sawn, red oak's tangential coefficient is 0.00369:

> 24" × 0.00369 × 4 = **0.354"**, about **3/8 inch**

The same panel quartersawn, radial 0.00158:

> 24" × 0.00158 × 4 = **0.152"**, about **5/32 inch**

Three eighths of an inch. Across the seasons, in a panel two feet wide, in a wood nobody thinks
of as especially lively. Every species WoodSketch knows is listed with both numbers in
[Wood movement by species](../reference/wood.md).

## What this breaks

Four classic failures, all the same mistake — a wide part held so it cannot move:

**A panel glued into a frame.** The panel shrinks, the frame doesn't, and either the panel
splits or it tears free of the glue line. The fix is as old as furniture: the panel **floats**
in a groove, glued nowhere, sized so there's room to shrink and room to swell.

{{#include ../diagrams/panel-trapped-or-floating.svg}}

*The oldest fix in furniture, and the pattern behind all four: the part is not made stronger,*
*it is **allowed to move** in the one direction it insists on moving.*

**A tabletop screwed down hard to an apron.** The top wants to shrink across its width; the
apron, running along its own grain, does not. Something gives, usually the top. The fix is
slotted screw holes, buttons, or figure-8 fasteners — a fixing that holds the top down but lets
it slide.

**A breadboard end glued along its length.** The end is cross-grain to the top. Glue the middle
tenon, peg the outer ones through elongated holes, and let the rest move.

**Wide boards trapped between two fixed points** — a shelf dadoed into both sides at full
depth with glue all along, a cabinet back glued into rabbets on four sides.

Notice what the fixes have in common. The joint is not made stronger. The part is **allowed to
move**, in the direction it insists on moving, and held in the other two.

## What WoodSketch does about it

The model knows enough to check this for you.

**Species.** A material made from the library carries its movement coefficients. One you make
yourself can borrow them: in the Materials panel, set *"Moves like"* to the nearest species.
Materials that aren't wood — MDF, hardboard, glass, steel — don't move and aren't checked.

**Grain direction.** Every part has one. By default it runs along the part's longest side,
which is right most of the time; set it yourself in **Entity Info** when it isn't. The grain
arrow on a selected part shows which way it's running. Getting this wrong matters twice: it
decides which dimension moves, and it decides how the part is laid out on the stock in the
cutting diagrams.

**Flat-sawn or quartersawn.** Also in Entity Info, per part, and it picks which of the two
coefficients applies.

**The seasonal swing.** One number for the drawing, 4% by default.

**The check.** **Cut List > Wood movement** lists every solid-wood part, how far it will move
across its grain, and — the useful part — every place where **two touching parts disagree**:
one wants to move, the other won't let it. **View > Wood Movement** draws the same thing as
arrows in the viewport, which is often where you see the problem rather than read it.

**What's already allowed for.** A panel let into a groove is understood to be floating and
isn't reported. Neither is a part you've marked **free to move** in Entity Info, which is how
you tell the drawing "there are slotted screws here". Drawers and doors are checked within
themselves, not against the case they sit in.

A clean movement check is not a promise the piece will survive. It's the promise that you
haven't made one of the four mistakes above by accident.

## The rules of thumb

If you remember nothing else:

1. **Wood moves across the grain, not along it.** Design so that cross-grain parts never
   constrain each other.
2. **Wide means trouble.** Under about 4", movement is usually inside the slop of the joint.
   Over 12", plan for it explicitly.
3. **Quartersawn moves half as much** as flat-sawn. Spend the money where flatness matters.
4. **Plywood and MDF don't move.** They also don't look like solid wood. Use that.
5. **Glue a panel nowhere, or glue it in the middle only.** Never along two opposite edges.
6. **Let the wood come to the shop first.** A week, at least, before you mill.

## Next

- [The reference face and edge](reference-face-and-edge.md) — how a part's faces get named, and
  why every cut is measured from the same two.
- [Choosing a joint](choosing-a-joint.md) — several of these decisions are really movement
  decisions.
