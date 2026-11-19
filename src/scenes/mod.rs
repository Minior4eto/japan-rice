//! Scenes are pure functions of time: draw(canvas, t) paints one frame.
//! Sizes are expressed relative to the canvas height, so every scene adapts to
//! any terminal size.

mod city;
mod koi;
mod lighthouse;
mod samurai;
mod shrine;
mod temple;
mod wave;

use crate::canvas::Pixels;
use crate::theme::{Recolor, Theme};
use crate::util::{hash, rgb, smoothstep, Color};

pub trait Scene {
    fn name(&self) -> &'static str;
    fn blurb(&self) -> &'static str;
    /// Paint one frame. `theme` is Some when colours should follow a theme.
    fn draw(&self, p: &mut Pixels, t: f32, theme: Option<&Theme>);
    /// Scenes that build their colours from the theme themselves. Others are
    /// painted in fixed colours and recoloured afterwards.
    fn native_theme(&self) -> bool {
        false
    }
}

/// Draw a scene, applying the theme if one is active.
pub fn paint(scene: &dyn Scene, p: &mut Pixels, t: f32, theme: Option<&Theme>) {
    scene.draw(p, t, theme);
    if let Some(th) = theme {
        if !scene.native_theme() {
            let rc = Recolor::new(th);
            p.map(|c| rc.apply(c));
        }
    }
}

pub fn all() -> Vec<Box<dyn Scene>> {
    vec![
        Box::new(temple::Temple),
        Box::new(wave::Wave),
        Box::new(samurai::Samurai),
        Box::new(koi::Koi),
        Box::new(shrine::Shrine),
        Box::new(lighthouse::Lighthouse),
        Box::new(city::City),
    ]
}

/// Cherry blossom petals blown across the frame by the wind.
pub fn petals(p: &mut Pixels, t: f32, count: u32, seed: u32) {
    petals_with(
        p,
        t,
        count,
        seed,
        rgb(255.0, 183.0, 197.0),
        rgb(255.0, 224.0, 232.0),
    );
}

/// Petals in chosen colours.
pub fn petals_with(p: &mut Pixels, t: f32, count: u32, seed: u32, c1: Color, c2: Color) {
    let (w, h) = (p.w as f32, p.h as f32);
    let s = h / 80.0;
    for i in 0..count {
        let r = |k: u32| hash(i.wrapping_mul(13).wrapping_add(k).wrapping_add(seed));
        let wind = 0.035 + 0.05 * r(3);
        let fall = 0.03 + 0.05 * r(4);
        let gust = 0.5 + 0.5 * (t * 0.35 + r(6) * 6.0).sin();
        let x = ((r(1) + t * wind * (0.7 + gust)) % 1.2) - 0.1 + 0.012 * (t * 1.6 + i as f32).sin();
        let y = ((r(2) + t * fall) % 1.1) - 0.05 + 0.008 * (t * 2.3 + i as f32 * 1.3).sin();
        let (px, py) = (x * w, y * h);
        let col = if r(5) > 0.5 { c1 } else { c2 };
        p.blend(px as i32, py as i32, col, 0.95);
        if s > 1.1 {
            p.blend(px as i32 + 1, py as i32, col, 0.7);
        }
        if s > 1.8 {
            p.blend(px as i32, py as i32 + 1, col, 0.6);
        }
    }
}

/// Low drifting mist: a soft band between y0 and y1 (fractions of height).
pub fn mist(p: &mut Pixels, t: f32, y0: f32, y1: f32, col: crate::util::Color, strength: f32) {
    let (w, h) = (p.w, p.h as f32);
    let mid = (y0 + y1) * 0.5;
    for y in (y0 * h) as usize..((y1 * h) as usize).min(p.h) {
        let f = y as f32 / h;
        let bump = 1.0 - smoothstep(0.0, (y1 - y0) * 0.5, (f - mid).abs());
        for x in 0..w {
            let n = crate::util::noise(
                x as f32 * 0.035 / (h / 80.0) * 2.0 + t * 0.35 + y as f32 * 0.05,
            );
            p.blend(x as i32, y as i32, col, strength * bump * (0.35 + 0.65 * n));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_scene_draws_at_many_sizes_and_times() {
        for scene in all() {
            for (w, h) in [(8, 8), (20, 10), (80, 48), (200, 100), (301, 77)] {
                for t in [0.0, 0.7, 3.0, 61.3, 1000.0] {
                    let mut p = Pixels::new(w, h);
                    paint(scene.as_ref(), &mut p, t, None);
                }
            }
        }
    }

    #[test]
    fn scenes_are_not_blank() {
        for scene in all() {
            let mut p = Pixels::new(120, 80);
            paint(scene.as_ref(), &mut p, 3.0, None);
            let first = p.at(0, 0);
            let varied = (0..80).any(|y| p.at(60, y) != first);
            assert!(varied, "{} rendered a flat frame", scene.name());
        }
    }

    #[test]
    fn every_scene_renders_in_every_mode() {
        use crate::render::{to_ansi, to_text, Mode};
        for scene in all() {
            for mode in [Mode::Blocks, Mode::Ascii, Mode::Dots] {
                for (cols, rows) in [(4, 2), (40, 12), (101, 33)] {
                    let (w, h) = mode.pixel_size(cols, rows);
                    let mut p = Pixels::new(w, h);
                    paint(scene.as_ref(), &mut p, 2.0, None);
                    let ansi = to_ansi(&p, mode);
                    assert!(ansi.contains("\x1b[1;1H"));
                    let text = to_text(&p, mode);
                    assert_eq!(text.lines().count(), rows, "{} {:?}", scene.name(), mode);
                }
            }
        }
    }

    #[test]
    fn every_scene_follows_themes() {
        for scene in all() {
            for name in Theme::builtin_names() {
                let th = Theme::builtin(name).unwrap();
                let mut p = Pixels::new(60, 40);
                paint(scene.as_ref(), &mut p, 4.0, Some(&th));
                let mut fixed = Pixels::new(60, 40);
                paint(scene.as_ref(), &mut fixed, 4.0, None);
                let differs = (0..40).any(|y| p.at(30, y) != fixed.at(30, y));
                assert!(differs, "{} ignored theme {name}", scene.name());
            }
        }
    }

    #[test]
    fn mode_cycles_through_all_three() {
        use crate::render::Mode;
        let m = Mode::Blocks;
        assert_eq!(m.next().next().next(), m);
        assert_eq!(Mode::parse("dots"), Some(Mode::Dots));
        assert_eq!(Mode::parse("nope"), None);
    }

    #[test]
    fn frames_are_deterministic() {
        for scene in all() {
            let (mut a, mut b) = (Pixels::new(90, 60), Pixels::new(90, 60));
            paint(scene.as_ref(), &mut a, 5.5, None);
            paint(scene.as_ref(), &mut b, 5.5, None);
            assert_eq!(crate::render::to_ppm(&a, 1), crate::render::to_ppm(&b, 1));
        }
    }
}
