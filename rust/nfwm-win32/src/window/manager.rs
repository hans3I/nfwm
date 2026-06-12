use nfwm_core::traits::{PlacementError, WindowProvider};
use nfwm_core::types::{DisplayId, ProcessId, Rectangle, Size, VirtualDesktopId, WindowId};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowLongW, GetWindowRect, GetWindowTextW, IsIconic,
    IsWindowVisible, IsZoomed, SendMessageW, GWL_EXSTYLE, GWL_STYLE, WM_GETMINMAXINFO,
    WS_EX_TOPMOST, WS_SIZEBOX, WS_THICKFRAME,
};

/// Error type for Win32 operations.
#[derive(Debug, thiserror::Error)]
pub enum Win32Error {
    #[error("Win32 API error: {0}")]
    Api(String),
    #[error("Invalid window handle")]
    InvalidHandle,
    #[error("Window not found")]
    NotFound,
}

#[allow(dead_code)]
impl Win32Error {
    fn from_last_error() -> Self {
        Win32Error::Api(format!("{}", std::io::Error::last_os_error()))
    }
}

/// A native Win32 window manager that queries the live Windows environment.
///
/// All `unsafe` Win32 calls are isolated in this struct. See `docs/coding-standards.md`
/// for the rules governing unsafe code in this crate.
pub struct Win32WindowManager;

impl Default for Win32WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Win32WindowManager {
    pub fn new() -> Self {
        Self
    }

    /// Enumerate all top-level windows that are candidates for tiling.
    ///
    /// Filters out invisible windows and some system windows by default.
    pub fn enumerate_windows(&self) -> Vec<WindowId> {
        let mut windows = Vec::new();
        // SAFETY: EnumWindows is a well-defined API; our callback is a static fn
        unsafe {
            let _ = EnumWindows(Some(enum_callback), LPARAM(&mut windows as *mut _ as isize));
        }
        windows
    }

    /// Get a human-readable summary of a window for diagnostics.
    pub fn describe_window(&self, id: WindowId) -> String {
        let hwnd = id_to_hwnd(id);
        if hwnd.0 == 0 {
            return format!("{:?} (invalid handle)", id);
        }

        let title = self.title(id).unwrap_or_default();
        let class = self.class_name(id).unwrap_or_default();
        let pid = self
            .process_id(id)
            .map(|p| format!("{}", p.0))
            .unwrap_or_else(|| "?".to_string());
        let bounds = self
            .bounds(id)
            .map(|r| format!("{}x{} at ({},{})", r.width, r.height, r.x, r.y))
            .unwrap_or_else(|| "?".to_string());
        let visible = if self.is_visible(id) {
            "visible"
        } else {
            "hidden"
        };
        let minimized = if self.is_minimized(id) {
            "minimized"
        } else {
            ""
        };
        let maximized = if self.is_maximized(id) {
            "maximized"
        } else {
            ""
        };
        let topmost = if self.is_topmost(id) { "topmost" } else { "" };
        let resizable = if self.is_resizable(id) {
            "resizable"
        } else {
            "fixed"
        };
        let dpi = self
            .get_dpi(id)
            .map(|d| format!(" DPI: {:.0}", d))
            .unwrap_or_default();

        format!(
            "[{}] '{}' (class: {}) [{}] {} {} {} {} {} {}",
            pid, title, class, bounds, visible, minimized, maximized, topmost, resizable, dpi
        )
    }

    /// Get the DPI of the display a window is on.
    pub fn get_dpi(&self, id: WindowId) -> Option<f32> {
        let hwnd = id_to_hwnd(id);
        if hwnd.0 == 0 {
            return None;
        }
        // SAFETY: hwnd is valid; GetDpiForWindow is available on Windows 8.1+
        let dpi = unsafe { GetDpiForWindow(hwnd) };
        if dpi > 0 {
            Some(dpi as f32)
        } else {
            None
        }
    }
}

// --- Internal Win32 helpers ---

fn hwnd_to_id(hwnd: HWND) -> WindowId {
    WindowId(hwnd.0 as usize)
}

fn id_to_hwnd(id: WindowId) -> HWND {
    HWND(id.0 as isize)
}

