#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod ui;

fn main() {
    if let Err(error) = ui::run() {
        eprintln!("Purge: {error:#}");
    }
}
