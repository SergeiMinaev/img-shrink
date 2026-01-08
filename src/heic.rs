use std::process::Command;
use std::path::PathBuf;
use crate::util;
use tempfile::NamedTempFile;

pub const QUALITY_LIST: [u8; 7] = [65, 70, 75, 80, 85, 90, 95];


pub fn encode(png_path: &PathBuf, quality_idx: usize) -> NamedTempFile {
	let quality = QUALITY_LIST[quality_idx];
	let tmp = util::named_tempfile("heic");
	let out_path = tmp.path().to_path_buf();
	let output = Command::new("/usr/bin/heif-enc")
					 .arg("-q")
					 .arg(quality.to_string())
					 .arg(png_path.as_path())
					 .arg(out_path.as_path())
					 .output()
					 .expect("failed to execute process");
	if output.status.success() == false {
		panic!("heic::encode() failed: {output:?}");
	}
	tmp
}

pub fn encode_quality(png_path: &PathBuf, quality: u8) -> NamedTempFile {
	let tmp = util::named_tempfile("heic");
	let out_path = tmp.path().to_path_buf();
	let output = Command::new("/usr/bin/heif-enc")
					 .arg("-q")
					 .arg(quality.to_string())
					 .arg(png_path.as_path())
					 .arg(out_path.as_path())
					 .output()
					 .expect("failed to execute process");
	if output.status.success() == false {
		panic!("heic::encode_quality() failed: {output:?}");
	}
	tmp
}

/// Decode HEIC/HEIF bytes to a temp PNG file and return its path.
///
/// The caller is responsible for deleting the temp file when done.
pub fn bytes_to_png(data: &Vec<u8>) -> PathBuf {
	let tmp = util::bytes_to_named_tempfile(data, "heic");
    let input_path = tmp.path().to_path_buf();
	file_to_png(&input_path)
}

/// Decode a HEIC/HEIF file to a temp PNG file and return its path.
///
/// The caller is responsible for deleting the temp file when done.
pub fn file_to_png(input_path: &PathBuf) -> PathBuf {
	#[cfg(feature = "magick")]
	{
		let out_path = util::mktemp("png");
		// decode using libheif tool
		let output = Command::new("/usr/bin/heif-convert")
					 .arg(input_path.as_path())
					 .arg(out_path.as_path())
					 .output()
					 .expect("failed to execute process");
		if output.status.success() == false {
			println!("heic decode failed: {output:?}");
		}
		// Ensure EXIF orientation is applied (ImageMagick)
		let _ = Command::new("/usr/bin/convert")
					 .arg(out_path.as_path())
					 .arg("-auto-orient")
					 .arg(out_path.as_path())
					 .output();
		return out_path;
	}
	#[cfg(feature = "vips")]
	{
		util::file_to_png(input_path)
	}
}
