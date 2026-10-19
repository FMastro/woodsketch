//! The 3D viewport widget: renders through [`ViewportRenderer`] into a texture shown as an
//! image, feeds pointer input to the tools, and draws their overlays.

use eframe::egui;
use glam::{DVec3, Vec2};
use woodsketch_doc::Document;
use woodsketch_render::{Scene, ViewportRenderer};
use woodsketch_tools::infer::Along;
use woodsketch_tools::{Axis, Modifiers, Overlay, PointerEvent, SnapKind, Tools};
use woodsketch_view::OrbitCamera;

/// Scroll distance, in points, that counts as one zoom step (about one mouse-wheel notch).
const POINTS_PER_ZOOM_STEP: f32 = 50.0;

pub struct Viewport {
    pub camera: OrbitCamera,
    renderer: ViewportRenderer,
    texture: Option<egui::TextureId>,
    /// The document version and grid setting the renderer's scene was built from.
    scene_version: Option<(u64, Vec<woodsketch_doc::InstanceId>, bool)>,
    /// The document version the selection highlight was built from.
    selection_version: Option<u64>,
    /// The drawing's soft edges, kept for rebuilding the selection highlight alone.
    /// The ground grid (View > Grid). SketchUp has none, so it starts off.
    pub show_grid: bool,
    /// Each solid-wood part's seasonal movement drawn on it (View > Wood Movement).
    pub show_movement: bool,
    /// The arrows, for the drawing version they were worked out for.
    movement: Option<(u64, Vec<woodsketch_doc::movement::Arrow>)>,
    zoom_extents_pending: bool,
    /// Walking (Camera > Walk): the cursor is hidden and kept in the middle while it is.
    walking: bool,
    /// Walk's speed, inches a second (the wheel changes it).
    walk_speed: f32,
    last_hover: Option<Vec2>,
    /// Where the viewport was last laid out, in window points.
    rect: egui::Rect,
}

impl Viewport {
    pub fn new(render_state: &egui_wgpu::RenderState) -> Self {
        Self {
            camera: OrbitCamera::default(),
            renderer: ViewportRenderer::new(&render_state.device, &Scene::reference(false)),
            show_grid: false,
            show_movement: false,
            movement: None,
            texture: None,
            scene_version: None,
            selection_version: None,
            zoom_extents_pending: false,
            walking: false,
            walk_speed: WALK_SPEED,
            last_hover: None,
            rect: egui::Rect::NOTHING,
        }
    }

