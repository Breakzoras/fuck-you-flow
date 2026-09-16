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
    /// The pill was put on screen only so the user can drag it somewhere
    /// else (the button in Settings); it takes the mouse and shows a hint.
    #[serde(default)]
    pub movable: bool,
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

/// Work area of the monitor under a point on screen, for a pill that was just
/// dropped there: it may have been dragged to another screen than the one
/// the target window lives on.
#[cfg(windows)]
fn work_area_at(x: i32, y: i32) -> Option<(i32, i32, i32, i32)> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST};
    unsafe {
        let mon = MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST);
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
fn work_area_at(_x: i32, _y: i32) -> Option<(i32, i32, i32, i32)> {
    None
}

/// Gap between the pill and the edge it sits on, in pixels.
const EDGE_GAP: i32 = 24;

/// Where a docked pill goes: `along` is 0.0 at the left (or top) end of the
/// edge and 1.0 at the right (or bottom) end, so the setting means the same
/// thing on a screen of any size.
pub fn docked_xy(work: (i32, i32, i32, i32), size: (i32, i32), edge: &str, along: f32) -> (i32, i32) {
    let (left, top, right, bottom) = work;
    let (w, h) = size;
    let along = along.clamp(0.0, 1.0);
    let span_x = (right - left - w - 2 * EDGE_GAP).max(0) as f32;
    let span_y = (bottom - top - h - 2 * EDGE_GAP).max(0) as f32;
    let x_along = left + EDGE_GAP + (span_x * along).round() as i32;
    let y_along = top + EDGE_GAP + (span_y * along).round() as i32;
    match edge {
        "top" => (x_along, top + EDGE_GAP),
        "left" => (left + EDGE_GAP, y_along),
        "right" => (right - w - EDGE_GAP, y_along),
        _ => (x_along, bottom - h - EDGE_GAP),
    }
}

/// The reverse: the pill was let go with its top-left corner at (x, y). It
/// sticks to the nearest of the four edges and keeps its place along it.
pub fn dock_from_xy(work: (i32, i32, i32, i32), size: (i32, i32), x: i32, y: i32) -> (&'static str, f32) {
    let (left, top, right, bottom) = work;
    let (w, h) = size;
    let cx = x + w / 2;
    let cy = y + h / 2;
    let d_left = (cx - left).max(0);
    let d_right = (right - cx).max(0);
    let d_top = (cy - top).max(0);
    let d_bottom = (bottom - cy).max(0);
    let span_x = (right - left - w - 2 * EDGE_GAP).max(1) as f32;
    let span_y = (bottom - top - h - 2 * EDGE_GAP).max(1) as f32;
    let along_x = ((x - left - EDGE_GAP) as f32 / span_x).clamp(0.0, 1.0);
    let along_y = ((y - top - EDGE_GAP) as f32 / span_y).clamp(0.0, 1.0);
    let min = d_left.min(d_right).min(d_top).min(d_bottom);
    // Ties go to the bottom, then top: the long edges are where a bar reads best.
    if min == d_bottom {
        ("bottom", along_x)
    } else if min == d_top {
        ("top", along_x)
    } else if min == d_left {
        ("left", along_y)
    } else {
        ("right", along_y)
    }
}

/// Turn the spot where the user dropped the pill into a saved dock.
pub fn dock_at(w: &tauri::WebviewWindow, x: i32, y: i32) -> (&'static str, f32) {
    let size = w.outer_size().unwrap_or(tauri::PhysicalSize { width: 260, height: 84 });
    let work = work_area_at(x + size.width as i32 / 2, y + size.height as i32 / 2).unwrap_or((0, 0, 1920, 1040));
    dock_from_xy(work, (size.width as i32, size.height as i32), x, y)
}

pub fn place(w: &tauri::WebviewWindow, settings: &OverlaySettings, target_hwnd: isize) {
    let size = w.outer_size().unwrap_or(tauri::PhysicalSize { width: 260, height: 84 });
    let (left, top, right, bottom) = work_area_for(target_hwnd).unwrap_or((0, 0, 1920, 1040));
    let (x, y) = match settings.position {
        OverlayPosition::BottomCenter => ((left + right) / 2 - size.width as i32 / 2, bottom - size.height as i32 - EDGE_GAP),
        OverlayPosition::TopCenter => ((left + right) / 2 - size.width as i32 / 2, top + EDGE_GAP),
        OverlayPosition::BottomRight => (right - size.width as i32 - EDGE_GAP, bottom - size.height as i32 - EDGE_GAP),
        OverlayPosition::BottomLeft => (left + EDGE_GAP, bottom - size.height as i32 - EDGE_GAP),
        OverlayPosition::Custom => (settings.custom_x, settings.custom_y),
        OverlayPosition::Docked => docked_xy((left, top, right, bottom), (size.width as i32, size.height as i32), &settings.dock_edge, settings.dock_along),
    };
    let _ = w.set_position(PhysicalPosition::new(x, y));
}

#[cfg(test)]
mod dock_tests {
    use super::*;

    const WORK: (i32, i32, i32, i32) = (0, 0, 1920, 1040);
    const SIZE: (i32, i32) = (340, 150);

    #[test]
    fn dropping_near_an_edge_sticks_to_it_and_keeps_the_spot() {
        // low on the screen, a third of the way across
        let (edge, along) = dock_from_xy(WORK, SIZE, 500, 800);
        assert_eq!(edge, "bottom");
        let (x, y) = docked_xy(WORK, SIZE, edge, along);
        assert_eq!(y, 1040 - 150 - EDGE_GAP);
        assert!((x - 500).abs() <= 1, "x stayed where it was dropped: {x}");

        let (edge, along) = dock_from_xy(WORK, SIZE, 30, 400);
        assert_eq!(edge, "left");
        let (x, y) = docked_xy(WORK, SIZE, edge, along);
        assert_eq!(x, EDGE_GAP);
        assert!((y - 400).abs() <= 1, "y stayed where it was dropped: {y}");

        assert_eq!(dock_from_xy(WORK, SIZE, 1500, 10).0, "top");
        assert_eq!(dock_from_xy(WORK, SIZE, 1560, 500).0, "right");
    }

    #[test]
    fn the_fraction_lands_on_the_same_spot_on_a_bigger_screen() {
        let (edge, along) = dock_from_xy(WORK, SIZE, 1920 - 340 - EDGE_GAP, 900);
        assert_eq!(edge, "bottom");
        assert!((along - 1.0).abs() < 0.01);
        let wide = (0, 0, 3840, 2120);
        let (x, y) = docked_xy(wide, SIZE, edge, along);
        assert_eq!((x, y), (3840 - 340 - EDGE_GAP, 2120 - 150 - EDGE_GAP));
    }

    #[test]
    fn a_pill_dropped_off_screen_is_pulled_back_inside() {
        let (edge, along) = dock_from_xy(WORK, SIZE, -300, 2000);
        assert_eq!(edge, "bottom");
        assert_eq!(along, 0.0);
        assert_eq!(docked_xy(WORK, SIZE, edge, along), (EDGE_GAP, 1040 - 150 - EDGE_GAP));
    }
}
