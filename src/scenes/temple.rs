//! A pagoda at dusk: swaying pines and grass, a flickering lit window, a
//! swinging lantern, a fluttering banner, fireflies and blowing petals.

use super::{mist, petals, Scene};
use crate::canvas::Pixels;
use crate::theme::Theme;
use crate::util::{fbm, hash, rgb, rotate, Color};

pub struct Temple;

fn ridge(p: &mut Pixels, base: f32, amp: f32, freq: f32, seed: f32, col: Color) {
    for x in 0..p.w {
        let y = base + amp * (fbm(x as f32 * freq + seed) - 0.5) * 2.0;
        p.rect(x as f32, y, x as f32 + 1.0, p.h as f32, col, 1.0);
    }
}

fn pine(p: &mut Pixels, t: f32, x: f32, base: f32, height: f32, phase: f32, s: f32) {
    let col = rgb(8.0, 10.0, 22.0);
    p.line(x, base, x, base - height * 0.5, 1.4 * s, col, 1.0);
    for i in 0..6 {
        let f = i as f32 / 6.0;
        let ty = base - height * (0.22 + 0.13 * i as f32);
        let wi = height * 0.30 * (1.0 - f * 0.78);
        let sway = (t * 1.1 + phase).sin() * 0.9 * s * (f + 0.25)
            + (t * 2.3 + phase * 2.0).sin() * 0.35 * s * (f + 0.2);
        p.poly(
            &[
                (x + sway - wi, ty),
                (x + sway + wi, ty),
                (x + sway * 1.2, ty - height * 0.2),
            ],
            col,
            1.0,
        );
    }
}