    pub fn request_zoom_extents(&mut self) {
        self.zoom_extents_pending = true;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, frame: &eframe::Frame, doc: &mut Document, tools: &mut Tools) {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
        let size = Vec2::new(rect.width(), rect.height());
        self.rect = rect;

        self.walk(ui.ctx(), rect, doc, tools);
        self.navigate(ui, &response, rect, size, doc);
        self.feed_tools(ui, &response, rect, size, doc, tools);
        // Right-click (not a right-drag, which pans): select what's there, then the menu.
        if response.secondary_clicked()
            && let Some(pos) = response.interact_pointer_pos()
        {
            tools.select_under(doc, &self.camera, size, to_vec2(pos - rect.min));
        }
        response.context_menu(|ui| {
            if context_menu(ui, doc, tools) {
                self.request_zoom_extents();
            }
        });
        if self.zoom_extents_pending && size.x > 1.0 && size.y > 1.0 {
            match doc.bounds() {
                Some((min, max)) => self.camera.zoom_extents(min.as_vec3(), max.as_vec3(), size),
                None => self.camera = OrbitCamera::default(),
            }
            self.zoom_extents_pending = false;
        }

        let Some(render_state) = frame.wgpu_render_state() else { return };
        // The drawing is rebuilt when it, the open group or the grid changes; the selection
        // highlight alone when only the selection does (cheap, so clicking stays instant).
        let geometry = (doc.model_version(), doc.path().to_vec(), self.show_grid);
        if self.scene_version.as_ref() != Some(&geometry) {
            let mut scene = Scene::reference(self.show_grid);
            scene.add_view(doc);
            self.renderer.set_scene(&render_state.device, &scene);
            self.scene_version = Some(geometry);
            self.selection_version = Some(doc.version());
        } else if self.selection_version != Some(doc.version()) {
            let mut scene = Scene::default();
            scene.add_selection(doc.view(), doc.selection(), &doc.context_transform(), doc.scene());
            self.renderer.set_selection(&render_state.device, &scene);
            self.selection_version = Some(doc.version());
        }
        let pixels_per_point = ui.ctx().pixels_per_point();
        let physical = [
            (rect.width() * pixels_per_point).round() as u32,
            (rect.height() * pixels_per_point).round() as u32,
        ];
        // The sky follows the theme (View > Theme).
        self.renderer.sky = if ui.visuals().dark_mode { woodsketch_render::Sky::dark() } else { woodsketch_render::Sky::light() };
        let resized = self.renderer.render(
            &render_state.device,
            &render_state.queue,
            physical,
            self.camera.view_proj(size),
            self.camera.eye(),
            pixels_per_point,
        );

        let view = self.renderer.output_view().expect("rendered at least once");
        let filter = egui_wgpu::wgpu::FilterMode::Linear;
        let mut egui_renderer = render_state.renderer.write();
        let id = match self.texture {
            Some(id) if resized => {
                egui_renderer.update_egui_texture_from_wgpu_texture(&render_state.device, view, filter, id);
                id
            }
            Some(id) => id,
            None => egui_renderer.register_native_texture(&render_state.device, view, filter),
        };
        drop(egui_renderer);
        self.texture = Some(id);

        let painter = ui.painter_at(rect);
        painter.image(
            id,
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        let overlay = tools.overlay(doc);
        draw_notes(&painter, rect, &self.camera, size, doc);
        draw_overlay(&painter, rect, &self.camera, size, &overlay);
        if self.show_movement {
            // Worked out again only when the drawing changes (it compares every pair of parts).
            let version = doc.model_version();
            if self.movement.as_ref().is_none_or(|(v, _)| *v != version) {
                let report = woodsketch_doc::movement::check(doc.scene(), doc.scene().moisture_swing());
                self.movement = Some((version, report.arrows));
            }
            if let Some((_, arrows)) = &self.movement {
                draw_movement(&painter, rect, &self.camera, size, arrows);
            }
        }
        editing_badge(&painter, rect, doc);
        // The tool's mode beside the pointer (the Tape Measure's "+ Guide").
        if let (Some(note), Some(pointer)) = (overlay.cursor_note, ui.input(|i| i.pointer.hover_pos()))
            && rect.contains(pointer)
        {
            painter.text(
                pointer + egui::vec2(14.0, -14.0),
                egui::Align2::LEFT_BOTTOM,
                note,
                egui::FontId::proportional(12.0),
                gray(&painter, 40),
            );
        }
    }

    /// Middle-drag orbits, Shift+middle-drag and right-drag pan, the wheel zooms at the cursor.
    fn navigate(&mut self, ui: &egui::Ui, response: &egui::Response, rect: egui::Rect, size: Vec2, doc: &Document) {
        let (shift, scroll) = ui.input(|i| (i.modifiers.shift, i.smooth_scroll_delta.y));
        let drag = to_vec2(response.drag_delta());
        if response.dragged_by(egui::PointerButton::Middle) {
            if shift {
                self.camera.pan(drag, size);
            } else {
                self.camera.orbit(drag);
            }
        } else if response.dragged_by(egui::PointerButton::Secondary) {
            self.camera.pan(drag, size);
        }
        // Walking, the wheel sets the speed instead.
        if scroll != 0.0
            && !self.walking
            && let Some(pointer) = response.hover_pos()
        {
            let cursor = to_vec2(pointer - rect.min);
            let depth = self.depth_under(cursor, size, doc);
            self.camera.zoom_toward(scroll / POINTS_PER_ZOOM_STEP, cursor, size, depth);
        }
    }

    /// How far along the view direction the surface under `cursor` is: the nearest face, else
    /// the ground. `None` when the cursor points at the sky.
    fn depth_under(&self, cursor: Vec2, size: Vec2, doc: &Document) -> Option<f32> {
        let to_world = doc.context_transform();
        let screen = woodsketch_tools::pick::Screen::in_context(&self.camera, size, to_world);
        let ray = screen.ray(cursor);
        let hit = woodsketch_tools::pick::hit_face(doc.view().model(), &ray)
            .map(|(_, p)| to_world.transform_point3(p))
            .or_else(|| {
                let (origin, dir) = self.camera.ray(cursor, size);
                let t = -origin.z / dir.z;
                (dir.z.abs() > 1e-6 && t > 0.0).then(|| (origin + dir * t).as_dvec3())
            })?;
        Some((hit.as_vec3() - self.camera.eye()).dot(self.camera.forward()))
    }

    /// Left-button input goes to the active tool.
    fn feed_tools(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        rect: egui::Rect,
        size: Vec2,
        doc: &mut Document,
        tools: &mut Tools,
    ) {
        let local = |p: egui::Pos2| to_vec2(p - rect.min);
        let (mods, press_origin, latest) = ui.input(|i| {
            let mods = Modifiers { shift: i.modifiers.shift, ctrl: i.modifiers.command, alt: i.modifiers.alt };
            (mods, i.pointer.press_origin(), i.pointer.latest_pos())
        });
        tools.set_shift(mods.shift);
        let mut send = |event| tools.pointer(event, doc, &mut self.camera, size);

        if let Some(pos) = response.hover_pos().map(local)
            && self.last_hover != Some(pos)
        {
            self.last_hover = Some(pos);
            send(PointerEvent::Move(pos));
        }

        let primary = egui::PointerButton::Primary;
        if response.drag_started_by(primary)
            && let Some(origin) = press_origin.map(local)
        {
            send(PointerEvent::DragStart { origin });
        }
        if response.dragged_by(primary)
            && let Some(pos) = response.interact_pointer_pos().map(local)
        {
            send(PointerEvent::Drag { pos, delta: to_vec2(response.drag_delta()) });
        }
        if response.drag_stopped_by(primary)
            && let Some(pos) = latest.map(local)
        {
            send(PointerEvent::DragEnd { pos, mods });
        }

        let count = if response.triple_clicked() {
            3
        } else if response.double_clicked() {
            2
        } else if response.clicked() {
            1
        } else {
            0
        };
        if count > 0
            && let Some(pos) = response.interact_pointer_pos().map(local)
        {
            send(PointerEvent::Click { pos, count, mods });
        }
    }
}

fn to_vec2(v: egui::Vec2) -> Vec2 {
    Vec2::new(v.x, v.y)
}

/// Protractor radius on screen, in points.
const PROTRACTOR_RADIUS: f32 = 45.0;

/// SketchUp's protractor: a circle with 15° ticks in the rotation plane, colored by the axis
/// it's aligned with, plus the starting (dashed) and current (solid) rays.
fn draw_protractor(
    painter: &egui::Painter,
    camera: &OrbitCamera,
    size: Vec2,
    screen: &dyn Fn(DVec3) -> Option<egui::Pos2>,
    protractor: &woodsketch_tools::Protractor,
) {
    let (center, n) = (protractor.center, protractor.normal.normalize());
    // Keep the protractor a constant size on screen.
    let depth = (center.as_vec3() - camera.eye()).dot(camera.forward()).max(0.01);
    let world_per_point = 2.0 * depth * (camera.fov_y * 0.5).tan() / size.y.max(1.0);
    let radius = f64::from(PROTRACTOR_RADIUS * world_per_point);
    let u = n.cross(if n.z.abs() < 0.9 { DVec3::Z } else { DVec3::X }).normalize();
    let v = n.cross(u);
    let at = |degrees: f64, r: f64| {
        let a = degrees.to_radians();
        center + (u * a.cos() + v * a.sin()) * r
    };
    let color = if n.z.abs() > 0.999 {
        egui::Color32::from_rgb(0, 0, 230)
    } else if n.x.abs() > 0.999 {
        egui::Color32::from_rgb(220, 0, 0)
    } else if n.y.abs() > 0.999 {
        egui::Color32::from_rgb(0, 160, 0)
    } else {
        gray(painter, 60)
    };
    let stroke = egui::Stroke::new(1.5, color);
    let circle: Option<Vec<egui::Pos2>> = (0..=72).map(|i| screen(at(f64::from(i) * 5.0, radius))).collect();
    if let Some(circle) = circle {
        painter.add(egui::Shape::line(circle, stroke));
    }
    for i in 0..24 {
        let degrees = f64::from(i) * 15.0;
        let inner = if i % 6 == 0 { 0.7 } else { 0.85 };
        if let (Some(a), Some(b)) = (screen(at(degrees, radius * inner)), screen(at(degrees, radius))) {
            painter.line_segment([a, b], stroke);
        }
    }
    let Some(c) = screen(center) else { return };
    painter.circle_filled(c, 2.5, color);
    if let Some(from) = protractor.from.and_then(screen) {
        let dashed = egui::Stroke::new(1.0, gray(painter, 70));
        painter.extend(egui::Shape::dashed_line(&[c, from], dashed, 5.0, 4.0));
    }
    if let Some(to) = protractor.to.and_then(screen) {
        painter.line_segment([c, to], egui::Stroke::new(1.5, gray(painter, 0)));
    }
}

/// Guides: dashed endless lines and small crosses, blue when selected.
fn draw_guides(
    painter: &egui::Painter,
    camera: &OrbitCamera,
    screen: &impl Fn(DVec3) -> Option<egui::Pos2>,
    overlay: &Overlay,
) {
    // How far guide lines reach either side of the camera, in inches; far beyond any furniture.
    const REACH: f64 = 20_000.0;
    let eye = camera.eye().as_dvec3();
    let forward = camera.forward().as_dvec3();
    let near = f64::from(camera.distance) * 0.02;
    for (guide, selected) in &overlay.guides {
        let selected = *selected;
        let color = if selected { egui::Color32::from_rgb(0, 90, 255) } else { gray(painter, 60) };
        let stroke = egui::Stroke::new(if selected { 1.5 } else { 1.0 }, color);
        match *guide {
            woodsketch_doc::Guide::Line { point, dir } => {
                // The stretch of the line nearest the eye, cut where it passes behind the camera.
                let middle = point + dir * (eye - point).dot(dir);
                let (mut a, mut b) = (middle - dir * REACH, middle + dir * REACH);
                let (da, db) = ((a - eye).dot(forward) - near, (b - eye).dot(forward) - near);
                if da < 0.0 && db < 0.0 {
                    continue;
                }
                if da < 0.0 {
                    a = a + (b - a) * (da / (da - db));
                } else if db < 0.0 {
                    b = a + (b - a) * (da / (da - db));
                }
                if let (Some(a), Some(b)) = (screen(a), screen(b)) {
                    painter.extend(egui::Shape::dashed_line(&[a, b], stroke, 6.0, 4.0));
                }
            }
            woodsketch_doc::Guide::Segment { a, b } => {
                if let (Some(a), Some(b)) = (screen(a), screen(b)) {
                    painter.extend(egui::Shape::dashed_line(&[a, b], stroke, 6.0, 4.0));
                }
            }
            woodsketch_doc::Guide::Point(p) => {
                if let Some(p) = screen(p) {
                    let r = 4.0;
                    painter.line_segment([p - egui::vec2(r, r), p + egui::vec2(r, r)], stroke);
                    painter.line_segment([p - egui::vec2(r, -r), p + egui::vec2(r, -r)], stroke);
                }
            }
            // A dimension or note being drawn, or highlighted by a tool.
            woodsketch_doc::Guide::Dimension { .. } | woodsketch_doc::Guide::Text { .. } => draw_note(painter, &screen, guide, Some(color)),
        }
    }
}

/// The drawing's dimensions and text notes, over the view (the renderer can't draw words).
/// Selected ones in the group being edited are blue.
fn draw_notes(painter: &egui::Painter, rect: egui::Rect, camera: &OrbitCamera, size: Vec2, doc: &Document) {
    let screen = |p: DVec3| camera.project(p.as_vec3(), size).map(|s| rect.min + egui::vec2(s.x, s.y));
    let view = doc.view();
    // The view is in the edited container's coordinates; the camera sees the world.
    let to_world = doc.context_transform();
    for (id, guide) in view.model().guides().filter(|(_, g)| g.is_note()) {
        let entity = woodsketch_doc::Entity::Guide(id);
        let selected = view.owner(entity) == woodsketch_doc::Owner::Active && doc.selection().contains(entity);
        let color = selected.then(|| egui::Color32::from_rgb(0, 90, 255));
        draw_note(painter, &screen, &guide.transformed(&to_world), color);
    }
}

/// A dimension (extension lines, its line with slash ticks, the length measured now in the
/// middle) or a text note (its leader and words), in `color` or the theme's ink.
fn draw_note(painter: &egui::Painter, screen: &dyn Fn(DVec3) -> Option<egui::Pos2>, guide: &woodsketch_doc::Guide, color: Option<egui::Color32>) {
    let ink = color.unwrap_or_else(|| gray(painter, 40));
    let stroke = egui::Stroke::new(1.0, ink);
    let label = |at: egui::Pos2, text: String| {
        let galley = painter.layout_no_wrap(text, egui::FontId::proportional(12.0), ink);
        let rect = egui::Rect::from_center_size(at, galley.size()).expand(2.0);
        painter.rect_filled(rect, 2.0, paper(painter, 210));
        painter.galley(rect.min + egui::vec2(2.0, 2.0), galley, ink);
    };
    match guide {
        woodsketch_doc::Guide::Dimension { a, b, offset } => {
            let (Some(pa), Some(pb), Some(qa), Some(qb)) = (screen(*a), screen(*b), screen(*a + *offset), screen(*b + *offset)) else { return };
            // Extension lines from the points to a little past the dimension line.
            let past = |from: egui::Pos2, to: egui::Pos2| if from.distance(to) > 1.0 { to + (to - from).normalized() * 5.0 } else { to };
            painter.line_segment([pa, past(pa, qa)], stroke);
            painter.line_segment([pb, past(pb, qb)], stroke);
            painter.line_segment([qa, qb], stroke);
            // A woodworker's slash at each end.
            let along = (qb - qa).normalized();
            let slash = (along + egui::vec2(-along.y, along.x)) * 4.0;
            for q in [qa, qb] {
                painter.line_segment([q - slash, q + slash], egui::Stroke::new(1.5, ink));
            }
            label(qa + (qb - qa) * 0.5, woodsketch_doc::units::format_length(a.distance(*b)));
        }
        woodsketch_doc::Guide::Text { at, offset, text } => {
            let (Some(p), Some(q)) = (screen(*at), screen(*at + *offset)) else { return };
            if p.distance(q) > 2.0 {
                painter.line_segment([p, q], stroke);
                painter.circle_filled(p, 2.0, ink);
            }
            label(q, text.clone());
        }
        _ => {}
    }
}

/// While inside a group or component, a label in the viewport's corner says which one, so it's
/// always clear whether new drawing goes into it.
fn editing_badge(painter: &egui::Painter, rect: egui::Rect, doc: &Document) {
    let names = doc.path_names();
    if names.is_empty() {
        return;
    }
    let text = format!("Editing  {}    Esc or click outside to close", names.join(" › "));
    let galley = painter.layout_no_wrap(text, egui::FontId::proportional(13.0), gray(painter, 30));
    let badge = egui::Rect::from_min_size(rect.min + egui::vec2(10.0, 10.0), galley.size()).expand2(egui::vec2(8.0, 5.0));
    painter.rect(
        badge.translate(egui::vec2(8.0, 5.0)),
        4.0,
        paper(painter, 230),
        egui::Stroke::new(1.0, gray(painter, 150)),
        egui::StrokeKind::Outside,
    );
    painter.galley(rect.min + egui::vec2(18.0, 15.0), galley, gray(painter, 30));
}

/// A SketchUp-style inference tooltip below-right of `pos`.
fn tooltip(painter: &egui::Painter, pos: egui::Pos2, label: &str) {
    let galley = painter.layout_no_wrap(label.into(), egui::FontId::proportional(12.0), gray(painter, 0));
    let text_rect = egui::Rect::from_min_size(pos + egui::vec2(12.0, 10.0), galley.size()).expand(3.0);
    painter.rect(
        text_rect,
        2.0,
        if painter.ctx().theme() == egui::Theme::Dark { egui::Color32::from_rgb(70, 66, 40) } else { egui::Color32::from_rgb(255, 255, 220) },
        egui::Stroke::new(1.0, egui::Color32::GRAY),
        egui::StrokeKind::Outside,
    );
    painter.galley(text_rect.min + egui::vec2(3.0, 3.0), galley, gray(painter, 0));
}

/// Draw tool previews on top of the rendered model: preview edges, the inference point and its
/// tooltip, and the selection window.
/// How fast Walk starts out, inches a second (a slow walk; Shift for three times that).
const WALK_SPEED: f32 = 48.0;
/// How close Walk comes to a wall, inches.
const WALL_CLEARANCE: f64 = 9.0;

impl Viewport {
    /// Walk (Camera > Walk), as in a game: the mouse looks around, W A S D or the arrow keys
    /// walk, Shift goes faster, E and Q go up and down, the wheel sets the speed; walls stop
    /// it. The mouse's own motion turns the view, with the cursor hidden and kept in the middle
    /// so it never runs into the screen's edge. Leaving Walk shows it. A typed eye height
    /// (Measurements box) puts the eye that high.
    fn walk(&mut self, ctx: &egui::Context, rect: egui::Rect, doc: &Document, tools: &mut Tools) {
        if tools.active() != woodsketch_tools::ToolId::Walk {
            if self.walking {
                self.walking = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(true));
            }
            return;
        }
        if !self.walking {
            self.walking = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(false));
        }
        if let Some(height) = tools.take_walk_eye() {
            self.camera.raise(height as f32 - self.camera.eye().z);
        }
        let typing = tools.is_typing();
        let (motion, delta, dt, keys, scroll) = ctx.input(|i| {
            // Letters typed into the Measurements box don't walk.
            let down = |keys: &[egui::Key]| !typing && keys.iter().any(|k| i.key_down(*k));
            let keys = (
                down(&[egui::Key::W, egui::Key::ArrowUp]),
                down(&[egui::Key::S, egui::Key::ArrowDown]),
                down(&[egui::Key::A, egui::Key::ArrowLeft]),
                down(&[egui::Key::D, egui::Key::ArrowRight]),
                i.modifiers.shift,
                f32::from(u8::from(down(&[egui::Key::E, egui::Key::PageUp]))) - f32::from(u8::from(down(&[egui::Key::Q, egui::Key::PageDown]))),
            );
            (i.pointer.motion(), i.pointer.delta(), i.stable_dt.min(0.1), keys, i.smooth_scroll_delta.y)
        });
        if scroll != 0.0 {
            self.walk_speed = (self.walk_speed * 1.25f32.powf(scroll / POINTS_PER_ZOOM_STEP)).clamp(6.0, 1200.0);
            tools.set_message(format!("Walking speed: {} a second", woodsketch_doc::units::format_length(f64::from(self.walk_speed))));
        }
        // Raw motion when the system gives it (and then the cursor goes back to the middle);
        // else how far the pointer moved.
        match motion {
            Some(m) if m != egui::Vec2::ZERO => {
                self.camera.look(Vec2::new(m.x, m.y));
                ctx.send_viewport_cmd(egui::ViewportCommand::CursorPosition(rect.center()));
            }
            Some(_) => {}
            None => self.camera.look(Vec2::new(delta.x, delta.y)),
        }
        let (ahead, back, left, right, fast, up) = keys;
        let forward = f32::from(u8::from(ahead)) - f32::from(u8::from(back));
        let across = f32::from(u8::from(right)) - f32::from(u8::from(left));
        let step = self.walk_speed * if fast { 3.0 } else { 1.0 } * dt;
        if forward != 0.0 || across != 0.0 {
            let before = self.camera.target;
            self.camera.walk(forward * step, across * step);
            let moved = (self.camera.target - before).as_dvec3();
            if blocked(doc, self.camera.eye().as_dvec3() - moved, moved) {
                self.camera.target = before;
            }
        }
        if up != 0.0 {
            self.camera.raise(up * step);
        }
        tools.set_walk_eye(f64::from(self.camera.eye().z));
        if forward != 0.0 || across != 0.0 || up != 0.0 {
            // Keep walking while a key is held.
            ctx.request_repaint();
        }
    }
}

