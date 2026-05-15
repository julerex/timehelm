//! Window and asset settings shared by 3D and 2D ship modes.

use bevy::prelude::*;

pub(crate) fn primary_window() -> Window {
    #[cfg(target_arch = "wasm32")]
    {
        Window {
            title: "Ship Game - Time Helm".into(),
            canvas: Some("#ship-game-canvas".into()),
            fit_canvas_to_parent: true,
            ..default()
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Window {
            title: "Ship Game - Time Helm".into(),
            ..default()
        }
    }
}

pub(crate) fn asset_plugin() -> AssetPlugin {
    #[cfg(target_arch = "wasm32")]
    {
        // Skip HTTP fetches for `.meta` sidecars: static hosting has no meta files, and Bevy
        // would fall back to default meta anyway (`AssetReaderError::NotFound`).
        AssetPlugin {
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
