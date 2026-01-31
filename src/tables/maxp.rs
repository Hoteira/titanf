use crate::font::{get_u16_be, get_u32_be, TrueTypeFont};

#[derive(Debug, Copy, Clone)]
pub(crate) struct MaxpTable {
    pub(crate) _version: u32,
    pub(crate) num_glyphs: u16,

    pub(crate) _max_points: u16,
    pub(crate) _max_contours: u16,
    pub(crate) _max_composite_points: u16,
    pub(crate) _max_composite_contours: u16,
    pub(crate) _max_zones: u16,
    pub(crate) _max_twilight_points: u16,
    pub(crate) _max_storage: u16,
    pub(crate) _max_function_defs: u16,
    pub(crate) _max_instruction_defs: u16,
    pub(crate) _max_stack_elements: u16,
    pub(crate) _max_size_of_instructions: u16,
    pub(crate) _max_component_elements: u16,
    pub(crate) _max_component_depth: u16,
}

use crate::font::FontError;

impl MaxpTable {
    pub(crate) fn new() -> Self {
        MaxpTable {
            _version: 0,
            num_glyphs: 0,

            _max_points: 0,
            _max_contours: 0,
            _max_composite_points: 0,
            _max_composite_contours: 0,
            _max_zones: 0,
            _max_twilight_points: 0,
            _max_storage: 0,
            _max_function_defs: 0,
            _max_instruction_defs: 0,
            _max_stack_elements: 0,
            _max_size_of_instructions: 0,
            _max_component_elements: 0,
            _max_component_depth: 0,
        }
    }
}

impl TrueTypeFont {
    pub(crate) fn load_maxp(&mut self, font_bytes: &[u8]) -> Result<(), FontError> {
        for table in &self.tables {
            if table.table_tag == "maxp".as_bytes() {
                let offset = table.offset as usize;

                self.maxp = MaxpTable {
                    _version: get_u32_be(font_bytes, offset),
                    num_glyphs: get_u16_be(font_bytes, offset + 4),

                    _max_points: get_u16_be(font_bytes, offset + 6),
                    _max_contours: get_u16_be(font_bytes, offset + 8),
                    _max_composite_points: get_u16_be(font_bytes, offset + 10),
                    _max_composite_contours: get_u16_be(font_bytes, offset + 12),
                    _max_zones: get_u16_be(font_bytes, offset + 14),
                    _max_twilight_points: get_u16_be(font_bytes, offset + 16),
                    _max_storage: get_u16_be(font_bytes, offset + 18),
                    _max_function_defs: get_u16_be(font_bytes, offset + 20),
                    _max_instruction_defs: get_u16_be(font_bytes, offset + 22),
                    _max_stack_elements: get_u16_be(font_bytes, offset + 24),
                    _max_size_of_instructions: get_u16_be(font_bytes, offset + 26),
                    _max_component_elements: get_u16_be(font_bytes, offset + 28),
                    _max_component_depth: get_u16_be(font_bytes, offset + 30),
                };

                return Ok(());
            }
        }

        Err(FontError::TableNotFound("maxp"))
    }
}