/// Whether walking `moved` from `eye` would take it into a wall: a face ahead, within the step
/// and a little room, at eye height or lower down (a tabletop at the waist stops it too).
fn blocked(doc: &Document, eye: DVec3, moved: DVec3) -> bool {
    let Some(dir) = moved.try_normalize() else { return false };
    let to_local = doc.context_transform().inverse();
    let reach = moved.length() + WALL_CLEARANCE;
    let model = doc.view().model();
    [0.0, 0.5].iter().any(|&down| {
        let from = eye - DVec3::Z * (eye.z.max(0.0) * down);
        let ray = woodsketch_tools::pick::Ray { origin: to_local.transform_point3(from), dir: to_local.transform_vector3(dir).normalize() };
        woodsketch_tools::pick::hit_face(model, &ray).is_some_and(|(_, p)| to_local.inverse().transform_point3(p).distance(from) < reach)
    })
}

/// View > Wood Movement: a double arrow across each solid-wood part's width with how much it
/// moves, red where a part it meets holds it, green where it's free.
fn draw_movement(painter: &egui::Painter, rect: egui::Rect, camera: &OrbitCamera, size: Vec2, arrows: &[woodsketch_doc::movement::Arrow]) {
    let screen = |p: DVec3| camera.project(p.as_vec3(), size).map(|s| rect.min + egui::vec2(s.x, s.y));
    for arrow in arrows {
        let (Some(a), Some(b)) = (screen(arrow.from), screen(arrow.to)) else { continue };
        if a.distance(b) < 16.0 {
            continue;
        }
        let color = if arrow.held { egui::Color32::from_rgb(200, 60, 40) } else { egui::Color32::from_rgb(40, 140, 70) };
        let stroke = egui::Stroke::new(2.5, color);
        painter.line_segment([a, b], stroke);
        let dir = (b - a).normalized();
        let side = egui::vec2(-dir.y, dir.x);
        for (tip, back) in [(a, dir), (b, -dir)] {
            painter.line_segment([tip, tip + (back + side * 0.6) * 9.0], stroke);
            painter.line_segment([tip, tip + (back - side * 0.6) * 9.0], stroke);
        }
        let galley = painter.layout_no_wrap(arrow.label.clone(), egui::FontId::proportional(12.0), color);
        let label = egui::Rect::from_center_size(a + (b - a) * 0.5 + side * 14.0, galley.size()).expand(3.0);
        painter.rect_filled(label, 3.0, paper(painter, 225));
        painter.galley(label.min + egui::vec2(3.0, 3.0), galley, color);
    }
}

