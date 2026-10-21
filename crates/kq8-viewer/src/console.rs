//! The command console (`docs/DESIGN.md`, "the command console"): lines
//! typed in the terminal that started the viewer run as script commands,
//! the same ones `--command "WORDS",N` runs. Our own developer tool, not
//! from the original game; it adds no game rules.
//!
//! A side thread reads the terminal, so the game never waits for typing.
//! Each line runs at the start of the next update, before the doors.

use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Mutex;

use bevy::prelude::*;

use crate::connor_view::ConnorSim;
use crate::door_view::DoorSim;
use crate::log::Log;

pub use screen::close as close_screen;

const HELP: &str = "\
console: type a command and press Enter (the game window keeps running).
  give <item> [count]              e.g. give SmCrossbow; give gold 50
  move Connor <flags> x y z 0 0 t  flags 0 set place and turn, 1 add place,
                                   2 add turn, 3 add both; t = turn (radians)
                                   e.g. move Connor 3 0 0 2000 (2,000 higher)
  move KQCamera <flags> x y z ...  the same for the camera
  where                            Connor's place and turn (for move)
  help                             this list
  Any other script command works too (sendEvent, deleteObject, KQSound::play,
  setCombatLevel, handsOff, ...): it runs as if a script ran it.
  The result is $Console::Return: 0 usually means done, 1 failed or unknown.";

/// The lines typed so far, and `console.log`.
#[derive(Resource)]
pub struct Console {
    lines: Mutex<Receiver<String>>,
    log: Option<Log>,
    /// The terminal was closed or can't be read: nothing more will come.
    closed: bool,
}

impl Console {
    /// Starts the thread that reads the terminal: the typing line at the
    /// bottom when it is a real terminal, plain lines otherwise (piped
    /// input, as in our tests).
    pub fn start() -> Self {
        let (send, lines) = mpsc::channel();
        let boxed = screen::open();
        let spawned = std::thread::Builder::new().name("console".into()).spawn(move || {
            if boxed {
                screen::read_keys(send);
                return;
            }
            for line in std::io::stdin().lines() {
                let Ok(line) = line else { break };
                if send.send(line).is_err() {
                    break;
                }
            }
        });
        let mut log = Log::create("console").ok();
        let first = match (spawned, boxed) {
            (Ok(_), true) => "console: type commands on the bottom line of this terminal and press Enter ('help' for the list)",
            (Ok(_), false) => "console: type commands in this terminal and press Enter ('help' for the list)",
            (Err(_), _) => {
                screen::close();
                "console: could not start the thread that reads the terminal; no console"
            }
        };
        println!("{first}");
        if let Some(log) = log.as_mut() {
            log.detail(first);
        }
        Console { lines: Mutex::new(lines), log, closed: false }
    }

    fn say(&mut self, line: impl AsRef<str>) {
        println!("{}", line.as_ref());
        if let Some(log) = self.log.as_mut() {
            log.detail(line);
        }
    }
}

/// Runs every line typed since the last update.
pub fn console_step(console: Option<ResMut<Console>>, sim: Option<ResMut<DoorSim>>, connor: Option<Res<ConnorSim>>) {
    let Some(mut console) = console else { return };
    // Lines typed while the world loads wait in the channel until it runs.
    if console.closed || sim.is_none() {
        return;
    }
    let mut typed = Vec::new();
    let mut ended = false;
    {
        let lines = console.lines.lock().unwrap();
        loop {
            match lines.try_recv() {
                Ok(line) => typed.push(line),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    ended = true;
                    break;
                }
            }
        }
    }
    let mut sim = sim;
    for line in typed {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let updates = sim.as_ref().map_or(0, |s| s.active.world.updates);
        console.say(format!("> {line}  (update {updates})"));
        let first = line.split_whitespace().next().unwrap_or("");
        if first.eq_ignore_ascii_case("help") {
            console.say(HELP);
            continue;
        }
        if first.eq_ignore_ascii_case("where") {
            let said = match connor.as_ref() {
                Some(c) => {
                    let p = c.mover.position;
                    format!("Connor is at {:.0} {:.0} {:.0}, turn {:.4} (move Connor 0 {:.0} {:.0} {:.0} 0 0 {:.4} puts him back here)", p[0], p[1], p[2], c.walker.heading, p[0], p[1], p[2], c.walker.heading)
                }
                None => "no Connor in this view".to_string(),
            };
            console.say(said);
            continue;
        }
        let Some(sim) = sim.as_mut() else { continue };
        // The log lines the command writes are shown here too (they also
        // go to doors.log as usual).
        let before = sim.active.world.log.len();
        let result = sim.active.test_command(line, "console");
        let wrote: Vec<String> = sim.active.world.log.get(before..).unwrap_or_default().to_vec();
        for w in wrote {
            console.say(format!("  {w}"));
        }
        match result {
            Some(r) => console.say(format!("  result {r}")),
            None => console.say(format!("  unknown command '{first}' ('help' for the list)")),
        }
    }
    // After the lines, so lines typed just before the end still run.
    if ended {
        console.closed = true;
        console.say("console: the terminal input was closed; no more commands");
    }
}

/// The typing line: the terminal's last line is ours, the line above it
/// a divider, and everything the game prints scrolls in the lines above
/// (a "scrolling region", a standard terminal feature). We read single
/// keys and draw the typed text ourselves.
mod screen {
    use std::io::{IsTerminal, Write};
    use std::sync::mpsc::Sender;
    use std::sync::Mutex;

    use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
    use crossterm::terminal;

