#!/usr/bin/env python3
"""Generate the Small house sample recipe (samples/house.json).

A stud-framed 2 bed / 2 bath house with a front porch. The layout is arithmetic -- studs 16" on
centre, sheets cut to fit, rafters at a 6:12 pitch -- so it is derived here rather than typed out
by hand. Everything placed more than once is a copy of one part, which is what makes the drawing
cheap to open and cheap to draw.

Edit this, never `samples/house.json`: the JSON is what this prints.

Run from the repository root:
    python -I scripts/make-house-sample.py > samples/house.json
"""

import json
import math
import sys

# ---------------------------------------------------------------- the plan, in inches

W, D = 384.0, 288.0          # 32' x 24' footprint
EXT = 5.5                    # 2x6 exterior walls
INT = 3.5                    # 2x4 interior walls
STUD_H = 92.625              # a precut stud: an 8'1-1/8" wall with one bottom and two top plates
PLATE = 1.5
WALL_TOP = PLATE + STUD_H + 2 * PLATE        # 97.125
STUD_TOP = PLATE + STUD_H    # underside of the top plates: where a cripple run stops
BLOCK_Z = 48.0 - PLATE / 2   # blocking centred on the drywall's horizontal seam
OC = 16.0                    # studs and joists on centre
PITCH = 6.0 / 12.0
RIDGE_Y = D / 2.0
RISE = RIDGE_Y * PITCH       # 72" over a 12' run
SLOPE = math.degrees(math.atan(PITCH))       # 26.565 degrees
EAVE = 12.0
OSB = 0.4375
BOARD = 0.5                  # 1/2" drywall
Y1 = 168.0                   # the bearing wall across the house

PORCH_W, PORCH_D, PORCH_X = 144.0, 72.0, 96.0
DECK_Z = -1.5

DOOR_H, WIN_H, SILL = 82.0, 48.0, 36.0

parts: list[dict] = []
tags: list[dict] = []
templates: list[dict] = []
openings: list[dict] = []    # every hole in every wall, for framing and for sheeting round


def r(v):
    v = round(float(v), 3)
    return int(v) if v == int(v) else v


def part(name, material, size, at, length="x", width="y", tag=None, copies=None, **rest):
    p = {"name": name, "material": material, "size": [r(v) for v in size], "at": [r(v) for v in at]}
    if length != "x":
        p["length"] = length
    if width != "y":
        p["width"] = width
    if tag:
        p["tag"] = tag
    p.update(rest)
    if copies:
        p["copies"] = copies
    parts.append(p)


def copies_at(label, positions):
    return [{"name": f"{label} {i + 2}", "at": [r(p[0]), r(p[1]), r(p[2])]} for i, p in enumerate(positions)]


def spread(a, b, step):
    out, x = [], a
    while x < b - 0.01:
        out.append(x)
        x += step
    return out


def tiles(across, up, sheet_across, sheet_up):
    """Cover `across` x `up` with sheets no bigger than one, the last of each row cut down.

    Sheet goods come in one size and are cut to fit, so a 408" roof takes four whole sheets and a
    24" rip, not four and a half of them.
    """
    out, v = [], 0.0
    while v < up - 0.01:
        h = min(sheet_up, up - v)
        u = 0.0
        while u < across - 0.01:
            w = min(sheet_across, across - u)
            out.append((round(u, 3), round(v, 3), round(w, 3), round(h, 3)))
            u += w
        v += h
    return out


def clear_spans(lo, hi, holes):
    """What's left of lo..hi with the holes taken out."""
    spans, cut = [], lo
    for h0, h1 in sorted(holes):
        h0, h1 = max(h0, lo), min(h1, hi)
        if h1 <= cut:
            continue
        if h0 > cut:
            spans.append((cut, h0))
        cut = max(cut, h1)
    if cut < hi:
        spans.append((cut, hi))
    return [(a, b) for a, b in spans if b - a > 0.5]


def holes_in(axis, plane, lo, hi):
    """The openings in the wall whose face this is, as (lo, hi, sill, head) along the face."""
    out = []
    for o in openings:
        if o["axis"] != axis or not (o["band"][0] - 0.6 <= plane <= o["band"][1] + 0.6):
            continue
        if o["hi"] <= lo or o["lo"] >= hi:
            continue
        out.append((max(o["lo"], lo), min(o["hi"], hi), o["sill"], o["head"]))
    return out


