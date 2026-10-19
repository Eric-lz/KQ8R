//! Which interface panels are shown, and their sliding (M4 step 4c-2).
//!
//! Rules from `docs/behaviour/interface.md` (A, 2026-10-18; checks 3, 4
//! and 7 done by the user in the original):
//!
//! - Panels move in groups: the top bar; the bottom bar (bottom left, the
//!   eight slots, bottom right and the meters together); the meters
//!   alone; the map; the caption; the help popup (rule 8).
//! - They slide, 0.2 s of real time for the whole distance; a panel asked
//!   to show while it is showing or sliding in is left alone, the same for
//!   hiding. The top bar plays `intrup.aud` when it starts to show and
//!   `intrdown.aud` when it starts to hide; the map `map.aud` /
//!   `mapbgone.aud`; the bottom panels nothing. Hidden, the top bar sits
//!   1/480 of the screen above its own height, the bottom panels just
//!   below the bottom edge, the map 50/640 of the screen beyond its own
//!   width to the left (rule 9).
//! - Space puts the bottom bar away or brings it back (rule 10).
//! - `handsOff 1`: a shown bottom bar is remembered and slides away, the
//!   map is hidden, the spinning mask shows; `handsOff 0`: the bottom bar
//!   comes back only if it was shown, the mask goes (rule 15).
//! - A conversation puts the bottom bar away (remembered) and brings it
//!   back at its end if it was shown and is now hidden; neither during
//!   handsOff (rule 16).
//! - A health or experience percent sent while the meters are hidden
//!   slides them up alone and away 1.5 s (real time) later; bringing the
//!   bottom bar back clears that timer (rule 25). A new percent while they
//!   are popped up renews the 1.5 s (rule 25, "inferred" by A).
//! - Scripts: `setLowerGui`, `setUpperGui`, `setMapGui` 0|1, and
//!   `getGuiStatus` (rule 15).
//!
//! Real time is counted in milliseconds by the caller; the game has no
//! pause yet ("Do not reproduce": pause the timers when the game pauses).

/// Rule 9: a whole slide.
pub const SLIDE_MS: f64 = 200.0;

/// Rules 13, 25: how long a panel that popped up for news stays.
pub const POP_MS: f64 = 1500.0;

/// One panel of the game-play screen (rule 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Top,
    BottomLeft,
    Slots,
    Meters,
    BottomRight,
    Map,
    Caption,
    Popup,
}

impl Panel {
    pub const ALL: [Panel; 8] = [Panel::Top, Panel::BottomLeft, Panel::Slots, Panel::Meters, Panel::BottomRight, Panel::Map, Panel::Caption, Panel::Popup];

    fn index(self) -> usize {
        Panel::ALL.iter().position(|p| *p == self).expect("every panel is listed")
    }

    pub fn name(self) -> &'static str {
        match self {
            Panel::Top => "top bar",
            Panel::BottomLeft => "bottom left",
            Panel::Slots => "eight slots",
            Panel::Meters => "meters",
            Panel::BottomRight => "bottom right",
            Panel::Map => "map",
            Panel::Caption => "caption",
            Panel::Popup => "popup",
        }
    }

    /// The layout's id name for this panel (`Gplayscr.gui`).
    pub fn layout_id(self) -> &'static str {
        match self {
            Panel::Top => "IDCTL_INVBART",
            Panel::BottomLeft => "IDCTL_INVBARL",
            Panel::Slots => "IDCTL_HEALTHBAR",
            Panel::Meters => "IDCTL_METERS",
            Panel::BottomRight => "IDCTL_INVBARR",
            Panel::Map => "IDCTL_MAPWIN",
            Panel::Caption => "IDDLG_CAPTION",
            Panel::Popup => "IDCTL_POPUP",
        }
    }

    /// Whether it is one of the panels that slide off the bottom edge.
    pub fn is_bottom(self) -> bool {
        matches!(self, Panel::BottomLeft | Panel::Slots | Panel::Meters | Panel::BottomRight)
    }
}

