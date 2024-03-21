use std::process::Command;
use std::path::PathBuf;
use crate::util;


pub fn encode(png_path: &PathBuf, quality_idx: usize) -> PathBuf {
    let quality_list = vec!["65", "70", "75", "80", "85", "90", "95"];
    let quality = quality_list[quality_idx];
    let out_path = util::mktemp("jpg");
    let output = Command::new("/usr/bin/cjpg")
                     .arg("-q")
                     .arg(quality)
                     .arg(png_path.as_path())
                     .arg(out_path.as_path())
                     .output()
                     .expect("failed to execute process");
    if output.status.success() == false {
        panic!("jpg::encode() failed: {output:?}");
    }
    return out_path
}

pub fn bytes_to_png(data: &Vec<u8>) -> PathBuf {
	let input_path = util::bytes_to_tempfile(data, "jpg");
	file_to_png(&input_path)
}

pub fn file_to_png(input_path: &PathBuf) -> PathBuf {
    let out_path = util::mktemp("png");
    let output = Command::new("/usr/bin/convert")
                     .arg(input_path.as_path())
                     .arg(out_path.as_path())
                     .output()
                     .expect("failed to execute process");
    if output.status.success() == false {
        println!("jxl::decode() failed: {output:?}");
    }
	out_path
}
