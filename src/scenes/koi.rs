//! A koi pond seen from above. Four koi glide on wandering paths with
//! undulating bodies, flapping fins and soft shadows on the pond floor. Light
//! caustics ripple across the bottom, rings spread where a fish touched the
//! surface, and lily pads with lotus flowers drift on top.
//!
//! Every colour is derived from a theme: the pond's own palette when colours
//! are fixed, or your terminal theme with --sync / --theme.

use super::{petals_with, Scene};
use crate::canvas::Pixels;
use crate::theme::Theme;
use crate::util::{hash, noise, noise2, rgb, smoothstep, Color};
use std::f32::consts::TAU;

pub struct Koi;

#[derive(Clone, Copy)]
enum Kind {
    /// White with red patches.
    Kohaku,
    /// Metallic gold.
    Ogon,
    /// White with red patches and black spots.
    Sanke,
    /// Orange with a pale head.
    Hi,
}

struct Fish {
    seed: u32,
    /// Body length in scene units (the scene is 80 units tall).
    len: f32,
    speed: f32,
    /// 0 = just under the surface, 1 = deep (fainter, smaller shadow offset).
    depth: f32,
    kind: Kind,
}

const SCHOOL: [Fish; 4] = [
    Fish {
        seed: 51,
        len: 21.0,
        speed: 1.30,
        depth: 0.55,
        kind: Kind::Hi,
    },
    Fish {
        seed: 23,
        len: 25.0,
        speed: 1.15,
        depth: 0.35,
        kind: Kind::Ogon,
    },
    Fish {
        seed: 37,
        len: 31.0,
        speed: 0.85,
        depth: 0.15,
        kind: Kind::Sanke,
    },
    Fish {
        seed: 11,
        len: 28.0,
        speed: 1.00,
        depth: 0.0,
        kind: Kind::Kohaku,
    },
];

/// Lily pads: position (fraction of width / height), radius (units), lotus.
const PADS: [(f32, f32, f32, bool); 7] = [
    (0.09, 0.20, 10.0, true),
    (0.89, 0.15, 12.0, false),
    (0.81, 0.85, 9.0, true),
    (0.17, 0.84, 11.5, false),
    (0.53, 0.06, 6.0, false),
    (0.97, 0.56, 7.5, false),
    (0.03, 0.55, 6.5, false),
];

/// The pond's colours, all derived from a theme.
struct Pal {
    deep: Color,
    shallow: Color,
    caustic: Color,
    floor: Color,
    white: Color,
    red: Color,
    orange: Color,
    gold: Color,
    black: Color,
    fin: Color,
    eye: Color,
    pad: Color,
    pad_light: Color,
    lotus: Color,
    lotus_light: Color,
    pollen: Color,
}

impl Pal {
    fn from(th: &Theme) -> Pal {
        let a = |i: usize| th.ansi[i];
        let deep = th.bg.mix(a(4), 0.30);
        let shallow = th.bg.mix(a(6), 0.40).mix(a(12), 0.15);
        let white = th.fg.mix(a(15), 0.5).mix(rgb(255.0, 255.0, 255.0), 0.2);
        Pal {
            deep,
            shallow,
            caustic: a(14).mix(th.fg, 0.45),
            floor: deep.scale(0.75),
            white,
            red: a(1).mix(a(9), 0.35),
            orange: a(1).mix(a(3), 0.5).mix(a(11), 0.2),
            gold: a(11).mix(a(3), 0.3).mix(th.fg, 0.1),
            black: a(0).mix(th.bg, 0.2).scale(0.5),
            fin: white,
            eye: th.bg.scale(0.3),
            pad: a(2).mix(th.bg, 0.4),
            pad_light: a(10).mix(a(2), 0.3),
            lotus: a(13).mix(a(5), 0.3),
            lotus_light: a(13).mix(th.fg, 0.55),
            pollen: a(11),
        }
    }
}

type P = (f32, f32);

fn add(a: P, b: P) -> P {
    (a.0 + b.0, a.1 + b.1)
}

fn mul(a: P, k: f32) -> P {
    (a.0 * k, a.1 * k)
}

fn turn(v: P, ang: f32) -> P {
    let (s, c) = ang.sin_cos();
    (v.0 * c - v.1 * s, v.0 * s + v.1 * c)
}

