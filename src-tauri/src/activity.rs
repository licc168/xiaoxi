use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

const IDLE_AFTER_MS: u64 = 60_000;
const TOP_APPS: usize = 5;
const TITLE_MAX: usize = 40;

static KEYSTROKES: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppRow {
    pub name: String,
    pub title: String,
    pub secs: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkSummary {
    pub work_secs: u64,
    pub idle_secs: u64,
    pub keystrokes: u64,
    pub apps: Vec<AppRow>,
}

struct AppAccum {
    name: String,
    titles: HashMap<String, u64>,
    secs: u64,
}

struct Session {
    apps: HashMap<String, AppAccum>,
    idle_secs: u64,
    keystrokes: u64,
    last_keys: u64,
}

pub struct ActivityTracker {
    inner: Mutex<Session>,
}

impl ActivityTracker {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(empty_session()),
        }
    }

    pub fn install_hooks() {
        #[cfg(windows)]
        win::start_keyboard_hook();
    }

    pub fn reset(&self) {
        if let Ok(mut s) = self.inner.lock() {
            *s = empty_session();
            s.last_keys = KEYSTROKES.load(Ordering::Relaxed);
        }
    }

    pub fn sample(&self) {
        let keys_now = KEYSTROKES.load(Ordering::Relaxed);
        let idle = idle_millis() >= IDLE_AFTER_MS;
        let fg = if idle { None } else { foreground_app() };

        if let Ok(mut s) = self.inner.lock() {
            let delta = keys_now.saturating_sub(s.last_keys);
            s.last_keys = keys_now;
            if idle || fg.is_none() {
                s.idle_secs = s.idle_secs.saturating_add(1);
                return;
            }
            s.keystrokes = s.keystrokes.saturating_add(delta);
            if let Some((exe, name, title)) = fg {
                let entry = s.apps.entry(exe).or_insert_with(|| AppAccum {
                    name,
                    titles: HashMap::new(),
                    secs: 0,
                });
                entry.secs = entry.secs.saturating_add(1);
                if !title.is_empty() {
                    *entry.titles.entry(title).or_insert(0) += 1;
                }
            }
        }
    }

    pub fn finish(&self) -> Option<WorkSummary> {
        self.sample();
        let mut s = self.inner.lock().ok()?;
        let summary = snapshot(&s);
        *s = empty_session();
        s.last_keys = KEYSTROKES.load(Ordering::Relaxed);
        summary
    }
}

fn empty_session() -> Session {
    Session {
        apps: HashMap::new(),
        idle_secs: 0,
        keystrokes: 0,
        last_keys: KEYSTROKES.load(Ordering::Relaxed),
    }
}

fn snapshot(s: &Session) -> Option<WorkSummary> {
    let work_secs: u64 = s.apps.values().map(|a| a.secs).sum();
    if work_secs == 0 && s.idle_secs == 0 {
        return None;
    }

    let mut apps: Vec<AppRow> = s
        .apps
        .values()
        .map(|a| {
            let title = a
                .titles
                .iter()
                .max_by_key(|(_, secs)| *secs)
                .map(|(t, _)| t.clone())
                .unwrap_or_default();
            AppRow {
                name: a.name.clone(),
                title,
                secs: a.secs,
            }
        })
        .collect();
    apps.sort_by(|a, b| b.secs.cmp(&a.secs).then_with(|| a.name.cmp(&b.name)));

    if apps.len() > TOP_APPS {
        let rest: u64 = apps[TOP_APPS..].iter().map(|a| a.secs).sum();
        apps.truncate(TOP_APPS);
        if rest > 0 {
            apps.push(AppRow {
                name: "其他".into(),
                title: String::new(),
                secs: rest,
            });
        }
    }

    Some(WorkSummary {
        work_secs,
        idle_secs: s.idle_secs,
        keystrokes: s.keystrokes,
        apps,
    })
}

fn idle_millis() -> u64 {
    #[cfg(windows)]
    {
        win::idle_millis()
    }
    #[cfg(not(windows))]
    {
        0
    }
}

fn foreground_app() -> Option<(String, String, String)> {
    #[cfg(windows)]
    {
        win::foreground_app()
    }
    #[cfg(not(windows))]
    {
        None
    }
}

