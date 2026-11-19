//! A Shinto shrine on a snowy night. A vermilion torii frames a lit shrine
//! hall; stone lanterns flicker and light the falling flakes; snow-laden cedars
//! bend in the gusts; spindrift skims the ground, clumps drop from branches,
//! icicles hang from the beams and the snow slowly piles up.

use super::{mist, Scene};
use crate::canvas::Pixels;
use crate::theme::Theme;
use crate::util::{fbm, hash, lerp, noise, noise2, rgb, smoothstep, Color};

pub struct Shrine;

const SNOW: Color = rgb(228.0, 234.0, 248.0);
const SNOW_SHADE: Color = rgb(150.0, 162.0, 198.0);
const LAMP: Color = rgb(255.0, 176.0, 90.0);
const LAMP_LIT: Color = rgb(255.0, 214.0, 160.0);
const ICE: Color = rgb(196.0, 222.0, 240.0);

/// How far the air has carried things sideways, in screen widths.
fn drift(t: f32) -> f32 {
    0.02 * t + 0.26 * fbm(t * 0.07 + 5.0)
}

/// Gust strength, roughly -1 (lull) to 1 (strong gust to the right).
fn gust(t: f32) -> f32 {
    let rate = (drift(t + 0.1) - drift(t - 0.1)) / 0.2;
    ((rate - 0.02) / 0.016).clamp(-1.2, 1.4)
}

/// Snow depth on ledges grows over the first few minutes.
fn pile(t: f32) -> f32 {
    0.35 + 0.65 * smoothstep(0.0, 300.0, t)
}

#[allow(clippy::too_many_arguments)]
fn cedar(
    p: &mut Pixels,
    t: f32,
    x: f32,
    base: f32,
    height: f32,
    phase: f32,
    s: f32,
    dark: Color,
    snow: Color,
    g: f32,
) {
    let trunk = rgb(34.0, 24.0, 26.0).mix(dark, 0.4);
    p.line(
        x,
        base,
        x,
        base - height * 0.4,
        (height * 0.03).max(1.2),
        trunk,
        1.0,
    );
    let tiers = 7;
    for i in 0..tiers {
        let f = i as f32 / tiers as f32;
        let ty = base - height * (0.16 + 0.115 * i as f32);
        let wi = height * 0.25 * (1.0 - f * 0.78);
        let bend = (g * 0.8 + 0.22 * (t * 0.9 + phase).sin() + 0.1 * (t * 2.1 + phase * 3.0).sin())
            * s
            * (0.3 + f * 1.7)
            * (height / (40.0 * s)).max(0.5);
        let top = (x + bend * 1.15, ty - height * 0.2);
        let l = (x + bend - wi, ty + height * 0.012);
        let r = (x + bend + wi, ty - height * 0.012 * g);
        p.poly(&[l, r, top], dark, 1.0);
        // a lighter needle texture on the lit side
        p.poly(
            &[
                (l.0 + wi * 0.25, l.1 - height * 0.01),
                (top.0 - wi * 0.05, top.1 + height * 0.06),
                (l.0 + wi * 0.6, l.1 - height * 0.02),
            ],
            dark.scale(1.35),
            0.6,
        );
        // snow lying on the upper faces, lit from the left
        let th = (s * 0.95 * (0.7 + 0.5 * (1.0 - f))).max(1.0);
        p.line(
            top.0,
            top.1 + th * 0.6,
            l.0 + wi * 0.18,
            l.1 - th * 0.7,
            th,
            snow,
            0.93,
        );
        p.line(
            top.0,
            top.1 + th * 0.6,
            r.0 - wi * 0.2,
            r.1 - th * 0.7,
            th * 0.85,
            snow.mix(SNOW_SHADE, 0.45),
            0.85,
        );
        p.ellipse(
            l.0 + wi * 0.22,
            l.1 - th * 0.45,
            wi * 0.26,
            th * 0.75,
            snow,
            0.92,
        );
        p.ellipse(
            r.0 - wi * 0.24,
            r.1 - th * 0.45,
            wi * 0.22,
            th * 0.65,
            snow.mix(SNOW_SHADE, 0.45),
            0.85,
        );
    }
}

