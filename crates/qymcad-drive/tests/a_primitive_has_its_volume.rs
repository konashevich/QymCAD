use qymcad_drive::{BodyId, BottomRadius, BoxLength, Cone, Cuboid, Cylinder, DrawError, Height, Measure, Prism, Radius, Rebuild, Ring, Session, Sides, Sound, TopRadius, Torus, Tube, Width};

/// A box of 10 by 20 by 30 mm is 6000 mm^3, one solid, and the read is quiet.
#[test]
fn a_box_has_the_volume_of_its_three_sizes() {
    let mut session = Session::empty();
    let made = session.cuboid(box_10_by_20_by_30()).expect("a box");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), 6000.0, "the box");
    assert_eq!(doc.nodes.iter().filter(|node| node.name == "feat-name-box").count(), 1, "the box node");
}

/// A cylinder of radius 5 mm and height 10 mm is 250 pi mm^3.
#[test]
fn a_cylinder_has_the_volume_of_its_radius_and_height() {
    let mut session = Session::empty();
    let made = session.cylinder(Cylinder { radius: Radius { mm: 5.0 }, height: Height { mm: 10.0 } }).expect("a cylinder");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), 250.0 * std::f64::consts::PI, "the cylinder");
}

/// A sphere of radius 10 mm is 4000/3 pi mm^3.
#[test]
fn a_sphere_has_the_volume_of_its_radius() {
    let mut session = Session::empty();
    let made = session.sphere(Radius { mm: 10.0 }).expect("a sphere");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), 4000.0 / 3.0 * std::f64::consts::PI, "the sphere");
}

/// A sharp cone and a frustum have the volumes of their radii.
#[test]
fn a_cone_has_the_volume_of_its_radii() {
    let mut sharp = Session::empty();
    let point = sharp.cone(cone(10.0, 0.0, 20.0)).expect("a sharp cone");
    let doc = sharp.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, point), 2000.0 / 3.0 * std::f64::consts::PI, "the sharp cone");

    let mut blunt = Session::empty();
    let frustum = blunt.cone(cone(10.0, 5.0, 20.0)).expect("a frustum");
    let doc = blunt.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, frustum), 3500.0 / 3.0 * std::f64::consts::PI, "the frustum");
}

/// A torus of ring 12 mm and tube 4 mm is 384 pi^2 mm^3.
#[test]
fn a_torus_has_the_volume_of_its_ring_and_tube() {
    let mut session = Session::empty();
    let made = session.torus(Torus { ring: Ring { mm: 12.0 }, tube: Tube { mm: 4.0 } }).expect("a torus");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), 384.0 * std::f64::consts::PI * std::f64::consts::PI, "the torus");
}

/// A six-sided prism and a three-sided prism have the volumes of their polygons.
#[test]
fn a_prism_has_the_volume_of_its_polygon() {
    let mut six = Session::empty();
    let hex = six.prism(prism(10.0, 20.0, 6)).expect("a six-sided prism");
    let doc = six.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, hex), 3000.0 * 3.0_f64.sqrt(), "the six-sided prism");

    let mut three = Session::empty();
    let triangle = three.prism(prism(10.0, 20.0, 3)).expect("a three-sided prism");
    let doc = three.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, triangle), 1500.0 * 3.0_f64.sqrt(), "the three-sided prism");
}

/// A cylinder of radius 4 mm sits inside the box, so the union is still the box, and it is neither operand.
#[test]
fn a_cylinder_inside_a_box_is_one_body() {
    let mut session = Session::empty();
    let block = session.cuboid(box_10_by_20_by_30()).expect("a box");
    let joined = session.cylinder(Cylinder { radius: Radius { mm: 4.0 }, height: Height { mm: 10.0 } }).expect("a cylinder");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    let cylinder = doc.nodes.iter().find(|node| node.name == "feat-name-cylinder").expect("the cylinder node").id;
    assert_ne!(joined.id, block.id, "the union is the box");
    assert_ne!(joined.id, cylinder, "the union is the cylinder");
    assert_eq!(doc.nodes.iter().find(|node| node.id == joined.id).expect("the union").name, "feat-name-body-union");
    near(live(&doc, joined), 6000.0, "the union");
}

/// A bad length is the length key, before a node, and a later box is quiet at 6000 mm^3.
#[test]
fn a_bad_box_size_is_refused_before_a_node() {
    let mut session = Session::empty();
    let length = session_cuboid(&mut session, f64::INFINITY, 20.0, 30.0);
    refuse(&mut session, length, "sketch-box-length", "feat-name-box");
    let width = session_cuboid(&mut session, 10.0, 0.0, 30.0);
    refuse(&mut session, width, "sketch-box-width", "feat-name-box");
    let height = session_cuboid(&mut session, 10.0, 20.0, f64::NEG_INFINITY);
    refuse(&mut session, height, "sketch-box-height", "feat-name-box");
    let made = session.cuboid(box_10_by_20_by_30()).expect("a box");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), 6000.0, "the box after the refusals");
}

fn session_cuboid(session: &mut Session, length: f64, width: f64, height: f64) -> Result<BodyId, DrawError> {
    session.cuboid(Cuboid { length: BoxLength { mm: length }, width: Width { mm: width }, height: Height { mm: height } })
}

/// A bad cylinder radius is refused before a node, and a later cylinder has its own volume.
#[test]
fn a_bad_cylinder_radius_is_refused_before_a_node() {
    let mut session = Session::empty();
    let bad = session.cylinder(Cylinder { radius: Radius { mm: 0.0 }, height: Height { mm: 10.0 } });
    refuse(&mut session, bad, "sketch-cylinder-radius", "feat-name-cylinder");
    let made = session.cylinder(Cylinder { radius: Radius { mm: 5.0 }, height: Height { mm: 10.0 } }).expect("a cylinder");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), 250.0 * std::f64::consts::PI, "the cylinder after the refusal");
}

