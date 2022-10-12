#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ffi;

use crate::ffi::{enum_window, WINDOW_LIST};
use egui::{Label, Sense};
use focus_window::Window;

use windows::{
    Win32::Foundation::LPARAM,
    Win32::{
        Foundation::WPARAM,
        UI::WindowsAndMessaging::{
            EnumWindows, PostMessageA, SetForegroundWindow, ShowWindow, SW_SHOWNORMAL, WM_CLOSE,
            WM_QUIT,
        },
    },
    /* for finding type of windows when adding them on ignore list */
    // Win32::UI::WindowsAndMessaging::{GetWindowLongPtrW, GWL_EXSTYLE},
};

use winit::{dpi::LogicalPosition, event::WindowEvent};

fn main() {
    let clear_color = [0.1, 0.1, 0.1];

    let mut window_name = "".to_string();
    let mut first_draw = true;
    let mut quit = false;

    let event_loop = glutin::event_loop::EventLoopBuilder::with_user_event().build();

    let monitor_size = event_loop.primary_monitor().unwrap().size();
    let monitor_size = Size {
        height: monitor_size.height,
        width: monitor_size.width,
    };
    let window_size = Size {
        height: 800,
        width: 600,
    };

    let (gl_window, gl) = create_display(&event_loop, &monitor_size, &window_size);
    let gl = std::sync::Arc::new(gl);

    let mut egui_glow = egui_glow::EguiGlow::new(&event_loop, gl.clone());

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            let repaint_after = egui_glow.run(gl_window.window(), |ctx| {
                populate_window();

                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        let textbox = ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::singleline(&mut window_name),
                        );

                        if ui.input().key_pressed(egui::Key::Escape) {
                            quit = true;
                        }

                        if first_draw {
                            textbox.request_focus();
                            gl_window
                                .window()
                                .set_inner_size(glutin::dpi::PhysicalSize {
                                    height: window_size.height,
                                    width: window_size.width,
                                });
                            first_draw = false;
                        }
                    });

                    // Unsafe because we deal with the FFI; consider wrapping it
                    unsafe {
                        let matched_windows = WINDOW_LIST
                            .iter()
                            .filter(|entry| matches(entry, &window_name))
                            .into_iter();
                        let window_count = matched_windows.clone().count();
                        let mut last_entry: Option<&Window> = None;

                        for entry in matched_windows.clone() {
                            if matches(entry, &window_name) {
                                last_entry = Some(entry);
                                let label =
                                    ui.add(Label::new(entry.name.clone()).sense(Sense::click()));

                                if label.clicked() {
                                    ShowWindow(entry.window, SW_SHOWNORMAL);
                                    SetForegroundWindow(entry.window);
                                    quit = true;
                                }

                                if label.has_focus() && ui.input().key_pressed(egui::Key::Delete) {
                                    PostMessageA(entry.window, WM_CLOSE, WPARAM(0), LPARAM(0));
                                }

                                if label.has_focus()
                                    && ui
                                        .input_mut()
                                        .consume_key(egui::Modifiers::SHIFT, egui::Key::Delete)
                                {
                                    PostMessageA(entry.window, WM_QUIT, WPARAM(0), LPARAM(0));
                                }

                                /* for finding type of windows when adding them on ignore list */
                                // let code = GetWindowLongPtrW(entry.window, GWL_EXSTYLE);
                                // println((){}", code);
                            }
                        }
                        
                        // no matter which element is selected, if there's only one entry in the window list,
                        // we want to open it and close focus_window
                        if ui.input().key_pressed(egui::Key::Enter)
                            && window_count == 1
                        {
                            match last_entry {
                                Some(entry) => {
                                    ShowWindow(entry.window, SW_SHOWNORMAL);
                                    SetForegroundWindow(entry.window);
                                    quit = true;
                                }
                                None => {}
                            }
                            // figure out how to move to the labels when pressing enter
                        }
                    }
                });
                clear_window();
            });

            *control_flow = if quit {
                glutin::event_loop::ControlFlow::Exit
            } else if repaint_after.is_zero() {
                gl_window.window().request_redraw();
                glutin::event_loop::ControlFlow::Poll
            } else if let Some(repaint_after_instant) =
                std::time::Instant::now().checked_add(repaint_after)
            {
                glutin::event_loop::ControlFlow::WaitUntil(repaint_after_instant)
            } else {
                glutin::event_loop::ControlFlow::Wait
            };

            {
                unsafe {
                    use glow::HasContext as _;
                    gl.clear_color(clear_color[0], clear_color[1], clear_color[2], 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT);
                }

                // draw things behind egui here

                egui_glow.paint(gl_window.window());

                // draw things on top of egui here

                gl_window.swap_buffers().unwrap();
            }
        };

        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),

            glutin::event::Event::WindowEvent { event, .. } => {
                if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                }

                if let glutin::event::WindowEvent::Resized(physical_size) = &event {
                    gl_window.resize(*physical_size);
                } else if let glutin::event::WindowEvent::ScaleFactorChanged {
                    new_inner_size,
                    ..
                } = &event
                {
                    gl_window.resize(**new_inner_size);
                }

                let repaint = egui_glow.on_event(&event);

                if repaint {
                    gl_window.window().request_redraw();
                }
            }
            glutin::event::Event::LoopDestroyed => {
                egui_glow.destroy();
            }
            glutin::event::Event::NewEvents(glutin::event::StartCause::ResumeTimeReached {
                ..
            }) => {
                gl_window.window().request_redraw();
            }

            _ => (),
        }
    });
}

fn populate_window() {
    unsafe {
        EnumWindows(Some(enum_window), LPARAM(0))
            .ok()
            .expect("Unsafe code calling into WIN32 API through FFI failed.");
    }
}

// Reason for unsafe: Using static variable used by FFI calls
fn clear_window() {
    unsafe {
        WINDOW_LIST.clear();
    }
}

fn matches(entry: &Window, window_name: &str) -> bool {
    entry
        .name
        .to_lowercase()
        .contains(window_name.to_lowercase().trim())
}

struct Size {
    height: u32,
    width: u32,
}

pub(crate) fn create_display(
    event_loop: &glutin::event_loop::EventLoop<()>,
    monitor_size: &Size,
    window_size: &Size,
) -> (
    glutin::WindowedContext<glutin::PossiblyCurrent>,
    glow::Context,
) {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_position(LogicalPosition {
            x: monitor_size.width / 2 - window_size.width / 2,
            y: monitor_size.height / 2 - window_size.height / 2,
        })
        .with_resizable(false)
        // Setting initial size to 0 then resizing to normal once the screen has
        // been drawn works around issue described here: https://github.com/emilk/egui/issues/1802
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 0.0,
            height: 0.0,
        })
        .with_decorations(false)
        .with_title("focus_window");

    let gl_window = unsafe {
        glutin::ContextBuilder::new()
            .with_depth_buffer(0)
            .with_srgb(false)
            .with_stencil_buffer(0)
            .with_vsync(true)
            .build_windowed(window_builder, event_loop)
            .unwrap()
            .make_current()
            .unwrap()
    };

    let gl = unsafe { glow::Context::from_loader_function(|s| gl_window.get_proc_address(s)) };

    (gl_window, gl)
}
