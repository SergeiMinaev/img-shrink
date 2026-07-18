use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

pub mod util;
pub mod jxl;
pub mod jpg;
pub mod webp;
pub mod heic;
pub mod png;
pub mod diff;

#[cfg(all(feature = "vips", feature = "magick"))]
compile_error!("Choose exactly one backend: enable either feature `vips` or `magick`.");
#[cfg(not(any(feature = "vips", feature = "magick")))]
compile_error!("No backend selected. Enable feature `vips` (default) or `magick`.");

//// dSSIM presets expressed in terms of *output quality*
//// higher quality ⇒ smaller distance (stricter), lower quality ⇒ larger distance
pub const LOW_QUALITY_THRESHOLD:    f32 = 0.008; // strongest compression
pub const MEDIUM_QUALITY_THRESHOLD: f32 = 0.004; // balanced default (was 0.04)
pub const HIGH_QUALITY_THRESHOLD:   f32 = 0.002; // near-lossless
// Keep the old name so existing code keeps compiling.
pub const DEFAULT_DSSIM_THRESHOLD: f32 = MEDIUM_QUALITY_THRESHOLD;
pub const MAX_QUALITY_IDX: usize = 6;


/// Default watermark width as a fraction of the base image width.
pub const DEFAULT_WATERMARK_WIDTH_FRAC: f64 = 0.30;
/// Default watermark margin from the edges as a fraction of the base image width.
pub const DEFAULT_WATERMARK_MARGIN_FRAC: f64 = 0.02;

/// Corner of the base image where the watermark is placed.
#[derive(Clone, Copy, Default)]
pub enum Corner {
    #[default]
    SouthEast,
    SouthWest,
    NorthEast,
    NorthWest,
}

/// Watermark overlay: a PNG with alpha, scaled relative to the base image
/// and composited into the chosen corner after resize.
///
/// ```rust,ignore
/// let wm = img_shrink::Watermark::new(Path::new("logo.png"))
///     .width_frac(0.25)
///     .corner(img_shrink::Corner::NorthWest);
/// let opts = img_shrink::EncodeOptionsBuilder::new().size("800x800").watermark(wm).build();
/// ```
#[derive(Clone, Copy)]
pub struct Watermark<'a> {
    pub path: &'a Path,
    /// Watermark width as a fraction of the base image width.
    pub width_frac: f64,
    /// Margin from the edges as a fraction of the base image width.
    pub margin_frac: f64,
    pub corner: Corner,
}

impl<'a> Watermark<'a> {
    pub fn new(path: &'a Path) -> Self {
        Self {
            path,
            width_frac: DEFAULT_WATERMARK_WIDTH_FRAC,
            margin_frac: DEFAULT_WATERMARK_MARGIN_FRAC,
            corner: Corner::default(),
        }
    }

    pub fn width_frac(mut self, width_frac: f64) -> Self {
        self.width_frac = width_frac;
        self
    }

    pub fn margin_frac(mut self, margin_frac: f64) -> Self {
        self.margin_frac = margin_frac;
        self
    }

    pub fn corner(mut self, corner: Corner) -> Self {
        self.corner = corner;
        self
    }
}

#[derive(Clone, Copy)]
pub struct EncodeOptions<'a> {
    pub size:     &'a str,
    pub crop:     bool,
    pub threshold: Option<f32>,
    pub quality_idx: Option<usize>,
    pub quality_value: Option<u8>,
    pub sharp_yuv: bool,
    pub watermark: Option<Watermark<'a>>,
}

