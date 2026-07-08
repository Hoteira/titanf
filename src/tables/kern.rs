use crate::font::{
    get_i16_be,
    get_u16_be,
    TrueTypeFont,
};

impl TrueTypeFont {
    pub(crate) fn load_kerning_pairs(&mut self, font_bytes: &[u8]) {
        for table in &self.tables {
            if table.table_tag == "kern".as_bytes() {
                let offset = table.offset as usize;

                let n_tables = get_u16_be(font_bytes, offset + 2);

                let mut subtable_offset = offset + 4;

                for _ in 0..n_tables {
                    let length = get_u16_be(font_bytes, subtable_offset + 2);
                    let coverage = get_u16_be(font_bytes, subtable_offset + 4);

                    let format = coverage >> 8;
                    let horizontal = (coverage & 0x01) != 0;

                    if format == 0 && horizontal {
                        let n_pairs = get_u16_be(font_bytes, subtable_offset + 6);
                        let mut pair_offset = subtable_offset + 14;

                        for _ in 0..n_pairs {
                            let left = get_u16_be(font_bytes, pair_offset);
                            let right = get_u16_be(font_bytes, pair_offset + 2);
                            let value = get_i16_be(font_bytes, pair_offset + 4);

                            self.kern_table.insert((left as u32, right as u32), value);
                            pair_offset += 6;
                        }
                    }

                    subtable_offset += length as usize;
                }

                return;
            }
        }
    }

    /// Kerning adjustment (font units, X advance of the left glyph).
    /// Checks explicit pairs first (legacy `kern` and GPOS format-1),
    /// then falls back to GPOS class matrices.
    pub fn get_kerning_by_id(&self, left: u32, right: u32) -> Option<i16> {
        if let Some(v) = self.kern_table.get(&(left, right)) {
            return Some(*v);
        }
        for table in &self.gpos_kern {
            if let Some(v) = table.lookup(left as u16, right as u16)
                && v != 0 {
                    return Some(v);
                }
        }
        None
    }

    /// Kerning adjustment (font units) for a character pair.
    pub fn get_kerning(&self, left: char, right: char) -> Option<i16> {
        let left = self.glyph_id_table.get(left as u64).copied().unwrap_or(0);
        let right = self.glyph_id_table.get(right as u64).copied().unwrap_or(0);
        self.get_kerning_by_id(left, right)
    }
}