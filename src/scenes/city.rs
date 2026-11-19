//! A rainy night in a Tokyo back alley, rendered in real perspective. Buildings
//! line both sides with lit windows, shopfronts and shutters; neon blade signs
//! flicker (one is failing); paper lanterns sway on a wire; the wet street
//! mirrors everything through rippling puddles; a vending machine hums, steam
//! rises from a grate, someone with a clear umbrella walks away into the haze,
//! and lightning sometimes flickers behind the skyline.

use super::Scene;
use crate::canvas::Pixels;
use crate::theme::Theme;
use crate::util::{hash, noise, noise2, rgb, smoothstep, Color};

pub struct City;

const CAM_H: f32 = 1.6;
const HALF: f32 = 4.0;
const Z_FAR: f32 = 62.0;
const FOG: Color = rgb(54.0, 34.0, 66.0);

const NEON: [Color; 6] = [
    rgb(255.0, 64.0, 170.0),
    rgb(40.0, 230.0, 255.0),
    rgb(255.0, 170.0, 40.0),
    rgb(176.0, 96.0, 255.0),
    rgb(255.0, 56.0, 64.0),
    rgb(90.0, 255.0, 150.0),
];

/// 5x5 katakana, one row per byte (bit 4 = left column).
const GLYPHS: [[u8; 5]; 11] = [
    [0b01110, 0b00000, 0b11111, 0b00010, 0b01100], // ラ
    [0b00100, 0b00100, 0b00100, 0b00100, 0b00100], // ー (vertical)
    [0b00010, 0b10010, 0b01100, 0b01010, 0b10001], // メ
    [0b10000, 0b01001, 0b00001, 0b00010, 0b11100], // ン
    [0b00011, 0b01000, 0b01010, 0b10001, 0b10001], // バ
    [0b00100, 0b11111, 0b00100, 0b10101, 0b01100], // ホ
    [0b11111, 0b00000, 0b11111, 0b00100, 0b01000], // テ
    [0b01010, 0b01010, 0b01010, 0b01011, 0b10010], // ル
    [0b01000, 0b11111, 0b01001, 0b01001, 0b10011], // カ
    [0b00010, 0b11111, 0b00110, 0b01010, 0b10010], // オ
    [0b01000, 0b01111, 0b10010, 0b00010, 0b00100], // ケ
];

const WORDS: [&[usize]; 4] = [&[0, 1, 2, 3], &[4, 1], &[5, 6, 7], &[8, 0, 9, 10]];

struct Cam {
    vx: f32,
    vy: f32,
    f: f32,
}

impl Cam {
    fn at(&self, x: f32, y: f32, z: f32) -> (f32, f32) {
        (self.vx + x * self.f / z, self.vy + (CAM_H - y) * self.f / z)
    }
}

fn fog(z: f32) -> f32 {
    1.0 - (-z * 0.03).exp()
}

fn add(c: Color, l: Color) -> Color {
    rgb(
        (c.r + l.r).min(255.0),
        (c.g + l.g).min(255.0),
        (c.b + l.b).min(255.0),
    )
}

#[derive(Clone, Copy)]
struct Seg {
    z0: f32,
    z1: f32,
    h: f32,
    seed: u32,
}

fn segments(side: u32) -> Vec<Seg> {
    let mut v = Vec::new();
    let mut z = 0.2;
    let mut i = 0u32;
    while z < Z_FAR {
        let seed = side * 1000 + i;
        let len = 3.5 + hash(seed * 7 + 1) * 6.5;
        let h = 7.0 + hash(seed * 7 + 2) * 15.0;
        v.push(Seg {
            z0: z,
            z1: z + len,
            h,
            seed,
        });
        z += len + if hash(seed * 7 + 3) > 0.72 { 1.3 } else { 0.0 };
        i += 1;
    }
    v
}

struct Sign {
    side: f32,
    z: f32,
    y0: f32,
    len: f32,
    col: Color,
    word: usize,
    b: f32,
}

/// A point light: world position, colour, strength.
struct Light {
    x: f32,
    y: f32,
    z: f32,
    col: Color,
    k: f32,
    r2: f32,
}

