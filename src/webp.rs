use std::process::Command;
use std::path::PathBuf;
use crate::util;
use tempfile::NamedTempFile;


pub fn encode(png_path: &PathBuf, quality_idx: usize) -> NamedTempFile {
      let quality_list = vec!["65", "70", "75", "80", "85", "90", "95"];
      let quality = quality_list[quality_idx];
      let tmp = util::named_tempfile("webp");
      let out_path = tmp.path().to_path_buf();
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
      tmp
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
