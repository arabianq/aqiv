mod app;

use magick_rust::magick_wand_genesis;

use clap::Parser;
use std::path::PathBuf;
use std::sync::Once;

static START: Once = Once::new();

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(required = false, default_missing_value = "")]
    file_path: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    START.call_once(magick_wand_genesis);

    let img_path: Option<PathBuf>;

    let args = Args::parse();
    let img_path_arg = args.file_path.unwrap_or_default();

    if img_path_arg.is_empty() {
        img_path = None;
    } else {
        let _img_path = PathBuf::from(img_path_arg).canonicalize().unwrap();

        if !_img_path.exists() {
            println!("File {:?} does not exist!", _img_path);
            std::process::exit(1);
        }

        img_path = Some(_img_path);
    }

    app::run(img_path)
}