/// The shrine hall (haiden): stone platform, lit shoji, vermilion pillars and
/// a sweeping roof with crossed chigi and katsuogi logs on the ridge.
fn shrine_hall(p: &mut Pixels, t: f32, cx: f32, by: f32, s: f32, acc: f32) {
    let stone = rgb(84.0, 86.0, 100.0);
    let wood = rgb(58.0, 26.0, 24.0);
    let pillar = rgb(168.0, 48.0, 34.0);
    let pillar_dark = rgb(104.0, 28.0, 24.0);
    let roofc = rgb(34.0, 32.0, 42.0);
    let thick = s * (0.9 + 1.5 * acc);

    // platform and steps
    p.rect(cx - 25.0 * s, by - 4.0 * s, cx + 25.0 * s, by, stone, 1.0);
    p.rect(
        cx - 25.0 * s,
        by - 1.2 * s,
        cx + 25.0 * s,
        by,
        stone.scale(0.7),
        1.0,
    );
    for k in 0..3 {
        let k = k as f32;
        let wv = 7.0 * s + k * 1.6 * s;
        p.rect(
            cx - wv,
            by - 4.0 * s + k * 1.4 * s,
            cx + wv,
            by - 2.6 * s + k * 1.4 * s,
            stone.scale(1.1 - k * 0.12),
            1.0,
        );
        p.rect(
            cx - wv,
            by - 4.0 * s + k * 1.4 * s - thick * 0.4,
            cx + wv,
            by - 4.0 * s + k * 1.4 * s + 0.3,
            SNOW,
            0.85,
        );
    }
    p.rect(
        cx - 25.5 * s,
        by - 4.0 * s - thick * 0.7,
        cx - 8.0 * s,
        by - 4.0 * s,
        SNOW,
        0.95,
    );
    p.rect(
        cx + 8.0 * s,
        by - 4.0 * s - thick * 0.7,
        cx + 25.5 * s,
        by - 4.0 * s,
        SNOW.mix(SNOW_SHADE, 0.2),
        0.95,
    );

    // walls and glowing shoji panels
    let (wy0, wy1) = (by - 17.0 * s, by - 4.0 * s);
    p.rect(cx - 19.0 * s, wy0, cx + 19.0 * s, wy1, wood, 1.0);
    let flick = 0.88 + 0.12 * fbm(t * 1.7 + 7.0);
    let warm = rgb(255.0, 200.0, 130.0).scale(flick);
    let gx = (2.2 * s).max(2.0);
    let gy = (2.6 * s).max(2.0);
    for k in 0..4 {
        let x0 = cx - 19.0 * s + k as f32 * 9.5 * s + 1.3 * s;
        let x1 = x0 + 9.5 * s - 2.6 * s;
        let (y0, y1) = (by - 15.5 * s, by - 5.5 * s);
        for y in y0.round() as i32..y1.round() as i32 {
            for x in x0.round() as i32..x1.round() as i32 {
                let lx = (x as f32 - x0).rem_euclid(gx) < 1.0;
                let ly = (y as f32 - y0).rem_euclid(gy) < 1.0;
                // a figure's shadow drifts behind one panel
                let shadow = k == 2
                    && ((x as f32 - (x0 + (x1 - x0) * (0.5 + 0.3 * (t * 0.13).sin()))).abs()
                        < 1.6 * s)
                    && (y as f32) > y0 + 3.0 * s;
                let c = if lx || ly {
                    wood.scale(1.3)
                } else if shadow {
                    warm.scale(0.55)
                } else {
                    warm.scale(0.85 + 0.15 * (1.0 - (y as f32 - y0) / (y1 - y0)))
                };
                p.blend(x, y, c, 1.0);
            }
        }
    }
    p.glow(cx, by - 10.0 * s, 26.0 * s, LAMP, 0.18 * flick);
    for k in 0..5 {
        let px = cx - 19.0 * s + k as f32 * 9.5 * s;
        p.rect(px - 1.0 * s, wy0, px + 1.0 * s, wy1, pillar, 1.0);
        p.rect(px + 0.2 * s, wy0, px + 1.0 * s, wy1, pillar_dark, 0.8);
    }
    p.rect(
        cx - 21.0 * s,
        wy0 - 2.0 * s,
        cx + 21.0 * s,
        wy0,
        pillar_dark,
        1.0,
    );
    p.rect(
        cx - 21.0 * s,
        wy0 - 2.0 * s,
        cx + 21.0 * s,
        wy0 - 1.4 * s,
        pillar,
        1.0,
    );

    // offering box, bell and rope
    p.rect(
        cx - 3.5 * s,
        by - 7.5 * s,
        cx + 3.5 * s,
        by - 4.0 * s,
        wood.scale(0.8),
        1.0,
    );
    for k in 0..4 {
        let x = cx - 3.0 * s + k as f32 * 2.0 * s;
        p.rect(
            x,
            by - 7.3 * s,
            x + 0.5 * s,
            by - 4.3 * s,
            wood.scale(1.6),
            0.8,
        );
    }
    let sway = 0.6 * s * (t * 1.3).sin() * (0.6 + 0.4 * gust(t).max(0.0));
    p.line(
        cx,
        wy0,
        cx + sway,
        by - 8.5 * s,
        0.9 * s,
        rgb(220.0, 60.0, 50.0),
        1.0,
    );
    p.line(
        cx + 0.3 * s,
        wy0,
        cx + sway + 0.3 * s,
        by - 8.5 * s,
        0.4 * s,
        rgb(240.0, 236.0, 226.0),
        0.8,
    );
    p.circle(cx, wy0 + 1.4 * s, 1.5 * s, rgb(210.0, 170.0, 80.0), 1.0);

    // roof: concave sweep with upturned eaves
    let ridge = by - 31.0 * s;
    let eave = by - 18.0 * s;
    let span = 31.0 * s;
    let prof = |u: f32| {
        let a = ((u.abs() - 0.22) / 0.78).max(0.0);
        ridge + (eave - ridge) * (1.0 - (1.0 - a).powi(2))
            - 1.3 * s * smoothstep(0.86, 1.0, u.abs())
    };
    let n = 40;
    let mut roof = Vec::with_capacity(n + 6);
    for k in 0..=n {
        let u = -1.0 + 2.0 * k as f32 / n as f32;
        roof.push((cx + u * span, prof(u)));
    }
    roof.push((cx + span - 0.6 * s, eave + 1.7 * s));
    roof.push((cx + 21.0 * s, wy0 - 1.5 * s));
    roof.push((cx - 21.0 * s, wy0 - 1.5 * s));
    roof.push((cx - span + 0.6 * s, eave + 1.7 * s));
    p.poly(&roof, roofc, 1.0);
    // tile ribs
    for k in 0..28 {
        let u = -0.96 + 1.92 * k as f32 / 27.0;
        let x = cx + u * span;
        p.line(
            x,
            prof(u) + thick * 0.8,
            x + u * 1.5 * s,
            prof(u) + 1.6 * s,
            0.5 * s,
            roofc.scale(1.6),
            0.5,
        );
    }
    p.line(
        cx - span + 0.6 * s,
        eave + 1.5 * s,
        cx + span - 0.6 * s,
        eave + 1.5 * s,
        0.6 * s,
        roofc.scale(2.0),
        0.6,
    );
    // snow blanket on the roof
    let mut cap = Vec::with_capacity(2 * n + 2);
    for k in 0..=n {
        let u = -1.0 + 2.0 * k as f32 / n as f32;
        let bump = 0.25 * s * noise(u * 9.0 + 3.0);
        cap.push((
            cx + u * span * 0.985,
            prof(u) - thick * (1.0 - 0.35 * u.abs()) - bump,
        ));
    }
    for k in (0..=n).rev() {
        let u = -1.0 + 2.0 * k as f32 / n as f32;
        cap.push((
            cx + u * span * 0.985,
            prof(u) + 0.8 * s + 0.5 * thick * (1.0 - u.abs()),
        ));
    }
    p.poly(&cap, SNOW, 0.97);
    // shaded right slope
    let mut shade = Vec::new();
    for k in n / 2..=n {
        let u = -1.0 + 2.0 * k as f32 / n as f32;
        shade.push((
            cx + u * span * 0.985,
            prof(u) - thick * (1.0 - 0.35 * u.abs()) * 0.4,
        ));
    }
    for k in (n / 2..=n).rev() {
        let u = -1.0 + 2.0 * k as f32 / n as f32;
        shade.push((
            cx + u * span * 0.985,
            prof(u) + 0.8 * s + 0.5 * thick * (1.0 - u.abs()),
        ));
    }
    p.poly(&shade, SNOW_SHADE, 0.45);
    // overhanging lips and icicles under the eaves
    for side in [-1.0f32, 1.0] {
        p.ellipse(
            cx + side * (span - 1.5 * s),
            eave - 0.6 * s,
            2.2 * s,
            thick * 0.7,
            SNOW,
            0.9,
        );
    }
    for k in 0..22 {
        let u = -0.92 + 1.84 * hash(k * 17 + 3);
        let x = cx + u * span;
        let len = (0.6 + 2.6 * hash(k * 17 + 4)) * s * (0.4 + acc);
        let y = prof(u) + 1.8 * s;
        p.line(x, y, x, y + len, (0.45 * s).max(1.0), ICE, 0.65);
    }
    // chigi and katsuogi on the ridge
    let rx = 0.22 * span;
    for side in [-1.0f32, 1.0] {
        let ex = cx + side * rx;
        p.line(
            ex - 2.0 * s,
            ridge + 1.5 * s,
            ex + 2.2 * s,
            ridge - 5.5 * s,
            1.0 * s,
            roofc.scale(1.4),
            1.0,
        );
        p.line(
            ex + 2.0 * s,
            ridge + 1.5 * s,
            ex - 2.2 * s,
            ridge - 5.5 * s,
            1.0 * s,
            roofc.scale(1.4),
            1.0,
        );
        p.ellipse(ex - 2.0 * s, ridge - 5.6 * s, 0.9 * s, 0.6 * s, SNOW, 0.9);
        p.ellipse(ex + 2.0 * s, ridge - 5.6 * s, 0.9 * s, 0.6 * s, SNOW, 0.9);
    }
    for k in -1..=1 {
        let x = cx + k as f32 * 4.0 * s;
        p.ellipse(
            x,
            ridge - thick - 0.6 * s,
            1.4 * s,
            0.8 * s,
            rgb(150.0, 110.0, 60.0),
            1.0,
        );
        p.ellipse(
            x,
            ridge - thick - 1.3 * s,
            1.3 * s,
            0.5 * s * (0.6 + acc),
            SNOW,
            0.95,
        );
    }
}

