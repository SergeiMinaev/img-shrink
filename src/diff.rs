use std::path::Path;
use std::process::Command;

/// Return DSSIM distance between two images (0.0 ⇒ identical).
pub fn distance(a: &Path, b: &Path) -> f32 {
    let out = Command::new("dssim")
        .arg(a)
        .arg(b)
        .output()
        .expect("failed to execute `dssim`");

    if !out.status.success() {
        panic!("dssim failed: {out:?}");
    }

    let stdout = String::from_utf8(out.stdout).unwrap();
    stdout
        .split_whitespace()
        .next()
        .expect("unexpected dssim output")
        .parse::<f32>()
        .expect("failed to parse dssim score")
}
