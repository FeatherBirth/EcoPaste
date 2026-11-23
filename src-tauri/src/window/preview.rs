//! 系统级剪贴板预览窗口骨架。
//!
//! 预览窗口是透明的 full-screen overlay：Rust 负责按需建窗、复用窗口、
//! 收集坐标上下文并广播，前端在该窗口内渲染预览内容与连接曲线。
//! 窗口接入生命周期管理（`DestroyWhenIdle`）：隐藏空闲后销毁 WebView 释放内存，
//! 仅在预览请求到达时按需建窗，不随剪贴板窗口显示预热。

#![allow(clippy::unused_unit)]

#[cfg(target_os = "windows")]
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalRect, PhysicalSize, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};

use crate::core::Result;

use super::{get_window, lifecycle, CLIPBOARD_PREVIEW_WINDOW_LABEL, CLIPBOARD_WINDOW_LABEL};

#[cfg(target_os = "macos")]
use tauri_nspanel::{tauri_panel, ManagerExt, PanelLevel, WebviewWindowExt};

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, WPARAM};
#[cfg(target_os = "windows")]
use windows::Win32::System::SystemServices::GC_ZOOM;
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::Touch::{
    CloseGestureInfoHandle, GetGestureInfo, SetGestureConfig, GESTURECONFIG, GESTUREINFO, GID_ZOOM,
    HGESTUREINFO,
};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, GetForegroundWindow, IsWindow, SetForegroundWindow, SetWindowPos, GF_BEGIN,
    GF_END, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, WM_GESTURE,
    WM_GESTURENOTIFY,
};

const PREVIEW_UPDATED_EVENT: &str = "preview://updated";
const PREVIEW_INTERACTION_EVENT: &str = "preview://interaction";
#[cfg(target_os = "windows")]
const PREVIEW_GESTURE_EVENT: &str = "preview://gesture";
const PREVIEW_PANEL_WIDTH: f64 = 480.0;
const PREVIEW_PANEL_HEIGHT: f64 = 480.0;
const PREVIEW_PANEL_GAP: f64 = 40.0;
const PREVIEW_PANEL_MARGIN: f64 = 32.0;
const PREVIEW_HIDE_DELAY_MS: u64 = 180;
const PREVIEW_INTERACTION_POLL_MS: u64 = 16;

static PREVIEW_LATEST_REQUEST_ID: AtomicU64 = AtomicU64::new(0);
static PREVIEW_SHOW_GENERATION: AtomicU64 = AtomicU64::new(0);
static PREVIEW_SESSION_ID: AtomicU64 = AtomicU64::new(0);
static PREVIEW_SUPPRESSED: AtomicBool = AtomicBool::new(false);
static PREVIEW_STATE: LazyLock<Mutex<Option<ClipboardPreviewState>>> =
    LazyLock::new(|| Mutex::new(None));
static PREVIEW_COMMIT_LOCK: Mutex<()> = Mutex::new(());
static PREVIEW_INTERACTION: LazyLock<Mutex<PreviewInteractionRuntime>> =
    LazyLock::new(|| Mutex::new(PreviewInteractionRuntime::default()));
#[cfg(target_os = "windows")]
static PREVIEW_PREVIOUS_FOREGROUND: Mutex<Option<isize>> = Mutex::new(None);
#[cfg(target_os = "windows")]
static PREVIEW_GESTURE_APP: LazyLock<Mutex<Option<AppHandle>>> = LazyLock::new(|| Mutex::new(None));
#[cfg(target_os = "windows")]
static PREVIEW_GESTURE_WINDOWS: LazyLock<Mutex<Vec<isize>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));
#[cfg(target_os = "windows")]
static PREVIEW_GESTURE_DISTANCE: AtomicU64 = AtomicU64::new(0);
/// 串行化建窗：多个预览请求（如连续 hover）可能并发走到「检查不存在 → 建窗」，
/// 都过了存在性检查会触发重复 label 建窗报错。建窗都来自命令/后台线程、主线程从不持锁，
/// 不会与 builder 内部的主线程派发互锁。
static PREVIEW_BUILD_LOCK: Mutex<()> = Mutex::new(());

