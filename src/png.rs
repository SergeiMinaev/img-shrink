use std::path::PathBuf;
use crate::util;


pub fn resize(path: &PathBuf, size: &str, crop: bool) -> PathBuf {
    #[cfg(feature = "vips")]
    {
        let out_path = util::mktemp("png");
        let t0 = std::time::Instant::now();
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

pub fn bytes_to_png(data: &Vec<u8>) -> PathBuf {
	util::bytes_to_tempfile(data, "png")
}
