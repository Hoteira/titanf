use crate::font::TrueTypeFont;

#[derive(Debug)]
pub(crate) enum LocaTable {
    Short { offset: u32 },
    Long { offset: u32 },
    Empty,
}

use crate::font::FontError;

impl TrueTypeFont {
    pub(crate) fn load_loca(&mut self, font_bytes: &[u8]) -> Result<(), FontError> {
        for table in &self.tables {
            if table.table_tag == "loca".as_bytes() {
                let entries = self.maxp.num_glyphs as usize + 1;

                match self.head.index_to_loc_format {
                    0 => {
                        if table.offset as usize + entries * 2 > font_bytes.len() {
                            return Err(FontError::UnexpectedEndOfFile);
                        }

                        self.loca = LocaTable::Short { offset: table.offset };
                    }

                    1 => {
                        if table.offset as usize + entries * 4 > font_bytes.len() {
                            return Err(FontError::UnexpectedEndOfFile);
                        }

                        self.loca = LocaTable::Long { offset: table.offset };
                    }

                    _ => return Err(FontError::InvalidFile),
                }

                return Ok(());
            }
        }

        Err(FontError::TableNotFound("loca"))
    }
}