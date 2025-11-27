use crate::font::{get_u16_be, get_u32_be, TrueTypeFont};

#[derive(Debug, Copy, Clone)]
pub(crate) struct MaxpTable {
    pub(crate) version: u32,
    pub(crate) num_glyphs: u16,

    pub(crate) max_points: u16,
    pub(crate) max_contours: u16,
    pub(crate) max_composite_points: u16,
    pub(crate) max_composite_contours: u16,
    pub(crate) max_zones: u16,
    pub(crate) max_twilight_points: u16,
    pub(crate) max_storage: u16,
    pub(crate) max_function_defs: u16,
    pub(crate) max_instruction_defs: u16,
    pub(crate) max_stack_elements: u16,
    pub(crate) max_size_of_instructions: u16,
    pub(crate) max_component_elements: u16,
    pub(crate) max_component_depth: u16,
}

use crate::font::FontError;

impl MaxpTable {
    pub(crate) fn new() -> Self {
        MaxpTable {
            version: 0,
            num_glyphs: 0,

            max_points: 0,
            max_contours: 0,
            max_composite_points: 0,
            max_composite_contours: 0,
            max_zones: 0,
            max_twilight_points: 0,
            max_storage: 0,
            max_function_defs: 0,
            max_instruction_defs: 0,
            max_stack_elements: 0,
            max_size_of_instructions: 0,
            max_component_elements: 0,
            max_component_depth: 0,
        }
    }
}

impl TrueTypeFont {
    pub(crate) fn load_maxp(&mut self, font_bytes: &[u8]) -> Result<(), FontError> {
        for table in &self.tables {
            if table.table_tag == "maxp".as_bytes() {
                let offset = table.offset as usize;

                self.maxp = MaxpTable {
                    version: get_u32_be(font_bytes, offset),
                    num_glyphs: get_u16_be(font_bytes, offset + 4),

                    max_points: get_u16_be(font_bytes, offset + 6),
                    max_contours: get_u16_be(font_bytes, offset + 8),
                    max_composite_points: get_u16_be(font_bytes, offset + 10),
                    max_composite_contours: get_u16_be(font_bytes, offset + 12),
                    max_zones: get_u16_be(font_bytes, offset + 14),
                    max_twilight_points: get_u16_be(font_bytes, offset + 16),
                    max_storage: get_u16_be(font_bytes, offset + 18),
                    max_function_defs: get_u16_be(font_bytes, offset + 20),
                    max_instruction_defs: get_u16_be(font_bytes, offset + 22),
                    max_stack_elements: get_u16_be(font_bytes, offset + 24),
                    max_size_of_instructions: get_u16_be(font_bytes, offset + 26),
                    max_component_elements: get_u16_be(font_bytes, offset + 28),
                    max_component_depth: get_u16_be(font_bytes, offset + 30),
                };

                return Ok(());
            }
        }

        Err(FontError::TableNotFound("maxp"))
    }
}