mod activity;

use activity::{ActivityTracker, WorkSummary};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub work_minutes: u32,
    pub rest_minutes: u32,
    pub voice: bool,
    pub beat: bool,
    pub strict: bool,
    #[serde(default = "default_true")]
    pub summary: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            work_minutes: 45,
            rest_minutes: 10,
            voice: true,
            beat: true,
            strict: false,
            summary: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Idle,
    Work,
    Rest,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerSnapshot {
    pub phase: Phase,
    pub remaining_secs: u64,
    pub total_secs: u64,
    pub paused: bool,
    pub settings: Settings,
    pub last_summary: Option<WorkSummary>,
    pub rest_summary: Option<WorkSummary>,
}

struct Inner {
    settings: Settings,
    phase: Phase,
    remaining_secs: u64,
    total_secs: u64,
    paused: bool,
    last_summary: Option<WorkSummary>,
    summary_ready: bool,
}

struct AppState {
    inner: Mutex<Inner>,
    epoch: AtomicU64,
    activity: ActivityTracker,
}

fn clamp_settings(mut s: Settings) -> Settings {
    s.work_minutes = s.work_minutes.clamp(1, 180);
    s.rest_minutes = s.rest_minutes.clamp(1, 60);
    s
}

fn settings_path(app: &AppHandle) -> PathBuf {
    let dir = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    let _ = std::fs::create_dir_all(&dir);
    dir.join("settings.json")
}

fn load_settings(app: &AppHandle) -> Settings {
    std::fs::read_to_string(settings_path(app))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .map(clamp_settings)
        .unwrap_or_default()
}

fn save_settings_file(app: &AppHandle, settings: &Settings) {
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(settings_path(app), json);
    }
}

fn snapshot_of(inner: &Inner) -> TimerSnapshot {
    TimerSnapshot {
        phase: inner.phase,
        remaining_secs: inner.remaining_secs,
        total_secs: inner.total_secs.max(1),
        paused: inner.paused,
        settings: inner.settings.clone(),
        last_summary: inner.last_summary.clone(),
        rest_summary: if inner.phase == Phase::Rest && inner.summary_ready {
            inner.last_summary.clone()
        } else {
            None
        },
    }
}

fn capture_work_summary(app: &AppHandle, inner: &mut Inner) {
    if inner.settings.summary {
        inner.last_summary = app.state::<AppState>().activity.finish();
        inner.summary_ready = inner.last_summary.is_some();
    } else {
        app.state::<AppState>().activity.reset();
        inner.summary_ready = false;
    }
}

fn begin_fresh_work(app: &AppHandle, inner: &mut Inner) {
    app.state::<AppState>().activity.reset();
    begin_work_locked(inner);
}

fn emit_tick(app: &AppHandle, snap: &TimerSnapshot) {
    let _ = app.emit("timer-tick", snap);
}

fn show_overlay(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("overlay") {
        let _ = w.set_always_on_top(true);
        let _ = w.set_fullscreen(true);
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn hide_overlay(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("overlay") {
        let _ = w.hide();
        let _ = w.set_fullscreen(false);
        let _ = w.set_always_on_top(false);
    }
}

fn begin_work_locked(inner: &mut Inner) {
    inner.phase = Phase::Work;
    inner.paused = false;
    inner.total_secs = u64::from(inner.settings.work_minutes) * 60;
    inner.remaining_secs = inner.total_secs;
}

fn begin_rest_locked(inner: &mut Inner, secs: u64) {
    inner.phase = Phase::Rest;
    inner.paused = false;
    inner.total_secs = secs.max(1);
    inner.remaining_secs = inner.total_secs;
}

fn start_ticker(app: &AppHandle) {
    let state = app.state::<AppState>();
    let id = state.epoch.fetch_add(1, Ordering::SeqCst) + 1;
    let app = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));
        let st = app.state::<AppState>();
        if st.epoch.load(Ordering::SeqCst) != id {
            break;
        }
        step(&app);
    });
}

fn stop_ticker(app: &AppHandle) {
    app.state::<AppState>()
        .epoch
        .fetch_add(1, Ordering::SeqCst);
}

