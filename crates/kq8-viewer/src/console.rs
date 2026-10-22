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

/// The lines typed so far, and `console.log`.
#[derive(Resource)]
pub struct Console {
    lines: Mutex<Receiver<String>>,
    /// Typed lines held back because a world change was asked before
    /// them (M6 6e-3 fix): they run in the next world.
    waiting: Vec<String>,
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
        Console { lines: Mutex::new(lines), waiting: Vec::new(), log, closed: false }
    }

    fn say(&mut self, line: impl AsRef<str>) {
        println!("{}", line.as_ref());
        if let Some(log) = self.log.as_mut() {
            log.detail(line);
        }
    }
}

/// `--console "WORDS",N` (M6 6e-2 test): a console line typed before
/// update N + 1 of the world shown then; used once.
#[derive(Resource, Default)]
pub struct ConsoleAt(pub Vec<(String, u64)>);

/// Runs every line typed since the last update.
pub fn console_step(
    console: Option<ResMut<Console>>,
    sim: Option<ResMut<DoorSim>>,
    connor: Option<Res<ConnorSim>>,
    (mut at, mut change): (ResMut<ConsoleAt>, ResMut<crate::world_build::WorldChange>),
) {
    let Some(mut console) = console else { return };
    // Lines typed while the world loads wait in the channel until it runs.
    if sim.is_none() {
        return;
    }
    let mut typed = std::mem::take(&mut console.waiting);
    let mut ended = false;
    // Once the terminal is closed only `--console` lines come.
    if !console.closed {
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
    // `--console`: the lines due now, after the typed ones.
    let now = sim.as_ref().map_or(0, |s| s.active.world.updates);
    let (due, later): (Vec<_>, Vec<_>) = std::mem::take(&mut at.0).into_iter().partition(|(_, n)| now >= *n);
    at.0 = later;
    // Typed lines first, then the `--console` ones (with their update, to
    // hold them back if a world change comes first).
    let mut lines: Vec<(String, Option<u64>)> = typed.into_iter().map(|l| (l, None)).collect();
    lines.extend(due.into_iter().map(|(line, n)| (line, Some(n))));
    let mut sim = sim;
    let mut lines = lines.into_iter();
    while let Some((line, _)) = lines.next() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let updates = sim.as_ref().map_or(0, |s| s.active.world.updates);
        console.say(format!("> {line}  (update {updates})"));
        let Some(sim) = sim.as_mut() else { continue };
        // A world change (M6 6e-2) is asked for here; it happens at the end
        // of the frame (`world_build::world_change_step`).
        if let Some(said) = world_word(line, &mut change, connor.as_deref()) {
            console.say(said);
            if change.asked.is_some() {
                // The lines after it wait for the next world (a
                // `--console` line counts its update there again).
                for (rest, n) in lines.by_ref() {
                    match n {
                        Some(n) => at.0.push((rest, n)),
                        None => console.waiting.push(rest),
                    }
                }
            }
            continue;
        }
        // Our own words first, else a script command; the log lines it
        // writes are shown here too (they also go to doors.log as usual).
        let said = match crate::console_words::run(line, sim, connor.as_deref()) {
            Some(said) => said,
            None => crate::console_words::script(sim, line),
        };
        for s in said {
            console.say(s);
        }
    }
    // After the lines, so lines typed just before the end still run.
    if ended {
        console.closed = true;
        console.say("console: the terminal input was closed; no more commands");
    }
}

/// `reload`: take this world down and build it again, Connor where he
/// stands (M6 6e-2). `goworld <World> [x y heading]`: to that world,
/// Connor on the ground at x, y, or where its own start puts him (6e-3).
/// Connor's things go along (the journey). `None`: not a world word.
fn world_word(line: &str, change: &mut crate::world_build::WorldChange, connor: Option<&ConnorSim>) -> Option<String> {
    use crate::world_build::Request;
    let words: Vec<&str> = line.split_whitespace().collect();
    let first = words.first()?.to_ascii_lowercase();
    match first.as_str() {
        "reload" => {
            let at = connor.map(|c| [c.mover.position[0], c.mover.position[1], c.walker.heading]);
            let world = change.current.clone();
            change.asked = Some(Request { world: world.clone(), at, why: "console reload".into(), portal: None });
            Some(match at {
                Some([x, y, h]) => format!("reload: {world} is taken down and built again at the end of this frame; Connor back at {x:.0} {y:.0}, turn {h:.4}"),
                None => format!("reload: {world} is taken down and built again at the end of this frame"),
            })
        }
        "goworld" if words.get(1).is_some_and(|w| w.eq_ignore_ascii_case("portal")) => {
            // `goworld portal <file>` (6e-4b): through that portal file, as
            // a portal's hand-over would (world-change.md rule 6).
            let Some(file) = words.get(2) else { return Some("usage: goworld portal <file.kq>, e.g. goworld portal Davtele.kq".into()) };
            let read = change.game().ok_or("no game folder known".to_string()).and_then(|g| kq8_data::GameDir::open(g).map_err(|e| e.to_string())).and_then(|g| crate::portal_view::read_portal(&g, &change.current, file));
            Some(match read {
                Ok((keys, to)) => {
                    let arrival = crate::portal_view::PortalArrival::from_keys(&keys);
                    let line = format!("goworld portal {file}: [{}] to {to} at the end of this frame; Connor at {:?}", keys.name, arrival.con_loc);
                    change.asked = Some(Request { world: to, at: None, why: format!("console goworld portal {file}"), portal: Some(Box::new(arrival)) });
                    line
                }
                Err(e) => format!("goworld portal {file}: {e}"),
            })
        }
        "goworld" => {
            let usage = format!("usage: goworld <world> [x y heading]; worlds: {}", kq8_data::game_dir::WORLDS.join(", "));
            let Some(world) = words.get(1).and_then(|w| kq8_data::game_dir::WORLDS.iter().find(|n| n.eq_ignore_ascii_case(w))) else { return Some(usage) };
            let numbers: Vec<f32> = words[2..].iter().filter_map(|w| w.parse::<f32>().ok()).collect();
            let at = match numbers[..] {
                [] => None,
                [x, y] => Some([x, y, 0.0]),
                [x, y, h] => Some([x, y, h]),
                _ => return Some(usage),
            };
            change.asked = Some(Request { world: world.to_string(), at, why: "console goworld".into(), portal: None });
            Some(match at {
                Some([x, y, h]) => format!("goworld: to {world} at the end of this frame, Connor at {x:.0} {y:.0}, turn {h:.4}"),
                None => format!("goworld: to {world} at the end of this frame, Connor where its start puts him"),
            })
        }
        _ => None,
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