/// A group moved together (rule 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Top,
    Bottom,
    MetersAlone,
    Map,
    Caption,
    Popup,
}

impl Group {
    pub fn panels(self) -> &'static [Panel] {
        match self {
            Group::Top => &[Panel::Top],
            Group::Bottom => &[Panel::BottomLeft, Panel::Slots, Panel::BottomRight, Panel::Meters],
            Group::MetersAlone => &[Panel::Meters],
            Group::Map => &[Panel::Map],
            Group::Caption => &[Panel::Caption],
            Group::Popup => &[Panel::Popup],
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Group::Top => "top bar",
            Group::Bottom => "bottom bar",
            Group::MetersAlone => "meters alone",
            Group::Map => "map",
            Group::Caption => "caption",
            Group::Popup => "popup",
        }
    }
}

/// Where one panel is going and how far it is: `away` 0 = fully shown,
/// 1 = fully hidden.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Slide {
    shown: bool,
    away: f32,
}

/// The panels' state.
#[derive(Debug, Clone)]
pub struct Panels {
    slides: [Slide; 8],
    /// Real ms left of the meters' pop-up (rule 25).
    meters_pop: Option<f64>,
    /// Real ms left of the top bar's pop-up for news (rule 13; 4c-3).
    top_pop: Option<f64>,
    hands_off: bool,
    /// The bottom bar was shown when handsOff went on (rule 15).
    shown_before_hands_off: bool,
    /// The bottom bar was shown when the conversation started (rule 16).
    shown_before_talk: bool,
    /// The spinning mask of the status icons (rule 15).
    pub mask_icon: bool,
    /// Real time, ms since the screen opened.
    pub now_ms: f64,
    /// Lines for `gui.log`, taken by the caller.
    pub log: Vec<String>,
    /// Sounds to start (plain, file names), taken by the caller.
    pub sounds: Vec<String>,
}

impl Panels {
    /// The screen opens with `shown` shown and the rest hidden, all in
    /// place (rule 19).
    pub fn new(shown: &[Panel]) -> Self {
        let slides = Panel::ALL.map(|p| {
            let on = shown.contains(&p);
            Slide { shown: on, away: if on { 0.0 } else { 1.0 } }
        });
        Panels {
            slides,
            meters_pop: None,
            top_pop: None,
            hands_off: false,
            shown_before_hands_off: false,
            shown_before_talk: false,
            mask_icon: false,
            now_ms: 0.0,
            log: Vec::new(),
            sounds: Vec::new(),
        }
    }

    fn note(&mut self, line: impl AsRef<str>) {
        self.log.push(format!("t {:9.3} s (real): {}", self.now_ms / 1000.0, line.as_ref()));
    }

    /// Whether `p` is shown or on its way in.
    pub fn shown(&self, p: Panel) -> bool {
        self.slides[p.index()].shown
    }

    /// How far `p` is from its shown place: 0 shown … 1 hidden.
    pub fn away(&self, p: Panel) -> f32 {
        self.slides[p.index()].away
    }

    /// Whether `p` is sliding (neither fully shown nor fully hidden).
    pub fn moving(&self, p: Panel) -> bool {
        let a = self.away(p);
        a > 0.0 && a < 1.0
    }

    /// Whether `p` is at least partly on the screen.
    pub fn visible(&self, p: Panel) -> bool {
        self.away(p) < 1.0
    }

    /// The bottom bar counts as shown when bottom left is (Space, rules
    /// 10, 15).
    pub fn bottom_shown(&self) -> bool {
        self.shown(Panel::BottomLeft)
    }

