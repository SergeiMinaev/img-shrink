# img-shrink

Rust library for image recompression with external encoders.

## WebP `sharp_yuv`

WebP encoding enables `-sharp_yuv` by default.

- API control: `EncodeOptionsBuilder::sharp_yuv(bool)`.
- Default: `true`.

## Temporary files

This crate uses temp files under `/tmp` (prefix `img-shrink_`).

- Functions that return `NamedTempFile` (`encode`, `encode_from_png`, `encode_adaptive`)
  remove the output when the value is dropped. Call `keep()` if you want to
  persist the file.
- Functions that return a `PathBuf` to a temp file (`to_png`, `png::resize`,
  `jxl::bytes_to_png`, `jpg::bytes_to_png`, `webp::bytes_to_png`,
  `heic::bytes_to_png`, and `*_::file_to_png`) leave cleanup to the caller.

Example cleanup:

```rust
let png = img_shrink::to_png(&bytes, "jpg");
// ... use `png`
let _ = std::fs::remove_file(png);
```
