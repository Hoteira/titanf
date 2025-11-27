use crate::font::{
    get_i16_be,
    get_u16_be,
    TrueTypeFont,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct HheaTable {
    pub major_version: u16,
    pub minor_version: u16,
    pub ascender: i16,
    pub descender: i16,
    pub line_gap: i16,
    pub advance_width_max: u16,
    pub min_left_side_bearing: i16,
    pub min_right_side_bearing: i16,
    pub x_max_extent: i16,
    pub caret_slope_rise: i16,
    pub caret_slope_run: i16,
    pub caret_offset: i16,
    pub reserved1: i16,
    pub reserved2: i16,
    pub reserved3: i16,
    pub reserved4: i16,
    pub metric_data_format: i16,
    pub number_of_h_metrics: u16,
}

use crate::font::FontError;

impl HheaTable {
    pub(crate) fn new() -> Self {
        HheaTable {
            major_version: 0,
            minor_version: 0,
            ascender: 0,
            descender: 0,
            line_gap: 0,
            advance_width_max: 0,
            min_left_side_bearing: 0,
            min_right_side_bearing: 0,
            x_max_extent: 0,
            caret_slope_rise: 0,
            caret_slope_run: 0,
            caret_offset: 0,
            reserved1: 0,
            reserved2: 0,
            reserved3: 0,
            reserved4: 0,
            metric_data_format: 0,
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
                    major_version: get_u16_be(font_bytes, offset),
                    minor_version: get_u16_be(font_bytes, offset + 2),
                    ascender: get_i16_be(font_bytes, offset + 4),
                    descender: get_i16_be(font_bytes, offset + 6),
                    line_gap: get_i16_be(font_bytes, offset + 8),
                    advance_width_max: get_u16_be(font_bytes, offset + 10),
                    min_left_side_bearing: get_i16_be(font_bytes, offset + 12),
                    min_right_side_bearing: get_i16_be(font_bytes, offset + 14),
                    x_max_extent: get_i16_be(font_bytes, offset + 16),
                    caret_slope_rise: get_i16_be(font_bytes, offset + 18),
                    caret_slope_run: get_i16_be(font_bytes, offset + 20),
                    caret_offset: get_i16_be(font_bytes, offset + 22),
                    reserved1: get_i16_be(font_bytes, offset + 24),
                    reserved2: get_i16_be(font_bytes, offset + 26),
                    reserved3: get_i16_be(font_bytes, offset + 28),
                    reserved4: get_i16_be(font_bytes, offset + 30),
                    metric_data_format: get_i16_be(font_bytes, offset + 32),
                    number_of_h_metrics: get_u16_be(font_bytes, offset + 34),
                };

                return Ok(());
            }
        }

        Err(FontError::TableNotFound("hhea"))
    }
}