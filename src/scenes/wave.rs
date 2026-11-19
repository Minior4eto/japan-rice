//! A stylised great wave in the spirit of ukiyo-e prints: a curling wave with
//! foam claws, layered swell, Mount Fuji behind and small boats riding it.
//! Original drawing made with shapes in code.

use super::Scene;
use crate::canvas::Pixels;
use crate::theme::Theme;
use crate::util::{hash, rgb, Color};

pub struct Wave;

struct Layer {
    base: f32,
    amp: f32,
    f1: f32,
    sp1: f32,
    f2: f32,
    sp2: f32,
    ph: f32,
    col: Color,
}

fn layer_y(l: &Layer, x: f32, t: f32, s: f32) -> f32 {
    l.base
        + l.amp * s * (x * l.f1 / s + t * l.sp1 + l.ph).sin()
        + 0.5 * l.amp * s * (x * l.f2 / s - t * l.sp2 + l.ph * 2.0).sin()
}

fn draw_layer(p: &mut Pixels, l: &Layer, t: f32, s: f32, foam: bool) {
    let mut prev = layer_y(l, 0.0, t, s);
    for x in 0..p.w {
        let xf = x as f32;
        let y = layer_y(l, xf, t, s);
        p.rect(xf, y, xf + 1.0, p.h as f32, l.col, 1.0);
        p.blend(x as i32, y as i32, rgb(200.0, 226.0, 236.0), 0.55);
        if foam && (y - prev) < -0.5 * s && hash(x as u32 * 7 + (t * 6.0) as u32) > 0.55 {
            p.blend(x as i32, y as i32 - 1, rgb(245.0, 250.0, 250.0), 0.9);
        }
        prev = y;
    }
}

