use std::path::Path;
use std::process::Command;
use std::env;
use std::io;

/// Return DSSIM distance between two images (0.0 ⇒ identical).
pub fn distance(a: &Path, b: &Path) -> f32 {
    let mut cmd = Command::new("dssim");
    cmd.arg(a).arg(b);

    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Failed to spawn `dssim` command: {e}");
            eprintln!("Tried to run: `dssim {} {}`", a.display(), b.display());
            if let Ok(path) = env::var("PATH") {
                eprintln!("PATH={}", path);
            }
            if e.kind() == io::ErrorKind::NotFound {
                eprintln!("`dssim` executable not found. Is it installed and on PATH? Try `which dssim`.");
            }
            panic!("failed to execute `dssim`: {e:?}");
        }
    };

    if !output.status.success() {
        eprintln!("dssim exited with status: {:?}", output.status);
        eprintln!("dssim stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("dssim stderr:\n{}", String::from_utf8_lossy(&output.stderr));
        panic!("dssim failed: {output:?}");
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    stdout
        .split_whitespace()
        .next()
        .expect("unexpected dssim output")
        .parse::<f32>()
        .expect("failed to parse dssim score")
}
