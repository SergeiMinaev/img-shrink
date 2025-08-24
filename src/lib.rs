use std::path::PathBuf;
use tempfile::NamedTempFile;

pub mod util;
pub mod jxl;
pub mod jpg;
pub mod webp;
pub mod png;
pub mod diff;

const DSSIM_THRESHOLD: f32 = 0.004;


pub fn to_png(data: &Vec<u8>, input_format: &str) -> PathBuf {
	match input_format.to_lowercase().as_str() {
		"jxl" =>  {
			jxl::bytes_to_png(data)
		},
		"jpg" =>  {
			jpg::bytes_to_png(data)
		},
		"jpeg" =>  {
			jpg::bytes_to_png(data)
		},
		"png" =>  {
			png::bytes_to_png(data)
		},
		"webp" =>  {
			webp::bytes_to_png(data)
		},
		_ => panic!("Unsupported format: {input_format}"),
	}
}

pub fn encode_from_png(
		png_path: &PathBuf, output_format: &str, size: &str, crop: bool
) -> NamedTempFile {
	let png_path = png::resize(png_path, size, crop);
	let quality = 1;
	_encode_from_png(&png_path, output_format, quality)
}

pub fn encode_from_png_adaptive(
		png_path: &PathBuf, output_format: &str, size: &str, crop: bool
) -> NamedTempFile {
	let base_png = png::resize(png_path, size, crop);
	let mut last_candidate: Option<NamedTempFile> = None;

	for quality_idx in 0..7 {
		let cand = _encode_from_png(&base_png, output_format, quality_idx);

		// decode candidate back to PNG for DSSIM comparison
		let cand_png = match output_format.to_lowercase().as_str() {
			"jxl" => jxl::file_to_png(&cand.path().to_path_buf()),
			"jpg" | "jpeg" => jpg::file_to_png(&cand.path().to_path_buf()),
			"webp" => webp::file_to_png(&cand.path().to_path_buf()),
			_ => unreachable!(),
		};

		let dist = diff::distance(&base_png, &cand_png);

		if dist <= DSSIM_THRESHOLD {
			return cand;
		}

		last_candidate = Some(cand);
	}
	last_candidate.unwrap()
}

/// Create a resized & encoded version starting from raw bytes of any input
/// format. Internally converts the source to a temporary PNG and delegates
/// to `make_version_from_png`.
pub fn encode(
	data: &Vec<u8>,
	input_format: &str,
	output_format: &str,
	size: &str,
	crop: bool,
) -> NamedTempFile {
	let png = to_png(data, input_format);
	let result = encode_from_png(&png, output_format, size, crop);
	// best-effort cleanup of the intermediate PNG; ignore errors
	let _ = std::fs::remove_file(png);
	result
}

/// Same as `make_version` but automatically selects the lowest quality that
/// meets the DSSIM threshold, using `make_version_auto_from_png`.
pub fn encode_adaptive(
	data: &Vec<u8>,
	input_format: &str,
	output_format: &str,
	size: &str,
	crop: bool,
) -> NamedTempFile {
	let png = to_png(data, input_format);
	let result = encode_from_png_adaptive(&png, output_format, size, crop);
	let _ = std::fs::remove_file(png);
	result
}

//// Convenience wrapper: keep original size (no resize) and no crop.
/// Internally calls `encode` with default `size = ">"` and `crop = false`.
pub fn encode_simple(
	data: &Vec<u8>,
	input_format: &str,
	output_format: &str,
) -> NamedTempFile {
	encode(data, input_format, output_format, ">", false)
}

//// Same as `encode_simple` but uses adaptive quality selection.
pub fn encode_adaptive_simple(
	data: &Vec<u8>,
	input_format: &str,
	output_format: &str,
) -> NamedTempFile {
	encode_adaptive(data, input_format, output_format, ">", false)
}

fn _encode_from_png(png_path: &PathBuf, output_format: &str, quality: usize) -> NamedTempFile {
	match output_format.to_lowercase().as_str() {
		"jxl" =>  {
			jxl::encode(png_path, quality)
		},
		"jpg" =>  {
			jpg::encode(png_path, quality)
		},
		"jpeg" =>  {
			jpg::encode(png_path, quality)
		},
		"webp" =>  {
			webp::encode(png_path, quality)
		},
		_ => panic!("Unsupported format: {output_format}"),
	}
}
