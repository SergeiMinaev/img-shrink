use std::path::PathBuf;

pub mod util;
pub mod jxl;
pub mod jpg;
pub mod webp;
pub mod png;


pub fn make_png(data: &Vec<u8>, input_format: &str) -> PathBuf {
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

pub fn make_version(png_path: &PathBuf, output_format: &str, size: &str, crop: bool) -> PathBuf {
	let png_path = png::resize(png_path, size, crop);
	let quality = 1;
	let result = _make_version(&png_path, output_format, quality);
	result
}

fn _make_version(png_path: &PathBuf, output_format: &str, quality: usize) -> PathBuf {
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
