use std::path::Path;
use std::process::Command;

/// Return DSSIM distance between two images (0.0 ⇒ identical).
pub fn distance(a: &Path, b: &Path) -> f32 {
    let out = Command::new("/usr/bin/dssim")
        .arg("--single")   // print only the score
        .arg(a)
        .arg(b)
        .output()
        .expect("failed to execute `dssim`");

    if !out.status.success() {
        panic!("dssim failed: {out:?}");
    }

    String::from_utf8(out.stdout)
        .unwrap()
        .trim()
        .parse::<f32>()
        .unwrap()
}