/// The great gate, with snow caps, icicles and a swinging shimenawa rope.
fn torii(p: &mut Pixels, t: f32, cx: f32, base: f32, s: f32, acc: f32, g: f32) {
    let red = rgb(198.0, 50.0, 34.0);
    let red_dark = rgb(118.0, 26.0, 22.0);
    let black = rgb(26.0, 22.0, 28.0);
    let half = 19.0 * s;
    let top = base - 33.0 * s;
    let pw = 2.8 * s;
    let thick = s * (0.8 + 1.4 * acc);

    for side in [-1.0f32, 1.0] {
        let px = cx + side * half;
        let lean = side * 0.9 * s;
        let (x0b, x1b) = (px - pw * 0.5, px + pw * 0.5);
        let (x0t, x1t) = (px - pw * 0.42 - lean, px + pw * 0.42 - lean);
        p.poly(
            &[(x0t, top), (x1t, top), (x1b, base), (x0b, base)],
            red,
            1.0,
        );
        let (mt, mb) = ((x0t + x1t) * 0.5 + pw * 0.08, (x0b + x1b) * 0.5 + pw * 0.08);
        p.poly(
            &[(mt, top), (x1t, top), (x1b, base), (mb, base)],
            red_dark,
            0.75,
        );
        p.poly(
            &[
                (x0t, top),
                (x0t + pw * 0.18, top),
                (x0b + pw * 0.18, base),
                (x0b, base),
            ],
            rgb(236.0, 104.0, 74.0),
            0.5,
        );
        p.rect(
            x0b - 0.35 * s,
            base - 3.2 * s,
            x1b + 0.35 * s,
            base,
            black,
            1.0,
        );
        p.ellipse(
            px,
            base - 0.2 * s,
            pw * 1.7,
            1.3 * s * (0.6 + acc),
            SNOW,
            0.95,
        );
    }

    // nuki
    let ny = top + 7.0 * s;
    let (nx0, nx1) = (cx - half - 4.8 * s, cx + half + 4.8 * s);
    p.rect(nx0, ny, nx1, ny + 2.0 * s, red, 1.0);
    p.rect(nx0, ny + 1.3 * s, nx1, ny + 2.0 * s, red_dark, 0.85);
    p.rect(nx0, ny - thick, nx1, ny + 0.2, SNOW, 0.95);
    p.ellipse(
        nx0 + 0.5 * s,
        ny - thick * 0.4,
        1.0 * s,
        thick * 0.6,
        SNOW,
        0.9,
    );
    p.ellipse(
        nx1 - 0.5 * s,
        ny - thick * 0.4,
        1.0 * s,
        thick * 0.6,
        SNOW,
        0.9,
    );
    // gakuzuka and plaque
    p.rect(cx - 1.2 * s, top - 0.5 * s, cx + 1.2 * s, ny, red, 1.0);
    p.rect(
        cx - 2.7 * s,
        top + 0.6 * s,
        cx + 2.7 * s,
        ny - 0.7 * s,
        black,
        1.0,
    );
    p.rect(
        cx - 2.1 * s,
        top + 1.2 * s,
        cx + 2.1 * s,
        ny - 1.3 * s,
        rgb(40.0, 34.0, 44.0),
        1.0,
    );
    for k in 0..3 {
        let y = top + 1.9 * s + k as f32 * 1.4 * s;
        p.rect(
            cx - 0.8 * s,
            y,
            cx + 0.8 * s,
            y + 0.6 * s,
            rgb(214.0, 178.0, 90.0),
            0.9,
        );
    }
    // shimaki
    p.rect(
        cx - half - 6.2 * s,
        top - 2.0 * s,
        cx + half + 6.2 * s,
        top,
        red,
        1.0,
    );
    p.rect(
        cx - half - 6.2 * s,
        top - 0.6 * s,
        cx + half + 6.2 * s,
        top,
        red_dark,
        0.8,
    );
    // kasagi with upturned ends
    let span = half + 9.5 * s;
    let th = 2.3 * s;
    let lift = 3.4 * s;
    let ky = top - 2.0 * s;
    let n = 30;
    let at = |u: f32| u.abs().powi(4);
    let mut beam = Vec::with_capacity(2 * n + 2);
    for k in 0..=n {
        let u = -1.0 + 2.0 * k as f32 / n as f32;
        beam.push((cx + u * span, ky - th - lift * at(u)));
    }
    for k in (0..=n).rev() {
        let u = -1.0 + 2.0 * k as f32 / n as f32;
        beam.push((cx + u * span * 0.96, ky - lift * 0.75 * at(u)));
    }
    p.poly(&beam, black, 1.0);
    let mut cap = Vec::with_capacity(2 * n + 2);
    for k in 0..=n {
        let u = -1.0 + 2.0 * k as f32 / n as f32;
        let sit = 1.0 - smoothstep(0.75, 1.0, u.abs());
        cap.push((
            cx + u * span * 0.98,
            ky - th - lift * at(u) - thick * sit - 0.3 * s * noise(u * 11.0) * sit,
        ));
    }
    for k in (0..=n).rev() {
        let u = -1.0 + 2.0 * k as f32 / n as f32;
        cap.push((cx + u * span * 0.98, ky - th - lift * at(u) + 0.6 * s));
    }
    p.poly(&cap, SNOW, 0.97);

    // icicles under the nuki
    for k in 0..16 {
        let u = hash(k * 31 + 7) * 2.0 - 1.0;
        let x = cx + u * (half + 4.0 * s);
        if (x - (cx - half)).abs() < pw || (x - (cx + half)).abs() < pw || (x - cx).abs() < 1.5 * s
        {
            continue;
        }
        let len = (0.5 + 2.4 * hash(k * 31 + 8)) * s * (0.4 + acc);
        p.line(
            x,
            ny + 2.0 * s,
            x,
            ny + 2.0 * s + len,
            (0.45 * s).max(1.0),
            ICE,
            0.7,
        );
        let glint = (t * 1.5 + k as f32 * 2.1).sin();
        if glint > 0.93 {
            p.add(
                x as i32,
                (ny + 2.2 * s) as i32,
                rgb(255.0, 255.0, 255.0),
                0.8,
            );
        }
    }

    // shimenawa rope with shide paper, swinging in the gusts
    let (rx0, rx1) = (cx - half + 1.6 * s, cx + half - 1.6 * s);
    let ry = ny + 2.3 * s;
    let sag = 3.0 * s * (1.0 + 0.12 * (t * 1.1).sin() + 0.12 * g);
    let swing = |u: f32| 0.7 * s * g * (1.0 - u * u) + 0.25 * s * (t * 1.7).sin() * (1.0 - u * u);
    let rope_th = 1.9 * s;
    let m = 40;
    for k in 0..m {
        let u0 = -1.0 + 2.0 * k as f32 / m as f32;
        let u1 = -1.0 + 2.0 * (k + 1) as f32 / m as f32;
        let x0 = lerp(rx0, rx1, (u0 + 1.0) * 0.5) + swing(u0);
        let x1 = lerp(rx0, rx1, (u1 + 1.0) * 0.5) + swing(u1);
        let y0 = ry + sag * (1.0 - u0 * u0);
        let y1 = ry + sag * (1.0 - u1 * u1);
        let thk = rope_th * (1.0 - 0.35 * u0.abs());
        let col = if k % 3 == 0 {
            rgb(150.0, 120.0, 70.0)
        } else {
            rgb(206.0, 176.0, 112.0)
        };
        p.line(x0, y0, x1, y1, thk, col, 1.0);
    }
    for (j, u) in [-0.62f32, -0.22, 0.22, 0.62].into_iter().enumerate() {
        let x = lerp(rx0, rx1, (u + 1.0) * 0.5) + swing(u);
        let y = ry + sag * (1.0 - u * u) + rope_th * 0.4;
        let ang =
            0.5 * g + 0.3 * (t * 3.1 + j as f32 * 1.7).sin() + 0.15 * (t * 7.3 + j as f32).sin();
        let seg = 1.5 * s;
        let (mut px, mut py) = (x, y);
        for z in 0..4 {
            let zig = if z % 2 == 0 { 0.9 * s } else { -0.9 * s };
            let nx = px + zig + ang * seg;
            let ny2 = py + seg;
            p.line(
                px,
                py,
                nx,
                ny2,
                (0.8 * s).max(1.0),
                rgb(242.0, 242.0, 236.0),
                0.95,
            );
            px = nx;
            py = ny2;
        }
    }
}

