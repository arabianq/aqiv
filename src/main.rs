#![windows_subsystem = "windows"]
mod app;

use clap::Parser;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    file_path: String,
}

fn main() -> Result<(), eframe::Error> {
    let args = Args::parse();

    let img_path = Path::new(&args.file_path).to_path_buf();
    if !img_path.exists() {
        println!("File {:?} does not exist!", img_path);
        std::process::exit(1);
    }

    app::run(img_path)
}
