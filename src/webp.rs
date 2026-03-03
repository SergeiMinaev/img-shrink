use std::process::Command;
use std::path::PathBuf;
use crate::util;
use tempfile::NamedTempFile;


pub const QUALITY_LIST: [u8; 7] = [75, 80, 85, 90, 95, 99, 100];


pub fn encode(png_path: &PathBuf, quality_idx: usize, sharp_yuv: bool) -> NamedTempFile {
      let quality = QUALITY_LIST[quality_idx];
      let tmp = util::named_tempfile("webp");
      let out_path = tmp.path().to_path_buf();
      let mut cmd = Command::new("/usr/bin/cwebp");
      cmd.arg("-q")
         .arg(quality.to_string());
      if sharp_yuv {
          cmd.arg("-sharp_yuv");
      }
      let output = cmd.arg(png_path.as_path())
                      .arg("-o")
                      .arg(out_path.as_path())
                      .output()
                      .expect("failed to execute process");
      if output.status.success() == false {
          panic!("webp::encode() failed: {output:?}");
      }
      tmp
}

pub fn encode_quality(png_path: &PathBuf, quality: u8, sharp_yuv: bool) -> NamedTempFile {
      let tmp = util::named_tempfile("webp");
      let out_path = tmp.path().to_path_buf();
      let mut cmd = Command::new("/usr/bin/cwebp");
      cmd.arg("-q")
         .arg(quality.to_string());
      if sharp_yuv {
          cmd.arg("-sharp_yuv");
      }
      let output = cmd.arg(png_path.as_path())
                      .arg("-o")
                      .arg(out_path.as_path())
                      .output()
                      .expect("failed to execute process");
      if output.status.success() == false {
          panic!("webp::encode_quality() failed: {output:?}");
      }
      tmp
}

/// Decode WebP bytes to a temp PNG file and return its path.
///
/// The caller is responsible for deleting the temp file when done.
pub fn bytes_to_png(data: &Vec<u8>) -> PathBuf {
	let tmp = util::bytes_to_named_tempfile(data, "webp");
    let input_path = tmp.path().to_path_buf();
	file_to_png(&input_path)
}

/// Decode a WebP file to a temp PNG file and return its path.
///
/// The caller is responsible for deleting the temp file when done.
pub fn file_to_png(input_path: &PathBuf) -> PathBuf {
	#[cfg(feature = "magick")]
	{
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
		return out_path;
	}
	#[cfg(feature = "vips")]
	{
		util::file_to_png(input_path)
	}
}
