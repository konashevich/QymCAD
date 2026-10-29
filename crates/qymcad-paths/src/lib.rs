//! WHERE THE PROGRAM KEEPS ITS OWN FILES.
//!
//! THE APPLICATION ID IS ONE STRING, and this is it. The same name identifies the program to the desktop,
//! to macOS and to Flathub. It was written out in six places in the code, plus the macOS bundle, plus the
//! Flatpak manifest - eight copies of one decision, and three of them had already drifted apart:
//! `tech.qymis.qym-cad` in the code, `tech.qymis.qymcad` on macOS, `tech.qymis.cad` on Flathub.
//!
//! THE FOLDER IS A SECOND DECISION, next to it and not derived from it - see `FOLDER` below for what
//! deriving it cost.
//!
//! CHANGING EITHER LOSES WHAT PEOPLE HAVE. The old directory does not move by itself: settings, schemes,
//! templates and the parts library stay behind under the old name and simply vanish from view. That is the
//! price, it was paid deliberately while the audience is two dev releases small, and it is the reason both
//! names live here rather than in eight files.

/// The reverse-DNS name of the program: `cad.qymis.tech` backwards, the project's own site.
pub const APP_ID: &str = "tech.qymis.cad";

/// THE NAME OF THE FOLDER, and it is NOT the last part of the id.
///
/// Reported behaviour: settings and crash reports turned up in `~/.local/share/cad`.
///
/// `ProjectDirs` is given three parts and on Linux uses ONLY the third. `tech.qymis.cad` split at the
/// dots ends with `cad`, so the program claimed a folder named after a whole field of software - a name
/// any other CAD may take tomorrow - and stopped finding what it had already written under its own.
///
/// These are two decisions, not one. The id says who the program is to the desktop, to macOS and to
/// Flathub; the folder says where a person's settings, schemes, templates and library of parts live.
/// Deriving the second from the first is exactly what put a stranger's name on the folder.
const FOLDER: &str = "qymcad";

/// The two parts above the name, taken from `APP_ID` so the id and the folder stay in one file.
fn parts() -> Option<(&'static str, &'static str)> {
    let mut it = APP_ID.split('.');
    Some((it.next()?, it.next()?))
}

/// The program's own directories, or `None` where the system has no notion of them.
pub fn dirs() -> Option<directories::ProjectDirs> {
    let (tld, org) = parts()?;
    directories::ProjectDirs::from(tld, org, FOLDER)
}

/// A ROOT THAT STANDS IN FOR THE PERSON'S DIRECTORIES, for the whole process once set.
static ROOT: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// KEEP EVERYTHING THE PROGRAM WRITES UNDER `root`, for the rest of this process: settings, schemes,
/// templates, the parts library, crash and problem reports.
///
/// A program driven from outside - a run of acceptance checks - is still the whole program, and a whole
/// program writes where a person keeps their files. One such run on a working machine replaced a person's
/// settings and filled their crash folder. Set once: a second root would split one process between two
/// homes, so a different root after the first is refused and the answer is `false`.
pub fn keep_under(root: std::path::PathBuf) -> bool {
    ROOT.get_or_init(|| root.clone()) == &root
}

/// THE ONE FOLDER A PERSON'S FILES ARE IN - settings, schemes, templates, parts, reports: under the root when one was
/// given, the system's data folder of the program otherwise (`~/.local/share/qymcad`, `%APPDATA%\qymis\qymcad\data`,
/// `~/Library/Application Support/tech.qymis.qymcad`). One folder on every system, not a "config" beside a "data".
/// Reported behaviour: half a person's files were in `~/.config/qymcad` and half in `~/.local/share/qymcad`.
pub fn data_root() -> Option<std::path::PathBuf> {
    if let Some(root) = ROOT.get() {
        return Some(root.clone());
    }
    dirs().map(|d| d.data_dir().to_path_buf())
}

/// THE FILE THE SETTINGS ARE KEPT IN.
///
/// Named here, with the rest of the program's places, because the framework picks one of its own otherwise -
/// derived from the application id, which is a different string. On Linux both happen to land in the same
/// folder and nothing looks wrong; on Windows they came out as `AppData\Roaming\qymcad\data` against
/// `AppData\Roaming\qymis\qymcad\data`, and on macOS as `qymcad` beside `tech.qymis.qymcad` - a person's
/// settings in one place and their schemes, templates, parts and crash reports in another.
pub fn settings_file() -> Option<std::path::PathBuf> {
    data_root().map(|d| d.join("app.ron"))
}

/// THE SETTINGS OF A COPY STARTED BESIDE ONE THAT IS ALREADY OPEN.
///
/// That copy must not write `app.ron`: the file holds the project the person had open, and a second
/// process saving its own project there would make the next ordinary start open the copy's file.
pub fn side_settings_file() -> Option<std::path::PathBuf> {
    data_root().map(|d| d.join(side_settings_name()))
}

/// One file per copy. Two copies that share a name overwrite each other, and the first then
/// saves the second's project into the file it still has open.
fn side_settings_name() -> String {
    format!("side-app-{}.ron", std::process::id())
}

