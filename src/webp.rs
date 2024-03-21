use std::process::Command;
use std::path::PathBuf;
use crate::util;


pub fn encode(png_path: &PathBuf, quality_idx: usize) -> PathBuf {
	  let quality_list = vec!["65", "70", "75", "80", "85", "90", "95"];
	  let quality = quality_list[quality_idx];
	  let out_path = util::mktemp("webp");
	  let output = Command::new("/usr/bin/cwebp")
					   .arg("-q")
					   .arg(quality)
					   .arg(png_path.as_path())
					   .arg("-o")
					   .arg(out_path.as_path())
					   .output()
					   .expect("failed to execute process");
	  if output.status.success() == false {
		  panic!("webp::encode() failed: {output:?}");
	  }
	  return out_path
}

pub fn bytes_to_png(data: &Vec<u8>) -> PathBuf {
	let input_path = util::bytes_to_tempfile(data, "webp");
	file_to_png(&input_path)
}

pub fn file_to_png(input_path: &PathBuf) -> PathBuf {
	let out_path = util::mktemp("png");
	let output = Command::new("/usr/bin/dwebp")
					 .arg(input_path)
					 .arg("-o")
					 .arg(out_path.as_path())
					 .output()
					 .expect("failed to execute process");
	if output.status.success() == false {
		println!("webp decode failed: {output:?}");
	}
	out_path
}
