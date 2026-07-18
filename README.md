# img-shrink

Rust library for image recompression with external encoders.

## WebP `sharp_yuv`

WebP encoding enables `-sharp_yuv` by default.

- API control: `EncodeOptionsBuilder::sharp_yuv(bool)`.
- Default: `true`.

## Watermark

An optional watermark (PNG with alpha) can be composited onto the output.
It is applied after resize, so its size stays proportional to the final image.

```rust
let wm = img_shrink::Watermark::new(Path::new("logo.png"))
    .width_frac(0.25)                      // default 0.30 of base image width
    .margin_frac(0.03)                     // default 0.02 of base image width
    .corner(img_shrink::Corner::NorthWest); // default SouthEast
let opts = img_shrink::EncodeOptionsBuilder::new()
    .size("800x800")
    .watermark(wm)
    .build();
let out = img_shrink::encode(&bytes, "jpg", "webp", opts);
```

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
