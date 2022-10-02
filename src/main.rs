#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ffi;

use crate::ffi::{enum_window, WINDOW_LIST};
use eframe::{
    egui::{self, Label, Sense},
    epaint::{Pos2, Vec2},
};
use ffi::get_resolution;
use windows::{
    Win32::Foundation::LPARAM,
    Win32::UI::WindowsAndMessaging::{EnumWindows, SetForegroundWindow, ShowWindow, SW_SHOWNORMAL},
    /* for finding type of windows when adding them on ignore list */
    // Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, GWL_EXSTYLE},
};

fn main() {
    // Reason for unsafe: FFI calls
    unsafe {
        EnumWindows(Some(enum_window), LPARAM(0))
            .ok()
            .expect("Unsafe code calling into WIN32 API through FFI failed.");
    }

    let options = default_options();

    eframe::run_native(
        "Window chooser",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

fn default_options() -> eframe::NativeOptions {
    let window_size = Vec2 { x: 800.0, y: 800.0 };
    let resolution = get_resolution();
    let position = Pos2 {
        x: resolution.x / 2.0 - window_size.x / 2.0,
        y: resolution.y / 2.0 - window_size.y / 2.0,
    };
    let mut options = eframe::NativeOptions::default();
    options.initial_window_pos = Some(position);
    options.initial_window_size = Some(window_size);
    options.follow_system_theme = true;
    options.decorated = false;
    options
}

struct MyApp {
    window_name: String,
    startup: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            window_name: "".to_owned(),
            startup: true,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let textbox = ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::singleline(&mut self.window_name),
                );
                if textbox.has_focus() && ui.input().key_pressed(egui::Key::Enter) {
                    // figure out how to move to the labels when pressing enter
                }

                if ui.input().key_pressed(egui::Key::Escape) {
                    frame.close();
                }

                if self.startup {
                    textbox.request_focus();
                    self.startup = false;
                }
            });

            // Unsafe because we deal with the FFI; consider wrapping it
            unsafe {
                for entry in WINDOW_LIST.iter() {
                    if entry
                        .name
                        .to_lowercase()
                        .contains(&self.window_name.to_lowercase().trim())
                        && ui
                            .add(Label::new(entry.name.clone()).sense(Sense::click()))
                            .clicked()
                    {
                        ShowWindow(entry.window, SW_SHOWNORMAL);
                        SetForegroundWindow(entry.window);
                        frame.close();

                        /* for finding type of windows when adding them on ignore list */
                        // let code = GetWindowLongPtrW(entry.window, GWL_EXSTYLE);
                        // println!("{}", code);
                    }
                }
            }
        });
    }
}