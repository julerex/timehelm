//! Embeds WGSL into the binary (works for WASM without separate asset HTTP fetches).

use bevy::asset::embedded_asset;
use bevy::prelude::*;

pub struct ShipShaderEmbedPlugin;

impl Plugin for ShipShaderEmbedPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/ship_clip_forward.wgsl");
        embedded_asset!(app, "shaders/ship_clip_prepass.wgsl");
    }
}
