#[cfg(not(feature = "std"))]
use crate::F32NoStd;

use crate::tables::glyf::{
    CompositeComponent,
    Glyph,
    ProtoGlyph,
    ARGS_ARE_XY_VALUES,
    WE_HAVE_AN_X_AND_Y_SCALE,
    WE_HAVE_A_SCALE,
    WE_HAVE_A_TWO_BY_TWO
};

use crate::Vec;
use crate::font::TrueTypeFont;

#[derive(Debug, Copy, Clone)]
pub(crate) struct Point {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) on_curve: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct Contour {
    pub(crate) points: Vec<Point>,
}

impl Contour {
    pub(crate) fn new(size: usize) -> Self {
        Contour { points: Vec::with_capacity(size) }
    }
}

impl TrueTypeFont {
    pub(crate) fn load_points(&self, glyph: &mut ProtoGlyph, font: &TrueTypeFont, font_bytes: &[u8]) -> Glyph {
        match glyph {
            ProtoGlyph::Simple(g) => {
                let num_points = g.end_pts_of_contours.last().map(|&e| (e + 1) as usize).unwrap_or(0);
                let expanded_flags = expand_flags(&g.flags, num_points);

                g.points.reserve(g.end_pts_of_contours.len());

                let mut contour_start = 0;
                for i in 0..g.end_pts_of_contours.len() {
                    let contour_size = if i == 0 {
                        g.end_pts_of_contours[i] as usize + 1
                    } else {
                        (g.end_pts_of_contours[i] - g.end_pts_of_contours[i - 1]) as usize
                    };

                    let mut contour = Contour::new(contour_size);

                    for j in contour_start..=g.end_pts_of_contours[i] as usize {
                        contour.points.push(Point {
                            x: (g.x_coordinates[j] - g.x_min) as f32,
                            y: (g.y_max - g.y_coordinates[j]) as f32,
                            on_curve: expanded_flags[j],
                        });
                    }

                    contour_start = g.end_pts_of_contours[i] as usize + 1;
                    g.points.push(contour);
                }
            }

            ProtoGlyph::Composite(g) => {
                load_from_parent(&mut g.points, &g.components, font, font_bytes);
            }

            ProtoGlyph::Empty => {}
        }

        let mut glyph = glyph.finalize();
        fix_degenerates(&mut glyph.points);
        glyph.build_lines(self.head.units_per_em as f32);

        glyph
    }
}

pub(crate) fn load_from_parent(master: &mut Vec<Contour>, comps: &Vec<CompositeComponent>, font: &TrueTypeFont, font_bytes: &[u8]) {
    for component in comps.iter() {
        let real_glyph = &mut font.get_glyph(font_bytes, component.glyph_index as u32);

        match real_glyph {
            ProtoGlyph::Simple(g) => {
                let num_points = g.end_pts_of_contours.last().map(|&e| (e + 1) as usize).unwrap_or(0);
                let expanded_flags = expand_flags(&g.flags, num_points);

                g.points.reserve(g.end_pts_of_contours.len());

                let mut contour_start = 0;
                for i in 0..g.end_pts_of_contours.len() {
                    let contour_size = if i == 0 {
                        g.end_pts_of_contours[i] as usize + 1
                    } else {
                        (g.end_pts_of_contours[i] - g.end_pts_of_contours[i - 1]) as usize
                    };

                    let mut contour = Contour::new(contour_size + 1);

                    for j in contour_start..=g.end_pts_of_contours[i] as usize {

                        contour.points.push(Point {
                            x: (g.x_coordinates[j] - g.x_min) as f32,
                            y: (g.y_max - g.y_coordinates[j]) as f32,
                            on_curve: expanded_flags[j],
                        });
                    }

                    if !contour.points.is_empty() {
                        let first_point = contour.points[0];
                        contour.points.push(first_point);
                    }

                    transform_points(&mut contour.points, component);

                    contour_start = g.end_pts_of_contours[i] as usize + 1;
                    master.push(contour);
                }
            }

            ProtoGlyph::Composite(g) => {
                load_from_parent(master, &g.components, font, font_bytes);
            }

            ProtoGlyph::Empty => {}
        }
    }
}



fn transform_points(points: &mut [Point], component: &CompositeComponent) {
    let (x_scale, y_scale, scale_01, scale_10) = if component.flags & WE_HAVE_A_TWO_BY_TWO != 0 {
        (
            component.x_scale.unwrap_or(1.0),
            component.y_scale.unwrap_or(1.0),
            component.scale_01.unwrap_or(0.0),
            component.scale_10.unwrap_or(0.0),
        )
    } else if component.flags & WE_HAVE_AN_X_AND_Y_SCALE != 0 {
        (component.x_scale.unwrap_or(1.0), component.y_scale.unwrap_or(1.0), 0.0, 0.0)
    } else if component.flags & WE_HAVE_A_SCALE != 0 {
        let s = component.scale.unwrap_or(1.0);
        (s, s, 0.0, 0.0)
    } else {
        (1.0, 1.0, 0.0, 0.0)
    };

    for p in points.iter_mut() {
        let old_x = p.x;
        let old_y = p.y;
        let nx = old_x * x_scale + old_y * scale_10;
        let ny = old_x * scale_01 + old_y * y_scale;
        p.x = nx.round();
        p.y = ny.round();
    }

    if component.flags & ARGS_ARE_XY_VALUES != 0 {
        let dx = component.argument1 as f32;
        let dy = component.argument2 as f32;
        for p in points.iter_mut() {
            p.x = p.x + dx;
            p.y = p.y + dy;
        }
    }
}

fn expand_flags(raw_flags: &[u8], num_points: usize) -> Vec<bool> {
    let mut expanded = Vec::with_capacity(num_points);
    let mut i = 0;

    while expanded.len() < num_points && i < raw_flags.len() {
        let flag = raw_flags[i];
        expanded.push((flag & 0x01) != 0);
        i += 1;

        if flag & 0x08 != 0 {
            if i >= raw_flags.len() {
                break;
            }
            let repeat = raw_flags[i] as usize;
            i += 1;
            for _ in 0..repeat {
                if expanded.len() >= num_points {
                    break;
                }
                expanded.push((flag & 0x01) != 0);
            }
        }
    }

    expanded
}

pub fn fix_degenerates(contours: &mut Vec<Contour>) {
    for contour in contours.iter_mut() {
        let n = contour.points.len();
        if n < 2 {
            continue;
        }

        let mut i = 0;
        while i < n {
            let next = (i + 1) % n;
            let p0 = contour.points[i];
            let p1 = contour.points[next];

            if !p0.on_curve && !p1.on_curve {
                let dx = p1.x - p0.x;
                let dy = p1.y - p0.y;

                //Arbitrary fine-tuning, gotta fix it tho
                if dy.abs() < 1e-6 && dx.abs() > 200.0 {
                    contour.points[i].on_curve = true;
                    contour.points[next].on_curve = true;

                    i += 2;
                    continue;
                }
            }

            i += 1;
        }
    }
}
