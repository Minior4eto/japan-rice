//! Colour themes. A theme is a background, a foreground and the 16 ANSI
//! colours, the same things a terminal theme defines. Themes come from three
//! places: built-in palettes, kitty theme files, or the terminal itself, which
//! we ask for its colours with OSC 4 / 10 / 11 queries.

use crate::util::{rgb, Color};
use std::path::Path;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub ansi: [Color; 16],
}

fn hex(s: &str) -> Option<Color> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let v = u32::from_str_radix(s, 16).ok()?;
    Some(rgb(
        ((v >> 16) & 255) as f32,
        ((v >> 8) & 255) as f32,
        (v & 255) as f32,
    ))
}

pub fn to_hex(c: Color) -> String {
    let (r, g, b) = c.to_u8();
    format!("#{r:02x}{g:02x}{b:02x}")
}

type Spec = (&'static str, &'static str, &'static str, [&'static str; 16]);

/// The koi pond's own palette, used when colours are fixed.
const POND: Spec = (
    "pond",
    "#0a1a22",
    "#efe6d4",
    [
        "#101418", "#d8402f", "#4f7f3c", "#e9a93a", "#1d5470", "#e07a9a", "#3b8f94", "#e8e1d3",
        "#34444c", "#ff6a4d", "#7cae5a", "#ffd36b", "#3f86a8", "#f6a6bf", "#6cc3c0", "#fff8ec",
    ],
);

const BUILTIN: &[Spec] = &[
    (
        "kanagawa",
        "#1f1f28",
        "#dcd7ba",
        [
            "#16161d", "#c34043", "#76946a", "#c0a36e", "#7e9cd8", "#957fb8", "#6a9589", "#c8c093",
            "#727169", "#e82424", "#98bb6c", "#e6c384", "#7fb4ca", "#938aa9", "#7aa89f", "#dcd7ba",
        ],
    ),
    (
        "catppuccin-mocha",
        "#1e1e2e",
        "#cdd6f4",
        [
            "#45475a", "#f38ba8", "#a6e3a1", "#f9e2af", "#89b4fa", "#f5c2e7", "#94e2d5", "#bac2de",
            "#585b70", "#f38ba8", "#a6e3a1", "#f9e2af", "#89b4fa", "#f5c2e7", "#94e2d5", "#a6adc8",
        ],
    ),
    (
        "gruvbox-dark",
        "#282828",
        "#ebdbb2",
        [
            "#282828", "#cc241d", "#98971a", "#d79921", "#458588", "#b16286", "#689d6a", "#a89984",
            "#928374", "#fb4934", "#b8bb26", "#fabd2f", "#83a598", "#d3869b", "#8ec07c", "#ebdbb2",
        ],
    ),
    (
        "tokyo-night",
        "#1a1b26",
        "#c0caf5",
        [
            "#15161e", "#f7768e", "#9ece6a", "#e0af68", "#7aa2f7", "#bb9af7", "#7dcfff", "#a9b1d6",
            "#414868", "#f7768e", "#9ece6a", "#e0af68", "#7aa2f7", "#bb9af7", "#7dcfff", "#c0caf5",
        ],
    ),
    (
        "nord",
        "#2e3440",
        "#d8dee9",
        [
            "#3b4252", "#bf616a", "#a3be8c", "#ebcb8b", "#81a1c1", "#b48ead", "#88c0d0", "#e5e9f0",
            "#4c566a", "#bf616a", "#a3be8c", "#ebcb8b", "#81a1c1", "#b48ead", "#8fbcbb", "#eceff4",
        ],
    ),
    (
        "rose-pine",
        "#191724",
        "#e0def4",
        [
            "#26233a", "#eb6f92", "#31748f", "#f6c177", "#9ccfd8", "#c4a7e7", "#ebbcba", "#e0def4",
            "#6e6a86", "#eb6f92", "#31748f", "#f6c177", "#9ccfd8", "#c4a7e7", "#ebbcba", "#e0def4",
        ],
    ),
];

fn from_spec(s: &Spec) -> Theme {
    let mut ansi = [rgb(0.0, 0.0, 0.0); 16];
    for (i, h) in s.3.iter().enumerate() {
        ansi[i] = hex(h).expect("built-in colour");
    }
    Theme {
        bg: hex(s.1).expect("built-in colour"),
        fg: hex(s.2).expect("built-in colour"),
        ansi,
    }
}

impl Theme {
    pub fn pond() -> Theme {
        from_spec(&POND)
    }

    pub fn builtin(name: &str) -> Option<Theme> {
        BUILTIN.iter().find(|s| s.0 == name).map(from_spec)
    }

