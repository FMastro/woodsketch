//! Reading SketchUp files (`.skp`) from SketchUp 2021 and later, without Trimble's SDK.
//!
//! The format was worked out by examining `.skp` files only (clean room: no SketchUp or SDK code
//! was decompiled or read). A file is a short UTF-16 header naming the version (`{26.2.243}`),
//! then a ZIP archive holding `model.dat` (the drawing), `materials/<name>/material.xml` (colors)
//! and thumbnails. `model.dat` is nested records: a 2-byte tag, a 4-byte length, then the
//! contents (more records, or a value). Numbers are little-endian; lengths are inches.
//!
//! What's read: component definitions (name, geometry, nested instances), the top-level
//! geometry, vertices, edges, faces with holes and their material, instances (definition,
//! placement, material, layer), groups (listed apart from component instances), objects hidden
//! in SketchUp (onto a hidden "Hidden in SketchUp" tag), guide lines and construction lines,
//! hidden and softened edges (not drawn), materials (name and color) and layers (as tags). Not
//! yet: dimensions, text, images, textures, scenes, and files from SketchUp 2020 and older (a
//! different format).

use std::collections::HashMap;
use std::fmt;
use std::io::Read;

use glam::{DAffine3, DVec3};

use crate::{Container, Definition, DefinitionId, DefinitionKind, Instance, Material, MaterialId, MaterialKind, Scene, TagId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkpError {
    /// Not a SketchUp file at all.
    NotSketchUp,
    /// From SketchUp 2020 or older, saved in the older format.
    TooOld,
    /// It looks like a SketchUp file, but something in it can't be read.
    Damaged(String),
}

impl fmt::Display for SkpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotSketchUp => write!(f, "This isn't a SketchUp file."),
            Self::TooOld => write!(
                f,
                "This SketchUp file is from SketchUp 2020 or older, which WoodSketch can't read yet. Open it in SketchUp 2021 or later and save it again."
            ),
            Self::Damaged(why) => write!(f, "This SketchUp file couldn't be read ({why})."),
        }
    }
}

impl std::error::Error for SkpError {}

/// A SketchUp drawing read into a scene.
#[derive(Debug)]
pub struct Imported {
    pub scene: Scene,
    /// The SketchUp version that saved it, like `26.2.243`.
    pub version: String,
    /// Faces whose outline couldn't be followed, left out.
    pub skipped_faces: usize,
}

/// Read a `.skp` file's bytes.
pub fn read(bytes: &[u8]) -> Result<Imported, SkpError> {
    if !bytes.starts_with(&[0xff, 0xfe, 0xff]) {
        return Err(SkpError::NotSketchUp);
    }
    let version = header_version(bytes);
    let Some(zip_at) = find(bytes, b"PK\x03\x04") else {
        return Err(SkpError::TooOld);
    };
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&bytes[zip_at..])).map_err(|e| SkpError::Damaged(e.to_string()))?;
    let mut entry = |name: &str| -> Option<Vec<u8>> {
        let mut file = archive.by_name(name).ok()?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).ok()?;
        Some(data)
    };
    let model = entry("model.dat").ok_or_else(|| SkpError::Damaged("no model.dat".into()))?;
    let top = records(&model).next().ok_or_else(|| SkpError::Damaged("empty model.dat".into()))?;
    let mut reader = Reader { scene: Scene::default(), materials: HashMap::new(), tags: HashMap::new(), definitions: HashMap::new(), skipped_faces: 0, hidden_tag: None, places: Places::default() };
    reader.places = Places::of(top);
    reader.materials(top, &mut entry);
    reader.layers(top);
    reader.definitions(top);
    if let Some(entities) = top.child(0x01f6).and_then(|r| r.child(0x1388)) {
        reader.scene.root = reader.container(entities);
    }
    lumber_by_size(&mut reader.scene);
    Ok(Imported { scene: reader.scene, version, skipped_faces: reader.skipped_faces })
}

/// The version in the header's braces, like `{26.2.243}` (UTF-16).
fn header_version(bytes: &[u8]) -> String {
    let head = &bytes[..bytes.len().min(256)];
    let text: String = head.as_chunks::<2>().0.iter().map(|&c| u16::from_le_bytes(c)).filter_map(|u| char::from_u32(u.into())).collect();
    text.split_once('{').and_then(|(_, rest)| rest.split_once('}')).map(|(v, _)| v.to_owned()).unwrap_or_default()
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// One record: a tag and its contents.
#[derive(Clone, Copy)]
struct Record<'a> {
    tag: u16,
    body: &'a [u8],
}

/// The records laid end to end in `bytes`; stops at anything that doesn't fit.
fn records(bytes: &[u8]) -> impl Iterator<Item = Record<'_>> {
    let mut at = 0;
    std::iter::from_fn(move || {
        let head = bytes.get(at..at + 6)?;
        let tag = u16::from_le_bytes([head[0], head[1]]);
        let len = u32::from_le_bytes([head[2], head[3], head[4], head[5]]) as usize;
        let body = bytes.get(at + 6..at + 6 + len)?;
        at += 6 + len;
        Some(Record { tag, body })
    })
}

impl<'a> Record<'a> {
    fn children(self) -> impl Iterator<Item = Record<'a>> {
        records(self.body)
    }

    fn child(self, tag: u16) -> Option<Record<'a>> {
        self.children().find(|r| r.tag == tag)
    }

    fn all(self, tag: u16) -> impl Iterator<Item = Record<'a>> {
        self.children().filter(move |r| r.tag == tag)
    }

    /// A little-endian whole number of any width up to 8 bytes (ids, counts, flags).
    fn int(self) -> Option<u64> {
        (!self.body.is_empty() && self.body.len() <= 8).then(|| self.body.iter().rev().fold(0u64, |n, &b| (n << 8) | u64::from(b)))
    }

    fn floats(self) -> Vec<f64> {
        self.body.as_chunks::<8>().0.iter().map(|&c| f64::from_le_bytes(c)).collect()
    }

    fn text(self) -> String {
        String::from_utf8_lossy(self.body).into_owned()
    }

    /// The entity id: in an id record (`05dc` > `05de`), directly or inside the drawing
    /// element's common part (`07d0`).
    fn id(self) -> Option<u64> {
        let common = self.child(0x07d0).unwrap_or(self);
        common.child(0x05dc)?.child(0x05de)?.int()
    }

    /// The material (`07d1`) and layer (`07d2`) every drawing element has.
    fn common(self, tag: u16) -> Option<u64> {
        self.child(0x07d0)?.child(tag)?.int()
    }
}

