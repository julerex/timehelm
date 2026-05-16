//! Time Helm ship client: 3D cruise-ship simulation and a top-down 2D plan view (Bevy, WASM).
//!
//! **Native desktop:** `cargo run` defaults to 3D; pass `--2d` or `-2` for the 2D plan, `--3d` or `-3` for 3D.

mod app_2d;
mod app_3d;
mod cell;
pub mod cell_box;
mod deck_geometry;
mod deck_layout;
mod shader_embed;
mod shared;
mod ship_hull;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// `0` = 3D ship, `1` = top-down 2D plan.
pub const RUN_MODE_3D: u8 = 0;
pub const RUN_MODE_2D: u8 = 1;

fn run_with_mode(mode: u8) {
    match mode {
        RUN_MODE_2D => app_2d::run_app_2d(),
        _ => app_3d::run_app_3d(),
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run(mode: u8) {
    run_with_mode(mode);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run_native() {
    run_with_mode(parse_native_mode());
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_native_mode() -> u8 {
    let args = std::env::args().skip(1);
    for a in args {
        if a == "--2d" || a == "-2" {
            return RUN_MODE_2D;
        }
        if a == "--3d" || a == "-3" {
            return RUN_MODE_3D;
        }
    }
    RUN_MODE_3D
}