    pub fn builtin_names() -> Vec<&'static str> {
        BUILTIN.iter().map(|s| s.0).collect()
    }

    /// Parse a kitty theme / config file: `background`, `foreground` and
    /// `color0`..`color15`. Missing colours fall back to the pond palette.
    pub fn parse_kitty(text: &str) -> Option<Theme> {
        let mut th = Theme::pond();
        let mut found = false;
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('#') {
                continue;
            }
            let mut parts = line.split_whitespace();
            let (Some(key), Some(val)) = (parts.next(), parts.next()) else {
                continue;
            };
            let Some(c) = hex(val) else { continue };
            match key {
                "background" => {
                    th.bg = c;
                    found = true;
                }
                "foreground" => {
                    th.fg = c;
                    found = true;
                }
                k => {
                    if let Some(n) = k
                        .strip_prefix("color")
                        .and_then(|n| n.parse::<usize>().ok())
                    {
                        if n < 16 {
                            th.ansi[n] = c;
                            found = true;
                        }
                    }
                }
            }
        }
        found.then_some(th)
    }

    /// A built-in name or a path to a kitty theme file.
    pub fn load(arg: &str) -> Result<Theme, String> {
        if let Some(t) = Theme::builtin(arg) {
            return Ok(t);
        }
        let path = expand_home(arg);
        let text = std::fs::read_to_string(&path).map_err(|e| {
            format!(
                "'{arg}' is not a built-in theme ({}) and could not be read as a file: {e}",
                Theme::builtin_names().join(", ")
            )
        })?;
        Theme::parse_kitty(&text).ok_or(format!("no colours found in {arg}"))
    }
}

pub fn expand_home(arg: &str) -> std::path::PathBuf {
    if let Some(rest) = arg.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return Path::new(&home).join(rest);
        }
    }
    Path::new(arg).to_path_buf()
}

// ---------------------------------------------------------------------------
// Recolouring scenes that were painted with fixed colours

/// Hue (0..1), saturation and value.
pub fn hsv(c: Color) -> (f32, f32, f32) {
    let (r, g, b) = (c.r / 255.0, c.g / 255.0, c.b / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let s = if max > 0.0 { d / max } else { 0.0 };
    let h = if d < 1e-5 {
        0.0
    } else if max == r {
        ((g - b) / d).rem_euclid(6.0) / 6.0
    } else if max == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };
    (h, s, max)
}

/// Maps a fixed-colour picture onto a theme. Brightness becomes a blend from
/// the theme background to its foreground, and saturated colours are pulled
/// toward the theme colour with the nearest hue. Dark areas end up exactly on
/// the theme background, so the scene blends into the terminal.
pub struct Recolor {
    bg: Color,
    fg: Color,
    accents: Vec<(f32, Color, f32)>,
}

impl Recolor {
    pub fn new(th: &Theme) -> Recolor {
        let mut accents = Vec::new();
        for i in (1..=6).chain(9..=14) {
            let c = th.ansi[i];
            let (h, s, _) = hsv(c);
            if s > 0.12 {
                accents.push((h, c, (c.luma() / 255.0).clamp(0.05, 0.95)));
            }
        }
        Recolor {
            bg: th.bg,
            fg: th.fg,
            accents,
        }
    }

    pub fn apply(&self, c: Color) -> Color {
        let l = (c.luma() / 255.0).clamp(0.0, 1.0);
        let base = self.bg.mix(self.fg, l.powf(0.9));
        let (h, s, _) = hsv(c);
        if s < 0.08 || self.accents.is_empty() {
            return base;
        }
        let mut best = self.accents[0];
        let mut bd = f32::MAX;
        for a in &self.accents {
            let d = (a.0 - h).abs();
            let d = d.min(1.0 - d);
            if d < bd {
                bd = d;
                best = *a;
            }
        }
        let (_, acc, la) = best;
        let tinted = if l < la {
            self.bg.mix(acc, l / la)
        } else {
            acc.mix(self.fg, (l - la) / (1.0 - la))
        };
        base.mix(tinted, (s * 1.3).min(1.0))
    }
}

// ---------------------------------------------------------------------------
// Asking the terminal for its colours

pub struct Query {
    /// The colours the terminal reported, if any.
    pub theme: Option<Theme>,
    /// Whether the terminal answered at all (we fence the queries with a
    /// device-attributes request that every terminal answers).
    pub answered: bool,
}

fn parse_component(s: &str) -> Option<f32> {
    if s.is_empty() || s.len() > 4 {
        return None;
    }
    let v = u32::from_str_radix(s, 16).ok()?;
    let max = (1u32 << (4 * s.len())) - 1;
    Some(v as f32 / max as f32 * 255.0)
}

fn parse_rgb(s: &str) -> Option<Color> {
    let s = s.strip_prefix("rgb:")?;
    let mut it = s.split('/');
    let r = parse_component(it.next()?)?;
    let g = parse_component(it.next()?)?;
    let b = parse_component(it.next()?)?;
    Some(rgb(r, g, b))
}

