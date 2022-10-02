use std::ops::BitAnd;

use eframe::epaint::Pos2;
use focus_window::Window;
use windows::{
    Win32::Foundation::{BOOL, HWND, LPARAM},
    Win32::UI::WindowsAndMessaging::{
        GetAncestor, GetLastActivePopup, GetSystemMetrics, GetWindowLongPtrW, GetWindowTextW,
        IsWindow, IsWindowVisible, GA_ROOTOWNER, GWL_EXSTYLE, SM_CXSCREEN, SM_CYSCREEN,
        WINDOW_EX_STYLE, WS_EX_TOOLWINDOW,
    },
};

#[no_mangle]
#[used]
pub static mut WINDOW_LIST: Vec<Window> = Vec::new();

pub extern "system" fn enum_window(window: HWND, _: LPARAM) -> BOOL {
    unsafe {
        let mut text: [u16; 512] = [0; 512];
        if is_alt_tab_window(window) && // not sure if this is needed anymore
            IsWindow(window).as_bool() && 
            IsWindowVisible(window).as_bool()
        {
            let len = GetWindowTextW(window, &mut text);
            let text = String::from_utf16_lossy(&text[..len as usize]);

            if !text.is_empty() && !ignore_window(window) {
                WINDOW_LIST.push(Window::new(text, window));
            }
        }

        true.into()
    }
}

pub fn is_alt_tab_window(hwnd: HWND) -> bool {
    unsafe {
        let mut hwnd_walk = GetAncestor(hwnd, GA_ROOTOWNER);
        let mut hwnd_try = GetLastActivePopup(hwnd);
        while hwnd_walk != hwnd_try {
            if IsWindowVisible(hwnd_try).as_bool() {
                break;
            }
            hwnd_walk = hwnd_try;
            hwnd_try = GetLastActivePopup(hwnd_try);
        }

        hwnd_walk == hwnd
    }
}

pub fn get_resolution() -> Pos2 {
    unsafe {
        let height = GetSystemMetrics(SM_CYSCREEN);
        let width = GetSystemMetrics(SM_CXSCREEN);

        Pos2 {
            x: width as f32,
            y: height as f32,
        }
    }
}

pub fn ignore_window(window: HWND) -> bool {
    unsafe {
        let ptr = WINDOW_EX_STYLE(GetWindowLongPtrW(window, GWL_EXSTYLE) as u32);

        // add more when I spot them
        if WS_EX_TOOLWINDOW.bitand(ptr) == WS_EX_TOOLWINDOW {
            return true;
        }

        false
    }
}