#[cfg(target_os = "macos")]
tauri_panel! {
    panel!(PreviewPanel {
        config: {
            is_floating_panel: true,
            can_become_key_window: true,
            can_become_main_window: false
        }
    })
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewAnchorRect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewWorkArea {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewClipboardWindowRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewRect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PreviewPlacement {
    Right,
    Left,
    Bottom,
    Top,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewLayout {
    pub overlay_rect: PreviewRect,
    pub source_rect: PreviewRect,
    pub panel_rect: PreviewRect,
    pub placement: PreviewPlacement,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardPreviewState {
    pub request_id: u64,
    pub session_id: u64,
    pub item_id: String,
    pub anchor: PreviewAnchorRect,
    pub scale_factor: f64,
    pub work_area: PreviewWorkArea,
    pub clipboard_window: Option<PreviewClipboardWindowRect>,
    pub layout: PreviewLayout,
}

#[derive(Clone, Debug, Default)]
struct PreviewInteractionRuntime {
    request_id: u64,
    session_id: u64,
    panel_rect: Option<PreviewRect>,
    active: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreviewInteractionPayload {
    request_id: u64,
    session_id: u64,
    active: bool,
}

#[cfg(target_os = "windows")]
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreviewGesturePayload {
    request_id: u64,
    session_id: u64,
    factor: f64,
}

/// 打开或重定向预览窗口，并把最新预览状态广播到预览 webview。
pub async fn show_clipboard_preview(
    app: &AppHandle,
    item_id: String,
    anchor: PreviewAnchorRect,
    request_id: u64,
) -> Result<Option<ClipboardPreviewState>> {
    validate_anchor(&anchor)?;

    if request_id == 0 || !claim_preview_request(request_id) {
        return Ok(None);
    }
    let show_generation = PREVIEW_SHOW_GENERATION.load(Ordering::SeqCst);

    if PREVIEW_SUPPRESSED.load(Ordering::SeqCst) || !is_clipboard_window_visible(app) {
        close_clipboard_preview_now(app)?;
        return Ok(None);
    }

    let window = ensure_preview_window(app).await?;
    if !is_preview_request_current(request_id)
        || !is_preview_show_generation_current(show_generation)
    {
        return Ok(None);
    }

    let monitor = resolve_preview_monitor(app)?;
    let work_area = preview_overlay_bounds(&monitor);
    let scale_factor = monitor.scale_factor();
    let clipboard_window = clipboard_window_rect(app);
    let layout = build_preview_layout(&anchor, scale_factor, &work_area, clipboard_window.as_ref());
    let session_id = preview_session_id_for_show();

    let state = {
        let _guard = preview_commit_lock();
        if !is_preview_request_current(request_id)
            || !is_preview_show_generation_current(show_generation)
        {
            return Ok(None);
        }

        prepare_preview_window_for_show(app, &window, &work_area)?;

        let state = ClipboardPreviewState {
            request_id,
            session_id,
            item_id,
            anchor,
            scale_factor,
            work_area: PreviewWorkArea {
                x: work_area.position.x,
                y: work_area.position.y,
                width: work_area.size.width,
                height: work_area.size.height,
            },
            layout,
            clipboard_window,
        };

        clear_preview_interaction(app);
        set_preview_interaction_runtime(&state, None);
        set_preview_state(Some(state.clone()));
        window
            .emit(PREVIEW_UPDATED_EVENT, &state)
            .map_err(|e| anyhow::anyhow!(e))?;
        show_preview_window(app, &window, &work_area)?;
        state
    };

    #[cfg(target_os = "windows")]
    install_windows_preview_gesture_handlers(app, &window);
    start_preview_interaction_tracking(app.clone(), window, state.clone());

    Ok(Some(state))
}

/// 隐藏预览窗口并清空当前预览状态。
pub fn close_clipboard_preview(app: &AppHandle, request_id: u64) -> Result<()> {
    if request_id == 0 || !claim_preview_request(request_id) {
        return Ok(());
    }

    let _guard = preview_commit_lock();
    set_preview_state(None);
    clear_preview_interaction(app);

    if let Some(window) = app.get_webview_window(CLIPBOARD_PREVIEW_WINDOW_LABEL) {
        window
            .emit(PREVIEW_UPDATED_EVENT, Option::<ClipboardPreviewState>::None)
            .map_err(|e| anyhow::anyhow!(e))?;
        schedule_preview_window_hide(app.clone(), window, request_id);
    }

    Ok(())
}

/// 立即隐藏预览窗口并清空状态；用于剪贴板窗口隐藏等不需要退出动画的路径。
pub fn close_clipboard_preview_now(app: &AppHandle) -> Result<()> {
    let _guard = preview_commit_lock();
    PREVIEW_SHOW_GENERATION.fetch_add(1, Ordering::SeqCst);
    set_preview_state(None);
    clear_preview_interaction(app);

    if let Some(window) = app.get_webview_window(CLIPBOARD_PREVIEW_WINDOW_LABEL) {
        window
            .emit(PREVIEW_UPDATED_EVENT, Option::<ClipboardPreviewState>::None)
            .map_err(|e| anyhow::anyhow!(e))?;
        hide_preview_window(app, &window)?;
    }

    Ok(())
}

/// 剪贴板窗口开始隐藏时压制后续过期 show 请求，并立即收起预览窗口。
pub fn suppress_for_clipboard_hide(app: &AppHandle) {
    PREVIEW_SUPPRESSED.store(true, Ordering::SeqCst);
    if let Err(error) = close_clipboard_preview_now(app) {
        log::error!("suppress preview on clipboard hide failed: {error}");
    }
}

/// 剪贴板窗口重新显示后允许新的预览请求进入。预览窗口不随剪贴板窗口预创建，
/// 由首次 [`show_clipboard_preview`] 经 `ensure_preview_window` 按需建窗。
pub fn resume_after_clipboard_show() {
    PREVIEW_SUPPRESSED.store(false, Ordering::SeqCst);
}

/// 返回预览窗口最近一次收到的状态，供预览页首屏补拉。
pub fn get_clipboard_preview_state() -> Result<Option<ClipboardPreviewState>> {
    Ok(preview_state_snapshot())
}

/// Return whether a physical screen point is inside the current preview panel.
/// The Windows outside-click hook uses this before hiding the clipboard window.
pub(crate) fn cursor_inside_preview_panel(cursor_x: i32, cursor_y: i32) -> bool {
    let Some(state) = preview_state_snapshot() else {
        return false;
    };
    let guard = PREVIEW_INTERACTION.lock().unwrap_or_else(|poisoned| {
        log::error!("preview interaction mutex poisoned on hit test, recovering");
        poisoned.into_inner()
    });
    if guard.request_id != state.request_id || guard.session_id != state.session_id {
        return false;
    }
    let Some(panel_rect) = guard.panel_rect else {
        return false;
    };

    preview_panel_contains_physical(&state, panel_rect, cursor_x as f64, cursor_y as f64)
}

/// 更新当前预览面板的可见矩形，用于只在实际面板范围开启鼠标输入。
pub fn set_clipboard_preview_panel_rect(request_id: u64, panel_rect: PreviewRect) -> Result<()> {
    validate_panel_rect(&panel_rect)?;

    let Some(state) = preview_state_snapshot() else {
        return Ok(());
    };
    if state.request_id != request_id {
        return Ok(());
    }

    let mut guard = PREVIEW_INTERACTION.lock().unwrap_or_else(|poisoned| {
        log::error!("preview interaction mutex poisoned on rect, recovering");
        poisoned.into_inner()
    });
    if guard.request_id != request_id || guard.session_id != state.session_id {
        return Ok(());
    }

    guard.panel_rect = Some(panel_rect);
    Ok(())
}

/// 按需重建预览窗口。预览窗口不再由 Tauri 配置预创建（改为空闲销毁 + 按需重建），
/// 所有选项必须在此用 builder 完整复刻原 `tauri.conf.json` 声明，否则重建后行为漂移。
///
/// 建窗后保持 `visible: false`：定位与显示由预览 show 流程统一处理；
/// macOS 的 NSPanel 转换由 [`ensure_preview_window`] 在每次取窗时兜底执行。
pub fn build_clipboard_preview_window(app: &AppHandle) -> Result<()> {
    let _guard = PREVIEW_BUILD_LOCK.lock().unwrap_or_else(|poisoned| {
        log::error!("preview build mutex poisoned, recovering");
        poisoned.into_inner()
    });

    if app
        .get_webview_window(CLIPBOARD_PREVIEW_WINDOW_LABEL)
        .is_some()
    {
        return Ok(());
    }

    WebviewWindowBuilder::new(
        app,
        CLIPBOARD_PREVIEW_WINDOW_LABEL,
        WebviewUrl::App("index.html/#/preview".into()),
    )
    .title("EcoPaste Preview")
    .inner_size(1.0, 1.0)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .always_on_top(true)
    .decorations(false)
    .shadow(false)
    .transparent(true)
    .skip_taskbar(true)
    .focused(false)
    .focusable(false)
    .disable_drag_drop_handler()
    .visible(false)
    .build()
    .map_err(|err| anyhow::anyhow!("build clipboard preview window: {err}"))?;

    Ok(())
}

fn set_preview_state(state: Option<ClipboardPreviewState>) {
    let mut guard = PREVIEW_STATE.lock().unwrap_or_else(|poisoned| {
        log::error!("preview state mutex poisoned on set, recovering");
        poisoned.into_inner()
    });
    *guard = state;
}

fn preview_state_snapshot() -> Option<ClipboardPreviewState> {
    let guard = PREVIEW_STATE.lock().unwrap_or_else(|poisoned| {
        log::error!("preview state mutex poisoned on get, recovering");
        poisoned.into_inner()
    });

    guard.clone()
}

fn preview_commit_lock() -> std::sync::MutexGuard<'static, ()> {
    PREVIEW_COMMIT_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// 客户端请求号必须严格递增；等于或小于已见最大值都视为过期请求。
fn should_accept_preview_request(previous: u64, request_id: u64) -> bool {
    request_id > previous
}

fn claim_preview_request(request_id: u64) -> bool {
    let previous = PREVIEW_LATEST_REQUEST_ID.fetch_max(request_id, Ordering::SeqCst);

    should_accept_preview_request(previous, request_id)
}

fn is_preview_request_current(request_id: u64) -> bool {
    PREVIEW_LATEST_REQUEST_ID.load(Ordering::SeqCst) == request_id
}

fn is_preview_show_generation_current(generation: u64) -> bool {
    PREVIEW_SHOW_GENERATION.load(Ordering::SeqCst) == generation
}

fn set_preview_interaction_runtime(state: &ClipboardPreviewState, panel_rect: Option<PreviewRect>) {
    let mut guard = PREVIEW_INTERACTION.lock().unwrap_or_else(|poisoned| {
        log::error!("preview interaction mutex poisoned on reset, recovering");
        poisoned.into_inner()
    });

    *guard = PreviewInteractionRuntime {
        request_id: state.request_id,
        session_id: state.session_id,
        panel_rect,
        active: false,
    };
}

fn clear_preview_interaction(app: &AppHandle) {
    {
        let mut guard = PREVIEW_INTERACTION.lock().unwrap_or_else(|poisoned| {
            log::error!("preview interaction mutex poisoned on clear, recovering");
            poisoned.into_inner()
        });
        *guard = PreviewInteractionRuntime::default();
    }

    if let Some(window) = app.get_webview_window(CLIPBOARD_PREVIEW_WINDOW_LABEL) {
        if let Err(error) = apply_preview_input_capture(app, &window, false) {
            log::warn!("reset preview input capture failed: {error}");
        }
    }

    #[cfg(target_os = "windows")]
    clear_windows_preview_gesture_handlers();
}

#[cfg(target_os = "windows")]
fn install_windows_preview_gesture_handlers(app: &AppHandle, window: &WebviewWindow) {
    let Ok(raw_hwnd) = window.hwnd() else {
        return;
    };
    let hwnd = HWND(raw_hwnd.0 as isize);

    {
        let mut guard = PREVIEW_GESTURE_APP
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = Some(app.clone());
    }

    unsafe {
        install_preview_gesture_hwnd(hwnd);
        let _ = EnumChildWindows(hwnd, Some(install_preview_gesture_child), LPARAM(0));
    }
}

#[cfg(target_os = "windows")]
extern "system" fn install_preview_gesture_child(hwnd: HWND, _lparam: LPARAM) -> BOOL {
    unsafe {
        install_preview_gesture_hwnd(hwnd);
    }
    BOOL(1)
}

/// Route native Windows precision-touchpad pinch gestures through the preview
/// while the pointer is inside the visible panel.
#[cfg(target_os = "windows")]
unsafe fn install_preview_gesture_hwnd(hwnd: HWND) {
    {
        let installed = PREVIEW_GESTURE_WINDOWS
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if installed.contains(&hwnd.0) {
            return;
        }
    }

    if !configure_preview_gesture_hwnd(hwnd) {
        return;
    }

    if !SetWindowSubclass(
        hwnd,
        Some(preview_gesture_subclass_proc),
        preview_gesture_subclass_id(),
        0,
    )
    .as_bool()
    {
        return;
    }

    PREVIEW_GESTURE_WINDOWS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .push(hwnd.0);
    log::debug!("preview gesture enabled on hwnd=0x{:x}", hwnd.0);
}

#[cfg(target_os = "windows")]
fn clear_windows_preview_gesture_handlers() {
    let handles = {
        let mut guard = PREVIEW_GESTURE_WINDOWS
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::mem::take(&mut *guard)
    };

    PREVIEW_GESTURE_DISTANCE.store(0, Ordering::SeqCst);
    PREVIEW_GESTURE_APP
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take();

    for raw_hwnd in handles {
        unsafe {
            RemoveWindowSubclass(
                HWND(raw_hwnd),
                Some(preview_gesture_subclass_proc),
                preview_gesture_subclass_id(),
            );
        }
    }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn preview_gesture_subclass_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _subclass_id: usize,
    _ref_data: usize,
) -> LRESULT {
    if message == WM_GESTURENOTIFY {
        let _ = configure_preview_gesture_hwnd(hwnd);
        return DefSubclassProc(hwnd, message, wparam, lparam);
    }

    if message != WM_GESTURE {
        return DefSubclassProc(hwnd, message, wparam, lparam);
    }

    let gesture_handle = HGESTUREINFO(lparam.0);
    let mut info = GESTUREINFO {
        cbSize: size_of::<GESTUREINFO>() as u32,
        ..Default::default()
    };
    if GetGestureInfo(gesture_handle, &mut info).is_err() {
        return DefSubclassProc(hwnd, message, wparam, lparam);
    }

    let Some(state) = preview_state_snapshot() else {
        return DefSubclassProc(hwnd, message, wparam, lparam);
    };
    let inside =
        cursor_inside_preview_panel(i32::from(info.ptsLocation.x), i32::from(info.ptsLocation.y));
    if info.dwID != GID_ZOOM.0 || !inside || !is_preview_request_current(state.request_id) {
        return DefSubclassProc(hwnd, message, wparam, lparam);
    }

    let distance = info.ullArguments & u64::from(u32::MAX);
    let previous = PREVIEW_GESTURE_DISTANCE.swap(distance, Ordering::SeqCst);
    let should_emit =
        info.dwFlags & GF_BEGIN == 0 && info.dwFlags & GF_END == 0 && previous > 0 && distance > 0;
    let factor = if should_emit {
        (distance as f64 / previous as f64).clamp(0.8, 1.25)
    } else {
        1.0
    };

    if info.dwFlags & GF_END != 0 {
        PREVIEW_GESTURE_DISTANCE.store(0, Ordering::SeqCst);
    }

    if should_emit && factor != 1.0 {
        let app = PREVIEW_GESTURE_APP
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if let Some(app) = app {
            let payload = PreviewGesturePayload {
                request_id: state.request_id,
                session_id: state.session_id,
                factor,
            };
            if let Err(error) = app.emit(PREVIEW_GESTURE_EVENT, payload) {
                log::warn!("emit {PREVIEW_GESTURE_EVENT} failed: {error}");
            }
        }
    }

    let _ = CloseGestureInfoHandle(gesture_handle);
    LRESULT(0)
}

#[cfg(target_os = "windows")]
const fn preview_gesture_subclass_id() -> usize {
    0xE0C0
}

#[cfg(target_os = "windows")]
unsafe fn configure_preview_gesture_hwnd(hwnd: HWND) -> bool {
    let config = GESTURECONFIG {
        dwID: GID_ZOOM,
        dwWant: GC_ZOOM.0,
        dwBlock: 0,
    };
    SetGestureConfig(hwnd, 0, &[config], size_of::<GESTURECONFIG>() as u32).is_ok()
}

fn start_preview_interaction_tracking(
    app: AppHandle,
    window: WebviewWindow,
    state: ClipboardPreviewState,
) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(PREVIEW_INTERACTION_POLL_MS)).await;

            let Some(current) = preview_state_snapshot() else {
                break;
            };
            if current.request_id != state.request_id || current.session_id != state.session_id {
                break;
            }

            let panel_rect = {
                let guard = PREVIEW_INTERACTION.lock().unwrap_or_else(|poisoned| {
                    log::error!("preview interaction mutex poisoned on track, recovering");
                    poisoned.into_inner()
                });
                if guard.request_id != state.request_id || guard.session_id != state.session_id {
                    break;
                }

                guard.panel_rect
            };
            let Some(panel_rect) = panel_rect else {
                continue;
            };

            let Ok(cursor) = window.cursor_position() else {
                continue;
            };
            let inside = preview_panel_contains_physical(&state, panel_rect, cursor.x, cursor.y);
            let transition = {
                let mut guard = PREVIEW_INTERACTION.lock().unwrap_or_else(|poisoned| {
                    log::error!("preview interaction mutex poisoned on transition, recovering");
                    poisoned.into_inner()
                });
                if guard.request_id != state.request_id || guard.session_id != state.session_id {
                    break;
                }
                if guard.active == inside {
                    None
                } else {
                    guard.active = inside;
                    Some(inside)
                }
            };
            let Some(active) = transition else {
                continue;
            };

            let payload = PreviewInteractionPayload {
                request_id: state.request_id,
                session_id: state.session_id,
                active,
            };
            if let Err(error) = app.emit(PREVIEW_INTERACTION_EVENT, payload) {
                log::warn!("emit {PREVIEW_INTERACTION_EVENT} failed: {error}");
            }
            if let Err(error) = apply_preview_input_capture(&app, &window, active) {
                log::warn!("apply preview input capture failed: {error}");
            }
        }
    });
}