struct Reader {
    scene: Scene,
    materials: HashMap<u64, MaterialId>,
    tags: HashMap<u64, TagId>,
    definitions: HashMap<u64, DefinitionId>,
    /// The tag hidden objects go on ("Hidden in SketchUp", itself hidden), made when one turns up.
    hidden_tag: Option<TagId>,
    /// Every vertex and instance by id, for the points dimensions and text hang on.
    places: Places,
    skipped_faces: usize,
}

impl Reader {
    /// Materials (`01f7`): names here, colors in each one's `material.xml`.
    fn materials(&mut self, top: Record<'_>, entry: &mut impl FnMut(&str) -> Option<Vec<u8>>) {
        let Some(list) = top.child(0x01f7).and_then(|r| r.child(0x30d4)).and_then(|r| r.child(0x30d5)) else { return };
        for record in list.all(0x32c8) {
            let (Some(id), Some(name)) = (record.id(), record.child(0x32cc).map(Record::text)) else { continue };
            // Its folder, from the thumbnail's path (`materials/<folder>/thumbnail.jpg`).
            let folder = record
                .child(0x32ce)
                .and_then(|r| r.child(0x2134))
                .map(Record::text)
                .and_then(|p| p.rsplit_once('/').map(|(dir, _)| dir.to_owned()))
                .unwrap_or_else(|| format!("materials/{name}"));
            let color = entry(&format!("{folder}/material.xml")).and_then(|xml| xml_color(&String::from_utf8_lossy(&xml))).unwrap_or([200, 200, 200]);
            let material = material_for(&name, color);
            let existing = self.scene.materials().find(|(_, m)| m.name == material.name).map(|(i, _)| i);
            let mid = existing.unwrap_or_else(|| self.scene.add_material(material));
            self.materials.insert(id, mid);
        }
    }

    /// Layers (`01f8`) become tags (Layer0 is Untagged), with whether they're hidden (`3c8e`);
    /// then the tag folders (`3a9b`).
    fn layers(&mut self, top: Record<'_>) {
        let Some(layers) = top.child(0x01f8).and_then(|r| r.child(0x3a98)) else { return };
        for record in layers.child(0x3a99).into_iter().flat_map(|r| r.all(0x3c8c)) {
            let (Some(id), Some(name)) = (record.id(), record.child(0x3c8d).map(Record::text)) else { continue };
            if name == "Layer0" || name == "Untagged" {
                continue;
            }
            let tag = self.scene.add_tag(name);
            if let Some(t) = self.scene.tag_mut(tag) {
                t.visible = record.child(0x3c8e).and_then(Record::int) != Some(1);
            }
            self.tags.insert(id, tag);
        }
        // The folders: a top folder (unnamed) holding folders (`3e83`) and tags (`3e84`).
        if let Some(top_folder) = layers.child(0x3a9b).and_then(|r| r.child(0x3e80)) {
            self.folder(top_folder, None, 0);
        }
    }

    /// A tag folder's contents, put in `parent` (`None` for the top).
    fn folder(&mut self, folder: Record<'_>, parent: Option<TagId>, depth: usize) {
        for tag in ids(folder.child(0x3e84).map_or(&[][..], |r| r.body)) {
            if let Some(&t) = self.tags.get(&tag) {
                self.scene.move_tag(t, parent);
            }
        }
        if depth > 32 {
            return;
        }
        for inner in folder.child(0x3e83).into_iter().flat_map(|r| r.all(0x3e80)) {
            let name = inner.child(0x3e81).map(Record::text).filter(|n| !n.is_empty()).unwrap_or_else(|| "Folder".into());
            let new = self.scene.add_tag_folder(name, parent);
            if let Some(t) = self.scene.tag_mut(new) {
                t.visible = inner.child(0x3e82).and_then(Record::int) != Some(1);
            }
            self.folder(inner, Some(new), depth + 1);
        }
    }

