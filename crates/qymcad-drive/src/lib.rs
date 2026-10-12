//! A headless session over one document.
//!
//! The session owns the project and the kernel that rebuilds it, and hands back the tree. The project
//! stays inside this crate.

use qymcad_core::feature::ComponentKind;
use qymcad_core::model::Project;
use qymcad_kernel::OcctKernel;

/// The tree of a document after a rebuild.
#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    /// Parts and assemblies, in the order the document holds them.
    pub components: Vec<Component>,
    /// Sketches, in document order.
    pub sketches: Vec<Sketch>,
    /// Timeline nodes, in timeline order.
    pub nodes: Vec<Node>,
    /// Bodies the rebuild produced, in document order.
    pub bodies: Vec<Body>,
    /// What the rebuild reported.
    pub rebuild: Rebuild,
}

/// A part or an assembly.
#[derive(Clone, Debug, PartialEq)]
pub struct Component {
    pub id: u64,
    pub name: String,
    pub kind: Kind,
    /// The assembly this one stands in. `None` for the root.
    pub parent: Option<u64>,
}

/// Which container a component is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Part,
    Assembly,
}

/// Where a sketch lies. A world plane is one of the three coordinate planes. A datum or a face is named
/// by the id the document already uses.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Plane {
    Xy,
    Xz,
    Yz,
    Datum { id: u64 },
    Face { body: u64 },
}

/// A coordinate plane. A datum or a face is not a place this call can draw: a face needs a key, and a
/// datum has to exist already. Those show up on the tree when a document already holds them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WorldPlane {
    Xy,
    Xz,
    Yz,
}

/// One corner of a rectangle. It is its own type so it cannot be passed where the opposite corner goes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct From {
    pub at: (f64, f64),
}

/// The opposite corner of a rectangle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct To {
    pub at: (f64, f64),
}

/// The two corners of a rectangle, in the plane of the sketch.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Corners {
    pub from: From,
    pub to: To,
}

/// Whether a contour is a closed loop or an open chain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Form {
    Closed,
    Open,
}

/// A contour of a sketch, as the rebuild left it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contour {
    pub id: u64,
    pub form: Form,
}

/// The sketch and the contour a rectangle produced. The two ids are different types, so one cannot be
/// passed where the other goes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Profile {
    pub sketch: SketchId,
    pub contour: ContourId,
}

/// The id of a sketch.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SketchId {
    pub id: u64,
}

/// The id of a contour.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContourId {
    pub id: u64,
}

/// How far an extrusion grows from the sketch plane.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Height {
    pub mm: f64,
}

/// The id of a body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyId {
    pub id: u64,
}

/// The name of a document parameter. It is its own type so it cannot be passed where the formula goes.
#[derive(Clone, Debug, PartialEq)]
pub struct ParameterName {
    pub text: String,
}

/// A formula. It is its own type so it cannot be passed where a parameter name goes.
#[derive(Clone, Debug, PartialEq)]
pub struct Expression {
    pub text: String,
}

/// Which number of a body an expression drives. The only one this call sets is the extrusion height, so
/// the dimension key cannot be a free string.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Dimension {
    Height,
}

/// The volume of a live shape, in mm³.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Volume {
    pub mm3: f64,
}

/// Whether the live shape passed the kernel's validity check.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Sound {
    Valid,
    Invalid,
}

/// The live measurement of a body, or the fact that the rebuild left no shape.
///
/// A missing shape has no volume. Zero would read as an empty solid the rebuild had accepted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Measure {
    Live { volume: Volume, solids: u32 },
    Absent,
}

/// A body in the document.
#[derive(Clone, Debug, PartialEq)]
pub struct Body {
    pub id: u64,
    pub name: String,
    pub sound: Sound,
    pub measure: Measure,
    /// Flat faces from the rebuild, each named by the document's name table.
    pub faces: Vec<Face>,
    /// Edges from the rebuild, each named by the pair of faces it lies between.
    pub edges: Vec<Edge>,
}

/// The name-table descriptor of a face. A kernel ordinal is not a face id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FaceId {
    pub id: u32,
}

/// The name-table descriptor of an edge. A kernel ordinal is not an edge id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EdgeId {
    pub id: u32,
}

/// The sketch line a wall was extruded from. It is not a body id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LineId {
    pub id: u64,
}

/// How the recipe names a face.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FaceRole {
    /// The cap on the sketch plane.
    CapStart,
    /// The cap at the end of the extrusion.
    CapEnd,
    /// A side face produced by one sketch line.
    Wall { source: LineId },
    /// A recipe this read does not name further.
    Other,
}

/// The area of a face, in mm².
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Area {
    pub mm2: f64,
}

/// The length of an edge, in mm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Length {
    pub mm: f64,
}

/// A flat face of a body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Face {
    pub id: FaceId,
    pub role: FaceRole,
    pub area: Area,
}

/// An edge of a body. The two faces it lies between stay in the name table.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge {
    pub id: EdgeId,
    pub length: Length,
}

/// A sketch, named, on a plane, with the contours the rebuild wrote.
#[derive(Clone, Debug, PartialEq)]
pub struct Sketch {
    pub id: u64,
    pub name: String,
    pub plane: Plane,
    pub contours: Vec<Contour>,
}

/// One node of the timeline.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: u64,
    pub name: String,
    /// The component the node belongs to.
    pub parent: Option<u64>,
}

/// A node the rebuild could not finish, in the core's own words.
#[derive(Clone, Debug, PartialEq)]
pub struct Refusal {
    pub node: u64,
    pub text: String,
}

/// The outcome of one rebuild.
#[derive(Clone, Debug, PartialEq)]
pub enum Rebuild {
    /// No errors, and the pass was not cancelled.
    Quiet,
    /// One or more nodes failed. The text is the core error's own wording.
    Refused { refusals: Vec<Refusal> },
    /// The pass stopped part-way. Its report is incomplete and is not a success.
    Stopped,
}

/// One document, driven from outside the window.
///
/// The kernel's shapes are pointers that belong to the thread that built them, so the session cannot
/// be sent to another thread.
pub struct Session {
    project: Project,
    kernel: OcctKernel,
    /// Documents from before each kept change. A source record is here, and its bytes are not.
    steps: Vec<Project>,
    _on_this_thread: std::marker::PhantomData<*const ()>,
}

impl Session {
    /// The root assembly and nothing else.
    ///
    /// A part is not added here. The model keeps an empty document and a new part as two calls, and
    /// this one is the empty document.
    pub fn empty() -> Self {
        let mut project = Project::default();
        project.new_empty_document();
        Self::from_project(project)
    }

    fn from_project(project: Project) -> Self {
        let kernel = OcctKernel { quality_k: project.geom_quality.deflection_k(), ..OcctKernel::default() };
        Self { project, kernel, steps: Vec::new(), _on_this_thread: std::marker::PhantomData }
    }

