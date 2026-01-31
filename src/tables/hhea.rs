use crate::font::{
    get_i16_be,
    get_u16_be,
    TrueTypeFont,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct HheaTable {
    pub _major_version: u16,
    pub _minor_version: u16,
    pub _ascender: i16,
    pub _descender: i16,
    pub _line_gap: i16,
    pub _advance_width_max: u16,
    pub _min_left_side_bearing: i16,
    pub _min_right_side_bearing: i16,
    pub _x_max_extent: i16,
    pub _caret_slope_rise: i16,
    pub _caret_slope_run: i16,
    pub _caret_offset: i16,
    pub _reserved1: i16,
    pub _reserved2: i16,
    pub _reserved3: i16,
    pub _reserved4: i16,
    pub _metric_data_format: i16,
    pub number_of_h_metrics: u16,
}

use crate::font::FontError;

impl HheaTable {
    pub(crate) fn new() -> Self {
        HheaTable {
            _major_version: 0,
            _minor_version: 0,
            _ascender: 0,
            _descender: 0,
            _line_gap: 0,
            _advance_width_max: 0,
            _min_left_side_bearing: 0,
            _min_right_side_bearing: 0,
            _x_max_extent: 0,
            _caret_slope_rise: 0,
            _caret_slope_run: 0,
            _caret_offset: 0,
            _reserved1: 0,
            _reserved2: 0,
            _reserved3: 0,
            _reserved4: 0,
            _metric_data_format: 0,
            number_of_h_metrics: 0,
        }
    }
}

impl TrueTypeFont {
    pub(crate) fn load_hhea(&mut self, font_bytes: &[u8]) -> Result<(), FontError> {
        for table in &self.tables {
            if table.table_tag == "hhea".as_bytes() {
                let offset = table.offset as usize;
                self.hhea = HheaTable {
                    _major_version: get_u16_be(font_bytes, offset),
                    _minor_version: get_u16_be(font_bytes, offset + 2),
                    _ascender: get_i16_be(font_bytes, offset + 4),
                    _descender: get_i16_be(font_bytes, offset + 6),
                    _line_gap: get_i16_be(font_bytes, offset + 8),
                    _advance_width_max: get_u16_be(font_bytes, offset + 10),
                    _min_left_side_bearing: get_i16_be(font_bytes, offset + 12),
                    _min_right_side_bearing: get_i16_be(font_bytes, offset + 14),
                    _x_max_extent: get_i16_be(font_bytes, offset + 16),
                    _caret_slope_rise: get_i16_be(font_bytes, offset + 18),
                    _caret_slope_run: get_i16_be(font_bytes, offset + 20),
                    _caret_offset: get_i16_be(font_bytes, offset + 22),
                    _reserved1: get_i16_be(font_bytes, offset + 24),
                    _reserved2: get_i16_be(font_bytes, offset + 26),
                    _reserved3: get_i16_be(font_bytes, offset + 28),
                    _reserved4: get_i16_be(font_bytes, offset + 30),
                    _metric_data_format: get_i16_be(font_bytes, offset + 32),
                    number_of_h_metrics: get_u16_be(font_bytes, offset + 34),
                };

                return Ok(());
            }
        }

        Err(FontError::TableNotFound("hhea"))
    }
}