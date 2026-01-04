use std::fs::File;
use std::io::Write;
use std::process::{ Command };
use std::str;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;


pub fn mktemp(ext: &str) -> PathBuf {
    let path = format!("/tmp/img-shrink_XXXXXXXXXXXX.{ext}");
    let out = Command::new("/usr/bin/mktemp")
        .arg(path)
        .output().expect("Failed to execute `mktemp`");
    if out.status.success() == false {
        panic!("`mktemp` failed: {:?}", out);
    }
    return PathBuf::from(str::from_utf8(&out.stdout).unwrap().trim().to_string())
}

fn is_img_shrink_temp(path: &Path) -> bool {
    let parent = path.parent().and_then(|p| p.to_str());
    let file = path.file_name().and_then(|n| n.to_str());
    parent == Some("/tmp") && file.map_or(false, |name| name.starts_with("img-shrink_"))
}

pub fn cleanup_tempfile(path: &Path) {
    if is_img_shrink_temp(path) {
        let _ = std::fs::remove_file(path);
    }
}

pub fn bytes_to_tempfile(data: &Vec<u8>, input_format: &str) -> PathBuf {
	let path = mktemp(input_format);
	let mut f = File::create(path.clone()).unwrap();
	let _ = f.write_all(&data).unwrap();
	path
}

pub fn named_tempfile(ext: &str) -> NamedTempFile {
    tempfile::Builder::new()
        .prefix("img-shrink_")
        .suffix(&format!(".{ext}"))
        .tempfile_in("/tmp")
        .expect("Failed to create NamedTempFile")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup_removes_img_shrink_tempfiles() {
        let tmp = named_tempfile("png");
        let (file, path) = tmp.keep().expect("keep tempfile");
        drop(file);
        assert!(path.exists());
        cleanup_tempfile(&path);
        assert!(!path.exists());
    }

    #[test]
    fn cleanup_ignores_other_paths() {
        let tmp = tempfile::Builder::new()
            .prefix("not-img-shrink_")
            .suffix(".png")
            .tempfile_in("/tmp")
            .expect("create tempfile");
        let (file, path) = tmp.keep().expect("keep tempfile");
        drop(file);
        assert!(path.exists());
        cleanup_tempfile(&path);
        assert!(path.exists());
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(feature = "vips")]
pub fn file_to_png(input_path: &PathBuf) -> PathBuf {
    let out_path = mktemp("png");
    let t0 = std::time::Instant::now();
    if std::env::var("IMG_SHRINK_TIMINGS").ok().as_deref() == Some("1") {
        eprintln!(
            "vips cmd: /usr/bin/vips autorot {} {}",
            input_path.display(),
            out_path.display()
        );
    }
    if std::env::var("IMG_SHRINK_DEBUG").ok().as_deref() == Some("1") {
        let exif_out = Command::new("/usr/bin/vipsheader")
            .arg("-a")
            .arg(input_path.as_path())
            .output()
            .expect("failed to execute vipsheader");
        if exif_out.status.success() == false {
            panic!("vipsheader failed: {exif_out:?}");
        }
        eprintln!(
            "vipsheader -a {}:\n{}",
            input_path.display(),
            String::from_utf8_lossy(&exif_out.stdout)
        );
        if exif_out.stderr.is_empty() == false {
            eprintln!("vipsheader stderr: {}", String::from_utf8_lossy(&exif_out.stderr));
        }
    }
    let output = Command::new("/usr/bin/vips")
        .arg("autorot")
        .arg(input_path.as_path())
        .arg(&out_path)
        .output()
        .expect("failed to execute process");
    if output.status.success() == false {
        panic!("vips::file_to_png() failed: {output:?}");
    }
    if std::env::var("IMG_SHRINK_TIMINGS").ok().as_deref() == Some("1") {
        eprintln!("vips file_to_png: {} ms", t0.elapsed().as_millis());
    }
    PathBuf::from(out_path)
}

#[cfg(feature = "magick")]
pub fn file_to_png(input_path: &PathBuf) -> PathBuf {
    let out_path = mktemp("png");
    let output = Command::new("/usr/bin/convert")
        .arg(input_path.as_path())
        .arg("-auto-orient")
        .arg(&out_path)
        .output()
        .expect("failed to execute process");
    if output.status.success() == false {
        panic!("magick::file_to_png() failed: {output:?}");
    }
    PathBuf::from(out_path)
}
