use crate::tables::glyf::Glyph;
use crate::Vec;

#[cfg(not(feature = "std"))]
use crate::F32NoStd;

#[derive(Debug, Clone)]
pub struct Line {
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,

    pub dx: f32,
    pub dy: f32,

    pub dx_sign: i32,
    pub dy_sign: i32,

    pub dt_dx: f32,
    pub dt_dy: f32,

    pub is_degen: bool,

    pub abs_dx: f32,
    pub abs_dy: f32,

    pub dx_is_zero: bool,
    pub dy_is_zero: bool,
}

#[derive(Debug, Clone)]
pub struct Bounds {
    pub _x: f32,
    pub _y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for Bounds {
    fn default() -> Self {
        Bounds {
            _x: 0.0,
            _y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Segment {
    pub a_x: f32,
    pub a_y: f32,
    pub at: f32,
    pub c_x: f32,
    pub c_y: f32,
    pub ct: f32,
}

impl Segment {
    fn new(a_x: f32, a_y: f32, at: f32, c_x: f32, c_y: f32, ct: f32) -> Self {
        Segment { a_x, a_y, at, c_x, c_y, ct }
    }
}

pub struct GlyphLines {
    pub v_lines: Vec<Line>,
    pub m_lines: Vec<Line>,
    pub lines: Vec<Line>,
    pub bounds: Bounds,
}

impl GlyphLines {
    pub fn new() -> Self {
        Self {
            v_lines: Vec::new(),
            m_lines: Vec::new(),
            lines: Vec::new(),
            bounds: Bounds::default(),
        }
    }

    pub fn clear(&mut self) {
        self.v_lines.clear();
        self.m_lines.clear();
        self.lines.clear();
        self.bounds = Bounds::default();
    }
}

impl Glyph {
    pub(crate) fn build_lines<const COMPLETE: bool>(&self, units_per_em: f32, scale: f32) -> GlyphLines {
        let mut out = GlyphLines::new();
        let mut segments = Vec::new();
        self.build_lines_into::<COMPLETE>(units_per_em, scale, &mut out, &mut segments);
        out
    }

    pub(crate) fn build_lines_into<const COMPLETE: bool>(&self, _units_per_em: f32, scale: f32, out: &mut GlyphLines, line_segments: &mut Vec<(f32, f32, f32, f32)>) {
        out.clear();
        line_segments.clear();

        let temp = 0.1 / scale;
        let tolerance_sq = temp * temp / 9.0;

        let mut x_min = f32::MAX;
        let mut x_max = f32::MIN;
        let mut y_min = f32::MAX;
        let mut y_max = f32::MIN;

        for contour in &self.points {
            let points = &contour.points;
            if points.is_empty() {
                continue;
            }

            for p in points {
                x_min = x_min.min(p.x);
                x_max = x_max.max(p.x);
                y_min = y_min.min(p.y);
                y_max = y_max.max(p.y);
            }

            let mut first_on_curve: Option<(f32, f32)> = None;
            let mut first_off_curve: Option<(f32, f32)> = None;
            let mut last_off_curve: Option<(f32, f32)> = None;
            let mut current_pos = (0.0, 0.0);

            let mut i = 0;
            while i < points.len() {
                let curr = &points[i];
                let x = curr.x;
                let y = curr.y;
                let on_curve = curr.on_curve;

                if first_on_curve.is_none() {
                    if on_curve {
                        first_on_curve = Some((x, y));
                        current_pos = (x, y);
                        i += 1;
                    } else {
                        if let Some(offcurve) = first_off_curve {
                            let mid_x = (offcurve.0 + x) * 0.5;
                            let mid_y = (offcurve.1 + y) * 0.5;
                            first_on_curve = Some((mid_x, mid_y));
                            last_off_curve = Some((x, y));
                            current_pos = (mid_x, mid_y);
                            i += 1;
                        } else {
                            first_off_curve = Some((x, y));
                            i += 1;
                        }
                    }
                } else {
                    if on_curve {
                        if let Some(offcurve) = last_off_curve {
                            last_off_curve = None;
                            Self::flatten_quad(current_pos.0, current_pos.1, offcurve.0, offcurve.1, x, y, tolerance_sq, line_segments);
                        } else {
                            line_segments.push((current_pos.0, current_pos.1, x, y));
                        }
                        current_pos = (x, y);
                        i += 1;
                    } else {
                        let ctrl_x = x;
                        let ctrl_y = y;
                        let next_idx = (i + 1) % points.len();
                        let next = &points[next_idx];
                        let (next_x, next_y) = if next.on_curve {
                            if i + 1 < points.len() { i += 1; }
                            (next.x, next.y)
                        } else {
                            ((ctrl_x + next.x) / 2.0, (ctrl_y + next.y) / 2.0)
                        };
                        Self::flatten_quad(current_pos.0, current_pos.1, ctrl_x, ctrl_y, next_x, next_y, tolerance_sq, line_segments);
                        current_pos = (next_x, next_y);
                        i += 1;
                    }
                }
            }

            if let (Some(start), Some(off1)) = (first_on_curve, first_off_curve) {
                Self::flatten_quad(current_pos.0, current_pos.1, off1.0, off1.1, start.0, start.1, tolerance_sq, line_segments);
            } else if current_pos != first_on_curve.unwrap_or(current_pos) {
                 let start = first_on_curve.unwrap();
                 line_segments.push((current_pos.0, current_pos.1, start.0, start.1));
            }
        }

        if x_min == f32::MAX {
             out.clear();
             return;
        }

        let mut area = 0.0;
        for (x0, y0, x1, y1) in line_segments.iter() {
            area += (y1 - y0) * (x1 + x0);
        }
        let reverse = area > 0.0;

        let shift_x = (x_min * scale).floor() / scale;
        let shift_y = (y_max * scale).ceil() / scale;

        let width_aligned = ((x_max * scale).ceil() - (x_min * scale).floor()) / scale;
        let height_aligned = ((y_max * scale).ceil() - (y_min * scale).floor()) / scale;

        let width_scaled = width_aligned * scale;
        let height_scaled = height_aligned * scale;

        // Stem Darkening amount: roughly 1/50th of the pixel size
        let darkening = 0.02; 

        if COMPLETE {
            out.v_lines.reserve(self.points.len() * 3);
            out.m_lines.reserve(self.points.len() * 3);
            out.lines.reserve(self.points.len() * 4);

            for (x0, y0, x1, y1) in line_segments.iter() {
                let (px0, py0, px1, py1) = if reverse { (*x1, *y1, *x0, *y0) } else { (*x0, *y0, *x1, *y1) };
                let nx0 = px0 - shift_x;
                let ny0 = shift_y - py0;
                let nx1 = px1 - shift_x;
                let ny1 = shift_y - py1;
                insert_complete_line(&mut out.v_lines, &mut out.m_lines, &mut out.lines, nx0, ny0, nx1, ny1, scale, darkening);
            }
        } else {
            out.v_lines.reserve(self.points.len() * 3);
            out.m_lines.reserve(self.points.len() * 3);

            for (x0, y0, x1, y1) in line_segments.iter() {
                let (px0, py0, px1, py1) = if reverse { (*x1, *y1, *x0, *y0) } else { (*x0, *y0, *x1, *y1) };
                let nx0 = px0 - shift_x;
                let ny0 = shift_y - py0;
                let nx1 = px1 - shift_x;
                let ny1 = shift_y - py1;
                insert_line(&mut out.v_lines, &mut out.m_lines, nx0, ny0, nx1, ny1, scale, darkening);
            }
        }

        for line in out.v_lines.iter_mut().chain(out.m_lines.iter_mut()).chain(out.lines.iter_mut()) {
            if line.x0 < 0.0 { line.x0 = 0.0; }
            if line.x0 > width_scaled { line.x0 = width_scaled; }
            if line.x1 < 0.0 { line.x1 = 0.0; }
            if line.x1 > width_scaled { line.x1 = width_scaled; }
            if line.y0 < 0.0 { line.y0 = 0.0; }
            if line.y0 > height_scaled { line.y0 = height_scaled; }
            if line.y1 < 0.0 { line.y1 = 0.0; }
            if line.y1 > height_scaled { line.y1 = height_scaled; }

            line.dx = line.x1 - line.x0;
            line.dy = line.y1 - line.y0;
            line.dx_is_zero = line.dx.abs() < 1e-6;
            line.dy_is_zero = line.dy.abs() < 1e-6;
            line.dx_sign = if line.dx != 0.0 { line.dx.signum() as i32 } else { 0 };
            line.dy_sign = if line.dy != 0.0 { line.dy.signum() as i32 } else { 0 };
            line.dt_dx = if !line.dx_is_zero { 1.0 / line.dx.abs() } else { f32::MAX };
            line.dt_dy = if !line.dy_is_zero { 1.0 / line.dy.abs() } else { f32::MAX };
            line.is_degen = line.dx_is_zero && line.dy_is_zero;
            line.abs_dx = line.dx.abs();
            line.abs_dy = line.dy.abs();
        }

        out.bounds = Bounds {
            _x: 0.0,
            _y: 0.0,
            width: width_aligned,
            height: height_aligned,
        };
    }

    fn flatten_quad(
        p0_x: f32, p0_y: f32,
        p1_x: f32, p1_y: f32,
        p2_x: f32, p2_y: f32,
        tolerance_sq: f32,
        output: &mut Vec<(f32, f32, f32, f32)>
    ) {
        let mut stack = [Segment::default(); 64];
        let mut stack_count ;
        stack[0] = Segment::new(p0_x, p0_y, 0.0, p2_x, p2_y, 1.0);
        stack_count = 1;
        while stack_count > 0 {
            stack_count -= 1;
            let seg = stack[stack_count];
            let bt = (seg.at + seg.ct) * 0.5;
            let tm = 1.0 - bt;
            let a = tm * tm;
            let b = 2.0 * tm * bt;
            let c = bt * bt;
            let b_x = a * p0_x + b * p1_x + c * p2_x;
            let b_y = a * p0_y + b * p1_y + c * p2_y;
            let area = (b_x - seg.a_x) * (seg.c_y - seg.a_y) - (seg.c_x - seg.a_x) * (b_y - seg.a_y);
            let dx = seg.c_x - seg.a_x;
            let dy = seg.c_y - seg.a_y;
            let len_sq = dx * dx + dy * dy;
            if area * area > tolerance_sq * len_sq {
                if stack_count + 2 <= 64 {
                    stack[stack_count] = Segment::new(b_x, b_y, bt, seg.c_x, seg.c_y, seg.ct);
                    stack_count += 1;
                    stack[stack_count] = Segment::new(seg.a_x, seg.a_y, seg.at, b_x, b_y, bt);
                    stack_count += 1;
                } else {
                    output.push((seg.a_x, seg.a_y, seg.c_x, seg.c_y));
                }
            } else {
                output.push((seg.a_x, seg.a_y, seg.c_x, seg.c_y));
            }
        }
    }
}

fn insert_line(v_lines: &mut Vec<Line>, m_lines: &mut Vec<Line>, mut x0: f32, y0: f32, mut x1: f32, y1: f32, scale: f32, darkening: f32) {
    if y0 == y1 {
        return;
    }

    let dx = x1 - x0;
    let dy = y1 - y0;

    // Stem darkening: expand horizontal lines
    if dx != 0.0 {
        let sign = dx.signum();
        x0 -= darkening * sign;
        x1 += darkening * sign;
    }

    let dx = x1 - x0;
    let is_degen = dx == 0.0 && dy == 0.0;

    let line = Line {
        x0: x0 * scale,
        y0: y0 * scale,
        x1: x1 * scale,
        y1: y1 * scale,
        dx: dx * scale,
        dy: dy * scale,
        dx_sign: if dx != 0.0 { dx.signum() as i32 } else { 0 },
        dy_sign: if dy != 0.0 { dy.signum() as i32 } else { 0 },
        dt_dx: if dx != 0.0 { 1.0 / (dx * scale).abs() } else { f32::MAX },
        dt_dy: if dy != 0.0 { 1.0 / (dy * scale).abs() } else { f32::MAX },
        is_degen,
        abs_dx: (dx * scale).abs(),
        abs_dy: (dy * scale).abs(),
        dx_is_zero: dx == 0.0,
        dy_is_zero: dy == 0.0,
    };

    if x0 == x1 {
        v_lines.push(line);
    } else {
        m_lines.push(line);
    }
}

fn insert_complete_line(v_lines: &mut Vec<Line>, m_lines: &mut Vec<Line>, lines: &mut Vec<Line>, mut x0: f32, y0: f32, mut x1: f32, y1: f32, scale: f32, darkening: f32) {
    let dx = x1 - x0;
    let dy = y1 - y0;

    // Stem darkening: expand horizontal lines
    if dx != 0.0 {
        let sign = dx.signum();
        x0 -= darkening * sign;
        x1 += darkening * sign;
    }

    let dx = x1 - x0;
    let is_degen = dx == 0.0 && dy == 0.0;

    let line = Line {
        x0: x0 * scale,
        y0: y0 * scale,
        x1: x1 * scale,
        y1: y1 * scale,
        dx: dx * scale,
        dy: dy * scale,
        dx_sign: if dx != 0.0 { dx.signum() as i32 } else { 0 },
        dy_sign: if dy != 0.0 { dy.signum() as i32 } else { 0 },
        dt_dx: if dx != 0.0 { 1.0 / (dx * scale).abs() } else { f32::MAX },
        dt_dy: if dy != 0.0 { 1.0 / (dy * scale).abs() } else { f32::MAX },
        is_degen,
        abs_dx: (dx * scale).abs(),
        abs_dy: (dy * scale).abs(),
        dx_is_zero: dx == 0.0,
        dy_is_zero: dy == 0.0,
    };

    lines.push(line.clone());

    if x0 == x1 {
        v_lines.push(line);
    } else {
        m_lines.push(line);
    }
}
