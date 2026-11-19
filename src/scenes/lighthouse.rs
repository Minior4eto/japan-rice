//! A lighthouse on a cliff at night. Twin beams sweep through the sea haze and
//! flash when they face you; the moon lays a glittering path on the swell;
//! waves burst against the rocks; a ship crosses the horizon; stars twinkle
//! along the Milky Way and the occasional shooting star falls.

use super::{mist, Scene};
use crate::canvas::Pixels;
use crate::theme::Theme;
use crate::util::{fbm, hash, lerp, noise, noise2, rgb, smoothstep, Color};
use std::f32::consts::{PI, TAU};

pub struct Lighthouse;

const BEAM: Color = rgb(255.0, 236.0, 186.0);
const MOON: Color = rgb(236.0, 238.0, 250.0);
const FOAM: Color = rgb(220.0, 232.0, 246.0);

struct Geo {
    w: f32,
    h: f32,
    s: f32,
    hz: f32,
}

impl Geo {
    /// x of the cliff's sea face at row y.
    fn face(&self, y: f32) -> f32 {
        let top = self.h * 0.5;
        let f = ((y - top) / (self.h - top)).clamp(0.0, 1.0);
        self.w * 0.70 - f.powf(0.8) * self.w * 0.13
            + 1.8 * self.s * (noise(y * 0.28 / self.s) - 0.5)
            + 1.2 * self.s * (noise(y * 0.9 / self.s + 3.0) - 0.5)
    }

    /// y of the plateau surface at column x.
    fn surf(&self, x: f32) -> f32 {
        self.h * 0.5 - (x - self.w * 0.7) * 0.03 + 0.9 * self.s * (noise(x * 0.12 / self.s) - 0.5)
    }

    fn in_cliff(&self, x: f32, y: f32) -> bool {
        y >= self.surf(x) && x >= self.face(y)
    }
}

/// Twin lamps turning together. Returns (angle of first beam).
fn beam_angle(t: f32) -> f32 {
    t * TAU / 9.0
}