fn preview_panel_contains_physical(
    state: &ClipboardPreviewState,
    panel_rect: PreviewRect,
    cursor_x: f64,
    cursor_y: f64,
) -> bool {
    let left = state.work_area.x as f64 + panel_rect.left * state.scale_factor;
    let top = state.work_area.y as f64 + panel_rect.top * state.scale_factor;
    let right = left + panel_rect.width * state.scale_factor;
    let bottom = top + panel_rect.height * state.scale_factor;

    cursor_x >= left && cursor_x < right && cursor_y >= top && cursor_y < bottom
}

fn apply_preview_input_capture(
    app: &AppHandle,
    window: &WebviewWindow,
    active: bool,
) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let _ = app;
        window
            .set_ignore_cursor_events(!active)
            .map_err(|error| anyhow::anyhow!(error))?;

        if active {
            activate_windows_preview_focus(window)?;
        } else {
            deactivate_windows_preview_focus(window);
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        let handle = app.clone();
        app.run_on_main_thread(move || {
            if let Ok(panel) = handle.get_webview_panel(CLIPBOARD_PREVIEW_WINDOW_LABEL) {
                panel.set_ignores_mouse_events(!active);
                if active {
                    panel.make_key_window();
                } else {
                    panel.resign_key_window();
                }
            }
        })
        .map_err(|error| anyhow::anyhow!(error))?;

        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn activate_windows_preview_focus(window: &WebviewWindow) -> Result<()> {
    let raw_hwnd = window.hwnd().map_err(|error| anyhow::anyhow!(error))?;
    let preview_hwnd = HWND(raw_hwnd.0 as isize);

    {
        let mut guard = PREVIEW_PREVIOUS_FOREGROUND
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if guard.is_none() {
            let foreground = unsafe { GetForegroundWindow() };
            if foreground.0 != 0 && foreground != preview_hwnd {
                *guard = Some(foreground.0);
            }
        }
    }

    window
        .set_focusable(true)
        .map_err(|error| anyhow::anyhow!(error))?;
    window.set_focus().map_err(|error| anyhow::anyhow!(error))?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn deactivate_windows_preview_focus(window: &WebviewWindow) {
    let Ok(raw_hwnd) = window.hwnd() else {
        clear_windows_preview_foreground();
        return;
    };
    let preview_hwnd = HWND(raw_hwnd.0 as isize);
    let should_restore = unsafe { GetForegroundWindow() } == preview_hwnd;

    if let Err(error) = window.set_focusable(false) {
        log::warn!("set preview window non-focusable failed: {error}");
    }

    if should_restore {
        restore_windows_preview_foreground(preview_hwnd);
    } else {
        clear_windows_preview_foreground();
    }
}

#[cfg(target_os = "windows")]
fn restore_windows_preview_foreground(preview_hwnd: HWND) {
    let previous = PREVIEW_PREVIOUS_FOREGROUND
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take();
    let Some(previous) = previous else {
        return;
    };

    let previous_hwnd = HWND(previous);
    if previous_hwnd == preview_hwnd || !unsafe { IsWindow(previous_hwnd).as_bool() } {
        return;
    }

    if !unsafe { SetForegroundWindow(previous_hwnd).as_bool() } {
        log::debug!("restore preview foreground window was rejected by Windows");
    }
}

#[cfg(target_os = "windows")]
fn clear_windows_preview_foreground() {
    PREVIEW_PREVIOUS_FOREGROUND
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take();
}

/// 判断剪贴板窗口是否仍处于可见状态，防止过期 hover 请求在剪贴板窗口隐藏后唤起预览。
fn is_clipboard_window_visible(app: &AppHandle) -> bool {
    app.get_webview_window(CLIPBOARD_WINDOW_LABEL)
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false)
}

/// 延迟隐藏真实窗口，为前端退出动画留出一小段可见时间。
fn schedule_preview_window_hide(app: AppHandle, window: WebviewWindow, request_id: u64) {
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(PREVIEW_HIDE_DELAY_MS));

        if !is_preview_request_current(request_id) {
            return;
        }
        if get_clipboard_preview_state().ok().flatten().is_some() {
            return;
        }

        if let Err(error) = hide_preview_window(&app, &window) {
            log::error!("hide preview window after exit animation failed: {error}");
        }
    });
}

