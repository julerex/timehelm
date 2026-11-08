//! Time Helm 3D viewer. The ship is simulated on the server; this crate only watches it.

mod app_3d;
mod deck_geometry;
mod shader_embed;
mod shared;
mod viewer_net;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run(_mode: u8) {
    app_3d::run_app_3d();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run_native() {
    app_3d::run_app_3d();
}
