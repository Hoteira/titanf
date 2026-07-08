//! Shared OpenType layout-table machinery (GPOS/GSUB): coverage tables,
//! class definitions, and lookup-list walking with extension resolution.
//! Every read goes through the bounds-checked `get_*_be` readers, loop
//! counts come from u16 fields (bounded), and malformed structures degrade
//! to "no data" rather than panicking.

use crate::Vec;
use crate::font::{get_u16_be, get_u32_be};

/// A parsed coverage table: which glyphs a subtable applies to, and their
/// coverage indices.
pub(crate) enum Coverage {
    /// Format 1: sorted glyph list; coverage index == position.
    Glyphs(Vec<u16>),
    /// Format 2: sorted ranges of (start, end, start_coverage_index).
    Ranges(Vec<(u16, u16, u16)>),
}

impl Coverage {
    pub(crate) fn parse(b: &[u8], offset: usize) -> Option<Coverage> {
        match get_u16_be(b, offset) {
            1 => {
                let count = get_u16_be(b, offset + 2) as usize;
                if offset + 4 + count * 2 > b.len() {
                    return None;
                }
                let mut glyphs = Vec::with_capacity(count);
                for i in 0..count {
                    glyphs.push(get_u16_be(b, offset + 4 + i * 2));
                }
                Some(Coverage::Glyphs(glyphs))
            }
            2 => {
                let count = get_u16_be(b, offset + 2) as usize;
                if offset + 4 + count * 6 > b.len() {
                    return None;
                }
                let mut ranges = Vec::with_capacity(count);
                for i in 0..count {
                    let o = offset + 4 + i * 6;
                    ranges.push((
                        get_u16_be(b, o),
                        get_u16_be(b, o + 2),
                        get_u16_be(b, o + 4),
                    ));
                }
                Some(Coverage::Ranges(ranges))
            }
            _ => None,
        }
    }

    /// Coverage index of `glyph`, or None if not covered. Binary search:
    /// both formats are sorted by glyph id per spec.
    pub(crate) fn index_of(&self, glyph: u16) -> Option<u16> {
        match self {
            Coverage::Glyphs(glyphs) => glyphs.binary_search(&glyph).ok().map(|i| i as u16),
            Coverage::Ranges(ranges) => {
                let i = ranges.partition_point(|r| r.1 < glyph);
                let r = ranges.get(i)?;
                if glyph >= r.0 && glyph <= r.1 {
                    Some(r.2.wrapping_add(glyph - r.0))
                } else {
                    None
                }
            }
        }
    }

    /// Visit every covered glyph as (glyph_id, coverage_index).
    pub(crate) fn for_each(&self, mut f: impl FnMut(u16, u16)) {
        match self {
            Coverage::Glyphs(glyphs) => {
                for (i, &g) in glyphs.iter().enumerate() {
                    f(g, i as u16);
                }
            }
            Coverage::Ranges(ranges) => {
                for &(start, end, cov) in ranges {
                    if end < start {
                        continue;
                    }
                    for g in start..=end {
                        f(g, cov.wrapping_add(g - start));
                    }
                }
            }
        }
    }
}

/// A parsed class definition table; unlisted glyphs are class 0.
pub(crate) enum ClassDef {
    Fmt1 { start: u16, classes: Vec<u16> },
    Fmt2 { ranges: Vec<(u16, u16, u16)> },
}

impl ClassDef {
    pub(crate) fn parse(b: &[u8], offset: usize) -> Option<ClassDef> {
        match get_u16_be(b, offset) {
            1 => {
                let start = get_u16_be(b, offset + 2);
                let count = get_u16_be(b, offset + 4) as usize;
                if offset + 6 + count * 2 > b.len() {
                    return None;
                }
                let mut classes = Vec::with_capacity(count);
                for i in 0..count {
                    classes.push(get_u16_be(b, offset + 6 + i * 2));
                }
                Some(ClassDef::Fmt1 { start, classes })
            }
            2 => {
                let count = get_u16_be(b, offset + 2) as usize;
                if offset + 4 + count * 6 > b.len() {
                    return None;
                }
                let mut ranges = Vec::with_capacity(count);
                for i in 0..count {
                    let o = offset + 4 + i * 6;
                    ranges.push((
                        get_u16_be(b, o),
                        get_u16_be(b, o + 2),
                        get_u16_be(b, o + 4),
                    ));
                }
                Some(ClassDef::Fmt2 { ranges })
            }
            _ => None,
        }
    }

    pub(crate) fn class_of(&self, glyph: u16) -> u16 {
        match self {
            ClassDef::Fmt1 { start, classes } => {
                let idx = glyph.wrapping_sub(*start) as usize;
                if glyph >= *start && idx < classes.len() {
                    classes[idx]
                } else {
                    0
                }
            }
            ClassDef::Fmt2 { ranges } => {
                let i = ranges.partition_point(|r| r.1 < glyph);
                match ranges.get(i) {
                    Some(&(start, end, class)) if glyph >= start && glyph <= end => class,
                    _ => 0,
                }
            }
        }
    }
}

/// Lookup indices referenced by any feature with the given tag (e.g.
/// b"kern"), across all scripts/languages.
pub(crate) fn feature_lookups(b: &[u8], table_offset: usize, tag: &[u8; 4]) -> Vec<u16> {
    let mut out = Vec::new();
    let feat_list = table_offset + get_u16_be(b, table_offset + 6) as usize;
    let feat_count = get_u16_be(b, feat_list) as usize;
    let tag_be = u32::from_be_bytes(*tag);
    for i in 0..feat_count {
        let rec = feat_list + 2 + i * 6;
        if get_u32_be(b, rec) != tag_be {
            continue;
        }
        let feat = feat_list + get_u16_be(b, rec + 4) as usize;
        let lookup_count = get_u16_be(b, feat + 2) as usize;
        for j in 0..lookup_count {
            out.push(get_u16_be(b, feat + 4 + j * 2));
        }
    }
    out
}

/// Visit every subtable of one lookup as (lookup_type, subtable_offset),
/// resolving extension lookups (`ext_type`, GPOS 9 / GSUB 7) transparently.
pub(crate) fn for_each_subtable(
    b: &[u8],
    lookup_list: usize,
    lookup_index: u16,
    ext_type: u16,
    mut f: impl FnMut(u16, usize),
) {
    let lookup_count = get_u16_be(b, lookup_list);
    if lookup_index >= lookup_count {
        return;
    }
    let lookup = lookup_list + get_u16_be(b, lookup_list + 2 + lookup_index as usize * 2) as usize;
    let ty = get_u16_be(b, lookup);
    let sub_count = get_u16_be(b, lookup + 4) as usize;
    for i in 0..sub_count {
        let mut sub = lookup + get_u16_be(b, lookup + 6 + i * 2) as usize;
        let mut sub_ty = ty;
        if sub_ty == ext_type {
            // Extension subtable: format(2) extensionLookupType(2) offset(4)
            sub_ty = get_u16_be(b, sub + 2);
            let ext_off = get_u32_be(b, sub + 4) as usize;
            if sub_ty == ext_type {
                continue; // extensions may not nest
            }
            sub += ext_off;
        }
        if sub < b.len() {
            f(sub_ty, sub);
        }
    }
}

/// Offset of the lookup list within a GPOS/GSUB header.
pub(crate) fn lookup_list_offset(b: &[u8], table_offset: usize) -> usize {
    table_offset + get_u16_be(b, table_offset + 8) as usize
}
