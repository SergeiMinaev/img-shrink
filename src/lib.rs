use std::path::PathBuf;
use tempfile::NamedTempFile;

pub mod util;
pub mod jxl;
pub mod jpg;
pub mod webp;
pub mod png;
pub mod diff;

//// dSSIM presets expressed in terms of *output quality*
//// higher quality ⇒ smaller distance (stricter), lower quality ⇒ larger distance
pub const LOW_QUALITY_THRESHOLD:    f32 = 0.008; // strongest compression
pub const MEDIUM_QUALITY_THRESHOLD: f32 = 0.004; // balanced default (was 0.04)
pub const HIGH_QUALITY_THRESHOLD:   f32 = 0.002; // near-lossless
// Keep the old name so existing code keeps compiling.
pub const DEFAULT_DSSIM_THRESHOLD: f32 = MEDIUM_QUALITY_THRESHOLD;
pub const MAX_QUALITY_IDX: usize = 6;


#[derive(Clone, Copy)]
pub struct EncodeOptions<'a> {
    pub size:     &'a str,
    pub crop:     bool,
    pub threshold: Option<f32>,
}

impl<'a> Default for EncodeOptions<'a> {
    fn default() -> Self {
        Self {
            size: ">",
            crop: false,
            threshold: Some(DEFAULT_DSSIM_THRESHOLD),
        }
    }
}

pub struct EncodeOptionsBuilder<'a> {
    opts: EncodeOptions<'a>,
}

impl<'a> EncodeOptionsBuilder<'a> {
    pub fn new() -> Self {
        Self { opts: EncodeOptions::default() }
    }

    pub fn size(mut self, size: &'a str) -> Self {
        self.opts.size = size;
        self
    }

    pub fn crop(mut self, crop: bool) -> Self {
        self.opts.crop = crop;
        self
    }

    // ---------- quality preset helpers ----------
    pub fn low(mut self) -> Self {
        self.opts.threshold = Some(LOW_QUALITY_THRESHOLD);
        self
    }

    pub fn medium(mut self) -> Self {
        self.opts.threshold = Some(MEDIUM_QUALITY_THRESHOLD);
        self
    }

    pub fn high(mut self) -> Self {
        self.opts.threshold = Some(HIGH_QUALITY_THRESHOLD);
        self
    }

    pub fn max(mut self) -> Self {
        self.opts.threshold = None;
        self
    }

    pub fn threshold(mut self, threshold: f32) -> Self {
        self.opts.threshold = Some(threshold);
        self
    }

    pub fn build(self) -> EncodeOptions<'a> {
        self.opts
    }
}

/// Main entry-point.
/// 
/// Pass an `EncodeOptions` value to control resize, cropping, adaptive quality
/// selection and the optional dSSIM threshold.  For a quick one-liner that keeps
/// the defaults, use `encode_default`.
pub fn encode<'a>(
    data: &Vec<u8>,
    input_format: &str,
    output_format: &str,
    opts: EncodeOptions<'a>,
) -> NamedTempFile {
    let png = to_png(data, input_format);

    // `None`  ⇒ fixed quality (single encode)
    // `Some` ⇒ adaptive sweep until dSSIM ≤ threshold
    let res = encode_from_png_internal(
        &png,
        output_format,
        opts.size,
        opts.crop,
        opts.threshold,
    );
    let _ = std::fs::remove_file(png);
    res
}

// Adaptive-quality convenience wrapper.
// 
// Builds default `EncodeOptions`, switches on adaptive mode and keeps the
// library-wide `DEFAULT_DSSIM_THRESHOLD`.  No resize and no crop are applied.
// 
// Typical usage:
// ```rust
// let file = img_shrink::encode_adaptive(&bytes, "jpg", "jxl");
// ```
pub fn encode_adaptive(
    data: &Vec<u8>,
    input_format: &str,
    output_format: &str,
) -> NamedTempFile {
    encode(data, input_format, output_format, EncodeOptions::default())
}

/// Internal helper: adaptive encoding with caller-supplied threshold.
fn encode_from_png_internal(
    png_path: &PathBuf,
    output_format: &str,
    size: &str,
    crop: bool,
    threshold: Option<f32>,
) -> NamedTempFile {
    let base_png = png::resize(png_path, size, crop);

    // Fixed-quality
    if threshold.is_none() {
        return _encode_from_png(&base_png, output_format, MAX_QUALITY_IDX);
    }

    // Adaptive sweep
    let mut last_candidate: Option<NamedTempFile> = None;
    let thr = threshold.unwrap();

    for quality_idx in 0..MAX_QUALITY_IDX {
        let cand = _encode_from_png(&base_png, output_format, quality_idx);

        // decode candidate back to PNG for dSSIM
        let cand_png = match output_format.to_lowercase().as_str() {
            "jxl"           => jxl::file_to_png(&cand.path().to_path_buf()),
            "jpg" | "jpeg"  => jpg::file_to_png(&cand.path().to_path_buf()),
            "webp"          => webp::file_to_png(&cand.path().to_path_buf()),
            _               => unreachable!(),
        };

        let dist = diff::distance(&base_png, &cand_png);
        println!("dist: {dist}");

        if dist <= thr {
            return cand;
        }
        last_candidate = Some(cand);
    }
    last_candidate.unwrap()
}


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
