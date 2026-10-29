//! A line from another process, read at the start of a frame.
//!
//! The socket path is the environment variable `QYMCAD_COMMAND_QUEUE`. Unset, the frame does not
//! listen and the window is unchanged. One connection at a time, one line: the word `open`, a space,
//! and the path. The listener is not stored on the window.

use std::io::Read;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::Mutex;

use super::*;

/// A path longer than this cannot be opened, and a client that never ends the line must not grow the buffer.
const MAX_LINE: usize = 4096;

struct Queue {
    listener: UnixListener,
    stream: Option<UnixStream>,
    buf: Vec<u8>,
}

static QUEUE: Mutex<Option<Queue>> = Mutex::new(None);

/// The thread a check claimed, and the socket that thread listens on.
struct Claim {
    thread: std::thread::ThreadId,
    socket: std::ffi::OsString,
}

static OWNER: Mutex<Option<Claim>> = Mutex::new(None);

/// Where the socket path was found.
struct Heard {
    /// A copy started beside an already open program does not listen, even when the socket path is set.
    side: bool,
    variable: Option<std::ffi::OsString>,
    claimed: Option<std::ffi::OsString>,
    who: Who,
}

enum Who {
    Nobody,
    Owner,
    Other,
}

/// The socket to bind. A claimed listener belongs to that thread; every other thread stays away from it.
/// With nobody claiming, the environment variable is the path. Empty is the same as unset.
fn choose(heard: Heard) -> Option<std::ffi::OsString> {
    if heard.side {
        return None;
    }
    let value = match heard.who {
        Who::Other => return None,
        Who::Owner => heard.claimed,
        Who::Nobody => heard.variable,
    };
    value.filter(|path| !path.is_empty())
}

/// Read one `open` line, if a client has written one, and hand the path to the same door a recent file uses.
///
/// The command waits while a file is already opening, or while the person is already answering a question
/// about this document. Unsaved work is asked about; a window that has not taken its baseline yet has nothing
/// to lose, because the greeting is still up and no edit can have been made.
pub(crate) fn take_open(app: &mut App, ctx: &egui::Context) {
    // A copy started by another program does not come from a click. Asked on the first frame, once
    // the window exists: earlier than that the request has nowhere to go and the copy stays behind
    // the program that is already open.
    raise_a_side_copy_once(ctx);
    if app.regen.busy.is_some() || app.deferred.nav.is_some() || app.disk.pending_nav.is_some() || app.disk.file_ask.is_some() {
        return;
    }
    let Some(path) = next_path() else { return };
    if app.disk.edits.ready {
        app.request_nav(Nav::OpenPath(path), ctx);
    } else {
        open_recent(&mut app.regen, &mut app.set, &mut app.status, path);
    }
}

/// This check's thread owns `socket` until [`release`]. The process environment is left alone.
#[cfg(test)]
pub(crate) fn claim(socket: &std::path::Path) {
    *lock(&OWNER) = Some(Claim { thread: std::thread::current().id(), socket: socket.as_os_str().to_os_string() });
}

/// Close the listener and give the socket back.
#[cfg(test)]
pub(crate) fn release() {
    *lock(&OWNER) = None;
    *lock(&QUEUE) = None;
}

fn raise_a_side_copy_once(ctx: &egui::Context) {
    if !crate::crash::side_session(std::env::var_os("QYMCAD_SIDE").as_deref()) {
        return;
    }
    static ASKED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if ASKED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    for command in side_copy_placement() {
        ctx.send_viewport_cmd(command);
    }
    // The desktop does not raise a start that did not come from a click, so the copy can sit behind
    // the program that is already open. A notice with the file's name is drawn on top of whatever is open.
    announce_the_file();
}

fn side_copy_placement() -> Vec<egui::ViewportCommand> {
    vec![egui::ViewportCommand::RequestUserAttention(egui::UserAttentionType::Informational)]
}

