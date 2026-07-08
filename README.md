<div align="center">
  <img src="img/icon.svg" alt="TiTanF Logo" width="120" height="120">

# TiTanF

**High-Performance, Zero-Dependency TrueType Font Rasterizer**

[![Crates.io](https://img.shields.io/crates/v/titanf.svg?style=flat-square)](https://crates.io/crates/titanf)
[![Docs.rs](https://img.shields.io/docsrs/titanf?style=flat-square)](https://docs.rs/titanf)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![no_std](https://img.shields.io/badge/no__std-compatible-success.svg?style=flat-square)](https://docs.rust-embedded.org/book/)

</div>

<br>

##  Overview

**TiTanF** is a production-grade TrueType font rasterizer implemented entirely in Rust without any external dependencies (`libc`, `freetype`, etc.). It was engineered for high-performance vector graphics rendering in embedded systems and OS development.

The library features a hand-written parser for the TrueType format, a robust geometry processing pipeline, and a custom anti-aliased rasterizer accelerated by SIMD instructions.

##  Key Features

- **SIMD Accelerated:** Optimized pixel coverage accumulation using SSE2/AVX2 (x86_64) and NEON (AArch64).
- **Zero Dependencies:** No C bindings, no system libraries—just pure Rust.
- **Embedded Ready:** Fully `no_std` compatible (requires `alloc`), ideal for kernels and bootloaders.
- **Panic-Free on Bad Input:** Malformed or truncated fonts return `Err` or degrade to blank glyphs; they never panic the library.
- **Memory Safe:** Overwhelmingly safe Rust; `unsafe` is confined to SIMD intrinsics and a few bounds-proven hot-path writes.
- **Robust Parsing:** Bounds-checked parsing of TrueType tables (`glyf`, `cmap`, `kern`, `hmtx`, etc.), including composite glyphs.

## Architecture

The rendering pipeline is split into three distinct stages:

1.  **Parsing (`src/tables`):** All glyphs are parsed eagerly at `load_font` and resolved into raster-ready geometry: line lists plus monotonic quadratic pieces with precomputed coefficients, bounds and winding.
2.  **Geometry (`src/geometry`):** Nothing happens per rasterization — scaling geometry into pixel space is one or two SIMD multiply-adds per line or curve piece. Curves are never flattened.
3.  **Rasterization (`src/rasterizer`):** Precounted grid walks deposit exact area coverage per pixel crossing — curve crossings solve the quadratic directly (one hardware sqrt) — followed by span-limited SIMD prefix-sum accumulation that maps coverage to alpha and re-zeroes the buffer as it goes.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
titanf = "2.6"
```

Basic usage:

```rust
use titanf::TrueTypeFont;

fn main() {
    let font_data = include_bytes!("font.ttf");
    let mut font = TrueTypeFont::load_font(font_data).expect("Failed to parse font");
    
    // Rasterize 'A' at 24px
    let (metrics, bitmap) = font.get_char::<true>('A', 24.0);
    println!("Rendered 'A': {}x{} pixels", metrics.width, metrics.height);
}
```

## Performance

A 1:1 port of Fontdue's own benchmark: rasterizing a 37-character pangram per iteration (NotoSansSC-Medium), criterion medians (`cargo bench` to reproduce). Fontdue is configured with its tessellation quality optimized for each size; TiTanF renders exact curves at every size.

| Font Size | TiTanF | Fontdue | RustType | ab_glyph |
| :--- | :--- | :--- | :--- | :--- |
| **10px** | 12.0 µs | **9.1 µs** | 34.3 µs | 31.5 µs |
| **20px** | 17.5 µs | **13.9 µs** | 41.3 µs | 38.7 µs |
| **40px** | 27.3 µs | **23.6 µs** | 64.7 µs | 62.4 µs |
| **80px** | 56.6 µs | **50.1 µs** | 113.4 µs | 110.7 µs |
| **160px** | 153.1 µs | **135.5 µs** | 264.9 µs | 257.1 µs |
| **200px** | 212.3 µs | **200.6 µs** | 377.8 µs | 374.5 µs |
| **320px** | **460.3 µs** | 504.7 µs | 824.5 µs | 844.2 µs |

TiTanF and Fontdue trade the lead: Fontdue is quicker at small sizes, the gap closes with size, and TiTanF wins outright at large sizes. Quality is not symmetric, though: TiTanF rasterizes quadratic curves *exactly* at every size (no flattening exists in the pipeline), while Fontdue reuses geometry tessellated for 40px — at large sizes its curve error grows to several pixels. TiTanF's other differentiators: zero dependencies, `no_std`, and panic-free parsing suitable for kernels, bootloaders and other embedded targets.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
