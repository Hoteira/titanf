//! # TitanF
//!
//! **TitanF** is a blazingly fast, dependency-free font rasterizer written in pure Rust.
//!
//! - 🚀 Zero dependencies
//! - ⚙️ `no_std` compatible (requires `alloc`)
//! - 🎨 Subpixel anti-aliasing
//! - 🦀 Safe, stable Rust (no unsafe)
//!
//! ```rust
//! use titanf::TrueTypeFont;
//!
//! let font_data = include_bytes!("Roboto-Medium.ttf");
//! let mut font = TrueTypeFont::load_font(font_data);
//!
//! let (metrics, bitmap) = font.get_char::<false>('A', 16);
//! ```
//!
//! ## See Also
//! - [GitHub Repository](https://github.com/Hoteira/titan-f)
//! - [Crates.io Page](https://crates.io/crates/titanf)

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as Map;

#[cfg(feature = "std")]
use std::collections::HashMap as Map;


#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::vec::Vec;


#[cfg(not(feature = "std"))]
use alloc::vec;

#[cfg(feature = "std")]
use std::vec;


/// Font parsing and metrics structures
pub mod font;

/// Glyph rasterization and scanline algorithms
pub mod rasterizer;

/// Rendering utilities
pub mod render;

/// Caching mechanisms
pub mod cache;

/// Font table structures (CMAP, GLYF, etc.)
pub mod tables;
mod preprocess;

pub use crate::font::TrueTypeFont;

pub trait F32NoStd {
    fn floor(self) -> f32;
    fn ceil(self) -> f32;
    fn round(self) -> f32;
    fn abs(self) -> f32;
}


/// A `no_std` replacement for common `f32` methods
/// (floor, ceil, round, abs) for environments without `std`.
impl F32NoStd for f32 {
    #[inline]
    fn floor(self) -> f32 {
        let xi = self as i32;
        if self < xi as f32 {
            xi as f32 - 1.0
        } else {
            xi as f32
        }
    }

    #[inline]
    fn ceil(self) -> f32 {
        let xi = self as i32;
        if self > xi as f32 {
            xi as f32 + 1.0
        } else {
            xi as f32
        }
    }

    #[inline]
    fn round(self) -> f32 {
        if self >= 0.0 {
            (self + 0.5).floor()
        } else {
            (self - 0.5).ceil()
        }
    }
    
    #[inline]
    fn abs(self) -> f32 {
        if self < 0.0 { -self } else { self }
    }
}