/// 返回本次 show 所属的可见会话 id；隐藏后再次 show 会开启新会话。
fn preview_session_id_for_show() -> u64 {
    let guard = PREVIEW_STATE.lock().unwrap_or_else(|poisoned| {
        log::error!("preview state mutex poisoned on session, recovering");
        poisoned.into_inner()
    });

    if guard.is_some() {
        return PREVIEW_SESSION_ID.load(Ordering::SeqCst);
    }

    PREVIEW_SESSION_ID.fetch_add(1, Ordering::SeqCst) + 1
}

fn validate_anchor(anchor: &PreviewAnchorRect) -> Result<()> {
    let values = [anchor.left, anchor.top, anchor.width, anchor.height];
    if !values.iter().all(|value| value.is_finite()) || anchor.width <= 0.0 || anchor.height <= 0.0
    {
        return Err(anyhow::anyhow!("preview anchor is invalid").into());
    }

    Ok(())
}

fn validate_panel_rect(rect: &PreviewRect) -> Result<()> {
    let values = [rect.left, rect.top, rect.width, rect.height];
    if !values.iter().all(|value| value.is_finite()) || rect.width <= 0.0 || rect.height <= 0.0 {
        return Err(anyhow::anyhow!("preview panel rect is invalid").into());
    }

    Ok(())
}

