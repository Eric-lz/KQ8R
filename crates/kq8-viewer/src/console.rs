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
    /// Starts the thread that reads the terminal.
    pub fn start() -> Self {
        let (send, lines) = mpsc::channel();
        let spawned = std::thread::Builder::new().name("console".into()).spawn(move || {
            for line in std::io::stdin().lines() {
                let Ok(line) = line else { break };
                if send.send(line).is_err() {
                    break;
                }
            }
        });
        let mut log = Log::create("console").ok();
        let first = match spawned {
            Ok(_) => "console: type commands in this terminal and press Enter ('help' for the list)",
            Err(_) => "console: could not start the thread that reads the terminal; no console",
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
