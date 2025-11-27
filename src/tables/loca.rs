use crate::font::{
    get_u16_be,
    get_u32_be,
    TrueTypeFont,
};

use crate::Vec;

#[derive(Debug)]
pub(crate) enum LocaTable {
    Short(Vec<u16>),
    Long(Vec<u32>),
}

use crate::font::FontError;

impl TrueTypeFont {
    pub(crate) fn load_loca(&mut self, font_bytes: &[u8]) -> Result<(), FontError> {
        for table in &self.tables {
            if table.table_tag == "loca".as_bytes() {
                match self.head.index_to_loc_format {
                    0 => {
                        let mut loca: Vec<u16> = Vec::new();

                        for i in 0..(self.maxp.num_glyphs as usize + 1) {
                            let offset = table.offset as usize + i * 2;
                            let delta = get_u16_be(font_bytes, offset);
                            loca.push(delta);
                        }

                        self.loca = LocaTable::Short(loca);
                    }

                    1 => {
                        let mut loca: Vec<u32> = Vec::new();

                        for i in 0..(self.maxp.num_glyphs as usize + 1) {
                            let offset = table.offset as usize + i * 4;
                            let delta = get_u32_be(font_bytes, offset);
                            loca.push(delta);
                        }

                        self.loca = LocaTable::Long(loca);
                    }

                    _ => {}
                }

                return Ok(());
            }
        }

        Err(FontError::TableNotFound("loca"))
    }
}