fn step(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut events: Vec<(&'static str, TimerSnapshot)> = Vec::new();
    let mut show_rest = false;
    let mut hide_rest = false;
    let mut sample_work = false;

    {
        let mut inner = state.inner.lock().unwrap();
        if inner.paused || inner.phase == Phase::Idle {
            emit_tick(app, &snapshot_of(&inner));
            return;
        }
        if inner.remaining_secs > 0 {
            inner.remaining_secs -= 1;
        }
        if inner.remaining_secs == 0 {
            match inner.phase {
                Phase::Work => {
                    capture_work_summary(app, &mut inner);
                    let secs = u64::from(inner.settings.rest_minutes) * 60;
                    begin_rest_locked(&mut inner, secs);
                    show_rest = true;
                    let snap = snapshot_of(&inner);
                    events.push(("rest-begin", snap));
                }
                Phase::Rest => {
                    begin_fresh_work(app, &mut inner);
                    hide_rest = true;
                    let snap = snapshot_of(&inner);
                    events.push(("rest-end", snap.clone()));
                    events.push(("work-begin", snap));
                }
                Phase::Idle => {}
            }
        } else if inner.phase == Phase::Work {
            sample_work = true;
        }
        emit_tick(app, &snapshot_of(&inner));
    }

    if sample_work {
        state.activity.sample();
    }
    if show_rest {
        show_overlay(app);
    }
    if hide_rest {
        hide_overlay(app);
    }
    for (name, snap) in events {
        let _ = app.emit(name, snap);
    }
}

#[tauri::command]
fn get_state(state: State<AppState>) -> TimerSnapshot {
    snapshot_of(&state.inner.lock().unwrap())
}

#[tauri::command]
fn save_settings(app: AppHandle, state: State<AppState>, settings: Settings) -> TimerSnapshot {
    let settings = clamp_settings(settings);
    save_settings_file(&app, &settings);
    let mut inner = state.inner.lock().unwrap();
    inner.settings = settings;
    if inner.phase == Phase::Idle {
        inner.total_secs = u64::from(inner.settings.work_minutes) * 60;
        inner.remaining_secs = inner.total_secs;
    }
    let snap = snapshot_of(&inner);
    emit_tick(&app, &snap);
    snap
}

#[tauri::command]
fn start(app: AppHandle, state: State<AppState>) -> TimerSnapshot {
    let snap = {
        let mut inner = state.inner.lock().unwrap();
        match inner.phase {
            Phase::Idle => begin_fresh_work(&app, &mut inner),
            Phase::Work | Phase::Rest => inner.paused = false,
        }
        snapshot_of(&inner)
    };
    start_ticker(&app);
    if snap.phase == Phase::Work {
        let _ = app.emit("work-begin", &snap);
    }
    emit_tick(&app, &snap);
    snap
}

#[tauri::command]
fn pause(app: AppHandle, state: State<AppState>) -> TimerSnapshot {
    let snap = {
        let mut inner = state.inner.lock().unwrap();
        if inner.phase != Phase::Idle {
            inner.paused = true;
        }
        snapshot_of(&inner)
    };
    emit_tick(&app, &snap);
    snap
}

#[tauri::command]
fn reset(app: AppHandle, state: State<AppState>) -> TimerSnapshot {
    stop_ticker(&app);
    hide_overlay(&app);
    let snap = {
        let mut inner = state.inner.lock().unwrap();
        if inner.phase == Phase::Work {
            capture_work_summary(&app, &mut inner);
        } else {
            state.activity.reset();
        }
        inner.phase = Phase::Idle;
        inner.paused = false;
        inner.total_secs = u64::from(inner.settings.work_minutes) * 60;
        inner.remaining_secs = inner.total_secs;
        snapshot_of(&inner)
    };
    let _ = app.emit("rest-end", &snap);
    emit_tick(&app, &snap);
    snap
}

#[tauri::command]
fn rest_now(app: AppHandle, state: State<AppState>, secs: Option<u64>) -> TimerSnapshot {
    let snap = {
        let mut inner = state.inner.lock().unwrap();
        let secs = secs.unwrap_or_else(|| u64::from(inner.settings.rest_minutes) * 60);
        if inner.phase == Phase::Work {
            capture_work_summary(&app, &mut inner);
        } else {
            state.activity.reset();
            inner.summary_ready = false;
        }
        begin_rest_locked(&mut inner, secs);
        snapshot_of(&inner)
    };
    start_ticker(&app);
    show_overlay(&app);
    let _ = app.emit("rest-begin", &snap);
    emit_tick(&app, &snap);
    snap
}

#[tauri::command]
fn skip_rest(app: AppHandle, state: State<AppState>) -> TimerSnapshot {
    let (allowed, snap) = {
        let mut inner = state.inner.lock().unwrap();
        if inner.phase != Phase::Rest {
            return snapshot_of(&inner);
        }
        if inner.settings.strict {
            return snapshot_of(&inner);
        }
        begin_fresh_work(&app, &mut inner);
        (true, snapshot_of(&inner))
    };
    if allowed {
        hide_overlay(&app);
        start_ticker(&app);
        let _ = app.emit("rest-end", &snap);
        let _ = app.emit("work-begin", &snap);
        emit_tick(&app, &snap);
    }
    snap
}

fn rest_now_from_tray(app: &AppHandle) {
    let state = app.state::<AppState>();
    let snap = {
        let mut inner = state.inner.lock().unwrap();
        let secs = u64::from(inner.settings.rest_minutes) * 60;
        if inner.phase == Phase::Work {
            capture_work_summary(app, &mut inner);
        } else {
            state.activity.reset();
            inner.summary_ready = false;
        }
        begin_rest_locked(&mut inner, secs);
        snapshot_of(&inner)
    };
    start_ticker(app);
    show_overlay(app);
    let _ = app.emit("rest-begin", &snap);
    emit_tick(app, &snap);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let settings = load_settings(&app.handle());
            let total = u64::from(settings.work_minutes) * 60;
            ActivityTracker::install_hooks();
            app.manage(AppState {
                inner: Mutex::new(Inner {
                    settings,
                    phase: Phase::Idle,
                    remaining_secs: total,
                    total_secs: total,
                    paused: false,
                    last_summary: None,
                    summary_ready: false,
                }),
                epoch: AtomicU64::new(0),
                activity: ActivityTracker::new(),
            });

            let overlay = WebviewWindowBuilder::new(
                app,
                "overlay",
                WebviewUrl::App("overlay.html".into()),
            )
            .title("课间操")
            .decorations(false)
            .visible(false)
            .skip_taskbar(true)
            .resizable(false)
            .closable(false)
            .minimizable(false)
            .maximizable(false)
            .focused(false)
            .shadow(false)
            .inner_size(800.0, 600.0)
            .build()?;
            let _ = overlay.hide();

            if let Some(main) = app.get_webview_window("main") {
                let _ = main.show();
                let _ = main.unminimize();
                let _ = main.set_focus();
            }

            let open = MenuItem::with_id(app, "open", "打开主窗口", true, None::<&str>)?;
            let rest = MenuItem::with_id(app, "rest", "马上课间操", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出小息", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &rest, &quit])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("小息 · 工间操")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                    "rest" => rest_now_from_tray(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            save_settings,
            start,
            pause,
            reset,
            rest_now,
            skip_rest
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
