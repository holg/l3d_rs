//! L3D 3D Viewer - three-d based viewer for L3D luminaire files

mod viewer;

pub use viewer::run_viewer;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static PENDING_DATA: RefCell<Option<Vec<u8>>> = RefCell::new(None);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).ok();
    log::info!("L3D Viewer WASM initialized");
}

/// Queue L3D data to be loaded by the viewer
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn queue_l3d_data(data: &[u8]) {
    log::info!("Queuing L3D data: {} bytes", data.len());
    PENDING_DATA.with(|d| {
        *d.borrow_mut() = Some(data.to_vec());
    });
}

/// Check if there's pending data and take it
#[cfg(target_arch = "wasm32")]
pub fn take_pending_data() -> Option<Vec<u8>> {
    PENDING_DATA.with(|d| d.borrow_mut().take())
}

/// Start the 3D viewer (call once, handles file drops via queue_l3d_data)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start_viewer_wasm() {
    log::info!("Starting WASM viewer");
    viewer::run_viewer_wasm().await;
}
