//! The part section of Entity Info, for one selected group or component: its size measured
//! the way a woodworker does (length along the grain), the stock it's cut from, and its grain
//! direction and cut.

use eframe::egui;
use egui_phosphor::regular as ph;
use woodsketch_doc::cutlist::Part;
use woodsketch_doc::cutplan::{glue_up, match_stock};
use woodsketch_doc::part::{Cut, EdgeJoint, GlueUp, PartAxis};
use woodsketch_doc::units::format_length;
use woodsketch_i18n::t;

use woodsketch_doc::{Document, Entity, InstanceId, MaterialKind};

pub fn show(ui: &mut egui::Ui, doc: &mut Document, tools: &mut woodsketch_tools::Tools, instance: InstanceId) {
    let (Some(size), Some(extents), Some(info)) = (doc.part_size(instance), doc.part_extents(instance), doc.part_info(instance)) else {
        ui.weak(t!("part-assembly"));
        return;
    };
    let [length, width, thickness] = size;
    let material = doc.material_of(Entity::Instance(instance)).and_then(|m| doc.scene().material(m)).cloned();

    egui::Grid::new(("part_info", instance)).num_columns(2).spacing([8.0, 4.0]).show(ui, |ui| {
        ui.label(t!("part-size"));
        ui.label(format!("{} x {} x {}", format_length(length), format_length(width), format_length(thickness)))
            .on_hover_text(t!("part-size-hint"));
        ui.end_row();

        ui.label(t!("part-stock"));
        match &material {
            None => {
                ui.colored_label(ui.visuals().warn_fg_color, t!("part-needs-material"));
            }
            Some(m) => {
                let part = Part { count: 1, length, width, thickness, glue_up: info.glue_up, ..Part::default() };
                match match_stock(&part, m.kind, &m.standard) {
                    Ok(stock) => match glue_up(&part, &m.standard).filter(|_| m.kind == MaterialKind::SolidWood) {
                        Some(n) => {
                            ui.label(t!("part-stock-glued-up", "material" => m.name.as_str(), "stock" => stock.name.as_str(), "boards" => n));
                        }
                        None => {
                            ui.label(t!("part-stock-line", "material" => m.name.as_str(), "stock" => stock.name.as_str()));
                        }
                    },
                    Err(reason) if !matches!(m.kind, MaterialKind::Hardware | MaterialKind::Other) => {
                        ui.colored_label(ui.visuals().warn_fg_color, format!("{} {}", ph::WARNING, m.name)).on_hover_text(reason);
                    }
                    Err(_) => {
                        ui.label(&m.name);
                    }
                }
            }
        }
        ui.end_row();

        ui.label(t!("part-grain"));
        let side = |axis: PartAxis| t!("part-grain-along-side", "length" => format_length(extents[axis.index()]));
        let shown = info.grain.map_or_else(|| t!("part-grain-along-length"), side);
        let mut picked = None;
        egui::ComboBox::from_id_salt(("part_grain", instance)).selected_text(shown).show_ui(ui, |ui| {
            if ui.selectable_label(info.grain.is_none(), t!("part-grain-along-length")).on_hover_text(t!("part-grain-along-length-hint")).clicked() {
                picked = Some(None);
            }
            for axis in PartAxis::ALL {
                if ui.selectable_label(info.grain == Some(axis), side(axis)).clicked() {
                    picked = Some(Some(axis));
                }
            }
        });
        if let Some(grain) = picked {
            doc.edit_part(instance, |p| p.grain = grain);
        }
        ui.end_row();

        // The face and edge it's measured from, named by where they face in the drawing; stacked,
        // so the tray keeps its width.
        ui.label(t!("part-reference")).on_hover_text(t!("part-reference-hint"));
        let axes = doc.container().instance(instance).map(|i| doc.context_transform().matrix3 * i.transform.matrix3);
        let named = |side: Option<woodsketch_doc::part::Side>, unset: String| match (side, axes) {
            (Some(side), Some(axes)) => crate::profile::facing_name(axes * side.normal()),
            _ => unset,
        };
        ui.vertical(|ui| {
            ui.label(t!("part-reference-face-edge",
                "face" => named(info.reference_face, t!("part-reference-not-set")),
                "edge" => named(info.reference_edge, t!("part-reference-not-set"))));
            ui.label(t!("part-reference-end", "end" => named(info.reference_end, t!("part-reference-as-drawn"))));
            ui.horizontal(|ui| {
                let picking = tools.picking_reference();
                let pick = ui
                    .selectable_label(picking, format!("{} {}", ph::CURSOR_CLICK, t!("part-pick")))
                    .on_hover_text(t!("part-pick-hint"));
                if pick.clicked() {
                    if picking {
                        tools.cancel();
                    } else {
                        tools.pick_reference_on(instance);
                    }
                }
                if (info.reference_face.is_some() || info.reference_edge.is_some() || info.reference_end.is_some()) && ui.small_button(t!("part-clear")).clicked() {
                    doc.edit_part(instance, |p| {
                        p.reference_face = None;
                        p.reference_edge = None;
                        p.reference_end = None;
                    });
                }
            });
        });
        ui.end_row();

        // Sawn boards only: sheet goods don't move like solid wood.
        let sawn = material.as_ref().is_none_or(|m| matches!(m.kind, MaterialKind::SolidWood | MaterialKind::DimensionalLumber));
        if sawn {
            ui.label(t!("part-cut"));
            let mut picked = None;
            egui::ComboBox::from_id_salt(("part_cut", instance)).selected_text(info.cut.name()).show_ui(ui, |ui| {
                for cut in Cut::ALL {
                    let hint = match cut {
                        Cut::Unknown => t!("part-cut-unknown-hint"),
                        Cut::FlatSawn => t!("part-cut-flat-hint"),
                        Cut::QuarterSawn => t!("part-cut-quarter-hint"),
                    };
                    if ui.selectable_label(info.cut == cut, cut.name()).on_hover_text(hint).clicked() {
                        picked = Some(cut);
                    }
                }
            });
            if let Some(cut) = picked {
                doc.edit_part(instance, |p| p.cut = cut);
            }
            ui.end_row();

            // Glued solid, or fastened so it can move (the wood movement check lets it).
            ui.label(t!("part-movement"));
            let mut free = info.free_to_move;
            if ui
                .checkbox(&mut free, t!("part-free-to-move"))
                .on_hover_text(t!("part-free-to-move-hint"))
                .changed()
            {
                doc.edit_part(instance, |p| p.free_to_move = free);
            }
            ui.end_row();
        }

        // Solid wood: glued up from narrower boards (on its own when it's too wide for one).
        if let Some(m) = material.as_ref().filter(|m| m.kind == MaterialKind::SolidWood) {
            ui.label(t!("part-glue-up"));
            let part = Part { count: 1, length, width, thickness, glue_up: info.glue_up, ..Part::default() };
            let auto = Part { glue_up: GlueUp::Auto, ..part.clone() };
            let boards = |n: Option<u32>| n.map_or_else(|| t!("part-board-one"), |n| t!("part-boards-n", "count" => n));
            let label = |g: GlueUp| match g {
                GlueUp::Auto => t!("part-glue-auto", "boards" => boards(glue_up(&auto, &m.standard))),
                GlueUp::Boards(0 | 1) => t!("part-glue-one"),
                GlueUp::Boards(n) => t!("part-glue-from", "count" => n),
            };
            let mut picked = None;
            egui::ComboBox::from_id_salt(("part_glue", instance)).selected_text(label(info.glue_up)).show_ui(ui, |ui| {
                for g in [GlueUp::Auto, GlueUp::Boards(1), GlueUp::Boards(2), GlueUp::Boards(3), GlueUp::Boards(4), GlueUp::Boards(5)] {
                    if ui.selectable_label(info.glue_up == g, label(g)).clicked() {
                        picked = Some(g);
                    }
                }
            });
            if let Some(g) = picked {
                doc.edit_part(instance, |p| p.glue_up = g);
            }
            ui.end_row();

            // Glued up: what reinforces the seams.
            if glue_up(&part, &m.standard).is_some() {
                ui.label(t!("part-edge-joint"));
                let mut picked = None;
                egui::ComboBox::from_id_salt(("part_edge_joint", instance)).selected_text(info.edge_joint.name()).show_ui(ui, |ui| {
                    for e in EdgeJoint::ALL {
                        let response = ui.selectable_label(info.edge_joint == e, e.name());
                        let response = match e.piece(thickness) {
                            Some((piece, spacing)) => response.on_hover_text(t!("part-edge-joint-hint", "piece" => piece, "spacing" => format_length(spacing))),
                            None => response.on_hover_text(t!("part-edge-glued-hint")),
                        };
                        if response.clicked() {
                            picked = Some(e);
                        }
                    }
                });
                if let Some(e) = picked {
                    doc.edit_part(instance, |p| p.edge_joint = e);
                }
                ui.end_row();
            }
        }
    });
}
