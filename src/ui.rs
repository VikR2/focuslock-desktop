//! The FocusLock window: search and a session bar on top, one list of apps below.
//! Every app has the same Off / Warn / Lock control.

use focuslock::engine::blocker::{self, Event, Plan};
use focuslock::engine::catalog::{self, AppEntry, IconSource};
use focuslock::engine::session::{format_clock, now_secs, Phase, Session};
use focuslock::engine::store::{self, Mode, Rule, State};
use focuslock::engine::{exe_key, search, win};
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{h_flex, v_flex, Disableable as _, Icon, IconName, Sizable as _, Theme, ThemeMode};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Duration;

actions!(focuslock, [FocusSearch, ClearSearch]);

// Shape rule: controls 6, segments inside controls 4 (6 - 2 padding), containers 10.
const R_CTRL: f32 = 6.;
const R_SEG: f32 = 4.;
const R_BOX: f32 = 10.;
const PRESETS: [u64; 4] = [25, 45, 60, 90];
const NOTICE_SECS: u64 = 8;

#[derive(Clone, Copy)]
struct Pal {
    bg: Hsla,
    panel: Hsla,
    raised: Hsla,
    border: Hsla,
    text: Hsla,
    muted: Hsla,
    accent: Hsla,
    accent_soft: Hsla,
    on_accent: Hsla,
    warn: Hsla,
    warn_soft: Hsla,
    hover: Hsla,
}

/// FocusLock's tokens: graphite surfaces, one blue accent, Segoe UI.
fn pal(dark: bool) -> Pal {
    if dark {
        Pal {
            bg: rgb(0x16181c).into(),
            panel: rgb(0x1c1f24).into(),
            raised: rgb(0x23272d).into(),
            border: rgb(0x2e333a).into(),
            text: rgb(0xe6e9ee).into(),
            muted: rgb(0x9aa3ae).into(),
            accent: rgb(0x3b8cff).into(),
            accent_soft: rgba(0x3b8cff26).into(),
            on_accent: rgb(0xffffff).into(),
            warn: rgb(0xe0a84a).into(),
            warn_soft: rgba(0xe0a84a1f).into(),
            hover: rgba(0xffffff0d).into(),
        }
    } else {
        Pal {
            bg: rgb(0xf6f6f4).into(),
            panel: rgb(0xffffff).into(),
            raised: rgb(0xeeefef).into(),
            border: rgb(0xdedfe1).into(),
            text: rgb(0x1c2128).into(),
            muted: rgb(0x5b6470).into(),
            accent: rgb(0x0a6ce0).into(),
            accent_soft: rgba(0x0a6ce01f).into(),
            on_accent: rgb(0xffffff).into(),
            warn: rgb(0x9a6206).into(),
            warn_soft: rgba(0xc98a1a24).into(),
            hover: rgba(0x0000000a).into(),
        }
    }
}

fn apply_theme(dark: bool, window: Option<&mut Window>, cx: &mut App) {
    Theme::change(if dark { ThemeMode::Dark } else { ThemeMode::Light }, window, cx);
    let p = pal(dark);
    Theme::update(cx, |t| {
        t.primary = p.accent;
        t.primary_hover = p.accent.opacity(0.9);
        t.primary_active = p.accent.opacity(0.8);
        t.primary_foreground = p.on_accent;
        t.button_primary = p.accent;
        t.button_primary_hover = p.accent.opacity(0.9);
        t.button_primary_active = p.accent.opacity(0.8);
        t.button_primary_foreground = p.on_accent;
        t.ring = p.accent;
        t.background = p.panel;
        t.foreground = p.text;
        t.border = p.border;
        t.input = p.border;
        t.muted_foreground = p.muted;
        t.font_family = "Segoe UI".into();
        t.radius = px(R_CTRL);
    });
}