    /// Rebuild the document and return its tree.
    ///
    /// Parameter expressions are evaluated first. The rebuild reads the cached number, not the
    /// expression, so a formula that was stored and never evaluated would leave the previous size.
    /// The kernel lock is taken for this rebuild and released when it returns. The volume and the
    /// validity are read before that release, because those reads call the geometry kernel. A guard
    /// stored on the session would block every other rebuild in the process for as long as the session
    /// lived.
    #[must_use]
    pub fn document(&mut self) -> Document {
        self.project.eval_parameters();
        self.kernel.quality_k = self.project.geom_quality.deflection_k();
        let gate = qymcad_kernel::kernel_gate();
        let report = self.project.regenerate(&self.kernel);
        let doc = view_of(&self.project, &self.kernel, &report);
        drop(gate);
        doc
    }

    /// Write the document to `path`.
    ///
    /// An empty document is refused when that file already holds work, and the existing file stays. The
    /// write goes to a temporary file and replaces the target only after it succeeds. A path that is not
    /// Unicode is refused first: the file layer takes a Unicode path, and a replacement character would
    /// name a different file.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<(), StoreError> {
        let path = unicode_path(path.as_ref())?;
        qymcad_io::save_project_guarded(&self.project, path).map_err(store_error)
    }

    /// Read a document from `path`.
    ///
    /// Body bytes stored beside the document are dropped. This session rebuilds the tree from the
    /// document, so those bytes would be held and never read. The file stores nodes as clean, and a
    /// clean node is skipped, so the timeline is marked dirty first: otherwise the first read would
    /// find the body record and no live shape. A path that is not Unicode is refused first, so the
    /// read cannot follow a different name.
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, StoreError> {
        let path = unicode_path(path.as_ref())?;
        qymcad_io::load_project(path).map(Self::reopened).map_err(store_error)
    }

    /// A document read from a file, on a kernel that has never built it.
    fn reopened(mut project: Project) -> Self {
        project.mark_all_dirty();
        Self::from_project(project)
    }

    /// Draw a rectangle on `plane` and return its sketch and contour.
    ///
    /// An empty document is an assembly, and a sketch there would have no part to belong to. A part is
    /// made and stood in, and the session stays there: stepping back to the root would make the next
    /// body a second part. The rectangle is regenerated before any body is built from it. A later
    /// rectangle takes the next free sketch name, so two of them are not both the first sketch.
    ///
    /// A side no longer than the distance at which a sketch merges two points is refused before anything
    /// is added. A contour id of 0 is not a contour: a later extrusion reads 0 as "the first contour".
    /// If the rectangle is drawn and still has no closed contour, the call is refused instead of
    /// returning that id.
    pub fn rectangle(&mut self, plane: WorldPlane, corners: Corners) -> Result<Profile, DrawError> {
        if !spans_a_rectangle(&corners) {
            return Err(DrawError { text: "sketch-rectangle-empty".into() });
        }
        let _made = self.project.part_to_draw_in();
        let name = free_sketch_name(&self.project);
        let si = self.project.new_sketch(&name);
        self.project.sketches[si].plane = placed(plane);
        let sid = self.project.sketches[si].id;
        let _node = self.project.add_sketch_node(sid, &name);
        let (x0, y0) = corners.from.at;
        let (x1, y1) = corners.to.at;
        let _sides = self.project.add_rect_entity(si, x0, y0, x1, y1, qymcad_core::feature::Purpose::Real);
        self.project.regen_sketch(si);
        let contour = self.project.sketches[si]
            .contour_ids
            .iter()
            .copied()
            .find(|&id| contour_of(&self.project, id).is_some_and(|c| c.form == Form::Closed))
            .ok_or_else(|| DrawError { text: "sketch-rectangle-empty".into() })?;
        Ok(Profile { sketch: SketchId { id: sid }, contour: ContourId { id: contour } })
    }

    /// Extrude `profile` by `height` and return the body.
    ///
    /// The contour id is the one the profile names. Zero would mean the first contour, which is a
    /// different instruction. The body grows forward from the sketch plane, with no second side. The
    /// session stays in the part that owns the sketch: stepping back to the root would make this body
    /// a second part. A height that is not a positive finite length is refused before a node is added:
    /// the model takes the absolute value, so a negative height would build the positive solid and the
    /// rebuild would stay quiet. A contour that is not a closed contour of that sketch is refused the
    /// same way: the model would otherwise discard it and extrude the sketch's first contour. The shape
    /// itself appears when the document is rebuilt.
    pub fn extrude(&mut self, profile: Profile, height: Height) -> Result<BodyId, DrawError> {
        if !(height.mm.is_finite() && height.mm > 0.0) {
            return Err(DrawError { text: "sketch-extrude-height".into() });
        }
        if !closed_profile(&self.project, profile) {
            return Err(DrawError { text: "error-profile-not-found".into() });
        }
        let body = self.project.add_extrude_on(profile.sketch.id, profile.contour.id, height.mm, qymcad_core::feature::Reach::Forward, 0.0);
        Ok(BodyId { id: body })
    }

    /// Store one document parameter under `name`, with `expr` as its formula.
    ///
    /// A second call with the same name, ignoring letter case, replaces that formula. It does not add
    /// another parameter, and the first spelling stays: a formula matches that stored spelling. The
    /// formula is evaluated with this name taken out of the table first. A formula
    /// that mentions itself would otherwise be repeated, and the number stored afterwards would not be
    /// the number that was checked. If the settled number is different, or a bound extrusion height
    /// would no longer evaluate to a positive length, the previous parameters are put back. A formula
    /// that stops being a number is included: the rebuild would drop it and use the height stored on
    /// the node. A negative result is included too: the model takes the absolute value, so the solid
    /// would be the positive one and the rebuild would stay quiet. A formula that does not evaluate
    /// is refused before anything is written.
    /// Every feature that already has an expression is marked dirty. A clean node is skipped. A formula
    /// matches the stored spelling, and this session treats another spelling of that name as the same
    /// parameter, so every expression is marked.
    pub fn parameter(&mut self, name: ParameterName, expr: Expression) -> Result<(), DrawError> {
        let name = name.text.trim();
        match parameter_name(name) {
            Ok(()) => {}
            Err(NameFault::Shape) => return Err(DrawError { text: "par-name-bad".into() }),
            Err(NameFault::Reserved) => return Err(DrawError { text: "par-name-reserved".into() }),
        }
        let formula = expr.text.trim();
        let folded = name.to_lowercase();
        let mut vars = self.project.param_map();
        vars.retain(|key, _| key.to_lowercase() != folded);
        let value = match qymcad_core::expr::eval(formula, &vars) {
            Ok(value) => value,
            Err(error) => return Err(DrawError { text: error.key().into() }),
        };
        let step = kept_step(&self.project);
        let parameters_before = self.project.parameters.clone();
        let sketches_before = self.project.sketches.clone();
        if let Some(existing) = self.project.parameters.iter_mut().find(|p| p.name.to_lowercase() == folded) {
            existing.expr = formula.to_string();
        } else {
            self.project.parameters.push(qymcad_core::model::Param { name: name.to_string(), expr: formula.to_string(), value: 0.0 });
        }
        let errors = self.project.eval_parameters();
        let settled = self.project.parameters.iter().find(|p| p.name.to_lowercase() == folded).map(|p| p.value);
        let unsettled = errors.iter().any(|(found, _)| found.to_lowercase() == folded) || settled.is_none_or(|got| (got - value).abs() > 1e-9);
        let refusal = if unsettled { Some(DrawError { text: "sketch-param-settle".into() }) } else { extrusion_height_refused(&self.project) };
        if let Some(error) = refusal {
            self.project.parameters = parameters_before;
            self.project.sketches = sketches_before;
            return Err(error);
        }
        self.project.mark_param_dependents_dirty();
        self.steps.push(step);
        Ok(())
    }

    /// Store `expr` as the extrusion height of `body`.
    ///
    /// The formula is evaluated before it is stored. A formula that does not evaluate is refused and
    /// nothing is written: the rebuild falls back to the stored height when an expression fails, and
    /// the volume stays where it was. A result that is not a positive length is refused the same way.
    /// The model takes the absolute value, so a negative height would build the positive solid and the
    /// rebuild would stay quiet. An id that is not an extrusion is refused before that write. Writing
    /// the expression onto a sketch id would succeed inside the model and the extrusion would not read
    /// it, so the volume would not change. The stored height number on the node is left as it is.
    pub fn bind(&mut self, body: BodyId, dimension: Dimension, expr: Expression) -> Result<(), DrawError> {
        if !is_extrusion(&self.project, body.id) {
            return Err(DrawError { text: "sketch-extrude-target".into() });
        }
        let formula = expr.text.trim();
        let mm = match qymcad_core::expr::eval(formula, &self.project.param_map()) {
            Ok(mm) => mm,
            Err(error) => return Err(DrawError { text: error.key().into() }),
        };
        if !(mm.is_finite() && mm > 0.0) {
            return Err(DrawError { text: "sketch-extrude-height".into() });
        }
        let key = match dimension {
            Dimension::Height => "height",
        };
        let step = kept_step(&self.project);
        self.project.set_feat_dim(body.id, key, formula.to_string());
        self.steps.push(step);
        Ok(())
    }

    /// Put back the document from before the last kept change.
    ///
    /// The step records each source file and leaves the bytes on the live document. They move back
    /// with the step, before the next read. The restored nodes are clean, and a clean node is skipped
    /// while the kernel still holds the shape from after the change, so the timeline is marked dirty
    /// here. A rectangle or an extrusion is not a kept change, so this call cannot remove the body
    /// those calls added. A refusal keeps no step.
    pub fn back(&mut self) -> Result<(), DrawError> {
        let Some(mut restored) = self.steps.pop() else {
            return Err(DrawError { text: "sketch-undo-empty".into() });
        };
        restored.take_source_data_from(&mut self.project);
        restored.keep_ids_past(&self.project);
        restored.mark_all_dirty();
        self.project = restored;
        Ok(())
    }

    /// Move the rectangle of `profile` to `corners`.
    ///
    /// The sketch is the one already there. The corner points of that rectangle move, and the contour
    /// is rebuilt from them, so the body below follows on the next read. Corners that do not span a
    /// rectangle, and a profile that is not a closed contour of that sketch, are refused before a step
    /// is kept.
    pub fn resize(&mut self, profile: Profile, corners: Corners) -> Result<(), DrawError> {
        if !spans_a_rectangle(&corners) {
            return Err(DrawError { text: "sketch-rectangle-empty".into() });
        }
        if !closed_profile(&self.project, profile) {
            return Err(DrawError { text: "error-profile-not-found".into() });
        }
        let step = kept_step(&self.project);
        if !move_rectangle(&mut self.project, profile, &corners) {
            return Err(DrawError { text: "sketch-rectangle-empty".into() });
        }
        self.steps.push(step);
        Ok(())
    }
}