/// A stone lantern (toro). Returns the position of its flame and flicker.
fn toro(p: &mut Pixels, t: f32, x: f32, base: f32, s: f32, acc: f32, seed: f32) -> (f32, f32, f32) {
    let stone = rgb(100.0, 102.0, 118.0);
    let dark = rgb(56.0, 58.0, 74.0);
    let thick = s * (0.7 + 1.2 * acc);
    let flick = 0.78 + 0.22 * fbm(t * 5.5 + seed) - 0.08 * noise(t * 17.0 + seed);

    p.rect(x - 4.8 * s, base - 2.0 * s, x + 4.8 * s, base, dark, 1.0);
    p.rect(
        x - 3.6 * s,
        base - 3.4 * s,
        x + 3.6 * s,
        base - 2.0 * s,
        stone,
        1.0,
    );
    p.rect(
        x - 1.5 * s,
        base - 11.0 * s,
        x + 1.5 * s,
        base - 3.4 * s,
        stone,
        1.0,
    );
    p.rect(
        x + 0.2 * s,
        base - 11.0 * s,
        x + 1.5 * s,
        base - 3.4 * s,
        dark,
        0.6,
    );
    for k in 0..2 {
        let y = base - 6.0 * s - k as f32 * 2.6 * s;
        p.rect(x - 1.7 * s, y, x + 1.7 * s, y + 0.5 * s, dark, 0.8);
    }
    p.poly(
        &[
            (x - 2.2 * s, base - 11.0 * s),
            (x + 2.2 * s, base - 11.0 * s),
            (x + 4.2 * s, base - 13.0 * s),
            (x - 4.2 * s, base - 13.0 * s),
        ],
        stone,
        1.0,
    );
    p.ellipse(
        x - 0.8 * s,
        base - 13.0 * s,
        3.4 * s,
        thick * 0.5,
        SNOW,
        0.9,
    );
    // light box
    p.rect(
        x - 3.1 * s,
        base - 18.6 * s,
        x + 3.1 * s,
        base - 13.0 * s,
        dark,
        1.0,
    );
    let win = LAMP.scale(flick).mix(rgb(255.0, 240.0, 200.0), 0.3 * flick);
    p.rect(
        x - 1.8 * s,
        base - 17.7 * s,
        x + 1.8 * s,
        base - 13.9 * s,
        win,
        1.0,
    );
    p.rect(
        x - 0.25 * s,
        base - 17.7 * s,
        x + 0.25 * s,
        base - 13.9 * s,
        dark,
        0.9,
    );
    p.rect(
        x - 1.8 * s,
        base - 15.95 * s,
        x + 1.8 * s,
        base - 15.55 * s,
        dark,
        0.9,
    );
    p.glow(
        x,
        base - 15.8 * s,
        3.2 * s,
        rgb(255.0, 230.0, 170.0),
        0.7 * flick,
    );
    // umbrella roof with upturned corners
    p.poly(
        &[
            (x - 7.4 * s, base - 20.4 * s),
            (x - 5.0 * s, base - 19.6 * s),
            (x - 1.6 * s, base - 22.8 * s),
            (x + 1.6 * s, base - 22.8 * s),
            (x + 5.0 * s, base - 19.6 * s),
            (x + 7.4 * s, base - 20.4 * s),
            (x + 6.2 * s, base - 18.5 * s),
            (x - 6.2 * s, base - 18.5 * s),
        ],
        stone,
        1.0,
    );
    p.rect(
        x - 6.2 * s,
        base - 19.1 * s,
        x + 6.2 * s,
        base - 18.5 * s,
        dark,
        0.9,
    );
    p.ellipse(
        x,
        base - 21.6 * s,
        5.0 * s,
        1.5 * s * (0.6 + acc),
        SNOW,
        0.96,
    );
    p.ellipse(
        x + 2.0 * s,
        base - 21.3 * s,
        2.8 * s,
        1.0 * s * (0.6 + acc),
        SNOW_SHADE,
        0.4,
    );
    // jewel finial
    p.circle(x, base - 24.0 * s, 1.3 * s, stone, 1.0);
    p.ellipse(
        x,
        base - 25.0 * s,
        1.1 * s,
        0.6 * s * (0.6 + acc),
        SNOW,
        0.95,
    );
    p.ellipse(x, base - 3.4 * s, 3.4 * s, thick * 0.55, SNOW, 0.92);
    p.ellipse(x, base - 0.3 * s, 5.8 * s, thick * 0.9, SNOW, 0.95);
    (x, base - 15.8 * s, flick)
}

