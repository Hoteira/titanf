use crate::font::TrueTypeFont;
use crate::Vec;

#[derive(Debug, Clone)]
pub (crate)struct HmtxTable {
    pub(crate) h_metrics: Vec<LongHorMetric>,
    pub(crate) left_side_bearings: Vec<i16>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LongHorMetric {
    pub(crate) advance_width: u16,
    pub(crate) left_side_bearing: i16,
}

impl HmtxTable {
    pub(crate) fn new() -> HmtxTable {
        HmtxTable {
            h_metrics: Vec::new(),
            left_side_bearings: Vec::new(),
        }
    }
}

impl TrueTypeFont {

    pub(crate) fn load_hmtx(&mut self, font_bytes: &[u8]) {
        for table in &self.tables {
            if table.table_tag == "hmtx".as_bytes() {
                let table_offset = table.offset as usize;
                let mut offset = table_offset;

                let mut h_metrics = Vec::new();

                for _ in 0..self.hhea.number_of_h_metrics {
                    if offset + 4 > font_bytes.len() {
                        return;
                    }

                    h_metrics.push(LongHorMetric {
                        advance_width: u16::from_be_bytes([font_bytes[offset], font_bytes[offset + 1]]),
                        left_side_bearing: i16::from_be_bytes([font_bytes[offset + 2], font_bytes[offset + 3]]),
                    });
                    offset += 4;
                }

                let mut left_side_bearings = Vec::new();
                for _ in self.hhea.number_of_h_metrics..self.maxp.num_glyphs {
                    if offset + 2 > font_bytes.len() {
                        return;
                    }

                    left_side_bearings.push(i16::from_be_bytes([font_bytes[offset], font_bytes[offset + 1]]));
                    offset += 2;
                }

                self.hmtx = HmtxTable { h_metrics, left_side_bearings };
                return;
            }
        }

        panic!("HMTX table not found");
    }


    #[inline(always)]
    pub(crate) fn get_metrics(&self, glyph_id: &u32, scale: f32) -> (usize, isize) {
        let idx = *glyph_id as usize;

        if idx < self.hmtx.h_metrics.len() {

            let metric = self.hmtx.h_metrics[idx];
            ((metric.advance_width as f32 * scale) as usize, (metric.left_side_bearing as f32 * scale) as isize)

        } else {

            let lsb_idx = idx - self.hmtx.h_metrics.len();
            if lsb_idx < self.hmtx.left_side_bearings.len() {
                let advance = self.hmtx.h_metrics.last().unwrap().advance_width;
                let lsb = self.hmtx.left_side_bearings[lsb_idx];
                ((advance as f32 * scale) as usize, (lsb as f32 * scale) as isize)
            } else {
                (0, 0)
            }
        }
    }
}