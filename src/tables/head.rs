use crate::font::{
    get_i16_be,
    get_i64_be,
    get_u16_be,
    get_u32_be,
    TrueTypeFont,
};

#[derive(Debug, Copy, Clone)]
pub(crate) struct HeadTable {
    pub(crate) _major_version: u16,
    pub(crate) _minor_version: u16,
    pub(crate) _font_revision: u32,
    pub(crate) _checksum_adjustment: u32,
    pub(crate) _magic_number: u32, // 0x5F0F3CF5
    pub(crate) _flags: u16,
    pub(crate) units_per_em: u16,
    pub(crate) _created: i64,
    pub(crate) modified: i64,
    pub(crate) x_min: i16,
    pub(crate) y_min: i16,
    pub(crate) x_max: i16,
    pub(crate) y_max: i16,
    pub(crate) mac_style: u16,
    pub(crate) lowest_rec_ppem: u16,
    pub(crate) font_direction_hint: i16,
    pub(crate) index_to_loc_format: i16,
    pub(crate) glyph_data_format: i16,
}

use crate::font::FontError;

impl HeadTable {
    pub(crate) fn new() -> Self {
        HeadTable {
            _major_version: 0,
            _minor_version: 0,
            _font_revision: 0,
            _checksum_adjustment: 0,
            _magic_number: 0,
            _flags: 0,
            units_per_em: 0,
            _created: 0,
            modified: 0,
            x_min: 0,
            y_min: 0,
            x_max: 0,
            y_max: 0,
            mac_style: 0,
            lowest_rec_ppem: 0,
            font_direction_hint: 0,
            index_to_loc_format: 0,
            glyph_data_format: 0,
        }
    }
}

impl TrueTypeFont {
    pub(crate) fn load_head(&mut self, font_bytes: &[u8]) -> Result<(), FontError> {
        for table in &self.tables {
            if table.table_tag == "head".as_bytes() {
                let offset = table.offset as usize;

                self.head = HeadTable {
                    _major_version: get_u16_be(font_bytes, offset),
                    _minor_version: get_u16_be(font_bytes, offset + 2),
                    _font_revision: get_u32_be(font_bytes, offset + 4),
                    _checksum_adjustment: get_u32_be(font_bytes, offset + 8),
                    _magic_number: get_u32_be(font_bytes, offset + 12),
                    _flags: get_u16_be(font_bytes, offset + 16),
                    units_per_em: get_u16_be(font_bytes, offset + 18),
                    _created: get_i64_be(font_bytes, offset + 20),
                    modified: get_i64_be(font_bytes, offset + 28),
                    x_min: get_i16_be(font_bytes, offset + 36),
                    y_min: get_i16_be(font_bytes, offset + 38),
                    x_max: get_i16_be(font_bytes, offset + 40),
                    y_max: get_i16_be(font_bytes, offset + 42),
                    mac_style: get_u16_be(font_bytes, offset + 44),
                    lowest_rec_ppem: get_u16_be(font_bytes, offset + 46),
                    font_direction_hint: get_i16_be(font_bytes, offset + 48),
                    index_to_loc_format: get_i16_be(font_bytes, offset + 50),
                    glyph_data_format: get_i16_be(font_bytes, offset + 52),
                };

                return Ok(());
            }
        }

        Err(FontError::TableNotFound("head"))
    }
}