/// 取预览窗口；已被空闲销毁（或尚未创建）时按需重建。
/// macOS 下每次都兜底确保 NSPanel 转换完成，覆盖重建后的全新窗口。
async fn ensure_preview_window(app: &AppHandle) -> Result<WebviewWindow> {
    if app
        .get_webview_window(CLIPBOARD_PREVIEW_WINDOW_LABEL)
        .is_none()
    {
        build_clipboard_preview_window(app)?;
    }

    let window = super::wait_for_webview_usable(app, CLIPBOARD_PREVIEW_WINDOW_LABEL).await?;

    #[cfg(target_os = "macos")]
    ensure_macos_preview_panel(app, &window)?;

    Ok(window)
}

fn raise_preview_window(app: &AppHandle, window: &WebviewWindow) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let _ = window;
        set_macos_preview_panel_level(app)
    }

    #[cfg(target_os = "windows")]
    {
        let _ = app;
        window
            .set_always_on_top(true)
            .map_err(|e| anyhow::anyhow!(e))?;
        raise_windows_preview_window(window, false)?;

        Ok(())
    }
}

fn prepare_preview_window_for_show(
    app: &AppHandle,
    window: &WebviewWindow,
    work_area: &PhysicalRect<i32, u32>,
) -> Result<()> {
    apply_preview_window_bounds(window, work_area)?;
    window
        .set_ignore_cursor_events(true)
        .map_err(|e| anyhow::anyhow!(e))?;
    raise_preview_window(app, window)
}

/// 平台 show 收口点；成功后推进生命周期到 `Visible`，使未触发的空闲销毁计时器过期。
fn show_preview_window(
    app: &AppHandle,
    window: &WebviewWindow,
    work_area: &PhysicalRect<i32, u32>,
) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let _ = window;
        show_macos_preview_panel(app, work_area)?;
    }

    #[cfg(target_os = "windows")]
    {
        let _ = work_area;
        window.show().map_err(|e| anyhow::anyhow!(e))?;
        raise_windows_preview_window(window, true)?;
    }

    lifecycle::on_shown(app, CLIPBOARD_PREVIEW_WINDOW_LABEL);

    Ok(())
}