fn is_dark(window: &Window) -> bool {
    matches!(window.appearance(), WindowAppearance::Dark | WindowAppearance::VibrantDark)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scan {
    Loading,
    Ready,
    Failed,
}

pub struct FocusLock {
    state: State,
    state_path: PathBuf,
    save_error: Option<String>,
    plan: Arc<Mutex<Plan>>,
    events: Receiver<Event>,

    apps: Vec<AppEntry>,
    scan: Scan,
    icons: HashMap<String, Arc<RenderImage>>,

    query: Entity<InputState>,
    query_text: String,
    /// The app being unlocked mid-session, and when the wait started.
    unlock: Option<(String, u64)>,
    notice: Option<(String, u64)>,
    dark: bool,
    focus: FocusHandle,

    _subs: Vec<Subscription>,
    _tick: Task<()>,
}

impl FocusLock {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state_path = store::state_path();
        let state = store::load(&state_path);
        let (plan, events) = blocker::spawn();

        let query = cx.new(|cx| InputState::new(window, cx).placeholder("Search apps"));
        let input_sub = cx.subscribe_in(&query, window, |this, state, event, _, cx| {
            if matches!(event, InputEvent::Change) {
                this.query_text = state.read(cx).value().to_string();
                cx.notify();
            }
        });
        let theme_sub = cx.observe_window_appearance(window, |this, window, cx| {
            this.dark = is_dark(window);
            apply_theme(this.dark, Some(window), cx);
            cx.notify();
        });

        // Closing the window mid-session would stop blocking, so it minimizes instead.
        let weak = cx.entity().downgrade();
        window.on_window_should_close(cx, move |window, cx| {
            let in_session = weak.read_with(cx, |this, _| this.state.session.is_some()).unwrap_or(false);
            if in_session {
                window.minimize_window();
            }
            !in_session
        });

        let tick = cx.spawn_in(window, async move |this, cx| {
            let mut ticks: u32 = 0;
            loop {
                cx.background_executor().timer(Duration::from_millis(500)).await;
                if this.update_in(cx, |this, window, cx| this.tick(window, cx)).is_err() {
                    break;
                }
                ticks += 1;
                // Every 3 s, refresh which apps are open so "Running" stays true.
                if ticks.is_multiple_of(6) {
                    let running = cx.background_spawn(async { win::apps::running_keys() }).await;
                    let updated = this.update(cx, |this, cx| {
                        for a in &mut this.apps {
                            a.running = running.contains(&a.key);
                        }
                        cx.notify();
                    });
                    if updated.is_err() {
                        break;
                    }
                }
            }
        });

        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        let dark = is_dark(window);
        apply_theme(dark, Some(window), cx);

        let mut this = Self {
            state,
            state_path,
            save_error: None,
            plan,
            events,
            apps: Vec::new(),
            scan: Scan::Loading,
            icons: HashMap::new(),
            query,
            query_text: String::new(),
            unlock: None,
            notice: None,
            dark,
            focus,
            _subs: vec![input_sub, theme_sub],
            _tick: tick,
        };
        // A session saved before a restart resumes blocking straight away.
        this.finish_if_over();
        this.push_plan();
        this.scan_apps(cx);
        this
    }

    // ---------- state ----------

    fn enforcing(&self) -> bool {
        self.state.session.as_ref().is_some_and(|s| s.is_enforcing(now_secs()))
    }

    fn push_plan(&self) {
        if let Ok(mut plan) = self.plan.lock() {
            plan.rules = self.state.rules.clone();
            plan.session = self.state.session.clone();
        }
    }

    /// Save and hand the new rules to the blocker. Call after every change.
    fn commit(&mut self) {
        self.push_plan();
        self.save_error = store::save(&self.state_path, &self.state)
            .err()
            .map(|e| format!("Couldn't save your settings: {e}"));
    }

    fn finish_if_over(&mut self) -> bool {
        let over = self.state.session.as_ref().is_some_and(|s| s.phase(now_secs()) == Phase::Finished);
        if over {
            self.end_session();
        }
        over
    }

    fn end_session(&mut self) {
        self.state.session = None;
        self.unlock = None;
        for r in &mut self.state.rules {
            r.allowed_until = None;
        }
        self.commit();
    }

    fn tick(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let now = now_secs();
        let mut texts: Vec<String> = Vec::new();
        let mut warned = false;
        while let Ok(event) = self.events.try_recv() {
            let text = match event {
                Event::Closed { name } => format!("Closed {name}"),
                Event::CouldNotClose { name } => format!("Windows didn't let FocusLock close {name}"),
                Event::Warned { name } => {
                    warned = true;
                    format!("{name} is open. You set it to warn during focus")
                }
            };
            // A browser closes as many processes; name it once.
            if !texts.contains(&text) {
                texts.push(text);
            }
        }
        if warned {
            window.activate_window();
        }
        // Several apps can trip in the same tick; show them all.
        if !texts.is_empty() {
            self.notice = Some((texts.join(". "), now));
        }
        if self.notice.as_ref().is_some_and(|(_, at)| now >= at + NOTICE_SECS) {
            self.notice = None;
        }
        if self.finish_if_over() {
            self.notice = Some(("Session complete".into(), now));
        }
        // Expired temporary unlocks need no cleanup: the blocker checks the time.
        cx.notify();
    }

    fn scan_apps(&mut self, cx: &mut Context<Self>) {
        self.scan = Scan::Loading;
        cx.spawn(async move |this, cx| {
            let apps = cx
                .background_spawn(async {
                    win::init_com_for_thread();
                    catalog::merge(win::apps::scan())
                })
                .await;
            let order = match this.update(cx, |this, cx| {
                let own = std::env::current_exe().ok().map(|e| exe_key(&e.to_string_lossy()));
                this.apps = apps.into_iter().filter(|a| Some(&a.key) != own.as_ref()).collect();
                this.scan = if this.apps.is_empty() { Scan::Failed } else { Scan::Ready };
                cx.notify();
                this.icon_order()
            }) {
                Ok(order) => order,
                Err(_) => return,
            };
            for chunk in order.chunks(12) {
                let chunk = chunk.to_vec();
                let loaded = cx
                    .background_spawn(async move {
                        win::init_com_for_thread();
                        chunk
                            .into_iter()
                            .filter_map(|(key, src)| win::icons::icon(&src, 48).map(|b| (key, b)))
                            .collect::<Vec<_>>()
                    })
                    .await;
                let applied = this.update(cx, |this, cx| {
                    for (key, bmp) in loaded {
                        if let Some(buf) = image::RgbaImage::from_raw(bmp.width, bmp.height, bmp.bgra) {
                            this.icons.insert(key, Arc::new(RenderImage::new(vec![image::Frame::new(buf)])));
                        }
                    }
                    cx.notify();
                });
                if applied.is_err() {
                    return;
                }
            }
        })
        .detach();
    }

    /// Blocked apps first, then running ones, then the rest.
    fn icon_order(&self) -> Vec<(String, IconSource)> {
        let blocked: HashSet<&str> = self.state.rules.iter().map(|r| r.key.as_str()).collect();
        let rank = |a: &AppEntry| if blocked.contains(a.key.as_str()) { 0 } else if a.running { 1 } else { 2 };
        let mut apps: Vec<&AppEntry> = self.apps.iter().collect();
        apps.sort_by_key(|a| rank(a));
        apps.into_iter().map(|a| (a.key.clone(), a.icon.clone())).collect()
    }

    // ---------- actions ----------

    fn start(&mut self) {
        let minutes = self.state.settings.session_minutes;
        self.state.session = Some(Session::start(now_secs(), minutes * 60, ""));
        self.commit();
    }

    fn pause_or_resume(&mut self) {
        let now = now_secs();
        if let Some(s) = &mut self.state.session {
            match s.phase(now) {
                Phase::Paused => s.resume(now),
                Phase::Running => s.pause(now),
                Phase::Finished => {}
            }
        }
        self.unlock = None;
        self.commit();
    }

    fn set_preset(&mut self, minutes: u64) {
        if self.state.session.is_none() {
            self.state.settings.session_minutes = minutes;
            self.commit();
        }
    }

    /// Tightening applies at once. Loosening a locked app mid-session goes
    /// through the unlock wait instead.
    fn set_mode(&mut self, key: &str, name: &str, mode: Option<Mode>) {
        let index = self.state.rules.iter().position(|r| r.key == key);
        // During a session (running or paused) a locked app only ever gets the
        // timed unlock, so Lock is back after it expires and after a resume.
        if let Some(i) = index.filter(|&i| self.state.rules[i].mode == Mode::Lock) {
            if self.state.session.is_some() && mode != Some(Mode::Lock) {
                if !self.state.rules[i].is_allowed(now_secs()) {
                    self.unlock = Some((key.to_string(), now_secs()));
                }
                return;
            }
        }
        match (mode, index) {
            (Some(m), Some(i)) => self.state.rules[i].mode = m,
            (Some(m), None) => self.state.rules.push(Rule { key: key.into(), name: name.into(), mode: m, allowed_until: None }),
            (None, Some(i)) => {
                self.state.rules.remove(i);
            }
            (None, None) => {}
        }
        self.commit();
    }

    fn unlock_now(&mut self, key: &str) {
        let until = now_secs() + self.state.settings.unlock_minutes * 60;
        if let Some(r) = self.state.rules.iter_mut().find(|r| r.key == key) {
            r.allowed_until = Some(until);
        }
        self.unlock = None;
        self.commit();
    }

    fn focus_search(&mut self, _: &FocusSearch, window: &mut Window, cx: &mut Context<Self>) {
        self.query.update(cx, |s, cx| s.focus(window, cx));
    }

    fn clear_search(&mut self, _: &ClearSearch, window: &mut Window, cx: &mut Context<Self>) {
        self.query.update(cx, |s, cx| s.set_value("", window, cx));
        self.query_text.clear();
        cx.notify();
    }

    // ---------- pieces ----------

    fn app(&self, key: &str) -> Option<&AppEntry> {
        self.apps.iter().find(|a| a.key == key)
    }

    fn btn(&self, id: impl Into<SharedString>, label: impl Into<SharedString>, primary: bool) -> Button {
        let b = Button::new(ElementId::Name(id.into())).label(label.into());
        if primary {
            b.primary()
        } else {
            b.outline()
        }
    }

    fn icon(&self, key: &str, name: &str, size: f32, p: Pal) -> AnyElement {
        match self.icons.get(key) {
            Some(image) => img(image.clone()).size(px(size)).flex_none().into_any_element(),
            None => div()
                .size(px(size))
                .flex_none()
                .rounded(px(R_CTRL))
                .bg(p.raised)
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(size * 0.45))
                .text_color(p.muted)
                .child(name.chars().next().unwrap_or('?').to_uppercase().to_string())
                .into_any_element(),
        }
    }

    fn mode_control(&self, key: &str, name: &str, current: Option<Mode>, p: Pal, cx: &mut Context<Self>) -> Div {
        let mut row = h_flex().flex_none().h(px(28.)).p(px(2.)).gap(px(2.)).rounded(px(R_CTRL)).bg(p.raised);
        for (mode, label) in [(None, "Off"), (Some(Mode::Warn), "Warn"), (Some(Mode::Lock), "Lock")] {
            let selected = mode == current;
            let (k, n) = (key.to_string(), name.to_string());
            row = row.child(
                div()
                    .id(SharedString::from(format!("mode-{key}-{label}")))
                    .h_full()
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .rounded(px(R_SEG))
                    .text_size(px(12.))
                    .font_weight(FontWeight::MEDIUM)
                    .cursor_pointer()
                    .when(selected, |d| {
                        d.bg(p.panel).shadow_sm().text_color(match mode {
                            Some(Mode::Lock) => p.accent,
                            Some(Mode::Warn) => p.warn,
                            None => p.text,
                        })
                    })
                    .when(!selected, |d| d.text_color(p.muted).hover(|s| s.text_color(p.text)))
                    .child(label)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.set_mode(&k, &n, mode);
                        cx.notify();
                    })),
            );
        }
        row
    }

    fn unlock_strip(&self, key: &str, name: &str, p: Pal, cx: &mut Context<Self>) -> Div {
        let started = self.unlock.as_ref().map(|u| u.1).unwrap_or(0);
        let left = self.state.settings.unlock_wait_secs.saturating_sub(now_secs().saturating_sub(started));
        let minutes = self.state.settings.unlock_minutes;
        let k = key.to_string();
        h_flex()
            .gap_3()
            .p_3()
            .rounded(px(R_BOX))
            .bg(p.warn_soft)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_size(px(13.))
                    .text_color(p.text)
                    .child(format!("{name} stays locked until this session ends.")),
            )
            .child(self.btn(format!("keep-{key}"), "Keep locked", false).on_click(cx.listener(|this, _, _, cx| {
                this.unlock = None;
                cx.notify();
            })))
            .child(if left > 0 {
                self.btn(format!("unlock-{key}"), format!("Unlock in {left}s"), false).disabled(true)
            } else {
                self.btn(format!("unlock-{key}"), format!("Unlock for {minutes} min"), true).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.unlock_now(&k);
                        cx.notify();
                    },
                ))
            })
    }

    fn row(&self, key: &str, name: &str, subtitle: Option<String>, p: Pal, cx: &mut Context<Self>) -> AnyElement {
        let rule = self.state.rules.iter().find(|r| r.key == key);
        let current = rule.map(|r| r.mode);
        let now = now_secs();
        let allowed = rule.is_some_and(|r| r.is_allowed(now));
        let unlocking = self.unlock.as_ref().is_some_and(|u| u.0 == key);
        let subtitle = if allowed {
            Some(format!("Unlocked until {}", local_hm(rule.and_then(|r| r.allowed_until).unwrap_or(now))))
        } else {
            subtitle
        };
        let can_unlock = self.enforcing() && current == Some(Mode::Lock) && !allowed && !unlocking;
        let k = key.to_string();
        v_flex()
            .child(
                h_flex()
                    .id(SharedString::from(format!("row-{key}")))
                    .gap_3()
                    .h(px(48.))
                    .px_2()
                    .rounded(px(R_CTRL))
                    .hover(|s| s.bg(p.hover))
                    .child(self.icon(key, name, 26., p))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(div().text_size(px(14.)).text_color(p.text).truncate().child(name.to_string()))
                            .when_some(subtitle, |d, s| d.child(div().text_size(px(12.)).text_color(p.muted).truncate().child(s))),
                    )
                    .when(can_unlock, |d| {
                        d.child(
                            div()
                                .id(SharedString::from(format!("ask-unlock-{key}")))
                                .text_size(px(12.))
                                .text_color(p.muted)
                                .cursor_pointer()
                                .hover(|s| s.text_color(p.text))
                                .child("Unlock")
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.unlock = Some((k.clone(), now_secs()));
                                    cx.notify();
                                })),
                        )
                    })
                    .child(self.mode_control(key, name, current, p, cx)),
            )
            .when(unlocking, |d| d.child(div().px_2().pb_2().child(self.unlock_strip(key, name, p, cx))))
            .into_any_element()
    }

    fn section(&self, title: &str, p: Pal) -> Div {
        div().px_2().pt_4().pb_1().text_size(px(12.)).font_weight(FontWeight::SEMIBOLD).text_color(p.muted).child(title.to_string())
    }

    fn skeleton(&self, p: Pal) -> AnyElement {
        let mut col = v_flex().gap_1();
        for i in 0..6 {
            col = col.child(
                h_flex()
                    .gap_3()
                    .h(px(48.))
                    .px_2()
                    .child(div().size(px(26.)).rounded(px(R_CTRL)).bg(p.raised))
                    .child(div().h(px(10.)).w(px(120. + (i % 3) as f32 * 60.)).rounded(px(R_SEG)).bg(p.raised)),
            );
        }
        col.into_any_element()
    }

    fn message_row(&self, text: String, action: Option<Button>, p: Pal) -> AnyElement {
        h_flex()
            .gap_3()
            .px_2()
            .py_4()
            .child(div().flex_1().text_size(px(13.)).text_color(p.text).child(text))
            .when_some(action, |d, b| d.child(b))
            .into_any_element()
    }

    fn list(&self, p: Pal, cx: &mut Context<Self>) -> Div {
        let mut body = v_flex().px_4().pb_4();
        match self.scan {
            Scan::Loading if self.apps.is_empty() => return body.pt_3().child(self.skeleton(p)),
            Scan::Failed => {
                let retry = self.btn("retry-scan", "Retry", false).on_click(cx.listener(|this, _, _, cx| {
                    this.scan_apps(cx);
                    cx.notify();
                }));
                return body.child(self.message_row("Couldn't read the app list from Windows.".into(), Some(retry), p));
            }
            _ => {}
        }

        let query = self.query_text.trim();
        if !query.is_empty() {
            let hits = search::filter(&self.apps, query);
            if hits.is_empty() {
                return body.child(self.message_row(format!("No app matches \"{query}\""), None, p));
            }
            body = body.pt_2();
            for i in hits.into_iter().take(200) {
                let a = &self.apps[i];
                let sub = Some(if a.running { "Running".to_string() } else { exe_label(a) });
                body = body.child(self.row(&a.key, &a.name, sub, p, cx));
            }
            return body;
        }

        let blocked: HashSet<&str> = self.state.rules.iter().map(|r| r.key.as_str()).collect();
        if !self.state.rules.is_empty() {
            body = body.child(self.section("Blocked", p));
            for r in self.state.rules.clone() {
                let app = self.app(&r.key);
                let name = app.map(|a| a.name.clone()).unwrap_or(r.name.clone());
                let sub = app.filter(|a| a.running).map(|_| "Running".to_string());
                body = body.child(self.row(&r.key, &name, sub, p, cx));
            }
        }
        let running: Vec<AppEntry> = self.apps.iter().filter(|a| a.running && !blocked.contains(a.key.as_str())).cloned().collect();
        if !running.is_empty() {
            body = body.child(self.section("Running now", p));
            for a in &running {
                body = body.child(self.row(&a.key, &a.name, None, p, cx));
            }
        }
        body = body.child(self.section("All apps", p));
        let rest: Vec<AppEntry> = self.apps.iter().filter(|a| !a.running && !blocked.contains(a.key.as_str())).cloned().collect();
        for a in &rest {
            body = body.child(self.row(&a.key, &a.name, None, p, cx));
        }
        body
    }

    fn session_bar(&self, p: Pal, cx: &mut Context<Self>) -> Div {
        let now = now_secs();
        let phase = self.state.session.as_ref().map(|s| s.phase(now));
        let (clock, status) = match (&self.state.session, phase) {
            (Some(s), Some(Phase::Running)) => (format_clock(s.left(now)), format!("Blocking until {}", local_hm(s.ends_at))),
            (Some(s), Some(Phase::Paused)) => (format_clock(s.left(now)), "Paused. Apps are not blocked".into()),
            _ if self.state.rules.is_empty() => (format_clock(self.state.settings.session_minutes * 60), "Pick apps to block below".into()),
            _ => (format_clock(self.state.settings.session_minutes * 60), "Ready".into()),
        };
        let clock_block = v_flex()
            .items_end()
            .child(div().text_size(px(28.)).text_color(p.text).child(clock))
            .child(div().text_size(px(12.)).text_color(p.muted).child(status));

        let controls = match phase {
            None | Some(Phase::Finished) => {
                let mut presets = h_flex().gap_1();
                for m in PRESETS {
                    let selected = self.state.settings.session_minutes == m;
                    presets = presets.child(
                        div()
                            .id(SharedString::from(format!("preset-{m}")))
                            .h(px(26.))
                            .px(px(10.))
                            .flex()
                            .items_center()
                            .rounded(px(R_CTRL))
                            .text_size(px(12.))
                            .font_weight(FontWeight::MEDIUM)
                            .cursor_pointer()
                            .when(selected, |d| d.bg(p.accent_soft).text_color(p.accent))
                            .when(!selected, |d| d.text_color(p.muted).hover(|s| s.bg(p.hover)))
                            .child(format!("{m} min"))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.set_preset(m);
                                cx.notify();
                            })),
                    );
                }
                let start = self
                    .btn("start", "Start focus", true)
                    .disabled(self.state.rules.is_empty())
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.start();
                        cx.notify();
                    }));
                v_flex().gap_1().items_end().child(presets).child(start)
            }
            Some(ph) => v_flex().items_end().child(
                h_flex()
                    .gap_2()
                    .child(self.btn("pause", if ph == Phase::Paused { "Resume" } else { "Pause" }, ph == Phase::Paused).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.pause_or_resume();
                            cx.notify();
                        }),
                    ))
                    .child(self.btn("end", "End", false).on_click(cx.listener(|this, _, _, cx| {
                        this.end_session();
                        cx.notify();
                    }))),
            ),
        };

        h_flex()
            .gap_4()
            .px_5()
            .h(px(80.))
            .flex_none()
            .bg(p.panel)
            .child(div().flex_1().child(Input::new(&self.query).prefix(Icon::new(IconName::Search).small()).cleanable(true)))
            .child(clock_block)
            .child(controls)
    }

    fn progress_line(&self, p: Pal) -> Div {
        let frac = self.state.session.as_ref().map(|s| s.progress(now_secs())).unwrap_or(0.0);
        div()
            .h(px(2.))
            .w_full()
            .flex_none()
            .bg(p.border)
            .when(self.state.session.is_some(), |d| d.child(div().h_full().bg(p.accent).w(relative(frac))))
    }

    fn notice_bar(&self, p: Pal, cx: &mut Context<Self>) -> Option<Div> {
        let text = self.save_error.clone().or_else(|| self.notice.as_ref().map(|n| n.0.clone()))?;
        Some(
            h_flex()
                .absolute()
                .bottom_4()
                .left_4()
                .right_4()
                .gap_3()
                .px_3()
                .h(px(40.))
                .rounded(px(R_BOX))
                .border_1()
                .border_color(p.border)
                .bg(p.raised)
                .shadow_md()
                .child(div().flex_1().text_size(px(13.)).text_color(p.text).child(text))
                .child(
                    div()
                        .id("dismiss-notice")
                        .text_color(p.muted)
                        .cursor_pointer()
                        .hover(|s| s.text_color(p.text))
                        .child(Icon::new(IconName::Close).small())
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.notice = None;
                            this.save_error = None;
                            cx.notify();
                        })),
                ),
        )
    }
}

