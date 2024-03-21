use std::process::Command;
use std::path::PathBuf;
use crate::util;


pub fn resize(path: &PathBuf, size: &str, crop: bool) -> PathBuf {
	// println!("resize to {size}");
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
        panic!("png::encode() failed: {output:?}");
    }
    return PathBuf::from(out_path)
}

pub fn bytes_to_png(data: &Vec<u8>) -> PathBuf {
	util::bytes_to_tempfile(data, "png")
}