    /// Shows or hides every panel of `group` (rule 9: one already going
    /// that way is left alone). Returns whether any started to move.
    pub fn set(&mut self, group: Group, on: bool, why: &str) -> bool {
        let mut moved = Vec::new();
        for &p in group.panels() {
            let s = &mut self.slides[p.index()];
            if s.shown != on {
                s.shown = on;
                moved.push(p.name());
                let sound = match (p, on) {
                    (Panel::Top, true) => Some("intrup.aud"),
                    (Panel::Top, false) => Some("intrdown.aud"),
                    (Panel::Map, true) => Some("map.aud"),
                    (Panel::Map, false) => Some("mapbgone.aud"),
                    _ => None,
                };
                if let Some(s) = sound {
                    self.sounds.push(s.to_string());
                }
            }
        }
        if on && group == Group::Bottom && self.meters_pop.take().is_some() {
            self.note("the meters' pop-up timer cleared: the bottom bar is back (rule 25)");
        }
        if on && group == Group::Top {
            self.top_pop = None;
        }
        let verb = if on { "show" } else { "hide" };
        if moved.is_empty() {
            self.note(format!("{} {verb} ({why}): already {}; left alone (rule 9)", group.name(), if on { "shown or sliding in" } else { "hidden or sliding out" }));
            false
        } else {
            self.note(format!("{} {verb} ({why}): {} start to slide {} (0.2 s)", group.name(), moved.join(", "), if on { "in" } else { "out" }));
            true
        }
    }

    /// Rule 10: Space.
    pub fn toggle_bottom(&mut self, why: &str) {
        let on = !self.bottom_shown();
        self.set(Group::Bottom, on, why);
    }

    /// Rule 15: `handsOff` going on or off.
    pub fn hands_off(&mut self, on: bool) {
        if on == self.hands_off {
            return;
        }
        self.hands_off = on;
        if on {
            self.shown_before_hands_off = self.bottom_shown();
            if self.shown_before_hands_off {
                self.set(Group::Bottom, false, "handsOff 1; remembered as shown");
            } else {
                self.note("handsOff 1: the bottom bar is away; it stays away afterwards (rule 15)");
            }
            if self.shown(Panel::Map) {
                self.set(Group::Map, false, "handsOff 1");
            }
            self.mask_icon = true;
            self.note("handsOff 1: the spinning mask shows; the top bar is not touched (rule 15)");
        } else {
            if std::mem::take(&mut self.shown_before_hands_off) {
                self.set(Group::Bottom, true, "handsOff 0; it was shown before");
            } else {
                self.note("handsOff 0: the bottom bar was away before; it stays away (rule 15)");
            }
            self.mask_icon = false;
            self.note("handsOff 0: the spinning mask goes; the map is not brought back (rule 15)");
        }
    }

    /// Rule 16: a conversation starting (`true`) or ending.
    pub fn conversation(&mut self, start: bool) {
        if self.hands_off {
            self.note(format!("conversation {}: during handsOff; panels not touched (rule 16)", if start { "starts" } else { "ends" }));
            return;
        }
        if start {
            self.shown_before_talk = self.bottom_shown();
            if self.shown_before_talk {
                self.set(Group::Bottom, false, "conversation starts; remembered as shown");
            }
        } else if std::mem::take(&mut self.shown_before_talk) && !self.bottom_shown() {
            self.set(Group::Bottom, true, "conversation ends; it was shown before");
        }
    }

    /// Rule 25: a health or experience percent was sent.
    pub fn meters_sent(&mut self, why: &str) {
        if !self.shown(Panel::Meters) {
            self.set(Group::MetersAlone, true, &format!("{why} while hidden: pop-up for 1.5 s (rule 25)"));
            self.meters_pop = Some(POP_MS);
        } else if self.meters_pop.is_some() {
            self.meters_pop = Some(POP_MS);
            self.note(format!("meters: {why} while popped up: 1.5 s again (rule 25, inferred)"));
        }
    }

