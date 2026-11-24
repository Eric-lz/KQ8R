//! Full-motion AVI movie playback (`Replay Intro` and script `playMovie`,
//! `docs/behaviour/submenus.md` rules 26–28, M4 step 4g-5).
//!
//! Resolves `.dll` / `.avi` movie files in `<game>/` (`w32opn_1.avi`,
//! `W32mdl_1.avi`, `W32ndg_1.avi`), plays them through Windows MCI (`winmm.dll`),
//! polls `status kq8movie mode` every 50 ms until stopped, and supports
//! key/click skipping when `allow_skip` is true (`Replay Intro`).

use std::path::PathBuf;

use bevy::prelude::*;
use kq8_data::GameDir;

use crate::gui_view::{KeyAt, ScreenFrame};
use crate::menu_view::MenuView;
use crate::sound_view::SoundSim;
use crate::Shared;

const MCI_POLL_INTERVAL_MS: f64 = 50.0;
const SKIP_ARM_DELAY_MS: f64 = 150.0;

#[cfg(windows)]
mod winmm {
    #[link(name = "winmm")]
    extern "system" {
        pub fn mciSendStringW(
            cmd: *const u16,
            ret: *mut u16,
            ret_len: u32,
            cb: usize,
        ) -> u32;
    }

    #[link(name = "user32")]
    extern "system" {
        pub fn GetActiveWindow() -> isize;
    }
}

#[cfg(windows)]
fn mci_send(cmd: &str) -> Result<String, String> {
    let wide: Vec<u16> = cmd.encode_utf16().chain(std::iter::once(0)).collect();
    let mut buf = [0u16; 256];
    let rc = unsafe { winmm::mciSendStringW(wide.as_ptr(), buf.as_mut_ptr(), buf.len() as u32, 0) };
    if rc == 0 {
        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Ok(String::from_utf16_lossy(&buf[..len]).trim().to_string())
    } else {
        Err(format!("MCI error {rc} for command: {cmd}"))
    }
}

#[cfg(not(windows))]
fn mci_send(_cmd: &str) -> Result<String, String> {
    Err("MCI movie playback is only supported on Windows".into())
}

#[cfg(windows)]
fn active_hwnd() -> Option<isize> {
    let h = unsafe { winmm::GetActiveWindow() };
    (h != 0).then_some(h)
}

#[cfg(not(windows))]
fn active_hwnd() -> Option<isize> {
    None
}

/// Resolves `requested` (e.g. `"w32opn_1.dll"`, `"W32mdl_1.dll"`, `"W32ndg_1.dll"`)
/// in `<game>/`, preferring the H.264 `.avi` conversion shipped by GOG when a
/// `.dll` name is requested because Windows 11 lacks Indeo 5.0 (`iv50`).
pub fn resolve_movie_path(game: &GameDir, requested: &str) -> Option<PathBuf> {
    let trimmed = requested.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    if let Some(stem) = lower.strip_suffix(".dll") {
        let avi_name = format!("{stem}.avi");
        if let Some(p) = game.find(&avi_name) {
            return Some(p);
        }
    }
    game.find(trimmed)
}

#[derive(Debug)]
struct ActiveMovie {
    path: PathBuf,
    allow_skip: bool,
    from_replay_intro: bool,
    poll_timer_ms: f64,
    elapsed_ms: f64,
    skip_requested: bool,
    length_frames: u32,
}

#[derive(Resource, Default)]
pub struct MoviePlayer {
    active: Option<ActiveMovie>,
}

impl Drop for MoviePlayer {
    fn drop(&mut self) {
        self.stop();
    }
}

impl MoviePlayer {
    pub fn is_playing(&self) -> bool {
        self.active.is_some()
    }

