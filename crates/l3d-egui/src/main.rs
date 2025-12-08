//! L3D 3D Viewer - Native entry point

mod viewer;

use std::env;
use std::fs;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();

    // Check for command line argument (file path)
    let args: Vec<String> = env::args().collect();
    let content = if args.len() > 1 {
        let path = &args[1];
        match fs::read(path) {
            Ok(data) => {
                log::info!("Loading L3D file: {}", path);
                Some(data)
            }
            Err(e) => {
                log::error!("Failed to read '{}': {}", path, e);
                None
            }
        }
    } else {
        log::info!("No file specified - drop an L3D file into the viewer");
        None
    };

    viewer::run_viewer(content).await;
}