    /// Rule 13 (4c-3): the top bar comes down for news and goes up 1.5 s
    /// later.
    pub fn pop_top(&mut self, why: &str) {
        self.set(Group::Top, true, why);
        self.top_pop = Some(POP_MS);
    }

    /// Whether the top bar is in its popped time (rule 12).
    pub fn top_popped(&self) -> bool {
        self.top_pop.is_some()
    }

    /// `getGuiStatus` (rule 15): the five groups.
    pub fn status(&self) -> String {
        let on = |p: Panel| if self.shown(p) { 1 } else { 0 };
        format!(
            "top {} bottom {} map {} caption {} popup {} (meters {})",
            on(Panel::Top),
            on(Panel::BottomLeft),
            on(Panel::Map),
            on(Panel::Caption),
            on(Panel::Popup),
            on(Panel::Meters)
        )
    }

    /// Moves time on by `ms` real milliseconds: slides, then the pop-up
    /// timers.
    pub fn update(&mut self, ms: f64) {
        self.now_ms += ms;
        let step = (ms / SLIDE_MS) as f32;
        for p in Panel::ALL {
            let s = &mut self.slides[p.index()];
            let before = s.away;
            s.away = if s.shown { (s.away - step).max(0.0) } else { (s.away + step).min(1.0) };
            if before != s.away && (s.away == 0.0 || s.away == 1.0) {
                let line = format!("{} {} (rule 9)", p.name(), if s.away == 0.0 { "fully shown" } else { "fully hidden" });
                self.note(line);
            }
        }
        if let Some(left) = self.meters_pop.as_mut() {
            *left -= ms;
            if *left <= 0.0 {
                self.meters_pop = None;
                self.set(Group::MetersAlone, false, "pop-up over, 1.5 s (rule 25)");
            }
        }
        if let Some(left) = self.top_pop.as_mut() {
            *left -= ms;
            if *left <= 0.0 {
                self.top_pop = None;
                self.set(Group::Top, false, "pop-up over, 1.5 s (rule 13)");
            }
        }
    }
}

/// Where panel `p` stands on a screen `screen` (pixels) when its shown
/// rectangle is `rect` (left, top, right, bottom) and it is `away` of the
/// way out (rule 9): (left, top).
pub fn place(p: Panel, rect: [f32; 4], screen: [f32; 2], away: f32) -> (f32, f32) {
    let [l, t, r, b] = rect;
    let lerp = |from: f32, to: f32| from + (to - from) * away;
    match p {
        Panel::Top => (l, lerp(t, -(b - t) - screen[1] / 480.0)),
        Panel::Map => (lerp(l, -(r - l) - 50.0 / 640.0 * screen[0]), t),
        p if p.is_bottom() => (l, lerp(t, screen[1])),
        _ => (l, t),
    }
}

