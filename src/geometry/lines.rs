use crate::Vec;
use crate::tables::glyf::Glyph;

#[allow(unused_imports)]
use crate::F32NoStd;
/// A resolved outline primitive in font units, produced once at font load.
/// TrueType's implied on-curve points and contour start rules are already
/// applied, so rasterization never re-walks the raw point lists.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Primitive {
    Line {
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
    },
    Quad {
        x0: f32,
        y0: f32,
        cx: f32,
        cy: f32,
        x1: f32,
        y1: f32,
    },
}

/// Per-axis traversal mode of a monotonic quad piece, packed two per byte
/// (x in the low nibble, y in the high nibble).
pub(crate) const QMODE_NONE: u8 = 0; // axis is constant: never crosses a boundary
pub(crate) const QMODE_LINEAR: u8 = 1; // |a| negligible: t advances linearly
pub(crate) const QMODE_QUAD: u8 = 2; // full quadratic root per crossing

/// Split one winding-corrected quadratic into monotonic pieces
/// and append them in raster-ready coefficient form.
/// Format: endpoints (q0), coefficients (q1), and reciprocals (q2).
fn push_monotonic_quads(
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    q0v: &mut Vec<crate::geometry::simd::f32x4>,
    q1v: &mut Vec<crate::geometry::simd::f32x4>,
    q2v: &mut Vec<crate::geometry::simd::f32x4>,
    qmodes: &mut Vec<u8>,
) {
    // Interior extrema (where the derivative changes sign) in each axis:
    // t* = (p0 - p1) / (p0 - 2 p1 + p2).
    let mut ts = [1.0f32; 3];
    let mut nts = 0;
    for (u0, u1, u2) in [(p0.0, p1.0, p2.0), (p0.1, p1.1, p2.1)] {
        let denom = u0 - 2.0 * u1 + u2;
        if denom != 0.0 {
            let t = (u0 - u1) / denom;
            if t > 1e-4 && t < 1.0 - 1e-4 {
                ts[nts] = t;
                nts += 1;
            }
        }
    }
    if nts == 2 && ts[0] > ts[1] {
        ts.swap(0, 1);
    }
    ts[nts] = 1.0;

    let mut push_piece = |a: (f32, f32), b: (f32, f32), c: (f32, f32)| {
        // (a, b, c) are the piece's control points p0, p1, p2.
        if a.1 == c.1 {
            // Monotonic in y with equal endpoints => constant y => zero
            // winding contribution.
            return;
        }
        let mut inv = [0.0f32; 2];
        let mut mode = [QMODE_NONE; 2];
        for (i, (u0, u1, u2)) in [(a.0, b.0, c.0), (a.1, b.1, c.1)].into_iter().enumerate() {
            let ca = u0 - 2.0 * u1 + u2;
            let cb = 2.0 * (u1 - u0);
            if u0 == u2 && ca == 0.0 {
                mode[i] = QMODE_NONE;
            } else if ca.abs() > 1e-6 * cb.abs() && ca != 0.0 {
                mode[i] = QMODE_QUAD;
                inv[i] = 1.0 / (2.0 * ca);
            } else if cb != 0.0 {
                mode[i] = QMODE_LINEAR;
                inv[i] = 1.0 / cb;
            } else {
                mode[i] = QMODE_NONE;
            }
        }
        let ax = a.0 - 2.0 * b.0 + c.0;
        let ay = a.1 - 2.0 * b.1 + c.1;
        let bx = 2.0 * (b.0 - a.0);
        let by = 2.0 * (b.1 - a.1);
        q0v.push(crate::geometry::simd::f32x4::new(a.0, a.1, c.0, c.1));
        q1v.push(crate::geometry::simd::f32x4::new(ax, ay, bx, by));
        q2v.push(crate::geometry::simd::f32x4::new(inv[0], inv[1], 0.0, 0.0));
        qmodes.push(mode[0] | (mode[1] << 4));
    };

    // Subdivide sequentially at the sorted extrema (de Casteljau).
    let mut s0 = p0;
    let mut s1 = p1;
    let s2 = p2;
    let mut t_prev = 0.0f32;
    for &t_split in &ts[..nts] {
        // Local parameter of the global split point within the remaining
        // curve segment.
        let t = (t_split - t_prev) / (1.0 - t_prev);
        let q0 = (s0.0 + (s1.0 - s0.0) * t, s0.1 + (s1.1 - s0.1) * t);
        let q1 = (s1.0 + (s2.0 - s1.0) * t, s1.1 + (s2.1 - s1.1) * t);
        let mid = (q0.0 + (q1.0 - q0.0) * t, q0.1 + (q1.1 - q0.1) * t);
        push_piece(s0, q0, mid);
        s0 = mid;
        s1 = q1;
        t_prev = t_split;
    }
    push_piece(s0, s1, s2);
}

