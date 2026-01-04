use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use img_shrink::EncodeOptionsBuilder;
use tempfile::NamedTempFile;

const FIXED_QUALITY_HIGH: u8 = 85;
const FIXED_QUALITY_MEDIUM: u8 = 75;

#[derive(Clone, Copy)]
enum FixedQualitySpec {
    Index(usize),
    Value(u8),
}

#[derive(Clone, Copy)]
struct DssimPresets {
    high: bool,
    medium: bool,
}

struct Args {
    input_dir: PathBuf,
    output_dir: PathBuf,
    output_format: String,
    size: Option<String>,
    crop: bool,
    recursive: bool,
    dssim_presets: DssimPresets,
}

struct WebpDssimArgs {
    input_dir: PathBuf,
    output_dir: PathBuf,
    quality: Option<u8>,
    preset: Option<DssimPreset>,
    size: Option<String>,
    crop: bool,
    recursive: bool,
}

#[derive(Clone, Copy)]
enum DssimPreset {
    High,
    Medium,
}

pub fn entrypoint() {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    let mut iter = raw_args.into_iter();
    match iter.next() {
        Some(cmd) if cmd == "webp-dssim" => {
            let args = match WebpDssimArgs::parse_from(&mut iter) {
                Ok(args) => args,
                Err(err) => {
                    eprintln!("{err}");
                    print_usage_webp_dssim();
                    std::process::exit(1);
                }
            };
            if let Err(err) = run_webp_dssim(&args) {
                eprintln!("error: {err}");
                std::process::exit(1);
            }
        }
        Some(first) => {
            let mut all_args = std::iter::once(first).chain(iter);
            let args = match Args::parse_from(&mut all_args) {
                Ok(args) => args,
                Err(err) => {
                    eprintln!("{err}");
                    print_usage();
                    std::process::exit(1);
                }
            };
            if let Err(err) = run(&args) {
                eprintln!("error: {err}");
                std::process::exit(1);
            }
        }
        None => {
            print_usage();
            std::process::exit(1);
        }
    }
}

fn run(args: &Args) -> Result<(), String> {
    let output_format = normalize_format(&args.output_format);
    if !is_supported_output_format(&output_format) {
        return Err(format!("unsupported output format: {output_format}"));
    }
    let quality_high =
        fixed_quality_spec(&output_format, FIXED_QUALITY_HIGH, "quality-85")?;
    let quality_medium =
        fixed_quality_spec(&output_format, FIXED_QUALITY_MEDIUM, "quality-75")?;
    let run_high = args.dssim_presets.high;
    let run_medium = args.dssim_presets.medium;

    let mut files = Vec::new();
    collect_files(&args.input_dir, args.recursive, &mut files)?;

    let quality_dir = args.output_dir.join("quality-85");
    let quality_medium_dir = args.output_dir.join("quality-75");
    fs::create_dir_all(&quality_dir).map_err(|e| format!("create dir: {e}"))?;
    fs::create_dir_all(&quality_medium_dir).map_err(|e| format!("create dir: {e}"))?;
    let dssim_dir = if run_high {
        let dir = args.output_dir.join("dssim-high");
        fs::create_dir_all(&dir).map_err(|e| format!("create dir: {e}"))?;
        Some(dir)
    } else {
        None
    };
    let dssim_medium_dir = if run_medium {
        let dir = args.output_dir.join("dssim-medium");
        fs::create_dir_all(&dir).map_err(|e| format!("create dir: {e}"))?;
        Some(dir)
    } else {
        None
    };

    let mut total_fixed = 0u64;
    let mut total_fixed_medium = 0u64;
    let mut total_high = 0u64;
    let mut total_medium = 0u64;
    let mut processed = 0u64;
    let mut skipped = 0u64;

    for path in files {
        let input_format = match input_format_for_path(&path) {
            Some(fmt) => fmt,
            None => {
                skipped += 1;
                continue;
            }
        };

        let data = fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let fixed_out = output_path(&args.input_dir, &path, &quality_dir, &output_format)?;
        let fixed_medium_out =
            output_path(&args.input_dir, &path, &quality_medium_dir, &output_format)?;

        let fixed_opts = build_fixed_opts(args, quality_high);
        let fixed_tmp = img_shrink::encode(&data, &input_format, &output_format, fixed_opts);
        total_fixed += save_tempfile(&fixed_tmp, &fixed_out)?;

        let fixed_medium_opts = build_fixed_opts(args, quality_medium);
        let fixed_medium_tmp =
            img_shrink::encode(&data, &input_format, &output_format, fixed_medium_opts);
        total_fixed_medium += save_tempfile(&fixed_medium_tmp, &fixed_medium_out)?;

        if run_high {
            let high_out = output_path(
                &args.input_dir,
                &path,
                dssim_dir.as_ref().ok_or_else(|| "dssim-high disabled".to_string())?,
                &output_format,
            )?;
            let high_opts = build_high_opts(args);
            let high_tmp = img_shrink::encode(&data, &input_format, &output_format, high_opts);
            total_high += save_tempfile(&high_tmp, &high_out)?;
        }

        if run_medium {
            let medium_out = output_path(
                &args.input_dir,
                &path,
                dssim_medium_dir
                    .as_ref()
                    .ok_or_else(|| "dssim-medium disabled".to_string())?,
                &output_format,
            )?;
            let medium_opts = build_medium_opts(args);
            let medium_tmp = img_shrink::encode(&data, &input_format, &output_format, medium_opts);
            total_medium += save_tempfile(&medium_tmp, &medium_out)?;
        }

        processed += 1;
    }

    println!("processed files: {processed}");
    println!("skipped files: {skipped}");
    println!("total size quality-85: {total_fixed} bytes");
    println!("total size quality-75: {total_fixed_medium} bytes");
    if run_high {
        println!("total size dssim-high: {total_high} bytes");
    }
    if run_medium {
        println!("total size dssim-medium: {total_medium} bytes");
    }
    Ok(())
}

