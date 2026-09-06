//! The floating pill: state broadcasting and window placement. The overlay
//! window never takes focus; it only listens to "lalia://overlay" events.

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, PhysicalPosition};

use crate::settings::{OverlayPosition, OverlaySettings};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverlayState {
    Idle,
    Starting,
    Recording,
    HandsFree,
    Processing,
    Cleaning,
    Success,
    Cancelled,
    NoSpeech,
    MicUnavailable,
    ModelUnavailable,
    Offline,
    Failed,
    TargetChanged,
    Sensitive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayPayload {
    pub state: OverlayState,
    pub message: Option<String>,
    /// Short preview of the inserted text on success.
    pub preview: Option<String>,
    pub can_retry: bool,
    pub seconds: f32,
}

pub fn emit_state(app: &tauri::AppHandle, payload: OverlayPayload) {
    let _ = app.emit_to("overlay", "lalia://overlay", &payload);
    let _ = app.emit_to("main", "lalia://overlay", &payload);
}

pub fn emit_level(app: &tauri::AppHandle, level: f32, seconds: f32) {
    let _ = app.emit_to("overlay", "lalia://level", serde_json::json!({ "level": level, "seconds": seconds }));
}

pub fn show(app: &tauri::AppHandle, settings: &OverlaySettings, target_hwnd: isize) {
    let Some(w) = app.get_webview_window("overlay") else { return };
    place(&w, settings, target_hwnd);
    let _ = w.show();
}

pub fn hide(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("overlay") {
        let _ = w.hide();
    }
}

/// Work area of the monitor that contains the target window (or the primary).
#[cfg(windows)]
fn work_area_for(target_hwnd: isize) -> Option<(i32, i32, i32, i32)> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTOPRIMARY};
    unsafe {
        let hwnd = HWND(target_hwnd as *mut core::ffi::c_void);
        let mon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY);
        let mut info = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        if GetMonitorInfoW(mon, &mut info).as_bool() {
            let r = info.rcWork;
            Some((r.left, r.top, r.right, r.bottom))
        } else {
            None
        }
    }
}

#[cfg(not(windows))]
fn work_area_for(_target_hwnd: isize) -> Option<(i32, i32, i32, i32)> {
    None
}

pub fn place(w: &tauri::WebviewWindow, settings: &OverlaySettings, target_hwnd: isize) {
    let size = w.outer_size().unwrap_or(tauri::PhysicalSize { width: 260, height: 84 });
    let (left, top, right, bottom) = work_area_for(target_hwnd).unwrap_or((0, 0, 1920, 1040));
    let (x, y) = match settings.position {
        OverlayPosition::BottomCenter => ((left + right) / 2 - size.width as i32 / 2, bottom - size.height as i32 - 24),
        OverlayPosition::TopCenter => ((left + right) / 2 - size.width as i32 / 2, top + 24),
        OverlayPosition::BottomRight => (right - size.width as i32 - 24, bottom - size.height as i32 - 24),
        OverlayPosition::BottomLeft => (left + 24, bottom - size.height as i32 - 24),
        OverlayPosition::Custom => (settings.custom_x, settings.custom_y),
    };
    let _ = w.set_position(PhysicalPosition::new(x, y));
}
