//! The Joinery panel, in the tray when two parts (groups or components) are selected: pick a
//! joint, set its sizes, and make it. Joints that don't fit how the parts sit say how to place
//! them. Entity Info lists the joints a part has.

use eframe::egui;
use woodsketch_doc::joinery::{BISCUITS, DOVETAIL_HALF_PIN, DOVETAIL_PIN, Dovetails, Dowels, Fit, JointKind, LOOSE_TENONS, Slots, Spline, Tenon, pocket_screw_for};
use woodsketch_doc::units::{format_length, parse_length};
use woodsketch_doc::{Document, Entity, InstanceId};

/// The two selected parts, if exactly two groups or components are selected.
pub fn selected_pair(doc: &Document) -> Option<(InstanceId, InstanceId)> {
    let parts: Vec<InstanceId> = doc.selection().iter().filter_map(|e| if let Entity::Instance(i) = e { Some(i) } else { None }).collect();
    match parts[..] {
        [a, b] if doc.selection().len() == 2 => Some((a, b)),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Choice {
    LetIn,
    Overlap,
    MortiseAndTenon,
    Dowels,
    Biscuits,
    LooseTenons,
    PocketScrews,
    Dovetails,
    Splines,
    BoxJoint,
    HalfLap,
}

impl Choice {
    const ALL: [Self; 11] = [
        Self::LetIn,
        Self::MortiseAndTenon,
        Self::Dovetails,
        Self::Dowels,
        Self::Biscuits,
        Self::LooseTenons,
        Self::Splines,
        Self::PocketScrews,
        Self::BoxJoint,
        Self::HalfLap,
        Self::Overlap,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::LetIn => "Dado, groove or rabbet",
            Self::Overlap => "Cut where they overlap",
            Self::MortiseAndTenon => "Mortise and tenon",
            Self::Dowels => "Dowels",
            Self::Biscuits => "Biscuits",
            Self::LooseTenons => "Loose tenons (Domino style)",
            Self::PocketScrews => "Pocket screws",
            Self::Dovetails => "Dovetails",
            Self::Splines => "Spline",
            Self::BoxJoint => "Box joint",
            Self::HalfLap => "Half-lap",
        }
    }

    fn fits(self, fit: &Fit) -> bool {
        match self {
            Self::LetIn | Self::MortiseAndTenon | Self::Dowels | Self::Biscuits | Self::LooseTenons | Self::Splines | Self::PocketScrews => fit.butt.is_some(),
            Self::Dovetails => fit.butt.is_some() && fit.corner,
            Self::Overlap => fit.overlap,
            Self::BoxJoint => fit.corner,
            Self::HalfLap => fit.half_lap,
        }
    }

    /// The picker entry that makes a joint of this kind (for Change).
    fn of(kind: JointKind) -> Self {
        match kind {
            JointKind::MortiseAndTenon => Self::MortiseAndTenon,
            JointKind::Dowels => Self::Dowels,
            JointKind::Biscuits => Self::Biscuits,
            JointKind::LooseTenons => Self::LooseTenons,
            JointKind::PocketScrews => Self::PocketScrews,
            JointKind::Dovetails => Self::Dovetails,
            JointKind::Splines => Self::Splines,
            JointKind::BoxJoint => Self::BoxJoint,
            JointKind::HalfLap => Self::HalfLap,
            JointKind::Mortise | JointKind::Notch => Self::Overlap,
            JointKind::Dado | JointKind::Groove | JointKind::Rabbet => Self::LetIn,
        }
    }

    /// How to place the parts for this joint.
    fn setup(self) -> &'static str {
        match self {
            Self::LetIn => "Butt one part's end or edge against the other (a shelf against a side, a panel against a post). It's let in by the depth: a dado across the grain, a groove along it, a rabbet at an edge.",
            Self::MortiseAndTenon => "Butt the rail's end against the other part (a rail against a leg or stile).",
            Self::Dowels | Self::Biscuits | Self::LooseTenons | Self::Splines | Self::PocketScrews => "Butt one part's end or edge against the other.",
            Self::Dovetails => "Butt one board's end against the inside of the other at its end, like a drawer's side against its front. The butting board gets the tails.",
            Self::BoxJoint => "Put the two parts' ends together at a corner (a drawer's front and side), butted or overlapping.",
            Self::HalfLap => "Cross the two boards so they overlap through their whole thickness.",
            Self::Overlap => "Move one part into the other by the joint's depth (a shelf 1/4\" into the side).",
        }
    }
}