    struct Bottom {
        rows: u16,
        cols: u16,
        text: String,
        history: Vec<String>,
        /// Which history line Up/Down shows (`history.len()` = the new one).
        back: usize,
    }

    static BOTTOM: Mutex<Option<Bottom>> = Mutex::new(None);

    /// Sets the terminal up; false if it is not a terminal (or too small,
    /// or can't do it), so the plain line reading is used.
    pub fn open() -> bool {
        if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
            return false;
        }
        #[cfg(windows)]
        if !crossterm::ansi_support::supports_ansi() {
            return false;
        }
        let Ok((cols, rows)) = terminal::size() else { return false };
        if rows < 6 || cols < 20 || terminal::enable_raw_mode().is_err() {
            return false;
        }
        {
            let out = std::io::stdout();
            let mut out = out.lock();
            let _err = std::io::stderr().lock();
            // Room for the two lines we take, then the game's text goes on
            // from where it was, at most at the region's last line.
            let _ = write!(out, "\n\n");
            let _ = out.flush();
            let row = crossterm::cursor::position().map_or(rows - 3, |(_, r)| r.min(rows - 3));
            let _ = write!(out, "\x1b[?25l\x1b[1;{}r\x1b[{};1H", rows - 2, row + 1);
            let _ = out.flush();
        }
        *BOTTOM.lock().unwrap() = Some(Bottom { rows, cols, text: String::new(), history: Vec::new(), back: 0 });
        // A crash still gives the terminal back.
        let before = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            close();
            before(info);
        }));
        draw();
        true
    }

    /// Gives the terminal back as it was (whole screen scrolls, cursor
    /// shown, keys echoed). Safe to call more than once.
    pub fn close() {
        let Some(b) = BOTTOM.lock().ok().and_then(|mut g| g.take()) else { return };
        let out = std::io::stdout();
        let mut out = out.lock();
        let _err = std::io::stderr().lock();
        let _ = write!(out, "\x1b[r\x1b[{};1H\x1b[J\x1b[?25h", b.rows - 1);
        let _ = out.flush();
        let _ = terminal::disable_raw_mode();
    }

    /// Draws the divider and the typing line, then puts the cursor back
    /// where the game's text goes on. Holds the terminal's output locks so
    /// no game line is printed half-way through.
    fn draw() {
        let guard = BOTTOM.lock().unwrap();
        let Some(b) = guard.as_ref() else { return };
        let width = b.cols as usize;
        let label = " console: Enter runs, Up/Down history, Esc clears, 'help' lists commands ";
        let divider: String = format!("──{label}").chars().chain(std::iter::repeat('─')).take(width).collect();
        // The end of the text when it is longer than the line.
        let room = width.saturating_sub(4);
        let chars: Vec<char> = b.text.chars().collect();
        let shown: String = chars[chars.len().saturating_sub(room)..].iter().collect();
        let out = std::io::stdout();
        let mut out = out.lock();
        let _err = std::io::stderr().lock();
        let _ = write!(out, "\x1b7\x1b[{};1H\x1b[2K\x1b[2m{divider}\x1b[0m\x1b[{};1H\x1b[2K> {shown}\x1b[7m \x1b[0m\x1b8", b.rows - 1, b.rows);
        let _ = out.flush();
    }

    /// The window was resized: the region again, the cursor at its last
    /// line.
    fn resized(cols: u16, rows: u16) {
        {
            let mut guard = BOTTOM.lock().unwrap();
            let Some(b) = guard.as_mut() else { return };
            if rows < 6 || cols < 20 {
                return;
            }
            b.cols = cols;
            b.rows = rows;
            let out = std::io::stdout();
            let mut out = out.lock();
            let _err = std::io::stderr().lock();
            let _ = write!(out, "\x1b[?25l\x1b[1;{}r\x1b[{};1H", rows - 2, rows - 2);
            let _ = out.flush();
        }
        draw();
    }

    /// Reads keys until the terminal goes away; each Enter sends a line.
    pub fn read_keys(send: Sender<String>) {
        while let Ok(ev) = event::read() {
            let key = match ev {
                Event::Key(k) if k.kind != KeyEventKind::Release => k,
                Event::Resize(c, r) => {
                    resized(c, r);
                    continue;
                }
                _ => continue,
            };
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
            let mut line = None;
            {
                let mut guard = BOTTOM.lock().unwrap();
                let Some(b) = guard.as_mut() else { break };
                match key.code {
                    // Raw mode turns Ctrl+C into a key; it still stops the game.
                    KeyCode::Char('c') if ctrl => {
                        drop(guard);
                        close();
                        std::process::exit(130);
                    }
                    KeyCode::Enter => {
                        let t = std::mem::take(&mut b.text);
                        if !t.trim().is_empty() && b.history.last() != Some(&t) {
                            b.history.push(t.clone());
                        }
                        b.back = b.history.len();
                        line = Some(t);
                    }
                    KeyCode::Backspace => {
                        b.text.pop();
                    }
                    KeyCode::Esc => b.text.clear(),
                    KeyCode::Up if b.back > 0 => {
                        b.back -= 1;
                        b.text = b.history[b.back].clone();
                    }
                    KeyCode::Down if b.back < b.history.len() => {
                        b.back += 1;
                        b.text = b.history.get(b.back).cloned().unwrap_or_default();
                    }
                    KeyCode::Char(c) if !ctrl && !key.modifiers.contains(KeyModifiers::ALT) => b.text.push(c),
                    _ => {}
                }
            }
            draw();
            if let Some(line) = line {
                if send.send(line).is_err() {
                    break;
                }
            }
        }
    }
}
