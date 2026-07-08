use core::mem::size_of;

use crate::font::{
    get_u16_be,
    get_u32_be,
    TrueTypeFont,
};

use crate::vec;
use crate::Vec;

pub struct CmapTable {
    pub offset: usize,
    pub header: CmapHeader,
    pub encodings: Vec<EncodingRecord>,
    pub encoding_formats: Vec<u16>,
    pub subtables: Vec<SupportedCmapFormats>,
}

#[derive(Copy, Clone, Debug)]
pub struct CmapHeader {
    pub _version: u16,
    pub num_tables: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct EncodingRecord {
    pub platform_id: u16,
    pub encoding_id: u16,
    pub offset: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct SubtableFormat {
    pub format: u16,
}

#[derive(Clone, Debug)]
pub enum SupportedCmapFormats {
    Format0 {
        platform_id: u16,
        encoding_id: u16,
        data: CmapFormat0,
    },
    Format4 {
        platform_id: u16,
        encoding_id: u16,
        data: CmapFormat4,
    },
    Format6 {
        platform_id: u16,
        encoding_id: u16,
        data: CmapFormat6,
    },
    Format12 {
        platform_id: u16,
        encoding_id: u16,
        data: CmapFormat12,
    },
}

#[derive(Clone, Debug)]
pub struct CmapFormat0 {
    pub _format: u16,
    pub _length: u16,
    pub _language: u16,
    pub glyph_id_array: [u8; 256],
}

#[derive(Clone, Debug)]
pub struct CmapFormat4 {
    pub _format: u16,
    pub length: u16,
    pub _language: u16,
    pub seg_count_x2: u16,
    pub _search_range: u16,
    pub _entry_selector: u16,
    pub _range_shift: u16,
    pub end_count: Vec<u16>,
    pub reserved_pad: u16,
    pub start_count: Vec<u16>,
    pub id_delta: Vec<i16>,
    pub id_range_offset: Vec<u16>,
    pub glyph_id_array: Vec<u16>,
}

#[derive(Clone, Debug)]
pub struct CmapFormat6 {
    pub _format: u16,
    pub _length: u16,
    pub _language: u16,
    pub first_code: u16,
    pub entry_count: u16,
    pub glyph_id_array: Vec<u16>,
}

#[derive(Debug, Clone)]
pub struct CmapFormat12 {
    pub _format: u16,
    pub _reserved: u16,
    pub _length: u32,
    pub _language: u32,
    pub _num_groups: u32,
    pub groups: Vec<SequentialMapGroup>,
}

#[derive(Debug, Clone)]
pub struct SequentialMapGroup {
    pub start_char_code: u32,
    pub end_char_code: u32,
    pub start_glyph_id: u32,
}

use crate::font::FontError;

impl CmapTable {
    pub fn new() -> CmapTable {
        CmapTable {
            offset: 0,
            header: CmapHeader { _version: 0, num_tables: 0 },
            encodings: Vec::new(),
            encoding_formats: Vec::new(),
            subtables: Vec::new(),
        }
    }
}

impl TrueTypeFont {
    pub fn load_cmap(&mut self, font_bytes: &[u8]) -> Result<(), FontError> {
        for table in &self.tables {
            if table.table_tag == "cmap".as_bytes() {
                self.cmap.offset = table.offset as usize;
                self.cmap.header = CmapHeader {
                    _version: get_u16_be(font_bytes, self.cmap.offset),
                    num_tables: get_u16_be(font_bytes, self.cmap.offset + 2),
                };
                return Ok(());
            }
        }

        Err(FontError::TableNotFound("cmap"))
    }

    pub fn load_cmap_encodings(&mut self, font_bytes: &[u8]) {
        let mut offset = self.cmap.offset + size_of::<CmapHeader>();
        for _ in 0..self.cmap.header.num_tables {
            let record = EncodingRecord {
                platform_id: get_u16_be(font_bytes, offset),
                encoding_id: get_u16_be(font_bytes, offset + 2),
                offset: get_u32_be(font_bytes, offset + 4),
            };
            self.cmap.encodings.push(record);
            offset += size_of::<EncodingRecord>();
        }
    }

    pub fn load_cmap_subtable_formats(&mut self, font_bytes: &[u8]) {
        for encoding in &self.cmap.encodings {
            let subtable_offset = self.cmap.offset + encoding.offset as usize;
            let subtable = SubtableFormat {
                format: get_u16_be(font_bytes, subtable_offset),
            };

            self.cmap.encoding_formats.push(subtable.format);
        }
    }

    pub fn load_cmap_subtables(&mut self, font_bytes: &[u8]) {
        for (count, sf) in self.cmap.encoding_formats.clone().iter().enumerate() {
            let platform_id = self.cmap.encodings[count].platform_id;
            let encoding_id = self.cmap.encodings[count].encoding_id;
            let offset = self.cmap.offset + self.cmap.encodings[count].offset as usize;

            // Subtables that run past the end of the file are skipped rather
            // than parsed with garbage: a missing mapping degrades to the
            // missing-glyph, a bad mapping renders the wrong glyph.
            match sf {
                0 => {
                    let Some(ids) = font_bytes.get(offset + 6..offset + 262) else {
                        continue;
                    };
                    let fmt = CmapFormat0 {
                        _format: get_u16_be(font_bytes, offset),
                        _length: get_u16_be(font_bytes, offset + 2),
                        _language: get_u16_be(font_bytes, offset + 4),
                        glyph_id_array: ids.try_into().unwrap(),
                    };

                    self.cmap.subtables.push(SupportedCmapFormats::Format0 {
                        platform_id,
                        encoding_id,
                        data: fmt,
                    });
                }
                4 => {
                    let seg_count = get_u16_be(font_bytes, offset + 6) as usize / 2;

                    // Header + four parallel seg_count arrays + reservedPad
                    // must fit in the file.
                    let arrays_end = offset + 14 + seg_count * 8 + 2;
                    if arrays_end > font_bytes.len() {
                        continue;
                    }

                    let mut fmt = CmapFormat4 {
                        _format: get_u16_be(font_bytes, offset),
                        length: get_u16_be(font_bytes, offset + 2),
                        _language: get_u16_be(font_bytes, offset + 4),
                        seg_count_x2: (seg_count * 2) as u16,
                        _search_range: get_u16_be(font_bytes, offset + 8),
                        _entry_selector: get_u16_be(font_bytes, offset + 10),
                        _range_shift: get_u16_be(font_bytes, offset + 12),
                        end_count: vec![0; seg_count],
                        reserved_pad: 0,
                        start_count: vec![0; seg_count],
                        id_delta: vec![0; seg_count],
                        id_range_offset: vec![0; seg_count],
                        glyph_id_array: Vec::new(),
                    };

                    let mut base_offset = offset + 14;
                    for i in 0..seg_count {
                        fmt.end_count[i] = u16::from_be_bytes([font_bytes[base_offset], font_bytes[base_offset + 1]]);
                        base_offset += 2;
                    }

                    fmt.reserved_pad = u16::from_be_bytes([font_bytes[base_offset], font_bytes[base_offset + 1]]);
                    base_offset += 2;
                    for i in 0..seg_count {
                        fmt.start_count[i] = u16::from_be_bytes([font_bytes[base_offset], font_bytes[base_offset + 1]]);
                        base_offset += 2;
                    }

                    for i in 0..seg_count {
                        fmt.id_delta[i] = i16::from_be_bytes([font_bytes[base_offset], font_bytes[base_offset + 1]]);
                        base_offset += 2;
                    }

                    for i in 0..seg_count {
                        fmt.id_range_offset[i] = u16::from_be_bytes([font_bytes[base_offset], font_bytes[base_offset + 1]]);
                        base_offset += 2;
                    }

                    // Trailing glyph id array: bounded by both the declared
                    // subtable length and the actual end of the file.
                    let declared = (fmt.length as usize)
                        .saturating_sub(base_offset - offset)
                        / 2;
                    let available = (font_bytes.len() - base_offset) / 2;
                    let glyph_id_len = declared.min(available);

                    fmt.glyph_id_array = vec![0; glyph_id_len];
                    for i in 0..glyph_id_len {
                        fmt.glyph_id_array[i] = u16::from_be_bytes([font_bytes[base_offset], font_bytes[base_offset + 1]]);
                        base_offset += 2;
                    }

                    self.cmap.subtables.push(SupportedCmapFormats::Format4 {
                        platform_id,
                        encoding_id,
                        data: fmt,
                    });
                }
                6 => {
                    let entry_count = get_u16_be(font_bytes, offset + 8) as usize;
                    if offset + 10 + entry_count * 2 > font_bytes.len() {
                        continue;
                    }

                    let mut fmt = CmapFormat6 {
                        _format: get_u16_be(font_bytes, offset),
                        _length: get_u16_be(font_bytes, offset + 2),
                        _language: get_u16_be(font_bytes, offset + 4),
                        first_code: get_u16_be(font_bytes, offset + 6),
                        entry_count: entry_count as u16,
                        glyph_id_array: vec![0; entry_count],
                    };

                    let mut base_offset = offset + 10;
                    for i in 0..entry_count {
                        fmt.glyph_id_array[i] = u16::from_be_bytes([font_bytes[base_offset], font_bytes[base_offset + 1]]);
                        base_offset += 2;
                    }

                    self.cmap.subtables.push(SupportedCmapFormats::Format6 {
                        platform_id,
                        encoding_id,
                        data: fmt,
                    });
                }
                12 => {
                    // Cap the group count by the bytes actually present so a
                    // corrupt header can't request a multi-gigabyte Vec.
                    let declared_groups = get_u32_be(font_bytes, offset + 12) as usize;
                    let base = offset + 16;
                    let available_groups = font_bytes
                        .len()
                        .saturating_sub(base)
                        / size_of::<SequentialMapGroup>();
                    let num_groups = declared_groups.min(available_groups);

                    let mut fmt = CmapFormat12 {
                        _format: get_u16_be(font_bytes, offset),
                        _reserved: get_u16_be(font_bytes, offset + 2),
                        _length: get_u32_be(font_bytes, offset + 4),
                        _language: get_u32_be(font_bytes, offset + 8),
                        _num_groups: num_groups as u32,
                        groups: Vec::with_capacity(num_groups),
                    };

                    let mut base_offset = base;
                    for _ in 0..num_groups {
                        let smg = SequentialMapGroup {
                            start_char_code: get_u32_be(font_bytes, base_offset),
                            end_char_code: get_u32_be(font_bytes, base_offset + 4),
                            start_glyph_id: get_u32_be(font_bytes, base_offset + 8),
                        };

                        fmt.groups.push(smg);
                        base_offset += size_of::<SequentialMapGroup>();
                    }

                    self.cmap.subtables.push(SupportedCmapFormats::Format12 {
                        platform_id,
                        encoding_id,
                        data: fmt,
                    });
                }
                _ => {}
            }
        }

        self.cmap.subtables.sort_by_key(|subtable| match subtable {
            SupportedCmapFormats::Format0 { platform_id, encoding_id, .. } |
            SupportedCmapFormats::Format4 { platform_id, encoding_id, .. } |
            SupportedCmapFormats::Format6 { platform_id, encoding_id, .. } |
            SupportedCmapFormats::Format12 { platform_id, encoding_id, .. } => {
                match (*platform_id, *encoding_id) {
                    (0, _) => 0,      // Unicode
                    (3, 10) => 1,     // Windows
                    (3, 1) => 2,      // Windows Unicode
                    _ => 3,
                }
            }
        });

        self.cmap.subtables.truncate(1);
    }
}