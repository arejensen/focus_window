use std::{ops::BitAnd, path::PathBuf};

use focus_window::Window;
use windows::{
    Win32::Foundation::{BOOL, HWND, LPARAM},
    Win32::System::ProcessStatus::K32GetProcessImageFileNameA,
    Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION},
    Win32::UI::WindowsAndMessaging::{
        GetAncestor, GetLastActivePopup, GetWindowLongPtrW, GetWindowTextW,
        GetWindowThreadProcessId, IsWindow, IsWindowVisible, GA_ROOTOWNER, GWL_EXSTYLE,
        WINDOW_EX_STYLE, WS_EX_TOOLWINDOW,
    },
};
#[no_mangle]
#[used]
pub static mut WINDOW_LIST: Vec<Window> = Vec::new();

pub extern "system" fn enum_window(window: HWND, _: LPARAM) -> BOOL {
    unsafe {
        if is_alt_tab_window(window) && // not sure if this is needed anymore
            !ignore_window(window) &&
            IsWindow(window).as_bool() &&
            IsWindowVisible(window).as_bool()
        {
            let mut text: [u16; 512] = [0; 512];
            let mut process_exe_name: [u8; 512] = [0; 512];
            let mut process_id = 0;
            let process_id: *mut u32 = &mut process_id;

            let len = GetWindowTextW(window, &mut text);
            let mut text = String::from_utf16_lossy(&text[..len as usize]);

            if text.is_empty() {
                // note: false is error, not 'not enumerate'
                return true.into(); 
            }

            GetWindowThreadProcessId(window, Some(process_id));

            let handle = OpenProcess(PROCESS_QUERY_INFORMATION, false, *process_id);

            if let Ok(handle) = handle {
                let len = K32GetProcessImageFileNameA(handle, &mut process_exe_name);
                let process_exe_name = String::from_utf8_lossy(&process_exe_name[..len as usize]);
                text = text
                    + " | "
                    + PathBuf::from(&process_exe_name.as_ref())
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap();
            };

            WINDOW_LIST.push(Window::new(text, window));
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