    /// Opens and begins playback of `path` using Windows MCI (`kq8movie`).
    pub fn start(
        &mut self,
        path: PathBuf,
        allow_skip: bool,
        from_replay_intro: bool,
        sound_off: bool,
        client_dest: Option<[i32; 4]>,
    ) -> Result<u32, String> {
        self.stop();
        let abs = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        let mut abs_str = abs.to_string_lossy().to_string();
        if let Some(stripped) = abs_str.strip_prefix(r"\\?\") {
            abs_str = stripped.to_string();
        }

        let _ = mci_send("close kq8movie");

        let mut opened_child = false;
        if let (Some(hwnd), Some(_)) = (active_hwnd(), client_dest) {
            let cmd = format!("open \"{abs_str}\" type mpegvideo alias kq8movie style child parent {hwnd}");
            if mci_send(&cmd).is_ok() {
                opened_child = true;
            }
        }
        if !opened_child {
            let cmd = format!("open \"{abs_str}\" type mpegvideo alias kq8movie");
            mci_send(&cmd)?;
        }

        if opened_child {
            if let Some([x, y, w, h]) = client_dest {
                let _ = mci_send(&format!("put kq8movie destination at {x} {y} {w} {h}"));
            }
        }
        if sound_off {
            let _ = mci_send("setaudio kq8movie off");
        }

        let length_frames = mci_send("status kq8movie length")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        mci_send("play kq8movie")?;

        self.active = Some(ActiveMovie {
            path,
            allow_skip,
            from_replay_intro,
            poll_timer_ms: MCI_POLL_INTERVAL_MS,
            elapsed_ms: 0.0,
            skip_requested: false,
            length_frames,
        });
        Ok(length_frames)
    }

    /// Stops and closes any active MCI movie (`submenus.md` rule 28).
    pub fn stop(&mut self) {
        if self.active.take().is_some() {
            let _ = mci_send("stop kq8movie");
            let _ = mci_send("close kq8movie");
        }
    }
}

/// Polls active MCI playback (`50 ms` interval), handles `playMovie` script
/// requests, and triggers `Replay Intro` post-movie 200 ms theme restart.
#[allow(clippy::too_many_arguments)]
pub fn movie_step(
    mut movie: Option<ResMut<MoviePlayer>>,
    mut menu: Option<ResMut<MenuView>>,
    mut sim: Option<ResMut<crate::door_view::DoorSim>>,
    mut sound: Option<ResMut<SoundSim>>,
    time: Res<Time<Real>>,
    (keys, buttons, window, frame, mut key_at): (
        Res<ButtonInput<KeyCode>>,
        Res<ButtonInput<MouseButton>>,
        Query<&Window, With<bevy::window::PrimaryWindow>>,
        Res<ScreenFrame>,
        Option<ResMut<KeyAt>>,
    ),
    shared: Res<Shared>,
) {
    let Some(movie) = movie.as_deref_mut() else { return };

    // 1. Drain queued `playMovie <filename>` script requests (`allowSkip = false`).
    if let Some(s) = sim.as_deref_mut() {
        let requests = std::mem::take(&mut s.active.world.movie_requests);
        if !requests.is_empty() && !movie.is_playing() {
            let game = menu.as_ref().map(|m| m.game.clone());
            let sound_off = sound.as_ref().is_some_and(|snd| snd.output_off());
            if let (Some((name, allow_skip)), Some(game)) = (requests.into_iter().next(), game) {
                if let Some(path) = resolve_movie_path(&game, &name) {
                    if let Some(snd) = sound.as_deref_mut() {
                        snd.set_game_paused(true);
                    }
                    let dest = movie_client_dest(&window, &frame);
                    match movie.start(path.clone(), allow_skip, false, sound_off, dest) {
                        Ok(len) => shared.log.lock().unwrap().summary(format!(
                            "movie: playMovie {name} -> {} ({len} frames, allowSkip={allow_skip}; submenus.md rule 27)",
                            path.display()
                        )),
                        Err(e) => {
                            shared.log.lock().unwrap().summary(format!("movie: failed to play {name}: {e}"));
                            if let Some(snd) = sound.as_deref_mut() {
                                snd.set_game_paused(false);
                            }
                        }
                    }
                } else {
                    shared.log.lock().unwrap().summary(format!(
                        "movie: playMovie {name}: not found in game folder; continuing without error (submenus.md rule 28)"
                    ));
                }
            }
        }
    }

    // 2. Drive active playback (`skip` and `50 ms` MCI mode polling).
    let Some(active) = movie.active.as_mut() else { return };
    let dt_ms = time.delta_secs_f64() * 1000.0;
    active.elapsed_ms += dt_ms;

    // Test hook `--key menu:movie:skip,N` or `--key movie:skip,N`.
    if let Some(ka) = key_at.as_mut() {
        let n_frame = menu.as_ref().map_or(0, |m| m.total_frames);
        if let Some(pos) = ka.0.iter().position(|(name, n)| {
            let lower = name.to_ascii_lowercase();
            (lower == "menu:movie:skip" || lower == "movie:skip") && n_frame >= *n
        }) {
            let (cmd, _) = ka.0.remove(pos);
            active.skip_requested = true;
            if let Some(m) = menu.as_deref_mut() {
                m.note(format!("test --key {cmd}"));
            }
        }
    }

    let mut finished_reason: Option<&'static str> = None;
    if active.allow_skip {
        let manual_input = active.elapsed_ms >= SKIP_ARM_DELAY_MS
            && (keys.get_just_pressed().next().is_some()
                || buttons.just_pressed(MouseButton::Left)
                || buttons.just_pressed(MouseButton::Right));
        if active.skip_requested || manual_input {
            finished_reason = Some("skipped");
        }
    }

    if finished_reason.is_none() {
        active.poll_timer_ms -= dt_ms;
        if active.poll_timer_ms <= 0.0 {
            active.poll_timer_ms += MCI_POLL_INTERVAL_MS;
            match mci_send("status kq8movie mode") {
                Ok(mode) if mode.eq_ignore_ascii_case("stopped") => {
                    finished_reason = Some("finished");
                }
                Ok(_) => {}
                Err(_) => {
                    finished_reason = Some("stopped");
                }
            }
        }
    }

    if let Some(reason) = finished_reason {
        let path = active.path.clone();
        let len = active.length_frames;
        let from_replay = active.from_replay_intro;
        movie.stop();

        if from_replay {
            if let Some(m) = menu.as_deref_mut() {
                let more = m.state.finish_replay_intro();
                for a in more {
                    if let kq8_core::main_menu::MenuAction::Note(line) = a {
                        m.note(line);
                    }
                }
                m.theme_restart_timer_ms = Some(200.0);
                m.mark_dirty();
                m.summary(
                    &shared,
                    format!(
                        "Replay Intro {reason} ({}, {len} frames): MCI closed, theme.aud restart scheduled in 200 ms (submenus.md rules 27-28)",
                        path.display()
                    ),
                );
            }
        } else {
            if let Some(snd) = sound.as_deref_mut() {
                snd.set_game_paused(false);
            }
            shared.log.lock().unwrap().summary(format!(
                "movie: playMovie {reason} ({}, {len} frames); gameplay resumed",
                path.display()
            ));
        }
    }
}

