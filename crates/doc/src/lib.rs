//! The WoodSketch document: geometry, groups and components, selection and undo history.
//! No UI or GPU dependencies.

pub mod angled;
mod arena;
pub mod assembly;
pub mod board;
pub mod buildsheet;
pub mod clipboard;
pub mod curves;
pub mod cutlist;
pub mod cutplan;
pub mod cutsheet;
mod document;
pub mod export;
pub mod expr;
pub mod feedback;
pub mod file;
pub mod flat;
pub mod graphics;
pub mod hardware;
pub mod joinery;
pub mod library;
mod lines;
mod model;
pub mod movement;
mod offset;
pub mod parametric;
pub mod part;
pub mod profile;
mod push_pull;
pub mod recipe;
pub mod recipe_of;
pub mod samples;
mod scene;
pub mod skp;
pub mod solid;
pub mod template;
mod transform;
pub mod units;

pub use document::{Document, ReferenceMarks, Selection};
pub use model::{EPS, Edge, EdgeId, Entity, Face, FaceEntry, FaceId, Guide, GuideId, Model, VertexId};
pub use offset::OffsetError;
pub use push_pull::PushPullError;
pub use scene::{
    Container, Definition, DefinitionId, DefinitionKind, Instance, InstanceId, Material, MaterialId, MaterialKind, Owner, Scene, Tag, TagId, View,
};
pub use transform::{MoveError, mirroring, rotation, scaling};