def place_group(label, material, tag, pieces, where, length, width, thick, turn=None, called=None, offset=None):
    """A part per size of piece, and a copy for every piece of that size.

    `where` gives a piece's corner *before* any turn, which is what a part's `at` means. A copy is
    placed by the lowest corner of the turned part instead, so `offset(w, h)` says how far that is
    from the corner the part was turned about.
    """
    sizes: dict[tuple[float, float], list] = {}
    for (u, v, w, h) in pieces:
        sizes.setdefault((w, h), []).append(where(u, v, w, h))
    for n, ((w, h), ats) in enumerate(sorted(sizes.items(), key=lambda kv: (-len(kv[1]), -kv[0][0]))):
        # A letter, not a number: "Sheathing 2" is already this part's own second copy.
        name = label if n == 0 else f"{label} {chr(ord('a') + n)}"
        off = offset(w, h) if offset else (0.0, 0.0, 0.0)
        moved = [(a[0] + off[0], a[1] + off[1], a[2] + off[2]) for a in ats[1:]]
        part(name, material, [h, w, thick], ats[0], length=length, width=width,
             called=called or label, tag=tag, copies=copies_at(name, moved),
             **({"turn": turn} if turn else {}))


def sheet_face(label, material, tag, axis, plane, lo, hi, z0, z1, sheet, thick, tag_name=None, called=None):
    """Sheet one wall face in courses, cut round every opening in that wall.

    Sheathing goes on and the openings are cut out of it, so the pieces are the clear runs full
    height plus the strips over and under each window. Boarding straight across a doorway is what
    tells you a model was never meant to be walked through.
    """
    holes = holes_in(axis, plane, lo, hi)
    pieces = []
    for (a, b) in clear_spans(lo, hi, [(h[0], h[1]) for h in holes]):
        pieces += [(a - lo + u, v, w, h) for (u, v, w, h) in tiles(b - a, z1 - z0, sheet[0], sheet[1])]
    for (h0, h1, sill, head) in holes:
        if sill > z0 + 0.5:
            pieces += [(h0 - lo + u, v, w, h) for (u, v, w, h) in tiles(h1 - h0, sill - z0, sheet[0], sheet[1])]
        if head < z1 - 0.5:
            pieces += [(h0 - lo + u, head - z0 + v, w, h) for (u, v, w, h) in tiles(h1 - h0, z1 - head, sheet[0], sheet[1])]
    # A piece's height goes up the wall and its width along it, so `length` is z either way.
    if axis == "y":
        where = lambda u, v, w, h: (lo + u, plane, z0 + v)        # noqa: E731
        wd = "x"
    else:
        where = lambda u, v, w, h: (plane, lo + u, z0 + v)        # noqa: E731
        wd = "y"
    place_group(label, material, tag_name or tag, pieces, where, "z", wd, thick, called=called)


def tilted_floor(span, depth, degrees):
    """Where a part turned about x ends up, relative to the corner it was turned about.

    A turn swings the part's y and z, and a copy is placed by the lowest corner of the result, so
    the generator has to know where that corner lands.
    """
    c, s = math.cos(math.radians(degrees)), math.sin(math.radians(degrees))
    ys = [y * c - z * s for y in (0.0, span) for z in (0.0, depth)]
    zs = [y * s + z * c for y in (0.0, span) for z in (0.0, depth)]
    return min(ys), min(zs)


# ---------------------------------------------------------------- materials

materials = [
    {"id": "fir", "library": "Douglas Fir Lumber"},
    {"id": "pt", "library": "Pressure-Treated Lumber"},
    {"id": "osb", "name": "OSB Sheathing", "kind": "sheet", "color": [198, 164, 110]},
    {"id": "ply", "library": "Plywood"},
    {"id": "cedar", "library": "Cedar Lumber"},
    {"id": "maple", "library": "Hard Maple"},
    {"id": "cabply", "library": "Baltic Birch Plywood"},
    {"id": "slab", "name": "Concrete Slab", "kind": "other", "color": [176, 176, 172]},
    {"id": "board", "name": "Drywall", "kind": "other", "color": [238, 236, 230]},
    {"id": "glass", "name": "Glazing", "kind": "other", "color": [168, 204, 214]},
    {"id": "shingle", "name": "Asphalt Shingles", "kind": "other", "color": [74, 74, 78]},
    {"id": "stone", "name": "Quartz Countertop", "kind": "other", "color": [226, 224, 216]},
    {"id": "china", "name": "Plumbing Fixture", "kind": "other", "color": [246, 246, 248]},
    {"id": "metal", "name": "Satin Nickel Hardware", "kind": "hardware", "color": [176, 180, 186]},
]

# Tag folders, so the Tags panel peels the house apart a layer at a time.
for folder, inside in [("Shell", ["Slab", "Wall framing", "Sheathing"]),
                       ("Roof", ["Roof framing", "Roof sheathing", "Roofing", "Eaves"]),
                       ("Inside", ["Drywall", "Doors", "Windows", "Kitchen", "Bathrooms"]),
                       ("Outside", ["Porch"])]:
    tags.append({"name": folder, "folder": True})
    tags += [{"name": t, "inside": folder} for t in inside]

