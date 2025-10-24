<div align="center">
  <br>
  <br>
  <img src="https://raw.githubusercontent.com/Hoteira/titan-f/refs/heads/master/img/icon.png" alt="TitanF Logo" width="120" height="120">
  
  # TitanF
  
  **The font rasterizer that doesn't slow down**
  
  [![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
  [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
  [![no_std](https://img.shields.io/badge/no__std-compatible-success.svg)](https://docs.rust-embedded.org/book/)
  [![crates.io](https://img.shields.io/crates/v/titanf.svg)](https://crates.io/crates/titanf)

</div>

---

## Quick Start
```rust
use titanf::TrueTypeFont;

fn main() {
    let font_data = include_bytes!("Roboto-Medium.ttf");
    let mut font = TrueTypeFont::load_font(font_data);
    
    // Render a character!
    let (metrics, bitmap) = font.get_char::<false>('A', 16);
    
    //Enable built-in glyph caching
    let (metrics, bitmap) = font.get_char::<true>('B', 16);
    //                                      ^^^^
}
```

**Add to your `Cargo.toml`:**
```toml
[dependencies]
titanf = "1.0.0"
```

---

## Features

- 🚀 **Blazingly Fast** 
- 🦀 **Zero Dependencies** — Pure Rust, no external crates
- 📦 **`no_std` Compatible** — Originally built for my own OS, it works fine in baremetal environments (just needs `alloc`)
- 💯 **Stable Rust** — No nightly features, no unsafe code
- 🔧 **Built-in TrueType Parser** — Handles CMAP, GLYF, HEAD, HHEA, HMTX, KERN, LOCA, MAXP and keeps it dependency free

---

## Benchmarking Notes

**Hardware:** All benchmarks run on the same machine with consistent methodology.

**Methodology:**
- Each rasterizer called with identical parameters
- Results wrapped in `black_box()` to prevent optimization
- Multiple runs averaged for consistency
- No caching enabled

**Reproducibility:** Benchmark code available in the repo. Run it yourself:
```bash
cargo bench
```

---

## License

Licensed under the [MIT License](LICENSE-MIT).

---

## Contributing

Found a bug? Have a performance improvement? Contributions are welcome!

Please open an issue or PR on GitHub.

---

<div align="center">
  <br><sub>🦀 Pure Rust • 📦 Zero Dependencies • ⚙️ no_std Compatible</sub>
</div>


<br>
<br>