/// 平台 hide 收口点；成功后推进生命周期到 `HiddenWarm`，启动空闲销毁计时。
/// 对已隐藏窗口的重复 hide（如剪贴板窗口隐藏时的压制路径）也会走到这里，
/// 由生命周期管理器对重复进入 `HiddenWarm` 去重计时。
fn hide_preview_window(app: &AppHandle, window: &WebviewWindow) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let _ = window;
        hide_macos_preview_panel(app)?;
    }

    #[cfg(target_os = "windows")]
    window.hide().map_err(|e| anyhow::anyhow!(e))?;

    lifecycle::on_hidden(app, CLIPBOARD_PREVIEW_WINDOW_LABEL, "preview-hide");

    Ok(())
}

#[cfg(target_os = "macos")]
fn ensure_macos_preview_panel(app: &AppHandle, window: &WebviewWindow) -> Result<()> {
    let handle = app.clone();
    let preview_window = window.clone();
    let (tx, rx) = std::sync::mpsc::channel();

    app.run_on_main_thread(move || {
        let result = setup_macos_preview_panel(&handle, &preview_window);
        let _ = tx.send(result);
    })
    .map_err(|e| anyhow::anyhow!(e))?;

    rx.recv()
        .map_err(|e| anyhow::anyhow!("preview panel setup channel closed: {e}"))?
}

#[cfg(target_os = "macos")]
fn setup_macos_preview_panel(app: &AppHandle, window: &WebviewWindow) -> Result<()> {
    let panel = match app.get_webview_panel(CLIPBOARD_PREVIEW_WINDOW_LABEL) {
        Ok(panel) => panel,
        Err(_) => window
            .to_panel::<PreviewPanel>()
            .map_err(|e| anyhow::anyhow!("to_panel failed: {e:?}"))?,
    };

    panel.set_level(PanelLevel::Status.value());
    panel.set_ignores_mouse_events(true);

    Ok(())
}

#[cfg(target_os = "macos")]
fn set_macos_preview_panel_level(app: &AppHandle) -> Result<()> {
    let handle = app.clone();

    app.run_on_main_thread(move || {
        if let Ok(panel) = handle.get_webview_panel(CLIPBOARD_PREVIEW_WINDOW_LABEL) {
            panel.set_level(PanelLevel::Status.value());
            panel.set_ignores_mouse_events(true);
        }
    })
    .map_err(|e| anyhow::anyhow!(e))?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn show_macos_preview_panel(app: &AppHandle, work_area: &PhysicalRect<i32, u32>) -> Result<()> {
    let handle = app.clone();
    let preview_window = get_window(app, CLIPBOARD_PREVIEW_WINDOW_LABEL)?;
    let work_area = *work_area;
    let (tx, rx) = std::sync::mpsc::channel();

    app.run_on_main_thread(move || {
        let result = (|| -> Result<()> {
            apply_preview_window_bounds(&preview_window, &work_area)?;
            let panel = handle
                .get_webview_panel(CLIPBOARD_PREVIEW_WINDOW_LABEL)
                .map_err(|e| anyhow::anyhow!("preview panel not found: {e:?}"))?;
            panel.set_ignores_mouse_events(true);
            panel.set_level(PanelLevel::Status.value());
            panel.show();
            Ok(())
        })();
        let _ = tx.send(result);
    })
    .map_err(|e| anyhow::anyhow!(e))?;

    rx.recv()
        .map_err(|e| anyhow::anyhow!("preview panel show channel closed: {e}"))?
}

#[cfg(target_os = "macos")]
fn hide_macos_preview_panel(app: &AppHandle) -> Result<()> {
    let handle = app.clone();

    app.run_on_main_thread(move || {
        if let Ok(panel) = handle.get_webview_panel(CLIPBOARD_PREVIEW_WINDOW_LABEL) {
            panel.hide();
        }
    })
    .map_err(|e| anyhow::anyhow!(e))?;

    Ok(())
}

/// 将预览窗口重新压到 Windows topmost 栈顶，避免被同为 always-on-top 的剪贴板窗口盖住。
#[cfg(target_os = "windows")]
fn raise_windows_preview_window(window: &WebviewWindow, show: bool) -> Result<()> {
    let raw_hwnd = window.hwnd().map_err(|e| anyhow::anyhow!(e))?;
    let hwnd = HWND(raw_hwnd.0 as isize);
    let mut flags = SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE;

    if show {
        flags |= SWP_SHOWWINDOW;
    }

    unsafe {
        SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, flags).map_err(|e| anyhow::anyhow!(e))?;
    }

    Ok(())
}

fn resolve_preview_monitor(app: &AppHandle) -> Result<tauri::Monitor> {
    let clipboard_window = get_window(app, CLIPBOARD_WINDOW_LABEL)?;
    if let Some(monitor) = clipboard_window
        .current_monitor()
        .map_err(|e| anyhow::anyhow!(e))?
    {
        return Ok(monitor);
    }

    clipboard_window
        .primary_monitor()
        .map_err(|e| anyhow::anyhow!(e))?
        .ok_or_else(|| anyhow::anyhow!("primary monitor not found").into())
}

/// 用完整显示器区域作为预览 overlay 边界，避免 macOS Dock 压缩 `work_area` 后截断连线。
fn preview_overlay_bounds(monitor: &tauri::Monitor) -> PhysicalRect<i32, u32> {
    PhysicalRect {
        position: *monitor.position(),
        size: *monitor.size(),
    }
}

fn apply_preview_window_bounds(
    window: &WebviewWindow,
    work_area: &PhysicalRect<i32, u32>,
) -> Result<()> {
    let size = preview_window_size(work_area);

    window
        .set_position(PhysicalPosition::new(
            work_area.position.x,
            work_area.position.y,
        ))
        .map_err(|e| anyhow::anyhow!(e))?;
    window.set_size(size).map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}

/// 返回实际预览窗口尺寸；Windows 避免精确全屏触发系统勿扰模式。
fn preview_window_size(work_area: &PhysicalRect<i32, u32>) -> PhysicalSize<u32> {
    #[cfg(target_os = "windows")]
    {
        PhysicalSize::new(
            work_area.size.width.saturating_sub(1),
            work_area.size.height,
        )
    }

    #[cfg(target_os = "macos")]
    {
        PhysicalSize::new(work_area.size.width, work_area.size.height)
    }
}