impl Scene for Lighthouse {
    fn name(&self) -> &'static str {
        "lighthouse"
    }

    fn blurb(&self) -> &'static str {
        "Lighthouse on a cliff at night: sweeping beams, moonlit sea, crashing waves"
    }

    fn draw(&self, p: &mut Pixels, t: f32, _theme: Option<&Theme>) {
        let (w, h) = (p.w as f32, p.h as f32);
        let s = h / 80.0;
        let g = Geo {
            w,
            h,
            s,
            hz: h * 0.56,
        };
        let hz = g.hz;
        let (mx, my, mr) = (w * 0.22, h * 0.17, 5.5 * s);

        // ---------- sky ----------
        p.gradient(&[
            (0.0, rgb(3.0, 5.0, 16.0)),
            (0.32, rgb(10.0, 16.0, 40.0)),
            (0.56, rgb(34.0, 48.0, 84.0)),
            (1.0, rgb(34.0, 48.0, 84.0)),
        ]);
        // Milky Way: a soft diagonal band
        let (ax, ay, bx, by) = (w * 0.45, -h * 0.05, w * 1.05, hz);
        let (dx, dy) = (bx - ax, by - ay);
        let dl = (dx * dx + dy * dy).sqrt();
        let band = |x: f32, y: f32| {
            let d = ((x - ax) * dy - (y - ay) * dx).abs() / dl;
            (-(d / (h * 0.11)).powi(2)).exp()
        };
        for y in 0..(hz as usize).min(p.h) {
            for x in 0..p.w {
                let (fx, fy) = (x as f32, y as f32);
                let b = band(fx, fy);
                if b > 0.02 {
                    let n = noise2(fx * 0.05 / s, fy * 0.05 / s) * 0.6
                        + noise2(fx * 0.14 / s + 7.0, fy * 0.14 / s) * 0.4;
                    let dust = smoothstep(0.35, 0.75, n) * b;
                    p.blend(x as i32, y as i32, rgb(86.0, 92.0, 140.0), dust * 0.45);
                    // dark dust lane
                    let lane =
                        smoothstep(0.62, 0.8, noise2(fx * 0.03 / s + 2.0, fy * 0.09 / s)) * b;
                    p.blend(x as i32, y as i32, rgb(6.0, 8.0, 20.0), lane * 0.5);
                }
            }
        }
        // stars, denser in the band
        let n_st = (w * hz / 18.0) as u32;
        for i in 0..n_st {
            let x = hash(i * 4 + 1) * w;
            let y = hash(i * 4 + 2) * hz;
            if hash(i * 4 + 3) > 0.22 + 0.78 * band(x, y) {
                continue;
            }
            let tw = 0.55 + 0.45 * (t * (1.5 + 2.5 * hash(i * 4 + 4)) + i as f32).sin();
            let fade = 1.0 - smoothstep(hz * 0.6, hz, y);
            let bright = hash(i * 4 + 5);
            let c = if bright > 0.97 {
                rgb(255.0, 220.0, 190.0)
            } else if bright > 0.94 {
                rgb(190.0, 210.0, 255.0)
            } else {
                rgb(240.0, 242.0, 255.0)
            };
            p.blend(x as i32, y as i32, c, (0.25 + 0.6 * bright) * tw * fade);
            if bright > 0.985 {
                for (ox, oy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    p.blend(x as i32 + ox, y as i32 + oy, c, 0.35 * tw * fade);
                }
            }
        }
        // shooting star
        let period = 6.5;
        let k = (t / period).floor() as i32 as u32;
        let ph = t - (t / period).floor() * period;
        if hash(k * 5 + 77) > 0.35 && ph < 0.8 {
            let sx = hash(k * 5 + 78) * w * 0.7;
            let sy = hash(k * 5 + 79) * hz * 0.4;
            let (vx, vy) = (0.9 * w, 0.32 * w);
            let (hx, hy) = (sx + vx * ph, sy + vy * ph);
            let len = 16.0 * s;
            let n = (len as i32).max(4);
            let fade = 1.0 - ph / 0.8;
            for j in 0..n {
                let f = j as f32 / n as f32;
                let px = hx - vx / (vx * vx + vy * vy).sqrt() * len * f;
                let py = hy - vy / (vx * vx + vy * vy).sqrt() * len * f;
                if py < hz {
                    p.add(
                        px as i32,
                        py as i32,
                        rgb(255.0, 250.0, 235.0),
                        (1.0 - f) * fade,
                    );
                }
            }
        }

        // moon with halo and maria
        p.glow(mx, my, mr * 7.0, rgb(120.0, 140.0, 200.0), 0.35);
        p.glow(mx, my, mr * 2.4, rgb(200.0, 210.0, 240.0), 0.35);
        p.circle(mx, my, mr, MOON, 1.0);
        for (ox, oy, r) in [
            (-0.3, -0.25, 0.3),
            (0.3, 0.2, 0.22),
            (0.05, 0.45, 0.16),
            (-0.35, 0.3, 0.12),
        ] {
            p.circle(
                mx + ox * mr,
                my + oy * mr,
                r * mr,
                rgb(196.0, 202.0, 222.0),
                0.75,
            );
        }
        // thin clouds drifting past, lit near the moon
        for y in 0..((hz - 3.0 * s) as usize).min(p.h) {
            let fy = y as f32;
            let lat = smoothstep(0.0, 0.12, fy / h) * (1.0 - smoothstep(0.35, 0.5, fy / h));
            if lat <= 0.0 {
                continue;
            }
            for x in 0..p.w {
                let fx = x as f32;
                let n = noise2(fx * 0.008 / s + t * 0.012, fy * 0.06 / s) * 0.7
                    + noise2(fx * 0.025 / s + t * 0.02, fy * 0.12 / s + 3.0) * 0.3;
                let c = smoothstep(0.52, 0.82, n) * lat;
                if c > 0.0 {
                    let dm = ((fx - mx).powi(2) + (fy - my).powi(2)).sqrt() / (h * 0.45);
                    let lit = (1.0 - dm).max(0.0).powi(2);
                    let col = rgb(26.0, 32.0, 58.0).mix(rgb(170.0, 178.0, 210.0), lit);
                    p.blend(x as i32, y as i32, col, c * 0.85);
                }
            }
        }

        // ---------- sea ----------
        let glitter_slot = (t * 9.0).floor() as i32 as u32;
        for y in (hz as usize).min(p.h)..p.h {
            let fy = y as f32;
            let d = ((fy - hz) / (h - hz)).max(0.001);
            let z = 1.0 / (d + 0.03);
            let amp = d.powf(0.45);
            let base = rgb(40.0, 54.0, 88.0).mix(rgb(6.0, 12.0, 28.0), d.powf(0.35));
            let spread = w * (0.02 + 0.13 * d);
            for x in 0..p.w {
                let fx = x as f32;
                let wx = (fx - w * 0.5) / h * z;
                let v = 0.5 * (wx * 1.4 + z * 2.1 - t * 1.0).sin()
                    + 0.3 * (-wx * 1.1 + z * 3.3 - t * 1.5 + 1.3).sin()
                    + 0.55 * (noise2(wx * 2.4 + t * 0.15, z * 5.5 - t * 0.8) - 0.5)
                    + 0.3 * (noise2(wx * 6.0, z * 14.0 - t * 1.6) - 0.5);
                let mut c = base.scale(1.0 + 0.45 * v * amp);
                // moon path
                let col = (-((fx - mx) / spread).powi(2)).exp();
                let spark = smoothstep(0.15, 0.85, v) * col;
                c = c.mix(rgb(150.0, 166.0, 210.0), col * 0.25 * (1.0 - d * 0.5));
                c = c.mix(MOON, spark * 0.85);
                if col > 0.15 && v > 0.25 {
                    let hsh = hash(
                        (x as u32).wrapping_mul(73_856_093)
                            ^ (y as u32).wrapping_mul(19_349_663)
                            ^ glitter_slot.wrapping_mul(83_492_791),
                    );
                    if hsh > 0.86 {
                        c = c.mix(rgb(255.0, 255.0, 255.0), col);
                    }
                }
                // scattered whitecaps closer in
                if d > 0.35 && v > 0.62 && noise2(wx * 3.0 + 40.0, z * 8.0 - t * 0.9) > 0.7 {
                    c = c.mix(FOAM.scale(0.6), 0.5 * (d - 0.35));
                }
                p.blend(x as i32, y as i32, c, 1.0);
            }
        }
        // a bright line where moonlit water meets the sky
        p.rect(0.0, hz - 0.5, w, hz + 0.5, rgb(90.0, 106.0, 150.0), 0.5);

        // ship crossing the horizon
        let shx = ((0.05 + t * 0.006).rem_euclid(1.3) - 0.15) * w;
        let shy = hz - 0.2 * s;
        let hull = rgb(8.0, 10.0, 20.0);
        p.poly(
            &[
                (shx - 5.0 * s, shy - 1.0 * s),
                (shx + 5.5 * s, shy - 1.0 * s),
                (shx + 4.6 * s, shy + 0.2 * s),
                (shx - 4.4 * s, shy + 0.2 * s),
            ],
            hull,
            1.0,
        );
        p.rect(
            shx - 2.5 * s,
            shy - 2.4 * s,
            shx + 1.5 * s,
            shy - 1.0 * s,
            hull,
            1.0,
        );
        p.rect(
            shx - 0.6 * s,
            shy - 4.0 * s,
            shx - 0.1 * s,
            shy - 2.4 * s,
            hull,
            1.0,
        );
        let blink = if (t * 1.3).sin() > 0.0 { 1.0 } else { 0.3 };
        p.add(
            (shx - 0.35 * s) as i32,
            (shy - 4.0 * s) as i32,
            rgb(255.0, 250.0, 230.0),
            0.9 * blink,
        );
        p.add(
            (shx - 4.0 * s) as i32,
            (shy - 1.3 * s) as i32,
            rgb(255.0, 60.0, 50.0),
            0.9,
        );
        p.add(
            (shx - 1.5 * s) as i32,
            (shy - 1.9 * s) as i32,
            rgb(255.0, 210.0, 140.0),
            0.7,
        );
        p.add(
            shx as i32,
            (shy - 1.9 * s) as i32,
            rgb(255.0, 210.0, 140.0),
            0.7,
        );
        for j in 1..(5.0 * s) as i32 {
            if (j as f32 * 0.7 + t * 4.0).sin() > 0.0 {
                p.blend(
                    (shx - 0.35 * s) as i32,
                    (hz + j as f32) as i32,
                    rgb(255.0, 240.0, 200.0),
                    0.35,
                );
            }
        }
        mist(p, t, 0.5, 0.6, rgb(40.0, 54.0, 90.0), 0.4);

        // ---------- cliff ----------
        for y in 0..p.h {
            let fy = y as f32;
            for x in 0..p.w {
                let fx = x as f32;
                if !g.in_cliff(fx, fy) {
                    continue;
                }
                let edge = fx - g.face(fy);
                let top = fy - g.surf(fx);
                let strata = noise2(fx * 0.06 / s, fy * 0.4 / s);
                let crack = noise2(fx * 0.7 / s, fy * 0.06 / s);
                let ledge =
                    ((fy / (2.2 * s) + noise(fx * 0.05 / s) * 3.0).fract() < 0.18) as i32 as f32;
                let mut c = rgb(20.0, 22.0, 34.0).scale(0.8 + 0.4 * strata - 0.15 * ledge);
                if crack > 0.8 {
                    c = c.scale(0.55);
                }
                // moonlight on the seaward face
                let lit = (-edge / (5.0 * s)).exp() * (0.6 + 0.4 * strata);
                c = c.mix(rgb(70.0, 80.0, 116.0), lit * 0.8);
                // grass on the plateau
                if top < 1.6 * s {
                    let gr = rgb(24.0, 36.0, 40.0)
                        .mix(rgb(60.0, 76.0, 90.0), (1.0 - top / (1.6 * s)) * 0.6);
                    c = gr;
                }
                p.blend(x as i32, y as i32, c, 1.0);
            }
        }
        // grass tufts swaying along the cliff top
        let gx0 = g.face(g.h * 0.5) as i32;
        for x in (gx0.max(0))..p.w as i32 {
            let fx = x as f32;
            let hgt = (0.6 + 1.6 * noise(fx * 0.4 / s)) * s;
            let sway = (t * 1.6 + fx * 0.15).sin() * 0.5 * s;
            if hash(x as u32 + 4400) > 0.45 {
                let y0 = g.surf(fx);
                p.line(fx, y0, fx + sway, y0 - hgt, 0.5, rgb(40.0, 56.0, 64.0), 0.9);
            }
        }

        // ---------- lighthouse ----------
        let lx = (w * 0.83).max(g.face(h * 0.5) + 9.0 * s);
        let tb = g.surf(lx) + 0.8 * s;
        let th = 21.0 * s;
        let top = tb - th;
        let (wb, wt) = (3.9 * s, 2.5 * s);
        let red = rgb(196.0, 58.0, 50.0);
        let white = rgb(226.0, 228.0, 236.0);
        let ang = beam_angle(t);
        let facing = (ang.sin().max(0.0)).max((ang + PI).sin().max(0.0));
        let flash = smoothstep(0.86, 1.0, facing).powi(2);
        for y in top.max(0.0) as i32..tb as i32 {
            let fy = y as f32;
            let f = (fy - top) / th;
            let half = lerp(wt, wb, f);
            let band = ((fy - top) / (4.3 * s)).floor() as i32;
            let base = if band % 2 == 0 { red } else { white };
            for x in (lx - half).floor() as i32..=(lx + half).ceil() as i32 {
                let u = ((x as f32 + 0.5 - (lx - half)) / (2.0 * half)).clamp(0.0, 1.0);
                let shade = lerp(1.1, 0.32, u.powf(0.8));
                let mut c = base.scale(shade);
                if u < 0.12 {
                    c = c.mix(rgb(220.0, 226.0, 250.0), 0.25);
                }
                c = c.mix(BEAM, flash * 0.25 * (1.0 - u));
                p.blend(x, y, c, 1.0);
            }
        }
        // windows and door
        for f in [0.3f32, 0.62] {
            let y = top + th * f;
            p.rect(
                lx - 0.6 * s,
                y,
                lx + 0.4 * s,
                y + 1.4 * s,
                rgb(255.0, 196.0, 120.0),
                0.85,
            );
        }
        p.rect(
            lx - 1.2 * s,
            tb - 3.2 * s,
            lx + 0.8 * s,
            tb,
            rgb(30.0, 24.0, 30.0),
            1.0,
        );
        // gallery and railing
        p.rect(
            lx - wt - 1.9 * s,
            top - 0.9 * s,
            lx + wt + 1.9 * s,
            top + 0.3 * s,
            rgb(34.0, 34.0, 44.0),
            1.0,
        );
        p.rect(
            lx - wt - 1.9 * s,
            top - 2.6 * s,
            lx + wt + 1.9 * s,
            top - 2.2 * s,
            rgb(40.0, 40.0, 52.0),
            1.0,
        );
        let posts = ((2.0 * wt + 3.8 * s) / (1.1 * s)) as i32;
        for k in 0..=posts {
            let x = lx - wt - 1.9 * s + k as f32 * (2.0 * wt + 3.8 * s) / posts.max(1) as f32;
            p.rect(
                x - 0.2,
                top - 2.6 * s,
                x + 0.25,
                top - 0.9 * s,
                rgb(40.0, 40.0, 52.0),
                1.0,
            );
        }
        // lantern room
        let ly = top - 3.4 * s;
        let lamp_on = 0.55 + 0.45 * facing;
        p.rect(
            lx - wt * 0.85,
            top - 5.6 * s,
            lx + wt * 0.85,
            top - 0.9 * s,
            rgb(30.0, 34.0, 52.0),
            1.0,
        );
        p.rect(
            lx - wt * 0.7,
            top - 5.2 * s,
            lx + wt * 0.7,
            top - 1.4 * s,
            rgb(255.0, 230.0, 170.0).scale(0.45 + 0.55 * lamp_on),
            0.9,
        );
        p.circle(lx, ly, 1.1 * s, rgb(255.0, 250.0, 230.0), 1.0);
        for k in [-0.35f32, 0.35] {
            let x = lx + k * wt * 1.4;
            p.rect(
                x - 0.2,
                top - 5.6 * s,
                x + 0.25,
                top - 0.9 * s,
                rgb(26.0, 26.0, 36.0),
                0.9,
            );
        }
        // dome, vent ball and lightning rod
        let mut dome = Vec::new();
        for k in 0..=16 {
            let a = PI * k as f32 / 16.0;
            dome.push((lx - a.cos() * wt * 0.95, top - 5.6 * s - a.sin() * 2.4 * s));
        }
        p.poly(&dome, rgb(96.0, 26.0, 28.0), 1.0);
        p.poly(
            &dome[..9]
                .iter()
                .copied()
                .chain([(lx, top - 5.6 * s)])
                .collect::<Vec<_>>(),
            rgb(150.0, 50.0, 46.0),
            0.6,
        );
        p.circle(lx, top - 8.4 * s, 0.7 * s, rgb(70.0, 24.0, 26.0), 1.0);
        p.line(
            lx,
            top - 9.0 * s,
            lx,
            top - 11.5 * s,
            0.4,
            rgb(60.0, 60.0, 70.0),
            1.0,
        );

        // keeper's house with a lit window and chimney smoke
        let hx = lx + 9.0 * s;
        let hb = g.surf(hx) + 0.8 * s;
        let wall = rgb(150.0, 152.0, 168.0);
        p.rect(hx - 4.5 * s, hb - 5.5 * s, hx + 5.5 * s, hb, wall, 1.0);
        p.rect(
            hx + 1.0 * s,
            hb - 5.5 * s,
            hx + 5.5 * s,
            hb,
            wall.scale(0.55),
            1.0,
        );
        p.poly(
            &[
                (hx - 5.5 * s, hb - 5.3 * s),
                (hx + 0.5 * s, hb - 9.0 * s),
                (hx + 6.5 * s, hb - 5.3 * s),
            ],
            rgb(60.0, 30.0, 34.0),
            1.0,
        );
        p.rect(
            hx + 2.5 * s,
            hb - 10.0 * s,
            hx + 3.7 * s,
            hb - 7.0 * s,
            rgb(70.0, 66.0, 76.0),
            1.0,
        );
        let wflick = 0.85 + 0.15 * fbm(t * 2.0 + 3.0);
        p.rect(
            hx - 3.0 * s,
            hb - 4.2 * s,
            hx - 0.8 * s,
            hb - 1.8 * s,
            rgb(255.0, 190.0, 110.0).scale(wflick),
            1.0,
        );
        p.rect(
            hx - 1.95 * s,
            hb - 4.2 * s,
            hx - 1.85 * s + 0.4,
            hb - 1.8 * s,
            rgb(60.0, 40.0, 40.0),
            0.8,
        );
        p.glow(
            hx - 1.9 * s,
            hb - 3.0 * s,
            6.0 * s,
            rgb(255.0, 170.0, 90.0),
            0.3 * wflick,
        );
        for i in 0..18u32 {
            let age = (t * 0.22 + hash(i * 3 + 300)).rem_euclid(1.0);
            let px = hx + 3.1 * s - age * 16.0 * s - 2.0 * s * (t * 0.5 + i as f32).sin() * age;
            let py = hb - 10.0 * s - age * 11.0 * s;
            let r = (0.6 + age * 2.6) * s;
            p.circle(
                px,
                py,
                r,
                rgb(110.0, 120.0, 150.0),
                0.16 * (1.0 - age) * smoothstep(0.0, 0.1, age),
            );
        }

        // ---------- beams ----------
        let (bx0, by0) = (lx, ly);
        let haze_t = t * 0.25;
        for b in 0..2 {
            let a = ang + b as f32 * PI;
            let (ca, sa) = (a.cos(), a.sin());
            let dir = if ca >= 0.0 { 1.0 } else { -1.0 };
            let len = w * 1.1 * ca.abs().powf(0.55) + 3.0 * s;
            let away = (-sa).max(0.0);
            let end_y = by0 + (hz - by0) * 0.85 * away - h * 0.04 * sa.max(0.0);
            let spread = 0.07 + 0.12 * sa.max(0.0);
            let strength = (0.75 + 0.45 * sa) * (0.45 + 0.55 * ca.abs().powf(0.3));
            let (ex, ey) = (bx0 + dir * len, end_y);
            let (vx, vy) = (ex - bx0, ey - by0);
            let vl = (vx * vx + vy * vy).sqrt().max(1.0);
            let (ux, uy) = (vx / vl, vy / vl);
            let max_half = 1.5 * s + vl * spread;
            let xa = bx0.min(ex).max(0.0) as i32;
            let xb = bx0.max(ex).min(w - 1.0) as i32;
            let ya = (by0.min(ey) - max_half).max(0.0) as i32;
            let yb = (by0.max(ey) + max_half).min(h - 1.0) as i32;
            for y in ya..=yb {
                for x in xa..=xb {
                    let (rx, ry) = (x as f32 + 0.5 - bx0, y as f32 + 0.5 - by0);
                    let u = rx * ux + ry * uy;
                    if u < 0.0 || u > vl {
                        continue;
                    }
                    let perp = (rx * uy - ry * ux).abs();
                    let half = 1.2 * s + u * spread;
                    let core = (-(perp / half).powi(2) * 2.6).exp();
                    if core < 0.02 {
                        continue;
                    }
                    let fall = (1.0 - u / vl).powf(1.3);
                    let haze = 0.55
                        + 0.45
                            * noise2(
                                x as f32 * 0.04 / s - haze_t,
                                y as f32 * 0.12 / s + b as f32 * 9.0,
                            );
                    let mut k = core * fall * haze * strength;
                    // the tower and lantern block the beam when it points away
                    if away > 0.2 && u < 3.0 * s {
                        k *= 0.3;
                    }
                    if g.in_cliff(x as f32, y as f32) {
                        k *= 0.15;
                    }
                    p.add(x, y, BEAM, k * 0.6);
                }
            }
        }
        // flash when a lamp faces you
        if flash > 0.01 {
            p.glow(bx0, by0, 34.0 * s, BEAM, 0.9 * flash);
            p.glow(bx0, by0, 9.0 * s, rgb(255.0, 255.0, 255.0), 1.4 * flash);
            let arm = 26.0 * s * flash;
            for j in 0..(arm as i32) {
                let f = 1.0 - j as f32 / arm;
                p.add(bx0 as i32 + j, by0 as i32, BEAM, f * flash);
                p.add(bx0 as i32 - j, by0 as i32, BEAM, f * flash);
                if (j as f32) < arm * 0.4 {
                    p.add(bx0 as i32, by0 as i32 + j, BEAM, f * flash * 0.6);
                    p.add(bx0 as i32, by0 as i32 - j, BEAM, f * flash * 0.6);
                }
            }
        }
        p.glow(bx0, by0, 6.0 * s, BEAM, 0.5 + 0.3 * lamp_on);

        // ---------- surf against the cliff ----------
        let wave_rate = 0.85;
        for y in (hz as i32 + 1).max(0)..p.h as i32 {
            let fy = y as f32;
            let d = (fy - hz) / (h - hz);
            let edge = g.face(fy);
            let surge = ((t * wave_rate - d * 2.2).sin().max(0.0)).powi(3);
            let width = (1.0 + (2.0 + 9.0 * surge) * d) * s;
            for x in (edge - width).floor() as i32..=edge.ceil() as i32 {
                let off = edge - x as f32;
                let n = noise2(x as f32 * 0.3 / s + t * 0.6, fy * 0.3 / s - t * 0.4);
                let a = (1.0 - off / width).max(0.0) * (0.45 + 0.55 * n);
                p.blend(x, y, FOAM, a * (0.5 + 0.5 * surge));
            }
            // streaks of foam trailing back out to sea
            if hash(y as u32 + 1200) > 0.6 {
                let len = (4.0 + 18.0 * surge) * s * d;
                let n = len as i32;
                for j in 0..n {
                    let a = 0.25 * (1.0 - j as f32 / n.max(1) as f32) * (0.3 + surge);
                    p.blend((edge - width - j as f32) as i32, y, FOAM, a);
                }
            }
        }
        // spray bursts when a big wave hits
        let n_burst = (t * wave_rate / TAU).floor();
        for back in 0..2 {
            let kb = n_burst - back as f32;
            let t_hit = (kb * TAU + PI * 0.5) / wave_rate;
            let age = t - t_hit + 0.25;
            if !(0.0..1.8).contains(&age) {
                continue;
            }
            let kbu = kb as i32 as u32;
            for i in 0..110u32 {
                let r = |q: u32| hash(kbu.wrapping_mul(977) ^ (i * 11 + q));
                let yy = lerp(hz + (h - hz) * 0.45, h * 0.98, r(1));
                let x0 = g.face(yy);
                let vx = -(2.0 + 10.0 * r(2)) * s;
                let vy = -(18.0 + 34.0 * r(3)) * s * (0.6 + 0.6 * (yy - hz) / (h - hz));
                let px = x0 + vx * age;
                let py = yy + vy * age + 30.0 * s * age * age;
                if py > yy {
                    continue;
                }
                let a = (1.0 - age / 1.8) * (0.5 + 0.5 * r(4));
                p.circle(
                    px,
                    py,
                    (0.35 + 0.45 * r(5)) * s * (1.0 + 0.6 * age),
                    FOAM,
                    a * 0.55,
                );
            }
            p.glow(
                g.face(h * 0.85),
                h * 0.8,
                14.0 * s,
                rgb(140.0, 160.0, 200.0),
                0.25 * (1.0 - age / 1.8),
            );
        }

        // ---------- foreground rocks ----------
        for (k, (rx, ry, rs)) in [
            (0.13f32, 0.95f32, 1.15f32),
            (0.36, 0.84, 0.6),
            (0.03, 0.8, 0.5),
        ]
        .into_iter()
        .enumerate()
        {
            let (cx, cy) = (w * rx, h * ry);
            let sz = 9.0 * s * rs;
            let ku = k as u32 * 50;
            // jagged silhouette: a few peaks over a wide base
            let n = 14;
            let mut pts = vec![(cx - sz * 1.5, cy)];
            for j in 0..=n {
                let u = j as f32 / n as f32;
                let env = (u * PI).sin().powf(0.7);
                let jag = 0.55 + 0.45 * hash(ku + j);
                pts.push((cx - sz * 1.4 + u * sz * 2.8, cy - sz * 1.7 * env * jag));
            }
            pts.push((cx + sz * 1.5, cy));
            p.poly(&pts, rgb(12.0, 14.0, 24.0), 1.0);
            // moonlight catches the left-facing edges
            for j in 1..pts.len() - 2 {
                let (a, b) = (pts[j], pts[j + 1]);
                if b.1 < a.1 {
                    p.line(
                        a.0,
                        a.1 + 0.3 * s,
                        b.0,
                        b.1 + 0.3 * s,
                        0.8 * s,
                        rgb(78.0, 90.0, 128.0),
                        0.85,
                    );
                }
            }
            // swell rising and falling around the base, foam in the gaps
            let swell = 0.5 + 0.5 * (t * 1.1 + k as f32 * 1.9).sin();
            let wl = cy - (0.5 + 1.8 * swell) * s * rs;
            p.rect(
                cx - sz * 1.6,
                wl,
                cx + sz * 1.6,
                cy + 0.5 * s,
                rgb(10.0, 18.0, 36.0),
                0.85,
            );
            let steps = (sz * 3.4) as i32;
            for j in 0..steps {
                let x = cx - sz * 1.7 + j as f32;
                let n = noise(x * 0.35 / s + t * 1.4 + k as f32 * 9.0);
                if n > 0.4 {
                    p.blend(
                        x as i32,
                        wl as i32,
                        FOAM,
                        (n - 0.4) * 1.6 * (0.4 + 0.6 * swell),
                    );
                    p.blend(x as i32, wl as i32 + 1, FOAM, (n - 0.4) * 0.8 * swell);
                }
            }
            // water pouring off when the swell drops
            if swell < 0.35 {
                let a = (0.35 - swell) * 2.0;
                for j in 0..8 {
                    let x = cx - sz + 2.0 * sz * hash(ku + 30 + j);
                    let top_y = cy - sz * 0.5 * hash(ku + 40 + j);
                    p.line(x, top_y, x, wl, 0.5, FOAM, a * 0.5);
                }
            }
        }
    }
}