/// Parse OSC colour replies such as `ESC ] 4;1;rgb:cccc/2424/1d1d ESC \`.
pub fn parse_osc(buf: &[u8]) -> Option<Theme> {
    let s = String::from_utf8_lossy(buf);
    let mut th = Theme::pond();
    let (mut got_bg, mut got_fg) = (false, false);
    for piece in s.split(['\x1b', '\x07']) {
        let Some(rest) = piece.strip_prefix(']') else {
            continue;
        };
        if let Some(r) = rest.strip_prefix("4;") {
            if let Some((idx, val)) = r.split_once(';') {
                if let (Ok(i), Some(c)) = (idx.parse::<usize>(), parse_rgb(val)) {
                    if i < 16 {
                        th.ansi[i] = c;
                    }
                }
            }
        } else if let Some(v) = rest.strip_prefix("10;") {
            if let Some(c) = parse_rgb(v) {
                th.fg = c;
                got_fg = true;
            }
        } else if let Some(v) = rest.strip_prefix("11;") {
            if let Some(c) = parse_rgb(v) {
                th.bg = c;
                got_bg = true;
            }
        }
    }
    (got_bg && got_fg).then_some(th)
}

/// True once the reply to `ESC [ c` (primary device attributes) has arrived.
pub fn has_da1(buf: &[u8]) -> bool {
    let mut i = 0;
    while i + 3 <= buf.len() {
        if &buf[i..i + 3] == b"\x1b[?" {
            let mut j = i + 3;
            while j < buf.len() && (buf[j].is_ascii_digit() || buf[j] == b';') {
                j += 1;
            }
            if j < buf.len() && buf[j] == b'c' {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// Ask the terminal for its colours. The terminal must be in raw mode.
#[cfg(unix)]
pub fn query_terminal(timeout: Duration) -> Query {
    use std::io::{Read, Write};
    use std::os::unix::io::AsRawFd;
    use std::time::Instant;

    let none = Query {
        theme: None,
        answered: false,
    };
    let Ok(mut tty) = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
    else {
        return none;
    };
    let mut q = String::new();
    for i in 0..16 {
        q.push_str(&format!("\x1b]4;{i};?\x1b\\"));
    }
    q.push_str("\x1b]10;?\x1b\\\x1b]11;?\x1b\\\x1b[c");
    if tty
        .write_all(q.as_bytes())
        .and_then(|_| tty.flush())
        .is_err()
    {
        return none;
    }

    let fd = tty.as_raw_fd();
    let deadline = Instant::now() + timeout;
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let left = deadline
            .saturating_duration_since(Instant::now())
            .as_millis() as i32;
        if left <= 0 {
            break;
        }
        let mut pfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: pfd is a valid pollfd and we pass a count of 1.
        let r = unsafe { libc::poll(&mut pfd, 1, left) };
        if r <= 0 {
            break;
        }
        match tty.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
        }
        if has_da1(&buf) {
            break;
        }
    }
    Query {
        theme: parse_osc(&buf),
        answered: has_da1(&buf),
    }
}

#[cfg(not(unix))]
pub fn query_terminal(_timeout: Duration) -> Query {
    Query {
        theme: None,
        answered: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_load() {
        for n in Theme::builtin_names() {
            assert!(Theme::builtin(n).is_some());
        }
        let _ = Theme::pond();
    }

    #[test]
    fn parses_osc_replies() {
        let reply = b"\x1b]4;1;rgb:cccc/2424/1d1d\x1b\\\x1b]10;rgb:ebeb/dbdb/b2b2\x07\x1b]11;rgb:28/28/28\x1b\\\x1b[?62;22c";
        let th = parse_osc(reply).expect("theme");
        assert_eq!(th.ansi[1].to_u8(), (204, 36, 29));
        assert_eq!(th.fg.to_u8(), (235, 219, 178));
        assert_eq!(th.bg.to_u8(), (40, 40, 40));
        assert!(has_da1(reply));
        assert!(!has_da1(b"\x1b]11;rgb:28/28/28\x1b\\"));
    }

    #[test]
    fn parses_kitty_theme_file() {
        let text =
            "# a theme\nbackground #112233\nforeground   #ddeeff\ncolor4 #0000ff\ncursor #ffffff\n";
        let th = Theme::parse_kitty(text).expect("theme");
        assert_eq!(th.bg.to_u8(), (0x11, 0x22, 0x33));
        assert_eq!(th.ansi[4].to_u8(), (0, 0, 255));
        assert!(Theme::parse_kitty("nothing here").is_none());
    }

    #[test]
    fn recolor_puts_black_on_the_background() {
        let th = Theme::builtin("gruvbox-dark").unwrap();
        let rc = Recolor::new(&th);
        assert_eq!(rc.apply(rgb(0.0, 0.0, 0.0)).to_u8(), th.bg.to_u8());
        let red = rc.apply(rgb(220.0, 30.0, 30.0));
        assert!(red.r > red.g && red.r > red.b);
    }
}