fn run_webp_dssim(args: &WebpDssimArgs) -> Result<(), String> {
    if let Some(quality) = args.quality {
        if quality > 100 {
            return Err("quality must be between 0 and 100".to_string());
        }
    }

    let mut files = Vec::new();
    collect_files(&args.input_dir, args.recursive, &mut files)?;

    fs::create_dir_all(&args.output_dir).map_err(|e| format!("create dir: {e}"))?;

    let mut processed = 0u64;
    let mut skipped = 0u64;

    for path in files {
        let input_format = match input_format_for_path(&path) {
            Some(fmt) => fmt,
            None => {
                skipped += 1;
                continue;
            }
        };

        let data = fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let base_png = img_shrink::to_png(&data, &input_format);

        let out_path = output_path(&args.input_dir, &path, &args.output_dir, "webp")?;
        let (webp_tmp, chosen_quality, dist) = if let Some(quality) = args.quality {
            let webp_opts = build_webp_opts(args);
            let webp_tmp = img_shrink::encode(&data, &input_format, "webp", webp_opts);
            let cand_png = img_shrink::webp::file_to_png(&webp_tmp.path().to_path_buf());
            let dist = img_shrink::diff::distance(&base_png, &cand_png);
            let _ = fs::remove_file(cand_png);
            (webp_tmp, quality, dist)
        } else {
            let threshold = match args.preset {
                Some(DssimPreset::High) => img_shrink::HIGH_QUALITY_THRESHOLD,
                Some(DssimPreset::Medium) => img_shrink::MEDIUM_QUALITY_THRESHOLD,
                None => return Err("missing required --quality <0-100> or --preset <high|medium>".to_string()),
            };
            adaptive_webp_dssim(&base_png, args, threshold)?
        };
        save_tempfile(&webp_tmp, &out_path)?;
        println!(
            "dssim: {dist:.6} {} quality={}",
            out_path.display(),
            chosen_quality
        );

        let _ = fs::remove_file(base_png);

        processed += 1;
    }

    println!("processed files: {processed}");
    println!("skipped files: {skipped}");
    Ok(())
}

fn input_format_for_path(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "jxl" | "jpg" | "jpeg" | "png" | "webp" | "heic" | "heif" => Some(ext),
        _ => None,
    }
}