/// M4 4c-6 (the user's choice, 2026-10-18): the original's screen is 4:3
/// and its panels are laid out for it, so they stand on a "virtual" 4:3
/// screen, the largest that fits in the window, centred; the world still
/// fills the window. Returns its left, top, width, height in window
/// pixels.
pub fn screen_frame(window: [f32; 2]) -> [f32; 4] {
    let [ww, wh] = window;
    let (w, h) = if ww * 3.0 > wh * 4.0 { (wh * 4.0 / 3.0, wh) } else { (ww, ww * 3.0 / 4.0) };
    [(ww - w) / 2.0, (wh - h) / 2.0, w, h]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_screen_is_4_3_in_the_middle_of_the_window() {
        assert_eq!(screen_frame([1920.0, 1080.0]), [240.0, 0.0, 1440.0, 1080.0]);
        assert_eq!(screen_frame([800.0, 600.0]), [0.0, 0.0, 800.0, 600.0]);
        assert_eq!(screen_frame([800.0, 800.0]), [0.0, 100.0, 800.0, 600.0]);
    }

    fn start() -> Panels {
        Panels::new(&[Panel::BottomLeft, Panel::Slots, Panel::Meters, Panel::BottomRight])
    }

    #[test]
    fn space_slides_the_bottom_bar_away_in_a_fifth_of_a_second_and_back() {
        let mut p = start();
        assert!(p.bottom_shown() && !p.visible(Panel::Top));
        p.toggle_bottom("Space");
        p.update(100.0);
        assert_eq!(p.away(Panel::Slots), 0.5);
        p.update(100.0);
        assert_eq!((p.away(Panel::BottomLeft), p.away(Panel::Meters)), (1.0, 1.0));
        assert!(p.log.iter().any(|l| l.contains("bottom left fully hidden")), "{:?}", p.log);
        p.toggle_bottom("Space");
        p.update(50.0);
        // Hiding again half-way: it goes back from where it is.
        p.toggle_bottom("Space");
        p.update(50.0);
        assert_eq!(p.away(Panel::BottomRight), 1.0);
        assert!(p.sounds.is_empty(), "the bottom panels play nothing");
        // Asked to hide while hiding: left alone.
        assert!(!p.set(Group::Bottom, false, "test"));
    }

    #[test]
    fn the_meters_pop_up_alone_for_one_and_a_half_seconds() {
        let mut p = start();
        p.meters_sent("health 100 %");
        assert!(p.log.is_empty() || !p.log.iter().any(|l| l.contains("pop-up")), "shown: no pop-up");
        p.toggle_bottom("Space");
        p.update(300.0);
        p.meters_sent("health 50 %");
        assert!(p.shown(Panel::Meters) && !p.shown(Panel::Slots));
        p.update(1000.0);
        p.meters_sent("health 40 %");
        p.update(1000.0);
        assert!(p.shown(Panel::Meters), "renewed at 1.0 s: still up at 2.0 s");
        p.update(600.0);
        assert!(!p.shown(Panel::Meters));
        // Brought back by Space: the timer is cleared, the meters stay.
        p.meters_sent("health 30 %");
        p.toggle_bottom("Space");
        p.update(2000.0);
        assert!(p.shown(Panel::Meters) && p.shown(Panel::Slots));
    }

    #[test]
    fn hands_off_remembers_whether_the_bar_was_shown() {
        let mut p = start();
        p.hands_off(true);
        assert!(!p.bottom_shown() && p.mask_icon);
        p.hands_off(false);
        assert!(p.bottom_shown() && !p.mask_icon);
        // Put away first: it stays away.
        p.toggle_bottom("Space");
        p.hands_off(true);
        p.hands_off(false);
        assert!(!p.bottom_shown());
        // The top bar is not touched.
        p.set(Group::Top, true, "test");
        p.hands_off(true);
        assert!(p.shown(Panel::Top));
        assert_eq!(p.sounds, ["intrup.aud"]);
    }

    #[test]
    fn conversations_put_the_bar_away_but_not_during_hands_off() {
        let mut p = start();
        p.conversation(true);
        assert!(!p.bottom_shown());
        p.conversation(false);
        assert!(p.bottom_shown());
        p.hands_off(true);
        p.conversation(true);
        p.hands_off(false);
        assert!(p.bottom_shown());
        assert!(p.status().starts_with("top 0 bottom 1 map 0"));
    }

    #[test]
    fn hidden_places() {
        let screen = [800.0, 600.0];
        // Top bar 800 x 80: hidden 1.25 above its own height.
        assert_eq!(place(Panel::Top, [0.0, 0.0, 800.0, 80.0], screen, 1.0), (0.0, -81.25));
        // Slots at 520: half-way to the bottom edge.
        assert_eq!(place(Panel::Slots, [240.0, 520.0, 560.0, 582.5], screen, 0.5), (240.0, 560.0));
        // Map 400 wide: its right edge ends at -62.5.
        assert_eq!(place(Panel::Map, [0.0, 80.0, 400.0, 520.0], screen, 1.0), (-462.5, 80.0));
    }
}
