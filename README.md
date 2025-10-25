<div align="center">
  <br>
  <img src="https://raw.githubusercontent.com/Hoteira/titan-f/refs/heads/master/img/icon.svg" alt="TitanF Logo" width="120" height="120">
  
  # TitanF
  
  **Fast TrueType font rasterization in pure Rust**
  
  [![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
  [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
  [![no_std](https://img.shields.io/badge/no__std-compatible-success.svg)](https://docs.rust-embedded.org/book/)
  [![crates.io](https://img.shields.io/crates/v/titanf.svg)](https://crates.io/crates/titanf)
  
  <sub>🦀 Pure Rust • 📦 Zero Dependencies • ⚙️ no_std Compatible</sub>
</div>

<br>

## Quick Start
```rust
use titanf::TrueTypeFont;

fn main() {
    let font_data = include_bytes!("Roboto-Medium.ttf");
    let mut font = TrueTypeFont::load_font(font_data);
    
    // Render a character
    let (metrics, bitmap) = font.get_char::('A', 16.0);
    
    // Enable built-in glyph caching
    let (metrics, bitmap) = font.get_char::('B', 16.0);
    //                                      ^^^^ caching enabled
}
```

**Add to your `Cargo.toml`:**
```toml
[dependencies]
titanf = "0.1"
```

## Features

- 🚀 **Fast** — Up to 100,000 glyphs/second at 16px
- 🦀 **Zero Dependencies** — Pure Rust, no external crates
- 📦 **`no_std` Compatible** — Works in bare-metal environments (requires `alloc`)
- 💯 **Stable Rust** — No nightly features, no unsafe code
- 🔧 **Built-in Parser** — Handles TrueType tables: `cmap`, `glyf`, `head`, `hhea`, `hmtx`, `kern`, `loca`, `maxp`

## Benchmarks

All benchmarks run on the same hardware with consistent methodology:
- Identical parameters across rasterizers
- Results wrapped in `black_box()` to prevent optimization
- Multiple runs averaged for consistency
- No caching enabled

Run benchmarks yourself:
```bash
cargo bench
```

## License

Licensed under the [MIT License](LICENSE-MIT).

## Contributing

Contributions are welcome! Open an issue or PR on GitHub.

<br>
<br>