fn build_preview_layout(
    anchor: &PreviewAnchorRect,
    scale_factor: f64,
    work_area: &PhysicalRect<i32, u32>,
    clipboard_window: Option<&PreviewClipboardWindowRect>,
) -> PreviewLayout {
    let overlay_rect = PreviewRect {
        left: 0.0,
        top: 0.0,
        width: work_area.size.width as f64 / scale_factor,
        height: work_area.size.height as f64 / scale_factor,
    };
    let source_rect = resolve_source_rect(
        anchor,
        scale_factor,
        work_area,
        clipboard_window,
        overlay_rect,
    );
    let (panel_rect, placement) = resolve_panel_rect(source_rect, overlay_rect);

    PreviewLayout {
        overlay_rect,
        source_rect,
        panel_rect,
        placement,
    }
}

fn resolve_source_rect(
    anchor: &PreviewAnchorRect,
    scale_factor: f64,
    work_area: &PhysicalRect<i32, u32>,
    clipboard_window: Option<&PreviewClipboardWindowRect>,
    overlay_rect: PreviewRect,
) -> PreviewRect {
    let source = if let Some(clipboard_window) = clipboard_window {
        let main_rect = PreviewRect {
            left: (clipboard_window.x - work_area.position.x) as f64 / scale_factor,
            top: (clipboard_window.y - work_area.position.y) as f64 / scale_factor,
            width: clipboard_window.width as f64 / scale_factor,
            height: clipboard_window.height as f64 / scale_factor,
        };
        PreviewRect {
            left: main_rect.left + anchor.left,
            top: main_rect.top + anchor.top,
            width: anchor.width,
            height: anchor.height,
        }
    } else {
        PreviewRect {
            left: anchor.left,
            top: anchor.top,
            width: anchor.width,
            height: anchor.height,
        }
    };

    intersect_rect(source, overlay_rect).unwrap_or_else(|| clamp_rect(source, overlay_rect))
}

fn resolve_panel_rect(
    source_rect: PreviewRect,
    overlay_rect: PreviewRect,
) -> (PreviewRect, PreviewPlacement) {
    let panel_size = (PREVIEW_PANEL_WIDTH, PREVIEW_PANEL_HEIGHT);
    let candidates = [
        (
            PreviewPlacement::Right,
            source_rect.right() + PREVIEW_PANEL_GAP + PREVIEW_PANEL_WIDTH + PREVIEW_PANEL_MARGIN
                <= overlay_rect.right(),
        ),
        (
            PreviewPlacement::Left,
            source_rect.left - PREVIEW_PANEL_GAP - PREVIEW_PANEL_WIDTH - PREVIEW_PANEL_MARGIN
                >= overlay_rect.left,
        ),
        (
            PreviewPlacement::Bottom,
            source_rect.bottom() + PREVIEW_PANEL_GAP + PREVIEW_PANEL_HEIGHT + PREVIEW_PANEL_MARGIN
                <= overlay_rect.bottom(),
        ),
        (
            PreviewPlacement::Top,
            source_rect.top - PREVIEW_PANEL_GAP - PREVIEW_PANEL_HEIGHT - PREVIEW_PANEL_MARGIN
                >= overlay_rect.top,
        ),
    ];

    let placement = candidates
        .iter()
        .find_map(|(placement, fits)| fits.then_some(*placement))
        .unwrap_or(PreviewPlacement::Right);
    let raw = raw_panel_rect(source_rect, placement, panel_size);

    (
        clamp_rect(raw, inset_rect(overlay_rect, PREVIEW_PANEL_MARGIN)),
        placement,
    )
}

fn raw_panel_rect(
    source_rect: PreviewRect,
    placement: PreviewPlacement,
    (width, height): (f64, f64),
) -> PreviewRect {
    let centered_top = source_rect.center_y() - height / 2.0;
    let centered_left = source_rect.center_x() - width / 2.0;

    match placement {
        PreviewPlacement::Right => PreviewRect {
            left: source_rect.right() + PREVIEW_PANEL_GAP,
            top: centered_top,
            width,
            height,
        },
        PreviewPlacement::Left => PreviewRect {
            left: source_rect.left - PREVIEW_PANEL_GAP - width,
            top: centered_top,
            width,
            height,
        },
        PreviewPlacement::Bottom => PreviewRect {
            left: centered_left,
            top: source_rect.bottom() + PREVIEW_PANEL_GAP,
            width,
            height,
        },
        PreviewPlacement::Top => PreviewRect {
            left: centered_left,
            top: source_rect.top - PREVIEW_PANEL_GAP - height,
            width,
            height,
        },
    }
}

fn clamp_rect(rect: PreviewRect, bounds: PreviewRect) -> PreviewRect {
    let max_left = (bounds.right() - rect.width).max(bounds.left);
    let max_top = (bounds.bottom() - rect.height).max(bounds.top);

    PreviewRect {
        left: rect.left.clamp(bounds.left, max_left),
        top: rect.top.clamp(bounds.top, max_top),
        width: rect.width,
        height: rect.height,
    }
}

fn intersect_rect(a: PreviewRect, b: PreviewRect) -> Option<PreviewRect> {
    let left = a.left.max(b.left);
    let top = a.top.max(b.top);
    let right = a.right().min(b.right());
    let bottom = a.bottom().min(b.bottom());

    if right <= left || bottom <= top {
        return None;
    }

    Some(PreviewRect {
        left,
        top,
        width: right - left,
        height: bottom - top,
    })
}

fn inset_rect(rect: PreviewRect, amount: f64) -> PreviewRect {
    PreviewRect {
        left: rect.left + amount,
        top: rect.top + amount,
        width: (rect.width - amount * 2.0).max(1.0),
        height: (rect.height - amount * 2.0).max(1.0),
    }
}

/// 返回剪贴板窗口内容区的屏幕几何，用于映射 WebView DOM rect 到预览 overlay 坐标。
fn clipboard_window_rect(app: &AppHandle) -> Option<PreviewClipboardWindowRect> {
    let window = app.get_webview_window(CLIPBOARD_WINDOW_LABEL)?;
    let pos = window.inner_position().ok()?;
    let size = window.inner_size().ok()?;

    Some(PreviewClipboardWindowRect {
        x: pos.x,
        y: pos.y,
        width: size.width,
        height: size.height,
    })
}

