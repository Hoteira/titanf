use crate::tables::glyf::Glyph;
use crate::vec;
use crate::Vec;

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

impl Glyph {
    pub(crate) fn build_lines<const COMPLETE: bool>(&self, _units_per_em: f32, scale: f32) -> GlyphLines {
        let tolerance_sq = (0.5 / scale).powi(2);
        let mut line_segments: Vec<(f32, f32, f32, f32)> = Vec::new();

        let mut x_min = f32::MAX;
        let mut x_max = f32::MIN;
        let mut y_min = f32::MAX;
        let mut y_max = f32::MIN;

        for contour in &self.points {
            let points = &contour.points;
            if points.is_empty() {
                continue;
            }

            let start_x = points[0].x;
            let start_y = points[0].y;
            let mut prev_x = start_x;
            let mut prev_y = start_y;

            x_min = x_min.min(start_x);
            x_max = x_max.max(start_x);
            y_min = y_min.min(start_y);
            y_max = y_max.max(start_y);

            let mut i = 1;
            while i < points.len() {
                let curr = &points[i];

                x_min = x_min.min(curr.x);
                x_max = x_max.max(curr.x);
                y_min = y_min.min(curr.y);
                y_max = y_max.max(curr.y);

                if curr.on_curve {
                    line_segments.push((prev_x, prev_y, curr.x, curr.y));
                    prev_x = curr.x;
                    prev_y = curr.y;
                    i += 1;
                } else {
                    let ctrl_x = curr.x;
                    let ctrl_y = curr.y;

                    let (next_x, next_y) = if i + 1 < points.len() && points[i + 1].on_curve {
                        i += 1;
                        (points[i].x, points[i].y)
                    } else {
                        let next_idx = (i + 1) % points.len();
                        let next = if next_idx == 0 { &points[0] } else { &points[next_idx] };
                        ((ctrl_x + next.x) / 2.0, (ctrl_y + next.y) / 2.0)
                    };

                    Self::flatten_quad(prev_x, prev_y, ctrl_x, ctrl_y, next_x, next_y, tolerance_sq, &mut line_segments);

                    prev_x = next_x;
                    prev_y = next_y;
                    i += 1;
                }
            }

            if prev_x != start_x || prev_y != start_y {
                line_segments.push((prev_x, prev_y, start_x, start_y));
            }
        }

        let mut v_lines = Vec::new();
        let mut m_lines = Vec::new();
        let mut lines = Vec::new();

        if COMPLETE {
            v_lines.reserve(self.points.len() * 3);
            m_lines.reserve(self.points.len() * 3);
            lines.reserve(self.points.len() * 4);

            for (x0, y0, x1, y1) in line_segments {
                insert_complete_line(&mut v_lines, &mut m_lines, &mut lines, x0, y0, x1, y1, scale);
            }

            for line in v_lines.iter_mut().chain(m_lines.iter_mut()).chain(lines.iter_mut()) {
                line.x0 -= x_min;
                line.y0 -= y_min;
                line.x1 -= x_min;
                line.y1 -= y_min;
            }
        } else {
            v_lines.reserve(self.points.len() * 3);
            m_lines.reserve(self.points.len() * 3);

            for (x0, y0, x1, y1) in line_segments {
                insert_line(&mut v_lines, &mut m_lines, x0, y0, x1, y1, scale);
            }

            for line in v_lines.iter_mut().chain(m_lines.iter_mut()) {
                line.x0 -= x_min;
                line.y0 -= y_min;
                line.x1 -= x_min;
                line.y1 -= y_min;
            }
        }

        let width = x_max - x_min;
        let height = y_max - y_min;

        for line in v_lines.iter_mut().chain(m_lines.iter_mut()) {
            if line.x0 < 0.0 { line.x0 = 0.0; }
            if line.x0 > width { line.x0 = width; }
            if line.x1 < 0.0 { line.x1 = 0.0; }
            if line.x1 > width { line.x1 = width; }
            if line.y0 < 0.0 { line.y0 = 0.0; }
            if line.y0 > height { line.y0 = height; }
            if line.y1 < 0.0 { line.y1 = 0.0; }
            if line.y1 > height { line.y1 = height; }

            // Recompute derived fields
            line.dx = line.x1 - line.x0;
            line.dy = line.y1 - line.y0;
            line.dx_is_zero = line.dx.abs() < 1e-6;
            line.dy_is_zero = line.dy.abs() < 1e-6;
            line.dx_sign = line.dx.signum() as i32;
            line.dy_sign = line.dy.signum() as i32;
            line.dt_dx = if !line.dx_is_zero { 1.0 / line.dx.abs() } else { f32::MAX };
            line.dt_dy = if !line.dy_is_zero { 1.0 / line.dy.abs() } else { f32::MAX };
            line.is_degen = line.dx_is_zero && line.dy_is_zero;
            line.abs_dx = line.dx.abs();
            line.abs_dy = line.dy.abs();
        }

        GlyphLines {
            v_lines,
            m_lines,
            lines,
            bounds: Bounds {
                _x: 0.0,
                _y: 0.0,
                width,
                height,
            },
        }
    }

    fn flatten_quad(
        p0_x: f32, p0_y: f32,
        p1_x: f32, p1_y: f32,
        p2_x: f32, p2_y: f32,
        tolerance_sq: f32,
        output: &mut Vec<(f32, f32, f32, f32)>
    ) {
        let mut stack = [Segment::default(); 64];
        let mut stack_count = 0;

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
                    // Fallback for deep recursion (very rare): just push the line
                    output.push((seg.a_x, seg.a_y, seg.c_x, seg.c_y));
                }
            } else {
                output.push((seg.a_x, seg.a_y, seg.c_x, seg.c_y));
            }
        }
    }
}

fn insert_line(v_lines: &mut Vec<Line>, m_lines: &mut Vec<Line>, x0: f32, y0: f32, x1: f32, y1: f32, scale: f32) {
    if y0 == y1 {
        return;
    }

    let dx = x1 - x0;
    let dy = y1 - y0;
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

fn insert_complete_line(v_lines: &mut Vec<Line>, m_lines: &mut Vec<Line>, lines: &mut Vec<Line>, x0: f32, y0: f32, x1: f32, y1: f32, scale: f32) {
    let dx = x1 - x0;
    let dy = y1 - y0;
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