impl Scene for Wave {
    fn name(&self) -> &'static str {
        "wave"
    }

    fn blurb(&self) -> &'static str {
        "A great wave curling over the sea, Fuji and fishing boats"
    }

    fn draw(&self, p: &mut Pixels, t: f32, _theme: Option<&Theme>) {
        let (w, h) = (p.w as f32, p.h as f32);
        let s = h / 80.0;
        let hz = h * 0.58;

        p.gradient(&[
            (0.0, rgb(58.0, 88.0, 130.0)),
            (0.32, rgb(156.0, 176.0, 178.0)),
            (hz / h, rgb(242.0, 228.0, 194.0)),
            (1.0, rgb(242.0, 228.0, 194.0)),
        ]);

        // sun and thin clouds
        let (sx, sy) = (w * 0.82, h * 0.2);
        p.glow(sx, sy, 16.0 * s, rgb(255.0, 150.0, 110.0), 0.35);
        p.circle(sx, sy, 6.5 * s, rgb(208.0, 62.0, 50.0), 1.0);
        for k in 0..4u32 {
            let span = w + 60.0 * s;
            let x = (hash(k + 5) * span + t * (0.5 + k as f32 * 0.25) * s) % span - 30.0 * s;
            p.ellipse(
                x,
                h * (0.12 + 0.07 * k as f32),
                14.0 * s,
                0.9 * s,
                rgb(250.0, 240.0, 224.0),
                0.6,
            );
        }

        // Fuji
        let (fx, peak) = (w * 0.68, h * 0.30);
        let half = 40.0 * s;
        let mut left = Vec::new();
        let mut right = Vec::new();
        for i in 0..=24 {
            let u = i as f32 / 24.0;
            let y = peak + (hz - peak) * u.powf(1.55);
            let dx = half * u.powf(0.92) + 0.8 * s;
            left.push((fx - dx, y));
            right.push((fx + dx, y));
        }
        let mut body = left.clone();
        body.extend(right.iter().rev());
        p.poly(&body, rgb(78.0, 104.0, 150.0), 1.0);
        // lighter right flank
        let mut flank = vec![(fx, peak)];
        flank.extend(right.iter());
        flank.push((fx + half * 0.15, hz));
        p.poly(&flank, rgb(96.0, 124.0, 168.0), 0.55);
        // snow cap with a jagged lower edge
        let cap = 7;
        let mut snow: Vec<(f32, f32)> = left[..=cap].to_vec();
        for (i, k) in (0..=cap).rev().enumerate() {
            let (rx, ry) = right[k];
            let zig = if i % 2 == 0 { 0.0 } else { 2.0 * s };
            snow.push((rx, ry + zig));
        }
        let mut snow_all = vec![];
        snow_all.extend(left[..=cap].iter().cloned());
        snow_all.extend(right[..=cap].iter().rev().cloned());
        let _ = snow;
        p.poly(&snow_all, rgb(240.0, 242.0, 246.0), 1.0);

        // far sea
        p.rect(0.0, hz, w, h, rgb(34.0, 78.0, 124.0), 1.0);

        let layers = [
            Layer {
                base: hz + 0.05 * h,
                amp: 0.8,
                f1: 0.30,
                sp1: 0.9,
                f2: 0.7,
                sp2: 0.6,
                ph: 0.0,
                col: rgb(40.0, 88.0, 136.0),
            },
            Layer {
                base: hz + 0.11 * h,
                amp: 1.2,
                f1: 0.22,
                sp1: 0.8,
                f2: 0.55,
                sp2: 0.7,
                ph: 1.3,
                col: rgb(30.0, 74.0, 124.0),
            },
            Layer {
                base: hz + 0.17 * h,
                amp: 1.6,
                f1: 0.18,
                sp1: 0.7,
                f2: 0.45,
                sp2: 0.9,
                ph: 2.1,
                col: rgb(22.0, 60.0, 112.0),
            },
        ];
        let near = [
            Layer {
                base: hz + 0.27 * h,
                amp: 2.2,
                f1: 0.15,
                sp1: 0.9,
                f2: 0.4,
                sp2: 0.8,
                ph: 3.0,
                col: rgb(14.0, 46.0, 98.0),
            },
            Layer {
                base: hz + 0.35 * h,
                amp: 2.6,
                f1: 0.12,
                sp1: 0.8,
                f2: 0.33,
                sp2: 1.0,
                ph: 4.2,
                col: rgb(9.0, 32.0, 78.0),
            },
        ];
        for l in &layers {
            draw_layer(p, l, t, s, true);
        }

        // boats riding the swell
        for (i, bxf) in [0.74f32, 0.90].iter().enumerate() {
            let bx = w * bxf;
            let by = layer_y(&layers[2], bx, t, s) - 0.5 * s + (t * 1.2 + i as f32).sin() * 0.5 * s;
            let hull = rgb(16.0, 20.0, 38.0);
            let k = 1.5 * s;
            let tilt = (t * 0.9 + i as f32 * 2.0).sin() * 0.5 * k;
            p.poly(
                &[
                    (bx - 6.0 * k, by - 2.4 * k - tilt),
                    (bx - 4.2 * k, by - 0.5 * k),
                    (bx + 4.2 * k, by - 0.5 * k),
                    (bx + 6.0 * k, by - 2.4 * k + tilt),
                    (bx + 3.6 * k, by + 0.9 * k),
                    (bx - 3.6 * k, by + 0.9 * k),
                ],
                hull,
                1.0,
            );
            for j in 0..3 {
                let rx = bx - 2.4 * k + j as f32 * 2.4 * k;
                p.rect(
                    rx - 0.45 * k,
                    by - 2.4 * k,
                    rx + 0.45 * k,
                    by - 0.5 * k,
                    hull,
                    1.0,
                );
                p.circle(rx, by - 2.9 * k, 0.55 * k, hull, 1.0);
                p.line(
                    rx,
                    by - 1.4 * k,
                    rx + 1.6 * k,
                    by + 0.4 * k,
                    0.4 * k,
                    hull,
                    1.0,
                );
            }
        }

        // the great wave
        let bob = (t * 0.7).sin();
        let r1 = 0.50 * h * (1.0 + 0.03 * bob);
        let c1 = (w * 0.27, h * 0.84 - 0.014 * h * bob);
        let r2 = 0.37 * h * (1.0 + 0.04 * bob);
        let c2 = (c1.0 + 0.12 * h, c1.1 + 0.075 * h);
        let (x0, x1) = ((c1.0 - r1).floor() as i32, (c1.0 + r1).ceil() as i32);
        let (y0, y1) = ((c1.1 - r1).floor() as i32, (c1.1 + r1).ceil() as i32);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
                let d1 = ((fx - c1.0).powi(2) + (fy - c1.1).powi(2)).sqrt();
                if d1 > r1 {
                    continue;
                }
                let d2 = ((fx - c2.0).powi(2) + (fy - c2.1).powi(2)).sqrt();
                if d2 < r2 {
                    continue;
                }
                let depth = ((r1 - d1) / (r1 - 0.5 * r2).max(1.0)).clamp(0.0, 1.0);
                let mut col = rgb(10.0, 40.0, 92.0).mix(rgb(24.0, 74.0, 130.0), 1.0 - depth);
                let ang = (fy - c1.1).atan2(fx - c1.0);
                let streak =
                    ((ang * 16.0 + d1 * 0.45 / s * 2.0 - t * 0.6).sin() * 0.5 + 0.5).powi(4);
                col = col.mix(
                    rgb(120.0, 176.0, 206.0),
                    streak * 0.22 * (1.0 - depth * 0.6),
                );
                if d2 - r2 < 2.4 * s {
                    col = col.mix(
                        rgb(176.0, 214.0, 230.0),
                        0.8 * (1.0 - (d2 - r2) / (2.4 * s)),
                    );
                }
                p.blend(x, y, col, 1.0);
            }
        }
        // foam claws along the crest
        let n = 72;
        for k in 0..n {
            let kf = k as f32 / (n - 1) as f32;
            let th = -2.95 + kf * 2.8;
            let (st, ct) = th.sin_cos();
            let pt = (c1.0 + r1 * ct, c1.1 + r1 * st);
            let top_w = 1.0 - ((kf - 0.55).abs() * 1.6).min(1.0) * 0.6;
            let dir = (-st * 0.75 + ct * 0.55, ct * 0.75 + st * 0.55);
            let len = (2.2 + 5.2 * (0.5 + 0.5 * (t * 1.1 + k as f32 * 0.9).sin())) * s * top_w;
            let tip = (pt.0 + dir.0 * len, pt.1 + dir.1 * len);
            let foam = rgb(244.0, 248.0, 250.0);
            p.line(
                pt.0 - dir.0 * 1.5 * s,
                pt.1 - dir.1 * 1.5 * s,
                tip.0,
                tip.1,
                1.3 * s,
                foam,
                0.95,
            );
            p.circle(tip.0, tip.1, 1.2 * s, foam, 0.95);
            // droplets thrown off the claws
            let fall = ((t * 0.9 + hash(k as u32 + 40) * 4.0) % 1.0) * 5.0 * s;
            p.blend(
                (tip.0 + dir.0 * fall * 0.6) as i32,
                (tip.1 + fall) as i32,
                foam,
                0.8 * (1.0 - fall / (5.0 * s)),
            );
        }

        for l in &near {
            draw_layer(p, l, t, s, true);
        }
    }
}
