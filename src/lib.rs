//! # TitanF
//!
//! **TitanF** is a blazingly fast, dependency-free font rasterizer written in pure Rust.
//!
//! - 🚀 Fast
//! - 📦 Zero dependencies
//! - ⚙️ `no_std` compatible (requires `alloc`)
//! - 🎨 Subpixel anti-aliasing
//! - 🦀 Safe, stable Rust
//!
//! ```rust
//! use titanf::TrueTypeFont;
//!
//! let font_data = include_bytes!("../Roboto-Medium.ttf");
//! let mut font = TrueTypeFont::load_font(font_data).unwrap();
//!
//! let (metrics, bitmap) = font.get_char::<false>('A', 16.0);
//! //                                      ^^^^^ turn caching on/off
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
#[allow(unused_imports)]
use alloc::vec;
#[cfg(feature = "std")]
#[allow(unused_imports)]
use std::vec;

pub mod font;
pub use crate::font::TrueTypeFont;

/// Glyph rasterization and scanline algorithms
pub mod rasterizer;

/// Rendering utilities
pub mod render;

/// Caching mechanisms
pub mod cache;

/// TrueType table structures (CMAP, GLYF, etc.)
pub mod tables;

pub(crate) mod geometry;

/// A `no_std` replacement for common `f32` methods
pub trait F32NoStd {
    fn floor(self) -> f32;
    fn ceil(self) -> f32;
    fn round(self) -> f32;
    fn abs(self) -> f32;
}

impl F32NoStd for f32 {
    #[inline]
    fn floor(self) -> f32 {
        #[cfg(feature = "std")]
        return self.floor();
        #[cfg(not(feature = "std"))]
        {
            let xi = self as i32;
            if self < xi as f32 {
                xi as f32 - 1.0
            } else {
                xi as f32
            }
        }
    }

    #[inline]
    fn ceil(self) -> f32 {
        #[cfg(feature = "std")]
        return self.ceil();
        #[cfg(not(feature = "std"))]
        {
            let xi = self as i32;
            if self > xi as f32 {
                xi as f32 + 1.0
            } else {
                xi as f32
            }
        }
    }

    #[inline]
    fn round(self) -> f32 {
        #[cfg(feature = "std")]
        return self.round();
        #[cfg(not(feature = "std"))]
        {
            if self >= 0.0 {
                (self + 0.5).floor()
            } else {
                (self - 0.5).ceil()
            }
        }
    }

    #[inline]
    fn abs(self) -> f32 {
        #[cfg(feature = "std")]
        return self.abs();
        #[cfg(not(feature = "std"))]
        {
            if self < 0.0 { -self } else { self }
        }
    }
}