# ---------------------------------------------------------------- wall framing

ext_studs: list[tuple] = []
int_studs: list[tuple] = []
sticks: list[tuple] = []     # headers, jacks and rough sills
plates: list[dict] = []


def stud_runs(a, b, blocked):
    out, x = [a], a + OC
    while x < b - 1.0:
        out.append(x - PLATE / 2)
        x += OC
    out.append(b - PLATE)
    return sorted({round(s, 3) for s in out
                   if not any(s + PLATE > h0 - 0.01 and s < h1 + 0.01 for h0, h1 in blocked)})


def cripples_in(o0, o1):
    """Where the short studs over a header and under a rough sill go.

    Laid out 16" on centre from the opening's near edge, with one always against the far jack,
    which is how the run actually gets framed: the last space is whatever is left.
    """
    out, x = [o0], o0 + OC
    while x < o1 - PLATE - 1.0:
        out.append(x)
        x += OC
    out.append(o1 - PLATE)
    return sorted({round(v, 3) for v in out})


def wall(x0, y0, x1, y1, thick, holes, interior):
    along_x = abs(x1 - x0) > abs(y1 - y0)
    a, b = (x0, x1) if along_x else (y0, y1)
    # The bottom plate is cut out of every doorway once the wall is up; the top plates run on
    # over them. A plate is a board, so a long wall takes two 16-footers rather than one 32.
    doorways = [(h[1], h[2]) for h in holes if h[3] <= 0.01]
    for z in (0.0, PLATE + STUD_H, PLATE + STUD_H + PLATE):
        for s0, s1 in (clear_spans(a, b, doorways) if z == 0.0 else [(a, b)]):
            start = s0
            while start < s1 - 0.01:
                span = min(192.0, s1 - start) if s1 - start > 192.0 else s1 - start
                span = (s1 - start) / 2 if 192.0 < s1 - start <= 384.0 else span
                at = (start, y0, z) if along_x else (x0, start, z)
                plates.append({"at": at, "span": span, "thick": thick, "length": "x" if along_x else "y"})
                start += span
    for label, o0, o1, sill, head in holes:
        openings.append({"axis": "y" if along_x else "x", "band": (y0, y0 + thick) if along_x else (x0, x0 + thick),
                         "lo": o0, "hi": o1, "sill": sill, "head": head})
    blocked = [(o[1] - 3.0, o[2] + 3.0) for o in holes]
    runs = stud_runs(a, b, blocked)
    for s in runs:
        (int_studs if interior else ext_studs).append((s, y0, PLATE) if along_x else (x0, s, PLATE))
    for label, o0, o1, sill, head in holes:
        for side in (o0 - PLATE, o1):                     # king studs
            (int_studs if interior else ext_studs).append((side, y0, PLATE) if along_x else (x0, side, PLATE))
        for side in (o0, o1 - PLATE):                     # jacks, carrying the header
            at = (side, y0, PLATE) if along_x else (x0, side, PLATE)
            sticks.append(("Jack", at, head - PLATE, thick, "z"))
        at = (o0 - PLATE, y0, head) if along_x else (x0, o0 - PLATE, head)
        sticks.append(("Header", at, (o1 - o0) + 2 * PLATE, thick, "x" if along_x else "y"))
        if sill > 0.0:
            at = (o0, y0, sill - PLATE) if along_x else (x0, o0, sill - PLATE)
            sticks.append(("Sill", at, o1 - o0, thick, "x" if along_x else "y"))
        # Cripples fill what the header and the rough sill leave over: 8-5/8" of it above a
        # window header, 33" below its sill. Without them the wall is open there.
        for c in cripples_in(o0, o1):
            over = STUD_TOP - (head + PLATE)
            if over > 1.0:
                at = (c, y0, head + PLATE) if along_x else (x0, c, head + PLATE)
                sticks.append(("Cripple", at, over, thick, "z"))
            if sill - 2 * PLATE > 1.0:
                at = (c, y0, PLATE) if along_x else (x0, c, PLATE)
                sticks.append(("Cripple", at, sill - 2 * PLATE, thick, "z"))
    # One row of blocking between the studs, on the seam where the drywall's two courses meet
    # (the sheets break at 48"), which is what backs it. It stops short of the openings.
    for s0, s1 in zip(runs, runs[1:]):
        gap = s1 - (s0 + PLATE)
        # A bay narrower than the wall is thick gets none: the block would be a square scrap
        # whose grain reads the wrong way round, and nothing spans a gap that small anyway.
        if gap <= thick or any(s0 + PLATE < h1 and s1 > h0 for h0, h1 in blocked):
            continue
        at = (s0 + PLATE, y0, BLOCK_Z) if along_x else (x0, s0 + PLATE, BLOCK_Z)
        sticks.append(("Block", at, gap, thick, "x" if along_x else "y"))


