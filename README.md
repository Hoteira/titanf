<div align="center">
  <br>
  <img src="https://raw.githubusercontent.com/Hoteira/titanf/refs/heads/master/img/icon.svg" alt="TiTanF Logo" width="120" height="120">

# TiTanF

**High-Performance TrueType Font Rasterizer**

[![Rust](https://img.shields.io/badge/rust-stable-d66f32?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![no_std](https://img.shields.io/badge/no__std-compatible-success.svg?style=flat-square)](https://docs.rust-embedded.org/book/)
[![crates.io](https://img.shields.io/crates/v/titanf.svg?style=flat-square)](https://crates.io/crates/titanf)

<sub>Pure Rust • SIMD Accelerated • Zero Dependencies • `no_std` Ready</sub>
</div>

<br>

## Overview

**TiTanF** is a production-ready, zero-dependency TrueType font rasterizer written entirely in Rust. It was developed to investigate efficient vector graphics rendering techniques without relying on external libraries like FreeType.

It features a custom-built parser, a robust geometry processing pipeline, and a SIMD-optimized anti-aliased rasterizer.

## Features

- ⚡ **SIMD Accelerated** — Hand-written SIMD intrinsics (SSE2/AVX2 for x86_64, NEON for AArch64) for the rasterization stage.
- 🦀 **Zero Dependencies** — No `libc`, no external crates. Just pure Rust.
- 📦 **`no_std` Support** — Designed for embedded systems and bare-metal environments (requires `alloc`).
- 🛡️ **Safe** — Minimal `unsafe` code (isolated to SIMD intrinsics), leveraging the borrow checker for memory safety.
- 🔧 **Comprehensive Parsing** — Supports `cmap` (Format 4/12), `glyf`, `head`, `hhea`, `hmtx`, `kern`, `loca`, and `maxp` tables.

## Architecture

The rendering pipeline is divided into three distinct stages:

1.  **Parsing (`src/tables`)**: The raw TrueType binary is parsed into zero-copy structures where possible. Complex tables like `glyf` are lazily evaluated.
2.  **Geometry Processing (`src/geometry`)**:
    -   **Points**: Glyph contours are extracted and transformed (scaled/translated).
    -   **Lines**: Bezier curves are flattened into line segments using recursive subdivision for sub-pixel accuracy.
3.  **Rasterization (`src/rasterizer`)**:
    -   **DDA**: An analytic coverage-based anti-aliasing approach (similar to FreeType's smooth rasterizer).
    -   **Accumulation**: The coverage buffer is converted to a bitmap using a parallel prefix sum algorithm optimized with SIMD.

## Quick Start

```rust
use titanf::TrueTypeFont;

fn main() {
    // Load font data (e.g., from a file or embedded bytes)
    let font_data = include_bytes!("../Roboto-Medium.ttf");
    let mut font = TrueTypeFont::load_font(font_data).expect("Failed to load font");

    // Render character 'A' at 16px
    let (metrics, bitmap) = font.get_char::<false>('A', 16.0);

    println!("Generated {}x{} bitmap for 'A'", metrics.width, metrics.height);
}
```

## Performance

Benchmarks were conducted rendering **1,000 characters** (mixed CJK and Latin) at various font sizes.

**TiTanF** demonstrates superior scaling characteristics compared to standard libraries like `rusttype` and `ab_glyph`, particularly at larger font sizes where the SIMD optimizations significantly reduce the overhead of pixel coverage calculations.

| Size (1k chars) | **TiTanF** | RustType | ab_glyph | Fontdue |
| :--- | :--- | :--- | :--- | :--- |
| **12pt** | **18.4 ms** | 18.6 ms | 16.9 ms | 4.8 ms |
| **72pt** | **51.5 ms** | 54.8 ms | 51.9 ms | 24.0 ms |
| **120pt** | **86.4 ms** | 99.5 ms | 98.0 ms | 51.2 ms |
| **250pt** | **244.0 ms** | 304.1 ms | 296.0 ms | 165.2 ms |

*Comparison Analysis:*
- **Scaling**: While `rusttype` and `ab_glyph` degrade linearly or worse at large sizes, **TiTanF** maintains better performance, running **~20% faster** than the competition at 250pt.
- **Efficiency**: The rasterizer is highly optimized for large coverage areas, making it ideal for UI rendering at high DPIs.

*(Note: `fontdue` uses a different, highly specialized rasterization technique and remains the fastest option, but TiTanF offers a competitive pure-Rust alternative with a focus on correctness and readability.)*

## License

This project is licensed under the [MIT License](LICENSE).