fn output_path(
    input_root: &Path,
    path: &Path,
    output_dir: &Path,
    output_format: &str,
) -> Result<PathBuf, String> {
    let rel_path = path.strip_prefix(input_root).unwrap_or(path);
    let rel_dir = rel_path.parent().unwrap_or_else(|| Path::new(""));
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("invalid filename: {}", path.display()))?;

    let filename = format!("{stem}.{output_format}");
    let target_dir = output_dir.join(rel_dir);
    fs::create_dir_all(&target_dir).map_err(|e| format!("create dir: {e}"))?;

    Ok(target_dir.join(&filename))
}

fn build_fixed_opts(args: &Args, quality: FixedQualitySpec) -> img_shrink::EncodeOptions<'_> {
    let mut builder = EncodeOptionsBuilder::new();
    if let Some(size) = args.size.as_deref() {
        builder = builder.size(size);
    }
    builder = builder.crop(args.crop);
    builder = match quality {
        FixedQualitySpec::Index(idx) => builder.quality_idx(idx),
        FixedQualitySpec::Value(value) => builder.quality_value(value),
    };
    builder.build()
}

fn build_high_opts(args: &Args) -> img_shrink::EncodeOptions<'_> {
    let mut builder = EncodeOptionsBuilder::new();
    if let Some(size) = args.size.as_deref() {
        builder = builder.size(size);
    }
    builder = builder.crop(args.crop).high();
    builder.build()
}

fn build_medium_opts(args: &Args) -> img_shrink::EncodeOptions<'_> {
    let mut builder = EncodeOptionsBuilder::new();
    if let Some(size) = args.size.as_deref() {
        builder = builder.size(size);
    }
    builder = builder.crop(args.crop).medium();
    builder.build()
}

fn build_webp_opts(args: &WebpDssimArgs) -> img_shrink::EncodeOptions<'_> {
    let mut builder = EncodeOptionsBuilder::new();
    if let Some(size) = args.size.as_deref() {
        builder = builder.size(size);
    }
    builder = builder.crop(args.crop);
    if let Some(quality) = args.quality {
        builder = builder.quality_value(quality);
    } else if let Some(preset) = args.preset {
        builder = match preset {
            DssimPreset::High => builder.high(),
            DssimPreset::Medium => builder.medium(),
        };
    }
    builder.build()
}

fn adaptive_webp_dssim(
    base_png: &PathBuf,
    args: &WebpDssimArgs,
    threshold: f32,
) -> Result<(NamedTempFile, u8, f32), String> {
    let size = args.size.as_deref().unwrap_or(">");
    let resized = img_shrink::png::resize(base_png, size, args.crop);
    let mut last: Option<(NamedTempFile, u8, f32)> = None;
    for (idx, quality) in img_shrink::webp::QUALITY_LIST.iter().enumerate() {
        let tmp = img_shrink::webp::encode(&resized, idx);
        let cand_png = img_shrink::webp::file_to_png(&tmp.path().to_path_buf());
        let dist = img_shrink::diff::distance(&resized, &cand_png);
        let _ = fs::remove_file(cand_png);
        if dist <= threshold {
            let _ = fs::remove_file(resized);
            return Ok((tmp, *quality, dist));
        }
        last = Some((tmp, *quality, dist));
    }
    let _ = fs::remove_file(resized);
    last.ok_or_else(|| "no candidates for dssim preset".to_string())
}

fn save_tempfile(tmp: &NamedTempFile, dest: &Path) -> Result<u64, String> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create dir: {e}"))?;
    }
    if dest.exists() {
        fs::remove_file(dest).map_err(|e| format!("remove file: {e}"))?;
    }
    fs::copy(tmp.path(), dest).map_err(|e| format!("write {}: {e}", dest.display()))?;
    let size = fs::metadata(dest)
        .map_err(|e| format!("metadata {}: {e}", dest.display()))?
        .len();
    Ok(size)
}