pub fn show(ui: &mut egui::Ui, doc: &mut Document, (a, b): (InstanceId, InstanceId)) {
    let fit = match doc.joint_fit(a, b) {
        Ok(fit) => fit,
        Err(woodsketch_doc::joinery::JoineryError::NotSquare) => {
            angled(ui, doc, (a, b));
            return;
        }
        Err(e) => {
            ui.weak(e.to_string());
            return;
        }
    };
    // Already joined where these two meet: say so, and keep the picker for adding another joint
    // out of the way. Copies of the same components joined somewhere else (a drawer's opposite
    // corner) don't count: this meeting can still be joined.
    let joints = doc.joints_here(a, b);
    if joints.is_empty() {
        let other = doc.container().instance(b).map(|i| i.definition);
        let elsewhere: Vec<String> = doc.joint_list(a).into_iter().filter(|(_, j, _)| Some(j.cut) == other || Some(j.into) == other).map(|(_, j, _)| j.kind.name().to_lowercase()).collect();
        if !elsewhere.is_empty() {
            ui.weak(format!("Other copies of these parts are joined ({}) where they meet elsewhere; not here yet.", elsewhere.join(", ")));
        }
        picker(ui, doc, (a, b), &fit);
    } else {
        ui.label("Already joined:");
        let mut change = None;
        let mut remove = None;
        for (index, joint, _) in &joints {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("{} {}, {}", egui_phosphor::regular::CHECK, joint.kind.name(), joint.detail));
                if ui.small_button("Change").on_hover_text("Take it out and set it up again from its sizes").clicked() {
                    change = Some((*index, joint.kind, joint.sizes.clone()));
                }
                if ui.small_button("Remove").on_hover_text("Take this joint out of both parts").clicked() {
                    remove = Some(*index);
                }
            });
        }
        let another = egui::Id::new(("another_joint", a, b));
        if let Some(index) = remove {
            let message = doc.remove_joint(index).map_or_else(|e| e.to_string(), |()| "Joint removed.".to_owned());
            ui.data_mut(|d| d.insert_temp(egui::Id::new("joinery_message"), (a, b, message)));
        }
        if let Some((index, kind, sizes)) = change {
            match doc.remove_joint(index) {
                Ok(()) => {
                    // Back to the picker on the same joint, filled in with its sizes, to set it up again.
                    restore(ui, kind, &sizes, (a, b));
                    ui.data_mut(|d| {
                        d.insert_temp(egui::Id::new(("joint_choice", a, b)), Choice::of(kind));
                        d.insert_temp(another, true);
                        d.insert_temp(egui::Id::new("joinery_message"), (a, b, "Set it up again, then make it.".to_owned()));
                    });
                }
                Err(e) => {
                    ui.data_mut(|d| d.insert_temp(egui::Id::new("joinery_message"), (a, b, e.to_string())));
                }
            }
        } else {
            let open = ui.data(|d| d.get_temp::<bool>(another)).unwrap_or(false);
            let header = egui::CollapsingHeader::new("Add another joint").id_salt(another).open(open.then_some(true));
            let shown = header.show(ui, |ui| picker(ui, doc, (a, b), &fit));
            if shown.header_response.clicked() || open {
                ui.data_mut(|d| d.insert_temp(another, false));
            }
        }
    }
    show_message(ui, (a, b));
}

