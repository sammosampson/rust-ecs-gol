use std::ptr;

use super::app::GLOBAL_APPLICATION;
use super::*;
use crate::prelude::*;

use winapi:: {
    shared::{
        windef::*,
        minwindef::*
    },
    um:: {
        winuser::*,
        wingdi::*, 
    }
};

pub unsafe extern "system" fn main_window_callback(window: HWND, message: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let mut result = LRESULT::default();
    if let Some(app) = &mut GLOBAL_APPLICATION {
        match message {
            WM_SETCURSOR => {
                if app.debug_show_cursor {
                    result = DefWindowProcA(window, message, wparam, lparam);
                } else {
                    SetCursor(ptr::null_mut());
                }
            }
            WM_ACTIVATEAPP => {
                if cfg!(feature = "0") {
                    if wparam == TRUE.try_into().unwrap() {
                        SetLayeredWindowAttributes(window, RGB(0, 0, 0), 255, LWA_ALPHA);
                    } else {
                        SetLayeredWindowAttributes(window, RGB(0, 0, 0), 64, LWA_ALPHA);
                    }
                }
            },
            WM_DESTROY => app.running = false,
            WM_CLOSE => app.running = false,
            WM_PAINT => {
                let mut paint = PAINTSTRUCT::default();
                let device_context = BeginPaint(window, &mut paint);
                let dimension = Win32WindowDimension::from(window);
                app.update_window(device_context, dimension.width, dimension.height);
                EndPaint(window, &paint);
            },
            WM_SYSKEYDOWN| WM_SYSKEYUP| WM_KEYDOWN | WM_KEYUP => {
                gol_assert!(1 != 1);
            },
            _ => result = DefWindowProcA(window, message, wparam, lparam)
        }
    }
    result
}

pub struct Win32WindowDimension {
    pub width: i32,
    pub height: i32
}

impl From<HWND> for Win32WindowDimension {
    fn from(from: HWND) -> Self {
        let mut client_rect = RECT::default();

        unsafe {
            GetClientRect(from, &mut client_rect);
        }

        Self {
            width: client_rect.right - client_rect.left,
            height: client_rect.bottom - client_rect.top
        }
    }
}

impl App {
    pub fn toggle_fullscreen(&mut self, window: HWND) {
        unsafe {
            let style = GetWindowLongA(window, GWL_STYLE) as u32;
            if style & WS_OVERLAPPEDWINDOW > 0 {
                let mut monitor_info = MONITORINFO::default();
                let window_placement_ok = GetWindowPlacement(window, &mut self.window_placement) > 0;
                let monitor = MonitorFromWindow(window, MONITOR_DEFAULTTOPRIMARY);
                let monitor_ok = GetMonitorInfoW(monitor, &mut monitor_info) > 0;
               
                if window_placement_ok && monitor_ok {
                    SetWindowLongA(window, GWL_STYLE, (style & !WS_OVERLAPPEDWINDOW) as i32);
                    SetWindowPos(
                        window, 
                        HWND_TOP,
                        monitor_info.rcMonitor.left,
                        monitor_info.rcMonitor.top,
                        monitor_info.rcMonitor.right - monitor_info.rcMonitor.left,
                        monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top,
                        SWP_NOOWNERZORDER | SWP_FRAMECHANGED
                    );
                }
            } else {
                SetWindowLongA(window, GWL_STYLE, (style | WS_OVERLAPPEDWINDOW) as i32);
                SetWindowPlacement(window, &mut self.window_placement);
                SetWindowPos(
                    window, 
                    ptr::null_mut(),
                     0,
                     0,
                     0,
                     0,
                     SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER |SWP_NOOWNERZORDER | SWP_FRAMECHANGED
                );
            }
        }
}
    pub unsafe fn update_window(
        &self,
        device_context: HDC,
        window_width: i32,
        window_height: i32
    ) {
        if window_width as u32 >= self.buffer.width * 2 && window_height as u32 >= self.buffer.height * 2 {
            StretchDIBits(
                device_context,
                0, 0, (self.buffer.width * 2) as i32, (self.buffer.height * 2) as i32,
                0, 0, self.buffer.width as i32, self.buffer.height as i32,
                self.buffer.memory,
                &self.buffer.info,
                DIB_RGB_COLORS, 
                SRCCOPY
            );
        } else {
            let offset_x = 10;
            let offset_y = 10;

            PatBlt(device_context, 0, 0, window_width, offset_y, BLACKNESS);
            PatBlt(device_context, 0, offset_y + self.buffer.height as i32, window_width, window_height, BLACKNESS);
            PatBlt(device_context, 0, 0, offset_x, window_height, BLACKNESS);
            PatBlt(device_context, offset_x + self.buffer.width as i32, 0, window_width, window_height, BLACKNESS);

            StretchDIBits(
                device_context,
                offset_x, offset_y, self.buffer.width as i32, self.buffer.height as i32,
                0, 0, self.buffer.width as i32, self.buffer.height as i32,
                self.buffer.memory,
                &self.buffer.info,
                DIB_RGB_COLORS, 
                SRCCOPY
            );

        }
    }
}