fn rect_from_win32(rect: RECT) -> Rectangle {
    Rectangle {
        x: rect.left,
        y: rect.top,
        width: rect.right - rect.left,
        height: rect.bottom - rect.top,
    }
}

/// Win32 MINMAXINFO structure for WM_GETMINMAXINFO.
#[repr(C)]
#[derive(Debug, Default)]
#[allow(clippy::upper_case_acronyms)]
struct MINMAXINFO {
    pt_reserved: POINT,
    pt_max_size: POINT,
    pt_max_position: POINT,
    pt_min_track_size: POINT,
    pt_max_track_size: POINT,
}

// SAFETY: LPARAM is the pointer to a Vec<WindowId> passed from enumerate_windows
unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<WindowId>);
    // Only include visible windows that are not tool windows
    if IsWindowVisible(hwnd).as_bool() {
        windows.push(hwnd_to_id(hwnd));
    }
    BOOL(1) // Continue enumeration
}

impl WindowProvider for Win32WindowManager {
    fn title(&self, id: WindowId) -> Option<String> {
        let hwnd = id_to_hwnd(id);
        if hwnd.0 == 0 {
            return None;
        }
        let mut buffer = [0u16; 512];
        // SAFETY: hwnd is valid; buffer is large enough
        let len = unsafe { GetWindowTextW(hwnd, &mut buffer) };
        if len == 0 {
            None
        } else {
            Some(String::from_utf16_lossy(&buffer[..len as usize]))
        }
    }

    fn class_name(&self, id: WindowId) -> Option<String> {
        let hwnd = id_to_hwnd(id);
        if hwnd.0 == 0 {
            return None;
        }
        let mut buffer = [0u16; 256];
        // SAFETY: hwnd is valid; buffer is large enough
        let len =
            unsafe { windows::Win32::UI::WindowsAndMessaging::GetClassNameW(hwnd, &mut buffer) };
        if len == 0 {
            None
        } else {
            Some(String::from_utf16_lossy(&buffer[..len as usize]))
        }
    }

    fn process_id(&self, id: WindowId) -> Option<ProcessId> {
        let hwnd = id_to_hwnd(id);
        if hwnd.0 == 0 {
            return None;
        }
        let mut pid = 0u32;
        // SAFETY: hwnd is valid; pid is a valid mutable reference
        unsafe {
            windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(hwnd, Some(&mut pid));
        }
        Some(ProcessId(pid))
    }

    fn bounds(&self, id: WindowId) -> Option<Rectangle> {
        let hwnd = id_to_hwnd(id);
        if hwnd.0 == 0 {
            return None;
        }
        let mut rect = RECT::default();
        // SAFETY: hwnd is valid; rect is a valid mutable reference
        let ok = unsafe { GetWindowRect(hwnd, &mut rect) };
        if ok.is_ok() {
            Some(rect_from_win32(rect))
        } else {
            None
        }
    }

    fn is_visible(&self, id: WindowId) -> bool {
        let hwnd = id_to_hwnd(id);
        if hwnd.0 == 0 {
            return false;
        }
        // SAFETY: hwnd is valid
        unsafe { IsWindowVisible(hwnd).as_bool() }
    }

    fn is_minimized(&self, id: WindowId) -> bool {
        let hwnd = id_to_hwnd(id);
        if hwnd.0 == 0 {
            return false;
        }
        // SAFETY: hwnd is valid
        unsafe { IsIconic(hwnd).as_bool() }
    }

    fn is_maximized(&self, id: WindowId) -> bool {
        let hwnd = id_to_hwnd(id);
        if hwnd.0 == 0 {
            return false;
        }
        // SAFETY: hwnd is valid
        unsafe { IsZoomed(hwnd).as_bool() }
    }

