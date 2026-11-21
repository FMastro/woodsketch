# Your first project

A bookcase: two sides, a top and bottom, three shelves, the shelves let into dados. Half an
hour, and at the end you will have a cut list, a shopping list and a sheet for every part.

If a key or a panel name here doesn't match what you see, trust the app and tell us — see
[WoodSketch](introduction.md) for how.

## 1. A new drawing, in the units you work in

WoodSketch opens on the Start page. **New** offers inches or millimetres, and the choice
belongs to the drawing, not to the program — a drawing made in millimetres stays in
millimetres on anyone's machine. You can change it later with **View > Units**.

Everything below is written in inches. Type `38mm` anywhere and it will be understood anyway.

## 2. Draw the first board

Press **W** for the Board tool. The **Board** panel appears at the top of the tray on the
right. It asks two things:

- **Material.** The dropdown groups the drawing's own woods first, then the library's.
  A new drawing has none of its own, so pick **Red Oak** under *Solid wood* — choosing it from
  the library adds it to the drawing.
- **Size.** The standard stock that material comes in. Pick 3/4".

Now drag in the viewport. For solid wood and sheet goods the Board tool drags out a rectangle
at the thickness you picked, so you are drawing the face of the board and its thickness is
already decided. Drag roughly, then type the real size and press **Enter**:

```
36x11 1/4
```

You never have to drag accurately. **Type the number you mean** — that is the habit worth
building on day one. A comma works as a decimal point if that's how you write them
(`38,5`), and a semicolon separates two lengths when a comma would be ambiguous.

That board is a **group**: a part, not loose geometry. Everything that will appear in the cut
list is a group or a component.

## 3. The rest of the carcase

Draw the second side the same way, or — better — select the first and press **Ctrl+C**,
**Ctrl+V**. A pasted part comes up *carried on the cursor*: move it where it goes and click to
put it down. **Esc** throws it away.

Press **M** for Move. Move locks to an axis when you push the cursor along one — red, green or
blue — and you can force it with the arrow keys. Type a distance to move exactly that far.

For the top and bottom, the Board tool again at 36" long by 11 1/4" wide. Three shelves the
same, a touch narrower if you want a reveal at the front.

Don't worry about the joints yet. Put the parts where they will sit when the joints are cut:
**shelves butted up against the inside of the sides**, not overlapping them. How the parts sit
is how WoodSketch works out which joints are possible — see
[Choosing a joint](concepts/choosing-a-joint.md).

## 4. Name the parts

Select a part and open **Entity Info** in the tray. The name you give it is the part name, and
it is what the cut list, the diagrams and the build sheets will call it. `Side`, `Shelf`,
`Top`. Parts with the same name, material and size are counted together, so both sides
become "Side, 2 off".

While you are there: Entity Info also shows the part's **grain** direction and lets you set
its **reference face and edge**. Both matter later, and both have a page of their own
— [grain](concepts/grain-and-movement.md) and
[the reference face](concepts/reference-face-and-edge.md).

## 5. Cut the joints

Select exactly **two** parts — a shelf and the side it meets — and the **Joinery** panel
appears in the tray. It offers only the joints that suit how those two parts actually sit, so
a shelf butted into a side offers **Let in**, which cuts a dado in the side and grows the
shelf to fill it.

Set the depth (1/4" into a 3/4" side is normal), and make it. One undo step; **Ctrl+Z** takes
the whole joint back out.

Then do the other end. If the sides are copies of one component, a dado cut on one is
recognised on the other, so the pair is cut once.

The full list of joints, and what each one needs, is in [Joints](reference/joints.md).

## 6. The cut list

**Cut List** on the toolbar. Four tabs:

- **Parts** — every part, grouped by material and size, lettered A, B, C. Plus the hardware:
  the screws, dowels, biscuits and hinges your joints called for.
- **Cutting diagrams** — the parts packed onto real stock, with the grain running the right
  way, and a shopping list of what to buy. Sheets are packed as guillotine cuts; boards are
  packed along standard lengths.
- **Stock & kerf** — the sizes you can actually buy and the width of your saw blade. These are
  saved with the drawing, and a material can override them.
- **Wood movement** — how much each solid-wood part will move across its grain through the
  year, and anywhere two parts will fight each other over it.

Export the parts as CSV, or **Cut List > Print cut sheet** for a page you can take to the shop.

## 7. Build sheets

**File > Print Build Sheets**, or the button in the Cut List window.

One page per part: its finished size, its stock, which way the grain runs, scale drawings of
the face, edge and end, and every cut marked out from its reference face and edge. After the
parts come the assemblies, in glue-up order, each with an exploded view.

This is the part that is hard to do by hand and easy to get wrong by hand. Every measurement
on a build sheet is taken from the model, so the dado on the side's sheet and the dado in the
assembly step are the same number by construction — they cannot disagree.

## Where next

- [Grain, and why wood moves](concepts/grain-and-movement.md) — the one bit of theory that
  will save you a cracked panel.
- [Templates](reference/templates.md) — a whole cabinet, door, drawer, face frame or table
  from a few sizes, instead of board by board.
- [Tools and keys](reference/tools.md) — the full set.
