use crate::font::{get_i16_be, get_u16_be, TrueTypeFont};
use crate::geometry::points::Contour;
use crate::tables::cmap::SupportedCmapFormats::{Format0, Format12, Format4, Format6};
use crate::tables::glyf::ProtoGlyph::{Composite, Simple};
use crate::tables::loca::LocaTable;

use crate::Vec;

pub(crate) const WE_HAVE_A_SCALE: u16 = 0x0008;
pub(crate) const WE_HAVE_AN_X_AND_Y_SCALE: u16 = 0x0040;
pub(crate) const WE_HAVE_A_TWO_BY_TWO: u16 = 0x0080;
pub(crate) const ARGS_ARE_XY_VALUES: u16 = 0x0002;
const ARGS_ARE_WORDS: u16 = 0x0001;
const MORE_COMPONENTS: u16 = 0x0020;
const WE_HAVE_INSTRUCTIONS: u16 = 0x0100;
const ROUND_XY_TO_GRID: u16 = 0x0004;

#[derive(Debug, Clone)]
pub(crate) struct SimpleGlyph {
    pub(crate) _number_of_contours: i16,
    pub(crate) x_min: i16,
    pub(crate) y_min: i16,
    pub(crate) x_max: i16,
    pub(crate) y_max: i16,
    pub(crate) end_pts_of_contours: Vec<u16>,
    pub(crate) instruction_length: u16,
    pub(crate) instructions: Vec<u8>,
    pub(crate) flags: Vec<u8>,
    pub(crate) x_coordinates: Vec<i16>,
    pub(crate) y_coordinates: Vec<i16>,
    pub(crate) points: Vec<Contour>,
}

#[derive(Debug, Clone)]
pub(crate) struct CompositeGlyph {
    pub(crate) number_of_contours: i16,
    pub(crate) x_min: i16,
    pub(crate) y_min: i16,
    pub(crate) x_max: i16,
    pub(crate) y_max: i16,
    pub(crate) components: Vec<CompositeComponent>,
    pub(crate) end_pts_of_contours: Vec<u16>,
    pub(crate) instructions: Vec<u8>,
    pub(crate) points: Vec<Contour>,
}

#[derive(Debug, Clone)]
pub(crate) struct CompositeComponent {
    pub(crate) flags: u16,
    pub(crate) glyph_index: u16,
    pub(crate) argument1: i16,
    pub(crate) argument2: i16,
    pub(crate) scale: Option<f32>,
    pub(crate) x_scale: Option<f32>,
    pub(crate) y_scale: Option<f32>,
    pub(crate) scale_01: Option<f32>,
    pub(crate) scale_10: Option<f32>,
}

#[derive(Clone)]
pub struct Glyph {
    pub points: Vec<Contour>,

    pub y_max: f32,
}

#[derive(Debug, Clone)]
pub(crate) enum ProtoGlyph {
    Simple(SimpleGlyph),
    Composite(CompositeGlyph),
    Empty,
}

impl ProtoGlyph {
    pub(crate) fn finalize(&self) -> Glyph {
        match self {
            Simple(SimpleGlyph { points, y_max, .. }) | Composite(CompositeGlyph { points, y_max, .. }) => {
                Glyph {
                    points: points.clone(),

                    y_max: *y_max as f32,
                }
            }

            _ => {
                Glyph::new()
            }
        }
    }
}

impl Glyph {
    pub(crate) fn new() -> Self {
        Glyph {
            points: Vec::new(),

            y_max: 0.0,
        }
    }
}

impl ProtoGlyph {
    pub(crate) fn get_x_min(&self) -> i16 {
        match self {
            ProtoGlyph::Simple(glyph) => glyph.x_min,
            ProtoGlyph::Composite(glyph) => glyph.x_min,
            ProtoGlyph::Empty => 0,
        }
    }

    pub(crate) fn get_x_max(&self) -> i16 {
        match self {
            ProtoGlyph::Simple(glyph) => glyph.x_max,
            ProtoGlyph::Composite(glyph) => glyph.x_max,
            ProtoGlyph::Empty => 0,
        }
    }