/// The joint picker: a joint, its sizes and the button that makes it.
fn picker(ui: &mut egui::Ui, doc: &mut Document, (a, b): (InstanceId, InstanceId), fit: &Fit) {
    let name = |doc: &Document, id: InstanceId| doc.instance_name(id).unwrap_or("Part").to_owned();
    let fit = fit.clone();
    let choice_id = egui::Id::new(("joint_choice", a, b));
    let suggested = Choice::ALL.into_iter().find(|c| c.fits(&fit)).unwrap_or(Choice::LetIn);
    let mut choice = ui.data(|d| d.get_temp::<Choice>(choice_id)).unwrap_or(suggested);
    ui.horizontal(|ui| {
        ui.label("Joint");
        egui::ComboBox::from_id_salt("joint_choice").selected_text(choice.name()).show_ui(ui, |ui| {
            for c in Choice::ALL {
                let text = if c.fits(&fit) { c.name().to_owned() } else { format!("{} (place the parts first)", c.name()) };
                ui.selectable_value(&mut choice, c, text);
            }
        });
    });
    ui.data_mut(|d| d.insert_temp(choice_id, choice));

    let mut message: Option<String> = None;
    if !choice.fits(&fit) {
        ui.weak(choice.setup());
    } else {
        match choice {
            Choice::Overlap => {
                for (cut, into) in [(a, b), (b, a)] {
                    let (cn, inn) = (name(doc, cut), name(doc, into));
                    if ui.button(format!("Cut {cn} into {inn}")).on_hover_text(format!("{cn} keeps its size; {inn} gets the cut")).clicked() {
                        message = Some(result(doc.cut_overlap(cut, into).map(|k| format!("{} cut in {inn}.", k.name()))));
                    }
                }
            }
            Choice::HalfLap => {
                if ui.button("Make the half-lap").clicked() {
                    message = Some(result(doc.half_lap(a, b).map(|()| "Half-lap made.".to_owned())));
                }
            }
            Choice::BoxJoint => {
                let [finger] = fields(ui, ("box", a, b), [("Finger width", fit.finger, "Rounded so there's an odd number of fingers")]);
                if button(ui, finger.is_some(), "Make the box joint") {
                    let finger = finger.unwrap_or(fit.finger);
                    message = Some(result(sized(doc, |doc| doc.box_joint(a, b, finger), vec![finger]).map(|()| "Box joint made.".to_owned())));
                }
            }
            Choice::LetIn
            | Choice::MortiseAndTenon
            | Choice::Dowels
            | Choice::Biscuits
            | Choice::LooseTenons
            | Choice::PocketScrews
            | Choice::Dovetails
            | Choice::Splines => {
                let (part, into) = fit.butt.expect("fits");
                let (pn, inn) = (name(doc, part), name(doc, into));
                ui.label(format!("{pn}'s end meets {inn}."));
                match choice {
                    Choice::LetIn => {
                        let [depth] = fields(ui, ("let_in", a, b), [("Depth", fit.let_in_depth, "How far it's let in; it grows by this")]);
                        if button(ui, depth.is_some(), &format!("Let {pn} into {inn}")) {
                            let depth = depth.unwrap_or(fit.let_in_depth);
                            message = Some(result(sized(doc, |doc| doc.let_in(part, into, depth), vec![depth]).map(|k| {
                                format!("{} cut in {inn}; {pn} is {} longer.", k.name(), format_length(depth))
                            })));
                        }
                    }
                    Choice::Dovetails => {
                        let (_, wide) = fit.contact;
                        let blind_id = egui::Id::new(("half_blind", a, b));
                        let mut half_blind = ui.data(|d| d.get_temp::<bool>(blind_id)).unwrap_or(false);
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut half_blind, false, "Through").on_hover_text("The tails show on both faces of the corner");
                            ui.selectable_value(&mut half_blind, true, "Half-blind").on_hover_text("Hidden from the front, as on a drawer front");
                        });
                        ui.data_mut(|d| d.insert_temp(blind_id, half_blind));
                        let [tails, pin, half_pin] = fields(
                            ui,
                            ("dovetails", a, b),
                            [
                                ("Tails", f64::from(Dovetails::suggested(wide)), "How many tails across the width"),
                                ("Pins", DOVETAIL_PIN, "Each pin's width where it shows: narrow (1/8\") for a hand-cut look, wider for a machine-cut one"),
                                ("Half pins", DOVETAIL_HALF_PIN, "The pins at each edge; the tails share the rest"),
                            ],
                        );
                        ui.weak(format!("{pn} gets the tails, {inn} the pins (1:8 slope)."));
                        if let (true, Some(tails), Some(pin), Some(half_pin)) = (button(ui, tails.is_some() && pin.is_some() && half_pin.is_some(), "Cut the dovetails"), tails, pin, half_pin) {
                            let style = Dovetails { tails: tails.round().max(0.0) as u32, half_blind, pin, half_pin };
                            message = Some(result(sized(doc, |doc| doc.dovetails(part, into, style), vec![f64::from(style.tails), f64::from(u8::from(style.half_blind)), pin, half_pin]).map(|()| format!("Dovetails cut; {pn} is longer by the tails."))));
                        }
                    }
                    Choice::Splines => {
                        let spline = Spline::for_stock(fit.contact.0);
                        ui.weak(format!("A {} along the whole joint, glued into a groove in each part.", spline.piece));
                        if button(ui, true, "Cut the spline groove") {
                            message = Some(result(doc.slots(part, into, &spline).map(|()| "Spline groove cut in both parts.".to_owned())));
                        }
                    }
                    Choice::Biscuits | Choice::LooseTenons => {
                        let (thick, wide) = fit.contact;
                        let biscuits = choice == Choice::Biscuits;
                        let size_id = egui::Id::new(("slot_size", biscuits, a, b));
                        let suggested = if biscuits { Slots::suggested_biscuit(thick, wide) } else { Slots::suggested_loose_tenon(thick) };
                        let mut size = ui.data(|d| d.get_temp::<usize>(size_id)).unwrap_or(suggested);
                        let names: Vec<String> = if biscuits {
                            BISCUITS.iter().map(|b| b.0.to_owned()).collect()
                        } else {
                            LOOSE_TENONS.iter().map(|(t, w, l)| format!("{t} x {w} x {l} mm")).collect()
                        };
                        ui.horizontal(|ui| {
                            ui.label("Size");
                            egui::ComboBox::from_id_salt(size_id).selected_text(&names[size]).show_ui(ui, |ui| {
                                for (i, n) in names.iter().enumerate() {
                                    ui.selectable_value(&mut size, i, n);
                                }
                            });
                        });
                        ui.data_mut(|d| d.insert_temp(size_id, size));
                        let mut slots = if biscuits { Slots::biscuits(size, wide) } else { Slots::loose_tenons(size, wide) };
                        let [count, room] = fields(
                            ui,
                            ("slot_count", a, b),
                            [("Count", f64::from(slots.count), "Spread evenly across the joint"), ("Fit", 0.0, "How much roomier the slots are than the pieces (thicker, longer, deeper); 0 for a tight fit")],
                        );
                        let label = if biscuits { "Cut biscuit slots" } else { "Cut loose tenon mortises" };
                        if let (true, Some(count), Some(room)) = (button(ui, count.is_some() && room.is_some(), label), count, room) {
                            slots.count = count.round().max(0.0) as u32;
                            slots.fit = room;
                            message = Some(result(sized(doc, |doc| doc.slots(part, into, &slots), vec![size as f64, f64::from(slots.count), slots.fit]).map(|()| format!("{} slots cut in each part, for {} x {}.", slots.count, slots.count, slots.piece))));
                        }
                    }
                    Choice::PocketScrews => {
                        let (thick, wide) = fit.contact;
                        let [count, screw] = fields(
                            ui,
                            ("pocket", a, b),
                            [
                                ("Count", ((wide / 4.0).round()).max(2.0), "About one every 4\""),
                                ("Screw", pocket_screw_for(thick), "Length, for the thickness of the part they go through"),
                            ],
                        );
                        ui.weak(format!("Drilled from the hidden face of {pn}; the holes aren't drawn."));
                        if let (true, Some(count), Some(screw)) = (button(ui, count.is_some() && screw.is_some(), "Add pocket screws"), count, screw) {
                            let n = count.round().max(0.0) as u32;
                            message = Some(result(sized(doc, |doc| doc.pocket_screws(part, into, n, screw), vec![f64::from(n), screw]).map(|()| format!("{n} pocket screws added."))));
                        }
                    }
                    Choice::MortiseAndTenon => {
                        let t = fit.tenon;
                        // Through: the tenon goes all the way and shows on the far side.
                        let through_id = egui::Id::new(("through", a, b));
                        let mut through = ui.data(|d| d.get_temp::<bool>(through_id)).unwrap_or(false);
                        ui.checkbox(&mut through, format!("Through tenon ({} long, shows on the far side)", format_length(fit.depth)));
                        ui.data_mut(|d| d.insert_temp(through_id, through));
                        // Haunch: a short stub filling the shoulder at the end, for a panel groove there.
                        let haunch_id = egui::Id::new(("haunch", a, b));
                        let mut haunched = ui.data(|d| d.get_temp::<bool>(haunch_id)).unwrap_or(false);
                        ui.checkbox(&mut haunched, "Haunch (fills a panel groove at the end)");
                        ui.data_mut(|d| d.insert_temp(haunch_id, haunched));
                        let [length, thickness, shoulder, allowance] = fields(
                            ui,
                            ("tenon", a, b),
                            [
                                ("Length", t.length, "How far the tenon goes into the mortise"),
                                ("Thickness", t.thickness, "About a third of the rail is usual"),
                                ("Shoulders", t.shoulder, "Set in from the rail's edges, each side"),
                                ("Fit", t.fit, "How much bigger the mortise is than the tenon (wider, thicker, deeper); 0 for a tight fit"),
                            ],
                        );
                        if let (true, Some(length), Some(thickness), Some(shoulder), Some(allowance)) = (
                            button(ui, length.is_some() && thickness.is_some() && shoulder.is_some() && allowance.is_some(), &format!("Tenon on {pn}, mortise in {inn}")),
                            length,
                            thickness,
                            shoulder,
                            allowance,
                        ) {
                            let length = if through { fit.depth } else { length };
                            let haunch = if haunched { 0.375f64.min(length / 2.0) } else { 0.0 };
                            message = Some(result(sized(doc, |doc| doc.mortise_and_tenon(part, into, Tenon { length, thickness, shoulder, haunch, fit: allowance }), vec![length, thickness, shoulder, allowance, f64::from(u8::from(through)), f64::from(u8::from(haunched))]).map(|()| {
                                format!("Mortise and tenon made; {pn} is {} longer in the cut list.", format_length(length))
                            })));
                        }
                    }
                    _ => {
                        let d = fit.dowels;
                        let [count, diameter, depth, allowance] = fields(
                            ui,
                            ("dowels", a, b),
                            [
                                ("Count", f64::from(d.count), "Spread evenly across the width"),
                                ("Diameter", d.diameter, "3/8\" for 3/4\" stock is usual"),
                                ("Depth", d.depth, "Hole depth in each part"),
                                ("Fit", d.fit, "How much bigger the holes are than the dowels; 0 for a tight fit"),
                            ],
                        );
                        if let (true, Some(count), Some(diameter), Some(depth), Some(allowance)) =
                            (button(ui, count.is_some() && diameter.is_some() && depth.is_some() && allowance.is_some(), "Drill for dowels"), count, diameter, depth, allowance)
                        {
                            let dowels = Dowels { count: count.round().max(0.0) as u32, diameter, depth, fit: allowance };
                            message = Some(result(sized(doc, |doc| doc.dowels(part, into, dowels), vec![f64::from(dowels.count), dowels.diameter, dowels.depth, dowels.fit]).map(|()| format!("{} dowel holes drilled in each part.", dowels.count))));
                        }
                    }
                }
            }
        }
    }

    if let Some(m) = message {
        ui.data_mut(|d| d.insert_temp(egui::Id::new("joinery_message"), (a, b, m)));
    }
}