#[allow(clippy::too_many_arguments)]
fn flakes(
    p: &mut Pixels,
    t: f32,
    n: u32,
    seed: u32,
    size: f32,
    speed: f32,
    par: f32,
    alpha: f32,
    lamps: &[(f32, f32, f32)],
    reach: f32,
) {
    let (w, h) = (p.w as f32, p.h as f32);
    let dr = drift(t) * par;
    for i in 0..n {
        let r = |k: u32| {
            hash(
                i.wrapping_mul(7919)
                    .wrapping_add(seed.wrapping_mul(131))
                    .wrapping_add(k),
            )
        };
        let fall = speed * (0.75 + 0.5 * r(3));
        let y = ((r(2) + t * fall).rem_euclid(1.0)) * 1.08 - 0.04;
        let wob =
            0.012 * (t * (0.7 + r(4)) + r(5) * 6.3).sin() + 0.006 * (t * 2.3 + r(6) * 9.0).sin();
        let x = (r(1) + dr + wob).rem_euclid(1.0) * 1.1 - 0.05;
        let (px, py) = (x * w, y * h);
        let mut col = SNOW;
        let mut a = alpha * (0.65 + 0.35 * r(7));
        for &(lx, ly, k) in lamps {
            let d = ((px - lx).powi(2) + (py - ly).powi(2)).sqrt() / reach;
            if d < 1.0 {
                let f = (1.0 - d).powi(2) * k;
                col = col.mix(LAMP_LIT, f * 0.85);
                a = (a + f * 0.35).min(1.0);
            }
        }
        if size <= 1.0 {
            p.blend(px as i32, py as i32, col, a);
        } else {
            let rad = size * 0.5 * (0.75 + 0.5 * r(8));
            p.circle(px, py, rad + 0.8, col, a * 0.25);
            p.circle(px, py, rad, col, a);
        }
    }
}