    pub(crate) fn get_y_min(&self) -> i16 {
        match self {
            ProtoGlyph::Simple(glyph) => glyph.y_min,
            ProtoGlyph::Composite(glyph) => glyph.y_min,
            ProtoGlyph::Empty => 0,
        }
    }

    pub(crate) fn get_y_max(&self) -> i16 {
        match self {
            ProtoGlyph::Simple(glyph) => glyph.y_max,
            ProtoGlyph::Composite(glyph) => glyph.y_max,
            ProtoGlyph::Empty => 0,
        }
    }

    pub(crate) fn get_contour_end_points(&self) -> Vec<u16> {
        match self {
            ProtoGlyph::Simple(glyph) => glyph.end_pts_of_contours.clone(),
            ProtoGlyph::Composite(glyph) => glyph.end_pts_of_contours.clone(),
            ProtoGlyph::Empty => Vec::new(),
        }
    }
}

use crate::font::FontError;

impl TrueTypeFont {
    pub(crate) fn load_glyf(&mut self) -> Result<(), FontError> {
        for table in &self.tables {
            if table.table_tag == "glyf".as_bytes() {
                self.glyf = *table;

                return Ok(());
            }
        }

        Err(FontError::TableNotFound("glyf"))
    }

    pub(crate) fn get_glyph(&self, font_bytes: &[u8], glyph_id: u32) -> ProtoGlyph {
        let (start_offset, end_offset) = match &self.loca {
            LocaTable::Short(offsets, ..) => {
                let start = offsets[glyph_id as usize] as u32 * 2;
                let end = offsets[glyph_id as usize + 1] as u32 * 2;
                (start, end)
            }

            LocaTable::Long(offsets) => {
                let start = offsets[glyph_id as usize];
                let end = offsets[glyph_id as usize + 1];
                (start, end)
            }
        };

        let glyf_length = end_offset as usize - start_offset as usize;
        let glyf_offset = self.glyf.offset as usize + start_offset as usize;

        if glyf_length == 0 { return ProtoGlyph::Empty; }

        let contours = get_i16_be(font_bytes, glyf_offset);

        if contours >= 0 {
            let mut glyph = SimpleGlyph {
                _number_of_contours: get_i16_be(font_bytes, glyf_offset),
                x_min: get_i16_be(font_bytes, glyf_offset + 2),
                y_min: get_i16_be(font_bytes, glyf_offset + 4),
                x_max: get_i16_be(font_bytes, glyf_offset + 6),
                y_max: get_i16_be(font_bytes, glyf_offset + 8),
                end_pts_of_contours: Vec::new(),
                instruction_length: 0,
                instructions: Vec::new(),
                flags: Vec::new(),
                x_coordinates: Vec::new(),
                y_coordinates: Vec::new(),
                points: Vec::new(),
            };

            let mut offset = glyf_offset + 10;
            for _i in 0..glyph._number_of_contours as usize {
                let contour = get_u16_be(font_bytes, offset);
                glyph.end_pts_of_contours.push(contour);
                offset += 2;
            }

            glyph.instruction_length = get_u16_be(font_bytes, offset);
            offset += 2;

            glyph.instructions.extend_from_slice(&font_bytes[offset..offset + glyph.instruction_length as usize]);
            offset += glyph.instruction_length as usize;


            let num_points = if glyph.end_pts_of_contours.is_empty() {
                0
            } else {
                glyph.end_pts_of_contours.last().unwrap() + 1
            } as usize;

            glyph.end_pts_of_contours.reserve(glyph._number_of_contours as usize);
            glyph.instructions.reserve(glyph.instruction_length as usize);
            glyph.flags.reserve(num_points);
            glyph.x_coordinates.reserve(num_points);
            glyph.y_coordinates.reserve(num_points);


            let mut flags_read = 0;
            while flags_read < num_points {
                let flag = font_bytes[offset];
                glyph.flags.push(flag);
                offset += 1;
                flags_read += 1;

                if flag & 0x08 != 0 {
                    let repeat_count = font_bytes[offset] as usize;
                    offset += 1;
                    for _ in 0..repeat_count {
                        glyph.flags.push(flag);
                        flags_read += 1;
                    }
                }
            }

            let mut x_coord = 0_i16;
            for i in 0..num_points {
                let flag = glyph.flags[i];
                if flag & 0x02 != 0 {
                    let delta = font_bytes[offset] as i16;
                    offset += 1;
                    if flag & 0x10 != 0 {
                        x_coord += delta;
                    } else {
                        x_coord -= delta;
                    }
                } else if flag & 0x10 == 0 {
                    let delta = get_i16_be(font_bytes, offset);
                    offset += 2;
                    x_coord += delta;
                }

                glyph.x_coordinates.push(x_coord);
            }

            let mut y_coord = 0_i16;
            for i in 0..num_points {
                let flag = glyph.flags[i];
                if flag & 0x04 != 0 {
                    let delta = font_bytes[offset] as i16;
                    offset += 1;

                    if flag & 0x20 != 0 {
                        y_coord += delta;
                    } else {
                        y_coord -= delta;
                    }
                } else if flag & 0x20 == 0 {
                    let delta = get_i16_be(font_bytes, offset);
                    offset += 2;
                    y_coord += delta;
                }

                glyph.y_coordinates.push(y_coord);
            }

            ProtoGlyph::Simple(glyph)
        } else {
            let mut glyph = CompositeGlyph {
                number_of_contours: get_i16_be(font_bytes, glyf_offset),
                x_min: get_i16_be(font_bytes, glyf_offset + 2),
                y_min: get_i16_be(font_bytes, glyf_offset + 4),
                x_max: get_i16_be(font_bytes, glyf_offset + 6),
                y_max: get_i16_be(font_bytes, glyf_offset + 8),
                components: Vec::new(),
                instructions: Vec::new(),
                end_pts_of_contours: Vec::new(),
                points: Vec::new(),
            };

            let mut offset = glyf_offset + 10;

            loop {
                let flags = get_u16_be(font_bytes, offset);
                offset += 2;

                let glyph_index = get_u16_be(font_bytes, offset);
                offset += 2;

                let mut component = CompositeComponent {
                    flags,
                    glyph_index,
                    argument1: 0,
                    argument2: 0,
                    scale: None,
                    x_scale: None,
                    y_scale: None,
                    scale_01: None,
                    scale_10: None,
                };

                if flags & ARGS_ARE_WORDS != 0 {
                    component.argument1 = get_i16_be(font_bytes, offset);
                    component.argument2 = get_i16_be(font_bytes, offset + 2);
                    offset += 4;
                } else {
                    component.argument1 = font_bytes[offset] as i8 as i16;
                    component.argument2 = font_bytes[offset + 1] as i8 as i16;
                    offset += 2;
                }

                if flags & WE_HAVE_A_SCALE != 0 {
                    component.scale = Some(get_i16_be(font_bytes, offset) as f32 / 16384.0);
                    offset += 2;
                } else if flags & WE_HAVE_AN_X_AND_Y_SCALE != 0 {
                    component.x_scale = Some(get_i16_be(font_bytes, offset) as f32 / 16384.0);
                    component.y_scale = Some(get_i16_be(font_bytes, offset) as f32 / 16384.0);
                    offset += 4;
                } else if flags & WE_HAVE_A_TWO_BY_TWO != 0 {
                    component.x_scale = Some(get_i16_be(font_bytes, offset) as f32 / 16384.0);
                    component.scale_01 = Some(get_i16_be(font_bytes, offset + 2) as f32 / 16384.0);
                    component.scale_10 = Some(get_i16_be(font_bytes, offset + 4) as f32 / 16384.0);
                    component.y_scale = Some(get_i16_be(font_bytes, offset + 6) as f32 / 16384.0);
                    offset += 8;
                }

                glyph.components.push(component);

                if flags & MORE_COMPONENTS == 0 {
                    break;
                }
            }

            if !glyph.components.is_empty() && glyph.components.last().unwrap().flags & WE_HAVE_INSTRUCTIONS != 0 {
                let instruction_length = get_u16_be(font_bytes, offset);
                offset += 2;

                for i in 0..instruction_length as usize {
                    glyph.instructions.push(font_bytes[offset + i]);
                }
            }

            ProtoGlyph::Composite(glyph)
        }
    }