fn collect_files(root: &Path, recursive: bool, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries =
        fs::read_dir(root).map_err(|e| format!("read dir {}: {e}", root.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read dir entry: {e}"))?;
        let path = entry.path();
        if path.is_dir() {
            if recursive {
                collect_files(&path, true, out)?;
            }
            continue;
        }
        if path.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

fn quality_idx_for_target(target: u8, list: &[u8], label: &str) -> Result<usize, String> {
    if let Some(idx) = list.iter().position(|q| *q == target) {
        return Ok(idx);
    }

    let mut best_idx = 0usize;
    let mut best_quality = list[0];
    let mut best_diff = best_quality.abs_diff(target);

    for (idx, quality) in list.iter().enumerate().skip(1) {
        let diff = quality.abs_diff(target);
        if diff < best_diff || (diff == best_diff && *quality > best_quality) {
            best_idx = idx;
            best_quality = *quality;
            best_diff = diff;
        }
    }

    eprintln!(
        "warning: requested {label}={target} not in presets; using {best_quality}"
    );
    Ok(best_idx)
}

fn fixed_quality_spec(format: &str, target: u8, label: &str) -> Result<FixedQualitySpec, String> {
    if format == "webp" {
        return Ok(FixedQualitySpec::Value(target));
    }
    let list = quality_list_for_format(format)
        .ok_or_else(|| format!("unsupported output format: {format}"))?;
    let idx = quality_idx_for_target(target, list, label)?;
    Ok(FixedQualitySpec::Index(idx))
}

fn quality_list_for_format(format: &str) -> Option<&'static [u8]> {
    match format {
        "jxl" => Some(&img_shrink::jxl::QUALITY_LIST),
        "jpg" | "jpeg" => Some(&img_shrink::jpg::QUALITY_LIST),
        "heic" | "heif" => Some(&img_shrink::heic::QUALITY_LIST),
        _ => None,
    }
}

fn normalize_format(value: &str) -> String {
    value.to_ascii_lowercase()
}

fn is_supported_output_format(format: &str) -> bool {
    matches!(format, "jxl" | "jpg" | "jpeg" | "webp" | "heic" | "heif")
}

fn parse_dssim_presets(value: &str) -> Result<DssimPresets, String> {
    let value = value.to_ascii_lowercase();
    if value == "both" || value == "all" {
        return Ok(DssimPresets {
            high: true,
            medium: true,
        });
    }

    let mut presets = DssimPresets {
        high: false,
        medium: false,
    };
    for part in value.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        match part {
            "high" => presets.high = true,
            "medium" => presets.medium = true,
            _ => return Err(format!("unknown dssim preset: {part}")),
        }
    }

    if !presets.high && !presets.medium {
        return Err("no dssim presets selected".to_string());
    }
    Ok(presets)
}

fn parse_quality_value(value: &str) -> Result<u8, String> {
    let quality: u8 = value
        .parse()
        .map_err(|_| format!("invalid quality value: {value}"))?;
    if quality > 100 {
        return Err("quality must be between 0 and 100".to_string());
    }
    Ok(quality)
}

fn print_usage() {
    let bin = env::args().next().unwrap_or_else(|| "img-shrink".to_string());
    println!("Usage:");
    println!("  {bin} --input <DIR> --output <DIR> [--output-format <fmt>]");
    println!("      [--size <size>] [--crop] [--recursive]");
    println!("      [--dssim-presets <high|medium|high,medium>]");
    println!("  {bin} webp-dssim --input <DIR> --output <DIR> --quality <0-100>");
    println!("      [--size <size>] [--crop] [--recursive]");
    println!();
    println!("Runs four encodes per file by default (quality=85, quality=75, dssim high, dssim medium).");
    println!("Outputs:");
    println!("  <output>/quality-85/<name>.<fmt>");
    println!("  <output>/quality-75/<name>.<fmt>");
    println!("  <output>/dssim-high/<name>.<fmt>");
    println!("  <output>/dssim-medium/<name>.<fmt>");
    println!();
    println!("Supported input formats: jpg, jpeg, png, webp, jxl, heic, heif");
    println!("Supported output formats: jxl, jpg, jpeg, webp, heic, heif");
    println!("Default output format: jxl");
    println!("Default dssim presets: high,medium");
}

fn print_usage_webp_dssim() {
    let bin = env::args().next().unwrap_or_else(|| "img-shrink".to_string());
    println!("Usage:");
    println!("  {bin} webp-dssim --input <DIR> --output <DIR> --quality <0-100>");
    println!("  {bin} webp-dssim --input <DIR> --output <DIR> --preset <high|medium>");
    println!("      [--size <size>] [--crop] [--recursive]");
    println!();
    println!("Encodes all images to webp with the requested quality or preset and prints dssim per file.");
}

impl Args {
    fn parse_from<I>(args: &mut I) -> Result<Self, String>
    where
        I: Iterator<Item = String>,
    {
        let mut input_dir = None;
        let mut output_dir = None;
        let mut output_format = "jxl".to_string();
        let mut size = None;
        let mut crop = false;
        let mut recursive = false;
        let mut dssim_presets = DssimPresets {
            high: true,
            medium: true,
        };

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--input" => {
                    input_dir = Some(next_value(args, "--input")?);
                }
                "--output" => {
                    output_dir = Some(next_value(args, "--output")?);
                }
                "--output-format" => {
                    output_format = next_value(args, "--output-format")?;
                }
                "--size" => {
                    size = Some(next_value(args, "--size")?);
                }
                "--crop" => {
                    crop = true;
                }
                "--recursive" => {
                    recursive = true;
                }
                "--dssim-presets" => {
                    let value = next_value(args, "--dssim-presets")?;
                    dssim_presets = parse_dssim_presets(&value)?;
                }
                "-h" | "--help" => {
                    print_usage();
                    std::process::exit(0);
                }
                _ => {
                    return Err(format!("unknown argument: {arg}"));
                }
            }
        }

        let input_dir =
            input_dir.ok_or_else(|| "missing required --input <DIR>".to_string())?;
        let output_dir =
            output_dir.ok_or_else(|| "missing required --output <DIR>".to_string())?;

        Ok(Self {
            input_dir: PathBuf::from(input_dir),
            output_dir: PathBuf::from(output_dir),
            output_format,
            size,
            crop,
            recursive,
            dssim_presets,
        })
    }
}