impl Scene for Shrine {
    fn name(&self) -> &'static str {
        "shrine"
    }

    fn blurb(&self) -> &'static str {
        "Shinto shrine in falling snow with a torii, stone lanterns and gusts"
    }

    fn draw(&self, p: &mut Pixels, t: f32, _theme: Option<&Theme>) {
        let (w, h) = (p.w as f32, p.h as f32);
        let s = h / 80.0;
        let g = gust(t);
        let acc = pile(t);
        let cx = w * 0.5;
        let gy = h * 0.66;

        // overcast night sky, faintly lit by the hidden moon
        p.gradient(&[
            (0.0, rgb(12.0, 16.0, 32.0)),
            (0.45, rgb(34.0, 42.0, 68.0)),
            (0.66, rgb(70.0, 80.0, 108.0)),
            (1.0, rgb(70.0, 80.0, 108.0)),
        ]);
        let (mx, my) = (w * 0.2, h * 0.15);
        p.glow(mx, my, 34.0 * s, rgb(150.0, 160.0, 200.0), 0.35);
        p.circle(mx, my, 5.0 * s, rgb(214.0, 220.0, 236.0), 0.75);
        for y in 0..(gy as usize).min(p.h) {
            for x in 0..p.w {
                let (fx, fy) = (x as f32, y as f32);
                let n = noise2(fx * 0.012 / s + t * 0.012, fy * 0.032 / s) * 0.65
                    + noise2(fx * 0.03 / s - t * 0.02, fy * 0.07 / s + 4.0) * 0.35;
                let dm = (((fx - mx) / w).powi(2) + ((fy - my) / h).powi(2)).sqrt();
                let lit = (1.0 - dm * 2.2).max(0.0);
                let cloud = rgb(46.0, 54.0, 80.0).mix(rgb(150.0, 158.0, 190.0), lit * 0.8);
                p.blend(x as i32, y as i32, cloud, smoothstep(0.38, 0.78, n) * 0.75);
            }
        }

        // distant snowy mountains
        for x in 0..p.w {
            let fx = x as f32;
            let top = h * 0.42 + h * 0.13 * (fbm(fx / w * 2.6 + 1.7) - 0.5) * 2.0;
            let cap = (3.0 + 7.0 * noise(fx * 0.07 / s)) * s;
            for y in top.max(0.0) as usize..(gy as usize).min(p.h) {
                let dy = y as f32 - top;
                let gully = noise2(fx * 0.18 / s, y as f32 * 0.035 / s) > 0.62;
                let c = if dy < cap || gully {
                    rgb(162.0, 172.0, 200.0).mix(rgb(120.0, 130.0, 162.0), dy / (h * 0.25))
                } else {
                    rgb(74.0, 84.0, 114.0)
                };
                let shade = if noise(fx * 0.09 / s + 2.0) > 0.55 {
                    0.9
                } else {
                    1.0
                };
                p.blend(x as i32, y as i32, c.scale(shade), 1.0);
            }
        }
        mist(p, t, 0.5, 0.66, rgb(96.0, 106.0, 138.0), 0.55);

        // far tree line
        let far_dark = rgb(44.0, 54.0, 78.0);
        let far_snow = rgb(140.0, 150.0, 182.0);
        let n_far = (w / (6.0 * s)) as i32 + 2;
        for i in 0..n_far {
            let x = i as f32 * 6.0 * s + 3.0 * s * hash(i as u32 * 5 + 1);
            if (x - cx).abs() < 24.0 * s {
                continue;
            }
            let ht = (9.0 + 7.0 * hash(i as u32 * 5 + 2)) * s;
            cedar(
                p,
                t,
                x,
                gy + 0.5 * s,
                ht,
                i as f32,
                s * 0.5,
                far_dark,
                far_snow,
                g * 0.5,
            );
        }
        flakes(
            p,
            t,
            (w * h / 45.0) as u32,
            1,
            1.0,
            0.035,
            0.45,
            0.45,
            &[],
            1.0,
        );

        // snowy ground with drifts
        for y in (gy as usize).min(p.h)..p.h {
            let f = (y as f32 - gy) / (h - gy);
            for x in 0..p.w {
                let fx = x as f32;
                let top = gy + 1.4 * s * (noise(fx * 0.025 / s) - 0.5);
                if (y as f32) < top {
                    continue;
                }
                let drifts = noise2(fx * 0.04 / s, y as f32 * 0.16 / s);
                let c = rgb(124.0, 134.0, 168.0)
                    .mix(rgb(198.0, 206.0, 228.0), f.powf(0.6))
                    .scale(0.9 + 0.18 * drifts);
                p.blend(x as i32, y as i32, c, 1.0);
            }
        }

        // cedars flanking the shrine
        let mid_dark = rgb(26.0, 40.0, 42.0);
        let mid_snow = rgb(190.0, 198.0, 222.0);
        for (k, off) in [-46.0f32, -36.0, 37.0, 47.0].into_iter().enumerate() {
            let ht = (30.0 + 8.0 * hash(k as u32 + 40)) * s;
            cedar(
                p,
                t,
                cx + off * s,
                gy + 1.0 * s,
                ht,
                k as f32 * 1.7,
                s,
                mid_dark,
                mid_snow,
                g,
            );
        }

        // stone terrace (ishigaki) with stairs, the shrine standing on top
        let terrace = gy - 10.5 * s;
        let stone_wall = rgb(78.0, 82.0, 102.0);
        p.poly(
            &[
                (cx - 34.0 * s, terrace),
                (cx + 34.0 * s, terrace),
                (cx + 37.0 * s, gy + 1.0 * s),
                (cx - 37.0 * s, gy + 1.0 * s),
            ],
            stone_wall,
            1.0,
        );
        for y in terrace as i32..(gy + 1.0 * s) as i32 {
            let row = ((y as f32 - terrace) / (1.6 * s)).floor();
            for x in (cx - 37.0 * s) as i32..(cx + 37.0 * s) as i32 {
                let off = if row as i32 % 2 == 0 { 0.0 } else { 1.5 * s };
                let jx = ((x as f32 + off - cx) / (3.0 * s)).rem_euclid(1.0);
                let jy = ((y as f32 - terrace) / (1.6 * s)).rem_euclid(1.0);
                if jx * 3.0 * s < 0.9 || jy * 1.6 * s < 0.9 {
                    p.blend(x, y, stone_wall.scale(0.62), 0.8);
                }
            }
        }
        p.rect(
            cx - 35.0 * s,
            terrace - 1.4 * s * (0.6 + acc),
            cx + 35.0 * s,
            terrace + 0.4 * s,
            SNOW,
            0.95,
        );
        shrine_hall(p, t, cx, terrace + 0.2 * s, s * 0.9, acc);
        // stairs down the terrace
        let nst = 6;
        for k in 0..nst {
            let f = k as f32 / nst as f32;
            let y0 = lerp(terrace, gy + 1.0 * s, f);
            let y1 = lerp(terrace, gy + 1.0 * s, (k + 1) as f32 / nst as f32);
            let half = lerp(6.5, 8.0, f) * s;
            p.rect(cx - half, y0, cx + half, y1, rgb(96.0, 100.0, 120.0), 1.0);
            p.rect(
                cx - half,
                y0,
                cx + half,
                y0 + (y1 - y0) * 0.45,
                SNOW.mix(SNOW_SHADE, 0.25),
                0.9,
            );
        }

        // trampled path and footprints
        let base_t = h * 0.86;
        p.poly(
            &[
                (cx - 7.0 * s, gy + 0.6 * s),
                (cx + 7.0 * s, gy + 0.6 * s),
                (cx + 24.0 * s, h),
                (cx - 24.0 * s, h),
            ],
            rgb(132.0, 140.0, 174.0),
            0.35,
        );
        let steps = 22;
        for i in 0..steps {
            let v = i as f32 / steps as f32;
            let y = lerp(h - 1.5 * s, gy + 2.0 * s, v.powf(0.75));
            let k = lerp(1.0, 0.3, v);
            let side = if i % 2 == 0 { -1.0 } else { 1.0 };
            let x = cx + side * 1.6 * s * k + 2.0 * s * k * (v * 7.0).sin();
            let fill = 0.55 * (1.0 - 0.7 * acc) * (0.6 + 0.4 * v);
            p.ellipse(
                x,
                y,
                1.0 * s * k,
                0.55 * s * k,
                rgb(92.0, 100.0, 136.0),
                fill,
            );
        }

        torii(p, t, cx, base_t, s, acc, g);
        flakes(
            p,
            t,
            (w * h / 140.0) as u32,
            2,
            1.0,
            0.06,
            0.9,
            0.75,
            &[],
            1.0,
        );

        // stone lanterns
        let lamps = [
            toro(p, t, cx - 33.0 * s, h * 0.985, s, acc, 1.0),
            toro(p, t, cx + 33.0 * s, h * 0.985, s, acc, 9.0),
        ];
        for &(lx, ly, k) in &lamps {
            p.glow(lx, ly, 28.0 * s, LAMP, 0.26 * k);
            p.glow(lx, h * 0.98, 20.0 * s, LAMP, 0.16 * k);
        }

        // near cedars at the edges, heavy with snow
        let near_dark = rgb(14.0, 26.0, 28.0);
        cedar(
            p,
            t,
            w * 0.05,
            h * 1.02,
            74.0 * s,
            0.4,
            s,
            near_dark,
            SNOW,
            g,
        );
        cedar(
            p,
            t,
            w * 0.16,
            h * 0.9,
            50.0 * s,
            2.1,
            s,
            near_dark,
            SNOW,
            g,
        );
        cedar(
            p,
            t,
            w * 0.85,
            h * 0.92,
            54.0 * s,
            3.3,
            s,
            near_dark,
            SNOW,
            g,
        );
        cedar(
            p,
            t,
            w * 0.97,
            h * 1.03,
            76.0 * s,
            4.8,
            s,
            near_dark,
            SNOW,
            g,
        );

        // a clump of snow slides off a branch every few seconds
        let period = 8.5;
        let k = (t / period).floor();
        let ph = t - k * period;
        let trees = [
            (w * 0.16, h * 0.9, 50.0),
            (w * 0.85, h * 0.92, 54.0),
            (cx + 37.0 * s, gy, 30.0),
        ];
        let pick = trees[(hash(k as i32 as u32 * 3 + 1) * 3.0) as usize % 3];
        let tier = 1.0 + (hash(k as i32 as u32 * 3 + 2) * 4.0).floor();
        let x0 = pick.0 + (hash(k as i32 as u32 * 3 + 5) - 0.5) * pick.2 * s * 0.3;
        let y0 = pick.1 - pick.2 * s * (0.16 + 0.115 * tier);
        let land = pick.1 - 0.5 * s;
        let grav = 70.0 * s;
        let t_land = ((land - y0) * 2.0 / grav).sqrt();
        if ph < t_land {
            let y = y0 + 0.5 * grav * ph * ph;
            for j in 0..6 {
                let jx = (hash(j * 3 + 11) - 0.5) * 2.4 * s;
                let jy = (hash(j * 3 + 12) - 0.5) * 2.0 * s * (1.0 + ph);
                p.circle(x0 + jx + g * ph * 3.0 * s, y + jy, 0.9 * s, SNOW, 0.9);
            }
        } else if ph < t_land + 1.4 {
            let a = ph - t_land;
            for j in 0..24 {
                let ang = hash(j * 5 + 21) * std::f32::consts::PI;
                let sp = (4.0 + 10.0 * hash(j * 5 + 22)) * s;
                let px = x0 + g * t_land * 3.0 * s + ang.cos() * sp * a;
                let py = land - ang.sin() * sp * a + 9.0 * s * a * a;
                p.circle(px, py, 0.6 * s + a * s, SNOW, 0.7 * (1.0 - a / 1.4));
            }
        }

        // sparkles on the snow
        let n_sp = (w * (h - gy) / 90.0) as u32;
        for i in 0..n_sp {
            let x = hash(i * 3 + 501) * w;
            let y = gy + hash(i * 3 + 502) * (h - gy);
            let tw = (t * (2.0 + 3.0 * hash(i * 3 + 503)) + i as f32).sin();
            if tw > 0.9 {
                p.add(
                    x as i32,
                    y as i32,
                    rgb(255.0, 255.0, 255.0),
                    (tw - 0.9) * 7.0,
                );
            }
        }

        // spindrift skimming the ground in gusts
        let sd = g.max(0.0);
        if sd > 0.02 {
            let n_sd = (w / (2.0 * s)) as u32;
            for i in 0..n_sd {
                let r = |k: u32| hash(i * 7 + 900 + k);
                let y = gy + (0.1 + 0.9 * r(1)) * (h - gy) - r(2) * 6.0 * s;
                let x = (r(3) + drift(t) * 3.5 * (0.6 + 0.8 * r(4))).rem_euclid(1.0)
                    * (w + 20.0 * s)
                    - 10.0 * s;
                let len = (3.0 + 9.0 * r(5)) * s;
                let a = 0.22 * sd * (0.5 + 0.5 * r(6));
                let steps = len as i32;
                for j in 0..steps {
                    let fj = j as f32 / steps.max(1) as f32;
                    p.blend(
                        (x + j as f32) as i32,
                        (y - fj * 0.8 * s * (t * 3.0 + i as f32).sin()) as i32,
                        SNOW,
                        a * (1.0 - (fj - 0.5).abs() * 2.0),
                    );
                }
            }
        }

        flakes(
            p,
            t,
            (w * h / 520.0) as u32,
            3,
            2.4 * s,
            0.11,
            1.9,
            0.92,
            &lamps,
            30.0 * s,
        );
    }
}