/// Write a sibling of the person's settings with the same bytes. The person's file is only read.
///
/// A missing source does not create the sibling: the copy then starts from the factory settings.
pub fn copy_settings_aside(person: &std::path::Path) -> bool {
    let Ok(bytes) = std::fs::read(person) else { return false };
    let Some(parent) = person.parent() else { return false };
    std::fs::write(parent.join(side_settings_name()), bytes).is_ok()
}

/// Remove the settings of copies whose process is gone. A copy that is still open keeps its file,
/// and `app.ron` is not a copy.
pub fn drop_finished_copies(dir: &std::path::Path) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let me = std::process::id();
    for entry in entries.flatten() {
        let Some(text) = entry.file_name().into_string().ok() else { continue };
        let Some(pid) = copy_pid(&text) else { continue };
        if pid == me || process_is_open(pid) {
            continue;
        }
        let _ = std::fs::remove_file(entry.path());
    }
}

fn copy_pid(name: &str) -> Option<u32> {
    name.strip_prefix("side-app-")?.strip_suffix(".ron")?.parse().ok()
}

/// On Linux a process that is still running has a directory under `/proc`. Elsewhere the file is
/// left, because a wrong guess would delete the settings of a copy that is still open.
fn process_is_open(pid: u32) -> bool {
    #[cfg(target_os = "linux")]
    {
        std::path::Path::new(&format!("/proc/{pid}")).is_dir()
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        true
    }
}

/// A directory of the person's own things that set how the program looks and starts, e.g. `schemes` or `templates` -
/// in the same one folder as the rest.
pub fn config(sub: &str) -> Option<std::path::PathBuf> {
    data_root().map(|d| d.join(sub))
}

/// A directory under the person's data, e.g. `crashes` or `library/parts`.
pub fn data(sub: &str) -> Option<std::path::PathBuf> {
    data_root().map(|d| d.join(sub))
}

#[cfg(test)]
mod tests {
    /// THE FOLDER IS NAMED AFTER THE PROGRAM, and this is asked of the finished path, not of the strings.
    ///
    /// The check that stood here compared the three parts against the id and was green while the program
    /// was writing into `~/.local/share/cad`: it asked whether the folder was DERIVED from the id, which
    /// was true, instead of what the folder came out as. So this one builds the path and reads it.
    #[test]
    fn the_folder_is_named_after_the_program() {
        let Some(d) = super::dirs() else {
            return; // a system without a notion of per-user directories has nothing to check
        };
        for dir in [d.config_dir(), d.data_dir()] {
            assert!(dir.to_string_lossy().to_lowercase().contains(super::FOLDER), "the program's own directory does not carry the program's name: {}", dir.display());
            assert!(!dir.components().any(|c| c.as_os_str() == "cad"), "the folder is named after the last part of the id again, and any other CAD may claim it: {}", dir.display());
        }
    }

    /// A COPY OF THE SETTINGS LEAVES THE PERSON'S FILE BYTE FOR BYTE.
    #[test]
    fn copying_the_settings_aside_does_not_change_the_persons_file() {
        let dir = std::env::temp_dir().join(format!("qymcad-settings-aside-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("the probe directory");
        let from = dir.join("app.ron");
        let to = dir.join(format!("side-app-{}.ron", std::process::id()));
        let body = b"(settings:\"dark\",last_project:\"/keep/this.qcad\")";
        std::fs::write(&from, body).expect("the probe settings");
        assert!(super::copy_settings_aside(&from), "the copy was not written");
        assert_eq!(std::fs::read(&from).expect("the person's file"), body, "the person's settings were rewritten");
        assert_eq!(std::fs::read(&to).expect("the copy"), body, "the copy is not the person's settings");
        assert!(!dir.join("side-app.ron").exists(), "every copy writes the same file");
        let missing = dir.join("no-such.ron");
        assert!(!super::copy_settings_aside(&missing), "a missing file still produced a copy");
        assert_eq!(std::fs::read(&to).expect("the earlier copy"), body, "a failed copy replaced the earlier one");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A FINISHED COPY'S FILE GOES. The person's file, and the file of a copy that is still open, stay.
    #[test]
    fn a_finished_copy_does_not_leave_its_settings() {
        let dir = std::env::temp_dir().join(format!("qymcad-settings-finished-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("the probe directory");
        let person = dir.join("app.ron");
        let mine = dir.join(format!("side-app-{}.ron", std::process::id()));
        let gone = dir.join("side-app-4294967294.ron");
        let shared = dir.join("side-app.ron");
        std::fs::write(&person, b"person").expect("the person's file");
        std::fs::write(&mine, b"mine").expect("this copy");
        std::fs::write(&gone, b"gone").expect("a finished copy");
        std::fs::write(&shared, b"shared").expect("the old shared name");
        super::drop_finished_copies(&dir);
        assert!(person.is_file(), "the person's settings were removed");
        assert!(mine.is_file(), "the open copy's settings were removed");
        assert!(shared.is_file(), "a name that is not one copy was removed");
        assert!(!gone.exists(), "a finished copy left its settings");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
