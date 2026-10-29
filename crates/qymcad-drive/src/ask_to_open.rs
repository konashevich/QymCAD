// Start another copy of the graphical program on one saved file.
//
// The copy is the executable named by `QYMCAD_GUI`. It is given the path as its first argument and
// `QYMCAD_SIDE=1`, and it does not inherit `QYMCAD_COMMAND_QUEUE`: that socket belongs to the copy
// that is already open, and a second listener removes the socket file and takes it. This process
// does not wait for the copy to close. A thread waits on the child so it is not left as a zombie.

fn ask_to_open(path: &str) -> Result<(), String> {
    if path.is_empty() || path.contains('\n') || path.contains('\r') || !std::path::Path::new(path).is_file() {
        return Err("The file is missing.".to_string());
    }
    let Some(program) = std::env::var_os("QYMCAD_GUI").filter(|program| std::path::Path::new(program).is_file()) else {
        return Err("The graphical application is missing.".to_string());
    };
    let mut child = std::process::Command::new(program)
        .arg(path)
        .env("QYMCAD_SIDE", "1")
        .env_remove("QYMCAD_COMMAND_QUEUE")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|_| "The graphical application is missing.".to_string())?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}
