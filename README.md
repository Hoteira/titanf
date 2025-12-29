<div align="center">
  <img src="img/icon.svg" alt="TiTanF Logo" width="120" height="120">

# TiTanF

**High-Performance, Zero-Dependency TrueType Font Rasterizer**

[![Crates.io](https://img.shields.io/crates/v/titanf.svg?style=flat-square)](https://crates.io/crates/titanf)
[![Docs.rs](https://img.shields.io/docsrs/titanf?style=flat-square)](https://docs.rs/titanf)
[![Build Status](https://img.shields.io/github/actions/workflow/status/Hoteira/titan-f/rust.yml?branch=master&style=flat-square)](https://github.com/Hoteira/titan-f/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![no_std](https://img.shields.io/badge/no__std-compatible-success.svg?style=flat-square)](https://docs.rust-embedded.org/book/)

<sub>Pure Rust • SIMD Accelerated • Memory Safe • Bare-Metal Ready</sub>
</div>

<br>

## Overview

**TiTanF** is a production-grade TrueType font rasterizer implemented entirely in Rust without any external dependencies (`libc`, `freetype`, etc.). It was engineered to explore high-performance vector graphics rendering techniques suitable for embedded systems and OS development.

The library features a hand-written parser for the TrueType format, a robust geometry processing pipeline, and a custom anti-aliased rasterizer accelerated by SIMD instructions (SSE2 on x86_64, NEON on AArch64).

## Key Features

- ** SIMD Accelerated:** Leveraging hand-written intrinsics for `x86_64` (SSE2) and `aarch64` (NEON) to optimize the pixel coverage accumulation stage.
- ** Zero Dependencies:** A strictly dependency-free library. No C bindings, no system libraries—just pure Rust.
- ** Embedded Ready:** Fully `no_std` compatible (requires `alloc`), making it ideal for kernels, bootloaders, and embedded graphical interfaces.
- ** Memory Safe:** Utilizes safe Rust for 99% of the codebase, with `unsafe` limited strictly to SIMD optimizations.
- ** Robust Parsing:** Zero-copy parsing of complex TrueType tables (`glyf`, `cmap` (Format 0, 4, 6, 12), `kern`, `hmtx`, etc.).

## Architecture

The rendering pipeline is architected into three distinct stages to maximize modularity and performance:

1.  **Parsing (`src/tables`):** 
    Raw binary data is parsed into strongly-typed Rust structures. Complex tables like `glyf` (glyph data) are lazy-loaded to minimize initial memory footprint.
2.  **Geometry (`src/geometry`):**
    -   **Contour Extraction:** Bezier curves (quadratic) are extracted from the glyph data.
    -   ** flattening:** Curves are recursively subdivided into monotonic line segments for rasterization.
3.  **Rasterization (`src/rasterizer`):**
    -   **Analytic Area:** Uses an analytic coverage-based anti-aliasing algorithm (DDA) to calculate exact pixel coverage.
    -   **Accumulation:** A SIMD-optimized parallel prefix sum converts edge deltas into the final alpha map.

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
titanf = "2.1.0"
```

**Rendering a Character:**

```rust
use titanf::TrueTypeFont;

fn main() {
    // 1. Load font bytes (e.g., from a file or include_bytes!)
    let font_data = include_bytes!("../assets/Roboto-Regular.ttf");
    let mut font = TrueTypeFont::load_font(font_data).expect("Failed to parse font");

    // 2. Configure render settings
    font.set_dpi(72.0);

    // 3. Rasterize 'A' at 24px
    // Returns metrics and a linear alpha buffer (Vec<u8>)
    let (metrics, bitmap) = font.get_char::<true>('A', 24.0);

    println!("Rendered 'A': {}x{} pixels", metrics.width, metrics.height);
}
```

## Performance

Benchmarks performed on an AMD Ryzen 9 5900X rendering **1,000 characters** (Mixed CJK & Latin).

| Font Size | **TiTanF** | RustType | ab_glyph | Fontdue |
| :--- | :--- | :--- | :--- | :--- |
| **12px** | **18.4 ms** | 18.6 ms | 16.9 ms | 4.8 ms |
| **72px** | **51.5 ms** | 54.8 ms | 51.9 ms | 24.0 ms |
| **120px** | **86.4 ms** | 99.5 ms | 98.0 ms | 51.2 ms |
| **250px** | **244.0 ms** | 304.1 ms | 296.0 ms | 165.2 ms |

*TiTanF scales significantly better than other pure-Rust alternatives at larger sizes due to its O(1) memory allocation strategy and SIMD optimizations.*

## License

Distributed under the [MIT](LICENSE) license.
