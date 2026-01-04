use std::process::Command;
use std::path::PathBuf;
use crate::util;
use tempfile::NamedTempFile;

pub const QUALITY_LIST: [u8; 7] = [65, 70, 75, 80, 85, 90, 95];


pub fn encode(png_path: &PathBuf, quality_idx: usize) -> NamedTempFile {
    let quality = QUALITY_LIST[quality_idx];
    let tmp = util::named_tempfile("jpg");
    let out_path = tmp.path().to_path_buf();
    let output = Command::new("/usr/bin/cjpg")
                     .arg("-q")
                     .arg(quality.to_string())
                     .arg(png_path.as_path())
                     .arg(out_path.as_path())
                     .output()
                     .expect("failed to execute process");
    if output.status.success() == false {
        panic!("jpg::encode() failed: {output:?}");
    }
    tmp
}

pub fn encode_quality(png_path: &PathBuf, quality: u8) -> NamedTempFile {
    let tmp = util::named_tempfile("jpg");
    let out_path = tmp.path().to_path_buf();
    let output = Command::new("/usr/bin/cjpg")
                     .arg("-q")
                     .arg(quality.to_string())
                     .arg(png_path.as_path())
                     .arg(out_path.as_path())
                     .output()
                     .expect("failed to execute process");
    if output.status.success() == false {
        panic!("jpg::encode_quality() failed: {output:?}");
    }
    tmp
}

pub fn bytes_to_png(data: &Vec<u8>) -> PathBuf {
	let input_path = util::bytes_to_tempfile(data, "jpg");
	let out = file_to_png(&input_path);
	util::cleanup_tempfile(&input_path);
	out
}

pub fn file_to_png(input_path: &PathBuf) -> PathBuf {
    util::file_to_png(input_path)
}
