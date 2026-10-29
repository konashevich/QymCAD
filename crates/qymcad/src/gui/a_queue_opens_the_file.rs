use qymcad_drive::{Corners, From, Height, Session, To, WorldPlane};

const MISSING: &str = "the file did not open";

/// A client asks the open window to read a saved document. The window's own open door loads it.
#[test]
fn the_queue_opens_the_file() {
    let filed = Filed::new();
    write_box(&filed);
    let socket = Socket::new();
    let _held = Held::set(&socket);
    let mut window = super::window::Window::new(egui::vec2(1280.0, 800.0));
    an_unsaved_document_is_asked(&mut window, &socket, &filed);
    // The socket file can disappear while the program is open. The next frame has to bind it again,
    // or a later command connects to nothing.
    let _ = std::fs::remove_file(&socket.path);
    let mut app = super::App::default();
    window.run(&mut app, egui::Modifiers::default(), Vec::new(), false);
    send(&socket, &filed, End::Closed);
    window.run(&mut app, egui::Modifiers::default(), Vec::new(), false);
    app.drain_busy_for_test();
    the_box(&app);
}

/// A document with an edit is not replaced. The command waits in the same question the menu asks.
fn an_unsaved_document_is_asked(window: &mut super::window::Window, socket: &Socket, filed: &Filed) {
    let mut app = super::App::default();
    // The greeting holds the first frames for five seconds, and the baseline is taken only once it steps
    // aside. Clear it so this frame takes the baseline, as a frame does once the greeting is over.
    app.waiting.splash_until = None;
    window.run(&mut app, egui::Modifiers::default(), Vec::new(), false);
    let si = app.create_sketch_on(qymcad_core::feature::SketchPlane::default());
    app.project.add_rect_entity(si, 0.0, 0.0, 5.0, 5.0, qymcad_core::feature::Purpose::Real);
    app.project.regen_sketch(si);
    app.finish_sketch_edit();
    if !qymcad_ui_state::is_dirty(&mut app.rebuild_ctx()) {
        panic!("the document was not unsaved");
    }
    let sketches = app.project.sketches.len();
    send(socket, filed, End::Line);
    window.run(&mut app, egui::Modifiers::default(), Vec::new(), false);
    app.drain_busy_for_test();
    let asked = matches!(&app.deferred.nav, Some(super::Nav::OpenPath(path)) if path == &filed.path.display().to_string());
    if !asked || app.project.sketches.len() != sketches || !app.project.bodies.is_empty() {
        panic!("the unsaved document was replaced");
    }
}

struct Filed {
    path: std::path::PathBuf,
}

impl Filed {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("qymcad-queue-box-{}.qcad", std::process::id()));
        remove(&path);
        Self { path }
    }
}

impl Drop for Filed {
    fn drop(&mut self) {
        remove(&self.path);
    }
}

fn remove(path: &std::path::Path) {
    let text = path.display().to_string();
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{text}.bak"));
    let _ = std::fs::remove_file(format!("{text}.tmp~"));
}

fn write_box(filed: &Filed) {
    let mut session = Session::empty();
    let drawn = session.rectangle(WorldPlane::Xz, Corners { from: From { at: (0.0, 0.0) }, to: To { at: (10.0, 20.0) } }).expect("a rectangle");
    let _block = session.extrude(drawn, Height { mm: 15.0 }).expect("an extrusion");
    let _read = session.document();
    session.save(&filed.path).expect("the file");
}

struct Socket {
    path: std::path::PathBuf,
}

impl Socket {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("qymcad-queue-{}.sock", std::process::id()));
        let _ = std::fs::remove_file(&path);
        Self { path }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

struct Held {
    socket: std::path::PathBuf,
}

impl Held {
    fn set(socket: &Socket) -> Self {
        let held = Self { socket: socket.path.clone() };
        super::command_queue::claim(&held.socket);
        held
    }
}

impl Drop for Held {
    fn drop(&mut self) {
        super::command_queue::release();
        let _ = std::fs::remove_file(&self.socket);
    }
}

/// How the client finishes the command. Closing the connection ends the line, as a newline does.
enum End {
    Line,
    Closed,
}

fn send(socket: &Socket, filed: &Filed, end: End) {
    let mut stream = std::os::unix::net::UnixStream::connect(&socket.path).expect("the command was not delivered");
    let mut bytes = format!("open {}", filed.path.display());
    if matches!(end, End::Line) {
        bytes.push('\n');
    }
    std::io::Write::write_all(&mut stream, bytes.as_bytes()).expect("the command was not delivered");
}

fn the_box(app: &super::App) {
    if app.project.bodies.len() != 1 || app.visible_mesh_count() != 1 {
        panic!("{MISSING}");
    }
    let volume = app.project.bodies[0].mesh.volume();
    if (volume - 10.0 * 20.0 * 15.0).abs() >= 1.0 || app.status.words != "The project is loaded." {
        panic!("{MISSING}");
    }
}
