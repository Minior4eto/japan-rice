//! A tiny software canvas. Scenes paint square "pixels"; the renderer turns two
//! pixel rows into one terminal row using half blocks.

use crate::util::{rgb, Color};

pub struct Pixels {
    pub w: usize,
    pub h: usize,
    data: Vec<Color>,
}

impl Pixels {
    pub fn new(w: usize, h: usize) -> Self {
        Pixels {
            w,
            h,
            data: vec![rgb(0.0, 0.0, 0.0); w * h],
        }
    }

    pub fn at(&self, x: usize, y: usize) -> Color {
        self.data[y * self.w + x]
    }

    /// Apply a function to every pixel.
    pub fn map(&mut self, f: impl Fn(Color) -> Color) {
        for c in &mut self.data {
            *c = f(*c);
        }
    }

    /// Vertical gradient. Stops are (position 0..1 from top, colour).
    pub fn gradient(&mut self, stops: &[(f32, Color)]) {
        for y in 0..self.h {
            let f = (y as f32 + 0.5) / self.h as f32;
            let mut col = stops[stops.len() - 1].1;
            for pair in stops.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                if f <= b.0 {
                    col = a.1.mix(b.1, (f - a.0) / (b.0 - a.0).max(1e-4));
                    break;
                }
            }
            if f < stops[0].0 {
                col = stops[0].1;
            }
            for x in 0..self.w {
                self.data[y * self.w + x] = col;
            }
        }
    }

    pub fn blend(&mut self, x: i32, y: i32, c: Color, a: f32) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return;
        }
        let i = y as usize * self.w + x as usize;
        self.data[i] = self.data[i].mix(c, a);
    }

    /// Additive light, used for glows.
    pub fn add(&mut self, x: i32, y: i32, c: Color, k: f32) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return;
        }
        let i = y as usize * self.w + x as usize;
        let d = self.data[i];
        self.data[i] = rgb(
            (d.r + c.r * k).min(255.0),
            (d.g + c.g * k).min(255.0),
            (d.b + c.b * k).min(255.0),
        );
    }

    pub fn rect(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, c: Color, a: f32) {
        let xa = (x0 - 0.5).ceil() as i32;
        let xb = (x1 - 0.5).floor() as i32;
        let ya = (y0 - 0.5).ceil() as i32;
        let yb = (y1 - 0.5).floor() as i32;
        for y in ya..=yb {
            for x in xa..=xb {
                self.blend(x, y, c, a);
            }
        }
    }

    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, c: Color, a: f32) {
        let (rx, ry) = (rx.max(0.4), ry.max(0.4));
        for y in (cy - ry).floor() as i32..=(cy + ry).ceil() as i32 {
            for x in (cx - rx).floor() as i32..=(cx + rx).ceil() as i32 {
                let dx = (x as f32 + 0.5 - cx) / rx;
                let dy = (y as f32 + 0.5 - cy) / ry;
                if dx * dx + dy * dy <= 1.0 {
                    self.blend(x, y, c, a);
                }
            }
        }
    }

    pub fn circle(&mut self, cx: f32, cy: f32, r: f32, c: Color, a: f32) {
        self.ellipse(cx, cy, r, r, c, a);
    }

    /// Soft additive glow with quadratic falloff.
    pub fn glow(&mut self, cx: f32, cy: f32, r: f32, c: Color, strength: f32) {
        for y in (cy - r).floor() as i32..=(cy + r).ceil() as i32 {
            for x in (cx - r).floor() as i32..=(cx + r).ceil() as i32 {
                let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt() / r;
                if d < 1.0 {
                    let f = (1.0 - d) * (1.0 - d) * strength;
                    self.add(x, y, c, f);
                }
            }
        }
    }

    /// Filled polygon (even-odd scanline fill).
    pub fn poly(&mut self, pts: &[(f32, f32)], c: Color, a: f32) {
        if pts.len() < 3 {
            return;
        }
        let ymin = pts.iter().map(|p| p.1).fold(f32::MAX, f32::min);
        let ymax = pts.iter().map(|p| p.1).fold(f32::MIN, f32::max);
        let y0 = (ymin.floor() as i32).max(0);
        let y1 = (ymax.ceil() as i32).min(self.h as i32 - 1);
        let n = pts.len();
        let mut xs: Vec<f32> = Vec::with_capacity(8);
        for y in y0..=y1 {
            let yc = y as f32 + 0.5;
            xs.clear();
            for i in 0..n {
                let (ax, ay) = pts[i];
                let (bx, by) = pts[(i + 1) % n];
                if (ay <= yc && by > yc) || (by <= yc && ay > yc) {
                    let t = (yc - ay) / (by - ay);
                    xs.push(ax + t * (bx - ax));
                }
            }
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            for pair in xs.chunks(2) {
                if pair.len() == 2 {
                    let xa = (pair[0] - 0.5).ceil() as i32;
                    let xb = (pair[1] - 0.5).floor() as i32;
                    for x in xa..=xb {
                        self.blend(x, y, c, a);
                    }
                }
            }
        }
    }

    /// Thick line drawn as a quad.
    #[allow(clippy::too_many_arguments)]
    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, th: f32, c: Color, a: f32) {
        let th = th.max(1.1);
        let (dx, dy) = (x1 - x0, y1 - y0);
        let len = (dx * dx + dy * dy).sqrt().max(1e-4);
        let (nx, ny) = (-dy / len * th * 0.5, dx / len * th * 0.5);
        self.poly(
            &[
                (x0 + nx, y0 + ny),
                (x1 + nx, y1 + ny),
                (x1 - nx, y1 - ny),
                (x0 - nx, y0 - ny),
            ],
            c,
            a,
        );
    }
}