impl<'a> Default for EncodeOptions<'a> {
    fn default() -> Self {
        Self {
            size: ">",
            crop: false,
            threshold: Some(DEFAULT_DSSIM_THRESHOLD),
            quality_idx: None,
            quality_value: None,
            sharp_yuv: true,
            watermark: None,
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

    // Fixed-quality mode: disables adaptive thresholding.
    pub fn quality_idx(mut self, quality_idx: usize) -> Self {
        self.opts.threshold = None;
        self.opts.quality_idx = Some(quality_idx);
        self.opts.quality_value = None;
        self
    }

    pub fn quality_value(mut self, quality: u8) -> Self {
        self.opts.threshold = None;
        self.opts.quality_idx = None;
        self.opts.quality_value = Some(quality);
        self
    }

    pub fn threshold(mut self, threshold: f32) -> Self {
        self.opts.threshold = Some(threshold);
        self
    }

    pub fn sharp_yuv(mut self, sharp_yuv: bool) -> Self {
        self.opts.sharp_yuv = sharp_yuv;
        self
    }

    pub fn watermark(mut self, watermark: Watermark<'a>) -> Self {
        self.opts.watermark = Some(watermark);
        self
    }

    pub fn build(self) -> EncodeOptions<'a> {
        self.opts
    }
}

/// Main entry-point.
///
/// Pass an `EncodeOptions` value to control resize, cropping, adaptive quality
/// selection and the optional dSSIM threshold. For a quick one-liner that keeps
/// the defaults, use `encode_default`.
///
/// The returned `NamedTempFile` is removed when dropped. Use `keep()` if you
/// want the output to persist on disk.
pub fn encode<'a>(
    data: &Vec<u8>,
    input_format: &str,
    output_format: &str,
    opts: EncodeOptions<'a>,
) -> NamedTempFile {
    let t0 = std::time::Instant::now();
    let t_png = std::time::Instant::now();
    let png = to_png(data, input_format);
    if std::env::var("IMG_SHRINK_TIMINGS").ok().as_deref() == Some("1") {
        eprintln!("img-shrink to_png: {} ms", t_png.elapsed().as_millis());
    }
    // `None`  ⇒ fixed quality (single encode)
    // `Some` ⇒ adaptive sweep until dSSIM ≤ threshold
    let res = encode_from_png_internal(
        &png,
        output_format,
        opts.size,
        opts.crop,
        opts.threshold,
        opts.quality_idx,
        opts.quality_value,
        opts.sharp_yuv,
        opts.watermark,
    );
    let _ = std::fs::remove_file(png);
    if std::env::var("IMG_SHRINK_TIMINGS").ok().as_deref() == Some("1") {
        eprintln!("img-shrink encode total: {} ms", t0.elapsed().as_millis());
    }
    res
}

/// Encode an existing PNG file to the desired output format.
///
/// The returned `NamedTempFile` is removed when dropped. Use `keep()` if you
/// want the output to persist on disk.
pub fn encode_from_png<'a>(
    png_path: &PathBuf,
    output_format: &str,
    opts: EncodeOptions<'a>,
) -> NamedTempFile {
    encode_from_png_internal(
        png_path,
        output_format,
        opts.size,
        opts.crop,
        opts.threshold,
        opts.quality_idx,
        opts.quality_value,
        opts.sharp_yuv,
        opts.watermark,
    )
}

