#[cfg(not(target_arch = "wasm32"))]
fn main() {
    ship_game::run_native();
}

#[cfg(target_arch = "wasm32")]
fn main() {}
