use std::fs::File;
use std::io::Write;
use std::process::{ Command };
use std::str;
use std::path::PathBuf;


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
