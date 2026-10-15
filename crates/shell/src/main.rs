#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> eframe::Result {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    oneoffice::run(paths)
}