/// A sphere radius that is not a number is refused before a node.
#[test]
fn a_bad_sphere_radius_is_refused_before_a_node() {
    let mut session = Session::empty();
    let bad = session.sphere(Radius { mm: f64::NAN });
    refuse(&mut session, bad, "sketch-sphere-radius", "feat-name-sphere");
    let made = session.sphere(Radius { mm: 10.0 }).expect("a sphere");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), 4000.0 / 3.0 * std::f64::consts::PI, "the sphere after the refusal");
}

/// A cone with no base is the bottom key, even when the top is also zero. A negative top is the top key.
#[test]
fn a_bad_cone_is_refused_before_a_node() {
    let mut session = Session::empty();
    let bottom = session.cone(cone(0.0, 0.0, 20.0));
    refuse(&mut session, bottom, "sketch-cone-bottom", "feat-name-cone");
    let top = session.cone(cone(10.0, -1.0, 20.0));
    refuse(&mut session, top, "sketch-cone-top", "feat-name-cone");
    let height = session.cone(cone(10.0, 0.0, 0.0));
    refuse(&mut session, height, "sketch-cone-height", "feat-name-cone");
    let made = session.cone(cone(10.0, 0.0, 20.0)).expect("a sharp cone");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), 2000.0 / 3.0 * std::f64::consts::PI, "the cone after the refusals");
}

/// A zero ring is the ring key. A tube as thick as the ring is the existing through-itself key, before a node.
#[test]
fn a_bad_torus_is_refused_before_a_node() {
    let mut session = Session::empty();
    let ring = session.torus(Torus { ring: Ring { mm: 0.0 }, tube: Tube { mm: 4.0 } });
    refuse(&mut session, ring, "sketch-torus-ring", "feat-name-torus");
    let tube = session.torus(Torus { ring: Ring { mm: 12.0 }, tube: Tube { mm: 0.0 } });
    refuse(&mut session, tube, "sketch-torus-tube", "feat-name-torus");
    let through = session.torus(Torus { ring: Ring { mm: 5.0 }, tube: Tube { mm: 5.0 } });
    refuse(&mut session, through, "error-torus-through-itself", "feat-name-torus");
    let made = session.torus(Torus { ring: Ring { mm: 12.0 }, tube: Tube { mm: 4.0 } }).expect("a torus");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), 384.0 * std::f64::consts::PI * std::f64::consts::PI, "the torus after the refusals");
}

/// A side count of 0, 2, or 65 is refused before a node. A count of 3 builds.
#[test]
fn a_bad_prism_is_refused_before_a_node() {
    let mut session = Session::empty();
    let radius = session.prism(prism(0.0, 20.0, 6));
    refuse(&mut session, radius, "sketch-prism-radius", "feat-name-prism");
    let height = session.prism(prism(10.0, 0.0, 6));
    refuse(&mut session, height, "sketch-prism-height", "feat-name-prism");
    for count in [0, 2, 65] {
        let sides = session.prism(prism(10.0, 20.0, count));
        refuse(&mut session, sides, "sketch-prism-sides", "feat-name-prism");
    }
    let made = session.prism(prism(10.0, 20.0, 3)).expect("a three-sided prism");
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    near(live(&doc, made), 1500.0 * 3.0_f64.sqrt(), "the prism after the refusals");
}

fn cone(bottom: f64, top: f64, height: f64) -> Cone {
    Cone { bottom: BottomRadius { mm: bottom }, top: TopRadius { mm: top }, height: Height { mm: height } }
}

fn prism(radius: f64, height: f64, count: u32) -> Prism {
    Prism { radius: Radius { mm: radius }, height: Height { mm: height }, sides: Sides { count } }
}

fn box_10_by_20_by_30() -> Cuboid {
    Cuboid { length: BoxLength { mm: 10.0 }, width: Width { mm: 20.0 }, height: Height { mm: 30.0 } }
}

fn refuse(session: &mut Session, result: Result<BodyId, DrawError>, key: &str, name: &str) {
    let err = result.expect_err("the size");
    assert_eq!(err.text, key, "the refusal was {}", err.text);
    let doc = session.document();
    assert_eq!(doc.rebuild, Rebuild::Quiet, "{}", refusal(&doc));
    assert!(doc.nodes.iter().all(|node| node.name != name), "a node was written");
}

fn live(doc: &qymcad_drive::Document, id: BodyId) -> f64 {
    let body = doc.bodies.iter().find(|body| body.id == id.id).unwrap_or_else(|| panic!("body {} is gone", id.id));
    assert_eq!(body.sound, Sound::Valid, "the shape is valid");
    match body.measure {
        Measure::Live { volume, solids } => {
            assert_eq!(solids, 1, "one solid");
            volume.mm3
        }
        Measure::Absent => panic!("body {} has no live shape", id.id),
    }
}

fn near(got: f64, want: f64, what: &str) {
    assert!((got - want).abs() < 1.0, "{what}: volume {got} mm^3, wanted {want} within 1");
}

fn refusal(doc: &qymcad_drive::Document) -> String {
    match &doc.rebuild {
        Rebuild::Refused { refusals } => refusals.iter().map(|refusal| refusal.text.clone()).collect::<Vec<_>>().join(" "),
        Rebuild::Quiet => "quiet".into(),
        Rebuild::Stopped => "stopped".into(),
    }
}