/// What the last joint made (or why not), for this pair.
fn show_message(ui: &mut egui::Ui, (a, b): (InstanceId, InstanceId)) {
    if let Some((ma, mb, m)) = ui.data(|d| d.get_temp::<(InstanceId, InstanceId, String)>(egui::Id::new("joinery_message")))
        && (ma, mb) == (a, b)
    {
        ui.weak(m);
    }
}

fn result(r: Result<String, woodsketch_doc::joinery::JoineryError>) -> String {
    r.unwrap_or_else(|e| e.to_string())
}

fn button(ui: &mut egui::Ui, enabled: bool, text: &str) -> bool {
    ui.add_enabled(enabled, egui::Button::new(text)).clicked()
}

/// Editable sizes (kept while the same parts and joint are picked), parsed: `None` where the
/// text isn't a length.
fn fields<const N: usize>(ui: &mut egui::Ui, key: (&str, InstanceId, InstanceId), specs: [(&str, f64, &str); N]) -> [Option<f64>; N] {
    let id = egui::Id::new(key);
    let mut texts: Vec<String> = ui.data(|d| d.get_temp::<Vec<String>>(id)).unwrap_or_else(|| {
        specs.iter().map(|(label, value, _)| if *label == "Count" { format!("{value}") } else { format_length(*value) }).collect()
    });
    egui::Grid::new(id).num_columns(2).show(ui, |ui| {
        for (i, (label, _, hint)) in specs.iter().enumerate() {
            ui.label(*label).on_hover_text(*hint);
            ui.add(egui::TextEdit::singleline(&mut texts[i]).desired_width(70.0));
            ui.end_row();
        }
    });
    // Sizes must be over 0; a fit allowance can be 0.
    let parsed = std::array::from_fn(|i| parse_length(&texts[i]).ok().filter(|v| *v > 0.0 || (specs[i].0 == "Fit" && *v == 0.0)));
    ui.data_mut(|d| d.insert_temp(id, texts));
    parsed
}