    /// Component definitions (`01f9` > `1770` > `1771` > `157c`): every one made first, then
    /// filled, since instances can name any of them.
    fn definitions(&mut self, top: Record<'_>) {
        let Some(list) = top.child(0x01f9).and_then(|r| r.child(0x1770)).and_then(|r| r.child(0x1771)) else { return };
        let found: Vec<(u64, String, Record<'_>)> = list
            .all(0x157c)
            .filter_map(|d| {
                let entities = d.child(0x1388)?;
                Some((entities.id()?, d.child(0x157e).map(Record::text).unwrap_or_default(), entities))
            })
            .collect();
        for (id, name, _) in &found {
            let definition = Definition {
                name: if name.is_empty() { "Component".into() } else { name.clone() },
                kind: DefinitionKind::Component,
                container: Container::default(),
                part: Default::default(),
                template: None,
                hardware: None,
                moves: false,
            };
            let new = self.scene.add_definition(definition);
            self.definitions.insert(*id, new);
        }
        for (id, _, entities) in found {
            let container = self.container(entities);
            if let Some(d) = self.scene.definition_mut(self.definitions[&id]) {
                d.container = container;
            }
        }
    }

    /// One entity list (`1388`): vertices, edges, faces and instances.
    fn container(&mut self, entities: Record<'_>) -> Container {
        let vertices: HashMap<u64, DVec3> = entities
            .child(0x1389)
            .into_iter()
            .flat_map(|r| r.all(0x09c4))
            .filter_map(|v| {
                let p = v.child(0x09c5)?.floats();
                Some((v.id()?, DVec3::new(*p.first()?, *p.get(1)?, *p.get(2)?)))
            })
            .collect();
        let edges: HashMap<u64, (u64, u64)> = entities
            .child(0x138a)
            .into_iter()
            .flat_map(|r| r.all(0x0bb8))
            .filter_map(|e| Some((e.id()?, (e.child(0x0bb9)?.int()?, e.child(0x0bba)?.int()?))))
            .collect();
        // Edges not drawn: hidden (flag bit 0) or softened (bit 3), as a curved surface's facets are.
        let hidden_edges: std::collections::HashSet<u64> = entities
            .child(0x138a)
            .into_iter()
            .flat_map(|r| r.all(0x0bb8))
            .filter(|e| e.common(0x07d3).is_some_and(|f| f & 0b1001 != 0))
            .filter_map(Record::id)
            .collect();

        // SketchUp's topology is taken as it is: each vertex and edge added once, faces made from
        // them, nothing merged or searched for (that's slow on big models, and already done).
        let mut container = Container::default();
        let model = &mut container.model;
        let mut ids: HashMap<u64, crate::VertexId> = HashMap::new();
        let mut vertex = |model: &mut crate::Model, v: u64| -> Option<crate::VertexId> {
            if let Some(&id) = ids.get(&v) {
                return Some(id);
            }
            let id = model.push_vertex(*vertices.get(&v)?);
            ids.insert(v, id);
            Some(id)
        };
        for (id, &(a, b)) in &edges {
            if let (Some(a), Some(b)) = (vertex(model, a), vertex(model, b))
                && a != b
            {
                let edge = model.push_edge(a, b);
                if hidden_edges.contains(id) {
                    model.set_edge_hidden(edge, true);
                }
            }
        }
        for face in entities.child(0x138b).into_iter().flat_map(|r| r.all(0x0dac)) {
            let loops: Vec<Vec<(u64, bool)>> = face
                .child(0x0dae)
                .into_iter()
                .flat_map(|r| r.all(0x1194))
                .map(|l| {
                    l.child(0x1195)
                        .into_iter()
                        .flat_map(|c| c.all(0x0fa0))
                        .filter_map(|c| Some((c.child(0x0fa1)?.int()?, c.child(0x0fa2).and_then(Record::int) == Some(1))))
                        .collect()
                })
                .collect();
            let corners: Option<Vec<Vec<crate::VertexId>>> =
                loops.iter().map(|l| walk(l, &edges)?.into_iter().map(|v| vertex(model, v)).collect()).collect();
            let Some(mut corners) = corners.filter(|c| !c.is_empty()) else {
                self.skipped_faces += 1;
                continue;
            };
            let mut outer = corners.remove(0);
            let points: Vec<DVec3> = outer.iter().map(|&v| model.vertex(v)).collect();
            let winding = newell(&points);
            if winding.length() <= crate::EPS * crate::EPS {
                self.skipped_faces += 1;
                continue;
            }
            let plane = face.child(0x0dad).map(Record::floats).filter(|p| p.len() >= 3).map(|p| DVec3::new(p[0], p[1], p[2]));
            let mut normal = winding.normalize();
            if plane.is_some_and(|n| normal.dot(n) < 0.0) {
                outer.reverse();
                normal = -normal;
            }
            let material = face.common(0x07d1).or_else(|| face.child(0x0daf).and_then(Record::int)).and_then(|m| self.materials.get(&m).copied());
            model.push_face(outer, corners, normal, material);
        }
        // Guide lines (`1391` > `4269` > `426a`: a point, a direction, and how far each way, 1e30
        // for endless ones), and construction lines between two points (`1392` > `426c`, ends `426d`
        // and `426e`).
        for guide in entities.child(0x1391).into_iter().flat_map(|r| r.all(0x4269)) {
            let v = guide.child(0x426a).map(Record::floats).unwrap_or_default();
            if v.len() >= 6
                && let Some(dir) = DVec3::new(v[3], v[4], v[5]).try_normalize()
            {
                container.model.add_guide(crate::Guide::Line { point: DVec3::new(v[0], v[1], v[2]), dir });
            }
        }
        for guide in entities.child(0x1392).into_iter().flat_map(|r| r.all(0x426c)) {
            let point = |tag: u16| guide.child(tag).map(Record::floats).filter(|v| v.len() >= 3).map(|v| DVec3::new(v[0], v[1], v[2]));
            if let (Some(a), Some(b)) = (point(0x426d), point(0x426e))
                && a.distance(b) > crate::EPS
            {
                container.model.add_guide(crate::Guide::Segment { a, b });
            }
        }
        // Dimensions (`1399` > `5bcc`): the two points measured (`5bcd`, `5bce`), the plane it's
        // drawn in (normal `5bcf`) and how far out the dimension line sits (`5bd2`).
        for dimension in entities.child(0x1399).into_iter().flat_map(|r| r.all(0x5bcc)) {
            let ends = (dimension.child(0x5bcd).and_then(|r| self.places.resolve(r)), dimension.child(0x5bce).and_then(|r| self.places.resolve(r)));
            let (Some(a), Some(b)) = ends else { continue };
            let normal = dimension.child(0x5bcf).map(Record::floats).filter(|v| v.len() >= 3).map_or(DVec3::Z, |v| DVec3::new(v[0], v[1], v[2]));
            let out = dimension.child(0x5bd2).map(Record::floats).and_then(|v| v.first().copied()).unwrap_or(0.0);
            let offset = normal.cross(b - a).try_normalize().map_or(DVec3::ZERO, |d| d * out);
            if a.distance(b) > crate::EPS {
                container.model.add_guide(crate::Guide::Dimension { a, b, offset });
            }
        }
        // Text (`1398` > `55f0`): its words (`55f1`), the point it points at (`55f4`) and where the
        // words sit from there (`55f5`). Screen text, pinned to the view and not the model, is left out.
        for text in entities.child(0x1398).into_iter().flat_map(|r| r.all(0x55f0)) {
            let words = text.child(0x55f1).map(Record::text).unwrap_or_default();
            let Some(at) = text.child(0x55f4).and_then(|r| self.places.resolve(r)) else { continue };
            let offset = text.child(0x55f5).map(Record::floats).filter(|v| v.len() >= 3).map_or(DVec3::ZERO, |v| DVec3::new(v[0], v[1], v[2]));
            if !words.trim().is_empty() {
                container.model.add_guide(crate::Guide::Text { at, offset, text: words });
            }
        }
        // Component instances (`138c` > `1964`), then groups (`138d` > `1d4c` > `1964`): the same
        // record, a group's naming its own definition.
        let components = entities.child(0x138c).into_iter().flat_map(|r| r.all(0x1964)).map(|i| (i, false));
        let groups = entities.child(0x138d).into_iter().flat_map(|r| r.all(0x1d4c)).filter_map(|g| g.child(0x1964)).map(|i| (i, true));
        for (instance, group) in components.chain(groups) {
            let Some(&definition) = instance.child(0x1967).and_then(Record::int).and_then(|d| self.definitions.get(&d)) else { continue };
            // A group's definition is its own: it takes the group's name (`1965`), or "Group"
            // as SketchUp shows an unnamed one (its definition is called `Group#24`).
            if group && let Some(d) = self.scene.definition_mut(definition) {
                d.kind = DefinitionKind::Group;
                let name = instance.child(0x1965).map(Record::text).unwrap_or_default();
                d.name = if name.trim().is_empty() { "Group".into() } else { name };
            }
            let transform = instance.child(0x1966).map(Record::floats).and_then(|m| placement(&m)).unwrap_or(DAffine3::IDENTITY);
            // Hidden in SketchUp (Edit > Hide): the low bit of the flags (`07d3`, normally 6).
            let hidden = instance.common(0x07d3).is_some_and(|f| f & 1 == 1);
            container.add_instance(Instance {
                definition,
                transform,
                tag: if hidden { Some(self.hidden_tag()) } else { instance.common(0x07d2).and_then(|l| self.tags.get(&l).copied()) },
                material: instance.common(0x07d1).and_then(|m| self.materials.get(&m).copied()),
            });
        }
        container
    }
}

impl Reader {
    fn hidden_tag(&mut self) -> TagId {
        if let Some(tag) = self.hidden_tag {
            return tag;
        }
        let tag = self.scene.add_tag("Hidden in SketchUp");
        if let Some(t) = self.scene.tag_mut(tag) {
            t.visible = false;
        }
        self.hidden_tag = Some(tag);
        tag
    }
}

/// Where things are, by their ids: every vertex (in its own entity list's coordinates) and every
/// instance (its definition and placement), for resolving what dimensions and text are attached to.
#[derive(Default)]
struct Places {
    vertices: HashMap<u64, DVec3>,
    instances: HashMap<u64, (u64, DAffine3)>,
}

impl Places {
    fn of(top: Record<'_>) -> Self {
        let mut places = Places::default();
        let definitions = top.child(0x01f9).and_then(|r| r.child(0x1770)).and_then(|r| r.child(0x1771));
        let lists = definitions.into_iter().flat_map(|l| l.all(0x157c)).filter_map(|d| d.child(0x1388));
        for entities in lists.chain(top.child(0x01f6).and_then(|r| r.child(0x1388))) {
            for v in entities.child(0x1389).into_iter().flat_map(|r| r.all(0x09c4)) {
                if let (Some(id), Some(p)) = (v.id(), v.child(0x09c5).map(Record::floats).filter(|p| p.len() >= 3)) {
                    places.vertices.insert(id, DVec3::new(p[0], p[1], p[2]));
                }
            }
            let components = entities.child(0x138c).into_iter().flat_map(|r| r.all(0x1964));
            let groups = entities.child(0x138d).into_iter().flat_map(|r| r.all(0x1d4c)).filter_map(|g| g.child(0x1964));
            for instance in components.chain(groups) {
                let transform = instance.child(0x1966).map(Record::floats).and_then(|m| placement(&m)).unwrap_or(DAffine3::IDENTITY);
                if let (Some(id), Some(definition)) = (instance.id(), instance.child(0x1967).and_then(Record::int)) {
                    places.instances.insert(id, (definition, transform));
                }
            }
        }
        places
    }