impl Scene for Temple {
    fn name(&self) -> &'static str {
        "temple"
    }

    fn blurb(&self) -> &'static str {
        "Pagoda at dusk with wind, lanterns, fireflies and petals"
    }

    fn draw(&self, p: &mut Pixels, t: f32, _theme: Option<&Theme>) {
        let (w, h) = (p.w as f32, p.h as f32);
        let s = h / 80.0;

        p.gradient(&[
            (0.0, rgb(12.0, 14.0, 42.0)),
            (0.45, rgb(58.0, 34.0, 86.0)),
            (0.72, rgb(236.0, 128.0, 96.0)),
            (1.0, rgb(60.0, 30.0, 70.0)),
        ]);

        // stars
        for i in 0..90u32 {
            let x = hash(i * 3 + 1) * w;
            let y = hash(i * 3 + 2) * h * 0.5;
            let tw = 0.5 + 0.5 * (t * (1.0 + hash(i * 3 + 3) * 2.0) + i as f32).sin();
            let a = (0.25 + 0.6 * tw) * (1.0 - y / (h * 0.5));
            p.blend(x as i32, y as i32, rgb(255.0, 250.0, 235.0), a);
        }

        // moon
        let (mx, my, mr) = (w * 0.74, h * 0.26, 8.5 * s);
        p.glow(mx, my, mr * 4.2, rgb(255.0, 190.0, 140.0), 0.5);
        p.circle(mx, my, mr, rgb(255.0, 243.0, 214.0), 1.0);
        p.circle(
            mx - mr * 0.3,
            my - mr * 0.2,
            mr * 0.28,
            rgb(240.0, 222.0, 192.0),
            0.7,
        );
        p.circle(
            mx + mr * 0.35,
            my + mr * 0.3,
            mr * 0.2,
            rgb(240.0, 222.0, 192.0),
            0.6,
        );

        // clouds
        for k in 0..5u32 {
            let sp = 0.6 + k as f32 * 0.35;
            let span = w + 80.0 * s;
            let x = (hash(k * 11) * span + t * sp * s * 1.3) % span - 40.0 * s;
            let y = h * (0.14 + 0.1 * k as f32);
            for j in 0..3 {
                let jf = j as f32;
                p.ellipse(
                    x + jf * 9.0 * s,
                    y + (jf - 1.0).abs() * 0.8 * s,
                    (11.0 - jf * 2.0) * s,
                    1.5 * s,
                    rgb(122.0, 74.0, 122.0),
                    0.45,
                );
            }
        }

        ridge(
            p,
            h * 0.68,
            4.0 * s,
            0.02 / s * 2.0,
            3.0,
            rgb(92.0, 52.0, 104.0),
        );
        mist(p, t, 0.64, 0.80, rgb(235.0, 150.0, 120.0), 0.35);
        ridge(
            p,
            h * 0.75,
            5.0 * s,
            0.025 / s * 2.0,
            40.0,
            rgb(48.0, 30.0, 72.0),
        );

        // pagoda
        let pcx = w * 0.34;
        let base = h * 0.84;
        let col = rgb(30.0, 20.0, 48.0);
        let mut y = base;
        p.poly(
            &[
                (pcx - 18.0 * s, y),
                (pcx - 14.5 * s, y - 2.6 * s),
                (pcx + 14.5 * s, y - 2.6 * s),
                (pcx + 18.0 * s, y),
            ],
            col,
            1.0,
        );
        y -= 2.6 * s;
        for i in 0..5 {
            let fi = i as f32;
            let bw = (15.0 - fi * 2.2) * s;
            let bh = (5.6 - fi * 0.35) * s;
            p.rect(pcx - bw / 2.0, y - bh, pcx + bw / 2.0, y, col, 1.0);
            let flick = 0.7 + 0.3 * (t * 7.0 + fi * 3.0).sin() * (t * 3.1 + fi).cos();
            p.rect(
                pcx - bw * 0.17,
                y - bh * 0.8,
                pcx + bw * 0.17,
                y - bh * 0.3,
                rgb(255.0, 178.0, 86.0),
                0.9 * flick,
            );
            p.glow(
                pcx,
                y - bh * 0.55,
                5.0 * s,
                rgb(255.0, 150.0, 60.0),
                0.25 * flick,
            );
            y -= bh;
            let ew = bw * 0.5 + 4.4 * s;
            let rh = 3.8 * s;
            p.poly(
                &[
                    (pcx - ew, y - 2.0 * s),
                    (pcx - ew * 0.75, y + 0.6 * s),
                    (pcx + ew * 0.75, y + 0.6 * s),
                    (pcx + ew, y - 2.0 * s),
                    (pcx + ew * 0.55, y - rh * 0.55),
                    (pcx + bw * 0.3, y - rh),
                    (pcx - bw * 0.3, y - rh),
                    (pcx - ew * 0.55, y - rh * 0.55),
                ],
                col,
                1.0,
            );
            y -= rh * 0.85;
        }
        p.line(pcx, y, pcx, y - 8.0 * s, 0.9 * s, col, 1.0);
        for k in 0..4 {
            p.rect(
                pcx - 1.3 * s,
                y - (1.5 + k as f32 * 1.6) * s,
                pcx + 1.3 * s,
                y - (1.9 + k as f32 * 1.6) * s,
                col,
                1.0,
            );
        }

        // banner on a pole, fluttering in the wind
        let bx = pcx + 25.0 * s;
        p.line(bx, base, bx, base - 24.0 * s, 0.9 * s, col, 1.0);
        for j in 0..16 {
            let jf = j as f32;
            let off = (t * 3.0 + jf * 0.5).sin() * jf * 0.10 * s + jf * 0.12 * s;
            let yy = base - 23.0 * s + jf * 0.95 * s;
            p.rect(
                bx + 0.6 * s + off,
                yy,
                bx + 3.6 * s + off,
                yy + 1.0 * s,
                rgb(176.0, 38.0, 52.0),
                1.0,
            );
        }

        // hanging lantern on a post
        let post = w * 0.62;
        let top = base - 20.0 * s;
        p.line(post, base, post, top, 1.2 * s, col, 1.0);
        p.line(post, top, post + 7.0 * s, top, 1.0 * s, col, 1.0);
        let pivot = (post + 7.0 * s, top);
        let ang = 0.16 * (t * 1.4).sin() + 0.05 * (t * 3.0).sin();
        let lc = rotate((pivot.0, pivot.1 + 7.5 * s), pivot, ang);
        let top_pt = rotate((pivot.0, pivot.1 + 3.0 * s), pivot, ang);
        p.line(pivot.0, pivot.1, top_pt.0, top_pt.1, 0.5 * s, col, 1.0);
        p.glow(lc.0, lc.1, 12.0 * s, rgb(255.0, 130.0, 50.0), 0.45);
        p.ellipse(lc.0, lc.1, 2.4 * s, 3.2 * s, rgb(255.0, 118.0, 56.0), 1.0);
        for k in [-1.0f32, 0.0, 1.0] {
            p.rect(
                lc.0 - 2.4 * s,
                lc.1 + k * 1.3 * s,
                lc.0 + 2.4 * s,
                lc.1 + k * 1.3 * s + 0.35 * s,
                rgb(120.0, 40.0, 30.0),
                0.7,
            );
        }

        // ground
        let ground = rgb(13.0, 9.0, 25.0);
        let gh =
            |x: f32| h * 0.845 + 1.6 * s * (x / (16.0 * s)).sin() + 0.8 * s * (x / (5.0 * s)).sin();
        for x in 0..p.w {
            let xf = x as f32;
            p.rect(xf, gh(xf), xf + 1.0, h, ground, 1.0);
        }

        // pines
        pine(p, t, w * 0.07, gh(w * 0.07) + 1.0, 34.0 * s, 0.0, s);
        pine(p, t, w * 0.14, gh(w * 0.14) + 1.0, 22.0 * s, 1.7, s);
        pine(p, t, w * 0.91, gh(w * 0.91) + 1.0, 30.0 * s, 3.1, s);
        pine(p, t, w * 0.82, gh(w * 0.82) + 1.0, 20.0 * s, 4.4, s);

        // grass
        let mut x = 0.0f32;
        while x < w {
            let hh = (2.0 + 3.0 * hash(x as u32 + 99)) * s;
            let sway = ((t * 2.0 + x * 0.12).sin() + 0.4 * (t * 4.1 + x * 0.3).sin()) * 1.2 * s;
            let g = gh(x);
            p.line(x, g + 0.5 * s, x + sway, g - hh, 0.8 * s, ground, 1.0);
            x += 1.6 * s;
        }

        // fireflies
        for i in 0..14u32 {
            let f = i as f32;
            let x = w * (0.1 + 0.8 * hash(i * 5 + 1)) + (t * 0.6 + f).sin() * 6.0 * s;
            let y = h * (0.72 + 0.14 * hash(i * 5 + 2)) + (t * 0.8 + f * 2.0).cos() * 3.0 * s;
            let a = 0.5 + 0.5 * (t * 2.0 + f * 1.7).sin();
            p.glow(x, y, 3.2 * s, rgb(220.0, 255.0, 120.0), 0.7 * a);
            p.blend(x as i32, y as i32, rgb(250.0, 255.0, 190.0), a);
        }

        petals(p, t, 46, 7);
    }
}