fn announce_the_file() {
    let qymcad_ui_state::AskedFile::Path(path) = qymcad_ui_state::asked_file(std::env::args().skip(1)) else { return };
    let title = super::window_title::window_title(Some(&path), false);
    let _ = std::process::Command::new("notify-send")
        .arg("--icon=tech.qymis.cad")
        .arg(title)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn socket_of() -> Option<std::ffi::OsString> {
    let side = crate::crash::side_session(std::env::var_os("QYMCAD_SIDE").as_deref());
    let owner = lock(&OWNER);
    let heard = match owner.as_ref() {
        Some(claim) if claim.thread == std::thread::current().id() => Heard { side, variable: None, claimed: Some(claim.socket.clone()), who: Who::Owner },
        Some(_) => Heard { side, variable: None, claimed: None, who: Who::Other },
        None => Heard { side, variable: std::env::var_os("QYMCAD_COMMAND_QUEUE"), claimed: None, who: Who::Nobody },
    };
    choose(heard)
}

fn next_path() -> Option<String> {
    let socket = socket_of()?;
    let mut slot = lock(&QUEUE);
    if slot.as_ref().is_some_and(|_| !socket_file(&socket)) {
        // The file is gone, so this listener can take no new client. Bind the path again.
        *slot = None;
    }
    if slot.is_none() {
        let listener = listen(&socket)?;
        *slot = Some(Queue { listener, stream: None, buf: Vec::new() });
    }
    let held = slot.as_mut()?;
    if held.stream.is_none() {
        match held.listener.accept() {
            Ok((stream, _)) => {
                if stream.set_nonblocking(true).is_err() {
                    return None;
                }
                held.stream = Some(stream);
            }
            Err(_) => return None,
        }
    }
    read_line(held)
}

/// Read until one line is complete, the client closes, or the socket has nothing more this frame.
///
/// Closing after the path ends the line, the same as a newline: otherwise the command sits until the
/// client is gone and is then thrown away, and the file never opens.
fn read_line(held: &mut Queue) -> Option<String> {
    let mut tmp = [0u8; 1024];
    loop {
        let read = held.stream.as_mut()?.read(&mut tmp);
        match read {
            Ok(0) => return ended(held),
            Ok(n) => {
                held.buf.extend_from_slice(&tmp[..n]);
                if held.buf.contains(&b'\n') {
                    return finished_line(held);
                }
                if held.buf.len() > MAX_LINE {
                    held.stream = None;
                    held.buf.clear();
                    return None;
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => return None,
            Err(_) => {
                held.stream = None;
                held.buf.clear();
                return None;
            }
        }
    }
}

/// The client closed. Whatever was written, with or without a newline, is the line.
fn ended(held: &mut Queue) -> Option<String> {
    let path = if held.buf.is_empty() {
        None
    } else {
        held.buf.push(b'\n');
        finished_line(held)
    };
    held.stream = None;
    held.buf.clear();
    path
}

fn socket_file(path: &std::ffi::OsStr) -> bool {
    std::fs::metadata(path).ok().is_some_and(|meta| meta.file_type().is_socket())
}

/// Bind `path`. A stale socket is removed and the bind tried once more. Any other file is left where it is.
fn listen(path: &std::ffi::OsStr) -> Option<UnixListener> {
    if let Some(listener) = bound(path) {
        return Some(listener);
    }
    if !std::fs::metadata(path).ok().is_some_and(|meta| meta.file_type().is_socket()) {
        return None;
    }
    let _ = std::fs::remove_file(path);
    bound(path)
}

fn bound(path: &std::ffi::OsStr) -> Option<UnixListener> {
    let listener = UnixListener::bind(path).ok()?;
    listener.set_nonblocking(true).ok()?;
    // The socket can open a document, so only the user who started the program can connect.
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    Some(listener)
}

/// The first complete line. Anything that is not `open` and a path is dropped.
fn finished_line(held: &mut Queue) -> Option<String> {
    let end = held.buf.iter().position(|byte| *byte == b'\n')?;
    let raw: Vec<u8> = held.buf.drain(..=end).collect();
    held.stream = None;
    held.buf.clear();
    let text = String::from_utf8(raw).ok()?;
    let rest = text.trim().strip_prefix("open ")?;
    let path = rest.trim();
    if path.is_empty() {
        None
    } else {
        Some(path.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{choose, Heard, Who};

    fn text(path: &str) -> std::ffi::OsString {
        std::ffi::OsString::from(path)
    }

    #[test]
    fn the_variable_names_the_socket_when_nobody_claimed_it() {
        let heard = Heard { side: false, variable: Some(text("/tmp/q.sock")), claimed: None, who: Who::Nobody };
        assert_eq!(choose(heard).as_deref(), Some(std::ffi::OsStr::new("/tmp/q.sock")));
    }

    #[test]
    fn an_empty_variable_does_not_listen() {
        let heard = Heard { side: false, variable: Some(std::ffi::OsString::new()), claimed: None, who: Who::Nobody };
        assert!(choose(heard).is_none());
    }

    #[test]
    fn another_thread_does_not_take_a_claimed_socket() {
        let heard = Heard { side: false, variable: Some(text("/tmp/q.sock")), claimed: Some(text("/tmp/ours.sock")), who: Who::Other };
        assert!(choose(heard).is_none());
    }

    #[test]
    fn the_claiming_thread_listens_on_its_own_socket() {
        let heard = Heard { side: false, variable: Some(text("/tmp/q.sock")), claimed: Some(text("/tmp/ours.sock")), who: Who::Owner };
        assert_eq!(choose(heard).as_deref(), Some(std::ffi::OsStr::new("/tmp/ours.sock")));
    }

    #[test]
    fn a_side_copy_does_not_listen_even_when_the_socket_is_named() {
        let heard = Heard { side: true, variable: Some(text("/tmp/q.sock")), claimed: None, who: Who::Nobody };
        assert!(choose(heard).is_none(), "the copy would take the socket from the program that is already open");
    }

    /// A COPY IS NOT MINIMIZED. This desktop does not say when a window was minimized, and the program
    /// cannot undo it, so the file would stay hidden.
    #[test]
    fn a_side_copy_is_not_minimized() {
        let placed = super::side_copy_placement();
        assert!(placed.iter().all(|command| !matches!(command, egui::ViewportCommand::Minimized(_))), "the copy is minimized and cannot be brought back");
    }
}