fn draw_overlay(painter: &egui::Painter, rect: egui::Rect, camera: &OrbitCamera, size: Vec2, overlay: &Overlay) {
    let screen = |p: DVec3| camera.project(p.as_vec3(), size).map(|s| rect.min + egui::vec2(s.x, s.y));

    draw_guides(painter, camera, &screen, overlay);

    // The selected part's grain: a wood-brown arrow both ways along it.
    if let Some([a, b]) = overlay.grain
        && let (Some(a), Some(b)) = (screen(a), screen(b))
        && a.distance(b) > 12.0
    {
        let color = egui::Color32::from_rgb(150, 90, 30);
        let stroke = egui::Stroke::new(2.5, color);
        painter.line_segment([a, b], stroke);
        let dir = (b - a).normalized();
        let side = egui::vec2(-dir.y, dir.x);
        for (tip, back) in [(a, dir), (b, -dir)] {
            painter.line_segment([tip, tip + (back + side * 0.6) * 9.0], stroke);
            painter.line_segment([tip, tip + (back - side * 0.6) * 9.0], stroke);
        }
        let mid = a + (b - a) * 0.5;
        let galley = painter.layout_no_wrap("Grain".into(), egui::FontId::proportional(12.0), color);
        let label = egui::Rect::from_center_size(mid + side * 14.0, galley.size()).expand(3.0);
        painter.rect_filled(label, 3.0, paper(painter, 220));
        painter.galley(label.min + egui::vec2(3.0, 3.0), galley, color);
    }

    // The selected part's reference face (an outline on it) and edge (a line along it), with
    // the woodworker's marks' names.
    let mark = if painter.ctx().theme() == egui::Theme::Dark { egui::Color32::from_rgb(120, 170, 255) } else { egui::Color32::from_rgb(40, 90, 200) };
    let tag = |painter: &egui::Painter, at: egui::Pos2, text: &str| {
        let galley = painter.layout_no_wrap(text.into(), egui::FontId::proportional(12.0), mark);
        let label = egui::Rect::from_center_size(at, galley.size()).expand(3.0);
        painter.rect_filled(label, 3.0, paper(painter, 220));
        painter.galley(label.min + egui::vec2(3.0, 3.0), galley, mark);
    };
    if let Some(quad) = overlay.reference_face
        && let Some(points) = quad.iter().map(|p| screen(*p)).collect::<Option<Vec<_>>>()
    {
        painter.add(egui::Shape::closed_line(points.clone(), egui::Stroke::new(2.0, mark)));
        // Toward one end, clear of the grain arrow's label in the middle.
        let toward_end = points[0] + (points[3] - points[0]) * 0.5;
        let middle = points.iter().fold(egui::Vec2::ZERO, |sum, p| sum + p.to_vec2()) / points.len() as f32;
        tag(painter, middle.to_pos2() + (toward_end - middle.to_pos2()) * 0.6, "Reference face");
    }
    if let Some([a, b]) = overlay.reference_edge
        && let (Some(a), Some(b)) = (screen(a), screen(b))
    {
        painter.line_segment([a, b], egui::Stroke::new(3.0, mark));
        tag(painter, a + (b - a) * 0.8 + egui::vec2(0.0, 14.0), "Reference edge");
    }

    // The box around the group being edited, dashed as in SketchUp.
    let dashed = egui::Stroke::new(1.0, gray(painter, 70));
    for [a, b] in &overlay.dashed {
        if let (Some(a), Some(b)) = (screen(*a), screen(*b)) {
            painter.extend(egui::Shape::dashed_line(&[a, b], dashed, 6.0, 4.0));
        }
    }

    let highlight = egui::Stroke::new(3.0, egui::Color32::from_rgb(0, 90, 255));
    for [a, b] in &overlay.highlight {
        if let (Some(a), Some(b)) = (screen(*a), screen(*b)) {
            painter.line_segment([a, b], highlight);
        }
    }

    let preview = egui::Stroke::new(1.5, gray(painter, 0));
    for [a, b] in &overlay.lines {
        if let (Some(a), Some(b)) = (screen(*a), screen(*b)) {
            painter.line_segment([a, b], preview);
        }
    }

    // Move's rotation grips: small red crosses, bigger under the cursor.
    for &(point, hovered) in &overlay.grips {
        if let Some(p) = screen(point) {
            let (r, width) = if hovered { (7.0, 2.5) } else { (4.5, 1.5) };
            let stroke = egui::Stroke::new(width, egui::Color32::from_rgb(220, 0, 0));
            painter.line_segment([p - egui::vec2(r, 0.0), p + egui::vec2(r, 0.0)], stroke);
            painter.line_segment([p - egui::vec2(0.0, r), p + egui::vec2(0.0, r)], stroke);
        }
    }

    if let Some(protractor) = &overlay.protractor {
        draw_protractor(painter, camera, size, &screen, protractor);
    }

    // The rubber band from the base point: in the axis's color when following one, magenta when
    // parallel or perpendicular to an edge (as in SketchUp), otherwise dashed grey.
    let mut axis_label = None;
    if let Some(guide) = &overlay.guide
        && let (Some(from), Some(to)) = (screen(guide.from), screen(guide.to))
    {
        match guide.along {
            Some(along) => {
                let color = match along {
                    Along::Axis(axis) => axis_color(axis),
                    Along::Parallel | Along::Perpendicular => egui::Color32::from_rgb(200, 0, 200),
                };
                painter.line_segment([from, to], egui::Stroke::new(2.0, color));
                axis_label = Some((to, along.label()));
            }
            None => {
                let stroke = egui::Stroke::new(1.0, gray(painter, 80));
                painter.extend(egui::Shape::dashed_line(&[from, to], stroke, 5.0, 4.0));
            }
        }
    }

    // Axis guide lines through the protractor's center.
    for &(from, to, axis) in &overlay.axis_lines {
        if let (Some(a), Some(b)) = (screen(from), screen(to)) {
            painter.extend(egui::Shape::dashed_line(&[a, b], egui::Stroke::new(1.0, axis_color(axis).gamma_multiply(0.6)), 6.0, 4.0));
        }
    }

    // "From Point": dotted in the axis color from the remembered point to where the cursor
    // lines up with it.
    if let Some((anchor, point, axis)) = overlay.alignment
        && let (Some(a), Some(b)) = (screen(anchor), screen(point))
    {
        painter.extend(egui::Shape::dotted_line(&[a, b], axis_color(axis), 4.0, 1.2));
        if axis_label.is_none() {
            axis_label = Some((b, "From Point"));
        }
    }

    if let Some(snap) = &overlay.snap
        && let Some(pos) = screen(snap.point)
    {
        // SketchUp's inference colors: green endpoints, cyan midpoints, red on-edge, blue
        // on-face (and yellow-ish for the origin).
        let color = match snap.kind {
            SnapKind::Endpoint => egui::Color32::from_rgb(0, 170, 0),
            SnapKind::Origin => egui::Color32::from_rgb(230, 170, 0),
            SnapKind::Midpoint => egui::Color32::from_rgb(0, 190, 210),
            SnapKind::OnEdge => egui::Color32::from_rgb(220, 0, 0),
            SnapKind::GuidePoint | SnapKind::Intersection => gray(painter, 20),
            SnapKind::OnGuide => gray(painter, 110),
            SnapKind::OnFace => egui::Color32::from_rgb(0, 90, 255),
            SnapKind::OnPlane | SnapKind::OnGround => gray(painter, 90),
        };
        let radius = if matches!(snap.kind, SnapKind::OnPlane | SnapKind::OnGround) { 2.5 } else { 5.0 };
        painter.circle(pos, radius, color, egui::Stroke::new(1.0, egui::Color32::WHITE));
        if let Some(label) = snap.kind.label() {
            tooltip(painter, pos, label);
            axis_label = None;
        }
    }
    if let Some((pos, label)) = axis_label {
        tooltip(painter, pos, label);
    }

    if let Some(window) = &overlay.window {
        let a = rect.min + egui::vec2(window.start.x, window.start.y);
        let b = rect.min + egui::vec2(window.end.x, window.end.y);
        let r = egui::Rect::from_two_pos(a, b);
        let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 40));
        painter.rect_filled(r, 0.0, egui::Color32::from_rgba_unmultiplied(80, 120, 255, 24));
        if window.crossing {
            let corners = [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom(), r.left_top()];
            painter.extend(egui::Shape::dashed_line(&corners, stroke, 6.0, 4.0));
        } else {
            painter.rect_stroke(r, 0.0, stroke, egui::StrokeKind::Inside);
        }
    }
}