/// The joints a part has, for Entity Info.
pub fn part_joints(ui: &mut egui::Ui, doc: &mut Document, part: InstanceId) {
    let joints = doc.joint_rows(part);
    if joints.is_empty() {
        return;
    }
    ui.label("Joints");
    let mut remove = None;
    for row in joints {
        let (index, joint, other) = (row.index, row.joint, row.other);
        let at = row.at.map(|at| format!(" (at the {at})")).unwrap_or_default();
        ui.horizontal_wrapped(|ui| {
            ui.weak(format!("  {} with {other}{at}, {}", joint.kind.name(), joint.detail));
            if ui.small_button("Remove").on_hover_text("Take this joint out of both parts").clicked() {
                remove = Some(index);
            }
        });
    }
    if let Some(index) = remove
        && let Err(e) = doc.remove_joint(index)
    {
        ui.data_mut(|d| d.insert_temp(egui::Id::new(("part_joint_error", part)), e.to_string()));
    }
    if let Some(error) = ui.data(|d| d.get_temp::<String>(egui::Id::new(("part_joint_error", part)))) {
        ui.colored_label(ui.visuals().warn_fg_color, error);
    }
}

/// Two parts meeting at an angle (a brace into a post): an angled mortise and tenon or a housing,
/// either way round, where one is drawn into the other.
fn angled(ui: &mut egui::Ui, doc: &mut Document, (a, b): (InstanceId, InstanceId)) {
    use woodsketch_doc::angled::AngledJoint;
    let fit = match doc.angled_fit(a, b) {
        Ok(fit) => fit,
        Err(e) => {
            ui.weak(e.to_string());
            return;
        }
    };
    ui.label(format!("They meet at {:.0}°.", fit.angle));
    let name = |doc: &Document, id: InstanceId| doc.instance_name(id).unwrap_or("Part").to_owned();
    // Already joined: listed, each with Remove.
    let other = doc.container().instance(b).map(|i| i.definition);
    let joints: Vec<_> = doc.joint_list(a).into_iter().filter(|(_, j, _)| Some(j.cut) == other || Some(j.into) == other).collect();
    let mut remove = None;
    let mut change = None;
    for (index, joint, _) in &joints {
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("{} {}, {}", egui_phosphor::regular::CHECK, joint.kind.name(), joint.detail));
            if ui.small_button("Change").on_hover_text("Take it out and set it up again from its sizes").clicked() {
                change = Some((*index, joint.sizes.clone()));
            }
            if ui.small_button("Remove").on_hover_text("Take this joint out of both parts").clicked() {
                remove = Some(*index);
            }
        });
    }
    if let Some(index) = remove {
        let message = doc.remove_joint(index).map_or_else(|e| e.to_string(), |()| "Joint removed.".to_owned());
        ui.data_mut(|d| d.insert_temp(egui::Id::new("joinery_message"), (a, b, message)));
    }
    if let Some((index, sizes)) = change {
        let message = match doc.remove_joint(index) {
            Ok(()) => {
                // Back to the choice and sizes it was made with: [thickness, shoulders, housing, fit].
                if let [thickness, shoulder, housing, ..] = sizes[..] {
                    let fit = sizes.get(3).copied().unwrap_or(0.0);
                    ui.data_mut(|d| {
                        d.insert_temp(egui::Id::new(("angled_tenon", a, b)), housing > 0.5);
                        d.insert_temp(egui::Id::new(("angled", a, b)), vec![format_length(thickness), format_length(shoulder), format_length(fit)]);
                    });
                }
                "Set it up again, then make it.".to_owned()
            }
            Err(e) => e.to_string(),
        };
        ui.data_mut(|d| d.insert_temp(egui::Id::new("joinery_message"), (a, b, message)));
    }
    if joints.is_empty() {
        if !fit.overlap {
            ui.weak("Draw one part's end into the other where the joint goes: the overlap sets how deep it is.");
        } else {
            let tenon_id = egui::Id::new(("angled_tenon", a, b));
            let mut housing = ui.data(|d| d.get_temp::<bool>(tenon_id)).unwrap_or(false);
            ui.horizontal(|ui| {
                ui.selectable_value(&mut housing, false, "Mortise and tenon").on_hover_text("A tenon through the middle, into a mortise of its shape");
                ui.selectable_value(&mut housing, true, "Housing").on_hover_text("The whole end let into the other part");
            });
            ui.data_mut(|d| d.insert_temp(tenon_id, housing));
            let mut message = None;
            if housing {
                for (part, into) in [(a, b), (b, a)] {
                    let (pn, inn) = (name(doc, part), name(doc, into));
                    if ui.button(format!("Let {pn} into {inn}")).clicked() {
                        message = Some(result(sized(doc, |doc| doc.angled_joint(part, into, AngledJoint::Housing), vec![0.0, 0.0, 1.0]).map(|()| format!("{pn} let into {inn}."))));
                    }
                }
            } else {
                // A third of the thinner part, to the nearest 1/8".
                let third = ((fit.thickness.0.min(fit.thickness.1) / 3.0) * 8.0).round().max(1.0) / 8.0;
                let [thickness, shoulder, room] = fields(
                    ui,
                    ("angled", a, b),
                    [
                        ("Thickness", third, "Through the middle of the tenoned part; about a third"),
                        ("Shoulders", 0.25, "Set in from its edges, each side"),
                        ("Fit", 0.0, "How much smaller the tenon is than its mortise (thinner, narrower, shorter); 0 for a tight fit"),
                    ],
                );
                for (part, into) in [(a, b), (b, a)] {
                    let (pn, inn) = (name(doc, part), name(doc, into));
                    let ready = thickness.is_some() && shoulder.is_some() && room.is_some();
                    if let (true, Some(thickness), Some(shoulder), Some(fit)) = (button(ui, ready, &format!("Tenon on {pn}, mortise in {inn}")), thickness, shoulder, room) {
                        let joint = AngledJoint::MortiseAndTenon { thickness, shoulder, fit };
                        message = Some(result(sized(doc, |doc| doc.angled_joint(part, into, joint), vec![thickness, shoulder, 0.0, fit]).map(|()| format!("Angled mortise and tenon made: {pn}'s end is trimmed to {inn}'s face, with its tenon in."))));
                    }
                }
            }
            if let Some(message) = message {
                ui.data_mut(|d| d.insert_temp(egui::Id::new("joinery_message"), (a, b, message)));
            }
        }
    }
    show_message(ui, (a, b));
}