    /// The point a reference (`5208`) names, in the coordinates of the entity list holding it: a
    /// vertex by id (type 2) or a point (`520a`) in the coordinates of the thing it's on, carried
    /// out through the instances on its path (`520b` > `53fc` > `53fe`, outermost first); type 12
    /// gives its point in the holding list's coordinates already.
    fn resolve(&self, reference: Record<'_>) -> Option<DVec3> {
        let reference = reference.child(0x5208)?;
        let target = reference.child(0x520b)?.child(0x53fc)?;
        let point = || -> Option<DVec3> {
            let p = reference.child(0x520a)?.floats();
            Some(DVec3::new(*p.first()?, *p.get(1)?, *p.get(2)?))
        };
        let local = match reference.child(0x5209).and_then(Record::int) {
            Some(2) => *self.vertices.get(&target.child(0x53fd)?.int()?)?,
            Some(12) => return point().filter(|p| p.is_finite()),
            _ => point()?,
        };
        let path = ids(target.child(0x53fe).map_or(&[][..], |r| r.body));
        let mut transform = DAffine3::IDENTITY;
        for instance in path {
            transform *= self.instances.get(&instance)?.1;
        }
        Some(transform.transform_point3(local)).filter(|p| p.is_finite())
    }
}

/// A run of ids, each a length byte then that many bytes (little-endian), as folders list their tags.
fn ids(bytes: &[u8]) -> Vec<u64> {
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(&len) = bytes.get(at) {
        let Some(id) = bytes.get(at + 1..at + 1 + usize::from(len)) else { break };
        if id.is_empty() || id.len() > 8 {
            break;
        }
        out.push(id.iter().rev().fold(0u64, |n, &b| (n << 8) | u64::from(b)));
        at += 1 + usize::from(len);
    }
    out
}

/// A loop's corners, following its edges end to end. Each edge's direction in the loop comes
/// from which end the next edge shares; the loop's own flag (forward or not) settles a loop
/// that can't be followed that way.
fn walk(edges_in_loop: &[(u64, bool)], edges: &HashMap<u64, (u64, u64)>) -> Option<Vec<u64>> {
    let ends: Vec<(u64, u64, bool)> = edges_in_loop.iter().map(|&(e, forward)| edges.get(&e).map(|&(a, b)| (a, b, forward))).collect::<Option<_>>()?;
    if ends.len() < 3 {
        return None;
    }
    let chain = |start: u64| -> Option<Vec<u64>> {
        let mut at = start;
        let mut corners = Vec::with_capacity(ends.len());
        for &(a, b, _) in &ends {
            corners.push(at);
            at = if a == at {
                b
            } else if b == at {
                a
            } else {
                return None;
            };
        }
        (at == start).then_some(corners)
    };
    let (a, b, forward) = ends[0];
    chain(a).or_else(|| chain(b)).or_else(|| {
        // By the flags: each edge starts at its start when forward, at its end otherwise.
        Some(ends.iter().map(|&(a, b, f)| if f == forward { a } else { b }).collect())
    })
}

fn newell(points: &[DVec3]) -> DVec3 {
    (0..points.len()).fold(DVec3::ZERO, |n, i| n + points[i].cross(points[(i + 1) % points.len()]))
}

/// An instance's placement: 13 numbers, the rotation's three rows, the translation, and a divisor
/// (1 normally). (Read as columns, a beam whose length runs along its own Z lands backwards.)
fn placement(m: &[f64]) -> Option<DAffine3> {
    if m.len() < 12 {
        return None;
    }
    let w = m.get(12).copied().filter(|w| w.abs() > 1e-12).unwrap_or(1.0);
    let v = |i: usize| DVec3::new(m[i], m[i + 1], m[i + 2]) / w;
    let rotation = glam::DMat3::from_cols(v(0), v(3), v(6)).transpose();
    let affine = DAffine3 { matrix3: rotation, translation: v(9) };
    affine.is_finite().then_some(affine)
}

/// Wood named for a species (`cedar`) reads as that species' boards; parts made of it that are
/// lumber-sized (a 2x10's 1 1/2" x 9 1/4", 5/4 decking) are lumber instead, so they're matched to
/// lumber sections. When most of its parts are, the material becomes lumber; when only some are
/// (a rack's 2x4 frame and 3/4" slats), those are painted with a lumber copy of it. Only 5/4 and
/// thicker sections count: 3/4" boards are usually hardwood, not 1x lumber.
fn lumber_by_size(scene: &mut Scene) {
    let lumber = crate::cutplan::Standard::for_kind(MaterialKind::DimensionalLumber);
    let sections: Vec<(f64, f64)> = lumber.sections.iter().copied().filter(|&(t, _)| t >= 1.0 - 1e-9).collect();
    let near = |a: f64, b: f64| (a - b).abs() <= 1.0 / 32.0;
    let is_lumber = |size: DVec3| {
        let mut s = [size.x, size.y, size.z];
        s.sort_by(f64::total_cmp);
        sections.iter().any(|&(t, w)| near(s[0], t) && near(s[1], w))
    };
    // Each part painted with a material: where it's placed (`None`: the top level, else the
    // definition holding it), whether its paint is on the instance, and whether it's lumber-sized.
    struct Part {
        holder: Option<DefinitionId>,
        instance: crate::InstanceId,
        painted: bool,
        lumber: bool,
    }
    let mut parts: HashMap<MaterialId, Vec<Part>> = HashMap::new();
    let containers = std::iter::once((None, &scene.root)).chain(scene.definitions().map(|(id, d)| (Some(id), &d.container)));
    for (holder, container) in containers {
        for (instance, placed) in container.instances() {
            let Some(definition) = scene.definition(placed.definition) else { continue };
            let model = &definition.container.model;
            let paint = placed.material.or_else(|| model.faces().next().and_then(|(_, f)| f.material));
            if let (Some(paint), Some((lo, hi))) = (paint, model.bounds()) {
                parts.entry(paint).or_default().push(Part { holder, instance, painted: placed.material.is_some(), lumber: is_lumber(hi - lo) });
            }
        }
    }
    for (id, parts) in parts {
        let Some(material) = scene.material(id).filter(|m| m.kind == MaterialKind::SolidWood) else { continue };
        let lumbered = parts.iter().filter(|p| p.lumber).count();
        if lumbered < 2 {
            continue;
        }
        let name = material.name.to_lowercase();
        let preset = crate::library::LIBRARY
            .iter()
            .filter(|p| p.kind == MaterialKind::DimensionalLumber)
            .find(|p| p.name.strip_suffix(" Lumber").is_some_and(|species| name.contains(&species.to_lowercase())));
        let mut as_lumber = material.clone();
        as_lumber.kind = MaterialKind::DimensionalLumber;
        as_lumber.standard = preset.map_or(lumber.clone(), |p| p.standard());
        if lumbered * 5 >= parts.len() * 3 {
            if let Some(m) = scene.material_mut(id) {
                *m = as_lumber;
            }
            continue;
        }
        // Some of each: a lumber copy for the lumber-sized parts painted as a whole.
        as_lumber.name = format!("{} (lumber)", as_lumber.name);
        let copy = scene.add_material(as_lumber);
        for part in parts.iter().filter(|p| p.lumber && p.painted) {
            let container = match part.holder {
                None => Some(&mut scene.root),
                Some(d) => scene.definition_mut(d).map(|d| &mut d.container),
            };
            if let Some(instance) = container.and_then(|c| c.instance_mut(part.instance)) {
                instance.material = Some(copy);
            }
        }
    }
}

/// `colorRed="15" colorGreen="15" colorBlue="15"` in a `material.xml`.
fn xml_color(xml: &str) -> Option<[u8; 3]> {
    let channel = |name: &str| -> Option<u8> {
        let at = xml.find(&format!("{name}=\""))? + name.len() + 2;
        xml[at..].split('"').next()?.trim().parse().ok()
    };
    Some([channel("colorRed")?, channel("colorGreen")?, channel("colorBlue")?])
}

/// Words in a material's name that say it's wood: found anywhere in it, or as a whole word.
const WOOD_PARTS: [&str; 18] = [
    "wood", "lumber", "timber", "plank", "veneer", "mahogany", "walnut", "cherry", "maple", "cedar", "poplar", "hickory", "cypress",
    "spruce", "bamboo", "sapele", "rosewood", "decking",
];
const WOOD_WORDS: [&str; 14] = ["ash", "oak", "fir", "pine", "teak", "alder", "aspen", "birch", "beech", "elm", "board", "boards", "larch", "ipe"];

/// A drawing material for a SketchUp one: the library's when the name names a wood or sheet in it
/// (`Walnut`, `[Wood_Cherry]`, `Plywood 3/4`), else a sheet good or solid wood when the name says so,
/// else not wood at all (metal, glass, grass, paint colors: never cut from stock).
fn material_for(name: &str, color: [u8; 3]) -> Material {
    let plain = name.replace(['_', '[', ']'], " ").to_lowercase();
    let preset = crate::library::LIBRARY
        .iter()
        .filter(|p| plain.contains(&p.name.to_lowercase()))
        .max_by_key(|p| p.name.len());
    if let Some(preset) = preset {
        let mut material = preset.material();
        material.name = name.to_owned();
        material.color = color;
        return material;
    }
    let words: Vec<&str> = plain.split(|c: char| !c.is_alphabetic()).filter(|w| !w.is_empty()).collect();
    let sheet = ["plywood", "mdf", "hardboard", "melamine", "particle", "osb"].iter().any(|s| plain.contains(s));
    // Long names anywhere (`burlywood`), short ones only as words (`ash`, not `washer`).
    let wood = WOOD_PARTS.iter().any(|s| plain.contains(s)) || WOOD_WORDS.iter().any(|s| words.contains(s));
    if sheet {
        Material::new(name, color, MaterialKind::SheetGood, false)
    } else if wood {
        Material::new(name, color, MaterialKind::SolidWood, true)
    } else {
        Material::new(name, color, MaterialKind::Other, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Guide;

    /// Records written the way `.skp` files lay them out.
    fn rec(tag: u16, body: &[u8]) -> Vec<u8> {
        let mut out = tag.to_le_bytes().to_vec();
        out.extend((body.len() as u32).to_le_bytes());
        out.extend(body);
        out
    }

    fn id(n: u16) -> Vec<u8> {
        rec(0x05dc, &rec(0x05de, &n.to_le_bytes()))
    }

    fn floats(v: &[f64]) -> Vec<u8> {
        v.iter().flat_map(|f| f.to_le_bytes()).collect()
    }

    /// A 2x1 rectangle on the ground, as a definition's entities.
    fn rectangle_entities(entity: u16) -> Vec<u8> {
        rectangle_entities_with(entity, &[])
    }

    /// The same, with `flagged` edges carrying flags (`07d3`).
    fn rectangle_entities_with(entity: u16, flagged: &[(u16, u8)]) -> Vec<u8> {
        let corners = [(10, [0.0, 0.0, 0.0]), (11, [2.0, 0.0, 0.0]), (12, [2.0, 1.0, 0.0]), (13, [0.0, 1.0, 0.0])];
        let vertices: Vec<u8> = corners.iter().flat_map(|(n, p)| rec(0x09c4, &[id(*n), rec(0x09c5, &floats(p))].concat())).collect();
        // Edges listed in a jumbled direction, as in real files.
        let edge_ends = [(20, 10, 11), (21, 12, 11), (22, 12, 13), (23, 10, 13)];
        let edges: Vec<u8> = edge_ends
            .iter()
            .flat_map(|(e, a, b)| {
                let flags = flagged.iter().find(|(f, _)| f == e).map(|(_, f)| rec(0x07d3, &[*f])).unwrap_or_default();
                rec(0x0bb8, &[rec(0x07d0, &[id(*e), flags].concat()), rec(0x0bb9, &(*a as u16).to_le_bytes()), rec(0x0bba, &(*b as u16).to_le_bytes())].concat())
            })
            .collect();
        let coedges: Vec<u8> = [20u16, 21, 22, 23].iter().flat_map(|e| rec(0x0fa0, &[rec(0x0fa1, &e.to_le_bytes()), rec(0x0fa2, &[0])].concat())).collect();
        let face = rec(
            0x0dac,
            &[rec(0x07d0, &[id(30), rec(0x07d1, &77u16.to_le_bytes())].concat()), rec(0x0dad, &floats(&[0.0, 0.0, -1.0, 0.0])), rec(0x0dae, &rec(0x1194, &rec(0x1195, &coedges)))].concat(),
        );
        rec(0x1388, &[rec(0x07d0, &id(entity)), rec(0x1389, &vertices), rec(0x138a, &edges), rec(0x138b, &face)].concat())
    }

    fn file(model: &[u8], material_xml: &str) -> Vec<u8> {
        let mut zip_bytes = Vec::new();
        {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_bytes));
            let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
            zip.start_file("model.dat", options).unwrap();
            std::io::Write::write_all(&mut zip, model).unwrap();
            zip.start_file("materials/Walnut/material.xml", options).unwrap();
            std::io::Write::write_all(&mut zip, material_xml.as_bytes()).unwrap();
            zip.finish().unwrap();
        }
        let mut out = vec![0xff, 0xfe, 0xff, 0x0a];
        out.extend("{26.2.243}".encode_utf16().flat_map(u16::to_le_bytes));
        out.extend(zip_bytes);
        out
    }

    fn drawing() -> Vec<u8> {
        let material = rec(0x32c8, &[id(77), rec(0x32cc, b"Walnut"), rec(0x32ce, &rec(0x2134, b"materials/Walnut/thumbnail.jpg"))].concat());
        let materials = rec(0x01f7, &rec(0x30d4, &rec(0x30d5, &material)));
        let definition = rec(0x157c, &[rectangle_entities(50), rec(0x157e, b"Shelf")].concat());
        let definitions = rec(0x01f9, &rec(0x1770, &rec(0x1771, &definition)));
        // Two instances: one in place, one turned 90 degrees and moved up 10".
        let instance = |m: &[f64]| rec(0x1964, &[rec(0x07d0, &id(60)), rec(0x1966, &floats(m)), rec(0x1967, &50u16.to_le_bytes())].concat());
        let placed = [
            instance(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]),
            instance(&[0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 10.0, 1.0]),
        ]
        .concat();
        let root = rec(0x01f6, &rec(0x1388, &[rec(0x07d0, &id(1)), rec(0x138c, &placed)].concat()));
        let top = rec(0x01f4, &[materials, definitions, root].concat());
        file(&top, r#"<mat:material name="Walnut" colorRed="94" colorGreen="64" colorBlue="40">"#)
    }

    #[test]
    fn reads_components_faces_materials_and_placement() {
        let imported = read(&drawing()).unwrap();
        assert_eq!(imported.version, "26.2.243");
        assert_eq!(imported.skipped_faces, 0);
        let scene = &imported.scene;
        let instances: Vec<&Instance> = scene.root.instances().map(|(_, i)| i).collect();
        assert_eq!(instances.len(), 2);
        assert_eq!(instances[0].definition, instances[1].definition, "copies of one component");
        assert_eq!(instances[1].transform.translation.z, 10.0);
        let definition = scene.definition(instances[0].definition).unwrap();
        assert_eq!(definition.name, "Shelf");
        let model = &definition.container.model;
        assert_eq!(model.faces().count(), 1);
        let (_, face) = model.faces().next().unwrap();
        assert!(face.normal.z < -0.99, "faces keep their side: {:?}", face.normal);
        let walnut = scene.material(face.material.unwrap()).unwrap();
        assert_eq!((walnut.name.as_str(), walnut.color, walnut.kind), ("Walnut", [94, 64, 40], MaterialKind::SolidWood));
    }

    #[test]
    fn reads_groups_as_well_as_components() {
        // Groups are listed apart from component instances: `138d` > `1d4c` > `1964`.
        let definitions = rec(
            0x01f9,
            &rec(0x1770, &rec(0x1771, &[rec(0x157c, &[rectangle_entities(50), rec(0x157e, b"Group#7")].concat()), rec(0x157c, &[rectangle_entities(51), rec(0x157e, b"Group#8")].concat())].concat())),
        );
        let group = |definition: u16, name: &[u8]| {
            let m = floats(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
            rec(0x1d4c, &rec(0x1964, &[rec(0x07d0, &id(60)), rec(0x1965, name), rec(0x1966, &m), rec(0x1967, &definition.to_le_bytes())].concat()))
        };
        let root = rec(0x01f6, &rec(0x1388, &[rec(0x07d0, &id(1)), rec(0x138d, &[group(50, b"North Wall"), group(51, b"")].concat())].concat()));
        let imported = read(&file(&rec(0x01f4, &[definitions, root].concat()), "")).unwrap();
        let scene = &imported.scene;
        let mut names: Vec<(String, DefinitionKind)> =
            scene.root.instances().map(|(_, i)| scene.definition(i.definition).unwrap()).map(|d| (d.name.clone(), d.kind)).collect();
        names.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(names, vec![("Group".to_owned(), DefinitionKind::Group), ("North Wall".to_owned(), DefinitionKind::Group)]);
    }

    #[test]
    fn hidden_objects_go_on_a_hidden_tag() {
        let definitions = rec(0x01f9, &rec(0x1770, &rec(0x1771, &rec(0x157c, &[rectangle_entities(50), rec(0x157e, b"Board")].concat()))));
        let m = floats(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
        let instance = |flags: u8| rec(0x1964, &[rec(0x07d0, &[id(60), rec(0x07d3, &[flags])].concat()), rec(0x1966, &m), rec(0x1967, &50u16.to_le_bytes())].concat());
        let root = rec(0x01f6, &rec(0x1388, &[rec(0x07d0, &id(1)), rec(0x138c, &[instance(6), instance(7)].concat())].concat()));
        let scene = read(&file(&rec(0x01f4, &[definitions, root].concat()), "")).unwrap().scene;
        let tags: Vec<Option<TagId>> = scene.root.instances().map(|(_, i)| i.tag).collect();
        assert_eq!(tags[0], None, "shown");
        let hidden = scene.tag(tags[1].expect("hidden one tagged")).unwrap();
        assert_eq!((hidden.name.as_str(), hidden.visible), ("Hidden in SketchUp", false));
    }

    #[test]
    fn reads_guide_lines_and_construction_lines() {
        let endless = rec(0x4269, &rec(0x426a, &floats(&[5.0, 6.0, 7.0, 1.0, 0.0, 0.0, -1e30, 1e30])));
        let segment = rec(0x426c, &[rec(0x426d, &floats(&[1.0, 1.0, 10.0])), rec(0x426e, &floats(&[1.0, 1.0, 2.0]))].concat());
        let root = rec(0x01f6, &rec(0x1388, &[rec(0x07d0, &id(1)), rec(0x1391, &endless), rec(0x1392, &segment)].concat()));
        let scene = read(&file(&rec(0x01f4, &root), "")).unwrap().scene;
        let guides: Vec<Guide> = scene.root.model.guides().map(|(_, g)| g.clone()).collect();
        assert_eq!(guides, vec![
            Guide::Line { point: DVec3::new(5.0, 6.0, 7.0), dir: DVec3::X },
            Guide::Segment { a: DVec3::new(1.0, 1.0, 10.0), b: DVec3::new(1.0, 1.0, 2.0) },
        ]);
    }

    #[test]
    fn softened_and_hidden_edges_arent_drawn() {
        // Edge 20 softened (0e), 21 hidden (07), the others plain (06).
        let entities = rectangle_entities_with(1, &[(20, 0x0e), (21, 0x07), (22, 0x06)]);
        let scene = read(&file(&rec(0x01f4, &rec(0x01f6, &entities)), "")).unwrap().scene;
        let model = &scene.root.model;
        assert_eq!(model.edges().filter(|(_, e)| e.hidden).count(), 2);
        assert_eq!(model.edges().count(), 4);
    }

    /// A reference (`5208`) to vertex `vertex` through `path` (instance ids), or with `point` a point
    /// of `kind` given directly.
    fn reference(kind: u8, point: [f64; 3], vertex: u16, path: &[u16]) -> Vec<u8> {
        let path: Vec<u8> = path.iter().flat_map(|i| [&[2u8][..], &i.to_le_bytes()].concat()).collect();
        rec(
            0x5208,
            &[rec(0x5209, &[kind, 0, 0, 0]), rec(0x520a, &floats(&point)), rec(0x520b, &rec(0x53fc, &[rec(0x53fd, &vertex.to_le_bytes()), rec(0x53fe, &path)].concat()))].concat(),
        )
    }

    #[test]
    fn reads_dimensions_and_text_on_geometry_inside_a_component() {
        // The 2x1 rectangle component placed 10" up; a dimension across its long side, from
        // corner 10 to corner 11 through that placement, 3" out; a note on corner 12.
        let definitions = rec(0x01f9, &rec(0x1770, &rec(0x1771, &rec(0x157c, &[rectangle_entities(50), rec(0x157e, b"Shelf")].concat()))));
        let m = floats(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 10.0, 1.0]);
        let instance = rec(0x1964, &[rec(0x07d0, &id(60)), rec(0x1966, &m), rec(0x1967, &50u16.to_le_bytes())].concat());
        let dimension = rec(
            0x5bcc,
            &[
                rec(0x5bcd, &reference(2, [0.0; 3], 10, &[60])),
                rec(0x5bce, &reference(2, [0.0; 3], 11, &[60])),
                rec(0x5bcf, &floats(&[0.0, 0.0, 1.0])),
                rec(0x5bd2, &floats(&[3.0])),
            ]
            .concat(),
        );
        // A point given in the top level's coordinates (type 12): no placement applied.
        let free = rec(0x5bcc, &[rec(0x5bcd, &reference(12, [5.0, 5.0, 0.0], 0, &[60])), rec(0x5bce, &reference(2, [0.0; 3], 13, &[60]))].concat());
        let text = rec(0x55f0, &[rec(0x55f1, b"Glue here"), rec(0x55f4, &reference(2, [0.0; 3], 12, &[60])), rec(0x55f5, &floats(&[1.0, 2.0, 3.0]))].concat());
        let root = rec(
            0x01f6,
            &rec(0x1388, &[rec(0x07d0, &id(1)), rec(0x138c, &instance), rec(0x1398, &text), rec(0x1399, &[dimension, free].concat())].concat()),
        );
        let scene = read(&file(&rec(0x01f4, &[definitions, root].concat()), "")).unwrap().scene;
        let notes: Vec<Guide> = scene.root.model.guides().map(|(_, g)| g.clone()).collect();
        let up = DVec3::Z * 10.0;
        assert_eq!(notes.len(), 3, "{notes:?}");
        assert!(notes.contains(&Guide::Text { at: DVec3::new(2.0, 1.0, 0.0) + up, offset: DVec3::new(1.0, 2.0, 3.0), text: "Glue here".into() }));
        let Some(Guide::Dimension { a, b, offset }) = notes.iter().find(|g| matches!(g, Guide::Dimension { a, .. } if a.z == 10.0 && a.x == 0.0 && a.y == 0.0)) else { panic!("{notes:?}") };
        assert_eq!((*a, *b), (up, DVec3::X * 2.0 + up), "through the placement");
        assert!((offset.length() - 3.0).abs() < 1e-9 && offset.dot(DVec3::X).abs() < 1e-9, "3\" out, square to it: {offset}");
        assert!(notes.iter().any(|g| matches!(g, Guide::Dimension { a, .. } if *a == DVec3::new(5.0, 5.0, 0.0))), "type 12 taken as it is");
    }

    /// A part's entities spanning `size` (one edge, corner to corner: enough for its box).
    fn sized_entities(entity: u16, size: [f64; 3]) -> Vec<u8> {
        let vertices = [rec(0x09c4, &[id(entity + 1), rec(0x09c5, &floats(&[0.0; 3]))].concat()), rec(0x09c4, &[id(entity + 2), rec(0x09c5, &floats(&size))].concat())].concat();
        let edge = rec(0x0bb8, &[rec(0x07d0, &id(entity + 3)), rec(0x0bb9, &(entity + 1).to_le_bytes()), rec(0x0bba, &(entity + 2).to_le_bytes())].concat());
        rec(0x1388, &[rec(0x07d0, &id(entity)), rec(0x1389, &vertices), rec(0x138a, &edge)].concat())
    }

    #[test]
    fn wood_whose_parts_are_lumber_sized_reads_as_lumber() {
        let material = |id_: u16, name: &[u8]| rec(0x32c8, &[id(id_), rec(0x32cc, name)].concat());
        let materials = rec(0x01f7, &rec(0x30d4, &rec(0x30d5, &[material(77, b"cedar"), material(78, b"cedar boards"), material(79, b"pine")].concat())));
        let definition = |entity: u16, size: [f64; 3]| rec(0x157c, &[sized_entities(entity, size), rec(0x157e, b"Part")].concat());
        let definitions = rec(
            0x01f9,
            &rec(0x1770, &rec(0x1771, &[definition(100, [1.5, 9.25, 144.0]), definition(110, [9.25, 1.5, 120.0]), definition(120, [0.75, 5.5, 30.0])].concat())),
        );
        let m = floats(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
        let instance = |definition: u16, paint: u16| {
            rec(0x1964, &[rec(0x07d0, &[id(60), rec(0x07d1, &paint.to_le_bytes())].concat()), rec(0x1966, &m), rec(0x1967, &definition.to_le_bytes())].concat())
        };
        // Pine: a 2x4 frame (two lumber-sized parts) and three 3/4" slats.
        let placed = [instance(100, 77), instance(110, 77), instance(120, 78), instance(120, 78), instance(100, 79), instance(110, 79), instance(120, 79), instance(120, 79), instance(120, 79)].concat();
        let root = rec(0x01f6, &rec(0x1388, &[rec(0x07d0, &id(1)), rec(0x138c, &placed)].concat()));
        let scene = read(&file(&rec(0x01f4, &[materials, definitions, root].concat()), "")).unwrap().scene;
        let kind = |name: &str| scene.materials().find(|(_, m)| m.name == name).map(|(_, m)| m.kind).unwrap();
        assert_eq!(kind("cedar"), MaterialKind::DimensionalLumber, "2x10s");
        assert_eq!(kind("cedar boards"), MaterialKind::SolidWood, "3/4\" boards stay boards");
        let cedar = scene.materials().find(|(_, m)| m.name == "cedar").unwrap().1;
        assert!(cedar.standard.sections.iter().any(|&(t, w)| t == 1.5 && w == 9.25));
        assert_eq!(kind("pine"), MaterialKind::SolidWood, "the slats");
        assert_eq!(kind("pine (lumber)"), MaterialKind::DimensionalLumber, "the frame");
        let lumber = scene.materials().find(|(_, m)| m.name == "pine (lumber)").unwrap().0;
        assert_eq!(scene.root.instances().filter(|(_, i)| i.material == Some(lumber)).count(), 2);
    }

    #[test]
    fn placements_are_rotation_rows_then_translation() {
        // A beam lying along its own Z, laid along red (as in a real file): its own Z becomes +X.
        let m = [0.0, 0.0, 1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0, -7.0, 1.0, 82.0, 1.0];
        let p = placement(&m).unwrap();
        assert!((p.transform_point3(DVec3::new(0.0, 0.0, 95.5)) - DVec3::new(88.5, 1.0, 82.0)).length() < 1e-9);
        assert!((p.transform_vector3(DVec3::X) - DVec3::new(0.0, 0.0, -1.0)).length() < 1e-9);
    }

    #[test]
    fn folders_list_their_tags_as_length_and_bytes() {
        assert_eq!(ids(&[3, 0xbe, 0xce, 0x10, 2, 0x34, 0x12]), vec![0x10cebe, 0x1234]);
        assert_eq!(ids(&[]), Vec::<u64>::new());
    }

    #[test]
    fn refuses_what_it_cant_read() {
        assert_eq!(read(b"hello").unwrap_err(), SkpError::NotSketchUp);
        assert_eq!(read(&[0xff, 0xfe, 0xff, 0x0e, 0, 0, 0, 0]).unwrap_err(), SkpError::TooOld);
    }

    #[test]
    fn library_woods_are_recognized_in_sketchup_names() {
        assert_eq!(material_for("[Wood_Cherry_Original]", [1, 2, 3]).kind, MaterialKind::SolidWood);
        assert_eq!(material_for("Baltic Birch Plywood 3/4", [1, 2, 3]).standard.sheets, vec![(60.0, 60.0)]);
        assert_eq!(material_for("Sheet MDF", [1, 2, 3]).kind, MaterialKind::SheetGood);
        for wood in ["cedar", "pine-ponderosa", "[0036_BurlyWood]", "White Ash", "alder1", "Wood Veneer 02"] {
            assert_eq!(material_for(wood, [1, 2, 3]).kind, MaterialKind::SolidWood, "{wood}");
        }
        for other in ["[Metal Corrugated Shiny]", "[Grass Dark Green]", "[Color_009]", "Black silk leather", "Washer", "Sree_Hair"] {
            assert_eq!(material_for(other, [1, 2, 3]).kind, MaterialKind::Other, "{other}");
        }
    }

    /// A real file, when one is given: `WOODSKETCH_SKP=path cargo test -p woodsketch-doc skp -- --nocapture`.
    #[test]
    fn reads_a_real_file() {
        let Ok(path) = std::env::var("WOODSKETCH_SKP") else { return };
        let imported = read(&std::fs::read(&path).unwrap()).unwrap();
        let scene = &imported.scene;
        println!("{path}: SketchUp {}, {} skipped faces", imported.version, imported.skipped_faces);
        for (_, d) in scene.definitions() {
            println!("  {}: {} faces, {} instances", d.name, d.container.model.faces().count(), d.container.instances().count());
            for (_, g) in d.container.model.guides().filter(|(_, g)| g.is_note()) {
                println!("    note {g:?}");
            }
        }
        println!("  top level: {} faces, {} instances", scene.root.model.faces().count(), scene.root.instances().count());
        for (_, g) in scene.root.model.guides().filter(|(_, g)| g.is_note()) {
            println!("  note {g:?}");
        }
        println!("  materials: {:?}", scene.materials().map(|(_, m)| m.name.clone()).collect::<Vec<_>>());
        for (id, t) in scene.tags() {
            println!("  tag {}{}{}", scene.tag_path(id), if t.folder { " /" } else { "" }, if t.visible { "" } else { " (hidden)" });
        }
    }
}