/// Where a fish's head is at time t: a smooth wandering loop over the pond.
fn head(f: &Fish, t: f32, w: f32, h: f32) -> P {
    let r = |k: u32| hash(f.seed * 31 + k) * TAU;
    let u = t * f.speed * 0.30;
    let x = 0.5 + 0.34 * (u * 0.9 + r(1)).sin() + 0.10 * (u * 2.3 + r(2)).sin();
    let y = 0.5 + 0.31 * (u * 1.3 + r(3)).sin() + 0.09 * (u * 2.9 + r(4)).sin();
    (x * w, y * h)
}

const SEG: usize = 26;

/// Points along the body, evenly spaced along the path the head just swam,
/// so the body follows the head through every turn.
fn spine(f: &Fish, t: f32, w: f32, h: f32, len: f32) -> Vec<P> {
    let spacing = len / (SEG - 1) as f32;
    let mut pts = Vec::with_capacity(SEG);
    let mut prev = head(f, t, w, h);
    pts.push(prev);
    let (mut acc, mut tt) = (0.0f32, t);
    for _ in 0..4000 {
        if pts.len() >= SEG {
            break;
        }
        tt -= 0.01;
        let p = head(f, tt, w, h);
        let d = ((p.0 - prev.0).powi(2) + (p.1 - prev.1).powi(2)).sqrt();
        acc += d;
        while acc >= spacing && pts.len() < SEG {
            let k = (d - (acc - spacing)) / d.max(1e-6);
            pts.push((prev.0 + (p.0 - prev.0) * k, prev.1 + (p.1 - prev.1) * k));
            acc -= spacing;
        }
        prev = p;
    }
    while pts.len() < SEG {
        let n = pts.len();
        let a = pts[n - 1];
        let b = if n > 1 { pts[n - 2] } else { (a.0 + 1.0, a.1) };
        let d = ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt().max(1e-4);
        pts.push((
            a.0 + (a.0 - b.0) / d * spacing,
            a.1 + (a.1 - b.1) / d * spacing,
        ));
    }
    pts
}

/// Half-width of the body along its length (0 = nose, 1 = tail).
fn profile(f: f32) -> f32 {
    if f < 0.2 {
        0.55 + 0.45 * smoothstep(0.0, 0.2, f)
    } else {
        1.0 - 0.8 * smoothstep(0.2, 1.0, f)
    }
}

struct Body {
    c: Vec<P>,
    dir: Vec<P>,
    nrm: Vec<P>,
    r: Vec<f32>,
    len: f32,
    phase: f32,
}

fn body(f: &Fish, t: f32, w: f32, h: f32, s: f32) -> Body {
    let len = f.len * s;
    let sp = spine(f, t, w, h, len);
    let n = sp.len();
    let phase = t * 6.0 * f.speed;
    let amp = 0.06 * len;
    let (mut c, mut dir, mut nrm, mut r) = (vec![], vec![], vec![], vec![]);
    for k in 0..n {
        let a = sp[k.saturating_sub(1)];
        let b = sp[(k + 1).min(n - 1)];
        let (dx, dy) = (a.0 - b.0, a.1 - b.1);
        let l = (dx * dx + dy * dy).sqrt().max(1e-4);
        let d = (dx / l, dy / l);
        let nm = (-d.1, d.0);
        let fk = k as f32 / (n - 1) as f32;
        let off = amp * fk.powf(1.4) * (phase - fk * 4.2).sin();
        c.push(add(sp[k], mul(nm, off)));
        dir.push(d);
        nrm.push(nm);
        r.push(0.115 * len * profile(fk));
    }
    Body {
        c,
        dir,
        nrm,
        r,
        len,
        phase,
    }
}

fn outline(b: &Body, scale: f32, off: P) -> Vec<P> {
    let n = b.c.len();
    let mut pts = Vec::with_capacity(2 * n);
    for k in 0..n {
        pts.push(add(add(b.c[k], mul(b.nrm[k], b.r[k] * scale)), off));
    }
    for k in (0..n).rev() {
        pts.push(add(add(b.c[k], mul(b.nrm[k], -b.r[k] * scale)), off));
    }
    pts
}

fn tail_fin(b: &Body, off: P) -> Vec<P> {
    let n = b.c.len();
    let base = add(b.c[n - 1], off);
    let back = mul(b.dir[n - 1], -1.0);
    let nm = b.nrm[n - 1];
    let wag = (b.phase - 4.6).sin() * 0.55;
    let (fl, spread) = (0.24 * b.len, 0.15 * b.len);
    let tip_l = add(base, turn(add(mul(back, fl), mul(nm, spread)), wag));
    let tip_r = add(base, turn(add(mul(back, fl), mul(nm, -spread)), wag));
    let notch = add(base, turn(mul(back, fl * 0.55), wag));
    let rr = b.r[n - 1];
    vec![
        add(base, mul(nm, rr)),
        tip_l,
        notch,
        tip_r,
        add(base, mul(nm, -rr)),
    ]
}