wall(0, 0, W, 0, EXT, [("Front door", 96.0, 134.0, 0.0, DOOR_H),
                       ("Window", 30.0, 68.0, SILL, SILL + WIN_H),
                       ("Window", 160.0, 198.0, SILL, SILL + WIN_H),
                       ("Window", 250.0, 288.0, SILL, SILL + WIN_H),
                       ("Window", 320.0, 358.0, SILL, SILL + WIN_H)], False)
wall(0, D - EXT, W, D - EXT, EXT, [("Window", 50.0, 88.0, SILL, SILL + WIN_H),
                                   ("Window", 296.0, 334.0, SILL, SILL + WIN_H)], False)
wall(0, EXT, 0, D - EXT, EXT, [("Window", 60.0, 98.0, SILL, SILL + WIN_H),
                               ("Window", 200.0, 238.0, SILL, SILL + WIN_H)], False)
wall(W - EXT, EXT, W - EXT, D - EXT, EXT, [("Window", 60.0, 98.0, SILL, SILL + WIN_H),
                                           ("Window", 200.0, 238.0, SILL, SILL + WIN_H)], False)
wall(EXT, Y1, W - EXT, Y1, INT, [("Door", 40.0, 74.0, 0.0, DOOR_H),
                                 ("Door", 150.0, 184.0, 0.0, DOOR_H),
                                 ("Door", 290.0, 324.0, 0.0, DOOR_H)], True)
wall(216.0, EXT, 216.0, Y1, INT, [("Opening", 60.0, 120.0, 0.0, DOOR_H)], True)   # living to kitchen
wall(132.0, Y1 + INT, 132.0, D - EXT, INT, [], True)
wall(192.0, Y1 + INT, 192.0, D - EXT, INT, [], True)
wall(252.0, Y1 + INT, 252.0, D - EXT, INT, [("Door", 200.0, 234.0, 0.0, DOOR_H)], True)

part("Slab", "slab", [W, D, 4], [0, 0, -4], tag="Slab")
part("Exterior stud", "fir", [STUD_H, EXT, PLATE], [ext_studs[0][0], ext_studs[0][1], PLATE],
     length="z", width="y", tag="Wall framing", copies=copies_at("Exterior stud", ext_studs[1:]))
part("Interior stud", "fir", [STUD_H, INT, PLATE], [int_studs[0][0], int_studs[0][1], PLATE],
     length="z", width="y", tag="Wall framing", copies=copies_at("Interior stud", int_studs[1:]))

by_plate: dict[tuple, list] = {}
for p in plates:
    by_plate.setdefault((round(p["span"], 3), p["thick"], p["length"]), []).append(p["at"])
for n, ((span, thick, ln), ats) in enumerate(sorted(by_plate.items(), key=lambda kv: -len(kv[1]))):
    part(f"Plate {n + 1}", "fir", [span, thick, PLATE], ats[0], length=ln, width="y" if ln == "x" else "x",
         called="Plate", tag="Wall framing", copies=copies_at(f"Plate {n + 1}", ats[1:]))

by_stick: dict[tuple, list] = {}
for label, at, span, thick, axis in sticks:
    by_stick.setdefault((label, round(span, 3), thick, axis), []).append(at)
for n, ((label, span, thick, axis), ats) in enumerate(sorted(by_stick.items(), key=lambda kv: -len(kv[1]))):
    part(f"{label} {n + 1}", "fir", [span, thick, PLATE], ats[0], length=axis,
         width="y" if axis in ("x", "z") else "x", called=label, tag="Wall framing",
         copies=copies_at(f"{label} {n + 1}", ats[1:]))

# ---------------------------------------------------------------- sheathing, cut round the openings

sheet_face("Sheathing", "osb", "Sheathing", "y", -OSB, 0, W, 0, WALL_TOP, (48.0, 96.0), OSB, called="Sheathing")
sheet_face("Sheathing back", "osb", "Sheathing", "y", D, 0, W, 0, WALL_TOP, (48.0, 96.0), OSB, called="Sheathing")
sheet_face("Sheathing west", "osb", "Sheathing", "x", -OSB, EXT, D - EXT, 0, WALL_TOP, (48.0, 96.0), OSB, called="Sheathing")
sheet_face("Sheathing east", "osb", "Sheathing", "x", W, EXT, D - EXT, 0, WALL_TOP, (48.0, 96.0), OSB, called="Sheathing")

# ---------------------------------------------------------------- roof

