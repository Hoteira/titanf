
#[cfg(feature = "std")]
use std::mem::size_of;

#[cfg(not(feature = "std"))]
use core::mem::size_of;

use crate::Vec;
use crate::Map;
use crate::tables::cmap::CmapTable;
use crate::tables::glyf::Glyph;
use crate::tables::head::HeadTable;
use crate::tables::hhea::HheaTable;
use crate::tables::hmtx::HmtxTable;
use crate::tables::loca::LocaTable;
use crate::tables::maxp::MaxpTable;

#[derive(Copy, Clone, Debug)]
pub(crate) struct OffsetTable {
    pub(crate) _scaler_type: u32,
    pub(crate) num_tables: u16,
    pub(crate) _search_range: u16,
    pub(crate) _entry_selector: u16,
    pub(crate) _range_shift: u16,
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub(crate) struct TableRecord {
    pub(crate) table_tag: [u8; 4],
    pub(crate) check_sum: u32,
    pub(crate) offset: u32,
    pub(crate) length: u32,
}

impl TableRecord {
    pub(crate) fn new() -> TableRecord {
        TableRecord {
            table_tag: [0; 4],
            check_sum: 0,
            offset: 0,
            length: 0,
        }
    }
}


pub struct TrueTypeFont {
    pub(crate) offset_table: OffsetTable,
    pub(crate) tables: Vec<TableRecord>,
    pub(crate) cmap: CmapTable,
    pub(crate) head: HeadTable,
    pub(crate) loca: LocaTable,
    pub(crate) maxp: MaxpTable,
    pub(crate) glyf: TableRecord,
    pub(crate) hhea: HheaTable,
    pub(crate) hmtx: HmtxTable,

    pub(crate) glyph_data_table: Map<u32, Glyph>,
    pub(crate) glyph_id_table: Map<char, u32>,
    pub kern_table: Map<(u32, u32), i16>,

    pub cache: crate::cache::Cache,
    pub dpi: f32,
}



impl OffsetTable {
    pub(crate) fn new() -> OffsetTable {
        OffsetTable {
            _scaler_type: 0,
            num_tables: 0,
            _search_range: 0,
            _entry_selector: 0,
            _range_shift: 0,
        }
    }
}

impl TrueTypeFont {
    pub(crate) fn new() -> Self {
        TrueTypeFont {
            offset_table: OffsetTable::new(),
            tables: Vec::new(),
            cmap: CmapTable::new(),
            head: HeadTable::new(),
            loca: LocaTable::Short(Vec::new()),
            maxp: MaxpTable::new(),
            glyf: TableRecord::new(),
            hhea: HheaTable::new(),
            hmtx: HmtxTable::new(),

            glyph_data_table: Map::new(),
            glyph_id_table: Map::new(),
            kern_table: Map::new(),

            cache: crate::cache::Cache::new(),
            dpi: 72.0,
        }
    }

    pub fn set_dpi(&mut self, dpi: f32) {
        self.dpi = dpi;
    }

    pub(crate) fn load_offset_table(&mut self, font_bytes: &[u8]) {
        if font_bytes.len() >= 12 {
            self.offset_table = OffsetTable {
                _scaler_type: get_u32_be(font_bytes, 0),
                num_tables: get_u16_be(font_bytes, 4),
                _search_range: get_u16_be(font_bytes, 6),
                _entry_selector: get_u16_be(font_bytes, 8),
                _range_shift: get_u16_be(font_bytes, 10),
            }
        } else {
            panic!("Invalid font file");
        }
    }

    pub(crate) fn load_tables(&mut self, font_bytes: &[u8]) {
        let mut offset = size_of::<OffsetTable>();

        for _i in 0..self.offset_table.num_tables {
            let table = TableRecord {
                table_tag: [font_bytes[offset], font_bytes[offset + 1], font_bytes[offset + 2], font_bytes[offset + 3]],
                check_sum: get_u32_be(font_bytes, offset + 4),
                offset: get_u32_be(font_bytes, offset + 8),
                length: get_u32_be(font_bytes, offset + 12),
            };

            self.tables.push(table);
            offset += size_of::<TableRecord>();
        }
    }

    pub fn load_font(font_bytes: &[u8]) -> Self {
        let mut font = Self::new();


        font.load_offset_table(&font_bytes);
        font.load_tables(&font_bytes);

        font.load_cmap(&font_bytes);
        font.load_cmap_encodings(&font_bytes);
        font.load_cmap_subtable_formats(&font_bytes);
        font.load_cmap_subtables(&font_bytes);
        font.load_hhea(&font_bytes);

        font.load_head(&font_bytes);
        font.load_maxp(&font_bytes);
        font.load_loca(&font_bytes);
        font.load_glyf();
        font.load_hmtx(&font_bytes);

        font.cache_all_glyphs(&font_bytes);
        font.load_kerning_pairs(&font_bytes);

        font
    }
}

#[inline]
pub fn get_u32_be(base: &[u8], offset: usize) -> u32 {
    let bytes = &base[offset..offset + 4];
    u32::from_be_bytes(bytes.try_into().expect("slice with incorrect length"))
}

#[inline]
pub fn get_u16_be(base: &[u8], offset: usize) -> u16 {
    let bytes = &base[offset..offset + 2];
    u16::from_be_bytes(bytes.try_into().expect("slice with incorrect length"))
}

#[inline]
pub fn get_i16_be(base: &[u8], offset: usize) -> i16 {
    let bytes = &base[offset..offset + 2];
    i16::from_be_bytes(bytes.try_into().expect("slice with incorrect length"))
}

#[inline]
pub fn get_i64_be(base: &[u8], offset: usize) -> i64 {
    let bytes = &base[offset..offset + 8];
    i64::from_be_bytes(bytes.try_into().expect("slice with incorrect length"))
}
