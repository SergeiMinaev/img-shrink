use std::path::PathBuf;
use crate::util;


/// Resize a PNG, returning a temp file path when a new image is produced.
///
/// If no resize is needed, this returns the input path unchanged. If a temp
/// file is returned, the caller should remove it when done.
pub fn resize(path: &PathBuf, size: &str, crop: bool) -> PathBuf {
    #[cfg(feature = "vips")]
    {
        let out_path = util::mktemp("png");
        let t0 = std::time::Instant::now();
        if let Some((target_w, target_h)) = parse_size(size) {
            if let Some((w, h)) = vips_image_size(path) {
                if w <= target_w && h <= target_h {
                    return path.clone();
                }
            }
        }
        let mut cmd = std::process::Command::new("/usr/bin/vipsthumbnail");
        cmd.arg(path.as_path())
            .arg("--size")
            .arg(size)
            .arg("-o")
            .arg(&out_path);
        if crop {
            cmd.arg("--crop");
        }
        let output = cmd.output().expect("failed to execute process");
        if output.status.success() == false {
            panic!("png::resize(vips) failed: {output:?}");
        }
        if std::env::var("IMG_SHRINK_TIMINGS").ok().as_deref() == Some("1") {
            eprintln!("vips resize: {} ms", t0.elapsed().as_millis());
        }
        return PathBuf::from(out_path);
    }

    #[cfg(feature = "magick")]
    {
        use std::process::Command;
        let out_path = util::mktemp("png");
        let output = match crop {
            false => {
                Command::new("/usr/bin/convert")
                    .arg("-resize")
                    .arg(format!("{size}>"))
                    .arg(&path.display().to_string())
                    .arg(&out_path)
                    .output()
                    .expect("failed to execute process")
            },
            true => {
                Command::new("/usr/bin/convert")
                    .arg("-resize")
                    .arg(format!("{size}>"))
                    .arg("-gravity")
                    .arg("Center")
                    .arg("-extent")
                    .arg(format!("{size}+0+0"))
                    .arg(&path.display().to_string())
                    .arg(&out_path)
                    .output()
                    .expect("failed to execute process")
            },
        };
        if output.status.success() == false {
            panic!("png::resize(magick) failed: {output:?}");
        }
        return PathBuf::from(out_path);
    }
}

#[cfg(feature = "vips")]
fn parse_size(size: &str) -> Option<(i32, i32)> {
    let clean = size.trim().trim_end_matches('>');
    let mut parts = clean.split('x');
    let w = parts.next()?.parse::<i32>().ok()?;
    let h = parts.next()?.parse::<i32>().ok()?;
    Some((w, h))
}

#[cfg(feature = "vips")]
fn vips_image_size(path: &PathBuf) -> Option<(i32, i32)> {
    use std::process::Command;
    let output = Command::new("/usr/bin/vipsheader")
        .arg("-a")
        .arg(path.as_path())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut w: Option<i32> = None;
    let mut h: Option<i32> = None;
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix("width:") {
            w = rest.trim().parse::<i32>().ok();
        } else if let Some(rest) = line.strip_prefix("height:") {
            h = rest.trim().parse::<i32>().ok();
        }
        if w.is_some() && h.is_some() {
            break;
        }
    }
    Some((w?, h?))
}

/// Write PNG bytes to a temp file and return its path.
///
/// The caller is responsible for deleting the temp file when done.
pub fn bytes_to_png(data: &Vec<u8>) -> PathBuf {
	util::bytes_to_tempfile(data, "png")
}
