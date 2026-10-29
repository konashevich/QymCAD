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
    /// Mates, in the order the document holds them.
    pub mates: Vec<Fastened>,
    /// Mates the last solve could not hold, the rigid mate first. Empty when the solve held them.
    ///
    /// Not stored in the file. The next read computes the list again.
    pub violated: Vec<MateId>,
    /// What the rebuild reported.
    pub rebuild: Rebuild,
}

/// Ways a component can still move.
///
/// A grounded component is 0. One that does not reach a grounded component is 6: two parts fastened
/// only to each other can still move as a whole. After the rigid origin mate of the 10 by 20 by 30 mm
/// boxes, the root and both parts read 6.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Freedom {
    pub count: u8,
}

/// A part or an assembly.
#[derive(Clone, Debug, PartialEq)]
pub struct Component {
    pub id: u64,
    pub name: String,
    pub kind: Kind,
    /// The assembly this one stands in. `None` for the root.
    pub parent: Option<u64>,
    /// The 3x4 placement of this component in its parent. Index 3 is the shift along X, in millimetres.
    pub place: Place,
    /// Ways this component can still move, taken from the model at this read.
    pub freedom: Freedom,
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

/// How far a fillet rounds an edge, in mm. It is not a height.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Radius {
    pub mm: f64,
}

/// How far a chamfer cuts back from an edge, in mm. It is not a height or a radius.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Distance {
    pub mm: f64,
}

/// How thick the wall of a shell is, in mm. It is not a height, a radius, or a distance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Thickness {
    pub mm: f64,
}

/// How wide a hole is, in mm. It is not a height, a radius, a distance, or a depth.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Diameter {
    pub mm: f64,
}

/// How deep a hole is drilled, in mm. It is not a height, a radius, a distance, or a diameter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Depth {
    pub mm: f64,
}

/// How long a box is along X, in mm. It is not the length of an edge, a width, or a height.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxLength {
    pub mm: f64,
}

/// How wide a box is along Y, in mm. It is not a length or a height.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Width {
    pub mm: f64,
}

/// The three sizes of a box. The length, the width, and the height are different types, so they cannot be swapped.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cuboid {
    pub length: BoxLength,
    pub width: Width,
    pub height: Height,
}

/// The radius and the height of a cylinder. The two lengths are different types, so they cannot be swapped.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cylinder {
    pub radius: Radius,
    pub height: Height,
}

/// The bottom radius of a cone, in mm. It is not the top radius or the height. Zero is a point, and a cone
/// needs a base.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BottomRadius {
    pub mm: f64,
}

/// The top radius of a cone, in mm. It is not the bottom radius or the height. Zero is a sharp cone.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TopRadius {
    pub mm: f64,
}

/// The three sizes of a cone.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cone {
    pub bottom: BottomRadius,
    pub top: TopRadius,
    pub height: Height,
}

/// The ring radius of a torus, in mm. It is not the tube radius.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ring {
    pub mm: f64,
}

/// The tube radius of a torus, in mm. It is not the ring radius.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tube {
    pub mm: f64,
}

/// The ring and the tube of a torus.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Torus {
    pub ring: Ring,
    pub tube: Tube,
}

/// How many sides a prism has. It is not a radius or a height.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sides {
    pub count: u32,
}

/// The circumradius, the height, and the number of sides of a prism.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Prism {
    pub radius: Radius,
    pub height: Height,
    pub sides: Sides,
}

/// Which axis of the sketch a profile turns about.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SketchAxis {
    /// The sketch X axis, stored as 0.
    X,
    /// The sketch Y axis, stored as 1.
    Y,
}

/// How far a profile turns, in degrees. It is not a length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Angle {
    pub deg: f64,
}

/// The two ends of a line, in the plane of its sketch. The ends are the rectangle's corner types, so they
/// cannot be swapped.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub from: From,
    pub to: To,
}

/// How far a datum plane stands off a world plane, in mm. Zero lies on that plane. A negative distance
/// stands on the other side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Offset {
    pub mm: f64,
}

/// A datum plane the document already holds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlaneId {
    pub id: u64,
}

/// An open or closed contour used as a sweep path. It is not a profile: a profile is a closed section.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Path {
    pub sketch: SketchId,
    pub contour: ContourId,
}

/// The section and the path of a sweep. The two are different types, so the path cannot be passed as the section.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sweep {
    pub section: Profile,
    pub path: Path,
}

/// The lower section of a loft. It is not the upper section.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lower {
    pub profile: Profile,
}

/// The upper section of a loft. It is not the lower section.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Upper {
    pub profile: Profile,
}

/// The two sections of a loft, lower then upper.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Loft {
    pub lower: Lower,
    pub upper: Upper,
}

/// The id of a body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyId {
    pub id: u64,
}

/// Which boolean is applied to the base and the tool.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoolOp {
    /// The tool is removed from the base. Stored as 0.
    Cut,
    /// The tool is united with the base. Stored as 1.
    Union,
    /// What the two solids share. Stored as 2.
    Intersect,
}

/// The base of a boolean. It is not the tool.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Base {
    pub body: BodyId,
}

/// The tool of a boolean. It is not the base.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tool {
    pub body: BodyId,
}

/// The base, the kind, and the tool of one boolean. The base and the tool are different types.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Boolean {
    pub base: Base,
    pub op: BoolOp,
    pub tool: Tool,
}

/// One component of a pattern step, along X. It is not the Y or the Z component.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlongX {
    pub mm: f64,
}

/// One component of a pattern step, along Y. It is not the X or the Z component.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlongY {
    pub mm: f64,
}

/// One component of a pattern step, along Z. It is not the X or the Y component.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlongZ {
    pub mm: f64,
}

/// The step of a linear pattern. The three components are different types, so they cannot be swapped.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Step {
    pub x: AlongX,
    pub y: AlongY,
    pub z: AlongZ,
}

/// How many copies a pattern makes, the original included.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Copies {
    pub n: u32,
}

/// A body mirrored through a world plane. The result keeps the original and the image.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mirror {
    pub body: BodyId,
    pub plane: WorldPlane,
}

/// A linear pattern of one body. The step and the count are different types.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Linear {
    pub body: BodyId,
    pub step: Step,
    pub copies: Copies,
}

/// A circular pattern of one body about world Z. The count and the angle are different types.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Circular {
    pub body: BodyId,
    pub copies: Copies,
    pub turn: Angle,
}

/// A body split by a world plane. The offset is a distance along the plane normal, in mm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Split {
    pub body: BodyId,
    pub plane: WorldPlane,
    pub offset: Offset,
}

/// The wall a draft tilts. It is not the neutral face.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tilted {
    pub face: FaceId,
}

/// The face a draft tilts about. It is not the wall.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Neutral {
    pub face: FaceId,
}

/// One wall of a body, tilted about a neutral face by an angle in degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Draft {
    pub body: BodyId,
    pub tilted: Tilted,
    pub neutral: Neutral,
    pub angle: Angle,
}

/// A distance along a face's own normal, in millimetres. A positive value adds material.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shift {
    pub mm: f64,
}

/// One face of a body, moved along its own normal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Push {
    pub body: BodyId,
    pub face: FaceId,
    pub by: Shift,
}

/// How thick a plate grown on a face is, in millimetres. It is not the wall of a shell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plate {
    pub mm: f64,
}

/// A plate grown on one face of a body and welded to that body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Thicken {
    pub body: BodyId,
    pub face: FaceId,
    pub plate: Plate,
}

/// One face taken off a body. The neighbours are extended and the body stays closed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RemoveFace {
    pub body: BodyId,
    pub face: FaceId,
}

/// One face of a solid, copied out as a sheet. The solid stays in the part.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CopyFace {
    pub body: BodyId,
    pub face: FaceId,
}

/// How far a face moves along its own normal, in millimetres. A positive distance moves a solid's
/// face outward. A negative distance moves it inward. Zero leaves the face where it is. It is not
/// the distance of a datum plane.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outset {
    pub mm: f64,
}

/// One face of a solid, moved along its normal into a sheet. The solid stays in the part.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OffsetFace {
    pub body: BodyId,
    pub face: FaceId,
    pub by: Outset,
}

/// How a patch meets the edge it spans. By position the sheet ends on the edge. Smooth, it leaves
/// that edge tangent to the neighbouring face.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Meet {
    ByPosition,
    Smooth,
}

/// One closed edge of a solid, spanned into a sheet. The solid stays in the part.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Patch {
    pub body: BodyId,
    pub edge: EdgeId,
    pub meet: Meet,
}

/// How far two edges may stand and still count as one, in millimetres. It is not a face offset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gap {
    pub mm: f64,
}

/// Sheets sewn into one body. A set that closes becomes a solid. The sheets are consumed.
#[derive(Clone, Debug, PartialEq)]
pub struct Stitch {
    pub sheets: Vec<BodyId>,
    pub gap: Gap,
}

/// The x coordinate of the point that picks which piece of a trim stays, in millimetres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeepX {
    pub mm: f64,
}

/// The y coordinate of the point that picks which piece of a trim stays, in millimetres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeepY {
    pub mm: f64,
}

/// The z coordinate of the point that picks which piece of a trim stays, in millimetres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeepZ {
    pub mm: f64,
}

/// The point, in millimetres, nearest the piece of a trimmed sheet that stays.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Keep {
    pub x: KeepX,
    pub y: KeepY,
    pub z: KeepZ,
}

/// A sheet cut by another body. The piece nearest `keep` stays. The sheet is consumed. The tool is not.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trim {
    pub sheet: BodyId,
    pub keep: Keep,
    pub tool: BodyId,
}

/// One face of a solid, replaced by a sheet. The face stands between the two bodies. Both are consumed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReplaceFace {
    pub body: BodyId,
    pub face: FaceId,
    pub surface: BodyId,
}

/// A 3x4 placement, row-major, in millimetres. Index 3 is the shift along X.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Place {
    pub mat: [f64; 12],
}

/// A body moved by a placement. The body is consumed. The result is not.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Move {
    pub body: BodyId,
    pub by: Place,
}

/// The id of a part. A body id is not a part id.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PartId {
    pub id: u64,
}

/// A new part in the root assembly, at `place`. The placement does not move a body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Part {
    pub place: Place,
}

/// The part a mate leaves where it stands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stay {
    pub part: PartId,
}

/// The part a mate brings to the one that stays.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bring {
    pub part: PartId,
}

/// A rigid mate of the origins of two parts. The brought part moves when the document is read.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mate {
    pub stay: Stay,
    pub bring: Bring,
}

/// A distance along the slider, in millimetres. Positive is along Z.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Travel {
    pub mm: f64,
}

/// A slider between the origins of two parts, with its travel set.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slide {
    pub stay: Stay,
    pub bring: Bring,
    pub by: Travel,
}

/// The id of a mate. A part id is not a mate id.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MateId {
    pub id: u64,
}

/// A mate in the read. The part that stays is named first.
#[derive(Clone, Debug, PartialEq)]
pub struct Fastened {
    pub id: u64,
    pub name: String,
    pub stay: Stay,
    pub bring: Bring,
}

/// The nominal diameter of a metric thread, in mm. It is not a hole diameter, a pitch, or a length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nominal {
    pub mm: f64,
}

/// The pitch of a metric thread, in mm. Zero is the coarse pitch of the diameter. It is not the nominal
/// diameter or the length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pitch {
    pub mm: f64,
}

/// How far a metric thread runs along the rod, in mm. It is not the straight length of an edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThreadLength {
    pub mm: f64,
}

/// A metric thread cut on the circular rim of a body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Thread {
    pub body: BodyId,
    pub rim: EdgeId,
    pub diameter: Nominal,
    pub pitch: Pitch,
    pub length: ThreadLength,
}

/// The outer diameter of an auger flight, in mm. It is not a thread's nominal diameter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outer {
    pub mm: f64,
}

/// The pitch of an auger flight, in mm. Zero is not a pitch. It is not the pitch of a metric thread.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AugerPitch {
    pub mm: f64,
}

/// How thick the welded flight is, in mm. It is not the wall of a shell and not a plate grown on a face.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flight {
    pub mm: f64,
}

/// How far an auger flight runs along the shaft, in mm. It is not a thread length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AugerLength {
    pub mm: f64,
}

/// One helical flight welded onto the circular rim of a shaft.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Auger {
    pub body: BodyId,
    pub rim: EdgeId,
    pub outer: Outer,
    pub pitch: AugerPitch,
    pub thickness: Flight,
    pub length: AugerLength,
}

/// The pieces of a split, from the negative side of the normal toward the positive side.
///
/// The first id is the timeline node.
#[derive(Clone, Debug, PartialEq)]
pub struct Pieces {
    pub bodies: Vec<BodyId>,
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

/// The extent of a corner along X, in mm. It is its own type so it cannot stand where Y or Z goes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CornerX {
    pub mm: f64,
}

/// The extent of a corner along Y, in mm. It is its own type so it cannot stand where X or Z goes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CornerY {
    pub mm: f64,
}

/// The extent of a corner along Z, in mm. It is its own type so it cannot stand where X or Y goes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CornerZ {
    pub mm: f64,
}

/// The low corner of a solid, in the frame of the file that held it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LowCorner {
    pub x: CornerX,
    pub y: CornerY,
    pub z: CornerZ,
}

/// The high corner of a solid, in the frame of the file that held it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HighCorner {
    pub x: CornerX,
    pub y: CornerY,
    pub z: CornerZ,
}

/// One solid read from a STEP file. It is not a body of the document.
///
/// `low` and `high` are the bounding box in the file's frame. The writer baked the body's placement
/// into the solid, so a part shifted 100 mm along X is not sitting at the origin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolidRead {
    pub sound: Sound,
    pub measure: Measure,
    pub low: LowCorner,
    pub high: HighCorner,
}