RAFTER_RUN = math.hypot(RIDGE_Y + EAVE, (RIDGE_Y + EAVE) * PITCH)
RAFTER_D = 7.25
RIDGE_Z = WALL_TOP + RISE
EAVE_Z = WALL_TOP - RISE * (EAVE / RIDGE_Y)
UP = math.hypot(RIDGE_Y + EAVE, RISE + EAVE * PITCH)
ROOF_X, ROOF_W = -EAVE, W + 2 * EAVE

dy, dz = tilted_floor(RAFTER_RUN, RAFTER_D, SLOPE)
part("Rafter", "fir", [RAFTER_RUN, RAFTER_D, PLATE], [0, -EAVE, EAVE_Z], length="y", width="z",
     tag="Roof framing", turn={"axis": "x", "degrees": r(SLOPE)},
     copies=copies_at("Rafter", [(x, -EAVE + dy, EAVE_Z + dz) for x in spread(OC, W, OC)]))
dy2, dz2 = tilted_floor(RAFTER_RUN, RAFTER_D, -SLOPE)
part("Rafter north", "fir", [RAFTER_RUN, RAFTER_D, PLATE], [0, RIDGE_Y, RIDGE_Z], length="y", width="z",
     called="Rafter", tag="Roof framing", turn={"axis": "x", "degrees": r(-SLOPE)},
     copies=copies_at("Rafter north", [(x, RIDGE_Y + dy2, RIDGE_Z + dz2) for x in spread(OC, W, OC)]))
part("Ridge board", "fir", [192, 9.25, 1.5], [0, RIDGE_Y - 0.75, RIDGE_Z - 9.25], length="x", width="z",
     tag="Roof framing", free_to_move=True, copies=copies_at("Ridge board", [(192, RIDGE_Y - 0.75, RIDGE_Z - 9.25)]))
part("Ceiling joist front", "fir", [Y1 - EXT + INT, RAFTER_D, PLATE], [0, EXT, WALL_TOP], length="y",
     width="z", called="Ceiling joist", tag="Roof framing", free_to_move=True,
     copies=copies_at("Ceiling joist front", [(x, EXT, WALL_TOP) for x in spread(OC, W, OC)]))
part("Ceiling joist back", "fir", [D - EXT - Y1, RAFTER_D, PLATE], [0, Y1, WALL_TOP], length="y",
     width="z", called="Ceiling joist", tag="Roof framing", free_to_move=True,
     copies=copies_at("Ceiling joist back", [(x, Y1, WALL_TOP) for x in spread(OC, W, OC)]))


# The eaves. The rafter tails are cut square, so a tail's end leans outward as it rises: its
# bottom corner sits on the tail line and its top corner LEAN further out and RISE_D up. The
# fascia hangs plumb off that top corner, the way it goes on site, and the soffit closes the
# triangle back to the wall sheathing. Without the pair of them the tails stop in mid-air.
FASCIA_H, FASCIA_T, SOFFIT_T = 7.25, 0.75, 0.5
LEAN = RAFTER_D * math.sin(math.radians(SLOPE))      # how far out a tail's top corner sits
RISE_D = RAFTER_D * math.cos(math.radians(SLOPE))    # and how far up
FASCIA_Z = EAVE_Z + RISE_D - FASCIA_H                # its top meets the sheathing above the tail
SOFFIT_Z = FASCIA_Z - SOFFIT_T
for side, out_y, front in [("", -EAVE - LEAN, True), (" back", D + EAVE + LEAN, False)]:
    face = out_y - FASCIA_T if front else out_y      # the fascia's own lowest-y corner
    place_group(f"Fascia{side}", "cedar", "Eaves", tiles(ROOF_W, FASCIA_H, 136.0, FASCIA_H),
                lambda u, v, w, h, fy=face: (ROOF_X + u, fy, FASCIA_Z + v),
                "z", "x", FASCIA_T, called="Fascia")
    inner, wall_face = (out_y, -OSB) if front else (D + OSB, out_y)
    place_group(f"Soffit{side}", "ply", "Eaves", tiles(ROOF_W, wall_face - inner, 96.0, 48.0),
                lambda u, v, w, h, oy=inner: (ROOF_X + u, oy + v, SOFFIT_Z),
                "y", "x", SOFFIT_T, called="Soffit")

# Gable end walls: the triangle over each end wall. Without them the attic is open at both ends
# and you can see daylight straight through the house.
gable: dict[float, list] = {}
for d in spread(OC, RIDGE_Y, OC):
    h = RISE - PITCH * d
    if h < 6.0:
        continue
    for y in (RIDGE_Y - d, RIDGE_Y + d):
        for x in (0.0, W - EXT):
            gable.setdefault(round(h, 3), []).append((x, y - PLATE / 2, WALL_TOP))
