# The reference face and edge

A rough board has six surfaces and not one of them is flat, straight or square to any other.
Milling it is the business of choosing one face and one edge to be *right*, and then measuring
everything from those two. That choice is the reference face and the reference edge, and it is
the oldest idea in machine woodworking.

WoodSketch carries the same idea into the model, for the same reason: so that every cut on a
part is dimensioned from the same two surfaces, and the number on the build sheet is the number
you set your fence to.

## Why it matters in the shop

Mill a board and the sequence is fixed:

1. **One face flat** — the jointer, or a planer sled. That is the reference face.
2. **One edge straight and square to it** — the jointer again, with the reference face against
   the fence. That is the reference edge.
3. **The second face parallel to the first** — the thicknesser, reference face down.
4. **The second edge parallel to the first** — the table saw, reference edge against the fence.

From then on, every setting is taken against those two surfaces: the reference face on the
table, the reference edge against the fence. Mark them — a pencil squiggle on the face, a caret
on the edge, pointing at it — and keep them oriented the same way at every machine.

The reason is simple arithmetic. A mortise 1/4" from the reference edge is 1/4" from the fence,
every time, on every copy of the part. A mortise measured from whichever edge happened to be
nearest is a mortise in a slightly different place on each one, and a frame that doesn't close.

## Why it matters in the model

Without a reference, "the dado is 6 inches along" raises the question: along from which end?
Looked at from the other side of the bench, the part is the same part and the answer is
different. That ambiguity is where hand-drawn plans and models fall out of step.

So WoodSketch asks you to nominate the surfaces, and then says everything from them. A part's
build sheet places each cut as:

> *3" from the reference end, 2 1/2" long, 1/4" in from the reference edge, 1/4" deep from the
> reference face*

Set those three on the part, orient the real board the way the sheet draws it, and the
measurements are ready to use without being translated.

## Setting them

Select a part, open **Entity Info**, and use **Pick** beside the reference face, edge or end.
The cursor then takes the surface you click on the part itself. The picked surfaces are drawn
on the selected part — an outline just inside the reference face, a line along the reference
edge, a line across the reference end, with the traditional marks.

You can set one, two or all three. Anything you don't set falls back to how the views are drawn
— the bottom face, the bottom edge, the left end — which is consistent, just arbitrary.

## Which surfaces to choose

The model doesn't care; the wood does. Choose the way you'd choose at the bench:

- **The reference face is the show face** where one side will be seen — the outside of a case
  side, the top of a tabletop. Then any error from milling ends up on the hidden side.
- **The reference face is the flattest face** when neither side shows.
- **The reference edge is the edge the joints are measured from**, and ideally the one that is
  straightest. On a rail, that's the edge the mortise is laid out from. On a cabinet side, it's
  usually the front edge, because the front is what has to line up.
- **The reference end** is the end you'll register against a stop when cutting several parts to
  the same length.
- **On matched parts, pick the same surfaces** — but remember a pair of cabinet sides are
  *mirror images*, not copies. The reference face is the outside of each, so they are opposite
  faces of the same board shape. WoodSketch handles mirrored placements of a shared component
  properly, so one component turned the other hand keeps its joinery and its grain.

## How it reaches the build sheets

The reference surfaces are what the **build sheets** measure from. Each part gets a page with
face, edge and end views drawn to scale, and every joint cut on it placed from the reference
face, edge and end.

The same wording appears twice: on the part's own sheet, and in the assembly step that makes
that joint. Both come from the same calculation, so they cannot disagree — which was the whole
point. The feature exists because a hand-drawn joinery layout and a model disagreed, and the
hand-drawn one was trusted.

## Next

- [Choosing a joint](choosing-a-joint.md)
- [Stock, the cut list and what to buy](cut-list-and-stock.md)
