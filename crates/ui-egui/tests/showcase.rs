//! Screenshots for woodsketch.net: real projects opened in the real app, nothing selected, framed
//! by the camera. They check only that each project builds and shows; set
//! `WOODSKETCH_SNAPSHOT_DIR` to save the PNGs (1200 x 800, as the site uses them).

use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use glam::{DAffine3, DVec3};
use woodsketch_doc::{Document, Entity, InstanceId};
use woodsketch_ui_egui::WoodSketchApp;

/// One at a time on the GPU (see `workflow.rs`).
static GPU: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn app() -> (Harness<'static, WoodSketchApp>, std::sync::MutexGuard<'static, ()>) {
    let gpu = GPU.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut harness = Harness::builder()
        .with_size([1200.0, 800.0])
        .with_step_dt(1.0 / 60.0)
        .with_theme(egui::Theme::Light)
        .wgpu()
        .build_eframe(|cc| WoodSketchApp::new(cc));
    harness.run();
    (harness, gpu)
}

/// Open the project `recipe` (JSON, as in `docs/AGENTS.md`) in the app.
fn open(harness: &mut Harness<'static, WoodSketchApp>, name: &str, recipe: &str) {
    let doc = woodsketch_doc::recipe::build(&woodsketch_doc::recipe::parse(recipe).unwrap()).unwrap_or_else(|e| panic!("{e:?}"));
    let path = std::env::temp_dir().join(format!("woodsketch-showcase-{name}-{}.woodsketch", std::process::id()));
    std::fs::write(&path, woodsketch_doc::file::to_string(doc.scene())).unwrap();
    harness.state_mut().open_path(&path).unwrap();
    let _ = std::fs::remove_file(&path);
}

/// Look from `yaw` degrees round and `pitch` up, the whole drawing in view.
fn frame(harness: &mut Harness<'static, WoodSketchApp>, yaw: f32, pitch: f32) {
    let camera = harness.state_mut().camera_mut();
    camera.yaw = yaw.to_radians();
    camera.pitch = pitch.to_radians();
    harness.state_mut().zoom_extents();
    for _ in 0..3 {
        harness.run();
    }
}

fn snapshot(harness: &mut Harness<'static, WoodSketchApp>, name: &str) {
    if let Ok(dir) = std::env::var("WOODSKETCH_SNAPSHOT_DIR") {
        let image = harness.render().expect("render the frame");
        image.save(std::path::Path::new(&dir).join(format!("{name}.png"))).expect("save snapshot");
    }
}

/// The part named `name` at the top of the drawing.
fn part(doc: &Document, name: &str) -> InstanceId {
    doc.container().instances().map(|(id, _)| id).find(|&id| doc.instance_name(id) == Some(name)).unwrap_or_else(|| panic!("no part {name}"))
}

/// Slide a part `by`, to show the joint it hides.
fn pull_apart(doc: &mut Document, name: &str, by: DVec3) {
    let id = part(doc, name);
    doc.transform_entities(&[Entity::Instance(id)], &[DAffine3::from_translation(by)], false).unwrap();
}

#[test]
fn a_walnut_cabinet_with_drawers_and_doors() {
    use woodsketch_doc::template::{Cabinet, DoorStyle, Fill, Template};
    let (mut harness, _gpu) = app();
    {
        let doc = harness.state_mut().document_mut();
        let mut c = Cabinet { width: 30.0, height: 34.0, depth: 22.0, shelves: 2, doors: DoorStyle::FrameAndPanel, ..Default::default() };
        c.fills[0] = Fill::Doors;
        c.fills[1] = Fill::Drawer;
        c.fills[2] = Fill::Drawer;
        let walnut = doc.add_preset(woodsketch_doc::library::LIBRARY.iter().find(|p| p.name == "Walnut").unwrap());
        doc.add_template(&Template::Cabinet(c), Some(walnut)).unwrap();
        doc.selection_mut().clear();
    }
    frame(&mut harness, -62.0, 22.0);
    assert!(harness.state().document().scene().joints.len() > 10, "joined throughout");
    snapshot(&mut harness, "site_cabinet");
}

#[test]
fn joints_pulled_apart() {
    let (mut harness, _gpu) = app();
    open(
        &mut harness,
        "joinery",
        r#"{
        "materials": [{ "id": "cherry", "library": "Cherry" }, { "id": "oak", "library": "White Oak" }],
        "parts": [
            { "name": "Leg", "material": "oak", "size": [14, 1.75, 1.75], "at": [0, 0, 0], "length": "z", "width": "x" },
            { "name": "Apron", "material": "oak", "size": [8, 4, 0.75], "at": [1.75, 0.5, 9], "length": "x", "width": "z" },
            { "name": "Drawer front", "material": "cherry", "size": [8, 5, 0.75], "at": [13, 0, 0], "length": "x", "width": "z" },
            { "name": "Drawer side", "material": "cherry", "size": [8, 5, 0.5], "at": [13, 0.75, 0], "length": "y", "width": "z" }
        ],
        "joints": [
            { "kind": "mortise_and_tenon", "part": "Apron", "into": "Leg" },
            { "kind": "dovetails", "part": "Drawer side", "into": "Drawer front" }
        ]
    }"#,
    );
    {
        let doc = harness.state_mut().document_mut();
        assert_eq!(doc.scene().joints.len(), 2);
        pull_apart(doc, "Apron", DVec3::X * 2.5);
        pull_apart(doc, "Drawer side", DVec3::Y * 2.5);
        doc.selection_mut().clear();
    }
    frame(&mut harness, -72.0, 24.0);
    snapshot(&mut harness, "site_joinery");
}

#[test]
fn a_table_checked_for_wood_movement() {
    let (mut harness, _gpu) = app();
    open(
        &mut harness,
        "table",
        r#"{
        "materials": [{ "id": "cherry", "library": "Cherry" }],
        "templates": [{ "kind": "table", "name": "Hall table", "at": [0, 0, 0], "wood": "cherry", "sizes": { "width": 48, "depth": 16, "height": 30 } }]
    }"#,
    );
    harness.state_mut().document_mut().selection_mut().clear();
    harness.state_mut().show_wood_movement(true);
    // From above, to see the arrows on the top and where the aprons hold it.
    frame(&mut harness, -65.0, 38.0);
    assert!(harness.query_by_label_contains("moves").is_some(), "the movement is drawn on the parts");
    snapshot(&mut harness, "site_wood_movement");
}

#[test]
fn a_walnut_cabinet_in_the_dark() {
    use woodsketch_doc::template::{Cabinet, Fill, Template};
    let (mut harness, _gpu) = app();
    harness.ctx.set_theme(egui::Theme::Dark);
    {
        let doc = harness.state_mut().document_mut();
        let mut c = Cabinet { width: 24.0, height: 30.0, depth: 16.0, shelves: 1, ..Default::default() };
        c.fills[0] = Fill::Drawer;
        c.fills[1] = Fill::Drawer;
        let maple = doc.add_preset(woodsketch_doc::library::LIBRARY.iter().find(|p| p.name == "Hard Maple").unwrap());
        doc.add_template(&Template::Cabinet(c), Some(maple)).unwrap();
        doc.selection_mut().clear();
    }
    frame(&mut harness, -50.0, 20.0);
    snapshot(&mut harness, "site_dark");
}