// Adaptive-quality convenience wrapper.
//
// Builds default `EncodeOptions`, switches on adaptive mode and keeps the
// library-wide `DEFAULT_DSSIM_THRESHOLD`. No resize and no crop are applied.
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
    quality_idx: Option<usize>,
    quality_value: Option<u8>,
    sharp_yuv: bool,
    watermark: Option<Watermark>,
) -> NamedTempFile {
    let t_resize = std::time::Instant::now();
    let base_png = png::resize(png_path, size, crop);
    let mut base_is_input = base_png == *png_path;
    if std::env::var("IMG_SHRINK_TIMINGS").ok().as_deref() == Some("1") {
        eprintln!("img-shrink resize: {} ms", t_resize.elapsed().as_millis());
    }

    // The watermark is composited onto the already-resized image so its
    // size stays proportional to the final output.
    let base_png = match watermark {
        Some(wm) => {
            let stamped = png::watermark(&base_png, &wm);
            if !base_is_input {
                util::cleanup_tempfile(&base_png);
            }
            base_is_input = false;
            stamped
        }
        None => base_png,
    };

    // Fixed-quality
    if threshold.is_none() {
        if let Some(quality) = quality_value {
            let t_enc = std::time::Instant::now();
            let out = _encode_from_png_quality(&base_png, output_format, quality, sharp_yuv);
            if std::env::var("IMG_SHRINK_TIMINGS").ok().as_deref() == Some("1") {
                eprintln!("img-shrink encode fixed quality: {} ms", t_enc.elapsed().as_millis());
            }
            if !base_is_input {
                util::cleanup_tempfile(&base_png);
            }
            return out;
        }
        if let Some(idx) = quality_idx {
            if idx > MAX_QUALITY_IDX {
                panic!("quality_idx out of range: {idx} > {MAX_QUALITY_IDX}");
            }
        }
        let idx = quality_idx.unwrap_or(MAX_QUALITY_IDX);
        let t_enc = std::time::Instant::now();
        let out = _encode_from_png(&base_png, output_format, idx, sharp_yuv);
        if std::env::var("IMG_SHRINK_TIMINGS").ok().as_deref() == Some("1") {
            eprintln!("img-shrink encode fixed idx: {} ms", t_enc.elapsed().as_millis());
        }
        if !base_is_input {
            util::cleanup_tempfile(&base_png);
        }
        return out;
    }

    // Adaptive sweep
    let mut last_candidate: Option<NamedTempFile> = None;
    let thr = threshold.unwrap();

    for quality_idx in 0..=MAX_QUALITY_IDX {
        let cand = _encode_from_png(&base_png, output_format, quality_idx, sharp_yuv);

        // decode candidate back to PNG for dSSIM
        let cand_png = match output_format.to_lowercase().as_str() {
            "jxl"           => jxl::file_to_png(&cand.path().to_path_buf()),
            "jpg" | "jpeg"  => jpg::file_to_png(&cand.path().to_path_buf()),
            "webp"          => webp::file_to_png(&cand.path().to_path_buf()),
            _               => unreachable!(),
        };

        let dist = diff::distance(&base_png, &cand_png);
        util::cleanup_tempfile(&cand_png);
        if dist <= thr {
            if !base_is_input {
                util::cleanup_tempfile(&base_png);
            }
            return cand;
        }
        last_candidate = Some(cand);
    }
    if !base_is_input {
        util::cleanup_tempfile(&base_png);
    }
    last_candidate.unwrap()
}


/// Convert input bytes to a PNG file in `/tmp`.
///
/// The returned path points to a temp file; callers must remove it when done.
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
		"heic" | "heif" => {
			heic::bytes_to_png(data)
		},
		_ => panic!("Unsupported format: {input_format}"),
	}
}


fn _encode_from_png(
    png_path: &PathBuf,
    output_format: &str,
    quality: usize,
    sharp_yuv: bool,
) -> NamedTempFile {
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
			webp::encode(png_path, quality, sharp_yuv)
		},
		"heic" | "heif" => {
			heic::encode(png_path, quality)
		},
		_ => panic!("Unsupported format: {output_format}"),
	}
}

fn _encode_from_png_quality(
    png_path: &PathBuf,
    output_format: &str,
    quality: u8,
    sharp_yuv: bool,
) -> NamedTempFile {
	match output_format.to_lowercase().as_str() {
		"jxl" =>  {
			jxl::encode_quality(png_path, quality)
		},
		"jpg" =>  {
			jpg::encode_quality(png_path, quality)
		},
		"jpeg" =>  {
			jpg::encode_quality(png_path, quality)
		},
		"webp" =>  {
			webp::encode_quality(png_path, quality, sharp_yuv)
		},
		"heic" | "heif" => {
			heic::encode_quality(png_path, quality)
		},
		_ => panic!("Unsupported format: {output_format}"),
	}
}
