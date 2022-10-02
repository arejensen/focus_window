use windows::Win32::Foundation::HWND;

#[derive(Debug)]
pub struct Window {
    pub name: String,
    pub window: HWND,
}

impl Window {
    pub fn new(name: String, window: HWND) -> Window {
        Window { name, window }
    }
}