impl Render for FocusLock {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = pal(self.dark);
        let notice = self.notice_bar(p, cx);
        v_flex()
            .key_context("FocusLock")
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::focus_search))
            .on_action(cx.listener(Self::clear_search))
            .size_full()
            .font_family("Segoe UI")
            .bg(p.bg)
            .text_color(p.text)
            .child(self.session_bar(p, cx))
            .child(self.progress_line(p))
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .child(div().id("app-list").size_full().overflow_y_scroll().child(self.list(p, cx).pb(px(64.))))
                    .when_some(notice, |d, n| d.child(n)),
            )
    }
}

fn exe_label(a: &AppEntry) -> String {
    if a.key.starts_with("pkg:") {
        "Microsoft Store app".into()
    } else {
        a.key.clone()
    }
}

/// Local wall-clock time, e.g. `14:25`.
fn local_hm(unix: u64) -> String {
    use windows::Win32::System::Time::{GetTimeZoneInformation, TIME_ZONE_INFORMATION};
    let mut tz = TIME_ZONE_INFORMATION::default();
    let id = unsafe { GetTimeZoneInformation(&mut tz) };
    let bias = tz.Bias + if id == 2 { tz.DaylightBias } else { tz.StandardBias };
    let t = (unix as i64 - bias as i64 * 60).rem_euclid(86_400);
    format!("{:02}:{:02}", t / 3600, (t % 3600) / 60)
}

pub fn run() {
    gpui_kit::application().with_assets(gpui_kit::assets::Assets).run(|cx| {
        gpui_kit::init(cx);
        cx.bind_keys([
            KeyBinding::new("ctrl-f", FocusSearch, Some("FocusLock")),
            KeyBinding::new("escape", ClearSearch, Some("FocusLock")),
        ]);
        let bounds = Bounds::centered(None, size(px(980.), px(700.)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_min_size: Some(size(px(720.), px(480.))),
            titlebar: Some(TitlebarOptions { title: Some("FocusLock".into()), ..Default::default() }),
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, |window, cx| cx.new(|cx| FocusLock::new(window, cx))).expect("open the FocusLock window");
        cx.activate(true);
    });
}
