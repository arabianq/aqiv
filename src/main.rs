use std::env;
use std::path::Path;

mod app;

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        panic!("No file path provided!")
    }

    let file_path = Path::new(&args[0]).to_path_buf();

    if !file_path.exists() {
        panic!("File {:?}  does not exist!", file_path);
    }

    app::run(file_path)
}
