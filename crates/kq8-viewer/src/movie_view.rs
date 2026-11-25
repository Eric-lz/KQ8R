//! Full-motion AVI movie playback (`Replay Intro` and script `playMovie`,
//! `docs/behaviour/submenus.md` rules 26–28, M4 step 4g-5 + 4g-F1).
//!
//! Resolves `.dll` / `.avi` movie files in `<game>/` (`w32opn_1.avi`,
//! `W32mdl_1.avi`, `W32ndg_1.avi`), opens them on Bevy's main Win32 thread
//! (`MciThreadAnchor`) as a borderless, non-activating, click-transparent owned
//! popup overlay aligned to the 4:3 [`ScreenFrame`] inside the game window,
//! polls `status kq8movie mode` / `position` every 50 ms until stopped, and
//! supports key/click skipping when `allow_skip` is true (`Replay Intro`).

use std::path::PathBuf;
#[cfg(windows)]
use std::sync::Mutex;

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
    #[repr(C)]
    #[derive(Default, Copy, Clone)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    #[repr(C)]
    #[derive(Default, Copy, Clone)]
    pub struct Rect {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    pub const GW_OWNER: u32 = 4;
    pub const GWL_STYLE: i32 = -16;
    pub const GWL_EXSTYLE: i32 = -20;
    pub const WS_POPUP: isize = 0x8000_0000_u32 as isize;
    pub const WS_BORDER_CAPTION_FRAME: isize = 0x00CF_0000;
    pub const WS_EX_TRANSPARENT: isize = 0x0000_0020;
    pub const WS_EX_NOACTIVATE: isize = 0x0800_0000;
    pub const SWP_NOZORDER: u32 = 0x0004;
    pub const SWP_NOACTIVATE: u32 = 0x0010;
    pub const SWP_FRAMECHANGED: u32 = 0x0020;
    pub const SWP_SHOWWINDOW: u32 = 0x0040;
    pub const SW_HIDE: i32 = 0;

    pub const EVENT_OBJECT_LOCATIONCHANGE: u32 = 0x800B;
    pub const EVENT_SYSTEM_MINIMIZESTART: u32 = 0x0016;
    pub const EVENT_SYSTEM_MINIMIZEEND: u32 = 0x0017;
    pub const WINEVENT_OUTOFCONTEXT: u32 = 0x0000;
    pub const OBJID_WINDOW: i32 = 0;
    pub const CHILDID_SELF: i32 = 0;

    #[link(name = "winmm")]
    extern "system" {
        pub fn mciSendStringW(
            cmd: *const u16,
            ret: *mut u16,
            ret_len: u32,
            cb: usize,
        ) -> u32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        pub fn GetCurrentProcessId() -> u32;
        pub fn GetCurrentThreadId() -> u32;
    }

    #[link(name = "user32")]
    extern "system" {
        pub fn GetActiveWindow() -> isize;
        pub fn EnumThreadWindows(
            dw_thread_id: u32,
            lpfn: unsafe extern "system" fn(isize, isize) -> i32,
            l_param: isize,
        ) -> i32;
        pub fn IsWindowVisible(hwnd: isize) -> i32;
        pub fn IsIconic(hwnd: isize) -> i32;
        pub fn ShowWindow(hwnd: isize, cmd_show: i32) -> i32;
        pub fn GetParent(hwnd: isize) -> isize;
        pub fn GetWindow(hwnd: isize, cmd: u32) -> isize;
        pub fn GetClientRect(hwnd: isize, rect: *mut Rect) -> i32;
        pub fn ClientToScreen(hwnd: isize, point: *mut Point) -> i32;
        #[cfg(target_pointer_width = "64")]
        pub fn GetWindowLongPtrW(hwnd: isize, index: i32) -> isize;
        #[cfg(target_pointer_width = "64")]
        pub fn SetWindowLongPtrW(hwnd: isize, index: i32, new_long: isize) -> isize;
        pub fn SetWindowPos(
            hwnd: isize,
            insert_after: isize,
            x: i32,
            y: i32,
            cx: i32,
            cy: i32,
            flags: u32,
        ) -> i32;
        pub fn SetFocus(hwnd: isize) -> isize;
        pub fn SetWinEventHook(
            event_min: u32,
            event_max: u32,
            hmod_win_event_proc: isize,
            pfn_win_event_proc: unsafe extern "system" fn(isize, u32, isize, i32, i32, u32, u32),
            id_process: u32,
            id_thread: u32,
            dw_flags: u32,
        ) -> isize;
        pub fn UnhookWinEvent(h_win_event_hook: isize) -> i32;
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
unsafe extern "system" fn pick_thread_window(hwnd: isize, l_param: isize) -> i32 {
    if winmm::GetParent(hwnd) == 0
        && winmm::GetWindow(hwnd, winmm::GW_OWNER) == 0
        && winmm::IsWindowVisible(hwnd) != 0
    {
        *(l_param as *mut isize) = hwnd;
        return 0;
    }
    1
}

#[cfg(windows)]
fn primary_win32_hwnd() -> Option<isize> {
    let active = unsafe { winmm::GetActiveWindow() };
    if active != 0 && unsafe { winmm::GetParent(active) } == 0 {
        return Some(active);
    }
    let mut found: isize = 0;
    unsafe {
        let tid = winmm::GetCurrentThreadId();
        winmm::EnumThreadWindows(tid, pick_thread_window, (&mut found as *mut isize) as isize);
    }
    (found != 0).then_some(found)
}

#[cfg(not(windows))]
fn primary_win32_hwnd() -> Option<isize> {
    None
}

#[cfg(windows)]
#[derive(Debug, Clone, Copy)]
struct OverlayTrack {
    owner_hwnd: isize,
    mci_hwnd: isize,
    last_screen_rect: Option<[i32; 4]>,
    hidden_iconic: bool,
}

#[cfg(windows)]
static ACTIVE_OVERLAY: Mutex<Option<OverlayTrack>> = Mutex::new(None);

#[cfg(windows)]
fn overlay_screen_rect(owner_hwnd: isize) -> Option<[i32; 4]> {
    let mut rc = winmm::Rect::default();
    if unsafe { winmm::GetClientRect(owner_hwnd, &mut rc) } == 0 {
        return None;
    }
    let cw = (rc.right - rc.left).max(0) as f32;
    let ch = (rc.bottom - rc.top).max(0) as f32;
    if cw <= 0.0 || ch <= 0.0 {
        return None;
    }
    let [fl, ft, fw, fh] = kq8_core::panels::screen_frame([cw, ch]);
    let pw = fw.round() as i32;
    let ph = fh.round() as i32;
    if pw <= 0 || ph <= 0 {
        return None;
    }
    let mut pt = winmm::Point {
        x: fl.round() as i32,
        y: ft.round() as i32,
    };
    unsafe {
        winmm::ClientToScreen(owner_hwnd, &mut pt);
    }
    Some([pt.x, pt.y, pw, ph])
}

#[cfg(windows)]
fn sync_overlay_window(make_borderless: bool) {
    let mut dest_resize: Option<(i32, i32)> = None;
    let (owner_hwnd, mci_hwnd, screen_rect, apply_show) = {
        let Ok(mut guard) = ACTIVE_OVERLAY.try_lock() else {
            return;
        };
        let Some(track) = guard.as_mut() else {
            return;
        };
        if unsafe { winmm::IsIconic(track.owner_hwnd) } != 0 {
            if !track.hidden_iconic {
                unsafe {
                    winmm::ShowWindow(track.mci_hwnd, winmm::SW_HIDE);
                }
                track.hidden_iconic = true;
            }
            return;
        }
        let Some(rect) = overlay_screen_rect(track.owner_hwnd) else {
            return;
        };
        let size_changed = track
            .last_screen_rect
            .is_none_or(|prev| prev[2] != rect[2] || prev[3] != rect[3]);
        let moved_or_restored =
            make_borderless || track.hidden_iconic || track.last_screen_rect != Some(rect);
        if !moved_or_restored {
            return;
        }
        if size_changed || make_borderless || track.hidden_iconic {
            dest_resize = Some((rect[2], rect[3]));
        }
        track.last_screen_rect = Some(rect);
        track.hidden_iconic = false;
        (track.owner_hwnd, track.mci_hwnd, rect, true)
    };
    let _ = owner_hwnd;
    if make_borderless {
        #[cfg(target_pointer_width = "64")]
        unsafe {
            let style = winmm::GetWindowLongPtrW(mci_hwnd, winmm::GWL_STYLE);
            let new_style = (style & !winmm::WS_BORDER_CAPTION_FRAME) | winmm::WS_POPUP;
            winmm::SetWindowLongPtrW(mci_hwnd, winmm::GWL_STYLE, new_style);

            let ex_style = winmm::GetWindowLongPtrW(mci_hwnd, winmm::GWL_EXSTYLE);
            let new_ex = ex_style | winmm::WS_EX_NOACTIVATE | winmm::WS_EX_TRANSPARENT;
            winmm::SetWindowLongPtrW(mci_hwnd, winmm::GWL_EXSTYLE, new_ex);
        }
    }
    if apply_show {
        unsafe {
            let flags = winmm::SWP_NOACTIVATE
                | winmm::SWP_NOZORDER
                | winmm::SWP_SHOWWINDOW
                | if make_borderless { winmm::SWP_FRAMECHANGED } else { 0 };
            winmm::SetWindowPos(
                mci_hwnd,
                0,
                screen_rect[0],
                screen_rect[1],
                screen_rect[2],
                screen_rect[3],
                flags,
            );
        }
    }
    if let Some((pw, ph)) = dest_resize {
        let _ = mci_send(&format!("put kq8movie destination at 0 0 {pw} {ph}"));
    }
}

#[cfg(windows)]
unsafe extern "system" fn owner_move_winevent(
    _hook: isize,
    _event: u32,
    hwnd: isize,
    id_object: i32,
    id_child: i32,
    _thread_id: u32,
    _time_ms: u32,
) {
    if id_object != winmm::OBJID_WINDOW || id_child != winmm::CHILDID_SELF {
        return;
    }
    let matches_owner = ACTIVE_OVERLAY
        .try_lock()
        .ok()
        .and_then(|g| g.map(|t| t.owner_hwnd == hwnd))
        .unwrap_or(false);
    if matches_owner {
        sync_overlay_window(false);
    }
}

#[cfg(windows)]
fn install_overlay_hooks(owner_hwnd: isize, mci_hwnd: isize) -> Vec<isize> {
    if let Ok(mut guard) = ACTIVE_OVERLAY.lock() {
        *guard = Some(OverlayTrack {
            owner_hwnd,
            mci_hwnd,
            last_screen_rect: None,
            hidden_iconic: false,
        });
    }
    sync_overlay_window(true);
    let (pid, tid) = unsafe { (winmm::GetCurrentProcessId(), winmm::GetCurrentThreadId()) };
    let mut hooks = Vec::with_capacity(2);
    for (min_ev, max_ev) in [
        (winmm::EVENT_OBJECT_LOCATIONCHANGE, winmm::EVENT_OBJECT_LOCATIONCHANGE),
        (winmm::EVENT_SYSTEM_MINIMIZESTART, winmm::EVENT_SYSTEM_MINIMIZEEND),
    ] {
        let h = unsafe {
            winmm::SetWinEventHook(
                min_ev,
                max_ev,
                0,
                owner_move_winevent,
                pid,
                tid,
                winmm::WINEVENT_OUTOFCONTEXT,
            )
        };
        if h != 0 {
            hooks.push(h);
        }
    }
    hooks
}

#[cfg(windows)]
fn clear_overlay_hooks(hooks: &[isize]) {
    for &h in hooks {
        if h != 0 {
            unsafe {
                winmm::UnhookWinEvent(h);
            }
        }
    }
    if let Ok(mut guard) = ACTIVE_OVERLAY.lock() {
        *guard = None;
    }
}

#[cfg(not(windows))]
fn sync_overlay_window(_make_borderless: bool) {}

#[cfg(not(windows))]
fn install_overlay_hooks(_owner_hwnd: isize, _mci_hwnd: isize) -> Vec<isize> {
    Vec::new()
}

#[cfg(not(windows))]
fn clear_overlay_hooks(_hooks: &[isize]) {}

/// Resolves `requested` (e.g. `"w32opn_1.dll"`, `"W32mdl_1.dll"`, `"W32ndg_1.dll"`)
/// in `<game>/`, preferring the `.avi` conversion shipped by GOG when a
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

/// Main-thread anchor (`!Send`) pinning [`movie_step`] to Bevy's main Win32
/// thread so Windows MCI aliases (`kq8movie`) are always opened, polled, and
/// closed on the same OS thread that runs the Win32 message pump.
#[derive(Default)]
pub struct MciThreadAnchor(std::marker::PhantomData<*const ()>);

#[derive(Debug)]
struct PendingMovie {
    path: PathBuf,
    allow_skip: bool,
    from_replay_intro: bool,
    sound_off: bool,
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
    hooks: Vec<isize>,
}

#[derive(Resource, Default)]
pub struct MoviePlayer {
    pending: Option<PendingMovie>,
    active: Option<ActiveMovie>,
}

impl Drop for MoviePlayer {
    fn drop(&mut self) {
        self.stop();
    }
}

impl MoviePlayer {
    pub fn is_playing(&self) -> bool {
        self.active.is_some() || self.pending.is_some()
    }

    /// Queues a movie to be opened on the main Win32 thread in [`movie_step`].
    pub fn queue_start(
        &mut self,
        path: PathBuf,
        allow_skip: bool,
        from_replay_intro: bool,
        sound_off: bool,
    ) {
        self.pending = Some(PendingMovie {
            path,
            allow_skip,
            from_replay_intro,
            sound_off,
        });
    }

    /// Opens and begins playback of `path` using Windows MCI (`kq8movie`).
    pub fn start(
        &mut self,
        path: PathBuf,
        allow_skip: bool,
        from_replay_intro: bool,
        sound_off: bool,
        _client_dest: Option<[i32; 4]>,
    ) -> Result<u32, String> {
        self.stop();
        let abs = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        let mut abs_str = abs.to_string_lossy().to_string();
        if let Some(stripped) = abs_str.strip_prefix(r"\\?\") {
            abs_str = stripped.to_string();
        }

        let _ = mci_send("close kq8movie");

        let owner_hwnd = primary_win32_hwnd();
        let mut opened_popup = false;
        if let Some(hwnd) = owner_hwnd {
            let cmd = format!(
                "open \"{abs_str}\" type mpegvideo alias kq8movie parent {hwnd} style popup"
            );
            if mci_send(&cmd).is_ok() {
                opened_popup = true;
            }
        }
        if !opened_popup {
            let cmd = format!("open \"{abs_str}\" type mpegvideo alias kq8movie");
            mci_send(&cmd)?;
        }

        let mci_hwnd = mci_send("status kq8movie window handle")
            .ok()
            .and_then(|s| s.parse::<isize>().ok())
            .filter(|&h| h != 0);

        let hooks = if opened_popup {
            if let (Some(owner), Some(mci_w)) = (owner_hwnd, mci_hwnd) {
                install_overlay_hooks(owner, mci_w)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        if sound_off {
            let _ = mci_send("setaudio kq8movie off");
        }

        let length_frames = mci_send("status kq8movie length")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        mci_send("play kq8movie")?;

        #[cfg(windows)]
        if let Some(owner) = owner_hwnd {
            unsafe {
                winmm::SetFocus(owner);
            }
        }

        self.active = Some(ActiveMovie {
            path,
            allow_skip,
            from_replay_intro,
            poll_timer_ms: MCI_POLL_INTERVAL_MS,
            elapsed_ms: 0.0,
            skip_requested: false,
            length_frames,
            hooks,
        });
        Ok(length_frames)
    }

    /// Stops and closes any active MCI movie (`submenus.md` rule 28).
    pub fn stop(&mut self) {
        self.pending = None;
        if let Some(active) = self.active.take() {
            clear_overlay_hooks(&active.hooks);
            let _ = mci_send("stop kq8movie");
            let _ = mci_send("close kq8movie");
        }
    }
}

/// Polls active MCI playback (`50 ms` interval), handles queued `Replay Intro`
/// and `playMovie` requests on Bevy's main Win32 thread, and triggers
/// `Replay Intro` post-movie 200 ms theme restart.
#[allow(clippy::too_many_arguments)]
pub fn movie_step(
    mut movie: Option<ResMut<MoviePlayer>>,
    mut menu: Option<ResMut<MenuView>>,
    mut sim: Option<ResMut<crate::door_view::DoorSim>>,
    mut sound: Option<ResMut<SoundSim>>,
    time: Res<Time<Real>>,
    (keys, buttons, window, frame, mut key_at, _anchor): (
        Res<ButtonInput<KeyCode>>,
        Res<ButtonInput<MouseButton>>,
        Query<&Window, With<bevy::window::PrimaryWindow>>,
        Res<ScreenFrame>,
        Option<ResMut<KeyAt>>,
        Option<NonSend<MciThreadAnchor>>,
    ),
    shared: Res<Shared>,
) {
    let Some(movie) = movie.as_deref_mut() else { return };

    // 0. Start any queued `Replay Intro` movie on the main Win32 thread.
    if let Some(req) = movie.pending.take() {
        let dest = movie_client_dest(&window, &frame);
        match movie.start(
            req.path.clone(),
            req.allow_skip,
            req.from_replay_intro,
            req.sound_off,
            dest,
        ) {
            Ok(len) => {
                let msg = format!(
                    "Replay Intro started {} ({len} frames, allowSkip={}; theme.aud faded over 1 s, submenus.md rules 27-28)",
                    req.path.display(),
                    req.allow_skip,
                );
                if let Some(m) = menu.as_deref_mut() {
                    m.summary(&shared, msg);
                } else {
                    shared.log.lock().unwrap().summary(format!("movie: {msg}"));
                }
            }
            Err(e) => {
                if let Some(m) = menu.as_deref_mut() {
                    m.summary(
                        &shared,
                        format!("Replay Intro could not start {}: {e}", req.path.display()),
                    );
                    if req.from_replay_intro {
                        let more = m.state.finish_replay_intro();
                        for a in more {
                            if let kq8_core::main_menu::MenuAction::Note(n) = a {
                                m.note(n);
                            }
                        }
                        m.theme_restart_timer_ms = Some(200.0);
                        m.mark_dirty();
                    }
                } else {
                    shared.log.lock().unwrap().summary(format!(
                        "movie: failed to start {}: {e}",
                        req.path.display()
                    ));
                }
            }
        }
    }

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

    // 2. Drive active playback (keep overlay aligned, check skip, poll MCI).
    let Some(active) = movie.active.as_mut() else { return };
    sync_overlay_window(false);

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
                Ok(_) => {
                    if active.length_frames > 0 {
                        if let Ok(pos_s) = mci_send("status kq8movie position") {
                            if let Ok(pos) = pos_s.parse::<u32>() {
                                if pos >= active.length_frames {
                                    finished_reason = Some("finished");
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    shared.log.lock().unwrap().summary(format!("movie: MCI status error: {e}"));
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
                    if let kq8_core::main_menu::MenuAction::Note(n) = a {
                        m.note(n);
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