/// One mesh read from an STL file. It is not a body of the document.
///
/// `low` and `high` are the bounds of its vertices, in the file's frame. The writer moved a copy by the
/// body's placement, so a part shifted 100 mm along X is not sitting at the origin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshRead {
    pub volume: Volume,
    /// How many triangles the file held.
    pub triangles: u32,
    pub low: LowCorner,
    pub high: HighCorner,
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
    /// The volume of the body's mesh, in mm^3.
    ///
    /// The kernel integral of a helical solid is the wrong instrument. On the M10x1.5 rod of this call it
    /// reads 1686.45 mm^3 while the mesh is 1681.40 mm^3. A body the rebuild left without a mesh is 0.
    pub mesh: Volume,
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

    /// Write the live bodies to a STEP file.
    ///
    /// A body with no live shape is left out. A body a later feature has used is left out too, because
    /// that shape is still cached. A 2 mm round of one edge of the 10 by 20 by 30 mm box is one solid
    /// of 5974.25 mm^3. Each remaining body goes with its world placement. The plain box, after it has
    /// been read, comes back as one valid solid of 6000 mm^3.
    /// A write before that read, and a write of an empty document, are `cad-step-nothing-to-export`
    /// and create no file. A path that is not Unicode is `io-path-not-utf8`. The kernel writes a
    /// transfer summary while it saves. This call does not hide that summary.
    pub fn save_step(&self, path: impl AsRef<std::path::Path>) -> Result<(), StoreError> {
        let path = unicode_path(path.as_ref())?;
        let consumed = self.project.consumed_bodies();
        let gate = qymcad_kernel::kernel_gate();
        let shapes = self.kernel.shapes.borrow();
        let mut pairs = Vec::new();
        for body in &self.project.bodies {
            if consumed.contains(&body.id) {
                continue;
            }
            if let Some(shape) = shapes.get(&body.id) {
                pairs.push((shape, self.project.body_world_transform(body.id)));
            }
        }
        let written = qymcad_kernel::write_step(&pairs, path);
        drop(shapes);
        drop(gate);
        written.map_err(store_error)
    }

    /// Read the solids of a STEP file.
    ///
    /// The file is not the document. The timeline, the parameters, and the face names stay in the
    /// `.qcad` file. A missing file is `cad-file-not-found#` and the path, checked before the geometry
    /// reader, which would otherwise say the bodies could not be read. The 10 by 20 by 30 mm box comes
    /// back as one valid solid of 6000 mm^3. Each solid also carries its low corner and its high corner.
    /// The writer baked the placement into the solid, so the corner is in the file's frame. A part
    /// shifted 100 mm along X comes back as the second solid. Its low corner is 99.9999999000000059 mm
    /// along X, and each face sits 1e-7 mm outside the true face. A missing box is not reported as the
    /// origin.
    pub fn open_step(path: impl AsRef<std::path::Path>) -> Result<Vec<SolidRead>, StoreError> {
        let path = unicode_path(path.as_ref())?;
        if !std::path::Path::new(path).is_file() {
            return Err(StoreError { text: format!("cad-file-not-found#{path}") });
        }
        let gate = qymcad_kernel::kernel_gate();
        let solids = qymcad_kernel::step_solids(path).map_err(store_error)?;
        let read = solids.iter().map(read_solid).collect::<Result<Vec<_>, _>>()?;
        drop(gate);
        Ok(read)
    }

    /// Write the stored meshes to an STL file.
    ///
    /// The mesh is the one the rebuild already stored. It is copied, and the copy is moved by the body's
    /// world placement. The stored mesh stays where it is. A body a later feature has used is left out. A
    /// body with no triangle is left out. The 10 by 20 by 30 mm box, after it has been read, is 12 triangles
    /// and comes back at 6000 mm^3. A write before that read, and a write of an empty document, are
    /// `io-stl-no-triangles` and create no file. A path that is not Unicode is `io-path-not-utf8`.
    pub fn save_stl(&self, path: impl AsRef<std::path::Path>) -> Result<(), StoreError> {
        let path = unicode_path(path.as_ref())?;
        let consumed = self.project.consumed_bodies();
        let mut meshes = Vec::new();
        for body in &self.project.bodies {
            if consumed.contains(&body.id) || body.mesh.tris.is_empty() {
                continue;
            }
            let mut mesh = body.mesh.clone();
            let place = self.project.body_world_transform(body.id);
            mesh.transform(&place);
            meshes.push(mesh);
        }
        qymcad_io::export_stl(&meshes, path).map_err(store_error)
    }

    /// Read an STL file as one mesh.
    ///
    /// The file is not the document. A missing file is `cad-file-not-found#` and the path, checked before
    /// the reader, which would otherwise say the file cannot be read. The 10 by 20 by 30 mm box comes back
    /// as 12 triangles of 6000 mm^3. The mesh also carries its low corner and its high corner. A part shifted
    /// 100 mm along X, drawn there alone, comes back with its low corner at 100 mm and its high corner at
    /// 110 by 20 by 30 mm. The 32-bit float did not move this box. The mesh kept in the document stays at
    /// the origin. A mesh with no vertex is not reported as the origin.
    pub fn open_stl(path: impl AsRef<std::path::Path>) -> Result<MeshRead, StoreError> {
        let path = unicode_path(path.as_ref())?;
        if !std::path::Path::new(path).is_file() {
            return Err(StoreError { text: format!("cad-file-not-found#{path}") });
        }
        let mesh = qymcad_io::import_stl(path).map_err(store_error)?;
        read_mesh(&mesh)
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

    /// Round `edge` of `body` by `radius` and return the fillet.
    ///
    /// The edge is the descriptor from the last read. The model records a snapshot of that edge with the
    /// node, so a later edit of the sketch can repair the name. A descriptive reference records none, and
    /// an empty edge list rounds every edge, so the one named edge is what is stored. An id of 0 is stored
    /// as that empty list: the 10 by 20 by 30 mm box came back quiet at about 5805 mm^3, every edge
    /// rounded, so that id is refused before a node is added. A body already taken by a feature that the
    /// rebuild has not refused is refused the same way. Two fillets of the extrusion came back quiet, two
    /// bodies of one part, each about 5982.83 mm^3. A fillet the rebuild has refused is that same node
    /// again: a later 2 mm call had left the 100 mm refusal in place and a second body at about 5982.83
    /// mm^3, so the radius and the edge are written onto the refused node. Naming that fillet's own id
    /// does the same: it had added another node on a body the refusal never built. A chamfer the rebuild has
    /// refused is left as it is. A fillet added beside it left `chamfer 100.00 too big` in the read, and a
    /// fillet of that chamfer's own id added a node the rebuild answered with `error-source-body-not-built`.
    /// An id that was never a solid is refused before a node is added, so a later rounding of the real body
    /// is not left beside that refusal.
    /// The shape appears when the document is rebuilt. The returned id is the fillet. The extrusion's shape
    /// can still sit in the cache.
    pub fn fillet(&mut self, body: BodyId, edge: EdgeId, radius: Radius) -> Result<BodyId, DrawError> {
        if edge.id == 0 {
            return Err(DrawError { text: "error-edges-not-found".into() });
        }
        if body_is_taken(&self.project, body.id) {
            return Err(DrawError { text: "msg-body-consumed".into() });
        }
        if let Some(id) = replace_refused_fillet(&mut self.project, body.id, edge.id, radius.mm) {
            return Ok(BodyId { id });
        }
        if let Some(text) = refusal_holds(&self.project, body.id) {
            return Err(DrawError { text: text.into() });
        }
        if let Some(text) = source_has_no_solid(&self.project, body.id) {
            return Err(DrawError { text: text.into() });
        }
        let id = self.project.add_fillet(body.id, radius.mm, vec![edge.id]);
        Ok(BodyId { id })
    }

    /// Bevel `edge` of `body` by `distance` and return the chamfer.
    ///
    /// The edge is the descriptor from the last read. The model records a snapshot of that edge with the
    /// node, so a later edit of the sketch can repair the name. An empty edge list bevels every edge, so
    /// the one named edge is what is stored. An id of 0 is stored as that empty list and is refused before
    /// a node is added. A body already taken by a feature that the rebuild has not refused is refused the
    /// same way, including a chamfer that has not been read yet. A chamfer the rebuild has refused is that
    /// same node again, whether the caller names the source or the chamfer's own id. A fillet the rebuild
    /// has refused is left as it is. A chamfer added beside it left `fillet R100.00 failed on 1 edges` in
    /// the read, and a chamfer of that fillet's own id added a node the rebuild answered with
    /// `error-source-body-not-built`. An id that was never a solid is refused before a node is added: a
    /// chamfer of that id, then a 2 mm chamfer of the box, left the read on `error-source-body-not-built`
    /// beside a solid of 5960 mm^3. A chamfer of the sketch id stayed on `error-source-body-deleted`.
    /// The shape appears when the document is rebuilt. The returned id is the chamfer. The extrusion's
    /// shape can still sit in the cache.
    pub fn chamfer(&mut self, body: BodyId, edge: EdgeId, distance: Distance) -> Result<BodyId, DrawError> {
        if edge.id == 0 {
            return Err(DrawError { text: "error-edges-not-found".into() });
        }
        if body_is_taken(&self.project, body.id) {
            return Err(DrawError { text: "msg-body-consumed".into() });
        }
        if let Some(id) = replace_refused_chamfer(&mut self.project, body.id, edge.id, distance.mm) {
            return Ok(BodyId { id });
        }
        if let Some(text) = refusal_holds(&self.project, body.id) {
            return Err(DrawError { text: text.into() });
        }
        if let Some(text) = source_has_no_solid(&self.project, body.id) {
            return Err(DrawError { text: text.into() });
        }
        let id = self.project.add_chamfer(body.id, distance.mm, vec![edge.id]);
        Ok(BodyId { id })
    }

    /// Open `face` of `body` and leave a wall of `thickness`. The wall is taken out of the body.
    ///
    /// The face is the descriptor from the last read. The kernel takes the absolute value of the thickness
    /// and then chooses the side, so a thickness of -2 mm on this 10 by 20 by 30 mm box would hollow it to
    /// 3312 mm^3 and the read would stay quiet. A thickness that is not a positive finite length is refused
    /// before a node is added and before a refused shell is rewritten. The shape appears when the document
    /// is rebuilt. The returned id is the shell. The extrusion's shape can still sit in the cache.
    pub fn shell(&mut self, body: BodyId, face: FaceId, thickness: Thickness) -> Result<BodyId, DrawError> {
        if !(thickness.mm.is_finite() && thickness.mm > 0.0) {
            return Err(DrawError { text: "sketch-shell-thickness".into() });
        }
        if body_is_taken(&self.project, body.id) {
            return Err(DrawError { text: "msg-body-consumed".into() });
        }
        if let Some(id) = replace_refused_shell(&mut self.project, body.id, face.id, thickness.mm) {
            return Ok(BodyId { id });
        }
        if let Some(text) = refusal_holds(&self.project, body.id) {
            return Err(DrawError { text: text.into() });
        }
        if let Some(text) = source_has_no_solid(&self.project, body.id) {
            return Err(DrawError { text: text.into() });
        }
        let id = self.project.add_shell(body.id, thickness.mm, vec![face.id], false);
        Ok(BodyId { id })
    }

    /// Drill a plain hole of `diameter` and `depth` at the centre of `face`.
    ///
    /// The face is the descriptor from the last read. The centre and the normal come from that face on
    /// the source body. The kernel takes the absolute value of the depth, so a depth of -40 mm on this
    /// 10 by 20 by 30 mm box would drill a 40 mm hole and the read would stay quiet. A diameter or a depth
    /// that is not a positive finite length is refused before a node is added and before a refused hole is
    /// rewritten. A face id of 0 is not a pick: the rebuild would drill at the stored point, so that id is
    /// refused before a node. The shape appears when the document is rebuilt. The returned id is the hole.
    /// The extrusion's shape can still sit in the cache.
    pub fn hole(&mut self, body: BodyId, face: FaceId, diameter: Diameter, depth: Depth) -> Result<BodyId, DrawError> {
        if !(diameter.mm.is_finite() && diameter.mm > 0.0) {
            return Err(DrawError { text: "sketch-hole-diameter".into() });
        }
        if !(depth.mm.is_finite() && depth.mm > 0.0) {
            return Err(DrawError { text: "sketch-hole-depth".into() });
        }
        if face.id == 0 {
            return Err(DrawError { text: "error-face-not-found".into() });
        }
        if body_is_taken(&self.project, body.id) {
            return Err(DrawError { text: "msg-body-consumed".into() });
        }
        // A step back keeps the face name and drops the face list. Placing the hole then stores the origin,
        // and the next read projects that origin onto the cap: the drill stands at the corner. Measured on
        // this 10 by 20 by 30 mm box, a 4 mm hole left 5905.75 mm^3, a quarter of the cylinder, where the
        // centre would have left 5623.01 mm^3. The read puts the face list back before the centre is stored.
        let host = hole_source(&self.project, body.id);
        if named_face_has_no_centre(&self.project, host, face.id) {
            let _refresh = self.document();
        }
        if named_face_has_no_centre(&self.project, host, face.id) {
            return Err(DrawError { text: "error-face-not-found".into() });
        }
        if let Some(id) = replace_refused_hole(&mut self.project, body.id, face.id, HoleCut { diameter, depth }) {
            return Ok(BodyId { id });
        }
        if let Some(text) = refusal_holds(&self.project, body.id) {
            return Err(DrawError { text: text.into() });
        }
        if let Some(text) = source_has_no_solid(&self.project, body.id) {
            return Err(DrawError { text: text.into() });
        }
        let key = hole_face(&self.project, body.id, face.id);
        let id = self.project.add_hole(body.id, key, diameter.mm, depth.mm);
        Ok(BodyId { id })
    }

    /// Build a box from its three sizes and return the live body of the part.
    ///
    /// The box stands centred on X and Y, from Z = 0. A length, a width, or a height that is not a positive
    /// finite number is refused before a node. The footprint takes the absolute value of the length and the
    /// width, so a negative length would still build the positive box and the read would stay quiet. The
    /// first solid of the part is this box. A later solid is united with the one already there.
    pub fn cuboid(&mut self, block: Cuboid) -> Result<BodyId, DrawError> {
        if !above_zero(block.length.mm) {
            return Err(DrawError { text: "sketch-box-length".into() });
        }
        if !above_zero(block.width.mm) {
            return Err(DrawError { text: "sketch-box-width".into() });
        }
        if !above_zero(block.height.mm) {
            return Err(DrawError { text: "sketch-box-height".into() });
        }
        let made = self.project.add_box(block.length.mm, block.width.mm, block.height.mm);
        Ok(BodyId { id: settle(&mut self.project, &mut self.kernel, made) })
    }

    /// Build a cylinder from its radius and height and return the live body of the part.
    ///
    /// The cylinder stands on Z = 0, centred on the origin. A radius or a height that is not a positive
    /// finite number is refused before a node.
    pub fn cylinder(&mut self, round: Cylinder) -> Result<BodyId, DrawError> {
        if !above_zero(round.radius.mm) {
            return Err(DrawError { text: "sketch-cylinder-radius".into() });
        }
        if !above_zero(round.height.mm) {
            return Err(DrawError { text: "sketch-cylinder-height".into() });
        }
        let made = self.project.add_cylinder(round.radius.mm, round.height.mm);
        Ok(BodyId { id: settle(&mut self.project, &mut self.kernel, made) })
    }

    /// Build a sphere from its radius and return the live body of the part.
    ///
    /// The sphere is centred at the origin. A radius that is not a positive finite number is refused before
    /// a node.
    pub fn sphere(&mut self, radius: Radius) -> Result<BodyId, DrawError> {
        if !above_zero(radius.mm) {
            return Err(DrawError { text: "sketch-sphere-radius".into() });
        }
        let made = self.project.add_sphere(radius.mm);
        Ok(BodyId { id: settle(&mut self.project, &mut self.kernel, made) })
    }

    /// Build a cone from its two radii and its height and return the live body of the part.
    ///
    /// The cone stands on Z = 0. The bottom radius has to be a positive finite number. The top may be zero,
    /// which is a sharp cone. A negative top is refused before a node. When the bottom is also bad, the
    /// bottom key wins.
    pub fn cone(&mut self, cone: Cone) -> Result<BodyId, DrawError> {
        if !above_zero(cone.bottom.mm) {
            return Err(DrawError { text: "sketch-cone-bottom".into() });
        }
        if !(cone.top.mm.is_finite() && cone.top.mm >= 0.0) {
            return Err(DrawError { text: "sketch-cone-top".into() });
        }
        if !above_zero(cone.height.mm) {
            return Err(DrawError { text: "sketch-cone-height".into() });
        }
        let made = self.project.add_cone(cone.bottom.mm, cone.top.mm, cone.height.mm);
        Ok(BodyId { id: settle(&mut self.project, &mut self.kernel, made) })
    }

    /// Build a torus from its ring and its tube and return the live body of the part.
    ///
    /// The torus lies in XY about the origin. A ring or a tube that is not a positive finite number is
    /// refused before a node. A zero ring is the ring key, not the key for a tube that meets the ring. A
    /// tube that is not thinner than the ring is refused before a node with that existing key: the rebuild
    /// would write a node and keep the English sentence.
    pub fn torus(&mut self, torus: Torus) -> Result<BodyId, DrawError> {
        if !above_zero(torus.ring.mm) {
            return Err(DrawError { text: "sketch-torus-ring".into() });
        }
        if !above_zero(torus.tube.mm) {
            return Err(DrawError { text: "sketch-torus-tube".into() });
        }
        if torus.tube.mm >= torus.ring.mm {
            return Err(DrawError { text: "error-torus-through-itself".into() });
        }
        let made = self.project.add_torus(torus.ring.mm, torus.tube.mm);
        Ok(BodyId { id: settle(&mut self.project, &mut self.kernel, made) })
    }

    /// Build a prism from its circumradius, its height, and its number of sides, and return the live body.
    ///
    /// The prism stands on Z = 0, centred on the origin. A count below 3 is raised to 3 by the profile, so a
    /// two-sided prism would quietly become a triangle. A count outside 3 to 64 is refused before a node.
    pub fn prism(&mut self, prism: Prism) -> Result<BodyId, DrawError> {
        if !above_zero(prism.radius.mm) {
            return Err(DrawError { text: "sketch-prism-radius".into() });
        }
        if !above_zero(prism.height.mm) {
            return Err(DrawError { text: "sketch-prism-height".into() });
        }
        if !(3..=64).contains(&prism.sides.count) {
            return Err(DrawError { text: "sketch-prism-sides".into() });
        }
        let made = self.project.add_prism(prism.radius.mm, prism.sides.count, prism.height.mm);
        Ok(BodyId { id: settle(&mut self.project, &mut self.kernel, made) })
    }

    /// Turn `profile` about a sketch axis by `angle` and return the live body of the part.
    ///
    /// The turn is one-sided, which is the model's default. The angle is in degrees. It has to be finite,
    /// greater than zero, and at most a full turn. Anything else is refused before the profile is looked at,
    /// and before a node. The contour has to be a closed contour of that sketch. A contour id of 0 would
    /// turn every closed contour, so a missing contour is refused before a node. The shape appears when the
    /// document is rebuilt.
    pub fn revolve(&mut self, profile: Profile, axis: SketchAxis, angle: Angle) -> Result<BodyId, DrawError> {
        if !(angle.deg.is_finite() && angle.deg > 0.0 && angle.deg <= 360.0) {
            return Err(DrawError { text: "sketch-revolve-angle".into() });
        }
        if !closed_profile(&self.project, profile) {
            return Err(DrawError { text: "error-profile-not-found".into() });
        }
        let code = match axis {
            SketchAxis::X => 0,
            SketchAxis::Y => 1,
        };
        let made = self.project.add_revolve_on(profile.sketch.id, profile.contour.id, code, angle.deg);
        Ok(BodyId { id: settle(&mut self.project, &mut self.kernel, made) })
    }

    /// Draw an open line on `plane` and return its sketch and contour.
    ///
    /// The line is the path of a sweep. A closed rectangle is a section, and this contour stays open. Ends
    /// that are not two different finite points are refused before a sketch is added: the sketch would
    /// otherwise keep a contour the sweep cannot follow. The session stands in a part first, the same way
    /// a rectangle does.
    pub fn line(&mut self, plane: WorldPlane, segment: Segment) -> Result<Path, DrawError> {
        if !spans_a_segment(&segment) {
            return Err(DrawError { text: "sketch-line-empty".into() });
        }
        let (si, sid) = begin_sketch(&mut self.project, placed(plane));
        let (x0, y0) = segment.from.at;
        let (x1, y1) = segment.to.at;
        let _edge = self.project.add_line_entity(si, x0, y0, x1, y1, qymcad_core::feature::Purpose::Real);
        self.project.regen_sketch(si);
        let contour = self.project.sketches[si]
            .contour_ids
            .iter()
            .copied()
            .find(|&id| contour_of(&self.project, id).is_some_and(|contour| contour.form == Form::Open))
            .ok_or_else(|| DrawError { text: "sketch-line-empty".into() })?;
        Ok(Path { sketch: SketchId { id: sid }, contour: ContourId { id: contour } })
    }

    /// Raise a datum plane off `plane` by `offset` and return its id.
    ///
    /// Zero lies on the world plane. A negative distance stands on the other side of it. A distance that
    /// is not finite is refused before a plane is added. The session stands in a part first, so the plane
    /// is not left on the root assembly.
    pub fn offset_plane(&mut self, plane: WorldPlane, offset: Offset) -> Result<PlaneId, DrawError> {
        if !offset.mm.is_finite() {
            return Err(DrawError { text: "sketch-plane-offset".into() });
        }
        let _made = self.project.part_to_draw_in();
        let base = match plane {
            WorldPlane::Xy => qymcad_core::feature::BasePlane::XY,
            WorldPlane::Xz => qymcad_core::feature::BasePlane::XZ,
            WorldPlane::Yz => qymcad_core::feature::BasePlane::YZ,
        };
        Ok(PlaneId { id: self.project.add_offset_plane(base, offset.mm) })
    }

    /// Draw a rectangle on a datum `plane` and return its sketch and contour.
    ///
    /// A new sketch starts on the table. The plane is set to this datum, so the corners are read in the
    /// datum's own coordinates. The rebuild resolves the datum, because the plane node stands above the
    /// sketch. A rectangle that does not span both directions is refused before a sketch is added. A plane
    /// that is not a datum in the document is refused the same way: an id of 0 returned a profile, and
    /// lofting that square with one on the table wrote a node whose read was `error-empty-result`.
    pub fn rectangle_on(&mut self, plane: PlaneId, corners: Corners) -> Result<Profile, DrawError> {
        if !spans_a_rectangle(&corners) {
            return Err(DrawError { text: "sketch-rectangle-empty".into() });
        }
        if !raised_plane(&self.project, plane.id) {
            return Err(DrawError { text: "sketch-plane-missing".into() });
        }
        let (si, sid) = begin_sketch(&mut self.project, qymcad_core::feature::SketchPlane::Datum(plane.id));
        let (x0, y0) = corners.from.at;
        let (x1, y1) = corners.to.at;
        let _sides = self.project.add_rect_entity(si, x0, y0, x1, y1, qymcad_core::feature::Purpose::Real);
        self.project.regen_sketch(si);
        let contour = self.project.sketches[si]
            .contour_ids
            .iter()
            .copied()
            .find(|&id| contour_of(&self.project, id).is_some_and(|item| item.form == Form::Closed))
            .ok_or_else(|| DrawError { text: "sketch-rectangle-empty".into() })?;
        Ok(Profile { sketch: SketchId { id: sid }, contour: ContourId { id: contour } })
    }

    /// Sweep the section along the path and return the live body of the part.
    ///
    /// The section contour is the one named here. An empty list would let the model choose. A contour id
    /// of 0 is refused before a node: the model reads 0 as the first contour. A section that is not a
    /// closed contour of its sketch is `error-profile-not-found`. A path that is not a contour of its
    /// sketch, including a contour id of 0, is `error-sweep-path-missing`. The section and the path have
    /// to be two sketches: one square used as both came back `error-broken-solid` and was united with the
    /// box. A closed path is refused the same way: a 2 mm square swept along a 20 mm square on the front
    /// plane came back `error-broken-solid error-source-body-not-built`, and a union stood on the sweep.
    /// A 10 mm square swept 40 mm up
    /// the front plane is a bar of 4000 mm^3. A later solid is united with the one already in the part: a
    /// 2 mm square swept 10 mm up from the origin sits inside the 10 by 20 by 30 mm box, and the union
    /// stays 6000 mm^3.
    pub fn sweep(&mut self, asked: Sweep) -> Result<BodyId, DrawError> {
        if !closed_profile(&self.project, asked.section) {
            return Err(DrawError { text: "error-profile-not-found".into() });
        }
        if !named_path(&self.project, asked.path) {
            return Err(DrawError { text: "error-sweep-path-missing".into() });
        }
        if asked.section.sketch.id == asked.path.sketch.id {
            return Err(DrawError { text: "sketch-sweep-same-sketch".into() });
        }
        if !path_is_open(&self.project, asked.path) {
            return Err(DrawError { text: "sketch-sweep-closed-path".into() });
        }
        let made = self.project.add_sweep(asked.section.sketch.id, vec![asked.section.contour.id], asked.path.sketch.id, asked.path.contour.id);
        Ok(BodyId { id: settle(&mut self.project, &mut self.kernel, made) })
    }

    /// Loft from the lower section to the upper section and return the live body of the part.
    ///
    /// The two contour ids are passed in that order. A contour id of 0 is refused before a node: the
    /// model would take the first closed contour of that sketch. Two sections on one plane are refused
    /// before a node: lofting them beside the box united the failure with the box. Two sections that meet
    /// where their planes cross are refused the same way: a 10 mm square on the table and a 10 mm square
    /// on the front plane, both from the origin, came back `operation failed: Loft error-source-body-not-built`
    /// and a union. A front square that starts 0.05 mm above the table stayed a solid of 502.5 mm^3. The loft is a solid boss, not a sheet.
    /// A 10 mm square on the table and a 20 mm square 30 mm above it are a frustum of 7000 mm^3. A later
    /// solid is united with the one already in the part.
    pub fn loft(&mut self, asked: Loft) -> Result<BodyId, DrawError> {
        if !closed_profile(&self.project, asked.lower.profile) || !closed_profile(&self.project, asked.upper.profile) {
            return Err(DrawError { text: "error-profile-not-found".into() });
        }
        if same_plane(&self.project, asked.lower.profile.sketch.id, asked.upper.profile.sketch.id) {
            return Err(DrawError { text: "sketch-loft-same-plane".into() });
        }
        if sections_meet(&self.project, asked) {
            return Err(DrawError { text: "sketch-loft-sections-meet".into() });
        }
        let made = self.project.add_loft(vec![asked.lower.profile.sketch.id, asked.upper.profile.sketch.id], vec![asked.lower.profile.contour.id, asked.upper.profile.contour.id], false, 0, 1, false);
        Ok(BodyId { id: settle(&mut self.project, &mut self.kernel, made) })
    }

    /// Apply one boolean to two bodies and return that node.
    ///
    /// The call does not unite the result with a further body. The boolean is the one node. An unread
    /// solid is accepted: the box and the extrusion have no mesh until the first read, and the rebuild
    /// builds them before the boolean. An id of 0 is `error-source-body-not-built`. An id that was never
    /// a body is `error-source-body-deleted`. A body already taken by a node the rebuild has not refused
    /// is `msg-body-consumed`. The same id on both sides is `sketch-boolean-same`, because that body is
    /// not consumed yet. A body the rebuild will refuse, such as a turn across its axis, is
    /// `error-source-body-not-built` before a node: cutting the box with that turn came back
    /// `error-revolve-profile-crosses-axis error-source-body-not-built`. A 40 by 30 by 10 mm box cut by a 10 mm extrusion that sits inside it leaves
    /// 11000 mm^3. Their union leaves 12000 mm^3, and their common part leaves 1000 mm^3.
    pub fn boolean(&mut self, asked: Boolean) -> Result<BodyId, DrawError> {
        if let Some(text) = boolean_side(&self.project, asked.base.body.id) {
            return Err(DrawError { text: text.into() });
        }
        if asked.base.body.id == asked.tool.body.id {
            return Err(DrawError { text: "sketch-boolean-same".into() });
        }
        if let Some(text) = boolean_side(&self.project, asked.tool.body.id) {
            return Err(DrawError { text: text.into() });
        }
        let op = match asked.op {
            BoolOp::Cut => 0,
            BoolOp::Union => 1,
            BoolOp::Intersect => 2,
        };
        Ok(BodyId { id: self.project.add_body_boolean(asked.base.body.id, asked.tool.body.id, op) })
    }

    /// Mirror `body` through a world plane and return that node.
    ///
    /// The call keeps the original and the image. It does not unite the result with a further body: the mirror
    /// consumes its source, the way a fillet does. The plane codes are the ones the kernel already uses: 0 for XY,
    /// 1 for XZ, 2 for YZ. A 10 by 20 by 30 mm box mirrored through the table is one solid of 12000 mm^3, from
    /// z = -30 to z = 30, sharing the face at z = 0. An id of 0 is `error-source-body-not-built`. An id that was
    /// never a body is `error-source-body-deleted`. A body already taken is `msg-body-consumed`. A turn across
    /// its axis is `error-source-body-not-built` before a node.
    pub fn mirror(&mut self, asked: Mirror) -> Result<BodyId, DrawError> {
        if let Some(text) = boolean_side(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        let plane = match asked.plane {
            WorldPlane::Xy => 0,
            WorldPlane::Xz => 1,
            WorldPlane::Yz => 2,
        };
        Ok(BodyId { id: self.project.add_mirror(asked.body.id, plane, true, 0) })
    }

    /// Lay `body` out in a straight row and return that node.
    ///
    /// The count includes the copy that stays where the body already is. The call does not unite the result with
    /// a further body. A count below 2 is `error-array-of-one` before a node. A step that is not finite, or whose
    /// length is not greater than 1e-6 mm, is `sketch-array-step` before a node. A 10 by 20 by 30 mm box stepped
    /// 10 mm along X, three copies, is one solid of 18000 mm^3: the copies share a face.
    pub fn linear(&mut self, asked: Linear) -> Result<BodyId, DrawError> {
        if let Some(text) = boolean_side(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        if asked.copies.n < 2 {
            return Err(DrawError { text: "error-array-of-one".into() });
        }
        if !step_moves(asked.step) {
            return Err(DrawError { text: "sketch-array-step".into() });
        }
        Ok(BodyId { id: self.project.add_linear_array(asked.body.id, asked.step.x.mm, asked.step.y.mm, asked.step.z.mm, asked.copies.n) })
    }

    /// Turn `body` about the world Z axis and return that node.
    ///
    /// The count includes the copy that stays where the body already is. A full turn, an angle of 359.9 degrees
    /// or more in either direction, is passed through, and the model spaces it as `360 / count`. A smaller angle
    /// is passed through and divided by the count. A count below 2 is `error-array-of-one` before a node. An
    /// angle that is not finite, or an angle of zero, is `sketch-array-angle` before a node. A 10 mm square in
    /// the first quadrant, grown 10 mm, turned in four copies over a full turn, is one solid of 4000 mm^3.
    pub fn circular(&mut self, asked: Circular) -> Result<BodyId, DrawError> {
        if let Some(text) = boolean_side(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        if asked.copies.n < 2 {
            return Err(DrawError { text: "error-array-of-one".into() });
        }
        if !asked.turn.deg.is_finite() || asked.turn.deg == 0.0 {
            return Err(DrawError { text: "sketch-array-angle".into() });
        }
        Ok(BodyId { id: self.project.add_circular_array(asked.body.id, asked.copies.n, asked.turn.deg) })
    }

    /// Split `body` by a world plane and return the pieces, from the negative side of the normal.
    ///
    /// The count is taken from the live shape before a node is written. A 10 by 20 by 30 mm box cut by
    /// the table at 10 mm is a lower piece of 2000 mm^3 and an upper piece of 4000 mm^3. The count cuts
    /// the cached shape. A clean box is skipped by the next read, so the split would cut whatever the
    /// count left behind and the piece count would no longer match the node. The source is marked dirty
    /// after the count, and the next read rebuilds the box before it cuts. A plane that misses the body,
    /// or lies on a face, is `error-op-failed-split-body` before a node. An offset that is not finite is
    /// `sketch-plane-offset` before a node.
    pub fn split(&mut self, asked: Split) -> Result<Pieces, DrawError> {
        if let Some(text) = boolean_side(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        if !asked.offset.mm.is_finite() {
            return Err(DrawError { text: "sketch-plane-offset".into() });
        }
        let count = counted_pieces(&mut self.project, &mut self.kernel, asked.body.id, asked.plane, asked.offset.mm);
        mark_body_dirty(&mut self.project, asked.body.id);
        let pieces = match count {
            CutCount::Missing => return Err(DrawError { text: "error-source-body-not-built".into() }),
            CutCount::Short => return Err(DrawError { text: "error-op-failed-split-body".into() }),
            CutCount::Pieces(n) if n >= 2 => n,
            CutCount::Pieces(_) => return Err(DrawError { text: "error-op-failed-split-body".into() }),
        };
        let plane = match asked.plane {
            WorldPlane::Xy => 0,
            WorldPlane::Xz => 1,
            WorldPlane::Yz => 2,
        };
        let bodies = self.project.add_split_body(asked.body.id, plane, 0, asked.offset.mm, pieces);
        Ok(Pieces { bodies: bodies.into_iter().map(|id| BodyId { id }).collect() })
    }

    /// Mark the faces of `body` with a world plane and return that node.
    ///
    /// The body stays one. A 10 by 20 mm rectangle grown 30 mm, marked by the table at 10 mm, stays
    /// 6000 mm^3 and goes from 6 named faces to 10. The end cap stays one face of 200 mm^2. The
    /// section is asked of a copy, and the copy takes the kernel lock itself, so this rebuild lets
    /// the lock go first. A clean extrusion is skipped by the next read, so the source is marked
    /// dirty after the section and the next read rebuilds it before it splits the faces. A plane
    /// clear of the body is `error-op-failed-split-faces` before a node. An offset that is not
    /// finite is `sketch-plane-offset` before a node.
    pub fn split_faces(&mut self, asked: Split) -> Result<BodyId, DrawError> {
        if let Some(text) = boolean_side(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        if !asked.offset.mm.is_finite() {
            return Err(DrawError { text: "sketch-plane-offset".into() });
        }
        let cut = faces_split(&mut self.project, &mut self.kernel, asked.body.id, asked.plane, asked.offset.mm);
        mark_body_dirty(&mut self.project, asked.body.id);
        match cut {
            FaceCut::Missing => return Err(DrawError { text: "error-source-body-not-built".into() }),
            FaceCut::Clear => return Err(DrawError { text: "error-op-failed-split-faces".into() }),
            FaceCut::Crossed => {}
        }
        let plane = match asked.plane {
            WorldPlane::Xy => 0,
            WorldPlane::Xz => 1,
            WorldPlane::Yz => 2,
        };
        Ok(BodyId { id: self.project.add_split_face(asked.body.id, plane, 0, asked.offset.mm) })
    }

    /// Tilt one wall of a body about a neutral face and return that node.
    ///
    /// The wall and the neutral face are descriptors from the last read. The pull stored is the
    /// neutral face's own normal, so a positive angle opens outward. On the 10 by 20 by 30 mm box
    /// the end cap is the neutral face and a wall of 300 mm^2 shares a 10 mm edge with it: 10
    /// degrees adds `4500 * tan(10 deg)` mm^3 to the 6000 mm^3 box, and -10 degrees takes that
    /// wedge away. A face id of 0 is `error-face-not-found` before a node. The same face named as
    /// both the wall and the neutral face is `error-draft-needs-faces` before a node. An angle
    /// whose absolute value is below 1e-9 is `error-draft-angle-zero` before a node. An angle that
    /// is not finite is `error-draft-angle` before a node. Those refusals happen before a refused
    /// draft is rewritten, so a bad angle does not replace a draft the rebuild has already refused.
    /// A draft the rebuild has refused is that same node again, whether the caller names the source
    /// or the draft's own id. A fillet the rebuild has refused is left as it is. The call does not
    /// ask the kernel first: an angle the faces cannot take is a node, and the read carries
    /// `draft 90.0 failed`. The shape appears when the document is rebuilt. The returned id is the
    /// draft. The extrusion's shape can still sit in the cache.
    pub fn draft(&mut self, asked: Draft) -> Result<BodyId, DrawError> {
        if asked.tilted.face.id == 0 || asked.neutral.face.id == 0 {
            return Err(DrawError { text: "error-face-not-found".into() });
        }
        if asked.tilted.face.id == asked.neutral.face.id {
            return Err(DrawError { text: "error-draft-needs-faces".into() });
        }
        if asked.angle.deg.abs() < 1e-9 {
            return Err(DrawError { text: "error-draft-angle-zero".into() });
        }
        if !asked.angle.deg.is_finite() {
            return Err(DrawError { text: "error-draft-angle".into() });
        }
        if body_is_taken(&self.project, asked.body.id) {
            return Err(DrawError { text: "msg-body-consumed".into() });
        }
        if let Some(id) = replace_refused_draft(&mut self.project, asked) {
            return Ok(BodyId { id });
        }
        if let Some(text) = refusal_holds(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        if let Some(text) = source_has_no_solid(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        let id = self.project.add_draft(asked.body.id, vec![asked.tilted.face.id], asked.neutral.face.id, asked.angle.deg, false);
        Ok(BodyId { id })
    }

    /// Move one face of a body along its own normal and return that node.
    ///
    /// The face is a descriptor from the last read. A positive distance moves it along its own
    /// outward normal and adds material. On the 10 by 20 by 30 mm box the end cap is 200 mm^2:
    /// 10 mm out makes 8000 mm^3, and 10 mm in makes 4000 mm^3. A face id of 0 is
    /// `error-face-not-found` before a node. A distance whose absolute value is below 1e-9 is
    /// `error-zero-push-distance` before a node. A distance that is not finite is
    /// `error-push-distance` before a node. Those refusals happen before a refused push is
    /// rewritten, so a bad distance does not replace a push the rebuild has already refused. A
    /// push the rebuild has refused is that same node again, whether the caller names the source
    /// or the push's own id. A fillet the rebuild has refused is left as it is. The call does not
    /// ask the kernel first: a shift of the whole 30 mm height is a node, and the read carries
    /// `error-empty-result`. The shape appears when the document is rebuilt. The returned
    /// id is the push. The extrusion's shape can still sit in the cache.
    pub fn push(&mut self, asked: Push) -> Result<BodyId, DrawError> {
        if asked.face.id == 0 {
            return Err(DrawError { text: "error-face-not-found".into() });
        }
        if asked.by.mm.abs() < 1e-9 {
            return Err(DrawError { text: "error-zero-push-distance".into() });
        }
        if !asked.by.mm.is_finite() {
            return Err(DrawError { text: "error-push-distance".into() });
        }
        if body_is_taken(&self.project, asked.body.id) {
            return Err(DrawError { text: "msg-body-consumed".into() });
        }
        if let Some(id) = replace_refused_push(&mut self.project, asked) {
            return Ok(BodyId { id });
        }
        if let Some(text) = refusal_holds(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        if let Some(text) = source_has_no_solid(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        let key = hole_face(&self.project, asked.body.id, asked.face.id);
        let id = self.project.add_push_face(asked.body.id, key, asked.by.mm);
        Ok(BodyId { id })
    }

    /// Grow a plate on one face of a body and return that node.
    ///
    /// The face is a descriptor from the last read. A positive thickness grows the plate along that
    /// face's own outward normal and welds it on. On the 10 by 20 by 30 mm box the end cap is
    /// 200 mm^2: 2 mm makes 6400 mm^3. A face id of 0 is `error-face-not-found` before a node. A
    /// thickness whose absolute value is below 1e-9 is `error-zero-thickness` before a node. A
    /// thickness that is not finite is `error-thicken-thickness` before a node. Those refusals
    /// happen before a refused thicken is rewritten, so a bad thickness does not replace a thicken
    /// the rebuild has already refused. A thicken the rebuild has refused is that same node again,
    /// whether the caller names the source or the thicken's own id. The join id stays 0. A fillet
    /// the rebuild has refused is left as it is. The call does not ask the kernel first: a plate of
    /// -2 mm stays inside the box, and the read carries `the plate went into the body and added
    /// nothing`. The shape appears when the document is rebuilt. The returned id is the thicken.
    /// The extrusion's shape can still sit in the cache.
    pub fn thicken(&mut self, asked: Thicken) -> Result<BodyId, DrawError> {
        if asked.face.id == 0 {
            return Err(DrawError { text: "error-face-not-found".into() });
        }
        if asked.plate.mm.abs() < 1e-9 {
            return Err(DrawError { text: "error-zero-thickness".into() });
        }
        if !asked.plate.mm.is_finite() {
            return Err(DrawError { text: "error-thicken-thickness".into() });
        }
        if body_is_taken(&self.project, asked.body.id) {
            return Err(DrawError { text: "msg-body-consumed".into() });
        }
        if let Some(id) = replace_refused_thicken(&mut self.project, asked) {
            return Ok(BodyId { id });
        }
        if let Some(text) = refusal_holds(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        if let Some(text) = source_has_no_solid(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        let id = self.project.add_thicken(asked.body.id, asked.face.id, asked.plate.mm);
        Ok(BodyId { id })
    }

    /// Take one face off a body and return that node.
    ///
    /// The face is a descriptor from the last read. On the 10 by 20 by 30 mm block a 4 mm hole through
    /// the end cap leaves 5623.01 mm^3. Taking that hole's wall away returns the block to 6000 mm^3,
    /// six faces, and the wall is gone. A face id of 0 is `error-faces-not-found` before a node. That
    /// refusal happens before a refused removal is rewritten, so a face id of 0 does not replace a
    /// removal the rebuild has already refused. A removal the rebuild has refused is that same node
    /// again, whether the caller names the source or the removal's own id. A fillet the rebuild has
    /// refused is left as it is. The call does not ask the kernel first: the end cap of the drilled
    /// block stays a node, and the read carries `cannot remove faces: cad-neighbours-not-extendable`.
    /// The shape appears when the document is rebuilt. The returned id is the removal. The hole's
    /// shape can still sit in the cache.
    pub fn remove_face(&mut self, asked: RemoveFace) -> Result<BodyId, DrawError> {
        if asked.face.id == 0 {
            return Err(DrawError { text: "error-faces-not-found".into() });
        }
        if body_is_taken(&self.project, asked.body.id) {
            return Err(DrawError { text: "msg-body-consumed".into() });
        }
        if let Some(id) = replace_refused_removal(&mut self.project, asked) {
            return Ok(BodyId { id });
        }
        if let Some(text) = refusal_holds(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        if let Some(text) = source_has_no_solid(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        let key = hole_face(&self.project, asked.body.id, asked.face.id);
        let id = self.project.add_remove_face(asked.body.id, vec![key]);
        Ok(BodyId { id })
    }

    /// Copy one face of a body out as a sheet and return that node.
    ///
    /// The face is a descriptor from the last read. On the 10 by 20 by 30 mm box the end cap is
    /// 200 mm^2. The copy is a valid sheet of that area, kernel volume 0 mm^3, and 0 solids. The
    /// mesh of that sheet reads 2000 mm^3, so the kernel volume is the measure. A face id of 0 is
    /// `error-face-not-found` before a node. That refusal happens before a refused copy is
    /// rewritten, so a face id of 0 does not replace a copy the rebuild has already refused. A
    /// copy the rebuild has refused is that same node again, whether the caller names the box or
    /// the copy's own id. A fillet the rebuild has refused is left as it is. The call does not
    /// ask the kernel first: a face that is not on the box stays a node, and the read carries
    /// `error-faces-not-found`. The sheet appears when the document is rebuilt. The returned id
    /// is the copy. The box is not consumed.
    pub fn copy_face(&mut self, asked: CopyFace) -> Result<BodyId, DrawError> {
        if asked.face.id == 0 {
            return Err(DrawError { text: "error-face-not-found".into() });
        }
        if body_is_taken(&self.project, asked.body.id) {
            return Err(DrawError { text: "msg-body-consumed".into() });
        }
        if let Some(id) = replace_refused_copy(&mut self.project, asked) {
            return Ok(BodyId { id });
        }
        if let Some(text) = refusal_holds(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        if let Some(text) = source_has_no_solid(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        let id = self.project.add_face_copy(asked.body.id, qymcad_core::refs::Ref::picks(&[asked.face.id]));
        Ok(BodyId { id })
    }

    /// Offset one face of a body along its normal and return that node.
    ///
    /// The face is a descriptor from the last read. On the diameter 10 mm rod, 25 mm long, the side
    /// moved 2 mm outward is a sheet of 1098.56 mm^2. Moved 2 mm inward it is 470.98 mm^2. The mesh
    /// of the outward sheet reads 2556.35 mm^3, so the face area is the measure. A face id of 0 is
    /// `error-face-not-found` before a node. A distance that is not finite is `error-offset-distance`
    /// before a node. Those refusals happen before a refused offset is rewritten, so a face id of 0
    /// does not replace an offset the rebuild has already refused. An offset the rebuild has refused
    /// is that same node again, whether the caller names the rod or the offset's own id. A fillet
    /// the rebuild has refused is left as it is. The call does not ask the kernel first: a distance
    /// of the radius, moved inward, stays a node, and the read carries `a round face moved in by its
    /// radius 5.00 or more`. The sheet appears when the document is rebuilt. The returned id is the
    /// offset. The rod is not consumed.
    pub fn offset_face(&mut self, asked: OffsetFace) -> Result<BodyId, DrawError> {
        if asked.face.id == 0 {
            return Err(DrawError { text: "error-face-not-found".into() });
        }
        if !asked.by.mm.is_finite() {
            return Err(DrawError { text: "error-offset-distance".into() });
        }
        if body_is_taken(&self.project, asked.body.id) {
            return Err(DrawError { text: "msg-body-consumed".into() });
        }
        if let Some(id) = replace_refused_offset(&mut self.project, asked) {
            return Ok(BodyId { id });
        }
        if let Some(text) = refusal_holds(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        if let Some(text) = source_has_no_solid(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        let id = self.project.add_offset_surface(asked.body.id, qymcad_core::refs::Ref::picks(&[asked.face.id]), asked.by.mm);
        Ok(BodyId { id })
    }

    /// Span one edge of a body into a sheet and return that node.
    ///
    /// The edge is a descriptor from the last read. On the diameter 10 mm rod, 25 mm long, a rim is
    /// listed at 0 mm, because the read stores the chord and a closed edge meets itself. Spanned by
    /// position that rim is a sheet of 78.25 mm^2. Asked to meet smoothly it is 78.48 mm^2. The mesh
    /// of the first reads 652.06 mm^3 and the mesh of the other reads 0, so the face area is the
    /// measure. An edge id of 0 is `error-edges-not-found` before a node. That refusal happens before
    /// a refused patch is rewritten, so an edge id of 0 does not replace a patch the rebuild has
    /// already refused. A patch the rebuild has refused is that same node again, whether the caller
    /// names the rod or the patch's own id. A fillet the rebuild has refused is left as it is. The
    /// call does not ask the kernel first: the 25 mm generator stays a node, and the read carries
    /// `operation failed: Patch`. The sheet appears when the document is rebuilt. The returned id is
    /// the patch. The rod is not consumed.
    pub fn patch(&mut self, asked: Patch) -> Result<BodyId, DrawError> {
        if asked.edge.id == 0 {
            return Err(DrawError { text: "error-edges-not-found".into() });
        }
        if body_is_taken(&self.project, asked.body.id) {
            return Err(DrawError { text: "msg-body-consumed".into() });
        }
        if let Some(id) = replace_refused_patch(&mut self.project, asked) {
            return Ok(BodyId { id });
        }
        if let Some(text) = refusal_holds(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        if let Some(text) = source_has_no_solid(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        let id = self.project.add_patch(asked.body.id, qymcad_core::refs::Ref::picks(&[asked.edge.id]), meets_smoothly(asked.meet));
        Ok(BodyId { id })
    }

    /// Sew `asked.sheets` into one body and return that node.
    ///
    /// The gap is how far two edges may stand and still count as one. On the 10 by 20 by 30 mm box the
    /// end cap and the 300 mm^2 wall, sewn at 0.0001 mm, are one sheet: two faces, 200 mm^2 and 300 mm^2,
    /// kernel volume 0. A gap of 0 mm and a gap of -1 mm sew that same sheet. All six faces sewn at
    /// 0.0001 mm close into a solid of 6000 mm^3. The two end caps share no edge, and the read carries
    /// `surfaces share no edges -- nothing to stitch`. That stitch stays in the body list as the first
    /// sheet, one face of 200 mm^2. The mesh of the two-face sheet reads 2000 mm^3, so the face areas are
    /// the measure. Fewer than two sheets is `msg-stitch-needs-two` before a node. The id of a stitch the
    /// rebuild has refused names that node and is not itself a sheet, and a sheet listed twice is still
    /// one sheet: either way, fewer than two sheets writes nothing and does not rewrite the node. A gap
    /// that is not finite is `error-stitch-gap` before a node. A solid is `msg-stitch-only-sheets`. The
    /// call does not ask the kernel first, and it does not run settle. The sheets are consumed. The box
    /// is not.
    pub fn stitch(&mut self, asked: Stitch) -> Result<BodyId, DrawError> {
        if asked.sheets.len() < 2 {
            return Err(DrawError { text: "msg-stitch-needs-two".into() });
        }
        if !asked.gap.mm.is_finite() {
            return Err(DrawError { text: "error-stitch-gap".into() });
        }
        let sheets = distinct_sheets(&self.project, &asked);
        for sheet in &sheets {
            if let Some(text) = source_has_no_solid(&self.project, sheet.id) {
                return Err(DrawError { text: text.into() });
            }
            if !is_sheet(&self.project, sheet.id) {
                return Err(DrawError { text: "msg-stitch-only-sheets".into() });
            }
        }
        if sheets.len() < 2 {
            return Err(DrawError { text: "msg-stitch-needs-two".into() });
        }
        for sheet in &sheets {
            if body_is_taken(&self.project, sheet.id) {
                return Err(DrawError { text: "msg-body-consumed".into() });
            }
        }
        if let Some(id) = replace_refused_stitch(&mut self.project, &asked, &sheets) {
            return Ok(BodyId { id });
        }
        for sheet in &sheets {
            if let Some(text) = refusal_holds(&self.project, sheet.id) {
                return Err(DrawError { text: text.into() });
            }
            if let Some(src) = sheet_came_from(&self.project, sheet.id) {
                if let Some(text) = refusal_holds(&self.project, src) {
                    return Err(DrawError { text: text.into() });
                }
            }
        }
        let id = self.project.add_stitch(sheets.iter().map(|sheet| sheet.id).collect(), asked.gap.mm);
        Ok(BodyId { id })
    }

    /// Cut `asked.sheet` with `asked.tool` and return that node. The piece nearest `asked.keep` stays.
    ///
    /// On the 10 by 20 by 30 mm box the end cap, cut by a tool that crosses it at x = 4, keeps 80 mm^2
    /// for the point (2, 10, 30) and 120 mm^2 for (7, 10, 30). The origin keeps the 80 mm^2 piece. The
    /// mesh of that piece reads 800 mm^3, so the face area is the measure. A tool that misses stays a
    /// node, and the read carries `operation failed: Trim`. That trim stays in the body list as the cap,
    /// one face of 200 mm^2. A point that is not finite is `error-trim-keep` before a node. A solid asked
    /// as the sheet is `error-trim-sheet` before a node: asked of the kernel, the box is consumed and the
    /// result is a sheet of volume 0. A refused cut named as its own tool, or a body that stands later,
    /// is `error-trim-tool` before a rewrite. Opened again, that tool is not built yet, and the read had
    /// changed to `error-source-body-not-built`. The call does not ask the kernel first, and it does not
    /// run settle. The sheet is consumed. The tool is not.
    pub fn trim(&mut self, asked: Trim) -> Result<BodyId, DrawError> {
        if !asked.keep.x.mm.is_finite() || !asked.keep.y.mm.is_finite() || !asked.keep.z.mm.is_finite() {
            return Err(DrawError { text: "error-trim-keep".into() });
        }
        if let Some(text) = source_has_no_solid(&self.project, asked.sheet.id) {
            return Err(DrawError { text: text.into() });
        }
        if !is_sheet(&self.project, asked.sheet.id) {
            return Err(DrawError { text: "error-trim-sheet".into() });
        }
        if let Some(text) = source_has_no_solid(&self.project, asked.tool.id) {
            return Err(DrawError { text: text.into() });
        }
        if body_is_taken(&self.project, asked.sheet.id) {
            return Err(DrawError { text: "msg-body-consumed".into() });
        }
        if tool_is_built_later(&self.project, asked) {
            return Err(DrawError { text: "error-trim-tool".into() });
        }
        if let Some(id) = replace_refused_trim(&mut self.project, asked) {
            return Ok(BodyId { id });
        }
        if let Some(text) = refusal_holds(&self.project, asked.sheet.id) {
            return Err(DrawError { text: text.into() });
        }
        if let Some(src) = sheet_came_from(&self.project, asked.sheet.id) {
            if let Some(text) = refusal_holds(&self.project, src) {
                return Err(DrawError { text: text.into() });
            }
        }
        let id = self.project.add_trim(asked.sheet.id, asked.tool.id, [asked.keep.x.mm, asked.keep.y.mm, asked.keep.z.mm]);
        Ok(BodyId { id })
    }

    /// Replace one face of `asked.body` with `asked.surface` and return that node.
    ///
    /// On the 10 by 20 by 30 mm box the end cap, replaced by that face lifted 10 mm, is a solid of
    /// 7999.999999999997 mm^3. Sunk 10 mm it is 3999.999999999999 mm^3. The cap's own copy put back
    /// stays 6000 mm^3. A wall that does not meet the cap stays a node, and the read carries
    /// `surface leaves 6 free edges`. That result is absent, and the box stays 6000 mm^3, because a
    /// replacement that never built consumes nothing. A face id of 0 is `error-faces-not-found` before
    /// a node. A sheet asked as the solid is `error-replace-body` before a node: asked of the kernel,
    /// the read is `operation failed: ReplaceFaces` and nothing is consumed. A solid asked as the sheet
    /// is `error-replace-sheet` before a node: asked of the kernel, the read is `surface leaves 4 free
    /// edges` and the box stays. Those refusals happen before a refused replacement is rewritten, so a
    /// face id of 0 does not replace a replacement the rebuild has already refused. A sheet built after
    /// that replacement is `error-replace-built` before a rewrite. Left in place, the live read was
    /// quiet at 6000 mm^3, and opened again it was `error-source-body-not-built`. A replacement the
    /// rebuild has refused is that same node again, whether the caller names the box or the
    /// replacement's own id. The replacement's own id never built, so it is not treated as a missing
    /// solid. A fillet the rebuild has refused is left as it is. The call does not ask the kernel
    /// first, and it does not run settle. The solid and the sheet are consumed. The result is not.
    pub fn replace_face(&mut self, asked: ReplaceFace) -> Result<BodyId, DrawError> {
        if asked.face.id == 0 {
            return Err(DrawError { text: "error-faces-not-found".into() });
        }
        if !names_a_refused_replacement(&self.project, asked.body.id) {
            if let Some(text) = source_has_no_solid(&self.project, asked.body.id) {
                return Err(DrawError { text: text.into() });
            }
            if is_sheet(&self.project, asked.body.id) {
                return Err(DrawError { text: "error-replace-body".into() });
            }
        }
        if let Some(text) = source_has_no_solid(&self.project, asked.surface.id) {
            return Err(DrawError { text: text.into() });
        }
        if !is_sheet(&self.project, asked.surface.id) {
            return Err(DrawError { text: "error-replace-sheet".into() });
        }
        if body_is_taken(&self.project, asked.body.id) {
            return Err(DrawError { text: "msg-body-consumed".into() });
        }
        if sheet_is_built_later(&self.project, asked) {
            return Err(DrawError { text: "error-replace-built".into() });
        }
        if let Some(id) = replace_refused_surface(&mut self.project, asked) {
            return Ok(BodyId { id });
        }
        if let Some(text) = refusal_holds(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        if let Some(src) = sheet_came_from(&self.project, asked.surface.id) {
            if let Some(text) = refusal_holds(&self.project, src) {
                return Err(DrawError { text: text.into() });
            }
        }
        let id = self.project.add_surface_replace(asked.body.id, qymcad_core::refs::Ref::picks(&[asked.face.id]), asked.surface.id);
        Ok(BodyId { id })
    }

    /// Move `asked.body` by `asked.by` and return that node.
    ///
    /// The placement is a 3x4 row-major matrix. Index 3 is the shift along X, in millimetres. On the
    /// 10 by 20 by 30 mm box a shift of 100 mm leaves the body at 6000 mm^3. A 2 mm cube added after
    /// it is a union of 6008 mm^3 and two solids. The same box moved by the identity still meets that
    /// cube, and the union is 6006.000000000001 mm^3 and one solid. A placement that is not finite is
    /// `error-move-place` before a node and before a refused move is rewritten. A matrix of all zeros
    /// is finite, so it is a node, and the read is `operation failed: Move`. That result still has a
    /// mesh. A move the rebuild has refused is that same node again, whether the caller names the box
    /// or the move's own id. The rewrite stores the placement and does not store the node's own id as
    /// the body. A fillet the rebuild has refused is left as it is. The call does not run settle. The
    /// body is consumed. The result is not.
    pub fn move_body(&mut self, asked: Move) -> Result<BodyId, DrawError> {
        if asked.by.mat.iter().any(|value| !value.is_finite()) {
            return Err(DrawError { text: "error-move-place".into() });
        }
        if let Some(text) = source_has_no_solid(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        if body_is_taken(&self.project, asked.body.id) {
            return Err(DrawError { text: "msg-body-consumed".into() });
        }
        if let Some(id) = replace_refused_move(&mut self.project, asked) {
            return Ok(BodyId { id });
        }
        if let Some(text) = refusal_holds(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        let id = self.project.add_move(asked.body.id, asked.by.mat);
        Ok(BodyId { id })
    }

    /// Add a part at `asked.place` and stand in it.
    ///
    /// The placement is a 3x4 row-major matrix in the parent, and it does not move a body. On a
    /// document that already holds the 10 by 20 by 30 mm box, a shift of 100 mm adds `name-part-n#2`
    /// under the root. The twelve numbers are `1, 0, 0, 100, 0, 1, 0, 0, 0, 0, 1, 0`. A box drawn
    /// next is 6000 mm^3 and belongs to the new part. The first box stays 6000 mm^3 in the first
    /// part. The two do not join. A placement that is not finite is `error-part-place` before a part
    /// is added. The call does not run settle.
    pub fn part(&mut self, asked: Part) -> Result<PartId, DrawError> {
        if asked.place.mat.iter().any(|value| !value.is_finite()) {
            return Err(DrawError { text: "error-part-place".into() });
        }
        let name = self.project.free_part_name();
        let id = self.project.add_part(name);
        self.project.set_component_transform(id, asked.place.mat);
        self.project.set_active_component(Some(id));
        Ok(PartId { id })
    }

    /// Fasten the origins of two parts and return that mate.
    ///
    /// The part that stays is the first anchor. The other is brought to it when the document is read,
    /// not by this call. On the 10 by 20 by 30 mm box and a second part shifted 100 mm along X, that
    /// read leaves the first part on the identity. The second is within 1e-6 mm of the identity: the
    /// shift along X that remained was 7.28e-14 mm, and after the file was opened it was 1e-16 mm.
    /// Both boxes stay 6000 mm^3 and one solid each. The mate's name is `name-joint-kind-rigid-n#1`.
    /// A side that is not a part is `error-mate-part` before a connector. The same part on both sides
    /// is `error-mate-same` before a connector. The call does not ground a part, does not change which
    /// part the session stands in, and does not run settle.
    pub fn mate(&mut self, asked: Mate) -> Result<MateId, DrawError> {
        if !self.project.component_is_part(asked.stay.part.id) || !self.project.component_is_part(asked.bring.part.id) {
            return Err(DrawError { text: "error-mate-part".into() });
        }
        if asked.stay.part.id == asked.bring.part.id {
            return Err(DrawError { text: "error-mate-same".into() });
        }
        let stay = self.project.add_connector(asked.stay.part.id, qymcad_core::feature::AnchorRef::Origin);
        let bring = self.project.add_connector(asked.bring.part.id, qymcad_core::feature::AnchorRef::Origin);
        let id = self.project.add_joint(stay, bring, qymcad_core::feature::JointKind::Rigid);
        Ok(MateId { id })
    }

    /// Fasten the origins of two parts with a slider and set its travel along Z.
    ///
    /// The part that stays is the first anchor. The travel is the slider's offset. On the 10 by 20 by 30 mm
    /// box and a second part shifted 100 mm along X, a rigid mate and then a travel of 25 mm, before any
    /// read, leave both mates unheld. The brought part stays at +100 mm along X. The slider is named
    /// `name-joint-kind-slider-n#2`. The read is quiet and every component still has 6 ways to move. A side
    /// that is not a part is `error-mate-part` before a connector. The same part on both sides is
    /// `error-mate-same` before a connector. A travel that is not finite is `error-slide-travel` before a
    /// connector. The call does not ground a part, does not change which part the session stands in, and
    /// does not run settle.
    pub fn slide(&mut self, asked: Slide) -> Result<MateId, DrawError> {
        if !self.project.component_is_part(asked.stay.part.id) || !self.project.component_is_part(asked.bring.part.id) {
            return Err(DrawError { text: "error-mate-part".into() });
        }
        if asked.stay.part.id == asked.bring.part.id {
            return Err(DrawError { text: "error-mate-same".into() });
        }
        if !asked.by.mm.is_finite() {
            return Err(DrawError { text: "error-slide-travel".into() });
        }
        let stay = self.project.add_connector(asked.stay.part.id, qymcad_core::feature::AnchorRef::Origin);
        let bring = self.project.add_connector(asked.bring.part.id, qymcad_core::feature::AnchorRef::Origin);
        let id = self.project.add_joint(stay, bring, qymcad_core::feature::JointKind::Slider);
        // Slot 1 is the slider's only free slot, and that slot is the shift along Z.
        let _stored = self.project.set_joint_drive(id, 1, Some(asked.by.mm));
        Ok(MateId { id })
    }

    /// Cut a metric thread on the circular rim of `asked.body` and return that node.
    ///
    /// The rim is a descriptor from the last read. The rebuild reads the centre, the axis, and the radius
    /// from that edge. On the diameter 10 mm rod, 25 mm long, an M10x1.5 thread of 20 mm leaves a mesh of
    /// 1681.40 mm^3. The kernel integral of that helix reads 1686.45 mm^3, so the mesh is the measure. A
    /// rim id of 0 is `error-edges-not-found` before a node. A length that is not finite is
    /// `error-thread-length` before a node. A length of 0 or below is `error-thread-length-unset` before a
    /// node. A diameter that is not a positive finite number is `error-thread-diameter` before a node. A
    /// pitch below 0, or a pitch that is not finite, is `error-thread-pitch` before a node. A pitch of 0
    /// is the coarse pitch. Those refusals happen before a refused thread is rewritten. A thread the
    /// rebuild has refused is that same node again, whether the caller names the rod or the thread's own
    /// id. A fillet the rebuild has refused is left as it is. The call does not ask the kernel first: a
    /// thread of 30 mm on the 25 mm rod stays a node, and the read carries `a thread of 30.0 on a face
    /// 25.0 long`. The shape appears when the document is rebuilt. The returned id is the thread.
    pub fn thread(&mut self, asked: Thread) -> Result<BodyId, DrawError> {
        if asked.rim.id == 0 {
            return Err(DrawError { text: "error-edges-not-found".into() });
        }
        if !asked.length.mm.is_finite() {
            return Err(DrawError { text: "error-thread-length".into() });
        }
        if asked.length.mm <= 1e-6 {
            return Err(DrawError { text: "error-thread-length-unset".into() });
        }
        if !(asked.diameter.mm.is_finite() && asked.diameter.mm > 0.0) {
            return Err(DrawError { text: "error-thread-diameter".into() });
        }
        if !(asked.pitch.mm.is_finite() && asked.pitch.mm >= 0.0) {
            return Err(DrawError { text: "error-thread-pitch".into() });
        }
        if body_is_taken(&self.project, asked.body.id) {
            return Err(DrawError { text: "msg-body-consumed".into() });
        }
        if let Some(id) = replace_refused_thread(&mut self.project, asked) {
            return Ok(BodyId { id });
        }
        if let Some(text) = refusal_holds(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        if let Some(text) = source_has_no_solid(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        let id = self.project.add_thread(asked.body.id, asked.rim.id, metric_spec(asked.diameter, asked.pitch), asked.length.mm, 0.0, 0.0);
        Ok(BodyId { id })
    }

    /// Weld one flight onto the circular rim of `asked.body` and return that node.
    ///
    /// The rim is a descriptor from the last read. The rebuild reads the centre, the axis, and the radius
    /// from that edge, and it sets the shaft diameter from that radius. On the diameter 10 mm rod, 25 mm
    /// long, a flight of outer diameter 30 mm, pitch 20 mm, thickness 3 mm, and length 20 mm leaves a mesh
    /// of 3552.88 mm^3. The kernel integral of that helix reads 3565.71 mm^3, so the mesh is the measure.
    /// A rim id of 0 is `error-edges-not-found` before a node. A length that is not finite is
    /// `error-auger-length` before a node. A length of 0 or below is `error-auger-bad-pitch-or-length`
    /// before a node. An outer diameter that is not a positive finite number is `error-auger-outer` before
    /// a node. A pitch that is not finite is `error-auger-pitch` before a node. A pitch of 0 or below is
    /// `error-auger-bad-pitch-or-length` before a node. A thickness that is not a positive finite number is
    /// `error-auger-thickness` before a node. Those refusals happen before a refused flight is rewritten.
    /// A flight the rebuild has refused is that same node again, whether the caller names the rod or the
    /// flight's own id. A thread the rebuild has refused is left as it is. The call does not ask the kernel
    /// first: an outer diameter of 8 mm on the 10 mm rod stays a node, and the read carries
    /// `auger outer 8.0 not bigger than shaft 10.0`. The shape appears when the document is rebuilt. The
    /// returned id is the flight.
    pub fn auger(&mut self, asked: Auger) -> Result<BodyId, DrawError> {
        if asked.rim.id == 0 {
            return Err(DrawError { text: "error-edges-not-found".into() });
        }
        if !asked.length.mm.is_finite() {
            return Err(DrawError { text: "error-auger-length".into() });
        }
        if asked.length.mm <= 1e-6 {
            return Err(DrawError { text: "error-auger-bad-pitch-or-length".into() });
        }
        if !(asked.outer.mm.is_finite() && asked.outer.mm > 0.0) {
            return Err(DrawError { text: "error-auger-outer".into() });
        }
        if !asked.pitch.mm.is_finite() {
            return Err(DrawError { text: "error-auger-pitch".into() });
        }
        if asked.pitch.mm <= 1e-6 {
            return Err(DrawError { text: "error-auger-bad-pitch-or-length".into() });
        }
        if !(asked.thickness.mm.is_finite() && asked.thickness.mm > 0.0) {
            return Err(DrawError { text: "error-auger-thickness".into() });
        }
        if body_is_taken(&self.project, asked.body.id) {
            return Err(DrawError { text: "msg-body-consumed".into() });
        }
        if let Some(id) = replace_refused_auger(&mut self.project, asked) {
            return Ok(BodyId { id });
        }
        if let Some(text) = refusal_holds(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        if let Some(text) = source_has_no_solid(&self.project, asked.body.id) {
            return Err(DrawError { text: text.into() });
        }
        let id = self.project.add_auger(asked.body.id, asked.rim.id, auger_spec(asked), asked.length.mm, 0.0, 0.0);
        Ok(BodyId { id })
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

/// A length the size fields accept: finite and greater than zero.
fn above_zero(mm: f64) -> bool {
    mm.is_finite() && mm > 0.0
}

/// The body the part keeps after `made` is added.
///
/// The first solid is returned as itself. A later solid is united with the one already there, so the part
/// stays one body. A turn whose profile crosses its axis does not unite: on a 10 by 20 by 30 mm box the read
/// came back `error-revolve-profile-crosses-axis error-source-body-not-built`, and a union node stood where
/// the box had been the body. A solid the rebuild has refused, or a turn that will be refused the same way,
/// is not a body to unite with. The solid before that one is.
fn settle(project: &mut Project, kernel: &mut OcctKernel, made: u64) -> u64 {
    // An unread draft of 90 degrees, then a 2 mm box, wrote a union. The read was
    // `draft 90.0 failed error-source-body-not-built`, and the id handed back had no shape.
    // The refusal is known only once that draft has been built. A 10 degree draft that has
    // not been read still joins, so the part stays one body.
    if join_target(project, made).is_some_and(|prior| shape_missing(project, prior)) {
        project.eval_parameters();
        kernel.quality_k = project.geom_quality.deflection_k();
        let gate = qymcad_kernel::kernel_gate();
        let _report = project.regenerate(kernel);
        drop(gate);
    }
    match join_target(project, made) {
        Some(prior) => project.add_body_boolean(prior, made, 1),
        None => made,
    }
}

/// The earlier solid of the same part that `made` should join, when the part already has one.
fn join_target(project: &Project, made: u64) -> Option<u64> {
    if will_not_build(project, made) {
        return None;
    }
    let owner = project.body_owner(made)?;
    let place = project.timeline.iter().position(|node| node.id == made)?;
    let consumed = consumed_before(project, place);
    project.timeline.iter().take(place).rev().flat_map(|node| node.kind.bodies()).find(|body| {
        *body != made && !consumed.contains(body) && !is_sheet(project, *body) && project.body_owner(*body) == Some(owner) && !will_not_build(project, *body) && !unproven_cut(project, *body)
    })
}

/// `body` has not been rebuilt, so a refusal it will earn is not known yet.
fn shape_missing(project: &Project, body: u64) -> bool {
    project.bodies.iter().find(|item| item.id == body).is_none_or(|item| item.mesh.tris.is_empty())
}

/// Bodies hidden by a node above `place`. The node just added has not united anything yet.
fn consumed_before(project: &Project, place: usize) -> std::collections::HashSet<u64> {
    project.timeline.iter().take(place).flat_map(|node| node.kind.consumed()).collect()
}

/// A cut or an intersection that has not been built is not a body to join.
///
/// An unread cut of a 40 by 30 by 10 mm box by a tool that misses it, then a 2 mm box, came back
/// `error-cut-removed-nothing error-source-body-not-built`, and a union node stood on the cut. A union
/// the part adds for a new solid is a different node: that one is op 1, and a later solid still joins it.
fn unproven_cut(project: &Project, body: u64) -> bool {
    let Some(node) = project.timeline.iter().find(|node| node.kind.bodies().contains(&body)) else { return false };
    let qymcad_core::feature::FeatureKind::BodyBoolean { op, .. } = node.kind else { return false };
    op != 1 && project.bodies.iter().find(|item| item.id == body).is_none_or(|item| item.mesh.tris.is_empty())
}

fn is_sheet(project: &Project, body: u64) -> bool {
    project.bodies.iter().any(|item| item.id == body && item.sheet)
}

/// `body` is a node the rebuild has refused, or a turn whose profile crosses the axis. Uniting with it
/// leaves the read with `error-source-body-not-built`.
fn will_not_build(project: &Project, body: u64) -> bool {
    let Some(node) = project.timeline.iter().find(|node| node.id == body || node.kind.bodies().contains(&body)) else { return false };
    project.regen_errors.contains_key(&node.id) || turn_crosses(project, &node.kind)
}

fn turn_crosses(project: &Project, kind: &qymcad_core::feature::FeatureKind) -> bool {
    let qymcad_core::feature::FeatureKind::Revolve { sketch, profiles, axis, axis_datum, axis_line, .. } = kind else { return false };
    let contours = if profiles.is_empty() {
        project.sketches.iter().find(|item| item.id == *sketch).map(|item| item.contour_ids.clone()).unwrap_or_default()
    } else {
        profiles.clone()
    };
    let ax = qymcad_core::model::RevolveAxis { axis: *axis, datum: *axis_datum, line: *axis_line };
    let (origin, dir, _) = project.revolve_axis_local(*sketch, ax);
    contours.iter().any(|contour| project.contour_profile_xy(*contour).is_some_and(|xy| project.revolve_profile_crosses_axis(&xy, origin, dir).is_some()))
}

/// The file layer takes a Unicode path. Anything else is refused here, before a replacement character
/// can turn it into a different file that then succeeds or fails on its own.
fn unicode_path(path: &std::path::Path) -> Result<&str, StoreError> {
    path.to_str().ok_or_else(|| StoreError { text: "io-path-not-utf8".into() })
}

fn view_of(project: &Project, kernel: &OcctKernel, report: &qymcad_core::feature::RegenReport) -> Document {
    Document {
        components: project
            .components
            .iter()
            .map(|c| Component { id: c.id, name: c.name.clone(), kind: kind_of(c.kind), parent: c.parent, place: Place { mat: c.transform }, freedom: Freedom { count: project.component_dof(c.id) } })
            .collect(),
        sketches: project.sketches.iter().map(|s| sketch_of(project, s)).collect(),
        nodes: project.timeline.iter().map(|n| Node { id: n.id, name: n.name.clone(), parent: n.parent }).collect(),
        bodies: bodies_of(project, kernel),
        mates: mates_of(project),
        violated: violated_of(project),
        rebuild: rebuild_of(project, report),
    }
}

fn violated_of(project: &Project) -> Vec<MateId> {
    project.mates_violated.iter().copied().map(|id| MateId { id }).collect()
}

fn mates_of(project: &Project) -> Vec<Fastened> {
    project
        .joints
        .iter()
        .map(|joint| {
            let stay = project.connector(joint.a).map(|connector| connector.owner).unwrap_or(0);
            let bring = project.connector(joint.b).map(|connector| connector.owner).unwrap_or(0);
            Fastened { id: joint.id, name: joint.name.clone(), stay: Stay { part: PartId { id: stay } }, bring: Bring { part: PartId { id: bring } } }
        })
        .collect()
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
                mesh: Volume { mm3: body.mesh.volume() },
            },
            None => Body { id: body.id, name: body.name.clone(), sound: Sound::Invalid, measure: Measure::Absent, faces: Vec::new(), edges: Vec::new(), mesh: Volume { mm3: 0.0 } },
        })
        .collect()
}

/// The corners of one mesh. A mesh with no vertex is not reported as the origin: that corner would be a lie.
fn read_mesh(mesh: &qymcad_core::geom::Mesh) -> Result<MeshRead, StoreError> {
    let triangles = u32::try_from(mesh.tris.len()).map_err(|_| StoreError { text: "io-stl-too-many-triangles".into() })?;
    let Some(bounds) = mesh.bounds() else {
        return Err(StoreError { text: "io-stl-no-faces".into() });
    };
    Ok(MeshRead {
        volume: Volume { mm3: mesh.volume() },
        triangles,
        low: LowCorner { x: CornerX { mm: bounds.min.x }, y: CornerY { mm: bounds.min.y }, z: CornerZ { mm: bounds.min.z } },
        high: HighCorner { x: CornerX { mm: bounds.max.x }, y: CornerY { mm: bounds.max.y }, z: CornerZ { mm: bounds.max.z } },
    })
}

/// One solid and its corners. A missing box is not reported as the origin: that corner would be a lie.
fn read_solid(shape: &qymcad_kernel::Shape) -> Result<SolidRead, StoreError> {
    let Some(bounds) = shape.bbox() else {
        return Err(StoreError { text: "cad-step-no-shapes".into() });
    };
    Ok(SolidRead {
        sound: sound_of(shape.is_valid()),
        measure: Measure::Live { volume: Volume { mm3: shape.volume() }, solids: shape.solid_count() },
        low: LowCorner { x: CornerX { mm: bounds[0] }, y: CornerY { mm: bounds[1] }, z: CornerZ { mm: bounds[2] } },
        high: HighCorner { x: CornerX { mm: bounds[3] }, y: CornerY { mm: bounds[4] }, z: CornerZ { mm: bounds[5] } },
    })
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

/// Two ends that stay apart after the sketch merges points that are this close. A coordinate that is not
/// finite is not an end either.
fn spans_a_segment(segment: &Segment) -> bool {
    let (x0, y0) = segment.from.at;
    let (x1, y1) = segment.to.at;
    [x0, y0, x1, y1].iter().all(|value| value.is_finite()) && (x1 - x0).hypot(y1 - y0) > POINT_MERGE_MM
}

/// The path contour is one the path sketch holds. A contour id of 0 is not that contour: the model reads
/// 0 as the first path.
fn named_path(project: &Project, path: Path) -> bool {
    path.contour.id != 0 && project.sketches.iter().any(|sketch| sketch.id == path.sketch.id && sketch.contour_ids.contains(&path.contour.id)) && contour_of(project, path.contour.id).is_some()
}

/// The path contour stays open. A closed one is a loop the sweep cannot follow: a 2 mm square swept along
/// a 20 mm square on the front plane, beside the 10 by 20 by 30 mm box, came back `error-broken-solid
/// error-source-body-not-built`, and a union stood on the sweep.
fn path_is_open(project: &Project, path: Path) -> bool {
    contour_of(project, path.contour.id).is_some_and(|contour| contour.form == Form::Open)
}

/// Where a sketch lies: a world plane, or a datum offset from one. Two sketches on that same plane are
/// not a loft. Two squares on the table, lofted beside the 10 by 20 by 30 mm box, came back
/// `operation failed: Loft error-source-body-not-built` and a union.
fn same_plane(project: &Project, left: u64, right: u64) -> bool {
    match (plane_place(project, left), plane_place(project, right)) {
        (Some(left), Some(right)) => left.base == right.base && (left.dist - right.dist).abs() < 1e-9,
        _ => false,
    }
}

struct PlanePlace {
    base: qymcad_core::feature::BasePlane,
    dist: f64,
}

/// An offset datum this session raised. An id of 0 is not one, and neither is a sketch or a body.
fn raised_plane(project: &Project, id: u64) -> bool {
    project.planes.iter().any(|item| item.id == id && matches!(item.def, qymcad_core::model::PlaneDef::OffsetBase { .. }))
}

/// The frame of a world plane, or of a datum offset from one, without reading the plane's stored origin.
///
/// That origin stays at the world origin until the document is rebuilt. The offset is the distance stored
/// on the datum.
fn shifted(place: PlanePlace) -> qymcad_core::feature::PlaneFrame {
    let mut frame = place.base.frame();
    let normal = frame.normal();
    frame.origin = [normal[0] * place.dist, normal[1] * place.dist, normal[2] * place.dist];
    frame
}

/// Both sections reach the line where their planes cross.
///
/// A 10 mm square on the table and a 10 mm square on the front plane, both from the origin, lofted beside
/// the 10 by 20 by 30 mm box, came back `operation failed: Loft error-source-body-not-built` and a union.
/// A front square that starts 0.05 mm above the table does not reach that line, and the loft stayed 502.5 mm^3.
fn sections_meet(project: &Project, asked: Loft) -> bool {
    let (Some(lower), Some(upper)) = (plane_place(project, asked.lower.profile.sketch.id), plane_place(project, asked.upper.profile.sketch.id)) else {
        return false;
    };
    let lower_frame = shifted(lower);
    let upper_frame = shifted(upper);
    let (left, right) = (lower_frame.normal(), upper_frame.normal());
    let dot = left[0] * right[0] + left[1] * right[1] + left[2] * right[2];
    if dot.abs() > 1.0 - 1e-9 {
        return false;
    }
    reaches(project, asked.lower.profile, upper_frame) && reaches(project, asked.upper.profile, lower_frame)
}

/// `profile` has a point on `plane`, or points on both sides of it. The points are the contour's own, lifted
/// through the sketch frame.
fn reaches(project: &Project, profile: Profile, plane: qymcad_core::feature::PlaneFrame) -> bool {
    let Some(place) = plane_place(project, profile.sketch.id) else { return false };
    let frame = shifted(place);
    let Some(index) = project.contour_index(profile.contour.id) else { return false };
    let mut low = f64::INFINITY;
    let mut high = f64::NEG_INFINITY;
    for point in &project.contours[index].points {
        let side = plane.depth(frame.lift(*point));
        low = low.min(side);
        high = high.max(side);
    }
    low.is_finite() && low <= POINT_MERGE_MM && high >= -POINT_MERGE_MM
}

fn plane_place(project: &Project, sketch: u64) -> Option<PlanePlace> {
    let placed = project.sketches.iter().find(|item| item.id == sketch)?.plane;
    match placed {
        qymcad_core::feature::SketchPlane::World(base) => Some(PlanePlace { base, dist: 0.0 }),
        qymcad_core::feature::SketchPlane::Datum(id) => {
            let plane = project.planes.iter().find(|item| item.id == id)?;
            match plane.def {
                qymcad_core::model::PlaneDef::OffsetBase { base, dist } => Some(PlanePlace { base, dist }),
                _ => None,
            }
        }
        qymcad_core::feature::SketchPlane::Face(_, _) => None,
    }
}

/// How many pieces a plane cuts from a body that has already been rebuilt.
enum CutCount {
    /// The rebuild left no shape for the body.
    Missing,
    /// The plane misses the body, or lies on a face, so fewer than two pieces come back.
    Short,
    /// The plane cuts the body into this many pieces.
    Pieces(usize),
}

/// How many pieces `plane` at `offset` cuts from `body`.
///
/// The origin and the normal are the ones `op_plane` uses for a world plane, shifted by the offset
/// along that normal. The probe passes no section name. Parameter expressions are evaluated first,
/// the same way a read does, so an unread box has a shape. The cut is asked through the kernel's
/// public probe, which copies the body and takes the kernel lock itself. This function therefore
/// lets the lock go after the rebuild: holding it across that probe would wait on the same lock.
fn counted_pieces(project: &mut Project, kernel: &mut OcctKernel, body: u64, plane: WorldPlane, offset: f64) -> CutCount {
    project.eval_parameters();
    kernel.quality_k = project.geom_quality.deflection_k();
    {
        let _gate = qymcad_kernel::kernel_gate();
        let _rebuilt = project.regenerate(kernel);
    }
    let shapes = kernel.shapes.borrow();
    let Some(shape) = shapes.get(&body) else {
        return CutCount::Missing;
    };
    let at = match plane {
        WorldPlane::Xy => qymcad_core::feature::PlaneAt { origin: [0.0, 0.0, offset], normal: [0.0, 0.0, 1.0] },
        WorldPlane::Xz => qymcad_core::feature::PlaneAt { origin: [0.0, offset, 0.0], normal: [0.0, 1.0, 0.0] },
        WorldPlane::Yz => qymcad_core::feature::PlaneAt { origin: [offset, 0.0, 0.0], normal: [1.0, 0.0, 0.0] },
    };
    match qymcad_kernel::ask::split_pieces(shape, at) {
        Some(n) => CutCount::Pieces(n),
        None => CutCount::Short,
    }
}

/// Whether a plane divides a face of a body that has already been rebuilt.
enum FaceCut {
    /// The rebuild left no shape for the body.
    Missing,
    /// The plane passes clear of the body, so no face is divided.
    Clear,
    /// The plane divides at least one face.
    Crossed,
}

/// Whether `plane` at `offset` divides a face of `body`.
///
/// The origin and the normal are the ones `op_plane` uses for a world plane, shifted by the offset
/// along that normal. The section is asked of a copy, through the same probe the window uses, and
/// that probe takes the kernel lock itself. This function lets the lock go after the rebuild.
fn faces_split(project: &mut Project, kernel: &mut OcctKernel, body: u64, plane: WorldPlane, offset: f64) -> FaceCut {
    project.eval_parameters();
    kernel.quality_k = project.geom_quality.deflection_k();
    {
        let _gate = qymcad_kernel::kernel_gate();
        let _rebuilt = project.regenerate(kernel);
    }
    let shapes = kernel.shapes.borrow();
    let Some(shape) = shapes.get(&body) else {
        return FaceCut::Missing;
    };
    let at = match plane {
        WorldPlane::Xy => qymcad_core::feature::PlaneAt { origin: [0.0, 0.0, offset], normal: [0.0, 0.0, 1.0] },
        WorldPlane::Xz => qymcad_core::feature::PlaneAt { origin: [0.0, offset, 0.0], normal: [0.0, 1.0, 0.0] },
        WorldPlane::Yz => qymcad_core::feature::PlaneAt { origin: [offset, 0.0, 0.0], normal: [1.0, 0.0, 0.0] },
    };
    if qymcad_kernel::ask::splits_faces(shape, at) {
        FaceCut::Crossed
    } else {
        FaceCut::Clear
    }
}

/// Mark the node that builds `body`, so the next read rebuilds it from its recipe.
///
/// A piece count and a face section each ask a copy and leave the cached shape. A clean node is
/// skipped, so the next read would keep the shape from before the edit the question followed.
fn mark_body_dirty(project: &mut Project, body: u64) {
    let node = project.timeline.iter().find(|node| node.kind.bodies().contains(&body)).map(|node| node.id);
    if let Some(node) = node {
        project.mark_node_dirty(node);
    }
}

/// Whether `body` can be one side of a boolean, or the source of a mirror or a pattern, asked before the first read.
///
/// A mesh is not required. The id has to be a body on the timeline, and a node the rebuild has not
/// refused must not already have taken it. A fillet the rebuild has refused still holds its source:
/// mirroring that 10 by 20 by 30 mm extrusion came back `fillet R100.00 failed on 1 edges` with a mirror
/// of 12000 mm^3 beside it. Laying it out wrote a row of 18000 mm^3, turning it wrote a ring of 24000 mm^3,
/// and cutting it wrote a cut node, each with that same fillet sentence left in the read.
fn boolean_side(project: &Project, body: u64) -> Option<&'static str> {
    if body == 0 {
        return Some("error-source-body-not-built");
    }
    if body_is_taken(project, body) {
        return Some("msg-body-consumed");
    }
    if let Some(text) = refusal_holds(project, body) {
        return Some(text);
    }
    if will_not_build(project, body) {
        return Some("error-source-body-not-built");
    }
    let is_body = project.timeline.iter().any(|node| node.kind.bodies().contains(&body));
    if !is_body {
        Some("error-source-body-deleted")
    } else {
        None
    }
}

/// Whether a linear step moves the copies off the original.
///
/// The length has to be greater than `POINT_MERGE_MM`. A shorter step lands the copies on the body they
/// copy, so the result is not a row. A component that is not finite is the same refusal.
fn step_moves(step: Step) -> bool {
    let along = [step.x.mm, step.y.mm, step.z.mm];
    along.iter().all(|mm| mm.is_finite()) && along.iter().map(|mm| mm * mm).sum::<f64>() > POINT_MERGE_MM * POINT_MERGE_MM
}

/// A new sketch on `plane`, in the part the session is already in.
///
/// An empty document is the root assembly. `part_to_draw_in` makes a part and stands in it, so the sketch
/// is not left on the assembly. A later sketch reuses that part.
fn begin_sketch(project: &mut Project, plane: qymcad_core::feature::SketchPlane) -> (usize, u64) {
    let _made = project.part_to_draw_in();
    let name = free_sketch_name(project);
    let si = project.new_sketch(&name);
    project.sketches[si].plane = plane;
    let sid = project.sketches[si].id;
    let _node = project.add_sketch_node(sid, &name);
    (si, sid)
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

/// A node that the rebuild has not refused already consumes `body`, even before that node has been read.
fn body_is_taken(project: &Project, body: u64) -> bool {
    project.timeline.iter().any(|node| node.kind.consumed().contains(&body) && !project.regen_errors.contains_key(&node.id))
}

/// A refusal that still holds `body`, so the other operation must not add a node beside it.
///
/// The read would keep the old refusal. Measured on the 10 by 20 by 30 mm box: a chamfer beside a refused
/// fillet left `fillet R100.00 failed on 1 edges` in the read, and a chamfer of that fillet's own id added
/// a node answered with `error-source-body-not-built`. A fillet beside a refused chamfer did the same with
/// `chamfer 100.00 too big`.
fn refusal_holds(project: &Project, body: u64) -> Option<&'static str> {
    let holder = project.timeline.iter().find(|node| project.regen_errors.contains_key(&node.id) && (node.kind.consumed().contains(&body) || node.id == body))?;
    if holder.kind.consumed().contains(&body) {
        Some("msg-body-consumed")
    } else {
        Some("error-source-body-not-built")
    }
}

/// `body` has no solid, so a fillet or a chamfer must not add a node on it.
///
/// A node standing on nothing stays in the read after a later call on the real solid. Measured on the
/// 10 by 20 by 30 mm box: a chamfer of an id that was never a body, then a 2 mm chamfer of the extrusion,
/// left two chamfer nodes. The read stayed on `error-source-body-not-built` while the real chamfer was
/// 5960 mm^3. A chamfer of the sketch id stayed on `error-source-body-deleted`.
fn source_has_no_solid(project: &Project, body: u64) -> Option<&'static str> {
    if project.mesh_index(body).is_some() {
        return None;
    }
    if body == 0 || project.timeline.iter().any(|node| node.kind.bodies().contains(&body)) {
        Some("error-source-body-not-built")
    } else {
        Some("error-source-body-deleted")
    }
}

/// The refused fillet of `body`, written again with `edge` and `radius`.
///
/// `body` may be the source the fillet consumed, or the fillet's own id. A new node beside the refused
/// one leaves the refusal in the read and a second body in the part. Naming the fillet's own id had done
/// that too, because that id is not a body the refusal built. Any earlier refused fillet of the same
/// source is removed, and the latest one keeps its id.
fn replace_refused_fillet(project: &mut Project, body: u64, edge: u32, radius: f64) -> Option<u64> {
    let errored: Vec<u64> = project.regen_errors.keys().copied().collect();
    let mut refused = project
        .timeline
        .iter()
        .filter(|node| matches!(node.kind, qymcad_core::feature::FeatureKind::Fillet { src, .. } if src == body) && errored.contains(&node.id))
        .map(|node| node.id)
        .collect::<Vec<_>>();
    if refused.is_empty() {
        let itself = project.timeline.iter().any(|node| node.id == body && matches!(node.kind, qymcad_core::feature::FeatureKind::Fillet { .. }) && errored.contains(&node.id));
        if itself {
            refused.push(body);
        }
    }
    let keep = refused.pop()?;
    for id in refused {
        let _gone = project.delete_feature_op(id);
    }
    let source = {
        let node = project.timeline.iter_mut().find(|node| node.id == keep)?;
        let source = match &mut node.kind {
            qymcad_core::feature::FeatureKind::Fillet { src, radius: stored, edges, .. } => {
                let source = *src;
                *stored = radius;
                *edges = qymcad_core::refs::Ref::picks(&[edge]);
                source
            }
            _ => return None,
        };
        node.dirty = true;
        source
    };
    store_picked_edge(project, keep, source, edge);
    Some(keep)
}

/// The refused chamfer of `body`, written again with `edge` and `distance`.
///
/// A fillet that the rebuild refused is not this node. Rewriting it would turn the rounding into a bevel.
fn replace_refused_chamfer(project: &mut Project, body: u64, edge: u32, distance: f64) -> Option<u64> {
    let errored: Vec<u64> = project.regen_errors.keys().copied().collect();
    let mut refused = project
        .timeline
        .iter()
        .filter(|node| matches!(node.kind, qymcad_core::feature::FeatureKind::Chamfer { src, .. } if src == body) && errored.contains(&node.id))
        .map(|node| node.id)
        .collect::<Vec<_>>();
    if refused.is_empty() {
        let itself = project.timeline.iter().any(|node| node.id == body && matches!(node.kind, qymcad_core::feature::FeatureKind::Chamfer { .. }) && errored.contains(&node.id));
        if itself {
            refused.push(body);
        }
    }
    let keep = refused.pop()?;
    for id in refused {
        let _gone = project.delete_feature_op(id);
    }
    let source = {
        let node = project.timeline.iter_mut().find(|node| node.id == keep)?;
        let source = match &mut node.kind {
            qymcad_core::feature::FeatureKind::Chamfer { src, dist, edges, .. } => {
                let source = *src;
                *dist = distance;
                *edges = qymcad_core::refs::Ref::picks(&[edge]);
                source
            }
            _ => return None,
        };
        node.dirty = true;
        source
    };
    store_picked_edge(project, keep, source, edge);
    Some(keep)
}

/// The edge snapshot a fillet or a chamfer records, written again for a node that already exists.
fn store_picked_edge(project: &mut Project, feature: u64, body: u64, edge: u32) {
    let witnessed = project.regen_edges.get(&body).and_then(|edges| edges.iter().find(|item| item.id == edge).map(|item| (item.mid, item.dir)));
    match witnessed {
        Some((at, dir)) => {
            project.edge_refs.insert(feature, vec![qymcad_core::model::ElemSnapshot { id: edge, at, dir }]);
        }
        None => {
            project.edge_refs.remove(&feature);
        }
    }
}

/// The refused shell of `body`, written again with `face` and `thickness`.
///
/// A fillet or a chamfer that the rebuild refused is not this node. The snapshot is taken from the shell's
/// source, including when the caller names the shell's own id.
fn replace_refused_shell(project: &mut Project, body: u64, face: u32, thickness: f64) -> Option<u64> {
    let errored: Vec<u64> = project.regen_errors.keys().copied().collect();
    let mut refused = project
        .timeline
        .iter()
        .filter(|node| matches!(node.kind, qymcad_core::feature::FeatureKind::Shell { src, .. } if src == body) && errored.contains(&node.id))
        .map(|node| node.id)
        .collect::<Vec<_>>();
    if refused.is_empty() {
        let itself = project.timeline.iter().any(|node| node.id == body && matches!(node.kind, qymcad_core::feature::FeatureKind::Shell { .. }) && errored.contains(&node.id));
        if itself {
            refused.push(body);
        }
    }
    let keep = refused.pop()?;
    for id in refused {
        let _gone = project.delete_feature_op(id);
    }
    let source = {
        let node = project.timeline.iter_mut().find(|node| node.id == keep)?;
        let source = match &mut node.kind {
            qymcad_core::feature::FeatureKind::Shell { src, thickness: stored, faces, .. } => {
                let source = *src;
                *stored = thickness;
                *faces = qymcad_core::refs::Ref::picks(&[face]);
                source
            }
            _ => return None,
        };
        node.dirty = true;
        source
    };
    store_picked_face(project, keep, source, face);
    Some(keep)
}

/// The cylinder a plain hole cuts. The diameter and the depth are different types, so one cannot be
/// passed where the other goes.
struct HoleCut {
    diameter: Diameter,
    depth: Depth,
}

/// The refused hole of `body`, written again with `face` and `cut`.
///
/// A fillet, a chamfer, or a shell that the rebuild refused is not this node. The centre is taken from
/// the hole's source, including when the caller names the hole's own id. A face that is not on that
/// source keeps the id and is not given another face's centre.
fn replace_refused_hole(project: &mut Project, body: u64, face: u32, cut: HoleCut) -> Option<u64> {
    let errored: Vec<u64> = project.regen_errors.keys().copied().collect();
    let mut refused = project
        .timeline
        .iter()
        .filter(|node| matches!(node.kind, qymcad_core::feature::FeatureKind::Hole { src, .. } if src == body) && errored.contains(&node.id))
        .map(|node| node.id)
        .collect::<Vec<_>>();
    if refused.is_empty() {
        let itself = project.timeline.iter().any(|node| node.id == body && matches!(node.kind, qymcad_core::feature::FeatureKind::Hole { .. }) && errored.contains(&node.id));
        if itself {
            refused.push(body);
        }
    }
    let keep = refused.pop()?;
    for id in refused {
        let _gone = project.delete_feature_op(id);
    }
    let source = {
        let node = project.timeline.iter().find(|node| node.id == keep)?;
        match &node.kind {
            qymcad_core::feature::FeatureKind::Hole { src, .. } => *src,
            _ => return None,
        }
    };
    let key = hole_face(project, source, face);
    let node = project.timeline.iter_mut().find(|node| node.id == keep)?;
    match &mut node.kind {
        qymcad_core::feature::FeatureKind::Hole { diameter, depth, face: stored, point, normal, .. } => {
            *diameter = cut.diameter.mm;
            *depth = cut.depth.mm;
            *stored = qymcad_core::refs::Ref::one(key.id, qymcad_core::refs::Fingerprint { centroid: key.centroid, normal: key.normal });
            *point = key.centroid;
            *normal = key.normal;
        }
        _ => return None,
    }
    node.dirty = true;
    Some(keep)
}

/// The body a hole of `body` is cut from. A refused hole remembers its source, including when the caller
/// names the hole's own id.
fn hole_source(project: &Project, body: u64) -> u64 {
    let errored: Vec<u64> = project.regen_errors.keys().copied().collect();
    project
        .timeline
        .iter()
        .find_map(|node| match &node.kind {
            qymcad_core::feature::FeatureKind::Hole { src, .. } if errored.contains(&node.id) && (node.id == body || *src == body) => Some(*src),
            _ => None,
        })
        .unwrap_or(body)
}

/// `face` is a face of `body`, and the last rebuild did not leave its centre.
fn named_face_has_no_centre(project: &Project, body: u64, face: u32) -> bool {
    let belongs = project.names.get(face).is_some_and(|recipe| recipe.feature == body);
    belongs && project.regen_faces.get(&body).and_then(|faces| faces.iter().find(|item| item.id == face)).is_none()
}

/// The face key `add_hole` stores: the centre and the normal of `face` on `body`, or zeros when this body
/// has no such face. A centre from another body would drill the wrong place once the name was lost.
fn hole_face(project: &Project, body: u64, face: u32) -> qymcad_core::feature::FaceKey {
    match project.regen_faces.get(&body).and_then(|faces| faces.iter().find(|item| item.id == face)) {
        Some(item) => qymcad_core::feature::FaceKey { index: 0, centroid: [item.centroid.x, item.centroid.y, item.centroid.z], normal: item.normal, id: face },
        None => qymcad_core::feature::FaceKey { index: 0, centroid: [0.0; 3], normal: [0.0; 3], id: face },
    }
}

/// The face snapshot a shell records, written again for a node that already exists.
///
/// The point is the face centroid and the direction is the face normal, the two fields the model's own
/// capture stores. The model's capture is private to that crate.
fn store_picked_face(project: &mut Project, feature: u64, body: u64, face: u32) {
    let witnessed = project.regen_faces.get(&body).and_then(|faces| faces.iter().find(|item| item.id == face).map(|item| ([item.centroid.x, item.centroid.y, item.centroid.z], item.normal)));
    match witnessed {
        Some((at, dir)) => {
            project.face_refs.insert(feature, vec![qymcad_core::model::ElemSnapshot { id: face, at, dir }]);
        }
        None => {
            project.face_refs.remove(&feature);
        }
    }
}

/// The node kept when a refused draft is written again, and the body its faces are read from.
struct DraftKeep {
    node: u64,
    source: u64,
}

/// The refused draft of `asked.body`, written again with the wall, the neutral face, and the angle.
///
/// `asked.body` may be the source the draft consumed, or the draft's own id. A fillet that the rebuild
/// refused is not this node. The pull stays the neutral face's own normal. The snapshots are taken from
/// the draft's source, including when the caller names the draft's own id.
fn replace_refused_draft(project: &mut Project, asked: Draft) -> Option<u64> {
    let errored: Vec<u64> = project.regen_errors.keys().copied().collect();
    let mut refused = project
        .timeline
        .iter()
        .filter(|node| matches!(node.kind, qymcad_core::feature::FeatureKind::Draft { src, .. } if src == asked.body.id) && errored.contains(&node.id))
        .map(|node| node.id)
        .collect::<Vec<_>>();
    if refused.is_empty() {
        let itself = project.timeline.iter().any(|node| node.id == asked.body.id && matches!(node.kind, qymcad_core::feature::FeatureKind::Draft { .. }) && errored.contains(&node.id));
        if itself {
            refused.push(asked.body.id);
        }
    }
    let keep = refused.pop()?;
    for id in refused {
        let _gone = project.delete_feature_op(id);
    }
    let source = {
        let node = project.timeline.iter_mut().find(|node| node.id == keep)?;
        let source = match &mut node.kind {
            qymcad_core::feature::FeatureKind::Draft { src, faces, neutral, angle, flip, .. } => {
                let source = *src;
                *faces = qymcad_core::refs::Ref::picks(&[asked.tilted.face.id]);
                *neutral = qymcad_core::refs::Ref::one(asked.neutral.face.id, qymcad_core::refs::Fingerprint::default());
                *angle = asked.angle.deg;
                *flip = false;
                source
            }
            _ => return None,
        };
        node.dirty = true;
        source
    };
    store_draft_faces(project, DraftKeep { node: keep, source }, &asked);
    Some(keep)
}

/// The face snapshots a draft records, written again for a node that already exists.
///
/// Both faces go in one list. A shell stores one face, and storing the neutral face on its own would
/// drop the wall.
fn store_draft_faces(project: &mut Project, keep: DraftKeep, asked: &Draft) {
    let witnessed = |face: u32| {
        project.regen_faces.get(&keep.source).and_then(|items| {
            items.iter().find(|item| item.id == face).map(|item| qymcad_core::model::ElemSnapshot { id: face, at: [item.centroid.x, item.centroid.y, item.centroid.z], dir: item.normal })
        })
    };
    let snaps = [witnessed(asked.tilted.face.id), witnessed(asked.neutral.face.id)].into_iter().flatten().collect::<Vec<_>>();
    if snaps.is_empty() {
        project.face_refs.remove(&keep.node);
    } else {
        project.face_refs.insert(keep.node, snaps);
    }
}

/// A push the rebuild refused, written again with the face and the distance just given.
///
/// The reference stores the centre and the normal from the last read, the same key the first
/// write keeps. A zero fingerprint would leave only the face number.
fn replace_refused_push(project: &mut Project, asked: Push) -> Option<u64> {
    let errored: Vec<u64> = project.regen_errors.keys().copied().collect();
    let mut refused = project
        .timeline
        .iter()
        .filter(|node| matches!(node.kind, qymcad_core::feature::FeatureKind::PushFace { src, .. } if src == asked.body.id) && errored.contains(&node.id))
        .map(|node| node.id)
        .collect::<Vec<_>>();
    if refused.is_empty() {
        let itself = project.timeline.iter().any(|node| node.id == asked.body.id && matches!(node.kind, qymcad_core::feature::FeatureKind::PushFace { .. }) && errored.contains(&node.id));
        if itself {
            refused.push(asked.body.id);
        }
    }
    let keep = refused.pop()?;
    for id in refused {
        let _gone = project.delete_feature_op(id);
    }
    let source = project.timeline.iter().find(|node| node.id == keep).and_then(|node| match node.kind {
        qymcad_core::feature::FeatureKind::PushFace { src, .. } => Some(src),
        _ => None,
    })?;
    let key = hole_face(project, source, asked.face.id);
    {
        let node = project.timeline.iter_mut().find(|node| node.id == keep)?;
        match &mut node.kind {
            qymcad_core::feature::FeatureKind::PushFace { face, dist, .. } => {
                *face = qymcad_core::refs::Ref::one(key.id, qymcad_core::refs::Fingerprint { centroid: key.centroid, normal: key.normal });
                *dist = asked.by.mm;
            }
            _ => return None,
        }
        node.dirty = true;
    }
    store_picked_face(project, keep, source, asked.face.id);
    Some(keep)
}

/// A thicken the rebuild refused, written again with the face and the thickness just given.
///
/// The face stored on the node is that face's id. The place is written again, because the first
/// write records it and a rewrite has to keep the same witness. The join id stays 0: the plate is
/// welded to the source, not to some other body.
fn replace_refused_thicken(project: &mut Project, asked: Thicken) -> Option<u64> {
    let errored: Vec<u64> = project.regen_errors.keys().copied().collect();
    let mut refused = project
        .timeline
        .iter()
        .filter(|node| matches!(node.kind, qymcad_core::feature::FeatureKind::Thicken { src, .. } if src == asked.body.id) && errored.contains(&node.id))
        .map(|node| node.id)
        .collect::<Vec<_>>();
    if refused.is_empty() {
        let itself = project.timeline.iter().any(|node| node.id == asked.body.id && matches!(node.kind, qymcad_core::feature::FeatureKind::Thicken { .. }) && errored.contains(&node.id));
        if itself {
            refused.push(asked.body.id);
        }
    }
    let keep = refused.pop()?;
    for id in refused {
        let _gone = project.delete_feature_op(id);
    }
    let source = project.timeline.iter().find(|node| node.id == keep).and_then(|node| match node.kind {
        qymcad_core::feature::FeatureKind::Thicken { src, .. } => Some(src),
        _ => None,
    })?;
    {
        let node = project.timeline.iter_mut().find(|node| node.id == keep)?;
        match &mut node.kind {
            qymcad_core::feature::FeatureKind::Thicken { face, thickness, join, .. } => {
                *face = asked.face.id;
                *thickness = asked.plate.mm;
                *join = 0;
            }
            _ => return None,
        }
        node.dirty = true;
    }
    store_picked_face(project, keep, source, asked.face.id);
    Some(keep)
}

/// A removal the rebuild refused, written again with the face just given.
///
/// The face stored on the node is that face's id, the same pick the first write stores. The place is
/// written again, because a later edit finds the face by the witness the rebuild would have recorded.
fn replace_refused_removal(project: &mut Project, asked: RemoveFace) -> Option<u64> {
    let errored: Vec<u64> = project.regen_errors.keys().copied().collect();
    let mut refused = project
        .timeline
        .iter()
        .filter(|node| matches!(node.kind, qymcad_core::feature::FeatureKind::RemoveFace { src, .. } if src == asked.body.id) && errored.contains(&node.id))
        .map(|node| node.id)
        .collect::<Vec<_>>();
    if refused.is_empty() {
        let itself = project.timeline.iter().any(|node| node.id == asked.body.id && matches!(node.kind, qymcad_core::feature::FeatureKind::RemoveFace { .. }) && errored.contains(&node.id));
        if itself {
            refused.push(asked.body.id);
        }
    }
    let keep = refused.pop()?;
    for id in refused {
        let _gone = project.delete_feature_op(id);
    }
    let source = project.timeline.iter().find(|node| node.id == keep).and_then(|node| match node.kind {
        qymcad_core::feature::FeatureKind::RemoveFace { src, .. } => Some(src),
        _ => None,
    })?;
    {
        let node = project.timeline.iter_mut().find(|node| node.id == keep)?;
        match &mut node.kind {
            qymcad_core::feature::FeatureKind::RemoveFace { faces, .. } => {
                *faces = qymcad_core::refs::Ref::picks(&[asked.face.id]);
            }
            _ => return None,
        }
        node.dirty = true;
    }
    store_picked_face(project, keep, source, asked.face.id);
    Some(keep)
}

/// A copy the rebuild refused, written again with the face just given.
///
/// `asked.body` may be the solid the copy was read from, or the copy's own id. The face stored on
/// the node is that face's id. The rebuild records the face when the copy succeeds, so this write
/// does not store a snapshot of its own.
fn replace_refused_copy(project: &mut Project, asked: CopyFace) -> Option<u64> {
    let errored: Vec<u64> = project.regen_errors.keys().copied().collect();
    let mut refused = project
        .timeline
        .iter()
        .filter(|node| matches!(node.kind, qymcad_core::feature::FeatureKind::FaceCopy { src, .. } if src == asked.body.id) && errored.contains(&node.id))
        .map(|node| node.id)
        .collect::<Vec<_>>();
    if refused.is_empty() {
        let itself = project.timeline.iter().any(|node| node.id == asked.body.id && matches!(node.kind, qymcad_core::feature::FeatureKind::FaceCopy { .. }) && errored.contains(&node.id));
        if itself {
            refused.push(asked.body.id);
        }
    }
    let keep = refused.pop()?;
    for id in refused {
        let _gone = project.delete_feature_op(id);
    }
    let node = project.timeline.iter_mut().find(|node| node.id == keep)?;
    match &mut node.kind {
        qymcad_core::feature::FeatureKind::FaceCopy { faces, .. } => {
            *faces = qymcad_core::refs::Ref::picks(&[asked.face.id]);
        }
        _ => return None,
    }
    node.dirty = true;
    Some(keep)
}

/// An offset the rebuild refused, written again with the face and the distance just given.
///
/// `asked.body` may be the solid the offset was read from, or the offset's own id. The face stored
/// on the node is that face's id. The rebuild records the face when the offset succeeds, so this
/// write does not store a snapshot of its own.
fn replace_refused_offset(project: &mut Project, asked: OffsetFace) -> Option<u64> {
    let errored: Vec<u64> = project.regen_errors.keys().copied().collect();
    let mut refused = project
        .timeline
        .iter()
        .filter(|node| matches!(node.kind, qymcad_core::feature::FeatureKind::OffsetSurface { src, .. } if src == asked.body.id) && errored.contains(&node.id))
        .map(|node| node.id)
        .collect::<Vec<_>>();
    if refused.is_empty() {
        let itself = project.timeline.iter().any(|node| node.id == asked.body.id && matches!(node.kind, qymcad_core::feature::FeatureKind::OffsetSurface { .. }) && errored.contains(&node.id));
        if itself {
            refused.push(asked.body.id);
        }
    }
    let keep = refused.pop()?;
    for id in refused {
        let _gone = project.delete_feature_op(id);
    }
    let node = project.timeline.iter_mut().find(|node| node.id == keep)?;
    match &mut node.kind {
        qymcad_core::feature::FeatureKind::OffsetSurface { faces, dist, .. } => {
            *faces = qymcad_core::refs::Ref::picks(&[asked.face.id]);
            *dist = asked.by.mm;
        }
        _ => return None,
    }
    node.dirty = true;
    Some(keep)
}

/// A patch the rebuild refused, written again with the edge and the meeting just given.
///
/// `asked.body` may be the solid the patch was read from, or the patch's own id. The edge stored on
/// the node is that edge's id. The snapshot is written again, as a fillet writes its edge again, so
/// a later rebuild can find the edge after the solid changes.
fn replace_refused_patch(project: &mut Project, asked: Patch) -> Option<u64> {
    let errored: Vec<u64> = project.regen_errors.keys().copied().collect();
    let mut refused = project
        .timeline
        .iter()
        .filter(|node| matches!(node.kind, qymcad_core::feature::FeatureKind::Patch { src, .. } if src == asked.body.id) && errored.contains(&node.id))
        .map(|node| node.id)
        .collect::<Vec<_>>();
    if refused.is_empty() {
        let itself = project.timeline.iter().any(|node| node.id == asked.body.id && matches!(node.kind, qymcad_core::feature::FeatureKind::Patch { .. }) && errored.contains(&node.id));
        if itself {
            refused.push(asked.body.id);
        }
    }
    let keep = refused.pop()?;
    for id in refused {
        let _gone = project.delete_feature_op(id);
    }
    let source = {
        let node = project.timeline.iter_mut().find(|node| node.id == keep)?;
        let source = match &mut node.kind {
            qymcad_core::feature::FeatureKind::Patch { src, edges, tangent, .. } => {
                let source = *src;
                *edges = qymcad_core::refs::Ref::picks(&[asked.edge.id]);
                *tangent = meets_smoothly(asked.meet);
                source
            }
            _ => return None,
        };
        node.dirty = true;
        source
    };
    store_picked_edge(project, keep, source, asked.edge.id);
    Some(keep)
}

/// A stitch the rebuild refused, written again with the sheets and the gap just given.
///
/// One of the ids may be a sheet the stitch consumed, or the stitch's own id. The stitch's own id is
/// the name of the node and is not stored as a sheet. `sheets` is that list with the name and any
/// repeated sheet taken out. A new node beside the refused one leaves the refusal in the read.
fn replace_refused_stitch(project: &mut Project, asked: &Stitch, sheets: &[BodyId]) -> Option<u64> {
    let ids: Vec<u64> = asked.sheets.iter().map(|sheet| sheet.id).collect();
    let errored: Vec<u64> = project.regen_errors.keys().copied().collect();
    let mut refused = project
        .timeline
        .iter()
        .filter(|node| {
            errored.contains(&node.id)
                && match &node.kind {
                    qymcad_core::feature::FeatureKind::Stitch { parts, .. } => ids.contains(&node.id) || parts.iter().any(|part| ids.contains(part)),
                    _ => false,
                }
        })
        .map(|node| node.id)
        .collect::<Vec<_>>();
    let keep = refused.pop()?;
    for id in refused {
        let _gone = project.delete_feature_op(id);
    }
    let node = project.timeline.iter_mut().find(|node| node.id == keep)?;
    match &mut node.kind {
        qymcad_core::feature::FeatureKind::Stitch { parts, tol, .. } => {
            *parts = sheets.iter().map(|sheet| sheet.id).collect();
            *tol = asked.gap.mm;
        }
        _ => return None,
    }
    node.dirty = true;
    Some(keep)
}

/// The sheets to sew: a refused stitch's own id names that node, and a repeated id is one sheet.
fn distinct_sheets(project: &Project, asked: &Stitch) -> Vec<BodyId> {
    let mut sheets = Vec::new();
    for sheet in &asked.sheets {
        if refused_stitch(project, sheet.id) || sheets.iter().any(|kept: &BodyId| kept.id == sheet.id) {
            continue;
        }
        sheets.push(*sheet);
    }
    sheets
}

/// `id` is a stitch the rebuild has refused, so the caller may use it to name that node.
fn refused_stitch(project: &Project, id: u64) -> bool {
    project.regen_errors.contains_key(&id) && project.timeline.iter().any(|node| node.id == id && matches!(node.kind, qymcad_core::feature::FeatureKind::Stitch { .. }))
}

/// The tool is this cut, or a body that stands later than the cut this call would rewrite.
///
/// Naming a refused trim of the cap as its own tool left the live read on `operation failed: Trim`.
/// Opened again, that same cut read `error-source-body-not-built`. A later cut used as the tool did
/// the same. The call writes nothing, so the tool stored on the node stays one that is already built.
fn tool_is_built_later(project: &Project, asked: Trim) -> bool {
    let Some(place) = project.timeline.iter().rposition(|node| {
        project.regen_errors.contains_key(&node.id)
            && match &node.kind {
                qymcad_core::feature::FeatureKind::Trim { src, .. } => *src == asked.sheet.id || node.id == asked.sheet.id,
                _ => false,
            }
    }) else {
        return false;
    };
    let Some(tool_at) = project.timeline.iter().position(|node| node.kind.bodies().contains(&asked.tool.id)) else {
        return false;
    };
    tool_at >= place
}

/// A trim the rebuild refused, written again with the tool and the point just given.
///
/// `asked.sheet` may be the sheet the trim consumed, or the trim's own id. The trim's own id names the
/// node and is not stored as the sheet. A new node beside the refused one leaves the refusal in the read.
fn replace_refused_trim(project: &mut Project, asked: Trim) -> Option<u64> {
    let errored: Vec<u64> = project.regen_errors.keys().copied().collect();
    let mut refused = project
        .timeline
        .iter()
        .filter(|node| {
            errored.contains(&node.id)
                && match &node.kind {
                    qymcad_core::feature::FeatureKind::Trim { src, .. } => *src == asked.sheet.id || node.id == asked.sheet.id,
                    _ => false,
                }
        })
        .map(|node| node.id)
        .collect::<Vec<_>>();
    let keep = refused.pop()?;
    for id in refused {
        let _gone = project.delete_feature_op(id);
    }
    let node = project.timeline.iter_mut().find(|node| node.id == keep)?;
    match &mut node.kind {
        qymcad_core::feature::FeatureKind::Trim { tool, keep: point, .. } => {
            *tool = asked.tool.id;
            *point = [asked.keep.x.mm, asked.keep.y.mm, asked.keep.z.mm];
        }
        _ => return None,
    }
    node.dirty = true;
    Some(keep)
}

/// The sheet is this replacement, or a body that stands later than the replacement this call would rewrite.
///
/// Naming a cap copied after the refused replacement left the live read quiet at 6000 mm^3. Opened
/// again, that same replacement read `error-source-body-not-built`: the sheet is not built yet when
/// the node runs. The call writes nothing, so the sheet stored on the node stays one that is already built.
fn sheet_is_built_later(project: &Project, asked: ReplaceFace) -> bool {
    let Some(place) = project.timeline.iter().rposition(|node| {
        project.regen_errors.contains_key(&node.id)
            && match &node.kind {
                qymcad_core::feature::FeatureKind::SurfaceReplace { src, .. } => *src == asked.body.id || node.id == asked.body.id,
                _ => false,
            }
    }) else {
        return false;
    };
    let Some(sheet_at) = project.timeline.iter().position(|node| node.kind.bodies().contains(&asked.surface.id)) else {
        return false;
    };
    sheet_at >= place
}

/// `id` is a replacement the rebuild has refused, so the caller may use it to name that node.
///
/// The node never built, so it has no solid. Treating that id as a missing body would refuse the
/// rewrite with `error-source-body-not-built` and leave the opening unchanged.
fn names_a_refused_replacement(project: &Project, id: u64) -> bool {
    project.regen_errors.contains_key(&id) && project.timeline.iter().any(|node| node.id == id && matches!(node.kind, qymcad_core::feature::FeatureKind::SurfaceReplace { .. }))
}

/// A replacement the rebuild refused, written again with the face and the sheet just given.
///
/// `asked.body` may be the solid the replacement consumed, or the replacement's own id. The
/// replacement's own id names the node and is not stored as the solid. The face snapshot is written
/// again from the solid already stored, so a later rebuild can find the face after that solid changes.
/// Any earlier refused replacement of the same solid is removed, and the latest one keeps its id.
fn replace_refused_surface(project: &mut Project, asked: ReplaceFace) -> Option<u64> {
    let errored: Vec<u64> = project.regen_errors.keys().copied().collect();
    let mut refused = project
        .timeline
        .iter()
        .filter(|node| {
            errored.contains(&node.id)
                && match &node.kind {
                    qymcad_core::feature::FeatureKind::SurfaceReplace { src, .. } => *src == asked.body.id || node.id == asked.body.id,
                    _ => false,
                }
        })
        .map(|node| node.id)
        .collect::<Vec<_>>();
    let keep = refused.pop()?;
    for id in refused {
        let _gone = project.delete_feature_op(id);
    }
    let source = project.timeline.iter().find(|node| node.id == keep).and_then(|node| match node.kind {
        qymcad_core::feature::FeatureKind::SurfaceReplace { src, .. } => Some(src),
        _ => None,
    })?;
    {
        let node = project.timeline.iter_mut().find(|node| node.id == keep)?;
        match &mut node.kind {
            qymcad_core::feature::FeatureKind::SurfaceReplace { faces, surface, .. } => {
                *faces = qymcad_core::refs::Ref::picks(&[asked.face.id]);
                *surface = asked.surface.id;
            }
            _ => return None,
        }
        node.dirty = true;
    }
    store_picked_face(project, keep, source, asked.face.id);
    Some(keep)
}

/// A move the rebuild refused, written again with the placement just given.
///
/// `asked.body` may be the body the move consumed, or the move's own id. The move's own id names the
/// node and is not stored as the body. The refused move still has a mesh, so the refusal is the sign,
/// not an empty mesh. Any earlier refused move of the same body is removed, and the latest one keeps
/// its id.
fn replace_refused_move(project: &mut Project, asked: Move) -> Option<u64> {
    let errored: Vec<u64> = project.regen_errors.keys().copied().collect();
    let mut refused = project
        .timeline
        .iter()
        .filter(|node| {
            errored.contains(&node.id)
                && match &node.kind {
                    qymcad_core::feature::FeatureKind::Move { src, .. } => *src == asked.body.id || node.id == asked.body.id,
                    _ => false,
                }
        })
        .map(|node| node.id)
        .collect::<Vec<_>>();
    let keep = refused.pop()?;
    for id in refused {
        let _gone = project.delete_feature_op(id);
    }
    let node = project.timeline.iter_mut().find(|node| node.id == keep)?;
    match &mut node.kind {
        qymcad_core::feature::FeatureKind::Move { mat, .. } => *mat = asked.by.mat,
        _ => return None,
    }
    node.dirty = true;
    Some(keep)
}

/// The solid a copied, offset, or patched sheet was taken from.
fn sheet_came_from(project: &Project, id: u64) -> Option<u64> {
    project.timeline.iter().find_map(|node| match node.kind {
        qymcad_core::feature::FeatureKind::FaceCopy { src, body, .. } if body == id => Some(src),
        qymcad_core::feature::FeatureKind::OffsetSurface { src, body, .. } if body == id => Some(src),
        qymcad_core::feature::FeatureKind::Patch { src, body, .. } if body == id => Some(src),
        _ => None,
    })
}

/// Smooth asks the sheet to leave the edge tangent to the neighbouring face.
fn meets_smoothly(meet: Meet) -> bool {
    match meet {
        Meet::ByPosition => false,
        Meet::Smooth => true,
    }
}

/// A metric thread: one start, right-handed, no extra clearance. A pitch of 0 is the coarse pitch.
fn metric_spec(diameter: Nominal, pitch: Pitch) -> qymcad_core::thread::ThreadSpec {
    qymcad_core::thread::ThreadSpec { nominal_d: diameter.mm, pitch: pitch.mm, ..qymcad_core::thread::ThreadSpec::default() }
}

/// A thread the rebuild refused, written again with the rim, the size, and the length just given.
///
/// `asked.body` may be the rod the thread consumed, or the thread's own id. The edge stored on the node
/// is that rim's id. The rebuild finds the centre and the radius from it.
fn replace_refused_thread(project: &mut Project, asked: Thread) -> Option<u64> {
    let errored: Vec<u64> = project.regen_errors.keys().copied().collect();
    let mut refused = project
        .timeline
        .iter()
        .filter(|node| matches!(node.kind, qymcad_core::feature::FeatureKind::Thread { src, .. } if src == asked.body.id) && errored.contains(&node.id))
        .map(|node| node.id)
        .collect::<Vec<_>>();
    if refused.is_empty() {
        let itself = project.timeline.iter().any(|node| node.id == asked.body.id && matches!(node.kind, qymcad_core::feature::FeatureKind::Thread { .. }) && errored.contains(&node.id));
        if itself {
            refused.push(asked.body.id);
        }
    }
    let keep = refused.pop()?;
    for id in refused {
        let _gone = project.delete_feature_op(id);
    }
    let node = project.timeline.iter_mut().find(|node| node.id == keep)?;
    match &mut node.kind {
        qymcad_core::feature::FeatureKind::Thread { edge, spec, length, lead_in, lead_out, .. } => {
            *edge = asked.rim.id;
            *spec = metric_spec(asked.diameter, asked.pitch);
            *length = asked.length.mm;
            *lead_in = 0.0;
            *lead_out = 0.0;
        }
        _ => return None,
    }
    node.dirty = true;
    Some(keep)
}

/// One right-handed flight with a sharp edge. The shaft diameter stays 0: the rebuild reads it from the rim.
fn auger_spec(asked: Auger) -> qymcad_core::thread::AugerSpec {
    qymcad_core::thread::AugerSpec { shaft_d: 0.0, outer_d: asked.outer.mm, pitch: asked.pitch.mm, thickness: asked.thickness.mm, starts: 1, left: false, edge_r: 0.0 }
}

/// A flight the rebuild refused, written again with the rim and the flight just given.
///
/// `asked.body` may be the rod the flight consumed, or the flight's own id. The edge stored on the node
/// is that rim's id. The rebuild finds the centre and the radius from it, and sets the shaft diameter.
fn replace_refused_auger(project: &mut Project, asked: Auger) -> Option<u64> {
    let errored: Vec<u64> = project.regen_errors.keys().copied().collect();
    let mut refused = project
        .timeline
        .iter()
        .filter(|node| matches!(node.kind, qymcad_core::feature::FeatureKind::Auger { src, .. } if src == asked.body.id) && errored.contains(&node.id))
        .map(|node| node.id)
        .collect::<Vec<_>>();
    if refused.is_empty() {
        let itself = project.timeline.iter().any(|node| node.id == asked.body.id && matches!(node.kind, qymcad_core::feature::FeatureKind::Auger { .. }) && errored.contains(&node.id));
        if itself {
            refused.push(asked.body.id);
        }
    }
    let keep = refused.pop()?;
    for id in refused {
        let _gone = project.delete_feature_op(id);
    }
    let node = project.timeline.iter_mut().find(|node| node.id == keep)?;
    match &mut node.kind {
        qymcad_core::feature::FeatureKind::Auger { edge, spec, length, lead_in, lead_out, .. } => {
            *edge = asked.rim.id;
            *spec = auger_spec(asked);
            *length = asked.length.mm;
            *lead_in = 0.0;
            *lead_out = 0.0;
        }
        _ => return None,
    }
    node.dirty = true;
    Some(keep)
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
        let rects = sketch.rects();
        assert_eq!(rects.len(), 1, "the rectangle stays one shape");
        let rect = &rects[0];
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

    /// The fillet node stores the edge it was given, and the same descriptor is still there after the
    /// rectangle above it is resized.
    #[test]
    fn a_fillet_stores_the_edge_it_rounds() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let first = session.document();
        let recorded = long_side(&session, &first, made.id, drawn.sketch.id);
        let rounded = session.fillet(made, recorded.edge, Radius { mm: 2.0 }).expect("a fillet");
        edge_is_stored(&session, rounded.id, recorded.edge);
        session.resize(drawn, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 10.0) } }).expect("the rectangle moves");
        let _shrunk = session.document();
        edge_is_stored(&session, rounded.id, recorded.edge);
    }

    fn edge_is_stored(session: &Session, fillet: u64, edge: EdgeId) {
        let snaps = session.project.edge_refs.get(&fillet).expect("the fillet stored no edge snapshot");
        assert!(snaps.iter().any(|snap| snap.id == edge.id), "the snapshot is not this edge");
        let node = session.project.timeline.iter().find(|node| node.id == fillet).expect("the fillet node");
        match &node.kind {
            FeatureKind::Fillet { edges, .. } => assert_eq!(edges.query.picked_descs(), vec![edge.id], "the node's reference is not this edge"),
            _ => panic!("the node is a fillet"),
        }
        assert!(session.project.names.edge(edge.id).is_some(), "the edge is not an edge name");
        assert!(session.project.names.get(edge.id).is_none(), "the edge is a face name");
    }

    /// A fillet the rebuild refused is the same node when another radius is asked. A second node would
    /// leave the refusal in place, so the read could not come back quiet.
    #[test]
    fn a_replaced_fillet_stores_the_edge_it_is_given() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let first = session.document();
        let recorded = long_side(&session, &first, made.id, drawn.sketch.id);
        let other = first.bodies[0].edges.iter().find(|edge| edge.id != recorded.edge && (edge.length.mm - 20.0).abs() < 0.1).expect("another 20 mm edge").id;
        let failed = session.fillet(made, recorded.edge, Radius { mm: 100.0 }).expect("a radius the edge cannot take");
        let _refused = session.document();
        let replaced = session.fillet(made, other, Radius { mm: 2.0 }).expect("the fillet is tried again");
        assert_eq!(replaced, failed, "a second fillet was added");
        edge_is_stored(&session, replaced.id, other);
        assert_eq!(session.project.timeline.iter().filter(|node| matches!(node.kind, FeatureKind::Fillet { .. })).count(), 1, "the refused fillet was left beside the new one");
    }

    /// The chamfer node stores the edge it was given, as a symmetric chamfer, and the same descriptor is
    /// still there after the rectangle above it is resized.
    #[test]
    fn a_chamfer_stores_the_edge_it_bevels() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let first = session.document();
        let recorded = long_side(&session, &first, made.id, drawn.sketch.id);
        let beveled = session.chamfer(made, recorded.edge, Distance { mm: 2.0 }).expect("a chamfer");
        chamfer_edge_is_stored(&session, beveled.id, recorded.edge);
        session.resize(drawn, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 10.0) } }).expect("the rectangle moves");
        let _shrunk = session.document();
        chamfer_edge_is_stored(&session, beveled.id, recorded.edge);
    }

    /// A chamfer the rebuild refused keeps its id when another edge is asked, and that edge is what is stored.
    #[test]
    fn a_replaced_chamfer_stores_the_edge_it_is_given() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let first = session.document();
        let recorded = long_side(&session, &first, made.id, drawn.sketch.id);
        let other = first.bodies[0].edges.iter().find(|edge| edge.id != recorded.edge && (edge.length.mm - 20.0).abs() < 0.1).expect("another 20 mm edge").id;
        let failed = session.chamfer(made, recorded.edge, Distance { mm: 100.0 }).expect("a distance the edge cannot take");
        let _refused = session.document();
        let replaced = session.chamfer(made, other, Distance { mm: 2.0 }).expect("the chamfer is tried again");
        assert_eq!(replaced, failed, "a second chamfer was added");
        chamfer_edge_is_stored(&session, replaced.id, other);
        assert_eq!(session.project.timeline.iter().filter(|node| matches!(node.kind, FeatureKind::Chamfer { .. })).count(), 1, "the refused chamfer was left beside the new one");
        assert!(session.project.timeline.iter().all(|node| !matches!(node.kind, FeatureKind::Fillet { .. })), "the chamfer rewrote a fillet");
    }

    fn chamfer_edge_is_stored(session: &Session, chamfer: u64, edge: EdgeId) {
        let snaps = session.project.edge_refs.get(&chamfer).expect("the chamfer stored no edge snapshot");
        assert!(snaps.iter().any(|snap| snap.id == edge.id), "the snapshot is not this edge");
        let node = session.project.timeline.iter().find(|node| node.id == chamfer).expect("the chamfer node");
        match &node.kind {
            FeatureKind::Chamfer { edges, mode, d2, flip, ref_face, .. } => {
                assert_eq!(edges.query.picked_descs(), vec![edge.id], "the node's reference is not this edge");
                assert_eq!(*mode, qymcad_core::feature::ChamferMode::Symmetric, "the chamfer is not symmetric");
                assert_eq!(*d2, 0.0, "the chamfer has a second distance");
                assert!(!*flip, "the chamfer measures from the other side");
                assert_eq!(*ref_face, 0, "the chamfer names a reference face");
            }
            _ => panic!("the node is a chamfer"),
        }
        assert!(session.project.names.edge(edge.id).is_some(), "the edge is not an edge name");
        assert!(session.project.names.get(edge.id).is_none(), "the edge is a face name");
    }

    /// The shell node stores the end cap it was given, the wall is taken inward, and the same descriptor is
    /// still there after the rectangle above it is resized.
    #[test]
    fn a_shell_stores_the_face_it_opens() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let first = session.document();
        let recorded = long_side(&session, &first, made.id, drawn.sketch.id);
        let opened = session.shell(made, recorded.cap, Thickness { mm: 2.0 }).expect("a shell");
        let _built = session.document();
        shell_face_is_stored(&session, opened.id, recorded.cap);
        session.resize(drawn, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 10.0) } }).expect("the rectangle moves");
        let _shrunk = session.document();
        shell_face_is_stored(&session, opened.id, recorded.cap);
    }

    /// A shell the rebuild refused is the same node when another face is asked. The snapshot is that new face.
    #[test]
    fn a_replaced_shell_stores_the_face_it_is_given() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let first = session.document();
        let recorded = long_side(&session, &first, made.id, drawn.sketch.id);
        let failed = session.shell(made, recorded.cap, Thickness { mm: 6.0 }).expect("a thickness the cap cannot take");
        let _refused = session.document();
        let replaced = session.shell(made, recorded.wall, Thickness { mm: 2.0 }).expect("the shell is tried again");
        assert_eq!(replaced, failed, "a second shell was added");
        shell_face_is_stored(&session, replaced.id, recorded.wall);
        assert_eq!(session.project.timeline.iter().filter(|node| matches!(node.kind, FeatureKind::Shell { .. })).count(), 1, "the refused shell was left beside the new one");
        assert!(session.project.timeline.iter().all(|node| !matches!(node.kind, FeatureKind::Fillet { .. } | FeatureKind::Chamfer { .. })), "the shell rewrote another feature");
    }

    fn shell_face_is_stored(session: &Session, shell: u64, face: FaceId) {
        let snaps = session.project.face_refs.get(&shell).expect("the shell stored no face snapshot");
        assert!(snaps.iter().any(|snap| snap.id == face.id), "the snapshot is not this face");
        let node = session.project.timeline.iter().find(|node| node.id == shell).expect("the shell node");
        assert_eq!(node.name, "feat-name-shell", "the node is not a shell");
        match &node.kind {
            FeatureKind::Shell { faces, side, .. } => {
                assert_eq!(faces.query.picked_descs(), vec![face.id], "the node's reference is not this face");
                assert_eq!(*side, qymcad_core::feature::ShellSide::Inward, "the wall is not taken out of the body");
            }
            _ => panic!("the node is a shell"),
        }
        assert!(session.project.names.get(face.id).is_some(), "the face is not a face name");
        assert!(session.project.names.edge(face.id).is_none(), "the face is an edge name");
    }

    /// The hole node stores the end cap it was given, at that cap's centre.
    #[test]
    fn a_hole_stores_the_face_it_drills() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let first = session.document();
        let recorded = long_side(&session, &first, made.id, drawn.sketch.id);
        let drilled = session.hole(made, recorded.cap, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("a hole");
        let _built = session.document();
        hole_face_is_stored(&session, drilled.id, made.id, recorded.cap);
    }

    /// A hole the rebuild refused is the same node when another face is asked. The centre is that new face.
    #[test]
    fn a_replaced_hole_stores_the_face_it_is_given() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let made = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let first = session.document();
        let recorded = long_side(&session, &first, made.id, drawn.sketch.id);
        let failed = session.hole(made, FaceId { id: 1 }, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("a face that is not on the box");
        let _refused = session.document();
        let replaced = session.hole(made, recorded.wall, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("the hole is tried again");
        assert_eq!(replaced, failed, "a second hole was added");
        hole_face_is_stored(&session, replaced.id, made.id, recorded.wall);
        assert_eq!(session.project.timeline.iter().filter(|node| matches!(node.kind, FeatureKind::Hole { .. })).count(), 1, "the refused hole was left beside the new one");
        assert!(
            session.project.timeline.iter().all(|node| !matches!(node.kind, FeatureKind::Fillet { .. } | FeatureKind::Chamfer { .. } | FeatureKind::Shell { .. })),
            "the hole rewrote another feature"
        );
    }

    fn hole_face_is_stored(session: &Session, hole: u64, body: u64, face: FaceId) {
        let item = session.project.regen_faces.get(&body).and_then(|faces| faces.iter().find(|item| item.id == face.id)).expect("the face on the source");
        let at = [item.centroid.x, item.centroid.y, item.centroid.z];
        let node = session.project.timeline.iter().find(|node| node.id == hole).expect("the hole node");
        assert_eq!(node.name, "feat-name-hole", "the node is not a hole");
        match &node.kind {
            FeatureKind::Hole { face: stored, point, normal, kind, dia2, depth2, sketch, flip, .. } => {
                assert_eq!(stored.query.picked_descs(), vec![face.id], "the node's reference is not this face");
                assert_eq!(stored.hint.centroid, at, "the reference fingerprint is not this face's centre");
                assert_eq!(stored.hint.normal, item.normal, "the reference fingerprint is not this face's normal");
                assert_eq!(*point, at, "the hole is not at this face's centre");
                assert_eq!(*normal, item.normal, "the hole does not follow this face's normal");
                assert_eq!(*kind, 0, "the hole is not a plain hole");
                assert_eq!(*dia2, 0.0, "the hole has a recess");
                assert_eq!(*depth2, 0.0, "the recess has a depth");
                assert_eq!(*sketch, 0, "the hole is placed by a sketch");
                assert!(!*flip, "the hole drills against the sketch normal");
            }
            _ => panic!("the node is a hole"),
        }
    }

    /// The box stores its three sizes, and its top cap stands at the centre, not at a corner.
    #[test]
    fn a_box_stores_its_sizes_and_stands_at_the_centre() {
        let mut session = Session::empty();
        let made = session.cuboid(Cuboid { length: BoxLength { mm: 10.0 }, width: Width { mm: 20.0 }, height: Height { mm: 30.0 } }).expect("a box");
        let _built = session.document();
        let node = session.project.timeline.iter().find(|node| node.id == made.id).expect("the box node");
        assert_eq!(node.name, "feat-name-box", "the node is not a box");
        match &node.kind {
            FeatureKind::Box3 { dx, dy, dz, .. } => {
                assert_eq!(*dx, 10.0, "the length is not 10 mm");
                assert_eq!(*dy, 20.0, "the width is not 20 mm");
                assert_eq!(*dz, 30.0, "the height is not 30 mm");
            }
            _ => panic!("the node is a box"),
        }
        let top =
            session.project.regen_faces.get(&made.id).and_then(|faces| faces.iter().find(|face| face.centroid.x.abs() < 1e-6 && face.centroid.y.abs() < 1e-6 && (face.centroid.z - 30.0).abs() < 1e-6));
        assert!(top.is_some(), "the top cap is not at (0, 0, 30)");
    }

    /// A cylinder inside a box is united with it. Both operands are consumed, and the union is the one body.
    #[test]
    fn a_cylinder_inside_a_box_leaves_one_body() {
        let mut session = Session::empty();
        let block = session.cuboid(Cuboid { length: BoxLength { mm: 10.0 }, width: Width { mm: 20.0 }, height: Height { mm: 30.0 } }).expect("a box");
        let joined = session.cylinder(Cylinder { radius: Radius { mm: 4.0 }, height: Height { mm: 10.0 } }).expect("a cylinder");
        let cylinder = session.project.timeline.iter().find(|node| node.name == "feat-name-cylinder").expect("the cylinder").id;
        assert_ne!(joined.id, block.id, "the union is the box");
        assert_ne!(joined.id, cylinder, "the union is the cylinder");
        let consumed = session.project.consumed_bodies();
        assert!(consumed.contains(&block.id), "the box is not consumed");
        assert!(consumed.contains(&cylinder), "the cylinder is not consumed");
        let live: Vec<u64> = session.project.timeline.iter().flat_map(|node| node.kind.bodies()).filter(|body| !consumed.contains(body)).collect();
        assert_eq!(live, vec![joined.id], "the part has one body");
    }

    /// A cut of the box by the extrusion inside it is one node, and the returned id is neither operand.
    #[test]
    fn a_cut_is_one_node_and_neither_operand() {
        let mut session = Session::empty();
        let block = session.cuboid(Cuboid { length: BoxLength { mm: 40.0 }, width: Width { mm: 30.0 }, height: Height { mm: 10.0 } }).expect("a box");
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 10.0) } }).expect("a rectangle");
        let tool = session.extrude(drawn, Height { mm: 10.0 }).expect("an extrusion");
        let cut = session.boolean(Boolean { base: Base { body: block }, op: BoolOp::Cut, tool: Tool { body: tool } }).expect("a cut");
        let _built = session.document();
        assert_ne!(cut.id, block.id, "the cut is the box");
        assert_ne!(cut.id, tool.id, "the cut is the extrusion");
        let cuts: Vec<_> = session.project.timeline.iter().filter(|node| node.name == "feat-name-body-cut").collect();
        assert_eq!(cuts.len(), 1, "the cut is one node");
        assert_eq!(cuts[0].id, cut.id, "the returned id is not the cut node");
        match &cuts[0].kind {
            FeatureKind::BodyBoolean { a, b, op, .. } => {
                assert_eq!(*a, block.id, "the base is not the box");
                assert_eq!(*b, tool.id, "the tool is not the extrusion");
                assert_eq!(*op, 0, "the node is not a cut");
            }
            _ => panic!("the node is a cut"),
        }
        let consumed = session.project.consumed_bodies();
        assert!(consumed.contains(&block.id), "the box is not consumed");
        assert!(consumed.contains(&tool.id), "the extrusion is not consumed");
        assert!(!consumed.contains(&cut.id), "the cut consumed itself");
    }

    /// The mirror through the table keeps the box, names no datum and no face, and returns its own body.
    #[test]
    fn a_mirror_keeps_the_box_through_the_table() {
        let mut session = Session::empty();
        let block = session.cuboid(Cuboid { length: BoxLength { mm: 10.0 }, width: Width { mm: 20.0 }, height: Height { mm: 30.0 } }).expect("a box");
        let made = session.mirror(Mirror { body: block, plane: WorldPlane::Xy }).expect("a mirror");
        let _built = session.document();
        assert_ne!(made.id, block.id, "the mirror is the box");
        let mirrors: Vec<_> = session.project.timeline.iter().filter(|node| node.name == "feat-name-mirror").collect();
        assert_eq!(mirrors.len(), 1, "the mirror is one node");
        assert_eq!(mirrors[0].id, made.id, "the returned id is not the mirror node");
        match &mirrors[0].kind {
            FeatureKind::Mirror { src, plane, keep, datum, body, face } => {
                assert_eq!(*src, block.id, "the source is not the box");
                assert_eq!(*plane, 0, "the plane is not the table");
                assert!(*keep, "the original was thrown away");
                assert_eq!(*datum, 0, "the mirror names a datum");
                assert_eq!(*body, made.id, "the returned id is not the mirror's body");
                assert!(face.is_none(), "the mirror names a face");
            }
            _ => panic!("the node is a mirror"),
        }
        let consumed = session.project.consumed_bodies();
        assert!(consumed.contains(&block.id), "the box is not consumed");
        assert!(!consumed.contains(&made.id), "the mirror consumed itself");
    }

    /// The split through the table at 10 mm keeps no datum and no face, and the first piece is the node.
    #[test]
    fn a_split_keeps_the_lower_piece_as_the_node() {
        let mut session = Session::empty();
        let block = session.cuboid(Cuboid { length: BoxLength { mm: 10.0 }, width: Width { mm: 20.0 }, height: Height { mm: 30.0 } }).expect("a box");
        let made = session.split(Split { body: block, plane: WorldPlane::Xy, offset: Offset { mm: 10.0 } }).expect("a split");
        let _built = session.document();
        assert_eq!(made.bodies.len(), 2, "two pieces");
        assert_ne!(made.bodies[0], block, "the lower piece is the box");
        let splits: Vec<_> = session.project.timeline.iter().filter(|node| node.name == "feat-name-split-body").collect();
        assert_eq!(splits.len(), 1, "the split is one node");
        assert_eq!(splits[0].id, made.bodies[0].id, "the first piece is not the node");
        match &splits[0].kind {
            FeatureKind::SplitBody { src, plane, datum, offset, bodies, face } => {
                assert_eq!(*src, block.id, "the source is not the box");
                assert_eq!(*plane, 0, "the plane is not the table");
                assert_eq!(*datum, 0, "the split names a datum");
                assert!((*offset - 10.0).abs() < 1e-9, "the offset is not 10 mm");
                assert_eq!(bodies.as_slice(), &[made.bodies[0].id, made.bodies[1].id], "the piece ids are not the ones returned");
                assert!(face.is_none(), "the split names a face");
            }
            _ => panic!("the node is a split"),
        }
        let consumed = session.project.consumed_bodies();
        assert!(consumed.contains(&block.id), "the box is not consumed");
        assert!(!consumed.contains(&made.bodies[0].id), "the lower piece consumed itself");
        assert!(!consumed.contains(&made.bodies[1].id), "the upper piece is consumed");
    }

    /// The face mark through the table at 10 mm keeps no datum and no face, and the node is the returned body.
    #[test]
    fn a_split_face_keeps_the_extrusion_whole() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let made = session.split_faces(Split { body: block, plane: WorldPlane::Xy, offset: Offset { mm: 10.0 } }).expect("a mark");
        let _built = session.document();
        assert_ne!(made, block, "the marked body is the extrusion");
        let marks: Vec<_> = session.project.timeline.iter().filter(|node| node.name == "feat-name-split-face").collect();
        assert_eq!(marks.len(), 1, "the mark is one node");
        assert_eq!(marks[0].id, made.id, "the returned id is not the mark node");
        match &marks[0].kind {
            FeatureKind::SplitFace { src, plane, datum, offset, body, face } => {
                assert_eq!(*src, block.id, "the source is not the extrusion");
                assert_eq!(*plane, 0, "the plane is not the table");
                assert_eq!(*datum, 0, "the mark names a datum");
                assert!((*offset - 10.0).abs() < 1e-9, "the offset is not 10 mm");
                assert_eq!(*body, made.id, "the returned id is not the marked body");
                assert!(face.is_none(), "the mark names a face");
            }
            _ => panic!("the node is a face mark"),
        }
        let consumed = session.project.consumed_bodies();
        assert!(consumed.contains(&block.id), "the extrusion is not consumed");
        assert!(!consumed.contains(&made.id), "the marked body consumed itself");
    }

    /// The draft of 10 degrees stores that angle, the wall, and the end cap, and does not reverse the pull.
    #[test]
    fn a_draft_keeps_the_wall_and_the_cap() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let doc = session.document();
        let found = doc.bodies.iter().find(|body| body.id == block.id).expect("the extrusion");
        let tilted = found.faces.iter().find(|face| (face.area.mm2 - 300.0).abs() < 1.0).expect("the short wall").id;
        let cap = found.faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the end cap").id;
        let made = session.draft(Draft { body: block, tilted: Tilted { face: tilted }, neutral: Neutral { face: cap }, angle: Angle { deg: 10.0 } }).expect("a draft");
        let _built = session.document();
        assert_ne!(made, block, "the drafted body is the extrusion");
        let drafts: Vec<_> = session.project.timeline.iter().filter(|node| node.name == "feat-name-draft").collect();
        assert_eq!(drafts.len(), 1, "the draft is one node");
        assert_eq!(drafts[0].id, made.id, "the returned id is not the draft node");
        match &drafts[0].kind {
            FeatureKind::Draft { src, faces, neutral, angle, flip, body } => {
                assert_eq!(*src, block.id, "the source is not the extrusion");
                assert_eq!(faces.query.picked_descs(), vec![tilted.id], "the wall is not the one given");
                assert_eq!(neutral.query.picked_descs(), vec![cap.id], "the neutral face is not the end cap");
                assert!((*angle - 10.0).abs() < 1e-9, "the angle is not 10 degrees");
                assert!(!*flip, "the pull is reversed");
                assert_eq!(*body, made.id, "the returned id is not the drafted body");
            }
            _ => panic!("the node is a draft"),
        }
        let consumed = session.project.consumed_bodies();
        assert!(consumed.contains(&block.id), "the extrusion is not consumed");
        assert!(!consumed.contains(&made.id), "the drafted body consumed itself");
    }

    /// The push node stores the end cap it was given, with that cap's centre and normal, and the
    /// distance is the 10 mm the caller gave.
    #[test]
    fn a_push_keeps_the_cap() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let doc = session.document();
        let found = doc.bodies.iter().find(|body| body.id == block.id).expect("the extrusion");
        let cap = found.faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the end cap").id;
        let made = session.push(Push { body: block, face: cap, by: Shift { mm: 10.0 } }).expect("a push");
        let _built = session.document();
        assert_ne!(made, block, "the pushed body is the extrusion");
        let pushes: Vec<_> = session.project.timeline.iter().filter(|node| node.name == "feat-name-push-face").collect();
        assert_eq!(pushes.len(), 1, "the push is one node");
        assert_eq!(pushes[0].id, made.id, "the returned id is not the push node");
        match &pushes[0].kind {
            FeatureKind::PushFace { src, face, dist, body } => {
                assert_eq!(*src, block.id, "the source is not the extrusion");
                assert_eq!(face.query.picked_descs(), vec![cap.id], "the face is not the one given");
                assert!((*dist - 10.0).abs() < 1e-9, "the distance is not 10 mm");
                let witnessed = session.project.regen_faces.get(&block.id).and_then(|faces| faces.iter().find(|item| item.id == cap.id)).expect("the cap");
                assert_eq!(face.hint.centroid, [witnessed.centroid.x, witnessed.centroid.y, witnessed.centroid.z], "the centre is not the cap's");
                assert_eq!(face.hint.normal, witnessed.normal, "the normal is not the cap's");
                assert_eq!(*body, made.id, "the returned id is not the pushed body");
            }
            _ => panic!("the node is a push"),
        }
        let consumed = session.project.consumed_bodies();
        assert!(consumed.contains(&block.id), "the extrusion is not consumed");
        assert!(!consumed.contains(&made.id), "the pushed body consumed itself");
    }

    /// The thicken node stores the end cap it was given, the thickness is the 2 mm the caller gave,
    /// and the plate is welded to that source.
    #[test]
    fn a_thicken_keeps_the_cap() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let doc = session.document();
        let found = doc.bodies.iter().find(|body| body.id == block.id).expect("the extrusion");
        let cap = found.faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the end cap").id;
        let made = session.thicken(Thicken { body: block, face: cap, plate: Plate { mm: 2.0 } }).expect("a plate");
        let _built = session.document();
        assert_ne!(made, block, "the thickened body is the extrusion");
        let plates: Vec<_> = session.project.timeline.iter().filter(|node| node.name == "feat-name-thicken").collect();
        assert_eq!(plates.len(), 1, "the thicken is one node");
        assert_eq!(plates[0].id, made.id, "the returned id is not the thicken node");
        match &plates[0].kind {
            FeatureKind::Thicken { src, face, thickness, join, body } => {
                assert_eq!(*src, block.id, "the source is not the extrusion");
                assert_eq!(*face, cap.id, "the face is not the one given");
                assert!((*thickness - 2.0).abs() < 1e-9, "the thickness is not 2 mm");
                assert_eq!(*join, 0, "the plate is welded to another body");
                assert_eq!(*body, made.id, "the returned id is not the thickened body");
            }
            _ => panic!("the node is a thicken"),
        }
        let consumed = session.project.consumed_bodies();
        assert!(consumed.contains(&block.id), "the extrusion is not consumed");
        assert!(!consumed.contains(&made.id), "the thickened body consumed itself");
    }

    /// The removal node stores the hole wall it was given, and the hole is consumed.
    #[test]
    fn a_removal_keeps_the_wall() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let before = session.document();
        let cap = before.bodies.iter().find(|body| body.id == block.id).expect("the extrusion").faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the end cap").id;
        let drilled = session.hole(block, cap, Diameter { mm: 4.0 }, Depth { mm: 40.0 }).expect("a hole");
        let doc = session.document();
        let host = doc.bodies.iter().find(|body| body.id == drilled.id).expect("the hole");
        let wall = host.faces.iter().find(|face| !matches!(face.role, FaceRole::CapStart | FaceRole::CapEnd | FaceRole::Wall { .. })).expect("the hole wall").id;
        let made = session.remove_face(RemoveFace { body: drilled, face: wall }).expect("the wall is taken away");
        let _closed = session.document();
        assert_ne!(made, drilled, "the closed body is the hole");
        let removals: Vec<_> = session.project.timeline.iter().filter(|node| node.name == "feat-name-remove-face").collect();
        assert_eq!(removals.len(), 1, "the removal is one node");
        assert_eq!(removals[0].id, made.id, "the returned id is not the removal node");
        match &removals[0].kind {
            FeatureKind::RemoveFace { src, faces, body } => {
                assert_eq!(*src, drilled.id, "the source is not the hole");
                assert_eq!(faces.query.picked_descs(), vec![wall.id], "the face is not the one given");
                assert_eq!(*body, made.id, "the returned id is not the closed body");
            }
            _ => panic!("the node is a removal"),
        }
        let consumed = session.project.consumed_bodies();
        assert!(consumed.contains(&drilled.id), "the hole is not consumed");
        assert!(!consumed.contains(&made.id), "the closed body consumed itself");
    }

    /// The thread node stores the rim it was given, M10x1.5 over 20 mm, and the rod is consumed.
    #[test]
    fn a_thread_keeps_the_rim() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (25.0, 5.0) } }).expect("a rectangle");
        let rod = session.revolve(drawn, SketchAxis::X, Angle { deg: 360.0 }).expect("a rod");
        let doc = session.document();
        let edge = doc.bodies.iter().find(|body| body.id == rod.id).expect("the rod").edges.iter().find(|item| item.length.mm < 1.0).expect("a rim").id;
        let made = session.thread(Thread { body: rod, rim: edge, diameter: Nominal { mm: 10.0 }, pitch: Pitch { mm: 1.5 }, length: ThreadLength { mm: 20.0 } }).expect("a thread");
        let _cut = session.document();
        assert_ne!(made, rod, "the threaded body is the rod");
        let cuts: Vec<_> = session.project.timeline.iter().filter(|node| node.name == "feat-name-thread").collect();
        assert_eq!(cuts.len(), 1, "the thread is one node");
        assert_eq!(cuts[0].id, made.id, "the returned id is not the thread node");
        match &cuts[0].kind {
            FeatureKind::Thread { src, edge: stored, spec, length, lead_in, lead_out, body } => {
                assert_eq!(*src, rod.id, "the source is not the rod");
                assert_eq!(*stored, edge.id, "the rim is not the one given");
                assert!((spec.nominal_d - 10.0).abs() < 1e-9, "the diameter is not 10 mm");
                assert!((spec.pitch - 1.5).abs() < 1e-9, "the pitch is not 1.5 mm");
                assert!((*length - 20.0).abs() < 1e-9, "the length is not 20 mm");
                assert!(lead_in.abs() < 1e-9, "the thread has a lead-in");
                assert!(lead_out.abs() < 1e-9, "the thread has a lead-out");
                assert_eq!(*body, made.id, "the returned id is not the threaded body");
            }
            _ => panic!("the node is a thread"),
        }
        let consumed = session.project.consumed_bodies();
        assert!(consumed.contains(&rod.id), "the rod is not consumed");
        assert!(!consumed.contains(&made.id), "the threaded body consumed itself");
    }

    /// The flight node stores the rim it was given, outer 30 mm, pitch 20 mm, thickness 3 mm, over 20 mm.
    #[test]
    fn a_auger_keeps_the_rim() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (25.0, 5.0) } }).expect("a rectangle");
        let rod = session.revolve(drawn, SketchAxis::X, Angle { deg: 360.0 }).expect("a rod");
        let doc = session.document();
        let edge = doc.bodies.iter().find(|body| body.id == rod.id).expect("the rod").edges.iter().find(|item| item.length.mm < 1.0).expect("a rim").id;
        let made = session
            .auger(Auger { body: rod, rim: edge, outer: Outer { mm: 30.0 }, pitch: AugerPitch { mm: 20.0 }, thickness: Flight { mm: 3.0 }, length: AugerLength { mm: 20.0 } })
            .expect("a flight");
        let _welded = session.document();
        assert_ne!(made, rod, "the welded body is the rod");
        let flights: Vec<_> = session.project.timeline.iter().filter(|node| node.name == "feat-name-auger").collect();
        assert_eq!(flights.len(), 1, "the flight is one node");
        assert_eq!(flights[0].id, made.id, "the returned id is not the flight node");
        match &flights[0].kind {
            FeatureKind::Auger { src, edge: stored, spec, length, lead_in, lead_out, body } => {
                assert_eq!(*src, rod.id, "the source is not the rod");
                assert_eq!(*stored, edge.id, "the rim is not the one given");
                assert!(spec.shaft_d.abs() < 1e-9, "the shaft diameter is not 0");
                assert!((spec.outer_d - 30.0).abs() < 1e-9, "the outer diameter is not 30 mm");
                assert!((spec.pitch - 20.0).abs() < 1e-9, "the pitch is not 20 mm");
                assert!((spec.thickness - 3.0).abs() < 1e-9, "the thickness is not 3 mm");
                assert_eq!(spec.starts, 1, "the flight has more than one start");
                assert!(!spec.left, "the flight is left-handed");
                assert!(spec.edge_r.abs() < 1e-9, "the flight edge is rounded");
                assert!((*length - 20.0).abs() < 1e-9, "the length is not 20 mm");
                assert!(lead_in.abs() < 1e-9, "the flight has a lead-in");
                assert!(lead_out.abs() < 1e-9, "the flight has a lead-out");
                assert_eq!(*body, made.id, "the returned id is not the welded body");
            }
            _ => panic!("the node is a flight"),
        }
        let consumed = session.project.consumed_bodies();
        assert!(consumed.contains(&rod.id), "the rod is not consumed");
        assert!(!consumed.contains(&made.id), "the welded body consumed itself");
    }

    /// The copy node stores the end cap it was given, and neither the box nor the sheet is consumed.
    #[test]
    fn a_copy_keeps_the_face() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let doc = session.document();
        let cap = doc.bodies.iter().find(|body| body.id == block.id).expect("the box").faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the end cap").id;
        let made = session.copy_face(CopyFace { body: block, face: cap }).expect("a copy");
        let _sheet = session.document();
        assert_ne!(made, block, "the sheet is the box");
        let copies: Vec<_> = session.project.timeline.iter().filter(|node| node.name == "feat-name-face-copy").collect();
        assert_eq!(copies.len(), 1, "the copy is one node");
        assert_eq!(copies[0].id, made.id, "the returned id is not the copy node");
        match &copies[0].kind {
            FeatureKind::FaceCopy { src, faces, body } => {
                assert_eq!(*src, block.id, "the source is not the box");
                assert_eq!(faces.query.picked_descs(), vec![cap.id], "the face is not the end cap");
                assert_eq!(*body, made.id, "the returned id is not the sheet");
            }
            _ => panic!("the node is a copy"),
        }
        let consumed = session.project.consumed_bodies();
        assert!(!consumed.contains(&block.id), "the box is consumed");
        assert!(!consumed.contains(&made.id), "the sheet is consumed");
    }

    /// The offset node stores the side it was given and a distance of 2 mm, and neither body is consumed.
    #[test]
    fn an_offset_keeps_the_face() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (25.0, 5.0) } }).expect("a rectangle");
        let rod = session.revolve(drawn, SketchAxis::X, Angle { deg: 360.0 }).expect("a rod");
        let doc = session.document();
        let face = doc.bodies.iter().find(|body| body.id == rod.id).expect("the rod").faces.iter().max_by(|a, b| a.area.mm2.partial_cmp(&b.area.mm2).expect("an area")).expect("the side").id;
        let made = session.offset_face(OffsetFace { body: rod, face, by: Outset { mm: 2.0 } }).expect("an offset");
        let _sheet = session.document();
        assert_ne!(made, rod, "the sheet is the rod");
        let offsets: Vec<_> = session.project.timeline.iter().filter(|node| node.name == "feat-name-offset-surface").collect();
        assert_eq!(offsets.len(), 1, "the offset is one node");
        assert_eq!(offsets[0].id, made.id, "the returned id is not the offset node");
        match &offsets[0].kind {
            FeatureKind::OffsetSurface { src, faces, dist, body } => {
                assert_eq!(*src, rod.id, "the source is not the rod");
                assert_eq!(faces.query.picked_descs(), vec![face.id], "the face is not the side");
                assert!((*dist - 2.0).abs() < 1e-9, "the distance is not 2 mm");
                assert_eq!(*body, made.id, "the returned id is not the sheet");
            }
            _ => panic!("the node is an offset"),
        }
        let consumed = session.project.consumed_bodies();
        assert!(!consumed.contains(&rod.id), "the rod is consumed");
        assert!(!consumed.contains(&made.id), "the sheet is consumed");
    }

    /// The patch node stores the rim it was given, meeting it by position, and neither body is consumed.
    #[test]
    fn a_patch_keeps_the_rim() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (25.0, 5.0) } }).expect("a rectangle");
        let rod = session.revolve(drawn, SketchAxis::X, Angle { deg: 360.0 }).expect("a rod");
        let doc = session.document();
        let edge = doc.bodies.iter().find(|body| body.id == rod.id).expect("the rod").edges.iter().find(|item| item.length.mm < 1.0).expect("a rim").id;
        let made = session.patch(Patch { body: rod, edge, meet: Meet::ByPosition }).expect("a patch");
        let _sheet = session.document();
        assert_ne!(made, rod, "the sheet is the rod");
        let patches: Vec<_> = session.project.timeline.iter().filter(|node| node.name == "feat-name-patch").collect();
        assert_eq!(patches.len(), 1, "the patch is one node");
        assert_eq!(patches[0].id, made.id, "the returned id is not the patch node");
        match &patches[0].kind {
            FeatureKind::Patch { src, edges, tangent, body } => {
                assert_eq!(*src, rod.id, "the source is not the rod");
                assert_eq!(edges.query.picked_descs(), vec![edge.id], "the edge is not the rim");
                assert!(!tangent, "the sheet was asked to meet smoothly");
                assert_eq!(*body, made.id, "the returned id is not the sheet");
            }
            _ => panic!("the node is a patch"),
        }
        let consumed = session.project.consumed_bodies();
        assert!(!consumed.contains(&rod.id), "the rod is consumed");
        assert!(!consumed.contains(&made.id), "the sheet is consumed");
    }

    /// The stitch node stores the cap and the short wall at a gap of 0.0001 mm. Both sheets are consumed.
    #[test]
    fn a_stitch_keeps_its_sheets() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let doc = session.document();
        let box_body = doc.bodies.iter().find(|body| body.id == block.id).expect("the box");
        let cap = box_body.faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the end cap").id;
        let wall = box_body.faces.iter().find(|face| (face.area.mm2 - 300.0).abs() < 1.0).expect("the short wall").id;
        let end = session.copy_face(CopyFace { body: block, face: cap }).expect("the cap");
        let _cap = session.document();
        let side = session.copy_face(CopyFace { body: block, face: wall }).expect("the wall");
        let _wall = session.document();
        let made = session.stitch(Stitch { sheets: vec![end, side], gap: Gap { mm: 0.0001 } }).expect("a stitch");
        let _sheet = session.document();
        assert_ne!(made, block, "the sheet is the box");
        let sewn: Vec<_> = session.project.timeline.iter().filter(|node| node.name == "feat-name-stitch").collect();
        assert_eq!(sewn.len(), 1, "the stitch is one node");
        assert_eq!(sewn[0].id, made.id, "the returned id is not the stitch node");
        match &sewn[0].kind {
            FeatureKind::Stitch { parts, tol, body } => {
                assert_eq!(*parts, vec![end.id, side.id], "the sheets are not the cap and the wall");
                assert!((tol - 0.0001).abs() < 1e-9, "the gap is not 0.0001 mm");
                assert_eq!(*body, made.id, "the returned id is not the sheet");
            }
            _ => panic!("the node is a stitch"),
        }
        let consumed = session.project.consumed_bodies();
        assert!(consumed.contains(&end.id), "the cap is not consumed");
        assert!(consumed.contains(&side.id), "the wall is not consumed");
        assert!(!consumed.contains(&block.id), "the box is consumed");
        assert!(!consumed.contains(&made.id), "the stitch is consumed");
    }

    /// The trim node stores the cap, the crossing tool, and the point (2, 10, 30). The cap is consumed.
    #[test]
    fn a_trim_keeps_its_point() {
        let mut session = Session::empty();
        let tool_drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (4.0, -1.0) }, to: To { at: (12.0, 21.0) } }).expect("the tool");
        let tool = session.extrude(tool_drawn, Height { mm: 40.0 }).expect("the tool body");
        let _tool = session.document();
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("the box");
        let block = session.extrude(drawn, Height { mm: 30.0 }).expect("the box body");
        let doc = session.document();
        let cap = doc.bodies.iter().find(|body| body.id == block.id).expect("the box").faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the cap").id;
        let sheet = session.copy_face(CopyFace { body: block, face: cap }).expect("a copy");
        let _sheet = session.document();
        let kept = Keep { x: KeepX { mm: 2.0 }, y: KeepY { mm: 10.0 }, z: KeepZ { mm: 30.0 } };
        let made = session.trim(Trim { sheet, keep: kept, tool }).expect("a trim");
        let _piece = session.document();
        assert_ne!(made, block, "the piece is the box");
        let cuts: Vec<_> = session.project.timeline.iter().filter(|node| node.name == "feat-name-trim").collect();
        assert_eq!(cuts.len(), 1, "the trim is one node");
        assert_eq!(cuts[0].id, made.id, "the returned id is not the trim node");
        match &cuts[0].kind {
            FeatureKind::Trim { src, tool: stored, keep, body } => {
                assert_eq!(*src, sheet.id, "the sheet is not the cap");
                assert_eq!(*stored, tool.id, "the tool is not the crossing extrusion");
                assert!((keep[0] - 2.0).abs() < 1e-9 && (keep[1] - 10.0).abs() < 1e-9 && (keep[2] - 30.0).abs() < 1e-9, "the point is not (2, 10, 30)");
                assert_eq!(*body, made.id, "the returned id is not the piece");
            }
            _ => panic!("the node is a trim"),
        }
        let consumed = session.project.consumed_bodies();
        assert!(consumed.contains(&sheet.id), "the cap is not consumed");
        assert!(!consumed.contains(&tool.id), "the tool is consumed");
        assert!(!consumed.contains(&block.id), "the box is consumed");
        assert!(!consumed.contains(&made.id), "the trim is consumed");
        let wall = session.project.regen_faces.get(&tool.id).and_then(|faces| faces.iter().find(|face| (face.centroid.x - 4.0).abs() < 0.2 && face.normal[0] < -0.5)).expect("the wall at x = 4").id;
        let cutter = session.copy_face(CopyFace { body: tool, face: FaceId { id: wall } }).expect("the wall");
        let _cutter = session.document();
        let another = session.copy_face(CopyFace { body: block, face: cap }).expect("another cap");
        let _another = session.document();
        let second = session.trim(Trim { sheet: another, keep: kept, tool: cutter }).expect("a trim by the wall");
        let doc = session.document();
        let piece = doc.bodies.iter().find(|body| body.id == second.id).expect("the piece");
        let area = piece.faces.iter().map(|face| face.area.mm2).sum::<f64>();
        assert!((area - 80.0).abs() < 1.0, "the wall keeps {area} mm^2");
        let consumed = session.project.consumed_bodies();
        assert!(!consumed.contains(&cutter.id), "the copied wall is consumed");
    }

    /// The replacement node stores the box, the end cap, and the sheet lifted 10 mm. Both inputs are consumed.
    #[test]
    fn a_replace_keeps_its_face() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let doc = session.document();
        let cap = doc.bodies.iter().find(|body| body.id == block.id).expect("the box").faces.iter().find(|face| face.role == FaceRole::CapEnd).expect("the end cap").id;
        let sheet = session.offset_face(OffsetFace { body: block, face: cap, by: Outset { mm: 10.0 } }).expect("the lift");
        let _sheet = session.document();
        let made = session.replace_face(ReplaceFace { body: block, face: cap, surface: sheet }).expect("a replacement");
        let _body = session.document();
        assert_ne!(made, block, "the replacement is the box");
        let replaced: Vec<_> = session.project.timeline.iter().filter(|node| node.name == "feat-name-surface-replace").collect();
        assert_eq!(replaced.len(), 1, "the replacement is one node");
        assert_eq!(replaced[0].id, made.id, "the returned id is not the replacement node");
        match &replaced[0].kind {
            FeatureKind::SurfaceReplace { src, faces, surface, body } => {
                assert_eq!(*src, block.id, "the solid is not the box");
                assert_eq!(faces.query.picked_descs(), vec![cap.id], "the face is not the end cap");
                assert_eq!(*surface, sheet.id, "the sheet is not the lift");
                assert_eq!(*body, made.id, "the returned id is not the replacement");
            }
            _ => panic!("the node is a replacement"),
        }
        let consumed = session.project.consumed_bodies();
        assert!(consumed.contains(&block.id), "the box is not consumed");
        assert!(consumed.contains(&sheet.id), "the sheet is not consumed");
        assert!(!consumed.contains(&made.id), "the replacement is consumed");
    }

    /// The move node stores the box and the 100 mm shift along X. The box is consumed. The result is not.
    #[test]
    fn a_move_keeps_its_place() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let _box = session.document();
        let mut mat = qymcad_core::feature::PLACE_IDENTITY;
        mat[3] = 100.0;
        let made = session.move_body(Move { body: block, by: Place { mat } }).expect("a move");
        let _read = session.document();
        assert_ne!(made, block, "the move is the box");
        {
            let moved: Vec<_> = session.project.timeline.iter().filter(|node| node.name == "feat-name-move").collect();
            assert_eq!(moved.len(), 1, "the move is one node");
            assert_eq!(moved[0].id, made.id, "the returned id is not the move node");
            match &moved[0].kind {
                FeatureKind::Move { src, mat: stored, body } => {
                    assert_eq!(*src, block.id, "the body is not the box");
                    for index in 0..stored.len() {
                        assert!((stored[index] - mat[index]).abs() < 1e-9, "the placement differs at {index}");
                    }
                    assert_eq!(*body, made.id, "the returned id is not the move");
                }
                _ => panic!("the node is a move"),
            }
            let consumed = session.project.consumed_bodies();
            assert!(consumed.contains(&block.id), "the box is not consumed");
            assert!(!consumed.contains(&made.id), "the move is consumed");
            let faces = session.project.regen_faces.get(&made.id).expect("the faces");
            let top = faces.iter().max_by(|left, right| left.centroid.z.partial_cmp(&right.centroid.z).expect("a centre")).expect("the cap");
            assert!((top.centroid.x - 105.0).abs() < 1e-6, "the cap is at x {}", top.centroid.x);
            assert!((top.centroid.y - 10.0).abs() < 1e-6, "the cap is at y {}", top.centroid.y);
            assert!((top.centroid.z - 30.0).abs() < 1e-6, "the cap is at z {}", top.centroid.z);
        }
        let again = session.move_body(Move { body: made, by: Place { mat } }).expect("a second move");
        let _again = session.document();
        let nodes = session.project.timeline.iter().filter(|node| node.name == "feat-name-move").count();
        assert_eq!(nodes, 2, "the second move was not a node");
        let consumed = session.project.consumed_bodies();
        assert!(consumed.contains(&made.id), "the first move is not consumed");
        assert!(!consumed.contains(&again.id), "the second move is consumed");
        let faces = session.project.regen_faces.get(&again.id).expect("the second faces");
        let top = faces.iter().max_by(|left, right| left.centroid.z.partial_cmp(&right.centroid.z).expect("a centre")).expect("the cap");
        assert!((top.centroid.x - 205.0).abs() < 1e-6, "the cap is at x {}", top.centroid.x);
    }

    /// The part stores the 100 mm shift along X. The next box belongs to it, and the first box does not.
    #[test]
    fn a_part_keeps_its_place() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let _box = session.document();
        let first = session.project.current_ctx();
        let mut mat = qymcad_core::feature::PLACE_IDENTITY;
        mat[3] = 100.0;
        let made = session.part(Part { place: Place { mat } }).expect("a part");
        assert_eq!(session.project.current_ctx(), made.id, "the session is not in the new part");
        let stored = session.project.component_transform(made.id);
        for index in 0..stored.len() {
            assert!((stored[index] - mat[index]).abs() < 1e-9, "the placement differs at {index}");
        }
        let drawn2 = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let block2 = session.extrude(drawn2, Height { mm: 30.0 }).expect("an extrusion");
        let _read = session.document();
        assert_eq!(session.project.body_owner(block2.id), Some(made.id), "the new box is not in the new part");
        assert_eq!(session.project.body_owner(block.id), Some(first), "the first box left its part");
    }

    /// The mate stores a rigid joint between the origins of the two parts, and the session stays where it was.
    #[test]
    fn a_mate_keeps_its_parts() {
        use qymcad_core::feature::{AnchorRef, JointKind};
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let _box = session.document();
        let first = session.project.current_ctx();
        let mut mat = qymcad_core::feature::PLACE_IDENTITY;
        mat[3] = 100.0;
        let made = session.part(Part { place: Place { mat } }).expect("a part");
        let drawn2 = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let block2 = session.extrude(drawn2, Height { mm: 30.0 }).expect("an extrusion");
        let _second = session.document();
        let active = session.project.current_ctx();
        let joined = session.mate(Mate { stay: Stay { part: PartId { id: first } }, bring: Bring { part: made } }).expect("a mate");
        assert_eq!(session.project.current_ctx(), active, "the session left the part");
        let joint = session.project.joints.iter().find(|joint| joint.id == joined.id).expect("the mate");
        assert!(matches!(joint.kind, JointKind::Rigid), "the mate is not rigid");
        let stay = session.project.connector(joint.a).expect("the staying anchor");
        let bring = session.project.connector(joint.b).expect("the brought anchor");
        assert!(matches!(stay.anchor, AnchorRef::Origin), "the staying anchor is not the origin");
        assert!(matches!(bring.anchor, AnchorRef::Origin), "the brought anchor is not the origin");
        assert_eq!(stay.owner, first, "the staying anchor is on the other part");
        assert_eq!(bring.owner, made.id, "the brought anchor is on the other part");
        let _held = session.document();
        assert_eq!(session.project.body_owner(block.id), Some(first), "the first box left its part");
        assert_eq!(session.project.body_owner(block2.id), Some(made.id), "the second box left its part");
    }

    /// A free part reads 6, and a rigid mate between two free parts leaves the root and both parts at 6.
    #[test]
    fn a_part_tells_its_freedom() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let before = session.document();
        let first = session.project.current_ctx();
        let free = count_of(&before, first);
        assert_eq!(free, session.project.component_dof(first), "the read differs from the model");
        assert_eq!(free, 6, "the free part");
        let mut mat = qymcad_core::feature::PLACE_IDENTITY;
        mat[3] = 100.0;
        let made = session.part(Part { place: Place { mat } }).expect("a part");
        let drawn2 = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let block2 = session.extrude(drawn2, Height { mm: 30.0 }).expect("an extrusion");
        let _placed = session.document();
        let _joined = session.mate(Mate { stay: Stay { part: PartId { id: first } }, bring: Bring { part: made } }).expect("a mate");
        let held = session.document();
        let root = session.project.components.iter().find(|component| component.parent.is_none()).expect("the root").id;
        let fastened = [root, first, made.id];
        for id in fastened {
            let count = count_of(&held, id);
            assert_eq!(count, session.project.component_dof(id), "the read differs from the model");
            assert_eq!(count, 6, "the fastened component");
            assert!(!session.project.is_grounded(id), "the component is grounded");
        }
        assert_eq!(session.project.body_owner(block.id), Some(first), "the first box left its part");
        assert_eq!(session.project.body_owner(block2.id), Some(made.id), "the second box left its part");
    }

    /// A rigid mate and a slider travel of 25 mm along Z leave both mates unheld, and the brought part stays put.
    #[test]
    fn a_slide_argues_with_the_stay() {
        use qymcad_core::feature::{AnchorRef, JointKind};
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let _box = session.document();
        let first = session.project.current_ctx();
        let mut mat = qymcad_core::feature::PLACE_IDENTITY;
        mat[3] = 100.0;
        let made = session.part(Part { place: Place { mat } }).expect("a part");
        let drawn2 = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let block2 = session.extrude(drawn2, Height { mm: 30.0 }).expect("an extrusion");
        let active = session.project.current_ctx();
        let joined = session.mate(Mate { stay: Stay { part: PartId { id: first } }, bring: Bring { part: made } }).expect("a mate");
        let slider = session.slide(Slide { stay: Stay { part: PartId { id: first } }, bring: Bring { part: made }, by: Travel { mm: 25.0 } }).expect("a slide");
        assert_eq!(session.project.current_ctx(), active, "the session left the part");
        let held = session.document();
        let public: Vec<u64> = held.violated.iter().map(|mate| mate.id).collect();
        assert_eq!(public, session.project.mates_violated, "the read differs from the model");
        assert_eq!(public, vec![joined.id, slider.id], "the mates that argue");
        let joint = session.project.joints.iter().find(|joint| joint.id == slider.id).expect("the slider");
        assert!(matches!(joint.kind, JointKind::Slider), "the mate is not a slider");
        let stay = session.project.connector(joint.a).expect("the staying anchor");
        let bring = session.project.connector(joint.b).expect("the brought anchor");
        assert!(matches!(stay.anchor, AnchorRef::Origin), "the staying anchor is not the origin");
        assert!(matches!(bring.anchor, AnchorRef::Origin), "the brought anchor is not the origin");
        assert_eq!(stay.owner, first, "the staying anchor is on the other part");
        assert_eq!(bring.owner, made.id, "the brought anchor is on the other part");
        let drive = joint.drive[1].expect("the travel");
        assert!((drive - 25.0).abs() < 1e-9, "the travel");
        assert!(joint.offset.abs() < 1e-9, "the offset");
        let root = session.project.components.iter().find(|component| component.parent.is_none()).expect("the root").id;
        for id in [root, first, made.id] {
            let count = count_of(&held, id);
            assert_eq!(count, session.project.component_dof(id), "the read differs from the model");
            assert_eq!(count, 6, "the fastened component");
            assert!(!session.project.is_grounded(id), "the component is grounded");
        }
        assert_eq!(session.project.current_ctx(), active, "the session left the part");
        assert_eq!(session.project.body_owner(block.id), Some(first), "the first box left its part");
        assert_eq!(session.project.body_owner(block2.id), Some(made.id), "the second box left its part");
    }

    /// The 10 by 20 by 30 mm box, written after it has been read, comes back as the same solid.
    #[test]
    fn a_step_brings_the_box_back() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let _block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let _doc = session.document();
        let path = std::env::temp_dir().join(format!("qymcad-drive-step-crate-{}.step", std::process::id()));
        let _ = std::fs::remove_file(&path);
        session.save_step(&path).expect("the file");
        let solids = Session::open_step(&path).expect("the read");
        let solid = solids.first().unwrap_or_else(|| panic!("the box is not in the file"));
        let name = path.to_str().expect("the path");
        let gate = qymcad_kernel::kernel_gate();
        let read = qymcad_kernel::step_solids(name).expect("the solids");
        let kernel = read.first().expect("the kernel solid").volume();
        drop(gate);
        let Measure::Live { volume, solids: count } = solid.measure else { panic!("the solid has no volume") };
        assert!((volume.mm3 - kernel).abs() < 1.0, "the read differs from the file");
        assert_eq!(count, 1, "the solid");
        assert_eq!(solid.sound, Sound::Valid, "the solid");
        let _ = std::fs::remove_file(&path);
    }

    /// The 10 by 20 by 30 mm box, written after it has been read, comes back as the same mesh.
    #[test]
    fn a_mesh_brings_the_box_back() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let _block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let _doc = session.document();
        let path = std::env::temp_dir().join(format!("qymcad-drive-stl-crate-{}.stl", std::process::id()));
        let _ = std::fs::remove_file(&path);
        session.save_stl(&path).expect("the file");
        let read = Session::open_stl(&path).expect("the read");
        if read.volume.mm3.abs() < 1e-12 {
            panic!("the box is not in the file");
        }
        let name = path.to_str().expect("the path");
        let mesh = qymcad_io::import_stl(name).expect("the mesh");
        assert!((read.volume.mm3 - mesh.volume()).abs() < 1.0, "the read differs from the file");
        assert_eq!(read.triangles, u32::try_from(mesh.tris.len()).expect("the triangles"), "the mesh");
        let _ = std::fs::remove_file(&path);
    }

    /// A part shifted 100 mm along X comes back with the corner the file's bounding box reports.
    #[test]
    fn a_part_stands_in_the_file() {
        let mut session = Session::empty();
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let _block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let _placed = session.part(Part { place: Place { mat: [1.0, 0.0, 0.0, 100.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0] } }).expect("a part");
        let drawn2 = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let _block2 = session.extrude(drawn2, Height { mm: 30.0 }).expect("an extrusion");
        let _doc = session.document();
        let path = std::env::temp_dir().join(format!("qymcad-drive-step-place-{}.step", std::process::id()));
        let _ = std::fs::remove_file(&path);
        session.save_step(&path).expect("the file");
        let solids = Session::open_step(&path).expect("the read");
        let name = path.to_str().expect("the path");
        let gate = qymcad_kernel::kernel_gate();
        let read = qymcad_kernel::step_solids(name).expect("the solids");
        assert_eq!(solids.len(), read.len(), "the file");
        for (solid, shape) in solids.iter().zip(read.iter()) {
            let bounds = shape.bbox().expect("the box");
            same_x(solid.low.x, bounds[0], "the low corner");
            same_y(solid.low.y, bounds[1], "the low corner");
            same_z(solid.low.z, bounds[2], "the low corner");
            same_x(solid.high.x, bounds[3], "the high corner");
            same_y(solid.high.y, bounds[4], "the high corner");
            same_z(solid.high.z, bounds[5], "the high corner");
        }
        drop(gate);
        let _ = std::fs::remove_file(&path);
    }

    /// A part shifted 100 mm along X comes back with the corner the file's vertices report.
    #[test]
    fn a_part_stands_in_the_mesh() {
        let mut session = Session::empty();
        let _placed = session.part(Part { place: Place { mat: [1.0, 0.0, 0.0, 100.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0] } }).expect("a part");
        let drawn = session.rectangle(WorldPlane::Xy, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
        let block = session.extrude(drawn, Height { mm: 30.0 }).expect("an extrusion");
        let _doc = session.document();
        let path = std::env::temp_dir().join(format!("qymcad-drive-stl-place-{}.stl", std::process::id()));
        let _ = std::fs::remove_file(&path);
        session.save_stl(&path).expect("the file");
        let read = Session::open_stl(&path).expect("the read");
        let stored = session.project.bodies.iter().find(|body| body.id == block.id).expect("the body");
        let kept = stored.mesh.bounds().expect("the box");
        same_x(CornerX { mm: kept.min.x }, 0.0, "the stored mesh");
        same_y(CornerY { mm: kept.min.y }, 0.0, "the stored mesh");
        same_z(CornerZ { mm: kept.min.z }, 0.0, "the stored mesh");
        same_x(CornerX { mm: kept.max.x }, 10.0, "the stored mesh");
        same_y(CornerY { mm: kept.max.y }, 20.0, "the stored mesh");
        same_z(CornerZ { mm: kept.max.z }, 30.0, "the stored mesh");
        let name = path.to_str().expect("the path");
        let mesh = qymcad_io::import_stl(name).expect("the mesh");
        let bounds = mesh.bounds().expect("the box");
        same_x(read.low.x, bounds.min.x, "the low corner");
        same_y(read.low.y, bounds.min.y, "the low corner");
        same_z(read.low.z, bounds.min.z, "the low corner");
        same_x(read.high.x, bounds.max.x, "the high corner");
        same_y(read.high.y, bounds.max.y, "the high corner");
        same_z(read.high.z, bounds.max.z, "the high corner");
        let _ = std::fs::remove_file(&path);
    }

    fn same_x(got: CornerX, mm: f64, what: &str) {
        assert!((got.mm - mm).abs() < 1e-12, "{what}");
    }

    fn same_y(got: CornerY, mm: f64, what: &str) {
        assert!((got.mm - mm).abs() < 1e-12, "{what}");
    }

    fn same_z(got: CornerZ, mm: f64, what: &str) {
        assert!((got.mm - mm).abs() < 1e-12, "{what}");
    }

    fn count_of(doc: &Document, id: u64) -> u8 {
        doc.components.iter().find(|component| component.id == id).expect("the component").freedom.count
    }
}
