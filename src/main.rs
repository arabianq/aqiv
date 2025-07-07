#![windows_subsystem = "windows"]
mod app;

use clap::Parser;
use std::fs;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    file_path: String,
}

fn main() -> Result<(), eframe::Error> {
    let args = Args::parse();

    let mut img_path = Path::new(&args.file_path).to_path_buf();
    if !img_path.exists() {
        println!("File {:?} does not exist!", img_path);
        std::process::exit(1);
    }

    img_path = fs::canonicalize(&img_path).unwrap_or(img_path);

    app::run(img_path)
}