fn side_fin(b: &Body, at: f32, size: f32, flap_ofs: f32, side: f32, off: P) -> Vec<P> {
    let n = b.c.len();
    let k = ((n - 1) as f32 * at) as usize;
    let k2 = (k + 3).min(n - 1);
    let flap = 0.7 + 0.3 * (b.phase * 0.5 + flap_ofs + side).sin();
    let e = add(add(b.c[k], mul(b.nrm[k], side * b.r[k] * 0.85)), off);
    let tip = add(
        e,
        add(
            mul(b.nrm[k], side * size * b.len * flap),
            mul(b.dir[k], -0.6 * size * b.len),
        ),
    );
    let back = add(add(b.c[k2], mul(b.nrm[k2], side * b.r[k2] * 0.8)), off);
    vec![e, tip, back]
}

fn shadow(p: &mut Pixels, f: &Fish, b: &Body, pal: &Pal) {
    let k = 1.0 - f.depth * 0.6;
    let off = (b.len * 0.10 * k, b.len * 0.15 * k);
    let col = pal.deep.scale(0.4);
    let a = 0.42 * (1.0 - f.depth * 0.4);
    p.poly(&outline(b, 1.05, off), col, a);
    p.poly(&tail_fin(b, off), col, a * 0.7);
    p.circle(b.c[0].0 + off.0, b.c[0].1 + off.1, b.r[0], col, a);
}

fn fish(p: &mut Pixels, f: &Fish, b: &Body, t: f32, pal: &Pal) {
    let a = 1.0 - f.depth * 0.3;
    let sink = |c: Color| c.mix(pal.deep, f.depth * 0.35);
    let (base, patch, spots) = match f.kind {
        Kind::Kohaku => (pal.white, Some(pal.red), false),
        Kind::Sanke => (pal.white, Some(pal.red), true),
        Kind::Ogon => (pal.gold, None, false),
        Kind::Hi => (pal.orange, Some(pal.red.mix(pal.orange, 0.3)), false),
    };
    let (base, fin) = (sink(base), sink(base.mix(pal.fin, 0.55)));
    let none = (0.0, 0.0);

    // fins first, so the body covers their roots
    for side in [-1.0f32, 1.0] {
        p.poly(&side_fin(b, 0.18, 0.17, 0.0, side, none), fin, 0.6 * a);
        p.poly(&side_fin(b, 0.55, 0.09, 1.7, side, none), fin, 0.5 * a);
    }
    p.poly(&tail_fin(b, none), fin, 0.65 * a);

    // body: darker rim, then the lighter core, gives it some roundness
    p.poly(&outline(b, 1.0, none), base.scale(0.72), a);
    p.circle(b.c[0].0, b.c[0].1, b.r[0], base.scale(0.72), a);
    p.poly(&outline(b, 0.8, none), base, a);
    p.circle(b.c[0].0, b.c[0].1, b.r[0] * 0.8, base, a);

    let n = b.c.len();
    let sf = f.seed as f32;
    if let Some(pc) = patch {
        let pc = sink(pc);
        let thr = if matches!(f.kind, Kind::Hi) {
            0.62
        } else {
            0.48
        };
        for k in 1..n - 3 {
            let fk = k as f32 / (n - 1) as f32;
            let v = noise(fk * 3.4 + sf * 1.7);
            if v > thr {
                let lat = (noise(fk * 5.0 + sf * 0.9 + 20.0) - 0.5) * 0.5;
                let rr = b.r[k] * 0.74 * (0.6 + 0.4 * smoothstep(thr, thr + 0.12, v));
                let c = add(b.c[k], mul(b.nrm[k], lat * b.r[k]));
                p.circle(c.0, c.1, rr, pc, a);
            }
        }
    }
    if matches!(f.kind, Kind::Hi) {
        // pale head
        for k in 0..4 {
            p.circle(
                b.c[k].0,
                b.c[k].1,
                b.r[k] * 0.7,
                sink(pal.white.mix(pal.orange, 0.35)),
                a * (1.0 - k as f32 * 0.2),
            );
        }
    }
    if spots {
        for k in 3..n - 5 {
            if hash(f.seed * 97 + k as u32) > 0.8 {
                let side = hash(f.seed * 13 + k as u32) - 0.5;
                let c = add(b.c[k], mul(b.nrm[k], side * b.r[k] * 0.9));
                p.circle(c.0, c.1, (b.r[k] * 0.3).max(0.6), sink(pal.black), a);
            }
        }
    }

    // back highlight, and a glint running down the gold koi
    let shine = if matches!(f.kind, Kind::Ogon) {
        0.4
    } else {
        0.2
    };
    let glint = ((t * 0.6 + sf) % 1.0) * n as f32;
    for k in 1..(n * 7 / 10) {
        let g = (1.0 - ((k as f32 - glint).abs() / 3.0)).max(0.0);
        let s = if matches!(f.kind, Kind::Ogon) {
            shine + 0.45 * g
        } else {
            shine
        };
        p.circle(
            b.c[k].0,
            b.c[k].1,
            (b.r[k] * 0.2).max(0.55),
            pal.white,
            s * a,
        );
    }

    // eyes
    for side in [-1.0f32, 1.0] {
        let e = add(
            add(b.c[1], mul(b.nrm[1], side * b.r[1] * 0.62)),
            mul(b.dir[1], b.r[1] * 0.2),
        );
        p.circle(e.0, e.1, (b.len * 0.018).max(0.55), pal.eye, a);
    }
}