impl PreviewRect {
    fn right(self) -> f64 {
        self.left + self.width
    }

    fn bottom(self) -> f64 {
        self.top + self.height
    }

    fn center_x(self) -> f64 {
        self.left + self.width / 2.0
    }

    fn center_y(self) -> f64 {
        self.top + self.height / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overlay() -> PreviewRect {
        PreviewRect {
            left: 0.0,
            top: 0.0,
            width: 1200.0,
            height: 800.0,
        }
    }

    #[test]
    fn places_panel_on_right_when_space_exists() {
        let source = PreviewRect {
            left: 240.0,
            top: 200.0,
            width: 120.0,
            height: 40.0,
        };
        let (panel, placement) = resolve_panel_rect(source, overlay());

        assert!(matches!(placement, PreviewPlacement::Right));
        assert!(panel.left > source.right());
    }

    #[test]
    fn places_panel_on_left_when_right_side_is_tight() {
        let source = PreviewRect {
            left: 980.0,
            top: 200.0,
            width: 120.0,
            height: 40.0,
        };
        let (panel, placement) = resolve_panel_rect(source, overlay());

        assert!(matches!(placement, PreviewPlacement::Left));
        assert!(panel.right() < source.left);
    }

    #[test]
    fn clamps_panel_inside_overlay_margin() {
        let source = PreviewRect {
            left: 580.0,
            top: 740.0,
            width: 80.0,
            height: 40.0,
        };
        let (panel, _) = resolve_panel_rect(source, overlay());

        assert!(panel.left >= PREVIEW_PANEL_MARGIN);
        assert!(panel.top >= PREVIEW_PANEL_MARGIN);
        assert!(panel.right() <= overlay().right() - PREVIEW_PANEL_MARGIN);
        assert!(panel.bottom() <= overlay().bottom() - PREVIEW_PANEL_MARGIN);
    }

    #[test]
    fn maps_anchor_from_clipboard_window_to_overlay_local_rect() {
        let work_area = PhysicalRect {
            position: PhysicalPosition::new(100, 50),
            size: PhysicalSize::new(2400, 1600),
        };
        let clipboard = PreviewClipboardWindowRect {
            x: 300,
            y: 250,
            width: 800,
            height: 600,
        };
        let anchor = PreviewAnchorRect {
            left: 20.0,
            top: 30.0,
            width: 100.0,
            height: 40.0,
        };
        let source = resolve_source_rect(
            &anchor,
            2.0,
            &work_area,
            Some(&clipboard),
            PreviewRect {
                left: 0.0,
                top: 0.0,
                width: 1200.0,
                height: 800.0,
            },
        );

        assert_eq!(source.left, 120.0);
        assert_eq!(source.top, 130.0);
        assert_eq!(source.width, 100.0);
        assert_eq!(source.height, 40.0);
    }

    #[test]
    fn keeps_source_rect_on_clipboard_card_when_card_touches_main_edge() {
        let work_area = PhysicalRect {
            position: PhysicalPosition::new(100, 50),
            size: PhysicalSize::new(2400, 1600),
        };
        let clipboard = PreviewClipboardWindowRect {
            x: 300,
            y: 250,
            width: 800,
            height: 600,
        };
        let anchor = PreviewAnchorRect {
            left: -8.0,
            top: 40.0,
            width: 120.0,
            height: 80.0,
        };
        let source = resolve_source_rect(
            &anchor,
            2.0,
            &work_area,
            Some(&clipboard),
            PreviewRect {
                left: 0.0,
                top: 0.0,
                width: 1200.0,
                height: 800.0,
            },
        );

        assert_eq!(source.left, 92.0);
        assert_eq!(source.width, 120.0);
    }

    #[test]
    fn keeps_clipboard_card_vertical_center() {
        let work_area = PhysicalRect {
            position: PhysicalPosition::new(100, 50),
            size: PhysicalSize::new(2400, 1600),
        };
        let clipboard = PreviewClipboardWindowRect {
            x: 300,
            y: 250,
            width: 800,
            height: 600,
        };
        let anchor = PreviewAnchorRect {
            left: 20.0,
            top: 40.0,
            width: 120.0,
            height: 100.0,
        };
        let source = resolve_source_rect(
            &anchor,
            2.0,
            &work_area,
            Some(&clipboard),
            PreviewRect {
                left: 0.0,
                top: 0.0,
                width: 1200.0,
                height: 800.0,
            },
        );

        assert_eq!(source.left, 120.0);
        assert_eq!(source.top, 140.0);
        assert_eq!(source.width, 120.0);
        assert_eq!(source.height, 100.0);
        assert_eq!(source.center_y(), 190.0);
    }

    #[test]
    fn preview_request_ids_must_strictly_increase() {
        assert!(should_accept_preview_request(0, 1));
        assert!(should_accept_preview_request(1, 2));
        assert!(!should_accept_preview_request(2, 2));
        assert!(!should_accept_preview_request(3, 2));
    }

    #[test]
    fn preview_panel_hit_test_uses_physical_work_area_offset() {
        let state = ClipboardPreviewState {
            request_id: 1,
            session_id: 1,
            item_id: "item".to_owned(),
            anchor: PreviewAnchorRect {
                left: 0.0,
                top: 0.0,
                width: 100.0,
                height: 40.0,
            },
            scale_factor: 2.0,
            work_area: PreviewWorkArea {
                x: 100,
                y: 50,
                width: 2400,
                height: 1600,
            },
            clipboard_window: None,
            layout: PreviewLayout {
                overlay_rect: overlay(),
                source_rect: PreviewRect {
                    left: 0.0,
                    top: 0.0,
                    width: 100.0,
                    height: 40.0,
                },
                panel_rect: overlay(),
                placement: PreviewPlacement::Right,
            },
        };
        let panel = PreviewRect {
            left: 20.0,
            top: 30.0,
            width: 100.0,
            height: 80.0,
        };

        assert!(preview_panel_contains_physical(&state, panel, 140.0, 110.0));
        assert!(!preview_panel_contains_physical(
            &state, panel, 340.0, 110.0
        ));
        assert!(!preview_panel_contains_physical(
            &state, panel, 140.0, 270.0
        ));
    }
}
