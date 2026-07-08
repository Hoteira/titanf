use crate::font::{get_i16_be, get_u16_be, get_u32_be, TrueTypeFont};
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

#[derive(Debug, Clone)]
pub(crate) struct SimpleGlyph {
    pub(crate) _number_of_contours: i16,
    pub(crate) _x_min: i16,
    pub(crate) _y_min: i16,
    pub(crate) _x_max: i16,
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
    pub(crate) _number_of_contours: i16,
    pub(crate) _x_min: i16,
    pub(crate) _y_min: i16,
    pub(crate) _x_max: i16,
    pub(crate) y_max: i16,
    pub(crate) components: Vec<CompositeComponent>,
    pub(crate) _end_pts_of_contours: Vec<u16>,
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
    /// Raw contour points; drained by `preprocess()` at font load once the
    /// resolved primitives have been built.
    pub points: Vec<Contour>,

    pub y_max: f32,

    pub(crate) pre_v_lines: Vec<crate::geometry::simd::f32x4>,
    pub(crate) pre_m_lines: Vec<crate::geometry::simd::f32x4>,
    pub(crate) pre_m_params: Vec<crate::geometry::simd::f32x4>,
    /// Monotonic quadratic pieces (font units), rasterized directly:
    /// q0 = endpoints, q1 = polynomial coefficients, q2 = reciprocals.
    pub(crate) quad_q0: Vec<crate::geometry::simd::f32x4>,
    pub(crate) quad_q1: Vec<crate::geometry::simd::f32x4>,
    pub(crate) quad_q2: Vec<crate::geometry::simd::f32x4>,
    pub(crate) quad_modes: Vec<u8>,
    pub(crate) reverse: bool,
    pub(crate) has_points: bool,
    pub(crate) min_x: f32,
    pub(crate) max_x: f32,
    pub(crate) min_y: f32,
    pub(crate) max_y: f32,
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
                    ..Glyph::new()
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

            pre_v_lines: Vec::new(),
            pre_m_lines: Vec::new(),
            pre_m_params: Vec::new(),
            quad_q0: Vec::new(),
            quad_q1: Vec::new(),
            quad_q2: Vec::new(),
            quad_modes: Vec::new(),
            reverse: false,
            has_points: false,
            min_x: 0.0,
            max_x: 0.0,
            min_y: 0.0,
            max_y: 0.0,
        }
    }
}

impl ProtoGlyph {
    pub(crate) fn _get_x_min(&self) -> i16 {
        match self {
            ProtoGlyph::Simple(glyph) => glyph._x_min,
            ProtoGlyph::Composite(glyph) => glyph._x_min,
            ProtoGlyph::Empty => 0,
        }
    }

    pub(crate) fn _get_x_max(&self) -> i16 {
        match self {
            ProtoGlyph::Simple(glyph) => glyph._x_max,
            ProtoGlyph::Composite(glyph) => glyph._x_max,
            ProtoGlyph::Empty => 0,
        }
    }

    pub(crate) fn _get_y_min(&self) -> i16 {
        match self {
            ProtoGlyph::Simple(glyph) => glyph._y_min,
            ProtoGlyph::Composite(glyph) => glyph._y_min,
            ProtoGlyph::Empty => 0,
        }
    }

    pub(crate) fn _get_y_max(&self) -> i16 {
        match self {
            ProtoGlyph::Simple(glyph) => glyph.y_max,
            ProtoGlyph::Composite(glyph) => glyph.y_max,
            ProtoGlyph::Empty => 0,
        }
    }