impl Glyph {
    /// One-time outline resolution, run at font load.
    pub(crate) fn preprocess(&mut self, units_per_em: f32) {
        let mut x_min = f32::MAX;
        let mut x_max = f32::MIN;
        let mut y_min = f32::MAX;
        let mut y_max = f32::MIN;

        let point_count: usize = self.points.iter().map(|c| c.points.len()).sum();
        let mut prims: Vec<Primitive> = Vec::with_capacity(point_count);

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
                            prims.push(Primitive::Quad {
                                x0: current_pos.0,
                                y0: current_pos.1,
                                cx: offcurve.0,
                                cy: offcurve.1,
                                x1: x,
                                y1: y,
                            });
                        } else {
                            prims.push(Primitive::Line {
                                x0: current_pos.0,
                                y0: current_pos.1,
                                x1: x,
                                y1: y,
                            });
                        }
                        current_pos = (x, y);
                        i += 1;
                    } else {
                        let ctrl_x = x;
                        let ctrl_y = y;
                        let next_idx = (i + 1) % points.len();
                        let next = &points[next_idx];
                        let (next_x, next_y) = if next.on_curve {
                            if i + 1 < points.len() {
                                i += 1;
                            }
                            (next.x, next.y)
                        } else {
                            ((ctrl_x + next.x) / 2.0, (ctrl_y + next.y) / 2.0)
                        };
                        prims.push(Primitive::Quad {
                            x0: current_pos.0,
                            y0: current_pos.1,
                            cx: ctrl_x,
                            cy: ctrl_y,
                            x1: next_x,
                            y1: next_y,
                        });
                        current_pos = (next_x, next_y);
                        i += 1;
                    }
                }
            }

            if let Some(start) = first_on_curve {
                let dx = current_pos.0 - start.0;
                let dy = current_pos.1 - start.1;
                let dist_sq = dx * dx + dy * dy;

                if let Some(off1) = first_off_curve {
                    prims.push(Primitive::Quad {
                        x0: current_pos.0,
                        y0: current_pos.1,
                        cx: off1.0,
                        cy: off1.1,
                        x1: start.0,
                        y1: start.1,
                    });
                } else if dist_sq > 0.00001 {
                    prims.push(Primitive::Line {
                        x0: current_pos.0,
                        y0: current_pos.1,
                        x1: start.0,
                        y1: start.1,
                    });
                }
            }
        }

        if x_min == f32::MAX {
            self.has_points = false;
            return;
        }

        self.has_points = true;
        self.min_x = x_min;
        self.max_x = x_max;
        self.min_y = y_min;
        self.max_y = y_max;

        // Winding orientation via shoelace over primitive chords. Only the
        // sign matters, and a global flip is invisible anyway (coverage is
        // mapped through |acc|), so chord approximation is safe.
        let mut area = 0.0;
        for prim in &prims {
            let (x0, y0, x1, y1) = match *prim {
                Primitive::Line { x0, y0, x1, y1 } => (x0, y0, x1, y1),
                Primitive::Quad { x0, y0, x1, y1, .. } => (x0, y0, x1, y1),
            };
            area += (y1 - y0) * (x1 + x0);
        }
        self.reverse = area > 0.0;

        // Bake raster-ready geometry: straight edges as line lists, curves
        // as monotonic quadratic pieces walked directly by the rasterizer.
        // No flattening happens anywhere - curves are exact at every size.
        let _ = units_per_em;
        let reverse = self.reverse;

        let mut pre_v_lines = Vec::new();
        let mut pre_m_lines = Vec::new();
        let mut pre_m_params = Vec::new();
        let mut quad_q0 = Vec::new();
        let mut quad_q1 = Vec::new();
        let mut quad_q2 = Vec::new();
        let mut quad_modes = Vec::new();

        for prim in &prims {
            match *prim {
                Primitive::Line { mut x0, mut y0, mut x1, mut y1 } => {
                    if y0 == y1 {
                        continue;
                    }
                    if reverse {
                        core::mem::swap(&mut x0, &mut x1);
                        core::mem::swap(&mut y0, &mut y1);
                    }
                    if x0 == x1 {
                        pre_v_lines.push(crate::geometry::simd::f32x4::new(x0, y0, x1, y1));
                    } else {
                        let dx = x1 - x0;
                        let dy = y1 - y0;
                        let tdx = 1.0 / dx.abs();
                        let tdy = 1.0 / dy.abs();
                        pre_m_lines.push(crate::geometry::simd::f32x4::new(x0, y0, x1, y1));
                        pre_m_params.push(crate::geometry::simd::f32x4::new(tdx, tdy, dx, dy));
                    }
                }
                Primitive::Quad { x0, y0, cx, cy, x1, y1 } => {
                    // Reversing traversal of a quad = swapping its endpoints
                    // (t -> 1-t leaves the control point in place).
                    let (p0, p2) = if reverse {
                        ((x1, y1), (x0, y0))
                    } else {
                        ((x0, y0), (x1, y1))
                    };
                    push_monotonic_quads(
                        p0,
                        (cx, cy),
                        p2,
                        &mut quad_q0,
                        &mut quad_q1,
                        &mut quad_q2,
                        &mut quad_modes,
                    );
                }
            }
        }

        pre_v_lines.shrink_to_fit();
        pre_m_lines.shrink_to_fit();
        pre_m_params.shrink_to_fit();
        quad_q0.shrink_to_fit();
        quad_q1.shrink_to_fit();
        quad_q2.shrink_to_fit();
        quad_modes.shrink_to_fit();
        self.pre_v_lines = pre_v_lines;
        self.pre_m_lines = pre_m_lines;
        self.pre_m_params = pre_m_params;
        self.quad_q0 = quad_q0;
        self.quad_q1 = quad_q1;
        self.quad_q2 = quad_q2;
        self.quad_modes = quad_modes;

        // Raw point lists are no longer needed; free the memory.
        self.points = Vec::new();
    }
}