fn ring(p: &mut Pixels, c: P, r: f32, col: Color, a: f32) {
    let n = ((TAU * r * 1.3) as usize).max(12);
    for i in 0..n {
        let ang = i as f32 / n as f32 * TAU;
        p.blend(
            (c.0 + r * ang.cos()) as i32,
            (c.1 + r * ang.sin()) as i32,
            col,
            a,
        );
    }
}

/// Every few seconds each koi touches the surface and leaves spreading rings.
fn ripples(p: &mut Pixels, t: f32, w: f32, h: f32, s: f32, pal: &Pal) {
    for f in &SCHOOL {
        let period = 4.5 + hash(f.seed + 5) * 3.5;
        let off = hash(f.seed + 6) * period;
        let tt = t + off;
        let cycle = (tt / period).floor();
        let age = tt - cycle * period;
        if age > 3.2 {
            continue;
        }
        let c = head(f, cycle * period - off, w, h);
        for (i, lag) in [0.0f32, 0.5, 1.0].iter().enumerate() {
            let a = age - lag;
            if a < 0.0 {
                continue;
            }
            let r = 1.0 + a * 6.5 * s;
            let fade = (1.0 - a / 2.4).max(0.0);
            ring(p, c, r, pal.caustic, 0.55 * fade / (1.0 + i as f32 * 0.6));
        }
    }
}

fn water(p: &mut Pixels, t: f32, s: f32, pal: &Pal) {
    let (w, h) = (p.w as f32, p.h as f32);
    let step = if p.w * p.h > 60_000 { 2 } else { 1 };
    for y in (0..p.h).step_by(step) {
        for x in (0..p.w).step_by(step) {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let (dx, dy) = ((fx / w - 0.5) * w / h, fy / h - 0.5);
            let d = (dx * dx + dy * dy).sqrt();
            let mut col = pal.shallow.mix(pal.deep, smoothstep(0.1, 0.85, d));
            col = col.mix(pal.floor, 0.4 * noise2(fx / (9.0 * s), fy / (9.0 * s)));
            // caustics: thin bright lines where two warped waves cross zero
            let (q, r) = (fx / (7.0 * s), fy / (7.0 * s));
            let n = (q + 1.3 * (r * 0.8 + t * 0.55).sin() + t * 0.25).sin()
                + (r * 1.1 + 1.2 * (q * 0.7 - t * 0.45).sin() - t * 0.2).sin();
            let caus = (1.0 - (n * 0.9).abs().min(1.0)).powi(5);
            col = col.mix(pal.caustic, 0.3 * caus * (1.0 - 0.5 * d));
            // snap to small steps: invisible here, but neighbouring cells then share
            // colours, which keeps the output to the terminal small and fast
            let q4 = |v: f32| (v / 5.0).round() * 5.0;
            col = rgb(q4(col.r), q4(col.g), q4(col.b));
            for oy in 0..step {
                for ox in 0..step {
                    p.blend((x + ox) as i32, (y + oy) as i32, col, 1.0);
                }
            }
        }
    }
    // pebbles on the floor: soft, slightly lighter or darker than the floor
    for i in 0..60u32 {
        let x = hash(i * 7 + 1) * w;
        let y = hash(i * 7 + 2) * h;
        let rx = (0.8 + 1.6 * hash(i * 7 + 3)) * s;
        let ry = rx * (0.6 + 0.3 * hash(i * 7 + 4));
        let col = if hash(i * 7 + 5) > 0.5 {
            pal.floor.scale(0.85)
        } else {
            pal.shallow.mix(pal.white, 0.08)
        };
        p.ellipse(x, y, rx, ry, col, 0.14);
    }
}