fn signs(t: f32) -> Vec<Sign> {
    let spots = [
        (-1.0, 5.6),
        (1.0, 7.8),
        (-1.0, 11.0),
        (1.0, 14.5),
        (-1.0, 18.0),
        (1.0, 22.0),
        (-1.0, 27.0),
        (1.0, 32.0),
        (-1.0, 38.0),
        (1.0, 45.0),
    ];
    spots
        .iter()
        .enumerate()
        .map(|(i, &(side, z))| {
            let iu = i as u32;
            let hum = 0.93 + 0.07 * (t * 47.0 + i as f32).sin();
            // sign 3 is failing: it stutters and drops out
            let b = if i == 3 {
                let n = noise(t * 7.0);
                let stutter = (t * 31.0).sin() > 0.2;
                if n < 0.35 || (n < 0.5 && stutter) {
                    0.08
                } else {
                    1.0
                }
            } else if i == 6 {
                if noise(t * 0.6 + 4.0) < 0.2 {
                    0.25
                } else {
                    1.0
                }
            } else {
                1.0
            };
            Sign {
                side,
                z,
                y0: 2.9 + hash(iu * 5 + 1) * 1.1,
                len: 2.6 + hash(iu * 5 + 2) * 1.8,
                col: NEON[(i * 5 + 1) % NEON.len()],
                word: i % WORDS.len(),
                b: b * hum,
            }
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn wall_colour(
    sg: &Seg,
    side: f32,
    y: f32,
    z: f32,
    t: f32,
    lights: &[Light],
    lit_shop: &mut bool,
) -> Color {
    let r = |k: u32| hash(sg.seed.wrapping_mul(13).wrapping_add(k));
    let bases = [
        rgb(46.0, 40.0, 50.0),
        rgb(38.0, 40.0, 52.0),
        rgb(56.0, 44.0, 42.0),
        rgb(32.0, 32.0, 40.0),
    ];
    let base = bases[(r(1) * 4.0) as usize % 4];
    let u = z - sg.z0;
    let len = sg.z1 - sg.z0;
    let grime = 0.78 + 0.4 * noise2(u * 1.3, y * 0.7);
    let mut c = base.scale(grime);
    let level = (y / 3.0).floor();
    let fy = y / 3.0 - level;
    let bay = 1.8 + r(2) * 0.9;
    let col_i = (u / bay).floor();
    let fu = u / bay - col_i;

    if level < 1.0 {
        if r(3) > 0.35 {
            *lit_shop = true;
            let lit = if r(4) > 0.5 {
                rgb(255.0, 190.0, 120.0)
            } else {
                rgb(210.0, 228.0, 255.0)
            };
            if fy < 0.7 && fu > 0.05 && fu < 0.95 {
                c = lit.scale(0.5 + 0.25 * noise2(u * 2.0, y * 3.0));
                if (y * 3.5).fract() < 0.12 {
                    c = c.scale(0.55);
                }
                // noren curtain hanging in the doorway
                if fy > 0.5 {
                    let noren = if r(6) > 0.5 {
                        rgb(40.0, 50.0, 130.0)
                    } else {
                        rgb(150.0, 30.0, 40.0)
                    };
                    let slit = (fu * 5.0).fract() < 0.08;
                    c = if slit { lit.scale(0.7) } else { noren };
                }
            } else if (0.75..0.95).contains(&fy) {
                // lit shop sign above the door
                let sc = NEON[(r(5) * 6.0) as usize % 6].mix(rgb(255.0, 255.0, 255.0), 0.4);
                let bar = ((fu * 9.0).fract() < 0.6) && (0.8..0.9).contains(&fy);
                c = if bar { sc.scale(0.9) } else { sc.scale(0.45) };
            }
        } else if fy < 0.78 {
            // rolled-down shutter
            c = rgb(74.0, 76.0, 86.0)
                .scale(0.7 + 0.3 * ((y * 30.0).sin() * 0.5 + 0.5))
                .scale(grime);
            if noise2(u * 0.8 + 5.0, y * 1.5) > 0.72 {
                c = c.mix(NEON[(r(7) * 6.0) as usize % 6].scale(0.5), 0.5); // graffiti
            }
        }
    } else {
        if fy < 0.07 {
            c = c.scale(0.55);
        } else if fy < 0.1 {
            c = c.scale(1.25);
        }
        if fu > 0.2 && fu < 0.8 && fy > 0.28 && fy < 0.82 {
            let wid = hash(
                sg.seed
                    .wrapping_mul(97)
                    .wrapping_add(level as u32 * 13 + col_i as i32 as u32 * 7),
            );
            let frame = !(0.25..=0.75).contains(&fu) || !(0.33..=0.77).contains(&fy);
            if frame {
                c = c.scale(0.6);
            } else if wid > 0.48 {
                let tone = hash(
                    sg.seed
                        .wrapping_add(level as u32 * 31 + col_i as i32 as u32 * 3)
                        + 5,
                );
                let mut lit = if tone > 0.85 {
                    rgb(90.0, 140.0, 255.0).scale(0.45 + 0.55 * noise(t * 5.0 + wid * 50.0))
                } else if tone > 0.45 {
                    rgb(255.0, 200.0, 130.0).scale(0.8)
                } else {
                    rgb(200.0, 214.0, 255.0).scale(0.7)
                };
                // curtains half drawn
                if fu < 0.25 + 0.3 * wid {
                    lit = lit.scale(0.45);
                }
                c = lit;
            } else {
                c = rgb(16.0, 16.0, 26.0);
                if (fu - fy * 0.5).fract() < 0.1 {
                    c = c.scale(1.8);
                }
            }
        } else if fu > 0.82
            && fu < 0.97
            && fy > 0.12
            && fy < 0.3
            && hash(sg.seed + level as u32 * 7 + col_i as i32 as u32 * 3 + 900) > 0.55
        {
            c = rgb(112.0, 112.0, 124.0).scale(grime); // air-con unit
        }
    }
    if u < 0.12 || len - u < 0.12 {
        c = c.scale(0.7);
    }
    if (u - len * 0.5).abs() < 0.06 {
        c = rgb(58.0, 60.0, 70.0);
    }
    // light from nearby signs, lanterns and shops
    let wx = side * HALF;
    let mut l = rgb(0.0, 0.0, 0.0);
    for li in lights {
        if (li.z - z).abs() > 7.0 {
            continue;
        }
        let d2 = (li.x - wx).powi(2) + (li.y - y).powi(2) + (li.z - z).powi(2);
        let k = li.k * (-d2 / li.r2).exp();
        l = rgb(l.r + li.col.r * k, l.g + li.col.g * k, l.b + li.col.b * k);
    }
    add(c, l)
}

/// Draw a pixel-thin line by stepping.
fn thin(p: &mut Pixels, a: (f32, f32), b: (f32, f32), col: Color, alpha: f32) {
    let n = (b.0 - a.0).abs().max((b.1 - a.1).abs()).ceil().max(1.0) as i32;
    for j in 0..=n {
        let f = j as f32 / n as f32;
        p.blend(
            (a.0 + (b.0 - a.0) * f) as i32,
            (a.1 + (b.1 - a.1) * f) as i32,
            col,
            alpha,
        );
    }
}

fn quad(p: &mut Pixels, cam: &Cam, pts: [(f32, f32, f32); 4], mirror: bool, col: Color, a: f32) {
    let m = if mirror { -1.0 } else { 1.0 };
    let q: Vec<(f32, f32)> = pts.iter().map(|&(x, y, z)| cam.at(x, y * m, z)).collect();
    p.poly(&q, col, a);
}

fn vending(p: &mut Pixels, cam: &Cam, t: f32, mirror: bool) {
    let a = if mirror { 0.32 } else { 1.0 };
    let xf = -HALF + 0.75;
    let (z0, z1, hg) = (6.0, 7.0, 1.85);
    let hum = 0.94 + 0.06 * (t * 13.0).sin();
    // side panel facing the camera
    quad(
        p,
        cam,
        [
            (-HALF, 0.0, z0),
            (xf, 0.0, z0),
            (xf, hg, z0),
            (-HALF, hg, z0),
        ],
        mirror,
        rgb(150.0, 30.0, 40.0),
        a,
    );
    // front body
    quad(
        p,
        cam,
        [(xf, 0.0, z0), (xf, 0.0, z1), (xf, hg, z1), (xf, hg, z0)],
        mirror,
        rgb(214.0, 220.0, 232.0).scale(hum),
        a,
    );
    // display window
    let (d0, d1) = (z0 + 0.08, z1 - 0.08);
    quad(
        p,
        cam,
        [
            (xf, 0.95, d0),
            (xf, 0.95, d1),
            (xf, 1.72, d1),
            (xf, 1.72, d0),
        ],
        mirror,
        rgb(240.0, 248.0, 255.0).scale(hum),
        a,
    );
    let cans = [
        rgb(220.0, 40.0, 40.0),
        rgb(40.0, 90.0, 220.0),
        rgb(40.0, 170.0, 80.0),
        rgb(250.0, 150.0, 30.0),
        rgb(240.0, 240.0, 240.0),
        rgb(120.0, 70.0, 40.0),
    ];
    for row in 0..3 {
        let y0 = 1.02 + row as f32 * 0.24;
        for c in 0..6 {
            let za = d0 + 0.02 + c as f32 * (d1 - d0) / 6.0;
            let zb = za + (d1 - d0) / 6.0 - 0.04;
            let col = cans[(row * 2 + c) % cans.len()];
            quad(
                p,
                cam,
                [
                    (xf, y0, za),
                    (xf, y0, zb),
                    (xf, y0 + 0.15, zb),
                    (xf, y0 + 0.15, za),
                ],
                mirror,
                col,
                a,
            );
        }
    }
    // buttons and slot
    quad(
        p,
        cam,
        [
            (xf, 0.82, d0),
            (xf, 0.82, d1),
            (xf, 0.88, d1),
            (xf, 0.88, d0),
        ],
        mirror,
        rgb(80.0, 160.0, 255.0),
        a,
    );
    quad(
        p,
        cam,
        [
            (xf, 0.12, d0 + 0.1),
            (xf, 0.12, d1 - 0.1),
            (xf, 0.38, d1 - 0.1),
            (xf, 0.38, d0 + 0.1),
        ],
        mirror,
        rgb(30.0, 30.0, 36.0),
        a,
    );
    if !mirror {
        let (gx, gy) = cam.at(xf, 1.2, (z0 + z1) * 0.5);
        p.glow(
            gx,
            gy,
            cam.f * 1.1 / 6.5,
            rgb(190.0, 220.0, 255.0),
            0.3 * hum,
        );
    }
}

fn walker(p: &mut Pixels, cam: &Cam, t: f32, mirror: bool) {
    let cyc = 36.0;
    let ph = (t * 1.05).rem_euclid(cyc);
    let z = 4.2 + ph;
    let x = 0.5 - 0.25 * ph / cyc;
    let mut a =
        smoothstep(0.0, 1.8, ph) * (1.0 - smoothstep(cyc - 3.0, cyc, ph)) * (1.0 - fog(z) * 0.8);
    if mirror {
        a *= 0.3;
    }
    if a <= 0.01 {
        return;
    }
    let m = if mirror { -1.0 } else { 1.0 };
    let pr = |px: f32, py: f32| cam.at(px, py * m, z);
    let sc = cam.f / z;
    let step = t * 5.4;
    let bob = 0.03 * (step * 2.0).sin().abs();
    let body = rgb(14.0, 12.0, 22.0).mix(FOG, fog(z));
    let lf = (0.09 * step.sin()).max(0.0);
    let rf = (0.09 * (step + std::f32::consts::PI).sin()).max(0.0);
    for (fx, fy) in [(x - 0.1, lf), (x + 0.1, rf)] {
        let (h0, h1) = (pr(fx * 0.8 + x * 0.2, 0.9 + bob), pr(fx, fy));
        p.line(h0.0, h0.1, h1.0, h1.1, 0.13 * sc, body, a);
    }
    // long coat
    let coat = [
        pr(x - 0.22, 1.45 + bob),
        pr(x + 0.22, 1.45 + bob),
        pr(x + 0.25, 0.55 + bob),
        pr(x - 0.25, 0.55 + bob),
    ];
    p.poly(&coat, body, a);
    let (hx, hy) = pr(x, 1.62 + bob);
    p.circle(hx, hy, 0.11 * sc, body, a);
    // arm up to a clear umbrella
    let sh = pr(x + 0.2, 1.4 + bob);
    let hand = pr(x + 0.06, 1.3 + bob);
    p.line(sh.0, sh.1, hand.0, hand.1, 0.08 * sc, body, a);
    let top = pr(x, 2.1 + bob);
    thin(p, hand, top, rgb(60.0, 60.0, 70.0), a);
    let (ux, uy) = pr(x, 1.98 + bob);
    let (rx, ry) = (0.55 * sc, 0.3 * sc);
    let mut dome = Vec::new();
    for k in 0..=16 {
        let ang = std::f32::consts::PI * k as f32 / 16.0;
        dome.push((ux - ang.cos() * rx, uy - m * ang.sin() * ry));
    }
    p.poly(&dome, rgb(190.0, 200.0, 230.0), 0.22 * a);
    for k in [2usize, 5, 8, 11, 14] {
        thin(
            p,
            (ux, uy - m * ry),
            dome[k],
            rgb(220.0, 225.0, 245.0),
            0.35 * a,
        );
    }
    thin(p, dome[0], dome[16], rgb(255.0, 150.0, 220.0), 0.6 * a);
}

impl Scene for City {
    fn name(&self) -> &'static str {
        "city"
    }

    fn blurb(&self) -> &'static str {
        "Rainy Tokyo alley at night: neon signs, wet reflections, lanterns, steam"
    }

    fn draw(&self, p: &mut Pixels, t: f32, _theme: Option<&Theme>) {
        let (w, h) = (p.w as f32, p.h as f32);
        let s = h / 80.0;
        let cam = Cam {
            vx: w * 0.5,
            vy: h * 0.42,
            f: h * 0.7,
        };

        // lightning behind the skyline every so often
        let lp = 17.0;
        let lk = (t / lp).floor();
        let lt = t - lk * lp - (4.0 + 8.0 * hash(lk as i32 as u32 * 3 + 1));
        let flash = if hash(lk as i32 as u32 * 3 + 2) > 0.35 && lt > 0.0 {
            (-lt * 5.0).exp() + 0.6 * (-(lt - 0.18).abs() * 25.0).exp()
        } else {
            0.0
        };

        // ---------- sky and distant skyline ----------
        p.gradient(&[
            (0.0, rgb(10.0, 6.0, 20.0)),
            (0.25, rgb(30.0, 14.0, 40.0)),
            (0.44, rgb(96.0, 40.0, 80.0)),
            (1.0, rgb(96.0, 40.0, 80.0)),
        ]);
        for y in 0..(cam.vy as usize).min(p.h) {
            for x in 0..p.w {
                let (fx, fy) = (x as f32, y as f32);
                let n = noise2(fx * 0.02 / s + t * 0.03, fy * 0.07 / s);
                let c = smoothstep(0.42, 0.8, n) * (0.3 + 0.7 * fy / cam.vy);
                p.blend(
                    x as i32,
                    y as i32,
                    rgb(120.0, 56.0, 100.0).mix(rgb(220.0, 210.0, 255.0), flash),
                    c * 0.5,
                );
                if flash > 0.0 {
                    p.add(
                        x as i32,
                        y as i32,
                        rgb(200.0, 190.0, 255.0),
                        flash * 0.35 * (1.0 - fy / cam.vy * 0.5),
                    );
                }
            }
        }
        let tower = rgb(32.0, 20.0, 42.0).mix(FOG, 0.4);
        for i in 0..44u32 {
            let tx = (hash(i * 4 + 1) * 1.2 - 0.1) * w;
            let tw = (2.0 + hash(i * 4 + 2) * 6.0) * s;
            let top = cam.vy - (3.0 + hash(i * 4 + 3) * hash(i * 4 + 3) * 34.0) * s;
            p.rect(tx, top, tx + tw, cam.vy + 3.0 * s, tower, 1.0);
            let step = (1.4 * s).max(2.0) as i32;
            let mut wy = top as i32 + step;
            while (wy as f32) < cam.vy {
                let mut wx = tx as i32 + 1;
                while (wx as f32) < tx + tw - 1.0 {
                    let hh = hash((wx as u32).wrapping_mul(31) ^ (wy as u32).wrapping_mul(977) ^ i);
                    if hh > 0.72 {
                        let c = if hh > 0.93 {
                            rgb(120.0, 200.0, 255.0)
                        } else {
                            rgb(255.0, 190.0, 120.0)
                        };
                        p.blend(wx, wy, c, 0.45);
                    }
                    wx += step;
                }
                wy += step;
            }
            if top < cam.vy - 22.0 * s && (t * 1.3 + i as f32).sin() > 0.5 {
                p.add(
                    (tx + tw * 0.5) as i32,
                    top as i32 - 1,
                    rgb(255.0, 40.0, 40.0),
                    1.0,
                );
            }
        }
        // a big far billboard
        {
            let bx = w * 0.53;
            let byy = cam.vy - 13.0 * s;
            let col = NEON[1].scale(0.8 + 0.2 * (t * 2.0).sin());
            p.rect(bx, byy, bx + 6.0 * s, byy + 3.5 * s, col.scale(0.5), 0.8);
            p.rect(
                bx + 0.6 * s,
                byy + 0.8 * s,
                bx + 5.4 * s,
                byy + 1.4 * s,
                col,
                0.8,
            );
            p.glow(bx + 3.0 * s, byy + 1.8 * s, 9.0 * s, col, 0.25);
        }

        // ---------- lights ----------
        let signs = signs(t);
        let mut lights: Vec<Light> = signs
            .iter()
            .map(|sg| Light {
                x: sg.side * (HALF - 0.45),
                y: sg.y0 + sg.len * 0.5,
                z: sg.z,
                col: sg.col,
                k: 0.32 * sg.b,
                r2: 4.0,
            })
            .collect();
        let lantern_rows = [(9.5f32, 7usize, 4.4f32), (24.0, 6, 4.8)];
        for &(lz, n, ly) in &lantern_rows {
            for i in 0..n {
                let u = (i as f32 + 0.5) / n as f32 * 2.0 - 1.0;
                lights.push(Light {
                    x: u * (HALF - 0.5),
                    y: ly - 0.35 * (1.0 - u * u),
                    z: lz,
                    col: rgb(255.0, 80.0, 50.0),
                    k: 0.18,
                    r2: 2.5,
                });
            }
        }
        lights.push(Light {
            x: -HALF + 0.9,
            y: 1.1,
            z: 6.5,
            col: rgb(190.0, 220.0, 255.0),
            k: 0.3,
            r2: 2.5,
        });

        // ---------- walls and floor, ray by ray ----------
        let segs = [segments(0), segments(1)];
        let mut floor_z = vec![0f32; p.w * p.h];
        let mut puddle = vec![0f32; p.w * p.h];
        for y in 0..p.h {
            for x in 0..p.w {
                let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
                let dx = fx - cam.vx;
                let dy = fy - cam.vy;
                let zw = if dx.abs() > 0.01 {
                    HALF * cam.f / dx.abs()
                } else {
                    f32::MAX
                };
                let zf = if dy > 0.01 {
                    CAM_H * cam.f / dy
                } else {
                    f32::MAX
                };
                if zf <= zw && zf < f32::MAX {
                    let wx = dx * zf / cam.f;
                    let tex = noise2(wx * 2.0, zf * 1.0);
                    let mut c = rgb(18.0, 17.0, 24.0).scale(0.75 + 0.45 * tex);
                    if wx.abs() > HALF - 0.35 {
                        c = rgb(40.0, 38.0, 48.0).scale(0.8 + 0.3 * tex);
                    }
                    if ((wx - 0.6).powi(2) + (zf - 7.0).powi(2)).sqrt() < 0.35 {
                        c = rgb(34.0, 34.0, 40.0);
                    }
                    let pud = smoothstep(0.42, 0.56, noise2(wx * 0.55 + 3.1, zf * 0.22));
                    let mut l = rgb(0.0, 0.0, 0.0);
                    for li in &lights {
                        if (li.z - zf).abs() > 7.0 {
                            continue;
                        }
                        let d2 = (li.x - wx).powi(2) + li.y.powi(2) * 0.5 + (li.z - zf).powi(2);
                        let k = li.k * 0.7 * (-d2 / (li.r2 * 1.6)).exp();
                        l = rgb(l.r + li.col.r * k, l.g + li.col.g * k, l.b + li.col.b * k);
                    }
                    // shop light spilling out near the walls
                    let side = if wx < 0.0 { 0 } else { 1 };
                    if wx.abs() > HALF - 1.8 {
                        if let Some(sg) = segs[side].iter().find(|sg| zf >= sg.z0 && zf < sg.z1) {
                            if hash(sg.seed.wrapping_mul(13).wrapping_add(3)) > 0.35 {
                                let k = smoothstep(HALF - 1.8, HALF, wx.abs()) * 0.22;
                                l = add(l, rgb(255.0, 190.0, 130.0).scale(k));
                            }
                        }
                    }
                    c = add(c, l).scale(0.45 + 0.55 * smoothstep(1.5, 9.0, zf));
                    c = c.mix(FOG, fog(zf));
                    p.blend(x as i32, y as i32, c, 1.0);
                    floor_z[y * p.w + x] = zf;
                    puddle[y * p.w + x] = pud;
                } else if zw < Z_FAR {
                    let side = if dx < 0.0 { 0 } else { 1 };
                    let sd = if dx < 0.0 { -1.0 } else { 1.0 };
                    let wy = CAM_H - dy * zw / cam.f;
                    match segs[side].iter().find(|sg| zw >= sg.z0 && zw < sg.z1) {
                        Some(sg) if wy >= 0.0 && wy < sg.h => {
                            let mut shop = false;
                            let mut c = wall_colour(sg, sd, wy, zw, t, &lights, &mut shop);
                            c = c.scale(0.3 + 0.7 * smoothstep(2.5, 16.0, zw));
                            c = add(c, rgb(200.0, 190.0, 255.0).scale(flash * 0.12));
                            p.blend(x as i32, y as i32, c.mix(FOG, fog(zw)), 1.0);
                        }
                        None if wy < 14.0 => {
                            p.blend(
                                x as i32,
                                y as i32,
                                rgb(8.0, 6.0, 12.0).mix(FOG, fog(zw)),
                                1.0,
                            );
                        }
                        _ => {}
                    }
                }
            }
        }

        // ---------- overhead cables ----------
        for k in 0..6u32 {
            let za = 4.5 + k as f32 * 6.5 + hash(k + 70) * 2.0;
            let zb = za + 1.0 + hash(k + 80) * 2.5;
            let (ya, yb) = (5.0 + hash(k + 90) * 3.5, 5.0 + hash(k + 100) * 3.5);
            let n = 28;
            let mut prev = cam.at(-HALF, ya, za);
            for j in 1..=n {
                let u = j as f32 / n as f32;
                let xw = -HALF + 2.0 * HALF * u;
                let yw = ya + (yb - ya) * u - 0.7 * (1.0 - (2.0 * u - 1.0).powi(2));
                let zw = za + (zb - za) * u;
                let cur = cam.at(xw, yw, zw);
                thin(p, prev, cur, rgb(14.0, 10.0, 18.0), 0.85);
                prev = cur;
            }
        }

        // ---------- neon blade signs ----------
        for sg in &signs {
            let (xa, ya) = cam.at(sg.side * HALF, sg.y0 + sg.len, sg.z);
            let (xb, yb) = cam.at(sg.side * (HALF - 1.05), sg.y0, sg.z);
            let (x0, x1) = (xa.min(xb), xa.max(xb));
            let (y0, y1) = (ya, yb);
            let fz = fog(sg.z) * 0.55;
            let col = sg.col.scale(sg.b).mix(FOG, fz);
            p.rect(x0, y0, x1, y1, rgb(16.0, 10.0, 20.0).mix(FOG, fz), 1.0);
            // bracket to the wall
            let (bx, by) = cam.at(sg.side * HALF, sg.y0 + sg.len + 0.2, sg.z);
            thin(p, (bx, by), (bx, y0), rgb(30.0, 26.0, 34.0), 1.0);
            // neon tube border
            let th = (0.06 * cam.f / sg.z).max(1.0);
            p.rect(x0, y0, x1, y0 + th, col, 1.0);
            p.rect(x0, y1 - th, x1, y1, col, 1.0);
            p.rect(x0, y0, x0 + th, y1, col, 1.0);
            p.rect(x1 - th, y0, x1, y1, col, 1.0);
            // glyphs stacked vertically
            let word = WORDS[sg.word];
            let n = word.len() as f32;
            let (iw, ih) = (x1 - x0 - 4.0 * th, y1 - y0 - 4.0 * th);
            if iw > 2.0 && ih > 4.0 {
                let cell_h = ih / n;
                let gsz = (iw.min(cell_h * 0.9)) / 5.0;
                let core = col.mix(rgb(255.0, 255.0, 255.0), 0.45 * sg.b);
                for (gi, &gl) in word.iter().enumerate() {
                    let gx0 = x0 + 2.0 * th + (iw - gsz * 5.0) * 0.5;
                    let gy0 = y0 + 2.0 * th + gi as f32 * cell_h + (cell_h - gsz * 5.0) * 0.5;
                    for (ry, row) in GLYPHS[gl].iter().enumerate() {
                        for rx in 0..5 {
                            if row & (0b10000 >> rx) != 0 {
                                let cx = gx0 + rx as f32 * gsz;
                                let cy = gy0 + ry as f32 * gsz;
                                p.rect(cx, cy, cx + gsz.max(1.0), cy + gsz.max(1.0), core, 1.0);
                            }
                        }
                    }
                }
            }
            let gr = ((y1 - y0) * 0.75).max(3.0 * s);
            p.glow(
                (x0 + x1) * 0.5,
                (y0 + y1) * 0.5,
                gr,
                sg.col,
                0.32 * sg.b * (1.0 - fz),
            );
        }

        // ---------- paper lanterns on a wire ----------
        for &(lz, n, ly) in &lantern_rows {
            let sc = cam.f / lz;
            let mut prev: Option<(f32, f32)> = None;
            for i in 0..n {
                let u = (i as f32 + 0.5) / n as f32 * 2.0 - 1.0;
                let sway = 0.06 * (t * 1.4 + i as f32 * 0.8).sin();
                let wy = ly - 0.35 * (1.0 - u * u);
                let (cx, cy) = cam.at(u * (HALF - 0.5) + sway, wy - 0.38, lz);
                let (tx, ty) = cam.at(u * (HALF - 0.5), wy, lz);
                if let Some(pv) = prev {
                    thin(p, pv, (tx, ty), rgb(16.0, 12.0, 18.0), 0.9);
                }
                prev = Some((tx, ty));
                thin(p, (tx, ty), (cx, cy - 0.3 * sc), rgb(16.0, 12.0, 18.0), 0.9);
                let (rx, ry) = (0.22 * sc, 0.3 * sc);
                let fz = fog(lz) * 0.5;
                let flick = 0.9 + 0.1 * noise(t * 3.0 + i as f32 * 5.0);
                p.ellipse(
                    cx,
                    cy,
                    rx,
                    ry,
                    rgb(210.0, 40.0, 30.0).scale(flick).mix(FOG, fz),
                    1.0,
                );
                p.ellipse(
                    cx,
                    cy,
                    rx * 0.6,
                    ry * 0.75,
                    rgb(255.0, 150.0, 90.0).scale(flick).mix(FOG, fz),
                    0.8,
                );
                for r in [-0.45f32, 0.0, 0.45] {
                    let yy = cy + r * ry;
                    let ww = rx * (1.0 - r * r).sqrt();
                    thin(p, (cx - ww, yy), (cx + ww, yy), rgb(120.0, 20.0, 20.0), 0.6);
                }
                p.rect(
                    cx - rx * 0.55,
                    cy - ry - 0.08 * sc,
                    cx + rx * 0.55,
                    cy - ry + 0.05 * sc,
                    rgb(20.0, 16.0, 18.0),
                    1.0,
                );
                p.rect(
                    cx - rx * 0.55,
                    cy + ry - 0.05 * sc,
                    cx + rx * 0.55,
                    cy + ry + 0.08 * sc,
                    rgb(20.0, 16.0, 18.0),
                    1.0,
                );
                p.glow(
                    cx,
                    cy,
                    rx * 3.5,
                    rgb(255.0, 70.0, 40.0),
                    0.3 * flick * (1.0 - fz),
                );
            }
        }

        // ---------- the wet street mirrors everything ----------
        let snap: Vec<Color> = (0..p.h)
            .flat_map(|y| (0..p.w).map(move |x| (x, y)))
            .map(|(x, y)| p.at(x, y))
            .collect();
        for y in 0..p.h {
            for x in 0..p.w {
                let i = y * p.w + x;
                let zf = floor_z[i];
                if zf <= 0.0 {
                    continue;
                }
                let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
                let dx = fx - cam.vx;
                let zw = if dx.abs() > 0.01 {
                    (HALF * cam.f / dx.abs()).min(Z_FAR)
                } else {
                    Z_FAR
                };
                let yb = cam.vy + CAM_H * cam.f / zw;
                let pud = puddle[i];
                let rip =
                    (noise2(fx * 0.2 / s, fy * 1.2 / s - t * 2.5) - 0.5) * (0.4 + 1.6 * pud) * s;
                let smear = (noise2(fx * 0.6 / s, fy * 0.15 / s + t * 3.0) - 0.5)
                    * (2.0 + 4.0 * (1.0 - pud))
                    * s;
                let sy = (2.0 * yb - fy + smear).clamp(0.0, h - 1.0);
                let sx = (fx + rip).clamp(0.0, w - 1.0);
                let mut src = rgb(0.0, 0.0, 0.0);
                for j in 0..5 {
                    let yy = (sy + (j as f32 - 2.0) * 1.3 * s).clamp(0.0, h - 1.0);
                    let c = snap[yy as usize * p.w + sx as usize];
                    src = rgb(src.r + c.r * 0.2, src.g + c.g * 0.2, src.b + c.b * 0.2);
                }
                let near = (fy - cam.vy) / (h - cam.vy);
                let k = (0.18 + 0.62 * pud) * (0.5 + 0.5 * (1.0 - near));
                let cur = snap[i];
                p.blend(x as i32, y as i32, cur.mix(src.scale(0.85), k), 1.0);
            }
        }

        // ---------- street furniture and passers-by ----------
        vending(p, &cam, t, true);
        vending(p, &cam, t, false);

        // steam from a grate
        {
            let (gz, gx) = (9.0f32, 1.5f32);
            quad(
                p,
                &cam,
                [
                    (gx - 0.35, 0.0, gz - 0.3),
                    (gx + 0.35, 0.0, gz - 0.3),
                    (gx + 0.35, 0.0, gz + 0.3),
                    (gx - 0.35, 0.0, gz + 0.3),
                ],
                false,
                rgb(22.0, 22.0, 28.0),
                1.0,
            );
            for i in 0..34u32 {
                let age = (t * 0.28 + hash(i * 3 + 1)).rem_euclid(1.0);
                let wx = gx
                    + (hash(i * 3 + 2) - 0.5) * 0.4
                    + age * (0.4 + 0.8 * noise(t * 0.3 + i as f32));
                let wy = age * 3.4;
                let wz = gz + (hash(i * 3 + 3) - 0.5) * 0.4;
                let r = (0.16 + age * 0.75) * cam.f / wz;
                let (sx, sy) = cam.at(wx, wy, wz);
                let a = 0.11 * (1.0 - age) * smoothstep(0.0, 0.15, age);
                p.circle(sx, sy, r, rgb(200.0, 170.0, 220.0), a);
            }
        }

        walker(p, &cam, t, true);
        walker(p, &cam, t, false);

        // ---------- rain ----------
        let slant = 0.14 + 0.05 * (t * 0.3).sin();
        let layers = [
            (w * h / 80.0, 1.6 * s, 1.3, 0.14, 11u32),
            (w * h / 240.0, 4.5 * s, 1.9, 0.3, 12),
            (w * h / 700.0, 11.0 * s, 2.7, 0.42, 13),
        ];
        for &(n, len, speed, alpha, seed) in &layers {
            let nn = n as u32;
            for i in 0..nn {
                let r = |k: u32| {
                    hash(
                        i.wrapping_mul(7919)
                            .wrapping_add(seed * 100_003)
                            .wrapping_add(k),
                    )
                };
                let sp = speed * (0.85 + 0.3 * r(3));
                let y0 = (r(2) + t * sp).rem_euclid(1.0) * (h + len) - len;
                let x0 = r(1) * (w + slant * h) - slant * h + slant * y0;
                let steps = len.max(1.0) as i32;
                for j in 0..steps {
                    let f = j as f32 / steps as f32;
                    let px = (x0 + slant * j as f32) as i32;
                    let py = (y0 + j as f32) as i32;
                    if px < 0 || py < 0 || px >= p.w as i32 || py >= p.h as i32 {
                        continue;
                    }
                    let under = p.at(px as usize, py as usize);
                    let c = rgb(150.0, 156.0, 190.0).mix(under.scale(2.2), 0.55);
                    p.blend(px, py, c, alpha * (0.3 + 0.7 * f));
                    if len > 8.0 * s && s > 1.3 {
                        p.blend(px + 1, py, c, alpha * 0.4 * f);
                    }
                }
            }
        }

        // splashes and rings on the street
        let n_spl = (w * 0.45) as u32;
        for i in 0..n_spl {
            let tt = t / 0.45 + hash(i * 5 + 1);
            let cyc = tt.floor() as i32 as u32;
            let age = tt.fract();
            let key = i.wrapping_mul(977).wrapping_add(cyc.wrapping_mul(131));
            let sx = hash(key) * w;
            let sy = cam.vy + 2.0 + (h - cam.vy - 2.0) * hash(key ^ 0x5bd1).powf(0.6);
            let idx = (sy as usize).min(p.h - 1) * p.w + (sx as usize).min(p.w - 1);
            let zf = floor_z[idx];
            if zf <= 0.0 {
                continue;
            }
            let sc = cam.f / zf;
            let rx = age * 0.18 * sc;
            let ry = rx * 0.32;
            let a = (1.0 - age) * 0.28 * (1.0 - fog(zf));
            let col = rgb(190.0, 196.0, 230.0);
            if rx >= 0.8 {
                for k in 0..14 {
                    let ang = std::f32::consts::TAU * k as f32 / 14.0;
                    p.blend(
                        (sx + ang.cos() * rx) as i32,
                        (sy + ang.sin() * ry) as i32,
                        col,
                        a,
                    );
                }
            }
            if age < 0.25 {
                p.blend(sx as i32, sy as i32 - 1, col, 0.6 * (1.0 - age * 4.0));
                p.blend(sx as i32 - 1, sy as i32 - 1, col, 0.3 * (1.0 - age * 4.0));
                p.blend(sx as i32 + 1, sy as i32 - 1, col, 0.3 * (1.0 - age * 4.0));
            }
        }

        if flash > 0.0 {
            p.map(|c| add(c, rgb(150.0, 140.0, 200.0).scale(flash * 0.08)));
        }
    }
}