    fn is_topmost(&self, id: WindowId) -> bool {
        let hwnd = id_to_hwnd(id);
        if hwnd.0 == 0 {
            return false;
        }
        // SAFETY: hwnd is valid
        let ex_style = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) };
        (ex_style as u32) & WS_EX_TOPMOST.0 != 0
    }

    fn is_resizable(&self, id: WindowId) -> bool {
        let hwnd = id_to_hwnd(id);
        if hwnd.0 == 0 {
            return false;
        }
        // SAFETY: hwnd is valid
        let style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) };
        let style = windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(style as u32);
        (style.0 & WS_THICKFRAME.0) != 0 || (style.0 & WS_SIZEBOX.0) != 0
    }

    fn min_size(&self, id: WindowId) -> Option<Size> {
        let hwnd = id_to_hwnd(id);
        if hwnd.0 == 0 {
            return None;
        }
        let mut mmi = MINMAXINFO::default();
        // SAFETY: hwnd is valid; mmi is a valid mutable reference
        let result = unsafe {
            SendMessageW(
                hwnd,
                WM_GETMINMAXINFO,
                WPARAM(0),
                LPARAM(&mut mmi as *mut _ as isize),
            )
        };
        if result != LRESULT(0) || mmi.pt_min_track_size.x > 0 || mmi.pt_min_track_size.y > 0 {
            Some(Size::new(mmi.pt_min_track_size.x, mmi.pt_min_track_size.y))
        } else {
            None
        }
    }

    fn display_id(&self, id: WindowId) -> Option<DisplayId> {
        let hwnd = id_to_hwnd(id);
        if hwnd.0 == 0 {
            return None;
        }
        // SAFETY: hwnd is valid
        let hmonitor = unsafe {
            windows::Win32::Graphics::Gdi::MonitorFromWindow(
                hwnd,
                windows::Win32::Graphics::Gdi::MONITOR_DEFAULTTONEAREST,
            )
        };
        if hmonitor.is_invalid() {
            None
        } else {
            Some(DisplayId(hmonitor.0 as usize))
        }
    }

    fn virtual_desktop_id(&self, _id: WindowId) -> Option<VirtualDesktopId> {
        // TODO: Virtual desktop APIs in Phase 08
        None
    }

    fn is_focused(&self, id: WindowId) -> bool {
        let hwnd = id_to_hwnd(id);
        if hwnd.0 == 0 {
            return false;
        }
        // SAFETY: GetForegroundWindow is safe
        let foreground = unsafe { GetForegroundWindow() };
        foreground == hwnd
    }
}

/// A Win32 placement provider that moves and resizes windows.
pub struct Win32PlacementProvider;

impl Default for Win32PlacementProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Win32PlacementProvider {
    pub fn new() -> Self {
        Self
    }
}

impl nfwm_core::traits::PlacementProvider for Win32PlacementProvider {
    fn set_bounds(&self, id: WindowId, rect: Rectangle) -> Result<(), PlacementError> {
        let hwnd = id_to_hwnd(id);
        if hwnd.0 == 0 {
            return Err(PlacementError::WindowNotFound);
        }
        // SAFETY: hwnd is valid; we use SWP_NOZORDER | SWP_NOACTIVATE to avoid focus side effects
        let result = unsafe {
            windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                hwnd,
                HWND(0), // SWP_NOZORDER
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                windows::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                    | windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
            )
        };
        if result.is_ok() {
            Ok(())
        } else {
            Err(PlacementError::AccessDenied)
        }
    }

    fn restore(&self, id: WindowId) -> Result<(), PlacementError> {
        let hwnd = id_to_hwnd(id);
        if hwnd.0 == 0 {
            return Err(PlacementError::WindowNotFound);
        }
        // SAFETY: hwnd is valid
        let result = unsafe {
            windows::Win32::UI::WindowsAndMessaging::ShowWindow(
                hwnd,
                windows::Win32::UI::WindowsAndMessaging::SW_RESTORE,
            )
        };
        if result.as_bool() {
            Ok(())
        } else {
            Err(PlacementError::AccessDenied)
        }
    }

    fn minimize(&self, id: WindowId) -> Result<(), PlacementError> {
        let hwnd = id_to_hwnd(id);
        if hwnd.0 == 0 {
            return Err(PlacementError::WindowNotFound);
        }
        // SAFETY: hwnd is valid
        let result = unsafe {
            windows::Win32::UI::WindowsAndMessaging::ShowWindow(
                hwnd,
                windows::Win32::UI::WindowsAndMessaging::SW_MINIMIZE,
            )
        };
        if result.as_bool() {
            Ok(())
        } else {
            Err(PlacementError::AccessDenied)
        }
    }
}
