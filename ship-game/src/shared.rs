//! Window and asset settings for the 3D viewer.

use bevy::prelude::*;

pub(crate) fn primary_window() -> Window {
    #[cfg(target_arch = "wasm32")]
    {
        Window {
            title: "Time Helm".into(),
            canvas: Some("#ship-game-canvas".into()),
            fit_canvas_to_parent: true,
            ..default()
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Window {
            title: "Time Helm".into(),
            ..default()
        }
    }
}

pub(crate) fn asset_plugin() -> AssetPlugin {
    #[cfg(target_arch = "wasm32")]
    {
        // Absolute, so `/3d/` does not turn this into `/3d/assets/...`.
        AssetPlugin {
            file_path: "/assets".into(),
            meta_check: bevy::asset::AssetMetaCheck::Never,
            ..default()
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        AssetPlugin {
            file_path: "../assets".into(),
            ..default()
        }
    }
}
