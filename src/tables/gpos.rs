//! GPOS pair-adjustment kerning. Modern fonts carry kerning here rather
//! than in the legacy `kern` table (GSUB, its sibling, holds substitutions
//! - ligatures - not kerning).
//!
//! Lookups referenced by the `kern` feature are parsed for pair positioning
//! (lookup type 2, including type-9 extension wrappers):
//! - Format 1 (explicit glyph pairs) is folded straight into `kern_table`.
//! - Format 2 (class matrices) would explode combinatorially if expanded,
//!   so the compact class structures are kept and resolved at lookup time.

use crate::Vec;
use crate::font::{TrueTypeFont, get_u16_be, get_i16_be};
use crate::tables::otl::{ClassDef, Coverage, feature_lookups, for_each_subtable, lookup_list_offset};

/// One PairPos format-2 subtable, kept compact for lookup-time resolution.
pub(crate) struct ClassPairTable {
    coverage: Coverage,
    class1: ClassDef,
    class2: ClassDef,
    class1_count: usize,
    class2_count: usize,
    /// X-advance adjustment of the first glyph per (class1, class2).
    values: Vec<i16>,
}

impl ClassPairTable {
    /// Kerning value for a glyph pair, if this subtable covers the left
    /// glyph. `Some(0)` means "covered, no adjustment".
    pub(crate) fn lookup(&self, left: u16, right: u16) -> Option<i16> {
        self.coverage.index_of(left)?;
        let c1 = self.class1.class_of(left) as usize;
        let c2 = self.class2.class_of(right) as usize;
        if c1 < self.class1_count && c2 < self.class2_count {
            Some(self.values[c1 * self.class2_count + c2])
        } else {
            Some(0)
        }
    }
}

/// Byte size of a value record for the given value format.
#[inline]
fn value_len(vf: u16) -> usize {
    (vf as u32).count_ones() as usize * 2
}

/// Byte offset of the XAdvance field (bit 0x0004) inside a value record.
#[inline]
fn xadvance_offset(vf: u16) -> usize {
    (((vf & 0x0001) != 0) as usize + ((vf & 0x0002) != 0) as usize) * 2
}

impl TrueTypeFont {
    pub(crate) fn load_gpos_kerning(&mut self, b: &[u8]) {
        let Some(table) = self.tables.iter().find(|t| &t.table_tag == b"GPOS") else {
            return;
        };
        let gpos = table.offset as usize;

        let lookups = feature_lookups(b, gpos, b"kern");
        let lookup_list = lookup_list_offset(b, gpos);

        let mut class_tables = Vec::new();
        for li in lookups {
            for_each_subtable(b, lookup_list, li, 9, |ty, sub| {
                if ty == 2 {
                    self.parse_pair_pos(b, sub, &mut class_tables);
                }
            });
        }
        self.gpos_kern = class_tables;
    }

    fn parse_pair_pos(&mut self, b: &[u8], off: usize, class_tables: &mut Vec<ClassPairTable>) {
        let vf1 = get_u16_be(b, off + 4);
        let vf2 = get_u16_be(b, off + 6);
        // Kerning is the X-advance adjustment of the first glyph.
        if vf1 & 0x0004 == 0 {
            return;
        }
        let sz1 = value_len(vf1);
        let sz2 = value_len(vf2);
        let xadv = xadvance_offset(vf1);
        let Some(coverage) = Coverage::parse(b, off + get_u16_be(b, off + 2) as usize) else {
            return;
        };

        match get_u16_be(b, off) {
            1 => {
                // Explicit pairs: PairSet per covered first glyph.
                let pair_set_count = get_u16_be(b, off + 8);
                coverage.for_each(|first, cov_idx| {
                    if cov_idx >= pair_set_count {
                        return;
                    }
                    let ps = off + get_u16_be(b, off + 10 + cov_idx as usize * 2) as usize;
                    let count = get_u16_be(b, ps) as usize;
                    let rec_len = 2 + sz1 + sz2;
                    if ps + 2 + count * rec_len > b.len() {
                        return;
                    }
                    for j in 0..count {
                        let rec = ps + 2 + j * rec_len;
                        let second = get_u16_be(b, rec);
                        let v = get_i16_be(b, rec + 2 + xadv);
                        if v != 0 {
                            self.kern_table.insert((first as u32, second as u32), v);
                        }
                    }
                });
            }
            2 => {
                let class1 = ClassDef::parse(b, off + get_u16_be(b, off + 8) as usize);
                let class2 = ClassDef::parse(b, off + get_u16_be(b, off + 10) as usize);
                let (Some(class1), Some(class2)) = (class1, class2) else {
                    return;
                };
                let class1_count = get_u16_be(b, off + 12) as usize;
                let class2_count = get_u16_be(b, off + 14) as usize;
                let cells = class1_count.saturating_mul(class2_count);
                let rec_len = sz1 + sz2;
                // Sanity caps: the matrix must fit in the file and stay
                // within a reasonable memory budget (real fonts use a few
                // hundred classes; 1M cells = 2 MB).
                if cells == 0
                    || cells > 1_000_000
                    || off + 16 + cells.saturating_mul(rec_len) > b.len()
                {
                    return;
                }
                let mut values = Vec::with_capacity(cells);
                for i in 0..cells {
                    values.push(get_i16_be(b, off + 16 + i * rec_len + xadv));
                }
                class_tables.push(ClassPairTable {
                    coverage,
                    class1,
                    class2,
                    class1_count,
                    class2_count,
                    values,
                });
            }
            _ => {}
        }
    }
}