/// A save or an open that was refused. The text is a catalogue code: one the file layer returns, or
/// `io-path-not-utf8` when the path is not Unicode and that layer cannot name the file.
#[derive(Clone, Debug, PartialEq)]
pub struct StoreError {
    pub text: String,
}

/// A drawing that was refused. The text is a catalogue code.
#[derive(Clone, Debug, PartialEq)]
pub struct DrawError {
    pub text: String,
}

fn store_error(text: String) -> StoreError {
    StoreError { text }
}

/// The file layer takes a Unicode path. Anything else is refused here, before a replacement character
/// can turn it into a different file that then succeeds or fails on its own.
fn unicode_path(path: &std::path::Path) -> Result<&str, StoreError> {
    path.to_str().ok_or_else(|| StoreError { text: "io-path-not-utf8".into() })
}

fn view_of(project: &Project, kernel: &OcctKernel, report: &qymcad_core::feature::RegenReport) -> Document {
    Document {
        components: project.components.iter().map(|c| Component { id: c.id, name: c.name.clone(), kind: kind_of(c.kind), parent: c.parent }).collect(),
        sketches: project.sketches.iter().map(|s| sketch_of(project, s)).collect(),
        nodes: project.timeline.iter().map(|n| Node { id: n.id, name: n.name.clone(), parent: n.parent }).collect(),
        bodies: bodies_of(project, kernel),
        rebuild: rebuild_of(project, report),
    }
}

/// The volume is the live shape's, read from the kernel cache the rebuild just filled. A body record
/// with no shape is invalid and carries no volume.
fn bodies_of(project: &Project, kernel: &OcctKernel) -> Vec<Body> {
    let shapes = kernel.shapes.borrow();
    project
        .bodies
        .iter()
        .map(|body| match shapes.get(&body.id) {
            Some(shape) => Body {
                id: body.id,
                name: body.name.clone(),
                sound: sound_of(shape.is_valid()),
                measure: Measure::Live { volume: Volume { mm3: shape.volume() }, solids: shape.solid_count() },
                faces: faces_of(project, body.id),
                edges: edges_of(project, body.id),
            },
            None => Body { id: body.id, name: body.name.clone(), sound: Sound::Invalid, measure: Measure::Absent, faces: Vec::new(), edges: Vec::new() },
        })
        .collect()
}

fn sound_of(valid: bool) -> Sound {
    match valid {
        true => Sound::Valid,
        false => Sound::Invalid,
    }
}

/// Faces whose ids the name table accepts as faces. A kernel ordinal is not one of those ids, so it is
/// left out. The role is the recipe stored for that id, not the face's place in the list.
fn faces_of(project: &Project, body: u64) -> Vec<Face> {
    project.regen_faces.get(&body).map(|found| found.iter().filter_map(|face| named_face(project, face)).collect()).unwrap_or_default()
}

fn named_face(project: &Project, face: &qymcad_core::geom::MeshFace) -> Option<Face> {
    let recipe = project.names.get(face.id)?;
    Some(Face { id: FaceId { id: face.id }, role: role_of(recipe), area: Area { mm2: face.area } })
}

fn role_of(recipe: qymcad_core::names::GeoName) -> FaceRole {
    use qymcad_core::names::Role;
    match recipe.role {
        Role::CapStart => FaceRole::CapStart,
        Role::CapEnd => FaceRole::CapEnd,
        Role::Wall => FaceRole::Wall { source: LineId { id: recipe.src } },
        _ => FaceRole::Other,
    }
}