for x in (0.0, W - EXT):
    gable.setdefault(RISE, []).append((x, RIDGE_Y - PLATE / 2, WALL_TOP))
for n, (h, ats) in enumerate(sorted(gable.items(), key=lambda kv: -kv[0])):
    part(f"Gable stud {n + 1}", "fir", [h, EXT, PLATE], ats[0], length="z", width="y",
         called="Gable stud", tag="Wall framing", copies=copies_at(f"Gable stud {n + 1}", ats[1:]))
# One triangular sheet over each end, cut from its outline rather than framed out of rectangles.
part("Gable sheathing", "osb", [D, RISE, OSB], [-OSB, 0, WALL_TOP], length="y", width="z",
     called="Gable sheathing", tag="Sheathing", outline=[[0, 0], [D, 0], [RIDGE_Y, RISE]],
     copies=copies_at("Gable sheathing", [(W, 0, WALL_TOP)]))


def on_slope(deg, base_y, base_z):
    """A piece `v` up the slope and `u` along the house, by the corner it is turned about."""
    c, s = math.cos(math.radians(deg)), math.sin(math.radians(deg))
    return lambda u, v, w, h: (ROOF_X + u, base_y + v * c, base_z + v * s)


for side, deg, by, bz in [("", SLOPE, -EAVE, EAVE_Z + RAFTER_D), (" north", -SLOPE, RIDGE_Y, RIDGE_Z + RAFTER_D)]:
    turn = {"axis": "x", "degrees": r(deg)}
    for label, material, tag, sheet, t, name in [
            ("Roof sheathing", "ply", "Roof sheathing", (96.0, 48.0), 0.5, "Roof sheathing"),
            ("Shingles", "shingle", "Roofing", (144.0, 36.0), 0.25, "Shingles")]:
        lift = 0.0 if t == 0.5 else 0.5
        place_group(f"{label}{side}", material, tag, tiles(ROOF_W, UP, sheet[0], sheet[1]),
                    on_slope(deg, by, bz + lift), "y", "x", t, turn=turn, called=name,
                    offset=lambda w, h, tt=t: (0.0, *tilted_floor(h, tt, deg)))

# The porch roof gets the same shingles, so it reads as part of the house and not a shed.
part("Porch shingles", "shingle", [PORCH_W, PORCH_D + 6, 0.25], [PORCH_X, -PORCH_D, WALL_TOP - 12.0 + 0.5],
     length="x", width="y", tag="Roofing", called="Shingles", turn={"axis": "x", "degrees": 14.0})

# ---------------------------------------------------------------- porch

PIER_Z = DECK_Z - 7.25 - 12.0
part("Porch pier", "pt", [12, 5.5, 5.5], [PORCH_X + 2, -PORCH_D + 2, PIER_Z], length="z", width="x",
     tag="Porch", copies=copies_at("Porch pier", [(PORCH_X + PORCH_W / 2 - 2.75, -PORCH_D + 2, PIER_Z),
                                                  (PORCH_X + PORCH_W - 7.5, -PORCH_D + 2, PIER_Z)]))
part("Porch beam", "pt", [PORCH_W, 7.25, 3], [PORCH_X, -PORCH_D + 2, DECK_Z - 7.25], length="x", width="z",
     tag="Porch", free_to_move=True)
part("Porch ledger", "pt", [PORCH_W, 7.25, 1.5], [PORCH_X, 0, DECK_Z - 7.25], length="x", width="z", tag="Porch")
part("Porch joist", "pt", [PORCH_D - 2, 7.25, 1.5], [PORCH_X, -PORCH_D + 2, DECK_Z - 7.25], length="y", width="z",
     tag="Porch", copies=copies_at("Porch joist", [(x, -PORCH_D + 2, DECK_Z - 7.25)
                                                   for x in spread(PORCH_X + OC, PORCH_X + PORCH_W - 1, OC)]))
part("Decking", "cedar", [PORCH_W, 5.5, 1.5], [PORCH_X, -PORCH_D + 2, DECK_Z], tag="Porch",
     copies=copies_at("Decking", [(PORCH_X, y, DECK_Z) for y in spread(-PORCH_D + 8, 0, 6.0)]))

# The roof over it: posts up from the decking to a header, rafters sloping away from the house.
PORCH_TOP = WALL_TOP - 12.0
PORCH_EAVE = PORCH_TOP - 5.5 - 7.25
part("Porch post", "cedar", [PORCH_EAVE, 5.5, 5.5], [PORCH_X + 2, -PORCH_D + 2, 0], length="z", width="x",
     tag="Porch", copies=copies_at("Porch post", [(PORCH_X + PORCH_W - 7.5, -PORCH_D + 2, 0)]))
