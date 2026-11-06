//! The Joinery panel, in the tray when two parts (groups or components) are selected: pick a
//! joint, set its sizes, and make it. Joints that don't fit how the parts sit say how to place
//! them. Entity Info lists the joints a part has.

use eframe::egui;
use woodsketch_i18n::t;
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

    fn name(self) -> String {
        match self {
            Self::LetIn => t!("joinery-kind-let-in"),
            Self::Overlap => t!("joinery-kind-overlap"),
            Self::MortiseAndTenon => t!("joinery-kind-mortise-tenon"),
            Self::Dowels => t!("joinery-kind-dowels"),
            Self::Biscuits => t!("joinery-kind-biscuits"),
            Self::LooseTenons => t!("joinery-kind-loose-tenons"),
            Self::PocketScrews => t!("joinery-kind-pocket-screws"),
            Self::Dovetails => t!("joinery-kind-dovetails"),
            Self::Splines => t!("joinery-kind-splines"),
            Self::BoxJoint => t!("joinery-kind-box-joint"),
            Self::HalfLap => t!("joinery-kind-half-lap"),
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
    fn setup(self) -> String {
        match self {
            Self::LetIn => t!("joinery-how-let-in"),
            Self::MortiseAndTenon => t!("joinery-how-mortise-tenon"),
            Self::Dowels | Self::Biscuits | Self::LooseTenons | Self::Splines | Self::PocketScrews => t!("joinery-how-butt"),
            Self::Dovetails => t!("joinery-how-dovetails"),
            Self::BoxJoint => t!("joinery-how-box-joint"),
            Self::HalfLap => t!("joinery-how-half-lap"),
            Self::Overlap => t!("joinery-how-overlap"),
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
            ui.weak(t!("joinery-elsewhere", "joints" => elsewhere.join(", ")));
        }
        picker(ui, doc, (a, b), &fit);
    } else {
        ui.label(t!("joinery-already"));
        let mut change = None;
        let mut remove = None;
        for (index, joint, _) in &joints {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("{} {}, {}", egui_phosphor::regular::CHECK, joint.kind.name(), joint.detail));
                if ui.small_button(t!("joinery-change")).on_hover_text(t!("joinery-change-hint")).clicked() {
                    change = Some((*index, joint.kind, joint.sizes.clone()));
                }
                if ui.small_button(t!("joinery-remove")).on_hover_text(t!("joinery-remove-hint")).clicked() {
                    remove = Some(*index);
                }
            });
        }
        let another = egui::Id::new(("another_joint", a, b));
        if let Some(index) = remove {
            let message = doc.remove_joint(index).map_or_else(|e| e.to_string(), |()| t!("joinery-removed"));
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
                        d.insert_temp(egui::Id::new("joinery_message"), (a, b, t!("joinery-set-up-again")));
                    });
                }
                Err(e) => {
                    ui.data_mut(|d| d.insert_temp(egui::Id::new("joinery_message"), (a, b, e.to_string())));
                }
            }
        } else {
            let open = ui.data(|d| d.get_temp::<bool>(another)).unwrap_or(false);
            let header = egui::CollapsingHeader::new(t!("joinery-add-another")).id_salt(another).open(open.then_some(true));
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
    let name = |doc: &Document, id: InstanceId| doc.instance_name(id).map_or_else(|| t!("joinery-part-default"), str::to_owned);
    let fit = fit.clone();
    let choice_id = egui::Id::new(("joint_choice", a, b));
    let suggested = Choice::ALL.into_iter().find(|c| c.fits(&fit)).unwrap_or(Choice::LetIn);
    let mut choice = ui.data(|d| d.get_temp::<Choice>(choice_id)).unwrap_or(suggested);
    ui.horizontal(|ui| {
        ui.label(t!("joinery-joint"));
        egui::ComboBox::from_id_salt("joint_choice").selected_text(choice.name()).show_ui(ui, |ui| {
            for c in Choice::ALL {
                let text = if c.fits(&fit) { c.name() } else { t!("joinery-place-first", "joint" => c.name()) };
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
                    if ui
                        .button(t!("joinery-cut-into", "cut" => cn.as_str(), "into" => inn.as_str()))
                        .on_hover_text(t!("joinery-cut-into-hint", "cut" => cn.as_str(), "into" => inn.as_str()))
                        .clicked()
                    {
                        message = Some(result(doc.cut_overlap(cut, into).map(|k| t!("joinery-cut-in", "kind" => k.name(), "into" => inn.as_str()))));
                    }
                }
            }
            Choice::HalfLap => {
                if ui.button(t!("joinery-make-half-lap")).clicked() {
                    message = Some(result(doc.half_lap(a, b).map(|()| t!("joinery-half-lap-made"))));
                }
            }
            Choice::BoxJoint => {
                let [finger] = fields(ui, ("box", a, b), [("Finger width", &t!("joinery-finger-width"), fit.finger, &t!("joinery-finger-width-hint"))]);
                if button(ui, finger.is_some(), &t!("joinery-make-box-joint")) {
                    let finger = finger.unwrap_or(fit.finger);
                    message = Some(result(sized(doc, |doc| doc.box_joint(a, b, finger), vec![finger]).map(|()| t!("joinery-box-joint-made"))));
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
                ui.label(t!("joinery-end-meets", "part" => pn.as_str(), "into" => inn.as_str()));
                match choice {
                    Choice::LetIn => {
                        let [depth] = fields(ui, ("let_in", a, b), [("Depth", &t!("joinery-depth"), fit.let_in_depth, &t!("joinery-depth-hint"))]);
                        if button(ui, depth.is_some(), &t!("joinery-let-into", "part" => pn.as_str(), "into" => inn.as_str())) {
                            let depth = depth.unwrap_or(fit.let_in_depth);
                            message = Some(result(sized(doc, |doc| doc.let_in(part, into, depth), vec![depth]).map(|k| {
                                t!("joinery-let-in-done", "kind" => k.name(), "into" => inn.as_str(), "part" => pn.as_str(), "length" => format_length(depth))
                            })));
                        }
                    }
                    Choice::Dovetails => {
                        let (_, wide) = fit.contact;
                        let blind_id = egui::Id::new(("half_blind", a, b));
                        let mut half_blind = ui.data(|d| d.get_temp::<bool>(blind_id)).unwrap_or(false);
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut half_blind, false, t!("joinery-through")).on_hover_text(t!("joinery-through-hint"));
                            ui.selectable_value(&mut half_blind, true, t!("joinery-half-blind")).on_hover_text(t!("joinery-half-blind-hint"));
                        });
                        ui.data_mut(|d| d.insert_temp(blind_id, half_blind));
                        let [tails, pin, half_pin] = fields(
                            ui,
                            ("dovetails", a, b),
                            [
                                ("Tails", &t!("joinery-tails"), f64::from(Dovetails::suggested(wide)), &t!("joinery-tails-hint")),
                                ("Pins", &t!("joinery-pins"), DOVETAIL_PIN, &t!("joinery-pins-hint")),
                                ("Half pins", &t!("joinery-half-pins"), DOVETAIL_HALF_PIN, &t!("joinery-half-pins-hint")),
                            ],
                        );
                        ui.weak(t!("joinery-dovetail-note", "part" => pn.as_str(), "into" => inn.as_str()));
                        if let (true, Some(tails), Some(pin), Some(half_pin)) = (button(ui, tails.is_some() && pin.is_some() && half_pin.is_some(), &t!("joinery-cut-dovetails")), tails, pin, half_pin) {
                            let style = Dovetails { tails: tails.round().max(0.0) as u32, half_blind, pin, half_pin };
                            message = Some(result(sized(doc, |doc| doc.dovetails(part, into, style), vec![f64::from(style.tails), f64::from(u8::from(style.half_blind)), pin, half_pin]).map(|()| t!("joinery-dovetails-done", "part" => pn.as_str()))));
                        }
                    }
                    Choice::Splines => {
                        let spline = Spline::for_stock(fit.contact.0);
                        ui.weak(t!("joinery-spline-note", "piece" => spline.piece.as_str()));
                        if button(ui, true, &t!("joinery-cut-spline")) {
                            message = Some(result(doc.slots(part, into, &spline).map(|()| t!("joinery-spline-done"))));
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
                            ui.label(t!("joinery-size"));
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
                            [
                                ("Count", &t!("joinery-count"), f64::from(slots.count), &t!("joinery-count-hint-slots")),
                                ("Fit", &t!("joinery-fit"), 0.0, &t!("joinery-fit-hint-slots")),
                            ],
                        );
                        let label = if biscuits { t!("joinery-cut-biscuit-slots") } else { t!("joinery-cut-loose-tenon-mortises") };
                        if let (true, Some(count), Some(room)) = (button(ui, count.is_some() && room.is_some(), &label), count, room) {
                            slots.count = count.round().max(0.0) as u32;
                            slots.fit = room;
                            message = Some(result(sized(doc, |doc| doc.slots(part, into, &slots), vec![size as f64, f64::from(slots.count), slots.fit]).map(|()| t!("joinery-slots-done", "count" => slots.count, "piece" => slots.piece.as_str()))));
                        }
                    }
                    Choice::PocketScrews => {
                        let (thick, wide) = fit.contact;
                        let [count, screw] = fields(
                            ui,
                            ("pocket", a, b),
                            [
                                ("Count", &t!("joinery-count"), ((wide / 4.0).round()).max(2.0), &t!("joinery-count-hint-pocket")),
                                ("Screw", &t!("joinery-screw"), pocket_screw_for(thick), &t!("joinery-screw-hint")),
                            ],
                        );
                        ui.weak(t!("joinery-pocket-note", "part" => pn.as_str()));
                        if let (true, Some(count), Some(screw)) = (button(ui, count.is_some() && screw.is_some(), &t!("joinery-add-pocket-screws")), count, screw) {
                            let n = count.round().max(0.0) as u32;
                            message = Some(result(sized(doc, |doc| doc.pocket_screws(part, into, n, screw), vec![f64::from(n), screw]).map(|()| t!("joinery-pocket-done", "count" => n))));
                        }
                    }
                    Choice::MortiseAndTenon => {
                        let t = fit.tenon;
                        // Through: the tenon goes all the way and shows on the far side.
                        let through_id = egui::Id::new(("through", a, b));
                        let mut through = ui.data(|d| d.get_temp::<bool>(through_id)).unwrap_or(false);
                        ui.checkbox(&mut through, t!("joinery-through-tenon", "length" => format_length(fit.depth)));
                        ui.data_mut(|d| d.insert_temp(through_id, through));
                        // Haunch: a short stub filling the shoulder at the end, for a panel groove there.
                        let haunch_id = egui::Id::new(("haunch", a, b));
                        let mut haunched = ui.data(|d| d.get_temp::<bool>(haunch_id)).unwrap_or(false);
                        ui.checkbox(&mut haunched, t!("joinery-haunch"));
                        ui.data_mut(|d| d.insert_temp(haunch_id, haunched));
                        let [length, thickness, shoulder, allowance] = fields(
                            ui,
                            ("tenon", a, b),
                            [
                                ("Length", &t!("joinery-length"), t.length, &t!("joinery-length-hint")),
                                ("Thickness", &t!("joinery-thickness"), t.thickness, &t!("joinery-thickness-hint")),
                                ("Shoulders", &t!("joinery-shoulders"), t.shoulder, &t!("joinery-shoulders-hint")),
                                ("Fit", &t!("joinery-fit"), t.fit, &t!("joinery-fit-hint-mortise")),
                            ],
                        );
                        if let (true, Some(length), Some(thickness), Some(shoulder), Some(allowance)) = (
                            button(ui, length.is_some() && thickness.is_some() && shoulder.is_some() && allowance.is_some(), &t!("joinery-tenon-mortise", "part" => pn.as_str(), "into" => inn.as_str())),
                            length,
                            thickness,
                            shoulder,
                            allowance,
                        ) {
                            let length = if through { fit.depth } else { length };
                            let haunch = if haunched { 0.375f64.min(length / 2.0) } else { 0.0 };
                            message = Some(result(sized(doc, |doc| doc.mortise_and_tenon(part, into, Tenon { length, thickness, shoulder, haunch, fit: allowance }), vec![length, thickness, shoulder, allowance, f64::from(u8::from(through)), f64::from(u8::from(haunched))]).map(|()| {
                                t!("joinery-mortise-tenon-done", "part" => pn.as_str(), "length" => format_length(length))
                            })));
                        }
                    }
                    _ => {
                        let d = fit.dowels;
                        let [count, diameter, depth, allowance] = fields(
                            ui,
                            ("dowels", a, b),
                            [
                                ("Count", &t!("joinery-count"), f64::from(d.count), &t!("joinery-count-hint-dowels")),
                                ("Diameter", &t!("joinery-diameter"), d.diameter, &t!("joinery-diameter-hint")),
                                ("Depth", &t!("joinery-depth"), d.depth, &t!("joinery-depth-hint-dowels")),
                                ("Fit", &t!("joinery-fit"), d.fit, &t!("joinery-fit-hint-dowels")),
                            ],
                        );
                        if let (true, Some(count), Some(diameter), Some(depth), Some(allowance)) =
                            (button(ui, count.is_some() && diameter.is_some() && depth.is_some() && allowance.is_some(), &t!("joinery-drill-dowels")), count, diameter, depth, allowance)
                        {
                            let dowels = Dowels { count: count.round().max(0.0) as u32, diameter, depth, fit: allowance };
                            message = Some(result(sized(doc, |doc| doc.dowels(part, into, dowels), vec![f64::from(dowels.count), dowels.diameter, dowels.depth, dowels.fit]).map(|()| t!("joinery-dowels-done", "count" => dowels.count))));
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
fn fields<const N: usize>(ui: &mut egui::Ui, key: (&str, InstanceId, InstanceId), specs: [(&str, &str, f64, &str); N]) -> [Option<f64>; N] {
    let id = egui::Id::new(key);
    let mut texts: Vec<String> = ui.data(|d| d.get_temp::<Vec<String>>(id)).unwrap_or_else(|| {
        specs.iter().map(|(salt, _, value, _)| if *salt == "Count" { format!("{value}") } else { format_length(*value) }).collect()
    });
    egui::Grid::new(id).num_columns(2).show(ui, |ui| {
        for (i, (_, label, _, hint)) in specs.iter().enumerate() {
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
    ui.label(t!("joinery-joints"));
    let mut remove = None;
    for row in joints {
        let (index, joint, other) = (row.index, row.joint, row.other);
        let at = row.at.map(|at| format!(" {}", t!("joinery-at", "at" => at))).unwrap_or_default();
        ui.horizontal_wrapped(|ui| {
            ui.weak(format!("  {}", t!("joinery-row", "kind" => joint.kind.name(), "other" => other.as_str(), "at" => at.as_str(), "detail" => joint.detail.as_str())));
            if ui.small_button(t!("joinery-remove")).on_hover_text(t!("joinery-remove-hint")).clicked() {
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
    ui.label(t!("joinery-meet-angle", "angle" => format!("{:.0}", fit.angle)));
    let name = |doc: &Document, id: InstanceId| doc.instance_name(id).map_or_else(|| t!("joinery-part-default"), str::to_owned);
    // Already joined: listed, each with Remove.
    let other = doc.container().instance(b).map(|i| i.definition);
    let joints: Vec<_> = doc.joint_list(a).into_iter().filter(|(_, j, _)| Some(j.cut) == other || Some(j.into) == other).collect();
    let mut remove = None;
    let mut change = None;
    for (index, joint, _) in &joints {
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("{} {}, {}", egui_phosphor::regular::CHECK, joint.kind.name(), joint.detail));
            if ui.small_button(t!("joinery-change")).on_hover_text(t!("joinery-change-hint")).clicked() {
                change = Some((*index, joint.sizes.clone()));
            }
            if ui.small_button(t!("joinery-remove")).on_hover_text(t!("joinery-remove-hint")).clicked() {
                remove = Some(*index);
            }
        });
    }
    if let Some(index) = remove {
        let message = doc.remove_joint(index).map_or_else(|e| e.to_string(), |()| t!("joinery-removed"));
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
                        d.insert_temp(egui::Id::new(("angled_every", a, b)), sizes.get(4).is_some_and(|v| *v > 0.5));
                    });
                }
                t!("joinery-set-up-again")
            }
            Err(e) => e.to_string(),
        };
        ui.data_mut(|d| d.insert_temp(egui::Id::new("joinery_message"), (a, b, message)));
    }
    if joints.is_empty() {
        if !fit.overlap {
            ui.weak(t!("joinery-draw-into"));
        } else {
            let tenon_id = egui::Id::new(("angled_tenon", a, b));
            let mut housing = ui.data(|d| d.get_temp::<bool>(tenon_id)).unwrap_or(false);
            ui.horizontal(|ui| {
                ui.selectable_value(&mut housing, false, t!("joinery-kind-mortise-tenon")).on_hover_text(t!("joinery-angled-mortise-tenon-hint"));
                ui.selectable_value(&mut housing, true, t!("joinery-housing")).on_hover_text(t!("joinery-housing-hint"));
            });
            ui.data_mut(|d| d.insert_temp(tenon_id, housing));
            // Copies of a shared component: cut them all alike, or only this one (made its own).
            let shared = [a, b].iter().any(|&id| doc.container().instance(id).is_some_and(|i| doc.scene().instance_count(i.definition) > 1));
            let every_id = egui::Id::new(("angled_every", a, b));
            let mut every = ui.data(|d| d.get_temp::<bool>(every_id)).unwrap_or(false);
            if shared {
                ui.checkbox(&mut every, t!("joinery-every-copy"))
                    .on_hover_text(t!("joinery-every-copy-hint"));
                ui.data_mut(|d| d.insert_temp(every_id, every));
            }
            let every = shared && every;
            let flag = f64::from(u8::from(every));
            let mut message = None;
            if housing {
                for (part, into) in [(a, b), (b, a)] {
                    let (pn, inn) = (name(doc, part), name(doc, into));
                    if ui.button(t!("joinery-let-into", "part" => pn.as_str(), "into" => inn.as_str())).clicked() {
                        message = Some(result(sized(doc, |doc| doc.angled_joint_on(part, into, AngledJoint::Housing, every), vec![0.0, 0.0, 1.0, 0.0, flag]).map(|()| t!("joinery-let-into-done", "part" => pn.as_str(), "into" => inn.as_str()))));
                    }
                }
            } else {
                // A third of the thinner part, to the nearest 1/8".
                let third = ((fit.thickness.0.min(fit.thickness.1) / 3.0) * 8.0).round().max(1.0) / 8.0;
                let [thickness, shoulder, room] = fields(
                    ui,
                    ("angled", a, b),
                    [
                        ("Thickness", &t!("joinery-thickness"), third, &t!("joinery-angled-thickness-hint")),
                        ("Shoulders", &t!("joinery-shoulders"), 0.25, &t!("joinery-angled-shoulders-hint")),
                        ("Fit", &t!("joinery-fit"), 0.0, &t!("joinery-angled-fit-hint")),
                    ],
                );
                for (part, into) in [(a, b), (b, a)] {
                    let (pn, inn) = (name(doc, part), name(doc, into));
                    let ready = thickness.is_some() && shoulder.is_some() && room.is_some();
                    if let (true, Some(thickness), Some(shoulder), Some(fit)) = (button(ui, ready, &t!("joinery-tenon-mortise", "part" => pn.as_str(), "into" => inn.as_str())), thickness, shoulder, room) {
                        let joint = AngledJoint::MortiseAndTenon { thickness, shoulder, fit };
                        message = Some(result(sized(doc, |doc| doc.angled_joint_on(part, into, joint, every), vec![thickness, shoulder, 0.0, fit, flag]).map(|()| t!("joinery-angled-done", "part" => pn.as_str(), "into" => inn.as_str()))));
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