pub fn movie_client_dest(
    window: &Query<&Window, With<bevy::window::PrimaryWindow>>,
    frame: &ScreenFrame,
) -> Option<[i32; 4]> {
    let w = window.single().ok()?;
    let scale = w.scale_factor();
    let [fl, ft, fw, fh] = frame.0;
    let pw = (fw * scale).round() as i32;
    let ph = (fh * scale).round() as i32;
    if pw <= 0 || ph <= 0 {
        return None;
    }
    Some([
        (fl * scale).round() as i32,
        (ft * scale).round() as i32,
        pw,
        ph,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn resolve_movie_path_prefers_avi_over_dll_and_handles_missing() {
        let uniq = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("kq8_movie_test_{uniq}"));
        std::fs::create_dir_all(dir.join("data")).unwrap();
        std::fs::write(dir.join("Mask.exe"), b"").unwrap();
        let avi = dir.join("w32opn_1.avi");
        let dll = dir.join("w32opn_1.dll");
        let mdl_dll = dir.join("W32mdl_1.dll");
        std::fs::write(&avi, b"AVI").unwrap();
        std::fs::write(&dll, b"DLL").unwrap();
        std::fs::write(&mdl_dll, b"DLL").unwrap();

        let game = GameDir::open(&dir).unwrap();
        let picked_opn = resolve_movie_path(&game, "w32opn_1.dll").unwrap();
        assert_eq!(
            picked_opn.file_name().unwrap().to_str().unwrap().to_ascii_lowercase(),
            "w32opn_1.avi"
        );
        let picked_mdl = resolve_movie_path(&game, "W32mdl_1.dll").unwrap();
        assert_eq!(
            picked_mdl.file_name().unwrap().to_str().unwrap().to_ascii_lowercase(),
            "w32mdl_1.dll"
        );
        assert!(resolve_movie_path(&game, "nonexistent.dll").is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
