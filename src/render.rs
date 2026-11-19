//! Turns a pixel canvas into terminal output (or image files for snapshots).

use crate::canvas::Pixels;
use std::fmt::Write;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    /// Half blocks, two pixels per cell. Looks like a painting.
    Blocks,
    /// Characters picked by brightness. Looks like classic ASCII art.
    Ascii,
    /// Braille dots with ordered dithering, eight dots per cell. Looks like
    /// dot-matrix halftone art.
    Dots,
}

impl Mode {
    pub fn next(self) -> Mode {
        match self {
            Mode::Blocks => Mode::Ascii,
            Mode::Ascii => Mode::Dots,
            Mode::Dots => Mode::Blocks,
        }
    }

    pub fn parse(s: &str) -> Option<Mode> {
        match s {
            "blocks" | "block" => Some(Mode::Blocks),
            "ascii" => Some(Mode::Ascii),
            "dots" | "braille" => Some(Mode::Dots),
            _ => None,
        }
    }

    /// Canvas size in pixels for a terminal of `cols` x `rows` cells.
    pub fn pixel_size(self, cols: usize, rows: usize) -> (usize, usize) {
        match self {
            Mode::Blocks | Mode::Ascii => (cols, rows * 2),
            Mode::Dots => (cols * 2, rows * 4),
        }
    }

    fn cells(self, p: &Pixels) -> (usize, usize) {
        match self {
            Mode::Blocks | Mode::Ascii => (p.w, p.h / 2),
            Mode::Dots => (p.w / 2, p.h / 4),
        }
    }
}

type Rgb8 = (u8, u8, u8);

const RAMP: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

fn ascii_cell(p: &Pixels, x: usize, row: usize) -> (char, Rgb8, Rgb8) {
    let avg = p.at(x, 2 * row).mix(p.at(x, 2 * row + 1), 0.5);
    let l = ((avg.luma() / 255.0) * 1.35).clamp(0.0, 1.0).powf(0.5);
    let idx = (l * (RAMP.len() - 1) as f32).round() as usize;
    (RAMP[idx], avg.scale(1.2).to_u8(), avg.scale(0.14).to_u8())
}

/// Ordered-dither thresholds for the 2 x 4 dots of one braille cell, indexed
/// [dy][dx].
const BAYER: [[u8; 2]; 4] = [[0, 4], [6, 2], [3, 7], [5, 1]];
/// Braille dot bit for each sub-position, indexed [dy][dx].
const BITS: [[u32; 2]; 4] = [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];

fn dots_cell(p: &Pixels, cx: usize, row: usize) -> (char, Rgb8, Rgb8) {
    let (mut mask, mut on, mut n) = (0u32, (0.0f32, 0.0f32, 0.0f32), 0.0f32);
    let mut all = (0.0f32, 0.0f32, 0.0f32);
    for dy in 0..4 {
        for dx in 0..2 {
            let c = p.at(cx * 2 + dx, row * 4 + dy);
            all = (all.0 + c.r, all.1 + c.g, all.2 + c.b);
            let l = ((c.luma() / 255.0) * 1.5).min(1.0).powf(0.55);
            // ordered dither plus a little grain, so smooth gradients don't band
            let (gx, gy) = ((cx * 2 + dx) as u32, (row * 4 + dy) as u32);
            let grain = crate::util::hash(gx.wrapping_mul(7919) ^ gy.wrapping_mul(104_729)) - 0.5;
            let thr = (BAYER[dy][dx] as f32 + 0.5) / 8.0 + grain * 0.35;
            if l > thr {
                mask |= BITS[dy][dx];
                on = (on.0 + c.r, on.1 + c.g, on.2 + c.b);
                n += 1.0;
            }
        }
    }
    let to8 = |v: f32| v.clamp(0.0, 255.0) as u8;
    let bg = (
        to8(all.0 / 8.0 * 0.10),
        to8(all.1 / 8.0 * 0.10),
        to8(all.2 / 8.0 * 0.10),
    );
    let fg = if n > 0.0 {
        (
            to8(on.0 / n * 1.3),
            to8(on.1 / n * 1.3),
            to8(on.2 / n * 1.3),
        )
    } else {
        bg
    };
    (char::from_u32(0x2800 + mask).unwrap_or(' '), fg, bg)
}

fn cell(p: &Pixels, mode: Mode, x: usize, row: usize) -> (char, Rgb8, Rgb8) {
    match mode {
        Mode::Blocks => ('▀', p.at(x, 2 * row).to_u8(), p.at(x, 2 * row + 1).to_u8()),
        Mode::Ascii => ascii_cell(p, x, row),
        Mode::Dots => dots_cell(p, x, row),
    }
}

/// ANSI truecolor frame. Every row is positioned explicitly, so no newline is
/// ever printed and the terminal never scrolls.
pub fn to_ansi(p: &Pixels, mode: Mode) -> String {
    let (cols, rows) = mode.cells(p);
    let mut s = String::with_capacity(cols * rows * 24);
    for r in 0..rows {
        let _ = write!(s, "\x1b[{};1H", r + 1);
        // colours are only re-sent when they change, and each one separately
        let (mut last_fg, mut last_bg): (Option<Rgb8>, Option<Rgb8>) = (None, None);
        for x in 0..cols {
            let (ch, fg, bg) = cell(p, mode, x, r);
            let blank = ch == ' ' || ch == '\u{2800}';
            if !blank && last_fg != Some(fg) {
                let _ = write!(s, "\x1b[38;2;{};{};{}m", fg.0, fg.1, fg.2);
                last_fg = Some(fg);
            }
            if last_bg != Some(bg) {
                let _ = write!(s, "\x1b[48;2;{};{};{}m", bg.0, bg.1, bg.2);
                last_bg = Some(bg);
            }
            s.push(ch);
        }
        s.push_str("\x1b[0m");
    }
    s
}

/// Plain text version (no colour) in the given text mode.
pub fn to_text(p: &Pixels, mode: Mode) -> String {
    let (cols, rows) = mode.cells(p);
    let mut s = String::new();
    for r in 0..rows {
        for x in 0..cols {
            s.push(cell(p, mode, x, r).0);
        }
        s.push('\n');
    }
    s
}

/// Binary PPM of the pixel buffer, scaled up by an integer factor.
pub fn to_ppm(p: &Pixels, scale: usize) -> Vec<u8> {
    let mut out = format!("P6\n{} {}\n255\n", p.w * scale, p.h * scale).into_bytes();
    for y in 0..p.h * scale {
        for x in 0..p.w * scale {
            let (r, g, b) = p.at(x / scale, y / scale).to_u8();
            out.extend_from_slice(&[r, g, b]);
        }
    }
    out
}
