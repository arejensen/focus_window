# Overview

focus_window is a gui app, written in Rust using [egui](https://github.com/emilk/egui/), that enable you to quickly switch between windows.
Currently focus_window relies heavily on ffi through the [windows-rs crate](https://github.com/microsoft/windows-rs), thus it will only work on Windows.

# Installation
You will need to have rust with cargo installed. 

Clone the repo, open a command line prompt in the focus_window directory and enter:
```
cargo build --release
```

Cargo will download all dependencies, compile, and link focus_window. 
You'll find the executable, focus_window.exe, under the target/release subdirectory.

