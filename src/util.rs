use std::fs::File;
use std::io::Write;
use std::process::{ Command };
use std::str;
use std::path::PathBuf;
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

#[cfg(feature = "vips")]
pub fn file_to_png(input_path: &PathBuf) -> PathBuf {
    let out_path = mktemp("png");
    let t0 = std::time::Instant::now();
    let output = Command::new("/usr/bin/vips")
        .arg("copy")
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
