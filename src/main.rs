#![windows_subsystem = "windows"]
use std::env;
use std::path::Path;

mod app;

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        println!("No file path provided!");
        std::process::exit(-1);
    }

    let file_path = Path::new(&args[0]).to_path_buf();

    if !file_path.exists() {
        println!("File {:?}  does not exist!", file_path);
        std::process::exit(-1);
    }

    app::run(file_path)
}