fn friendly_name(stem: &str) -> String {
    match stem.to_ascii_lowercase().as_str() {
        "code" => "Visual Studio Code".into(),
        "devenv" => "Visual Studio".into(),
        "cursor" => "Cursor".into(),
        "windsurf" => "Windsurf".into(),
        "chrome" => "Google Chrome".into(),
        "msedge" => "Microsoft Edge".into(),
        "firefox" => "Firefox".into(),
        "wechat" | "weixin" | "wechatappex" => "微信".into(),
        "qq" | "qqex" => "QQ".into(),
        "dingtalk" => "钉钉".into(),
        "feishu" | "lark" => "飞书".into(),
        "wps" | "wpsmain" => "WPS".into(),
        "winword" => "Word".into(),
        "excel" => "Excel".into(),
        "powerpnt" => "PowerPoint".into(),
        "notepad" => "记事本".into(),
        "explorer" => "文件资源管理器".into(),
        "cmd" => "命令提示符".into(),
        "powershell" | "pwsh" | "windowsterminal" | "windowsterminalpreview" => "终端".into(),
        "idea64" | "idea" => "IntelliJ IDEA".into(),
        "pycharm64" | "pycharm" => "PyCharm".into(),
        "webstorm64" | "webstorm" => "WebStorm".into(),
        "goland64" | "goland" => "GoLand".into(),
        "clion64" | "clion" => "CLion".into(),
        "studio64" => "Android Studio".into(),
        "slack" => "Slack".into(),
        "discord" => "Discord".into(),
        "spotify" => "Spotify".into(),
        "obs64" | "obs32" | "obs" => "OBS".into(),
        "telegram" | "telegram.exe" => "Telegram".into(),
        "outlook" | "olk" => "Outlook".into(),
        "teams" | "ms-teams" => "Teams".into(),
        "zoom" => "Zoom".into(),
        "postman" => "Postman".into(),
        "figma" => "Figma".into(),
        "notepad++" => "Notepad++".into(),
        "sublime_text" => "Sublime Text".into(),
        "typora" => "Typora".into(),
        "obsidian" => "Obsidian".into(),
        "xiaoxi" => "小息".into(),
        "applicationframehost" => "Windows 应用".into(),
        other => other.to_string(),
    }
}

fn clip_title(raw: &str) -> String {
    let t = raw.trim();
    if t.chars().count() <= TITLE_MAX {
        t.to_string()
    } else {
        let clipped: String = t.chars().take(TITLE_MAX).collect();
        format!("{clipped}…")
    }
}

#[cfg(windows)]
mod win {
    use super::{clip_title, friendly_name, KEYSTROKES};
    use std::os::windows::ffi::OsStringExt;
    use std::path::Path;
    use std::sync::atomic::Ordering;
    use windows::core::PWSTR;
    use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::SystemInformation::GetTickCount;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetClassNameW, GetForegroundWindow, GetMessageW,
        GetWindowTextW, GetWindowThreadProcessId, SetWindowsHookExW, TranslateMessage,
        UnhookWindowsHookEx, HC_ACTION, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
    };

    const SKIP_CLASSES: &[&str] = &[
        "Progman",
        "WorkerW",
        "Shell_TrayWnd",
        "Shell_SecondaryTrayWnd",
        "NotifyIconOverflowWindow",
    ];

    pub fn start_keyboard_hook() {
        std::thread::Builder::new()
            .name("key-count".into())
            .spawn(|| unsafe {
                let hook = match SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0) {
                    Ok(h) => h,
                    Err(_) => return,
                };
                let mut msg = MSG::default();
                loop {
                    let ret = GetMessageW(&mut msg, None, 0, 0);
                    if ret.0 == 0 || ret.0 == -1 {
                        break;
                    }
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
                let _ = UnhookWindowsHookEx(hook);
            })
            .ok();
    }

    unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code == HC_ACTION as i32 {
            let msg = wparam.0 as u32;
            if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
                KEYSTROKES.fetch_add(1, Ordering::Relaxed);
            }
        }
        unsafe { CallNextHookEx(None, code, wparam, lparam) }
    }

    pub fn idle_millis() -> u64 {
        unsafe {
            let mut info = LASTINPUTINFO {
                cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
                dwTime: 0,
            };
            if GetLastInputInfo(&mut info).as_bool() {
                u64::from(GetTickCount().wrapping_sub(info.dwTime))
            } else {
                0
            }
        }
    }

    pub fn foreground_app() -> Option<(String, String, String)> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_invalid() || hwnd.0.is_null() {
                return None;
            }

            let class_name = read_class(hwnd);
            if SKIP_CLASSES.iter().any(|c| class_name.eq_ignore_ascii_case(c)) {
                return None;
            }

            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == 0 {
                return None;
            }

            let exe_path = process_path(pid);
            let stem = exe_path
                .as_ref()
                .and_then(|p| Path::new(p).file_stem())
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            if stem.is_empty() {
                return None;
            }

            let title = clip_title(&read_title(hwnd));
            let name = if stem.eq_ignore_ascii_case("applicationframehost") && !title.is_empty() {
                title.clone()
            } else {
                friendly_name(&stem)
            };
            Some((stem.to_ascii_lowercase(), name, title))
        }
    }

    unsafe fn read_title(hwnd: HWND) -> String {
        let mut buf = [0u16; 512];
        let n = GetWindowTextW(hwnd, &mut buf);
        if n <= 0 {
            String::new()
        } else {
            String::from_utf16_lossy(&buf[..n as usize])
        }
    }

    unsafe fn read_class(hwnd: HWND) -> String {
        let mut buf = [0u16; 256];
        let n = GetClassNameW(hwnd, &mut buf);
        if n <= 0 {
            String::new()
        } else {
            String::from_utf16_lossy(&buf[..n as usize])
        }
    }

    fn process_path(pid: u32) -> Option<String> {
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
            let mut size = 260u32;
            let mut buf = vec![0u16; size as usize];
            let result = QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_WIN32,
                PWSTR(buf.as_mut_ptr()),
                &mut size,
            );
            let _ = CloseHandle(handle);
            result.ok()?;
            let os = std::ffi::OsString::from_wide(&buf[..size as usize]);
            Some(os.to_string_lossy().into_owned())
        }
    }
}