/// Edges whose ids the name table accepts as edges. The length is the straight distance between the
/// two stored ends, which for a straight edge of this box is the edge itself.
fn edges_of(project: &Project, body: u64) -> Vec<Edge> {
    project.regen_edges.get(&body).map(|found| found.iter().filter_map(|edge| named_edge(project, edge)).collect()).unwrap_or_default()
}

fn named_edge(project: &Project, edge: &qymcad_core::geom::MeshEdge) -> Option<Edge> {
    project.names.edge(edge.id)?;
    Some(Edge { id: EdgeId { id: edge.id }, length: Length { mm: between(edge.a, edge.b) } })
}

fn between(from: [f64; 3], to: [f64; 3]) -> f64 {
    let span = |axis: usize| to[axis] - from[axis];
    (span(0).powi(2) + span(1).powi(2) + span(2).powi(2)).sqrt()
}

fn sketch_of(project: &Project, sketch: &qymcad_core::model::Sketch) -> Sketch {
    Sketch { id: sketch.id, name: sketch.name.clone(), plane: plane_of(sketch.plane), contours: sketch.contour_ids.iter().filter_map(|&id| contour_of(project, id)).collect() }
}

/// The contour record says whether the loop is closed. The sketch's own flag says whether its entity
/// chain was marked closed, which is a different fact.
fn contour_of(project: &Project, id: u64) -> Option<Contour> {
    let index = project.contour_index(id)?;
    let form = match project.contours.get(index)?.closed {
        true => Form::Closed,
        false => Form::Open,
    };
    Some(Contour { id, form })
}

fn plane_of(plane: qymcad_core::feature::SketchPlane) -> Plane {
    use qymcad_core::feature::{BasePlane, SketchPlane};
    match plane {
        SketchPlane::World(BasePlane::XY) => Plane::Xy,
        SketchPlane::World(BasePlane::XZ) => Plane::Xz,
        SketchPlane::World(BasePlane::YZ) => Plane::Yz,
        SketchPlane::Datum(id) => Plane::Datum { id },
        SketchPlane::Face(body, _) => Plane::Face { body },
    }
}

fn free_sketch_name(project: &Project) -> String {
    let taken: std::collections::HashSet<&str> = project.sketches.iter().map(|s| s.name.as_str()).collect();
    (1..)
        .find_map(|n| {
            let name = format!("name-sketch-n#{n}");
            (!taken.contains(name.as_str())).then_some(name)
        })
        .unwrap_or_else(|| "name-sketch-n#1".into())
}

/// A sketch keeps an existing point when the new one is at most this far away. A side of that length
/// collapses to one point, and the contour does not close. A coordinate that is not a finite length
/// is not a side either: it would be stored, and the solid would not be the rectangle that was asked for.
const POINT_MERGE_MM: f64 = 1e-6;

fn spans_a_rectangle(corners: &Corners) -> bool {
    let (x0, y0) = corners.from.at;
    let (x1, y1) = corners.to.at;
    [x0, y0, x1, y1].iter().all(|value| value.is_finite()) && (x1 - x0).abs() > POINT_MERGE_MM && (y1 - y0).abs() > POINT_MERGE_MM
}

/// A copy to keep, without the bytes of an embedded source and without the face and edge caches.
///
/// Those caches are rebuilt from the shape. Keeping them would show the body from after the change
/// once the older document is put back. The source record stays, with empty bytes, and the bytes
/// move back from the live document.
fn kept_step(project: &Project) -> Project {
    let mut step = project.clone_without_source_data();
    step.regen_faces.clear();
    step.regen_edges.clear();
    step
}

/// The four corners of the rectangle, not the frame of the sketch. Moving the origin or an axis
/// guide would move the plane the size is measured from.
fn rectangle_corners(sketch: &qymcad_core::model::Sketch) -> Option<Vec<qymcad_core::model::Id>> {
    let frame = sketch.system_ids();
    let mut ids = Vec::new();
    for entity in &sketch.entities {
        if entity.construction {
            continue;
        }
        let qymcad_core::model::EntityKind::Line { a, b } = entity.kind else {
            continue;
        };
        for id in [a, b] {
            if frame.contains(&id) || ids.contains(&id) {
                continue;
            }
            ids.push(id);
        }
    }
    (ids.len() == 4).then_some(ids)
}

/// Write `corners` onto the rectangle's own points, then rebuild its contour. A horizontal side keeps
/// one height and a vertical side keeps one width: two heights on one side are pulled back together
/// and the size is then not the one asked for. Returns false before any point moves when the sketch
/// is not that rectangle.
fn move_rectangle(project: &mut Project, profile: Profile, corners: &Corners) -> bool {
    let Some(index) = project.sketch_index(profile.sketch.id) else {
        return false;
    };
    let Some(ids) = rectangle_corners(&project.sketches[index]) else {
        return false;
    };
    let mut low_x = f64::INFINITY;
    let mut high_x = f64::NEG_INFINITY;
    let mut low_y = f64::INFINITY;
    let mut high_y = f64::NEG_INFINITY;
    for id in &ids {
        let Some(point) = project.sketches[index].points.iter().find(|point| point.id == *id) else {
            return false;
        };
        low_x = low_x.min(point.x);
        high_x = high_x.max(point.x);
        low_y = low_y.min(point.y);
        high_y = high_y.max(point.y);
    }
    if high_x - low_x <= POINT_MERGE_MM || high_y - low_y <= POINT_MERGE_MM {
        return false;
    }
    let (x0, y0) = corners.from.at;
    let (x1, y1) = corners.to.at;
    let left = x0.min(x1);
    let right = x0.max(x1);
    let bottom = y0.min(y1);
    let top = y0.max(y1);
    for id in ids {
        let Some(point) = project.sketches[index].points.iter_mut().find(|point| point.id == id) else {
            return false;
        };
        let on_left = (point.x - low_x).abs() <= (high_x - point.x).abs();
        let on_bottom = (point.y - low_y).abs() <= (high_y - point.y).abs();
        point.x = if on_left { left } else { right };
        point.y = if on_bottom { bottom } else { top };
    }
    project.regen_sketch(index);
    project.mark_sketch_dirty(profile.sketch.id);
    true
}

/// Why a parameter name was refused.
enum NameFault {
    /// Empty, a number, or more than one word.
    Shape,
    /// A constant the evaluator answers on its own, so a parameter of that name would never be read.
    Reserved,
}

