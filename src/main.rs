mod canvas;
mod render;
mod scenes;
mod theme;
mod util;

use canvas::Pixels;
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use render::Mode;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};
use theme::Theme;

const HELP: &str = "\
jrice: Japan rice pack. Animated Japanese scenes for your terminal.

USAGE
  jrice [SCENE] [OPTIONS]

SCENES
  temple    Pagoda at dusk with wind, lantern, fireflies and petals
  wave      A great wave curling over the sea, Fuji and fishing boats
  samurai   A ronin breathing slowly under the moon
  koi       Koi gliding through a lily pond

COLOURS
  Every scene has its own fixed colours by default. To match your theme:
  --sync             Use your terminal's colours. jrice asks the terminal for
                     them and follows theme changes while it runs.
  --theme T          Use a theme: a built-in name or a kitty theme file
                     (reloaded when the file changes)
  --themes           List the built-in themes

OPTIONS
  --mode M           blocks (default), ascii or dots (braille dot art)
  --ascii            Shortcut for --mode ascii
  --dots             Shortcut for --mode dots
  --fps N            Frames per second (default 24)
  --speed X          Animation speed multiplier (default 1.0)
  --match-bg         Paint the terminal's own background to match the scene,
                     so leftover margins in fullscreen blend in
  --info             Print terminal size and colour info
  --list             List the scenes
  -h, --help         Show this help

KEYS (while running)
  1-4 or Left/Right     switch scene
  a                     cycle blocks / ascii / dots
  t                     toggle fixed colours / theme colours
  q, Esc, Ctrl-C        quit

SNAPSHOTS (for screenshots and testing)
  --snapshot FILE    Write one frame to FILE (.ppm image, .txt text, .ans colour)
  --time T           Time in seconds of the frame (default 3.0)
  --size WxH         Size in terminal cells (default 120x40)
  --scale N          Pixel scale for .ppm output (default 6)
";

struct Opts {
    scene: usize,
    mode: Mode,
    fps: f32,
    speed: f32,
    snapshot: Option<String>,
    time: f32,
    size: (usize, usize),
    scale: usize,
    match_bg: bool,
    info: bool,
    sync: bool,
    theme: Option<String>,
}

fn parse(args: &[String], names: &[&str]) -> Result<Option<Opts>, String> {
    let mut o = Opts {
        scene: 0,
        mode: Mode::Blocks,
        fps: 24.0,
        speed: 1.0,
        snapshot: None,
        time: 3.0,
        size: (120, 40),
        scale: 6,
        match_bg: false,
        info: false,
        sync: false,
        theme: None,
    };
    let mut i = 0;
    let val = |i: &mut usize, flag: &str| -> Result<String, String> {
        *i += 1;
        args.get(*i).cloned().ok_or(format!("{flag} needs a value"))
    };
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "-h" | "--help" => {
                print!("{HELP}");
                return Ok(None);
            }
            "--list" => {
                for sc in scenes::all() {
                    println!("{:<8} {}", sc.name(), sc.blurb());
                }
                return Ok(None);
            }
            "--themes" => {
                for n in Theme::builtin_names() {
                    println!("{n}");
                }
                return Ok(None);
            }
            "--ascii" => o.mode = Mode::Ascii,
            "--dots" => o.mode = Mode::Dots,
            "--mode" => {
                let v = val(&mut i, a)?;
                o.mode = Mode::parse(&v).ok_or("--mode must be blocks, ascii or dots")?;
            }
            "--match-bg" => o.match_bg = true,
            "--info" => o.info = true,
            "--sync" => o.sync = true,
            "--theme" => o.theme = Some(val(&mut i, a)?),
            "--fps" => o.fps = val(&mut i, a)?.parse().map_err(|_| "bad --fps")?,
            "--speed" => o.speed = val(&mut i, a)?.parse().map_err(|_| "bad --speed")?,
            "--time" => o.time = val(&mut i, a)?.parse().map_err(|_| "bad --time")?,
            "--scale" => o.scale = val(&mut i, a)?.parse().map_err(|_| "bad --scale")?,
            "--snapshot" => o.snapshot = Some(val(&mut i, a)?),
            "--size" => {
                let v = val(&mut i, a)?;
                let (a, b) = v.split_once('x').ok_or("--size wants WxH, e.g. 120x40")?;
                o.size = (
                    a.parse().map_err(|_| "bad width")?,
                    b.parse().map_err(|_| "bad height")?,
                );
            }
            name if !name.starts_with('-') => {
                o.scene = names
                    .iter()
                    .position(|n| *n == name)
                    .ok_or(format!("unknown scene '{name}'. Try: {}", names.join(", ")))?;
            }
            other => return Err(format!("unknown option {other}")),
        }
        i += 1;
    }
    o.fps = o.fps.clamp(1.0, 120.0);
    Ok(Some(o))
}

// ---------------------------------------------------------------------------
// Where the theme colours come from, and keeping them up to date