/// Make a joint with `make`; when it's made, keep the numbers it was made with on it (in the
/// picker's order), so Change can start from them.
fn sized<T, E>(doc: &mut Document, make: impl FnOnce(&mut Document) -> Result<T, E>, sizes: Vec<f64>) -> Result<T, E> {
    let made = make(doc);
    if made.is_ok() {
        doc.note_joint_sizes(sizes);
    }
    made
}

/// Fill the picker's fields for a joint of `kind` with the `sizes` it was made with (see
/// [`sized`]), so Change starts from them. Joints made before sizes were kept leave it as is.
fn restore(ui: &mut egui::Ui, kind: JointKind, sizes: &[f64], (a, b): (InstanceId, InstanceId)) {
    if sizes.is_empty() {
        return;
    }
    let length = |i: usize| sizes.get(i).map_or_else(String::new, |v| format_length(*v));
    let count = |i: usize| sizes.get(i).map_or_else(String::new, |v| format!("{v}"));
    let flag = |i: usize| sizes.get(i).is_some_and(|v| *v > 0.5);
    let texts = |key: &str, texts: Vec<String>| (egui::Id::new((key, a, b)), texts);
    let (id, values) = match Choice::of(kind) {
        Choice::LetIn => texts("let_in", vec![length(0)]),
        Choice::BoxJoint => texts("box", vec![length(0)]),
        Choice::Dovetails => {
            ui.data_mut(|d| d.insert_temp(egui::Id::new(("half_blind", a, b)), flag(1)));
            let or = |i: usize, default: f64| format_length(sizes.get(i).copied().unwrap_or(default));
            texts("dovetails", vec![count(0), or(2, DOVETAIL_PIN), or(3, DOVETAIL_HALF_PIN)])
        }
        Choice::Biscuits | Choice::LooseTenons => {
            let biscuits = kind == JointKind::Biscuits;
            let size = sizes.first().map_or(0, |v| *v as usize);
            ui.data_mut(|d| d.insert_temp(egui::Id::new(("slot_size", biscuits, a, b)), size));
            texts("slot_count", vec![count(1), length(2)])
        }
        Choice::PocketScrews => texts("pocket", vec![count(0), length(1)]),
        Choice::MortiseAndTenon => {
            ui.data_mut(|d| {
                d.insert_temp(egui::Id::new(("through", a, b)), flag(4));
                d.insert_temp(egui::Id::new(("haunch", a, b)), flag(5));
            });
            texts("tenon", vec![length(0), length(1), length(2), length(3)])
        }
        Choice::Dowels => texts("dowels", vec![count(0), length(1), length(2), length(3)]),
        Choice::Overlap | Choice::HalfLap | Choice::Splines => return,
    };
    ui.data_mut(|d| d.insert_temp(id, values));
}