part("Porch header", "fir", [PORCH_W, 7.25, 3], [PORCH_X, -PORCH_D + 2, PORCH_EAVE], length="x", width="z",
     tag="Porch")

pdy, pdz = tilted_floor(PORCH_D + 6, 5.5, 14.0)
part("Porch rafter", "fir", [PORCH_D + 6, 5.5, PLATE], [PORCH_X, -PORCH_D, PORCH_TOP - 5.5], length="y",
     width="z", tag="Porch", turn={"axis": "x", "degrees": 14.0},
     copies=copies_at("Porch rafter", [(x, -PORCH_D + pdy, PORCH_TOP - 5.5 + pdz)
                                       for x in spread(PORCH_X + 24, PORCH_X + PORCH_W - 1, 24.0)]))
part("Porch roof deck", "ply", [PORCH_W, PORCH_D + 6, 0.5], [PORCH_X, -PORCH_D, PORCH_TOP], length="x",
     width="y", tag="Porch", turn={"axis": "x", "degrees": 14.0})

# ---------------------------------------------------------------- doors and windows

part("Front door", "fir", [82, 38, 1.75], [96, 1.875, 0], length="z", width="x", tag="Doors", free_to_move=True)
inside_doors = [(40.0, Y1 + 1.0, 0.0), (150.0, Y1 + 1.0, 0.0), (290.0, Y1 + 1.0, 0.0)]
part("Door", "ply", [82, 34, 1.375], list(inside_doors[0]), length="z", width="x", tag="Doors",
     copies=copies_at("Door", inside_doors[1:]))
part("Door side", "ply", [82, 34, 1.375], [253.0, 200.0, 0], length="z", width="y", called="Door", tag="Doors")

front_windows = [(30.0, 0.0, SILL), (160.0, 0.0, SILL), (250.0, 0.0, SILL), (320.0, 0.0, SILL),
                 (50.0, D - 0.25, SILL), (296.0, D - 0.25, SILL)]
part("Window", "glass", [38, WIN_H, 0.25], list(front_windows[0]), length="x", width="z", tag="Windows",
     copies=copies_at("Window", front_windows[1:]))
side_windows = [(0.0, 60.0, SILL), (0.0, 200.0, SILL), (W - 0.25, 60.0, SILL), (W - 0.25, 200.0, SILL)]
part("Window side", "glass", [38, WIN_H, 0.25], list(side_windows[0]), length="y", width="z", called="Window",
     tag="Windows", copies=copies_at("Window side", side_windows[1:]))

# ---------------------------------------------------------------- door hardware
# A knob and its rose on both sides of every door. These are `hardware`, not wood: they are
# named, counted in the shopping list, and kept out of the cutting diagrams and the cut list,
# which is what you want of a thing you buy in a box rather than cut from a board.
KNOB_D, KNOB_P, ROSE_D, ROSE_T = 2.25, 1.75, 2.75, 0.1875
KNOB_Z = 36.0                # a knob's height off the floor


def disc(d):
    """A circle of diameter `d` as an outline, which the recipe pushes through the thickness."""
    return [[0, d / 2], {"arc_to": [d, d / 2], "through": [d / 2, 0]},
            {"arc_to": [0, d / 2], "through": [d / 2, d]}]


# Each door as (the centre across its face, its two faces, the axis a knob's shank runs along).
knobs: dict[str, list] = {"y": [], "x": []}
roses: dict[str, list] = {"y": [], "x": []}
for across, near, far, axis in [(130.5, 1.875, 3.625, "y"),      # the front door
                                (70.5, 169.0, 170.375, "y"),     # three off the hall
                                (180.5, 169.0, 170.375, "y"),
                                (320.5, 169.0, 170.375, "y"),
                                (230.5, 253.0, 254.375, "x")]:   # bath 2, off bedroom 2
    for face, outward in ((near, False), (far, True)):
        rose = face if outward else face - ROSE_T
        knob = face + ROSE_T if outward else face - ROSE_T - KNOB_P
        if axis == "y":
            roses[axis].append((across - ROSE_D / 2, rose, KNOB_Z - ROSE_D / 2))
            knobs[axis].append((across - KNOB_D / 2, knob, KNOB_Z - KNOB_D / 2))
        else:
            roses[axis].append((rose, across - ROSE_D / 2, KNOB_Z - ROSE_D / 2))
            knobs[axis].append((knob, across - KNOB_D / 2, KNOB_Z - KNOB_D / 2))

for what, dia, thick, bought, held in [("knob", KNOB_D, KNOB_P, "Passage knob", knobs),
                                       ("rose", ROSE_D, ROSE_T, "Knob rose", roses)]:
    for axis, ats in held.items():
        name = f"Door {what}" if axis == "y" else f"Door {what} side"
        part(name, "metal", [dia, dia, thick], list(ats[0]), length="x" if axis == "y" else "y",
             width="z", tag="Doors", called=f"Door {what}", hardware=bought, outline=disc(dia),
             copies=copies_at(name, ats[1:]))