/// A parameter name is one word the evaluator will look up. A number or a formula is a shape refusal.
/// A constant such as pi evaluates on its own, so storing it as a name would not be the name the next
/// formula sees.
fn parameter_name(text: &str) -> Result<(), NameFault> {
    let mut letters = text.chars();
    let Some(first) = letters.next() else {
        return Err(NameFault::Shape);
    };
    if !(first.is_alphabetic() || first == '_') || !letters.all(|c| c.is_alphanumeric() || c == '_') {
        return Err(NameFault::Shape);
    }
    let mut vars = std::collections::HashMap::new();
    // The evaluator matches a parameter by the spelling that was stored, and a constant only in its
    // own lower-case form. Binding the lower-case form and then reading the typed spelling misses, so
    // a usable name is refused as a constant.
    vars.insert(text.to_string(), 1.0);
    match qymcad_core::expr::eval(text, &vars) {
        Ok(v) if (v - 1.0).abs() <= 1e-9 => Ok(()),
        _ => Err(NameFault::Reserved),
    }
}

/// An extrusion height expression that does not evaluate to a positive length. A formula that fails
/// is included: the rebuild drops it and uses the number stored on the node, so the solid becomes
/// that older height and the rebuild stays quiet. A negative result is included too, because the
/// model takes the absolute value.
fn extrusion_height_refused(project: &Project) -> Option<DrawError> {
    let vars = project.param_map();
    let bad = project.timeline.iter().any(|node| {
        matches!(node.kind, qymcad_core::feature::FeatureKind::Extrude { .. })
            && project.feat_dim(node.id, "height").is_some_and(|formula| match qymcad_core::expr::eval(formula, &vars) {
                Ok(mm) => !(mm.is_finite() && mm > 0.0),
                Err(_) => true,
            })
    });
    if bad {
        Some(DrawError { text: "sketch-extrude-height".into() })
    } else {
        None
    }
}

/// The id is an extrusion node. A sketch id is a node too, and the model would store a height
/// expression on it without the extrusion ever reading that expression.
fn is_extrusion(project: &Project, id: u64) -> bool {
    project.timeline.iter().any(|n| n.id == id && matches!(n.kind, qymcad_core::feature::FeatureKind::Extrude { .. }))
}

/// The named contour has to be a closed contour of that sketch. Any other id is dropped by the model,
/// which then extrudes the sketch's first contour and reports a quiet rebuild.
fn closed_profile(project: &Project, profile: Profile) -> bool {
    let on_sketch = project.sketches.iter().any(|s| s.id == profile.sketch.id && s.contour_ids.contains(&profile.contour.id));
    on_sketch && contour_of(project, profile.contour.id).is_some_and(|c| c.form == Form::Closed)
}

/// The plane the rectangle is drawn on.
fn placed(plane: WorldPlane) -> qymcad_core::feature::SketchPlane {
    use qymcad_core::feature::{BasePlane, SketchPlane};
    match plane {
        WorldPlane::Xy => SketchPlane::World(BasePlane::XY),
        WorldPlane::Xz => SketchPlane::World(BasePlane::XZ),
        WorldPlane::Yz => SketchPlane::World(BasePlane::YZ),
    }
}

fn kind_of(kind: ComponentKind) -> Kind {
    match kind {
        ComponentKind::Part => Kind::Part,
        ComponentKind::Assembly => Kind::Assembly,
    }
}