    pub(crate) fn cache_all_glyphs(&mut self, font_bytes: &[u8]) {
        // Pre-allocate the vector based on num_glyphs
        if self.maxp.num_glyphs > 0 {
            self.glyph_data_table.resize(self.maxp.num_glyphs as usize, None);
        }

        if self.cmap.subtables.is_empty() {
            return;
        }

        // Helper to load and insert a glyph if not present
        // We can't easily use a closure due to borrowing self mutably and immutably.
        // But we can iterate and load.

        // First, load the "missing glyph" (index 0)
        if !self.glyph_data_table.is_empty() {
             let mut glyph_data = self.get_glyph(font_bytes, 0);
             self.glyph_data_table[0] = Some(self.load_points(&mut glyph_data, &self, &font_bytes));
        }

        match &self.cmap.subtables[0] {
            Format0 { data, .. } => {
                for codepoint in 0..256 {
                    if let Some(ch) = char::from_u32(codepoint) {
                        let glyph_id = data.glyph_id_array[codepoint as usize] as u32;
                        
                        if glyph_id != 0 && (glyph_id as usize) < self.glyph_data_table.len() {
                            if self.glyph_data_table[glyph_id as usize].is_none() {
                                let mut glyph_data = self.get_glyph(font_bytes, glyph_id);
                                self.glyph_data_table[glyph_id as usize] = Some(self.load_points(&mut glyph_data, &self, &font_bytes));
                            }
                            self.glyph_id_table.insert(ch, glyph_id);
                        }
                    }
                }
            }

            Format4 { data, .. } => {
                for seg_idx in 0..data.seg_count_x2 / 2 {
                    let start = data.start_count[seg_idx as usize];
                    let end = data.end_count[seg_idx as usize];

                    for codepoint in start..=end {
                        if let Some(ch) = char::from_u32(codepoint as u32) {
                            let glyph_id = self.cmap.get_glyph_id(ch);
                            
                            if glyph_id != 0 && (glyph_id as usize) < self.glyph_data_table.len() {
                                if self.glyph_data_table[glyph_id as usize].is_none() {
                                    let mut glyph_data = self.get_glyph(font_bytes, glyph_id);
                                    self.glyph_data_table[glyph_id as usize] = Some(self.load_points(&mut glyph_data, &self, &font_bytes));
                                }
                                self.glyph_id_table.insert(ch, glyph_id);
                            }
                        }
                    }
                }
            }

            Format6 { data, .. } => {
                for i in 0..data.entry_count {
                    let codepoint = data.first_code + i;
                    if let Some(ch) = char::from_u32(codepoint as u32) {
                        let glyph_id = data.glyph_id_array[i as usize] as u32;
                        
                        if glyph_id != 0 && (glyph_id as usize) < self.glyph_data_table.len() {
                            if self.glyph_data_table[glyph_id as usize].is_none() {
                                let mut glyph_data = self.get_glyph(font_bytes, glyph_id);
                                self.glyph_data_table[glyph_id as usize] = Some(self.load_points(&mut glyph_data, &self, &font_bytes));
                            }
                            self.glyph_id_table.insert(ch, glyph_id);
                        }
                    }
                }
            }

            Format12 { data, .. } => {
                for group in &data.groups {
                    for codepoint in group.start_char_code..=group.end_char_code {
                        if let Some(ch) = char::from_u32(codepoint) {
                            let offset = (codepoint - group.start_char_code) as usize;
                            let glyph_id = group.start_glyph_id + offset as u32;
                            
                            if glyph_id != 0 && (glyph_id as usize) < self.glyph_data_table.len() {
                                if self.glyph_data_table[glyph_id as usize].is_none() {
                                    let mut glyph_data = self.get_glyph(font_bytes, glyph_id);
                                    self.glyph_data_table[glyph_id as usize] = Some(self.load_points(&mut glyph_data, &self, &font_bytes));
                                }
                                self.glyph_id_table.insert(ch, glyph_id);
                            }
                        }
                    }
                }
            }

            _ => {}
        }
    }
}