impl WebpDssimArgs {
    fn parse_from<I>(args: &mut I) -> Result<Self, String>
    where
        I: Iterator<Item = String>,
    {
        let mut input_dir = None;
        let mut output_dir = None;
        let mut quality = None;
        let mut preset: Option<DssimPreset> = None;
        let mut size = None;
        let mut crop = false;
        let mut recursive = false;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--input" => {
                    input_dir = Some(next_value(args, "--input")?);
                }
                "--output" => {
                    output_dir = Some(next_value(args, "--output")?);
                }
                "--quality" => {
                    let value = next_value(args, "--quality")?;
                    quality = Some(parse_quality_value(&value)?);
                }
                "--preset" => {
                    let value = next_value(args, "--preset")?;
                    preset = Some(parse_dssim_preset(&value)?);
                }
                "--size" => {
                    size = Some(next_value(args, "--size")?);
                }
                "--crop" => {
                    crop = true;
                }
                "--recursive" => {
                    recursive = true;
                }
                "-h" | "--help" => {
                    print_usage_webp_dssim();
                    std::process::exit(0);
                }
                _ => {
                    return Err(format!("unknown argument: {arg}"));
                }
            }
        }

        let input_dir =
            input_dir.ok_or_else(|| "missing required --input <DIR>".to_string())?;
        let output_dir =
            output_dir.ok_or_else(|| "missing required --output <DIR>".to_string())?;
        if quality.is_none() && preset.is_none() {
            return Err("missing required --quality <0-100> or --preset <high|medium>".to_string());
        }
        if quality.is_some() && preset.is_some() {
            return Err("use only one of --quality or --preset".to_string());
        }

        Ok(Self {
            input_dir: PathBuf::from(input_dir),
            output_dir: PathBuf::from(output_dir),
            quality,
            preset,
            size,
            crop,
            recursive,
        })
    }
}

fn parse_dssim_preset(value: &str) -> Result<DssimPreset, String> {
    match value {
        "high" => Ok(DssimPreset::High),
        "medium" => Ok(DssimPreset::Medium),
        _ => Err(format!("unknown preset: {value}")),
    }
}

fn next_value<I>(args: &mut I, flag: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    args.next()
        .ok_or_else(|| format!("missing value for {flag}"))
}