/// A cancelled pass is not reported as a list of failures: the rebuild abandoned the document, and
/// the errors it did record belong to a pass that did not finish.
///
/// A pass that did finish can still leave a mark on the document without repeating it in the report.
/// A retryable failure does not by itself make the node dirty, so the next pass skips the node and the
/// report is empty while the mark remains. Quiet means neither place has a failure.
fn rebuild_of(project: &Project, report: &qymcad_core::feature::RegenReport) -> Rebuild {
    if report.cancelled {
        return Rebuild::Stopped;
    }
    let mut seen = std::collections::HashSet::new();
    let mut refusals = Vec::new();
    for (node, error) in &report.errors {
        if seen.insert(*node) {
            refusals.push(Refusal { node: *node, text: error.to_string() });
        }
    }
    for node in &project.timeline {
        if seen.contains(&node.id) {
            continue;
        }
        if let Some(error) = project.regen_errors.get(&node.id) {
            seen.insert(node.id);
            refusals.push(Refusal { node: node.id, text: error.to_string() });
        }
    }
    if refusals.is_empty() {
        Rebuild::Quiet
    } else {
        Rebuild::Refused { refusals }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qymcad_core::errors::CoreError;
    use qymcad_core::feature::{FeatureKind, FeatureNode, RegenReport};

    #[test]
    fn a_refusal_uses_the_core_words_once() {
        let mut project = Project::default();
        project.timeline.push(node(4));
        project.regen_errors.insert(4, CoreError::MirrorOfHollowBody);
        let mut report = RegenReport::default();
        report.errors.push((4, CoreError::MirrorOfHollowBody));
        assert_eq!(rebuild_of(&project, &report), Rebuild::Refused { refusals: vec![Refusal { node: 4, text: "mirror of a hollow body is not supported".into() }] });
    }

    #[test]
    fn a_cancelled_pass_is_not_read_as_a_refusal() {
        let project = Project::default();
        let report = RegenReport { cancelled: true, errors: vec![(4, CoreError::MirrorOfHollowBody)], ..RegenReport::default() };
        assert_eq!(rebuild_of(&project, &report), Rebuild::Stopped);
    }

    #[test]
    fn a_mark_the_pass_did_not_repeat_is_still_a_refusal() {
        let mut project = Project::default();
        project.timeline.push(node(9));
        project.regen_errors.insert(9, CoreError::SourceBodyNotBuilt);
        assert_eq!(rebuild_of(&project, &RegenReport::default()), Rebuild::Refused { refusals: vec![Refusal { node: 9, text: "error-source-body-not-built".into() }] });
    }

    fn node(id: u64) -> FeatureNode {
        FeatureNode { id, name: "n".into(), kind: FeatureKind::Sketch { sketch: 1 }, parent: None, dirty: false, suppressed: true }
    }

    /// The corners are the profile a later extrusion measures. A closed contour of any size would still
    /// satisfy the tree, so the points themselves have to be the ones that were asked for, and they have to
    /// still be there after a rebuild.
    #[test]
    fn a_rectangle_keeps_the_corners_it_was_given() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let _doc = session.document();
        let index = session.project.contour_index(drawn.contour.id).expect("the contour is in the document");
        let contour = &session.project.contours[index];
        let (mut min_x, mut max_x) = (f64::MAX, f64::MIN);
        let (mut min_y, mut max_y) = (f64::MAX, f64::MIN);
        for point in &contour.points {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }
        assert!((min_x - 0.0).abs() < 1e-6 && (min_y - 0.0).abs() < 1e-6, "origin moved to ({min_x}, {min_y})");
        assert!((max_x - 10.0).abs() < 1e-6 && (max_y - 20.0).abs() < 1e-6, "far corner moved to ({max_x}, {max_y})");
    }

    /// The volume of one contour cannot tell a stored contour id from zero, and a symmetric extrusion of
    /// the same height has the same volume. The node itself has to name the contour, the height, a
    /// forward reach, and no second side.
    #[test]
    fn an_extrusion_stores_the_contour_and_a_forward_height() {
        use qymcad_core::feature::Reach;
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let node = session.project.timeline.iter().find(|n| n.id == made.id).expect("the extrude node");
        match node.kind {
            FeatureKind::Extrude { sketch, ref profiles, height, reach, down, .. } => {
                assert_eq!(sketch, drawn.sketch.id, "the extrusion uses the sketch");
                assert_eq!(profiles.as_slice(), &[drawn.contour.id], "the extrusion uses the contour, not the first-contour sentinel");
                assert!((height - 30.0).abs() < 1e-9, "the stored height is 30 mm");
                assert_eq!(reach, Reach::Forward, "the body grows forward from the plane");
                assert_eq!(down, 0.0, "there is no second side");
            }
            _ => panic!("the node is an extrusion"),
        }
    }

    /// A side shorter than the point-merge distance is not a rectangle. The document stays empty, so the
    /// next real rectangle is still the first sketch, and the answer is a refusal rather than contour 0.
    #[test]
    fn a_rectangle_with_no_area_is_refused_and_the_document_stays_empty() {
        let mut session = Session::empty();
        for corners in [
            Corners { from: From { at: (1.0, 2.0) }, to: To { at: (1.0, 2.0) } },
            Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 0.0) } },
            Corners { from: From { at: (0.0, 0.0) }, to: To { at: (1e-7, 20.0) } },
            Corners { from: From { at: (0.0, 0.0) }, to: To { at: (1e-6, 20.0) } },
        ] {
            let err = session.rectangle(WorldPlane::Xz, corners).expect_err("a rectangle with no area");
            assert_eq!(err.text, "sketch-rectangle-empty");
        }
        let doc = session.document();
        assert_eq!(doc.components.len(), 1, "a refused rectangle does not add a part");
        assert!(doc.sketches.is_empty(), "a refused rectangle does not add a sketch");
        assert!(doc.nodes.is_empty(), "a refused rectangle does not add a timeline node");
        assert_eq!(doc.rebuild, Rebuild::Quiet);
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let doc = session.document();
        assert_eq!(doc.sketches[0].name, "name-sketch-n#1", "the refused call did not take the first sketch name");
        assert_eq!(doc.sketches[0].contours[0].id, drawn.contour.id);
    }

    /// The rebuild uses the height expression and leaves the stored number alone. A refused replacement
    /// keeps the previous expression text: a failed evaluation would otherwise leave the old number in
    /// place and the volume would not show the change. A second spelling of the same name is still one
    /// parameter. The sketch id gets no height expression.
    #[test]
    fn a_live_expression_leaves_the_stored_height_and_one_parameter() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        session.parameter(ParameterName { text: "span".into() }, Expression { text: "40".into() }).expect("a parameter");
        session.bind(made, Dimension::Height, Expression { text: "span/2".into() }).expect("the height expression");
        session.parameter(ParameterName { text: "span".into() }, Expression { text: "80".into() }).expect("the same parameter");
        let at_eighty = session.document();
        let live = match at_eighty.bodies[0].measure {
            Measure::Live { volume, .. } => volume.mm3,
            Measure::Absent => panic!("the body has no live shape"),
        };
        assert!((live - 8000.0).abs() < 1.0, "volume {live} mm^3 before the stored number is edited");

        {
            let node = session.project.timeline.iter_mut().find(|n| n.id == made.id).expect("the extrude node");
            match &mut node.kind {
                FeatureKind::Extrude { height, .. } => *height = 99.0,
                _ => panic!("the node is an extrusion"),
            }
        }
        session.project.mark_node_dirty(made.id);
        let kept = session.document();
        let after = match kept.bodies[0].measure {
            Measure::Live { volume, .. } => volume.mm3,
            Measure::Absent => panic!("the body has no live shape"),
        };
        assert!((after - 8000.0).abs() < 1.0, "the stored 99 mm changed the volume to {after}");
        let node = session.project.timeline.iter().find(|n| n.id == made.id).expect("the extrude node");
        match node.kind {
            FeatureKind::Extrude { height, .. } => assert!((height - 99.0).abs() < 1e-9, "the rebuild wrote {height} over the stored height"),
            _ => panic!("the node is an extrusion"),
        }

        let refused = session.parameter(ParameterName { text: "span".into() }, Expression { text: "nope".into() }).expect_err("the name is not in the document");
        assert_eq!(refused.text, "error-expr-unknown-name");
        assert_eq!(session.project.parameters.len(), 1, "a refused replacement does not add a parameter");
        assert_eq!(session.project.parameters[0].expr, "80", "a refused replacement keeps the previous expression");

        session.parameter(ParameterName { text: "Span".into() }, Expression { text: "40".into() }).expect("the same name in another case");
        assert_eq!(session.project.parameters.len(), 1, "letter case is the same parameter");
        assert_eq!(session.project.parameters[0].expr, "40");

        for name in ["", "1", "span/2", "1*1"] {
            let err = session.parameter(ParameterName { text: name.into() }, Expression { text: "10".into() }).expect_err("the name is not one word");
            assert_eq!(err.text, "par-name-bad", "{name}");
        }
        assert_eq!(session.project.parameters.len(), 1, "a refused name is not stored");
        let unknown = session.parameter(ParameterName { text: "other".into() }, Expression { text: "missing".into() }).expect_err("the expression does not evaluate");
        assert_eq!(unknown.text, "error-expr-unknown-name");
        assert_eq!(session.project.parameters.len(), 1, "an expression that does not evaluate is not stored");
        assert_eq!(session.project.parameters[0].expr, "40");

        let sketch = session.bind(BodyId { id: drawn.sketch.id }, Dimension::Height, Expression { text: "span/2".into() }).expect_err("a sketch is not an extrusion");
        assert_eq!(sketch.text, "sketch-extrude-target");
        assert!(session.project.feat_dim(drawn.sketch.id, "height").is_none(), "the sketch has no height expression");
        assert_eq!(session.project.feat_dim(made.id, "height"), Some("span/2"));
    }

    /// A negative height expression is not stored. A formula that mentions its own parameter, and a
    /// pair of parameters that chase each other, are not stored either: the number written afterwards
    /// would not be the number that was checked. A constant is not a parameter name.
    #[test]
    fn a_refused_formula_leaves_the_previous_text() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let negative = session.bind(made, Dimension::Height, Expression { text: "-15".into() }).expect_err("the height is not positive");
        assert_eq!(negative.text, "sketch-extrude-height");
        assert!(session.project.feat_dim(made.id, "height").is_none(), "a refused height expression is not stored");

        session.parameter(ParameterName { text: "span".into() }, Expression { text: "40".into() }).expect("a parameter");
        session.bind(made, Dimension::Height, Expression { text: "span/2".into() }).expect("the height expression");
        let negative = session.parameter(ParameterName { text: "span".into() }, Expression { text: "-80".into() }).expect_err("the height would not be positive");
        assert_eq!(negative.text, "sketch-extrude-height");
        assert_eq!(session.project.parameters[0].expr, "40", "a refused parameter keeps the previous expression");
        session.parameter(ParameterName { text: "half".into() }, Expression { text: "span/2".into() }).expect("a parameter");
        session.bind(made, Dimension::Height, Expression { text: "half".into() }).expect("the height expression");
        let through = session.parameter(ParameterName { text: "span".into() }, Expression { text: "-80".into() }).expect_err("the height would not be positive");
        assert_eq!(through.text, "sketch-extrude-height");
        assert_eq!(session.project.parameters.iter().find(|p| p.name == "span").expect("span").expr, "40");
        {
            let half = session.project.parameters.iter().find(|p| p.name == "half").expect("half");
            assert_eq!(half.expr, "span/2");
            assert!((half.value - 20.0).abs() < 1e-9, "the other parameter was left at {}", half.value);
        }

        let itself = session.parameter(ParameterName { text: "span".into() }, Expression { text: "span*2".into() }).expect_err("the formula mentions itself");
        assert_eq!(itself.text, "error-expr-unknown-name");
        assert_eq!(session.project.parameters.iter().find(|p| p.name == "span").expect("span").expr, "40");

        for name in ["pi", "tau", "e"] {
            let reserved = session.parameter(ParameterName { text: name.into() }, Expression { text: "1".into() }).expect_err("a constant is not a parameter name");
            assert_eq!(reserved.text, "par-name-reserved", "{name}");
        }
        assert_eq!(session.project.parameters.len(), 2, "a reserved name is not stored");

        session.parameter(ParameterName { text: "b".into() }, Expression { text: "span".into() }).expect("a parameter");
        let chased = session.parameter(ParameterName { text: "span".into() }, Expression { text: "b+1".into() }).expect_err("the value does not settle");
        assert_eq!(chased.text, "sketch-param-settle");
        assert_eq!(session.project.parameters.iter().find(|p| p.name == "span").expect("span").expr, "40");
        assert_eq!(session.project.parameters.iter().find(|p| p.name == "b").expect("b").expr, "span");
    }

    /// Dividing by a parameter that has become zero is not stored. The height formula would fail, and
    /// the rebuild would use the number on the node instead.
    #[test]
    fn a_parameter_that_breaks_the_height_formula_is_not_stored() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        session.parameter(ParameterName { text: "other".into() }, Expression { text: "2".into() }).expect("a parameter");
        session.bind(made, Dimension::Height, Expression { text: "100/other".into() }).expect("the height expression");
        let broken = session.parameter(ParameterName { text: "other".into() }, Expression { text: "0".into() }).expect_err("the height would not be a number");
        assert_eq!(broken.text, "sketch-extrude-height");
        assert_eq!(session.project.parameters[0].expr, "2", "the parameter that broke the height is not stored");
        assert_eq!(session.project.feat_dim(made.id, "height"), Some("100/other"));
    }

    /// A corner that is not a finite length is refused before a step is kept. A 10 by 10 mm check does
    /// not prove the other axis moved, because that side was already 10 mm.
    #[test]
    fn a_corner_that_is_not_a_length_is_refused_and_both_axes_move() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let _ = session.document();
        let steps_before = session.steps.len();
        for corners in [Corners { from: From { at: (0.0, 0.0) }, to: To { at: (f64::INFINITY, 10.0) } }, Corners { from: From { at: (0.0, f64::NAN) }, to: To { at: (10.0, 20.0) } }] {
            let err = session.resize(drawn, corners).expect_err("the corner is not a length");
            assert_eq!(err.text, "sketch-rectangle-empty");
        }
        assert_eq!(session.steps.len(), steps_before, "a refused corner kept a step");
        session.resize(drawn, Corners { from: From { at: (5.0, 7.0) }, to: To { at: (35.0, 17.0) } }).expect("the rectangle moves");
        let doc = session.document();
        let live = match doc.bodies[0].measure {
            Measure::Live { volume, .. } => volume.mm3,
            Measure::Absent => panic!("the body has no live shape"),
        };
        assert!((live - 9000.0).abs() < 1.0, "volume {live} mm^3 after both sides moved");
        assert_eq!(doc.bodies[0].id, made.id);
        let sketch = session.project.sketches.iter().find(|sketch| sketch.id == drawn.sketch.id).expect("the sketch");
        let frame = sketch.system_ids();
        let mut low_x = f64::INFINITY;
        let mut high_x = f64::NEG_INFINITY;
        let mut low_y = f64::INFINITY;
        let mut high_y = f64::NEG_INFINITY;
        for point in sketch.points.iter().filter(|point| !frame.contains(&point.id)) {
            low_x = low_x.min(point.x);
            high_x = high_x.max(point.x);
            low_y = low_y.min(point.y);
            high_y = high_y.max(point.y);
        }
        assert!((low_x - 5.0).abs() < 1e-6 && (high_x - 35.0).abs() < 1e-6, "x ran from {low_x} to {high_x}");
        assert!((low_y - 7.0).abs() < 1e-6 && (high_y - 17.0).abs() < 1e-6, "y ran from {low_y} to {high_y}");
    }

    /// A 10 by 10 mm resize is not a 5 by 20 mm rectangle at the same height: both are 2000 mm³. The
    /// four corners run from 0 to 10 in both directions. The centre of the rectangle is the middle of
    /// that span, not a fifth corner, and the extrusion still names that contour.
    #[test]
    fn a_resized_rectangle_is_ten_millimetres_on_both_sides() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        session.parameter(ParameterName { text: "span".into() }, Expression { text: "40".into() }).expect("a parameter");
        session.bind(made, Dimension::Height, Expression { text: "span/2".into() }).expect("the height expression");
        let _ = session.document();
        session.resize(drawn, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 10.0) } }).expect("the rectangle moves");
        let _ = session.document();
        let sketch = session.project.sketches.iter().find(|sketch| sketch.id == drawn.sketch.id).expect("the sketch");
        assert_eq!(sketch.rects.len(), 1, "the rectangle stays one shape");
        let rect = &sketch.rects[0];
        let corners: Vec<_> = rect.corners.iter().filter_map(|id| sketch.points.iter().find(|point| point.id == *id)).collect();
        assert_eq!(corners.len(), 4, "the rectangle still has four corners");
        let mut low_x = f64::INFINITY;
        let mut high_x = f64::NEG_INFINITY;
        let mut low_y = f64::INFINITY;
        let mut high_y = f64::NEG_INFINITY;
        for point in corners {
            low_x = low_x.min(point.x);
            high_x = high_x.max(point.x);
            low_y = low_y.min(point.y);
            high_y = high_y.max(point.y);
        }
        assert!((low_x - 0.0).abs() < 1e-6 && (low_y - 0.0).abs() < 1e-6, "a corner moved to ({low_x}, {low_y})");
        assert!((high_x - 10.0).abs() < 1e-6 && (high_y - 10.0).abs() < 1e-6, "the far corner is ({high_x}, {high_y})");
        let centre = sketch.points.iter().find(|point| point.id == rect.centre).expect("the centre");
        assert!((centre.x - 5.0).abs() < 1e-6 && (centre.y - 5.0).abs() < 1e-6, "the centre stayed at ({}, {})", centre.x, centre.y);
        assert!(sketch.contour_ids.contains(&drawn.contour.id), "the extrusion's contour is still in the sketch");
        let node = session.project.timeline.iter().find(|node| node.id == made.id).expect("the extrude node");
        match &node.kind {
            FeatureKind::Extrude { profiles, .. } => assert_eq!(profiles.as_slice(), &[drawn.contour.id]),
            _ => panic!("the node is an extrusion"),
        }
    }

    /// A kept step records the source and does not copy its bytes. Putting the step back moves those
    /// bytes onto the live document before the next read.
    #[test]
    fn a_step_records_the_source_and_puts_its_bytes_back() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let _made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let bytes = vec![7u8; 64];
        let source = session.project.alloc_id();
        session.project.sources.push(qymcad_core::model::SourceFile { id: source, name: "plant.step".into(), ext: "step".into(), data: bytes.clone() });
        session.parameter(ParameterName { text: "span".into() }, Expression { text: "40".into() }).expect("a parameter");
        let step = session.steps.last().expect("the parameter kept a step");
        assert_eq!(step.sources.len(), 1, "the source record stays on the step");
        assert_eq!(step.sources[0].id, source);
        assert_eq!(step.sources[0].name, "plant.step");
        assert_eq!(step.sources[0].ext, "step");
        assert!(step.sources[0].data.is_empty(), "the step does not copy the source bytes");
        assert_eq!(session.project.sources[0].data, bytes, "the bytes stay on the live document");
        session.back().expect("a step back");
        assert_eq!(session.project.sources[0].data, bytes, "the bytes are back before the next read");
        assert_eq!(session.project.sources[0].id, source);
        assert_eq!(session.project.sources[0].name, "plant.step");
    }

    /// An empty document must not erase a file that already holds a sketch, and opening that file must
    /// show the sketch. A fresh empty document looks the same as a roundtrip of an empty one, so this
    /// is what proves the file was read.
    #[test]
    fn an_empty_save_keeps_a_planted_sketch_and_open_reads_it() {
        let path = std::env::temp_dir().join(format!("qymcad-drive-sketch-{}.qcad", std::process::id()));
        let _sweep = Planted(&path);
        let text = path.to_str().expect("the planted path is Unicode");
        let mut project = Project::default();
        project.new_empty_document();
        project.add_sketch("planted", Vec::new(), None);
        qymcad_io::save_project(&project, text).expect("the sketch file is planted");

        let refused = Session::empty().save(&path).expect_err("an empty document does not replace a sketch");
        assert!(refused.text.contains("io-refuse-empty-over-full"), "the refusal was {refused:?}");

        let mut opened = Session::open(&path).expect("the planted file is still there");
        let doc = opened.document();
        assert!(doc.sketches.iter().any(|s| s.name == "planted"), "open read the planted sketch, saw {doc:?}");
        assert_eq!(doc.rebuild, Rebuild::Quiet);
    }

    /// The planted bundle and the sidecars beside it, removed when the check ends.
    struct Planted<'a>(&'a std::path::Path);

    impl Drop for Planted<'_> {
        fn drop(&mut self) {
            let text = self.0.display().to_string();
            let _ = std::fs::remove_file(self.0);
            let _ = std::fs::remove_file(format!("{text}.bak"));
            let _ = std::fs::remove_file(format!("{text}.tmp~"));
        }
    }

    /// The face and edge ids are name-table descriptors. A positional number is not one, and the long
    /// wall's line is the same line after the rectangle is resized.
    #[test]
    fn a_face_name_is_the_recipe_not_an_ordinal() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let first = session.document();
        let recorded = long_side(&session, &first, made.id, drawn.sketch.id);
        session.resize(drawn, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 10.0) } }).expect("the rectangle moves");
        let shrunk = session.document();
        names_are_descriptors(&session, &shrunk);
        let wall = shrunk.bodies[0].faces.iter().find(|face| face.id == recorded.wall).expect("the wall name is gone");
        assert_eq!(wall.role, FaceRole::Wall { source: recorded.source }, "the wall's line changed");
        assert!((wall.area.mm2 - 300.0).abs() < 1.0, "the recorded wall stayed the short side");
        let edge = shrunk.bodies[0].edges.iter().find(|edge| edge.id == recorded.edge).expect("the edge name is gone");
        assert!((edge.length.mm - 10.0).abs() < 0.1, "the recorded edge stayed the height");
        assert_eq!(edge_faces(&session, recorded.edge), face_pair(recorded.cap.id, recorded.wall.id), "the edge no longer lies between that cap and that wall");
    }

    struct LongSide {
        cap: FaceId,
        wall: FaceId,
        source: LineId,
        edge: EdgeId,
    }

    fn long_side(session: &Session, doc: &Document, body: u64, sketch: u64) -> LongSide {
        names_are_descriptors(session, doc);
        assert!(session.project.names.get(1).is_none(), "a positional number was accepted as a face");
        assert!(session.project.names.edge(1).is_none(), "a positional number was accepted as an edge");
        let cap = doc.bodies[0].faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the end cap");
        let recipe = session.project.names.get(cap.id.id).expect("the end cap is not a face name");
        assert_eq!(recipe.feature, body, "the end cap belongs to another feature");
        assert_eq!(recipe.role, qymcad_core::names::Role::CapEnd, "the end cap's recipe is not the end cap");
        let wall = doc.bodies[0].faces.iter().find(|face| matches!(face.role, FaceRole::Wall { .. }) && (face.area.mm2 - 600.0).abs() < 1.0).expect("the long wall");
        let FaceRole::Wall { source } = wall.role else { unreachable!("the long wall was selected by its role") };
        let sketch = session.project.sketches.iter().find(|item| item.id == sketch).expect("the sketch");
        let line = sketch.entities.iter().any(|entity| entity.id == source.id && matches!(entity.kind, qymcad_core::model::EntityKind::Line { .. }) && !entity.construction);
        assert!(line, "the wall's source is not a line of the sketch");
        let edge = doc.bodies[0]
            .edges
            .iter()
            .find(|edge| (edge.length.mm - 20.0).abs() < 0.1 && edge_faces(session, edge.id) == face_pair(cap.id.id, wall.id.id))
            .expect("no 20 mm edge lies between the end cap and the long wall");
        LongSide { cap: cap.id, wall: wall.id, source, edge: edge.id }
    }

    fn names_are_descriptors(session: &Session, doc: &Document) {
        for face in &doc.bodies[0].faces {
            assert!(session.project.names.get(face.id.id).is_some(), "face {} is not a face name", face.id.id);
            assert!(session.project.names.edge(face.id.id).is_none(), "face {} is an edge name", face.id.id);
        }
        for edge in &doc.bodies[0].edges {
            assert!(session.project.names.edge(edge.id.id).is_some(), "edge {} is not an edge name", edge.id.id);
            assert!(session.project.names.get(edge.id.id).is_none(), "edge {} is a face name", edge.id.id);
        }
    }

    fn edge_faces(session: &Session, edge: EdgeId) -> [u32; 2] {
        session.project.names.edge(edge.id).expect("the edge is not an edge name").faces
    }

    fn face_pair(one: u32, other: u32) -> [u32; 2] {
        [one.min(other), one.max(other)]
    }
}
