//! The Edge profile panel, in the tray when one part is selected: a router-bit shape and size,
//! and which edges, named by where they face in the drawing (Top, Front...).

use eframe::egui;
use glam::{DMat3, DVec3};
use woodsketch_doc::profile::{Edges, Profile};
use woodsketch_doc::units::{format_length, parse_length};
use woodsketch_i18n::t;

use woodsketch_doc::{Document, InstanceId};

/// The panel's choices for one part, kept while it's selected.
#[derive(Clone)]
struct Choices {
    profile: Profile,
    size: String,
    along: bool,
    /// The face (its own axis and end) whose edges are profiled.
    face: (usize, bool),
    sides: [bool; 4],
    corners: [bool; 4],
}

/// Which way a direction in the part points, as a word this code compares against: it is asked
/// whether a face is the "Top" to decide what to profile by default, so it must not change with
/// the language. `facing_name` and `facing_lower` are what reaches the screen.
pub(crate) fn facing(world: DVec3) -> &'static str {
    let a = world.abs();
    if a.z >= a.x && a.z >= a.y {
        if world.z > 0.0 { "Top" } else { "Bottom" }
    } else if a.x >= a.y {
        if world.x > 0.0 { "Right" } else { "Left" }
    } else if world.y > 0.0 {
        "Back"
    } else {
        "Front"
    }
}

/// The same direction, named for showing: "Top".
pub(crate) fn facing_name(world: DVec3) -> String {
    match facing(world) {
        "Top" => t!("side-top"),
        "Bottom" => t!("side-bottom"),
        "Left" => t!("side-left"),
        "Right" => t!("side-right"),
        "Back" => t!("side-back"),
        _ => t!("side-front"),
    }
}

/// And as it reads inside a sentence: "around the top". English only lowercases it; a language
/// that inflects it has its own message to put here.
pub(crate) fn facing_lower(world: DVec3) -> String {
    match facing(world) {
        "Top" => t!("side-top-lower"),
        "Bottom" => t!("side-bottom-lower"),
        "Left" => t!("side-left-lower"),
        "Right" => t!("side-right-lower"),
        "Back" => t!("side-back-lower"),
        _ => t!("side-front-lower"),
    }
}

pub fn show(ui: &mut egui::Ui, doc: &mut Document, part: InstanceId) {
    let Some(instance) = doc.container().instance(part).copied() else { return };
    let Some(extents) = doc.part_extents(part) else {
        ui.weak(t!("profile-assembly"));
        return;
    };
    let axes: DMat3 = doc.context_transform().matrix3 * instance.transform.matrix3;
    let name = |axis: usize, positive: bool| facing(axes * (DVec3::AXES[axis] * if positive { 1.0 } else { -1.0 }));
    let dir = |axis: usize, positive: bool| axes * (DVec3::AXES[axis] * if positive { 1.0 } else { -1.0 });
    let longest = (0..3).max_by(|&a, &b| extents[a].total_cmp(&extents[b])).unwrap_or(0);

    let id = egui::Id::new(("profile_choices", part));
    let mut c = ui.data(|d| d.get_temp::<Choices>(id)).unwrap_or_else(|| {
        // Legs and posts (standing, long and narrow): their long corners. Otherwise the
        // edges around the face that's up.
        let up = (0..3).flat_map(|a| [(a, true), (a, false)]).find(|&(a, p)| name(a, p) == "Top").unwrap_or((2, true));
        let post = name(longest, true) == "Top" || name(longest, false) == "Top";
        Choices { profile: Profile::Roundover, size: "1/4\"".into(), along: post, face: up, sides: [true; 4], corners: [true; 4] }
    });

    egui::Grid::new(("profile_grid", part)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
        ui.label(t!("profile-shape"));
        egui::ComboBox::from_id_salt(("profile_shape", part)).selected_text(c.profile.name()).show_ui(ui, |ui| {
            for p in Profile::ALL {
                ui.selectable_value(&mut c.profile, p, p.name());
            }
        });
        ui.end_row();
        ui.label(t!("profile-size"));
        ui.add(egui::TextEdit::singleline(&mut c.size).desired_width(70.0));
        ui.end_row();
    });
    // Wrapped, so the panel never widens the tray.
    ui.horizontal_wrapped(|ui| {
        ui.selectable_value(&mut c.along, false, t!("profile-around-face"));
        ui.selectable_value(&mut c.along, true, t!("profile-along-length"));
    });

    let u_of = |axis: usize| ((axis + 1) % 3, (axis + 2) % 3);
    if c.along {
        let (u, v) = u_of(longest);
        // Corners [(u-, v-), (u+, v-), (u+, v+), (u-, v+)], named by the two faces they join.
        let corner_names = [(false, false), (true, false), (true, true), (false, true)]
            .map(|(pu, pv)| t!("profile-corner", "first" => facing_name(dir(v, pv)), "second" => facing_lower(dir(u, pu))));
        ui.horizontal_wrapped(|ui| {
            for (i, label) in corner_names.iter().enumerate() {
                ui.checkbox(&mut c.corners[i], label);
            }
        });
    } else {
        let faces: Vec<(usize, bool)> = (0..3).flat_map(|a| [(a, true), (a, false)]).collect();
        let around = |f: (usize, bool)| t!("profile-around", "side" => facing_lower(dir(f.0, f.1)));
        egui::ComboBox::from_id_salt(("profile_face", part)).selected_text(around(c.face)).show_ui(ui, |ui| {
            for f in faces {
                ui.selectable_value(&mut c.face, f, around(f));
            }
        });
        let (u, v) = u_of(c.face.0);
        let side_names = [facing_name(dir(u, false)), facing_name(dir(u, true)), facing_name(dir(v, false)), facing_name(dir(v, true))];
        ui.horizontal_wrapped(|ui| {
            for (i, label) in side_names.iter().enumerate() {
                ui.checkbox(&mut c.sides[i], t!("profile-edge", "edge" => label.as_str()));
            }
        });
    }

    let size = parse_length(&c.size).ok().filter(|s| *s > 0.0);
    let edges = if c.along {
        Edges::Along { axis: longest, corners: c.corners }
    } else {
        Edges::AroundFace { axis: c.face.0, positive: c.face.1, sides: c.sides }
    };
    let message_id = egui::Id::new(("profile_message", part));
    if ui.add_enabled(size.is_some(), egui::Button::new(t!("profile-rout", "profile" => c.profile.name().to_lowercase()))).clicked()
        && let Some(size) = size
    {
        let message = match doc.profile_edges(part, c.profile, size, edges) {
            Ok(()) => t!("profile-routed", "size" => format_length(size), "profile" => c.profile.name().to_lowercase()),
            Err(e) => e.to_string(),
        };
        ui.data_mut(|d| d.insert_temp(message_id, message));
    }
    if let Some(m) = ui.data(|d| d.get_temp::<String>(message_id)) {
        ui.weak(m);
    }
    ui.data_mut(|d| d.insert_temp(id, c));
}

/// The profiles a part has, for Entity Info.
pub fn part_profiles(ui: &mut egui::Ui, doc: &Document, part: InstanceId) {
    let profiles = doc.profiles_of(part);
    if profiles.is_empty() {
        return;
    }
    ui.label(t!("profile-edge-profiles"));
    for p in profiles {
        ui.weak(format!("  {}", p.describe()));
    }
}