enum Source {
    /// Ask the terminal (OSC queries), re-asked every couple of seconds.
    Terminal,
    /// A kitty theme file, reloaded when it changes.
    File(PathBuf, Option<SystemTime>),
    /// A built-in theme.
    Builtin(String),
}

struct Live {
    theme: Option<Theme>,
    source: Source,
    on: bool,
    last: Instant,
    /// None = not asked yet, Some(false) = the terminal did not answer.
    terminal_ok: Option<bool>,
}

fn mtime(path: &PathBuf) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

impl Live {
    fn from_opts(o: &Opts) -> Result<Live, String> {
        let mut live = Live {
            theme: None,
            source: Source::Terminal,
            on: o.sync || o.theme.is_some(),
            last: Instant::now(),
            terminal_ok: None,
        };
        if let Some(arg) = &o.theme {
            live.theme = Some(Theme::load(arg)?);
            live.source = if Theme::builtin(arg).is_some() {
                Source::Builtin(arg.clone())
            } else {
                let path = theme::expand_home(arg);
                let m = mtime(&path);
                Source::File(path, m)
            };
        }
        Ok(live)
    }

    /// Ask the terminal for its colours (raw mode must be on).
    fn ask(&mut self, timeout: Duration) {
        if !matches!(self.source, Source::Terminal) || self.terminal_ok == Some(false) {
            return;
        }
        let q = theme::query_terminal(timeout);
        if q.answered {
            self.terminal_ok = Some(true);
            if q.theme.is_some() {
                self.theme = q.theme;
            }
        } else {
            self.terminal_ok = Some(false);
        }
    }

    fn tick(&mut self) {
        if !self.on || self.last.elapsed() < Duration::from_secs(2) {
            return;
        }
        self.last = Instant::now();
        match &mut self.source {
            Source::Terminal => self.ask(Duration::from_millis(150)),
            Source::File(path, seen) => {
                let now = mtime(path);
                if now != *seen {
                    *seen = now;
                    if let Ok(text) = std::fs::read_to_string(&*path) {
                        if let Some(t) = Theme::parse_kitty(&text) {
                            self.theme = Some(t);
                        }
                    }
                }
            }
            Source::Builtin(_) => {}
        }
    }

    fn active(&self) -> Option<&Theme> {
        if self.on {
            self.theme.as_ref()
        } else {
            None
        }
    }

    fn label(&self) -> String {
        match &self.source {
            Source::Terminal => "your terminal".into(),
            Source::File(p, _) => p.display().to_string(),
            Source::Builtin(n) => n.clone(),
        }
    }

    fn toggle(&mut self) -> String {
        self.on = !self.on;
        if self.on && self.theme.is_none() {
            self.ask(Duration::from_millis(400));
        }
        if !self.on {
            "colours: fixed".into()
        } else if self.theme.is_some() {
            format!("colours: theme from {}", self.label())
        } else {
            "colours: your terminal didn't report its theme, try --theme".into()
        }
    }

