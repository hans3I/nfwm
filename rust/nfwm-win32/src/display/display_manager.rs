//! Display manager: enumerate and track displays.

use nfwm_core::traits::DisplayProvider;
use nfwm_core::types::{DisplayId, Rectangle};
use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HMONITOR, MONITORINFO};

// Raw Win32 import for EnumDisplaySettingsW.
extern "system" {
    fn EnumDisplaySettingsW(
        lpszdevicename: *const u16,
        imodenum: u32,
        lpdevmode: *mut DEVMODEW,
    ) -> BOOL;
}

#[repr(C)]
#[derive(Debug, Default)]
#[allow(clippy::upper_case_acronyms, non_snake_case)]
struct DEVMODEW {
    dmDeviceName: [u16; 32],
    dmSpecVersion: u16,
    dmDriverVersion: u16,
    dmSize: u16,
    dmDriverExtra: u16,
    dmFields: u32,
    // Union: dmOrientation, dmPaperSize, dmPaperLength, dmPaperWidth
    dmOrientation: i16,
    dmPaperSize: i16,
    dmPaperLength: i16,
    dmPaperWidth: i16,
    dmScale: i16,
    dmCopies: i16,
    dmDefaultSource: i16,
    dmPrintQuality: i16,
    dmColor: i16,
    dmDuplex: i16,
    dmYResolution: i16,
    dmTTOption: i16,
    dmCollate: i16,
    dmFormName: [u16; 32],
    dmLogPixels: u16,
    dmBitsPerPel: u32,
    dmPelsWidth: u32,
    dmPelsHeight: u32,
    dmDisplayFlags: u32,
    dmDisplayFrequency: u32,
    dmICMMethod: u32,
    dmICMIntent: u32,
    dmMediaType: u32,
    dmDitherType: u32,
    dmReserved1: u32,
    dmReserved2: u32,
    dmPanningWidth: u32,
    dmPanningHeight: u32,
}

/// A native Win32 display manager that queries connected monitors.
pub struct Win32DisplayManager;

impl Default for Win32DisplayManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Win32DisplayManager {
    pub fn new() -> Self {
        Self
    }

    /// Enumerate all monitors and return their info.
    pub fn enumerate_monitors(&self) -> Vec<MonitorInfo> {
        let mut monitors = Vec::new();
        // SAFETY: EnumDisplayMonitors is a well-defined API
        unsafe {
            EnumDisplayMonitors(
                None,
                None,
                Some(monitor_enum_callback),
                LPARAM(&mut monitors as *mut _ as isize),
            );
        }
        monitors
    }
}

/// A lightweight snapshot of a monitor for diagnostics.
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub id: DisplayId,
    pub bounds: Rectangle,
    pub work_area: Rectangle,
    pub is_primary: bool,
}

fn rect_from_win32(rect: RECT) -> Rectangle {
    Rectangle {
        x: rect.left,
        y: rect.top,
        width: rect.right - rect.left,
        height: rect.bottom - rect.top,
    }
}

// SAFETY: LPARAM is a pointer to a Vec<MonitorInfo>
unsafe extern "system" fn monitor_enum_callback(
    _hmonitor: HMONITOR,
    _hdc: windows::Win32::Graphics::Gdi::HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let monitors = &mut *(lparam.0 as *mut Vec<MonitorInfo>);
    let hmonitor = _hmonitor;

    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: MONITORINFO is properly initialized
    if GetMonitorInfoW(hmonitor, &mut info).as_bool() {
        monitors.push(MonitorInfo {
            id: DisplayId(hmonitor.0 as usize),
            bounds: rect_from_win32(info.rcMonitor),
            work_area: rect_from_win32(info.rcWork),
            is_primary: (info.dwFlags & 1) != 0, // MONITORINFOF_PRIMARY
        });
    }
    BOOL(1) // Continue enumeration
}

impl DisplayProvider for Win32DisplayManager {
    fn all_displays(&self) -> Vec<DisplayId> {
        self.enumerate_monitors()
            .into_iter()
            .map(|m| m.id)
            .collect()
    }

    fn primary_display(&self) -> DisplayId {
        self.enumerate_monitors()
            .into_iter()
            .find(|m| m.is_primary)
            .map(|m| m.id)
            .unwrap_or_else(|| DisplayId(0))
    }

    fn bounds(&self, id: DisplayId) -> Option<Rectangle> {
        self.enumerate_monitors()
            .into_iter()
            .find(|m| m.id == id)
            .map(|m| m.bounds)
    }

    fn work_area(&self, id: DisplayId) -> Option<Rectangle> {
        self.enumerate_monitors()
            .into_iter()
            .find(|m| m.id == id)
            .map(|m| m.work_area)
    }

    fn dpi(&self, id: DisplayId) -> Option<f32> {
        let hmonitor = HMONITOR(id.0 as isize);
        let mut dpi_x = 0u32;
        let mut dpi_y = 0u32;
        // SAFETY: hmonitor is valid (from our enumeration)
        let result = unsafe {
            windows::Win32::UI::HiDpi::GetDpiForMonitor(
                hmonitor,
                windows::Win32::UI::HiDpi::MDT_EFFECTIVE_DPI,
                &mut dpi_x,
                &mut dpi_y,
            )
        };
        if result.is_ok() {
            Some(dpi_x as f32)
        } else {
            None
        }
    }

    fn refresh_rate(&self, _id: DisplayId) -> Option<f32> {
        // Query current display settings for the primary display
        let mut devmode = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };
        // SAFETY: EnumDisplaySettingsW is a well-defined API; devmode is properly sized
        let result = unsafe { EnumDisplaySettingsW(std::ptr::null(), 0xFFFFFFFF, &mut devmode) };
        if result.as_bool() && devmode.dmDisplayFrequency > 0 {
            Some(devmode.dmDisplayFrequency as f32)
        } else {
            None
        }
    }
}
