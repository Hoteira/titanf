use crate::tables::glyf::Glyph;
use crate::Vec;
use crate::vec;

use crate::preprocess::points::Point;

#[derive(Debug)]
pub struct Line {
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,

    pub dx: f32,
    pub dy: f32,
}

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

const SIZE: f32 = 40.0;

struct Segment {
    a_x: f32,
    a_y: f32,
    at: f32,
    c_x: f32,
    c_y: f32,
    ct: f32,
}

impl Segment {
    fn new(a_x: f32, a_y: f32, at: f32, c_x: f32, c_y: f32, ct: f32) -> Self {
        Segment { a_x, a_y, at, c_x, c_y, ct }
    }
}

impl Glyph {
    pub(crate) fn build_lines(&mut self, units_per_em: f32) {
        let max_area = 3.0 * 2.0 * (units_per_em / SIZE);
        let mut line_segments: Vec<(f32, f32, f32, f32)> = Vec::new();

        let mut x_min = f32::MAX;
        let mut x_max = f32::MIN;
        let mut y_min = f32::MAX;
        let mut y_max = f32::MIN;
        let mut tot_area = 0.0;

        for contour in &self.points {
            let points = &contour.points;
            if points.is_empty() {
                continue;
            }

            tot_area += contour_area(points);

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

                    Self::flatten_quad(prev_x, prev_y, ctrl_x, ctrl_y, next_x, next_y, max_area, &mut line_segments);

                    prev_x = next_x;
                    prev_y = next_y;
                    i += 1;
                }
            }

            if prev_x != start_x || prev_y != start_y {
                line_segments.push((prev_x, prev_y, start_x, start_y));
            }
        }

        for (x0, y0, x1, y1) in line_segments {
            self.insert_line(x0, y0, x1, y1);
        }

        for line in self.v_lines.iter_mut().chain(self.m_lines.iter_mut()) {
            line.x0 -= x_min;
            line.y0 -= y_min;
            line.x1 -= x_min;
            line.y1 -= y_min;
        }

        self.bounds = Bounds {
            _x: 0.0,
            _y: 0.0,
            width: x_max - x_min,
            height: y_max - y_min,
        };

        self.points.clear();
    }

    fn flatten_quad(p0_x: f32, p0_y: f32, p1_x: f32, p1_y: f32, p2_x: f32, p2_y: f32, max_area: f32, output: &mut Vec<(f32, f32, f32, f32)>) {
        let mut stack = vec![Segment::new(p0_x, p0_y, 0.0, p2_x, p2_y, 1.0)];

        while let Some(seg) = stack.pop() {
            let bt = (seg.at + seg.ct) * 0.5;

            let tm = 1.0 - bt;
            let a = tm * tm;
            let b = 2.0 * tm * bt;
            let c = bt * bt;
            let b_x = a * p0_x + b * p1_x + c * p2_x;
            let b_y = a * p0_y + b * p1_y + c * p2_y;

            let area = (b_x - seg.a_x) * (seg.c_y - seg.a_y) - (seg.c_x - seg.a_x) * (b_y - seg.a_y);

            if area.abs() > max_area {
                stack.push(Segment::new(seg.a_x, seg.a_y, seg.at, b_x, b_y, bt));
                stack.push(Segment::new(b_x, b_y, bt, seg.c_x, seg.c_y, seg.ct));
            } else {
                output.push((seg.a_x, seg.a_y, seg.c_x, seg.c_y));
            }
        }
    }

    pub(crate) fn insert_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {

        if y0 == y1 {
            return;
        }

        let dx = x1 - x0;
        let dy = y1 - y0;

        let line = Line { x0, y0, x1, y1, dx, dy };

        if x0 == x1 {
            self.v_lines.push(line);
        } else {
            self.m_lines.push(line);
        }
    }
}

fn contour_area(points: &[Point]) -> f32 {
    let mut area = 0.0;
    for i in 0..points.len() {
        let j = (i + 1) % points.len();
        area += points[i].x * points[j].y;
        area -= points[j].x * points[i].y;
    }
    area * 0.5
}