    /// A note to print after leaving the alternate screen.
    fn note(&self) -> Option<String> {
        if self.terminal_ok == Some(false) && self.theme.is_none() {
            Some(
                "your terminal didn't report its colours, so jrice used fixed colours.\n\
                 Try a theme file instead, for kitty: --theme ~/.config/kitty/current-theme.conf"
                    .into(),
            )
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------

fn snapshot(o: &Opts, live: &Live, path: &str) -> io::Result<()> {
    if o.sync && o.theme.is_none() {
        eprintln!("jrice: --sync needs a running terminal; use --theme for snapshots. Using fixed colours.");
    }
    let all = scenes::all();
    let (cols, rows) = (o.size.0.max(8), o.size.1.max(4));
    let (pw, ph) = o.mode.pixel_size(cols, rows);
    let mut px = Pixels::new(pw, ph);
    scenes::paint(all[o.scene].as_ref(), &mut px, o.time, live.active());
    if path.ends_with(".txt") {
        let m = if o.mode == Mode::Blocks {
            Mode::Ascii
        } else {
            o.mode
        };
        std::fs::write(path, render::to_text(&px, m))
    } else if path.ends_with(".ans") {
        std::fs::write(path, render::to_ansi(&px, o.mode))
    } else {
        std::fs::write(path, render::to_ppm(&px, o.scale.max(1)))
    }
}

fn info() -> io::Result<()> {
    let (cols, rows) = terminal::size()?;
    println!("terminal grid: {cols} columns x {rows} rows");
    match terminal::window_size() {
        Ok(ws) if ws.width > 0 && ws.height > 0 => {
            let (cw, ch) = (
                ws.width as f32 / cols as f32,
                ws.height as f32 / rows as f32,
            );
            println!("window pixels: {} x {}", ws.width, ws.height);
            println!("cell size:     {cw:.2} x {ch:.2} px");
            println!(
                "jrice paints:  {} x {} px (the whole grid)",
                cols as f32 * cw,
                rows as f32 * ch
            );
        }
        _ => println!("window pixels: not reported by this terminal"),
    }
    terminal::enable_raw_mode()?;
    let q = theme::query_terminal(Duration::from_millis(500));
    terminal::disable_raw_mode()?;
    match (q.answered, q.theme) {
        (_, Some(t)) => {
            println!(
                "theme colours: background {}  foreground {}",
                theme::to_hex(t.bg),
                theme::to_hex(t.fg)
            );
            let list: Vec<String> = t.ansi.iter().map(|c| theme::to_hex(*c)).collect();
            println!("               palette {}", list.join(" "));
            println!("--sync will work here.");
        }
        (true, None) => println!(
            "theme colours: the terminal answered but did not report its colours (use --theme)"
        ),
        (false, None) => println!("theme colours: no answer from the terminal (use --theme)"),
    }
    Ok(())
}

/// Restores the terminal however we leave, including after a panic.
struct TermGuard {
    reset_bg: bool,
}

impl Drop for TermGuard {
    fn drop(&mut self) {
        let mut out = io::stdout();
        if self.reset_bg {
            let _ = out.write_all(b"\x1b]111\x1b\\");
        }
        let _ = execute!(out, Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

fn run(o: Opts, mut live: Live) -> io::Result<Option<String>> {
    let all = scenes::all();
    let mut idx = o.scene;
    let mut mode = o.mode;
    let mut out = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(out, EnterAlternateScreen, Hide, Clear(ClearType::All))?;
    let _guard = TermGuard {
        reset_bg: o.match_bg,
    };
    if live.on {
        live.ask(Duration::from_millis(500));
    }

    let frame_time = Duration::from_secs_f32(1.0 / o.fps);
    let start = Instant::now();
    let mut last_bg: Option<(u8, u8, u8)> = None;
    let mut toast: Option<(String, Instant)> = None;
    loop {
        let begun = Instant::now();
        live.tick();
        let (cols, rows) = terminal::size()?;
        let (w, h) = mode.pixel_size(cols as usize, rows as usize);
        if w >= 8 && h >= 8 {
            let mut px = Pixels::new(w, h);
            let t = start.elapsed().as_secs_f32() * o.speed;
            scenes::paint(all[idx].as_ref(), &mut px, t, live.active());
            if o.match_bg {
                // the terminal's margin shows its default background: use the colour at the bottom edge
                let edge = px.at(w / 2, h - 1).scale(0.9).to_u8();
                if last_bg != Some(edge) {
                    let _ = write!(
                        out,
                        "\x1b]11;rgb:{:02x}/{:02x}/{:02x}\x1b\\",
                        edge.0, edge.1, edge.2
                    );
                    last_bg = Some(edge);
                }
            }
            let mut frame = render::to_ansi(&px, mode);
            if let Some((msg, at)) = &toast {
                if at.elapsed() < Duration::from_millis(2200) {
                    let msg: String = msg.chars().take(cols.saturating_sub(4) as usize).collect();
                    frame.push_str(&format!("\x1b[{rows};2H\x1b[0m\x1b[1;7m {msg} \x1b[0m"));
                } else {
                    toast = None;
                }
            }
            out.write_all(frame.as_bytes())?;
            out.flush()?;
        }
        let mut wait = frame_time.saturating_sub(begun.elapsed());
        while event::poll(wait)? {
            if let Event::Key(k) = event::read()? {
                if k.kind == KeyEventKind::Release {
                    continue;
                }
                match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(live.note()),
                    KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(live.note())
                    }
                    KeyCode::Char('a') => {
                        mode = mode.next();
                        toast = Some((format!("mode: {mode:?}").to_lowercase(), Instant::now()));
                    }
                    KeyCode::Char('t') => toast = Some((live.toggle(), Instant::now())),
                    KeyCode::Right => idx = (idx + 1) % all.len(),
                    KeyCode::Left => idx = (idx + all.len() - 1) % all.len(),
                    KeyCode::Char(c @ '1'..='9') => {
                        let n = c as usize - '1' as usize;
                        if n < all.len() {
                            idx = n;
                        }
                    }
                    _ => {}
                }
            }
            wait = Duration::ZERO;
        }
    }
}

fn main() {
    let names: Vec<&str> = scenes::all().iter().map(|s| s.name()).collect();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let fail = |e: String| -> ! {
        eprintln!("jrice: {e}\nTry 'jrice --help'.");
        std::process::exit(2);
    };
    let opts = match parse(&args, &names) {
        Ok(Some(o)) => o,
        Ok(None) => return,
        Err(e) => fail(e),
    };
    let live = Live::from_opts(&opts).unwrap_or_else(|e| fail(e));
    let res = if opts.info {
        info().map(|_| None)
    } else if let Some(path) = opts.snapshot.clone() {
        snapshot(&opts, &live, &path).map(|_| None)
    } else {
        run(opts, live)
    };
    match res {
        Ok(Some(note)) => eprintln!("jrice: {note}"),
        Ok(None) => {}
        Err(e) => {
            eprintln!("jrice: {e}");
            std::process::exit(1);
        }
    }
}