impl Viewport {
    /// Where a model point appears in the window, if in front of the camera.
    pub fn to_window(&self, point: DVec3) -> Option<egui::Pos2> {
        let size = Vec2::new(self.rect.width(), self.rect.height());
        self.camera.project(point.as_vec3(), size).map(|s| self.rect.min + egui::vec2(s.x, s.y))
    }
}

impl Viewport {
    /// A different drawing was opened (or a new one started): rebuild everything and frame it.
    pub fn document_replaced(&mut self, doc: &Document) {
        self.scene_version = None;
        self.last_hover = None;
        if doc.bounds().is_some() {
            self.zoom_extents_pending = true;
        } else {
            self.camera = OrbitCamera::default();
        }
    }
}

/// SketchUp's axis colors.
fn axis_color(axis: Axis) -> egui::Color32 {
    match axis {
        Axis::X => egui::Color32::from_rgb(220, 0, 0),
        Axis::Y => egui::Color32::from_rgb(0, 160, 0),
        Axis::Z => egui::Color32::from_rgb(0, 0, 230),
    }
}

/// The right-click menu, for the selection. Returns whether Zoom Extents was picked.
fn context_menu(ui: &mut egui::Ui, doc: &mut Document, tools: &mut Tools) -> bool {
    use woodsketch_doc::{DefinitionKind, Entity};
    let selected: Vec<Entity> = doc.selection().iter().collect();
    let instances = selected.iter().filter(|e| matches!(e, Entity::Instance(_))).count();
    let faces = selected.iter().filter(|e| matches!(e, Entity::Face(_))).count();
    let shared = selected.iter().any(|e| match e {
        Entity::Instance(i) => doc.container().instance(*i).is_some_and(|inst| doc.scene().instance_count(inst.definition) > 1),
        _ => false,
    });
    let any = !selected.is_empty();
    let item = |ui: &mut egui::Ui, enabled: bool, text: &str, keys: &str| ui.add_enabled(enabled, egui::Button::new(text).shortcut_text(keys)).clicked();
    let mut zoom = false;
    if item(ui, any, "Cut", "Ctrl+X") {
        tools.cut(doc);
        crate::keys::note_copy(ui.ctx(), tools);
        ui.close();
    }
    if item(ui, any, "Copy", "Ctrl+C") {
        tools.copy(doc);
        crate::keys::note_copy(ui.ctx(), tools);
        ui.close();
    }
    if item(ui, tools.can_paste(), "Paste", "Ctrl+V") {
        tools.paste(doc);
        ui.close();
    }
    if item(ui, any, "Delete", "Delete") {
        tools.cancel();
        doc.erase_selection();
        ui.close();
    }
    ui.separator();
    if item(ui, any, "Make Group", "Ctrl+G") {
        tools.make_group(doc, DefinitionKind::Group);
        ui.close();
    }
    if item(ui, any, "Make Component", "G") {
        tools.make_group(doc, DefinitionKind::Component);
        ui.close();
    }
    if instances > 0 && item(ui, true, "Explode", "") {
        tools.explode(doc);
        ui.close();
    }
    if shared && ui.button("Make Unique").on_hover_text("Its own copy, so changing it leaves the other copies alone").clicked() {
        doc.make_selection_unique();
        ui.close();
    }
    if faces > 0 && ui.button("Reverse Faces").on_hover_text("Turn the selected faces over, front to back").clicked() {
        doc.reverse_selected_faces();
        ui.close();
    }
    ui.separator();
    if item(ui, true, "Zoom Extents", "Shift+Z") {
        zoom = true;
        ui.close();
    }
    zoom
}

/// A gray for lines and text over the drawing: as given in the light theme, flipped (dark becomes
/// light) in the dark one, so it stays readable against the sky.
fn gray(painter: &egui::Painter, value: u8) -> egui::Color32 {
    if painter.ctx().theme() == egui::Theme::Dark { egui::Color32::from_gray(255 - value) } else { egui::Color32::from_gray(value) }
}

/// A label's background: white in the light theme, charcoal in the dark one, `alpha` opaque.
fn paper(painter: &egui::Painter, alpha: u8) -> egui::Color32 {
    if painter.ctx().theme() == egui::Theme::Dark { egui::Color32::from_rgba_unmultiplied(40, 42, 46, alpha) } else { egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha) }
}