fn pad_shape(c: P, r: f32, ang: f32) -> Vec<P> {
    let mut v = vec![c];
    let n = 30;
    for i in 0..=n {
        let a = ang + 0.3 + (TAU - 0.6) * i as f32 / n as f32;
        v.push((c.0 + r * a.cos(), c.1 + r * a.sin()));
    }
    v
}

fn lily_pads(p: &mut Pixels, t: f32, s: f32, pal: &Pal) {
    let (w, h) = (p.w as f32, p.h as f32);
    for (i, &(px, py, pr, lotus)) in PADS.iter().enumerate() {
        let fi = i as f32;
        let c = (
            px * w + (t * 0.07 + fi).sin() * 1.5 * s,
            py * h + (t * 0.05 + fi * 2.0).cos() * 1.2 * s,
        );
        let r = pr * s;
        let ang = fi * 1.9 + 0.12 * (t * 0.1 + fi).sin();
        p.poly(
            &pad_shape((c.0 + 0.12 * r, c.1 + 0.18 * r), r, ang),
            pal.deep.scale(0.35),
            0.4,
        );
        p.poly(&pad_shape(c, r, ang), pal.pad, 1.0);
        p.poly(
            &pad_shape(c, r * 0.8, ang),
            pal.pad.mix(pal.pad_light, 0.25),
            1.0,
        );
        for v in 1..8 {
            let a = ang + 0.3 + (TAU - 0.6) * v as f32 / 8.0;
            p.line(
                c.0,
                c.1,
                c.0 + r * 0.9 * a.cos(),
                c.1 + r * 0.9 * a.sin(),
                0.3 * s,
                pal.pad_light,
                0.3,
            );
        }
        if lotus {
            let bloom = 1.0 + 0.05 * (t * 0.8 + fi).sin();
            let lc = (c.0 + 0.15 * r, c.1 - 0.1 * r);
            let lr = r * 0.62 * bloom;
            for (layer, (count, size, col, rot)) in
                [(8, 1.0, pal.lotus, 0.0f32), (6, 0.66, pal.lotus_light, 0.5)]
                    .iter()
                    .enumerate()
            {
                for k in 0..*count {
                    let th = ang + rot + k as f32 * TAU / *count as f32 + layer as f32 * 0.2;
                    let rr = lr * size;
                    let side = |o: f32| {
                        (
                            lc.0 + rr * 0.5 * (th + o).cos(),
                            lc.1 + rr * 0.5 * (th + o).sin(),
                        )
                    };
                    let tip = (lc.0 + rr * th.cos(), lc.1 + rr * th.sin());
                    p.poly(&[lc, side(-0.34), tip, side(0.34)], *col, 1.0);
                    p.line(lc.0, lc.1, tip.0, tip.1, 0.25 * s, col.scale(0.8), 0.35);
                }
            }
            p.circle(lc.0, lc.1, lr * 0.2, pal.pollen, 1.0);
        }
    }
}

impl Scene for Koi {
    fn name(&self) -> &'static str {
        "koi"
    }

    fn blurb(&self) -> &'static str {
        "Koi gliding through a lily pond, coloured by your theme with --sync"
    }

    fn native_theme(&self) -> bool {
        true
    }

    fn draw(&self, p: &mut Pixels, t: f32, theme: Option<&Theme>) {
        let pond = Theme::pond();
        let pal = Pal::from(theme.unwrap_or(&pond));
        let (w, h) = (p.w as f32, p.h as f32);
        let s = h / 80.0;

        water(p, t, s, &pal);
        let bodies: Vec<Body> = SCHOOL.iter().map(|f| body(f, t, w, h, s)).collect();
        for (f, b) in SCHOOL.iter().zip(&bodies) {
            shadow(p, f, b, &pal);
        }
        for (f, b) in SCHOOL.iter().zip(&bodies) {
            fish(p, f, b, t, &pal);
        }
        ripples(p, t, w, h, s, &pal);
        lily_pads(p, t, s, &pal);
        petals_with(p, t * 0.5, 16, 77, pal.lotus, pal.lotus_light);
    }
}