# ---------------------------------------------------------------- drywall
# Two 4' courses up each wall and board on the ceiling, cut round the doorways the same way the
# sheathing is cut round the windows.

rooms = [("Living", EXT, EXT, 216.0, Y1), ("Kitchen", 219.5, EXT, W - EXT, Y1),
         ("Bedroom 1", EXT, Y1 + INT, 132.0, D - EXT), ("Bath 1", 135.5, Y1 + INT, 192.0, D - EXT),
         ("Bath 2", 195.5, Y1 + INT, 252.0, D - EXT), ("Bedroom 2", 255.5, Y1 + INT, W - EXT, D - EXT)]
CEIL_Z = WALL_TOP - BOARD
for name, x0, y0, x1, y1 in rooms:
    sheet_face(f"Drywall {name} S", "board", "Drywall", "y", y0, x0, x1, 0, 96.0, (96.0, 48.0), BOARD, called="Drywall")
    sheet_face(f"Drywall {name} N", "board", "Drywall", "y", y1 - BOARD, x0, x1, 0, 96.0, (96.0, 48.0), BOARD, called="Drywall")
    sheet_face(f"Drywall {name} W", "board", "Drywall", "x", x0, y0, y1, 0, 96.0, (96.0, 48.0), BOARD, called="Drywall")
    sheet_face(f"Drywall {name} E", "board", "Drywall", "x", x1 - BOARD, y0, y1, 0, 96.0, (96.0, 48.0), BOARD, called="Drywall")
    place_group(f"Ceiling {name}", "board", "Drywall", tiles(x1 - x0, y1 - y0, 96.0, 48.0),
                lambda u, v, w, h, ox=x0, oy=y0: (ox + u, oy + v, CEIL_Z), "y", "x", BOARD, called="Drywall")

# ---------------------------------------------------------------- kitchen and baths

for i, x in enumerate(spread(222.0, 222.0 + 4 * 24.0, 24.0)):
    templates.append({"kind": "cabinet", "name": f"Base cabinet {i + 1}", "at": [r(x), r(EXT), 0],
                      "wood": "cabply", "tag": "Kitchen",
                      "sizes": {"width": 24, "height": 34.5, "depth": 24, "shelves": 1, "back": True,
                                "joints": "Dados", "fronts": "Overlay", "doors": "Slab",
                                "drawer_joints": "Rabbets", "pulls": "Bar",
                                "fills": ["Drawer", "Doors"] if i % 2 == 0 else ["Drawer", "Drawer"]}})
part("Countertop", "stone", [96, 25.5, 1.5], [222, EXT, 34.5], tag="Kitchen")
part("Vanity", "maple", [36, 21, 34], [137.5, D - EXT - 21, 0], tag="Bathrooms", free_to_move=True,
     copies=copies_at("Vanity", [(197.5, D - EXT - 21, 0)]))
part("Tub", "china", [60, 30, 20], [135.5, Y1 + INT, 0], tag="Bathrooms",
     copies=copies_at("Tub", [(195.5, Y1 + INT, 0)]))
part("Toilet", "china", [20, 28, 30], [176, Y1 + INT + 36, 0], length="y", width="x", tag="Bathrooms",
     copies=copies_at("Toilet", [(236, Y1 + INT + 36, 0)]))

# ---------------------------------------------------------------- out

recipe = {"name": "Small house", "units": "in", "materials": materials, "tags": tags,
          "parts": parts, "templates": templates}
count = sum(1 + len(p.get("copies", [])) for p in parts)
print(f"{len(parts)} designs, {count} placed parts, {len(templates)} cabinets", file=sys.stderr)


def one(obj):
    return json.dumps(obj, separators=(", ", ": "))


def rows(items):
    """A line per item, so the file diffs by the part that changed."""
    return ",\n".join("    " + one(i) for i in items)


lines = ['{', '  "name": "Small house",', '  "units": "in",', '  "materials": [',
         rows(materials), '  ],', '  "tags": [', rows(tags), '  ],', '  "parts": [']
body = []
for p in parts:
    copies = p.pop("copies", None)
    if not copies:
        body.append("    " + one(p))
        continue
    head = one(p)[:-1].rstrip()
    body.append(f'{"    "}{head}, "copies": [\n' + ",\n".join("      " + one(c) for c in copies) + "\n    ]}")
lines += [",\n".join(body), '  ],', '  "templates": [', rows(templates), '  ]', '}']
sys.stdout.write("\n".join(lines) + "\n")