    pub(crate) fn _get_contour_end_points(&self) -> Vec<u16> {
        match self {
            ProtoGlyph::Simple(glyph) => glyph.end_pts_of_contours.clone(),
            ProtoGlyph::Composite(glyph) => glyph._end_pts_of_contours.clone(),
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

    /// Malformed glyph records degrade to `ProtoGlyph::Empty` instead of
    /// panicking, ensuring corrupt glyphs don't crash the font parser.
    pub(crate) fn get_glyph(&self, font_bytes: &[u8], glyph_id: u32) -> ProtoGlyph {
        self.get_glyph_checked(font_bytes, glyph_id)
            .unwrap_or(ProtoGlyph::Empty)
    }

    fn get_glyph_checked(&self, font_bytes: &[u8], glyph_id: u32) -> Option<ProtoGlyph> {
        let idx = glyph_id as usize;
        // Composite components carry arbitrary u16 indices; an out-of-range
        // one would read loca entries from inside neighboring tables and
        // parse garbage geometry. (Only valid during load, before maxp is
        // cleared - which is the only time glyphs are parsed.)
        if idx >= self.maxp.num_glyphs as usize {
            return None;
        }
        let (start_offset, end_offset) = match &self.loca {
            LocaTable::Short { offset } => {
                let start = get_u16_be(font_bytes, *offset as usize + idx * 2) as u32 * 2;
                let end = get_u16_be(font_bytes, *offset as usize + (idx + 1) * 2) as u32 * 2;
                (start, end)
            }

            LocaTable::Long { offset } => {
                let start = get_u32_be(font_bytes, *offset as usize + idx * 4);
                let end = get_u32_be(font_bytes, *offset as usize + (idx + 1) * 4);
                (start, end)
            }
            
            LocaTable::Empty => return None,
        };

        let glyf_length = (end_offset as usize).checked_sub(start_offset as usize)?;
        // Keep every read inside the glyf table; corrupt loca offsets would
        // otherwise parse "geometry" out of whatever table follows.
        if end_offset > self.glyf.length {
            return None;
        }
        let glyf_offset = self.glyf.offset as usize + start_offset as usize;

        if glyf_length == 0 { return Some(ProtoGlyph::Empty); }

        let contours = get_i16_be(font_bytes, glyf_offset);

        if contours >= 0 {
            let mut glyph = SimpleGlyph {
                _number_of_contours: get_i16_be(font_bytes, glyf_offset),
                _x_min: get_i16_be(font_bytes, glyf_offset + 2),
                _y_min: get_i16_be(font_bytes, glyf_offset + 4),
                _x_max: get_i16_be(font_bytes, glyf_offset + 6),
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
            glyph.end_pts_of_contours.reserve(glyph._number_of_contours as usize);
            for _i in 0..glyph._number_of_contours as usize {
                let contour = get_u16_be(font_bytes, offset);
                glyph.end_pts_of_contours.push(contour);
                offset += 2;
            }

            glyph.instruction_length = get_u16_be(font_bytes, offset);
            offset += 2;

            let instr_end = offset + glyph.instruction_length as usize;
            glyph.instructions.reserve(glyph.instruction_length as usize);
            glyph.instructions.extend_from_slice(font_bytes.get(offset..instr_end)?);
            offset = instr_end;

            let num_points = if glyph.end_pts_of_contours.is_empty() {
                0
            } else {
                glyph.end_pts_of_contours.last().unwrap() + 1
            } as usize;

            glyph.flags.reserve(num_points);
            glyph.x_coordinates.reserve(num_points);
            glyph.y_coordinates.reserve(num_points);


            let mut flags_read = 0;
            while flags_read < num_points {
                let flag = *font_bytes.get(offset)?;
                glyph.flags.push(flag);
                offset += 1;
                flags_read += 1;

                if flag & 0x08 != 0 {
                    let repeat_count = *font_bytes.get(offset)? as usize;
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
                    let delta = *font_bytes.get(offset)? as i16;
                    offset += 1;
                    if flag & 0x10 != 0 {
                        x_coord = x_coord.wrapping_add(delta);
                    } else {
                        x_coord = x_coord.wrapping_sub(delta);
                    }
                } else if flag & 0x10 == 0 {
                    let delta = get_i16_be(font_bytes, offset);
                    offset += 2;
                    x_coord = x_coord.wrapping_add(delta);
                }

                glyph.x_coordinates.push(x_coord);
            }

            let mut y_coord = 0_i16;
            for i in 0..num_points {
                let flag = glyph.flags[i];
                if flag & 0x04 != 0 {
                    let delta = *font_bytes.get(offset)? as i16;
                    offset += 1;

                    if flag & 0x20 != 0 {
                        y_coord = y_coord.wrapping_add(delta);
                    } else {
                        y_coord = y_coord.wrapping_sub(delta);
                    }
                } else if flag & 0x20 == 0 {
                    let delta = get_i16_be(font_bytes, offset);
                    offset += 2;
                    y_coord = y_coord.wrapping_add(delta);
                }

                glyph.y_coordinates.push(y_coord);
            }

            Some(ProtoGlyph::Simple(glyph))
        } else {
            let mut glyph = CompositeGlyph {
                _number_of_contours: get_i16_be(font_bytes, glyf_offset),
                _x_min: get_i16_be(font_bytes, glyf_offset + 2),
                _y_min: get_i16_be(font_bytes, glyf_offset + 4),
                _x_max: get_i16_be(font_bytes, glyf_offset + 6),
                y_max: get_i16_be(font_bytes, glyf_offset + 8),
                components: Vec::new(),
                instructions: Vec::new(),
                _end_pts_of_contours: Vec::new(),
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
                    component.argument1 = *font_bytes.get(offset)? as i8 as i16;
                    component.argument2 = *font_bytes.get(offset + 1)? as i8 as i16;
                    offset += 2;
                }

                if flags & WE_HAVE_A_SCALE != 0 {
                    component.scale = Some(get_i16_be(font_bytes, offset) as f32 / 16384.0);
                    offset += 2;
                } else if flags & WE_HAVE_AN_X_AND_Y_SCALE != 0 {
                    component.x_scale = Some(get_i16_be(font_bytes, offset) as f32 / 16384.0);
                    component.y_scale = Some(get_i16_be(font_bytes, offset + 2) as f32 / 16384.0);
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

            if !glyph.components.is_empty() && glyph.components.last()?.flags & WE_HAVE_INSTRUCTIONS != 0 {
                let instruction_length = get_u16_be(font_bytes, offset) as usize;
                offset += 2;
                glyph
                    .instructions
                    .extend_from_slice(font_bytes.get(offset..offset + instruction_length)?);
            }

            Some(ProtoGlyph::Composite(glyph))
        }
    }

    pub(crate) fn cache_all_glyphs(&mut self, font_bytes: &[u8]) {
        // Pre-allocate the vector based on num_glyphs
        if self.maxp.num_glyphs > 0 {
            self.glyph_data_table.resize(self.maxp.num_glyphs as usize, None);
        }

        // Load the "missing glyph" (index 0) unconditionally so get_char
        // always has a fallback, even when no usable cmap subtable exists.
        if !self.glyph_data_table.is_empty() {
             let mut glyph_data = self.get_glyph(font_bytes, 0);
             self.glyph_data_table[0] = Some(self.load_points(&mut glyph_data, font_bytes));
        }

        if self.cmap.subtables.is_empty() {
            return;
        }

        match &self.cmap.subtables[0] {
            Format0 { data, .. } => {
                for codepoint in 0..256 {
                    if let Some(ch) = char::from_u32(codepoint) {
                        let glyph_id = data.glyph_id_array[codepoint as usize] as u32;
                        
                        if glyph_id != 0 && (glyph_id as usize) < self.glyph_data_table.len() {
                            if self.glyph_data_table[glyph_id as usize].is_none() {
                                let mut glyph_data = self.get_glyph(font_bytes, glyph_id);
                                self.glyph_data_table[glyph_id as usize] = Some(self.load_points(&mut glyph_data, font_bytes));
                            }
                            self.glyph_id_table.insert(ch as u64, glyph_id);
                        }
                    }
                }
            }

            Format4 { data, .. } => {
                let seg_count = (data.seg_count_x2 / 2) as usize;

                for seg_idx in 0..seg_count {
                    let start = data.start_count[seg_idx];
                    let end   = data.end_count[seg_idx];
                    if start == 0xFFFF { continue; }

                    for codepoint in start..=end {
                        let ch = match char::from_u32(codepoint as u32) {
                            Some(c) => c,
                            None => continue,
                        };

                        let glyph_id: u32 = if data.id_range_offset[seg_idx] == 0 {
                            ((codepoint as i32 + data.id_delta[seg_idx] as i32) as u32) & 0xFFFF
                        } else {
                            // i64 arithmetic: corrupt offsets would underflow
                            // u16 here and panic in debug builds.
                            let seg_count_u16 = data.seg_count_x2 / 2;
                            let idx = data.id_range_offset[seg_idx] as i64 / 2
                                + (codepoint - data.start_count[seg_idx]) as i64
                                - (seg_count_u16 as i64 - seg_idx as i64);

                            match usize::try_from(idx).ok().and_then(|i| data.glyph_id_array.get(i)) {
                                Some(&gid) if gid != 0 => {
                                    ((gid as i32 + data.id_delta[seg_idx] as i32) as u32) & 0xFFFF
                                }
                                _ => 0,
                            }
                        };

                        if glyph_id == 0 || glyph_id as usize >= self.glyph_data_table.len() {
                            continue;
                        }

                        if self.glyph_data_table[glyph_id as usize].is_none() {
                            let mut proto = self.get_glyph(font_bytes, glyph_id);
                            self.glyph_data_table[glyph_id as usize] =
                                Some(self.load_points(&mut proto, font_bytes));
                        }

                        self.glyph_id_table.insert(ch as u64, glyph_id);
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
                                self.glyph_data_table[glyph_id as usize] = Some(self.load_points(&mut glyph_data, font_bytes));
                            }
                            self.glyph_id_table.insert(ch as u64, glyph_id);
                        }
                    }
                }
            }

            Format12 { data, .. } => {
                for group in &data.groups {
                    // Reject inverted or absurd ranges (beyond Unicode) from
                    // corrupt groups; otherwise a single bad group means
                    // billions of loop iterations.
                    if group.start_char_code > group.end_char_code
                        || group.end_char_code > 0x0010_FFFF
                    {
                        continue;
                    }
                    for codepoint in group.start_char_code..=group.end_char_code {
                        if let Some(ch) = char::from_u32(codepoint) {
                            let offset = (codepoint - group.start_char_code) as usize;
                            let glyph_id = group.start_glyph_id + offset as u32;
                            
                            if glyph_id != 0 && (glyph_id as usize) < self.glyph_data_table.len() {
                                if self.glyph_data_table[glyph_id as usize].is_none() {
                                    let mut glyph_data = self.get_glyph(font_bytes, glyph_id);
                                    self.glyph_data_table[glyph_id as usize] = Some(self.load_points(&mut glyph_data, font_bytes));
                                }
                                self.glyph_id_table.insert(ch as u64, glyph_id);
                            }
                        }
                    }
                }
            }
        }
    }
}