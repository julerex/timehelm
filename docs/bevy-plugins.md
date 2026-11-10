# Bevy plugins for the ship viewer

Checked on 2026-11-10 against Bevy 0.18.1, published on crates.io on 2026-03-04. That is the newest 0.18 release. crates.io also lists Bevy 0.19.1 and 0.20.1. Every third-party version below is a release whose own manifest still depends on Bevy 0.18. Newer uploads of the same crates target later Bevy versions and are the wrong pin for `ship-game`.

The Bevy client is a read-only viewer. It renders snapshots. The Axum server is the only authority. One real second is one sim second, and `World::step` covers 60 seconds. The client is a `wasm32` cdylib on WebGL2 and a native binary. Bevy is pinned at 0.18 with default features off. `leafwing-input-manager` 0.20.0 is already the input stack. A crate is listed only when its own repository, its crates.io manifest, or the Bevy 0.18.1 tree gives it a job on that viewer.

## Use what Bevy 0.18 already ships

`ship-game` already enables `bevy_gltf`, `gltf_animation`, `bevy_text`, `bevy_ui`, `bevy_ui_render`, `bevy_sprite`, `bevy_sprite_render`, `bevy_state`, and `webgl2`. Skinned character clips, UI text, sprites, states, and glTF scenes stay on those features. The sections below are capabilities in the same 0.18.1 release that the crate does not turn on yet, or behaviour the viewer already has in its own code.

### Room and crew picking

Enable the `mesh_picking` feature and add `MeshPickingPlugin`. In the Bevy 0.18.1 manifest the `mesh_picking` example is marked `wasm = true` and requires that feature. The backend raycasts meshes that carry `RenderAssetUsages::MAIN_WORLD`. `MeshPickingSettings::require_markers` makes that opt-in, with `Pickable` on the camera and the target. Pointer observers (`Pointer<Over>`, `Pointer<Click>`) are the highlight and the future click.

A deck floor is one merged mesh. A mesh hit names the deck, not the room. Keep `cursor_on_deck` and the point-in-polygon test for rooms. Crew are separate glTF scenes, so the mesh backend can target a person. The 0.18.1 mesh-picking docs do not say whether a skinned mesh is tested in bind pose or in the posed shape.

Highlight the hovered room with a gizmo line around its polygon. `bevy_gizmos` draws lines, arcs, arrows, circles, curves, grids, and primitives. The 0.18.1 crate has no text gizmo. With default features off, enable both `bevy_gizmos` and `bevy_gizmos_render`. The 0.17-to-0.18 migration guide says the render backend was split out. The `3d_gizmos` example is marked `wasm = true`. Highlight a crew mesh by changing its material from a pointer observer. That stays inside `bevy_pbr`.

The 0.18 migration renamed the engine feature `bevy_mesh_picking_backend` to `mesh_picking`. The standalone crate `bevy_mod_picking` 0.20.1, its newest release, depends on Bevy `^0.14.0` and was published on 2024-07-09. It does not declare Bevy 0.18. Source: [aevyrie/bevy_mod_picking](https://github.com/aevyrie/bevy_mod_picking), crate 0.20.1 on crates.io.

### Names

The hover line already prints the room, up to four stock rows, and up to four crew names. World-space labels do not have a 0.18 crate to add. `bevy_mod_billboard` 0.7.0, the newest release, depends on Bevy `^0.14` and was published on 2024-07-10. `Text2d` is in Bevy 0.18.1, and its example is marked `wasm = true`, but that example spawns `Camera2d` and its header describes a 2D scene. This note did not verify that `Text2d` draws in the Z-up `Camera3d` rig. Gizmos in 0.18.1 do not draw text. Names stay on the `bevy_ui` hover line.

### Crew glide

`sample_crew` lerps from the previous delta to the next across 60 seconds. Crew walk a straight line at 1.3 m/s, at most 78 m per step. That lerp is the viewer interpolation. `gltf_animation` is already enabled for the skinned clips. A tween crate would put an ease curve on a motion that is constant speed.

### HUD

The status line already prints the clock, deck, crew count, food days, water days, and power. Stable `bevy_ui` widgets in 0.18.1 are button, image, label, text, and viewport. Checkbox, menu, popover, radio, scrollbar, and slider live in `bevy_ui_widgets`, behind the feature `experimental_bevy_ui_widgets`. That crate's own header says the API will change. The `standard_widgets` example is marked `wasm = true` and requires the experimental feature. `experimental_bevy_feathers` is the tooling set (the 0.18 release notes add a `ColorPlane` there). The ship HUD stays on `bevy_ui` text and nodes. A stock list that must scroll can use the experimental scrollbar later. It does not need a second UI renderer.

### Assets, camera, and the unbound action

Shaders are already embedded with `embedded_asset!`. On wasm the asset plugin reads `/assets` and sets `AssetMetaCheck::Never`. Character glTF files keep loading through `AssetServer`. `bevy_state` is already on, so a loading state does not need a new state crate by itself.

`leafwing-input-manager` 0.20.0 depends on Bevy `^0.18.0-rc.2`. Pan, orbit, deck changes, and the unbound `Interact` action stay there. Bevy 0.18.1 also ships `free_camera` and `pan_camera` under `bevy_camera_controller`. The 0.18 release notes describe them as development cameras: the free camera ignores geometry, and the pan camera is a 2D pan and zoom. They are not an orbit rig for a 318 m by 60 m hull, and they do not replace the leafwing action map. A later participant click can fire the existing `Interact` action. The server still answers `command_rejected` for `Role::Viewer`.

### Sky, and frames on wasm

`Atmosphere` is part of the 0.18.1 `atmosphere` example, and that example is marked `wasm = true`. The same example also switches the opaque renderer to deferred, and it adds volumetric fog, bloom, and screen-space reflections. Those extras are not required to name the type, and this note did not separate which of them the WebGL2 viewer can run. The example's water is a private material in that file, not a published crate. The ocean surface is the crate in the next section. On wasm, leave frame pacing to the browser. The reason is in the `bevy_framepace` source, cited under dev-only.

## Worth adding

### `bevy_water` 0.18.1

License MIT OR Apache-2.0. A tiled ocean material around the hull. The waves are a vertex offset on that material. Do not call `get_wave_point` to move the ship. The hull position comes from the snapshot.

WASM and WebGL2: supported. The 0.18.1 readme documents a `wasm32-unknown-unknown` WebGL2 build with `--no-default-features --features webgl2,embed_shaders`, and a separate WebGPU build. Default features also turn on `image_utils`, which enables Bevy's `jpeg` and `bevy_log` features. Use the documented WebGL2 set.

Source: [Neopallium/bevy_water](https://github.com/Neopallium/bevy_water), readme for 0.18.1 on [crates.io](https://crates.io/crates/bevy_water/0.18.1).

### `bevy_asset_loader` 0.26.0

License MIT OR Apache-2.0. The crate's compatibility table maps Bevy 0.18 to `bevy_asset_loader` 0.25 and 0.26. 0.26.0's manifest depends on `bevy_asset` and `bevy_state` 0.18. Use it to hold a loading state until the character glTF files have arrived over `/assets`, then spawn the crew scenes. The procedural deck meshes do not need it.

WASM and WebGL2: unstated. The 0.26.0 readme does not mention either. The crate is an asset and state helper, not a renderer.

Source: [NiklasEi/bevy_asset_loader](https://github.com/NiklasEi/bevy_asset_loader) at tag `v0.26.0`.

### `bevy_mod_outline` 0.12.1

License MIT OR Apache-2.0. The 0.12.1 readme maps 0.12.x to Bevy 0.18.x. It outlines meshes, and the `animated_fox` example is a jointed glTF with an outline. The `picking` example selects shapes by click. Use it on a crew scene when a person is the pointer target. A merged deck mesh would outline the whole deck, so rooms stay on the gizmo polygon.

WASM and WebGL2: unstated in the 0.12.1 readme. Ship it in the wasm viewer only after a `wasm32` WebGL2 build of this version succeeds. 0.13 and later target newer Bevy.

Source: [komadori/bevy_mod_outline](https://github.com/komadori/bevy_mod_outline) at tag `v0.12.1`.

## Dev-only, keep out of the production wasm build

### `bevy-inspector-egui` 0.36.0

License MIT OR Apache-2.0. The readme's support table maps Bevy 0.18 to 0.36. `WorldInspectorPlugin` shows entities, resources, and assets. That is the native window for the camera rig, crew entities, and the snapshot resource.

It depends on `bevy_egui` `^0.39`. Pin `bevy_egui` 0.39.1 (license MIT). The 0.39.1 readme says desktop and web, links a live WASM example, and its manifest depends on `wasm-bindgen` and `web-sys` for `wasm32`. WebGL2 is not named in that readme. The 0.39.1 dev dependency does enable Bevy's `webgl2` feature.

The 0.36.0 inspector readme does not mention WASM or WebGL2. Its crates.io dependency list includes `opener` with no target cfg. This note did not verify a `wasm32` build. Depend on the inspector only from `cfg(not(target_arch = "wasm32"))`.

`bevy_egui` is the inspector's UI. It is not the production HUD. The clock and the stock line are already `bevy_ui`.

Sources: [jakobhellermann/bevy-inspector-egui](https://github.com/jakobhellermann/bevy-inspector-egui) at tag `v0.36.0`, and [vladbat00/bevy_egui](https://github.com/vladbat00/bevy_egui) at tag `v0.39.1`.

### `bevy_framepace` 0.21.0

License MIT OR Apache-2.0. The 0.21.0 readme maps Bevy 0.18 to 0.21. On native it sleeps the main thread so a frame cap sits just before input. The 0.21.0 `lib.rs` says `spin_sleep` is not WASM compatible, and that frame limiting should not be used in a browser because it would compete with the browser's frame limiter. The sleep call is `cfg(not(target_arch = "wasm32"))`. Use it on the native binary if that window runs uncapped. Leave it out of the wasm artifact.

Source: [aevyrie/bevy_framepace](https://github.com/aevyrie/bevy_framepace) at tag `v0.21.0`.

### `bevy_dev_tools`

This is a Bevy feature, not a third-party crate. The 0.18 release notes add `EasyScreenshotPlugin` (PrintScreen writes a PNG, JPEG, or BMP) and `EasyScreenRecordPlugin`. Screen recording is off unless the `screenrecording` feature of `bevy_dev_tools` is enabled, and the notes say it is not supported on Windows. This note did not find a wasm statement for either plugin. Leave `bevy_dev_tools` off the production wasm feature list.

Source: [Bevy 0.18 release notes](https://bevy.org/news/bevy-0-18/) and the `bevy_dev_tools` feature in the [v0.18.1 manifest](https://github.com/bevyengine/bevy/blob/v0.18.1/Cargo.toml).

## Do not add

These crates publish a Bevy 0.18 release, or an explicit statement that they do not. Each one fights the read-only WebGL2 viewer or has no 0.18 line.

### `avian3d` 0.6.1

License MIT OR Apache-2.0. The manifest depends on Bevy `^0.18.0`. It is a physics engine. The viewer does not step bodies, and the server does not send velocities to integrate. Default features include `parallel`, which enables Bevy's `multi_threaded` feature. The wasm viewer does not want that thread pool either.

Source: [avianphysics/avian](https://github.com/avianphysics/avian), crate `avian3d` 0.6.1 on crates.io.

### `bevy_rapier3d` 0.34.0

License Apache-2.0. The manifest depends on Bevy `^0.18.1`. Same rejection as Avian: client-side physics would move the hull and the crew from the viewer. `bevy_rapier3d` 0.35.0 depends on Bevy `^0.19.0`, so 0.34.0 is the 0.18 pin. This note did not find a WebGL2 statement in the Rapier readme that was fetched.

Source: [dimforge/rapier](https://github.com/dimforge/rapier), crate `bevy_rapier3d` 0.34.0 on crates.io.

### `bevy_landmass` 0.11.1

License MIT OR Apache-2.0. The manifest depends on Bevy `^0.18`. The landmass readme describes path finding, path simplification, steering, and local collision avoidance. Crew already walk the door graph on the server at 1.3 m/s. The client glides the positions in successive deltas.

Source: [andriyDev/landmass](https://github.com/andriyDev/landmass). The 0.11.1 dependency list is on crates.io.

### `oxidized_navigation` 0.12.0

License MIT OR Apache-2.0. This is the newest release, published 2024-12-25. Its own version table lists Bevy 0.15 for crate 0.12.0. There is no Bevy 0.18 release. The readme also builds navmeshes from physics colliders. The viewer has no colliders.

Source: [TheGrimsey/oxidized_navigation](https://github.com/TheGrimsey/oxidized_navigation), docs for 0.12.0 on docs.rs.

### `bevy_replicon` 0.40.0

License MIT OR Apache-2.0. The manifest depends on Bevy `^0.18`. The readme describes it as a server-authoritative replication crate for ECS entities and components, with no built-in I/O. The Axum server is already the authority, and the wire format is the JSON `snapshot`, `delta`, `unavailable`, and `command_rejected` messages. Replicon would make the client a peer of that world instead of a renderer of those messages.

Source: [projectharmonia/bevy_replicon](https://github.com/projectharmonia/bevy_replicon). The 0.40.0 dependency list is on crates.io.

### `lightyear` 0.27.0

License MIT OR Apache-2.0. The manifest depends on `bevy_app` and `bevy_ecs` `^0.18`. The project readme describes server-client replication, client-side prediction, and snapshot interpolation, and it says world replication goes through `bevy_replicon`. Prediction would invent positions between minute steps. The viewer plays the server's deltas and does not roll the world back.

The same readme says the library is compatible with wasm via WebTransport. That does not make it the viewer's socket. The socket is already `web-sys` `WebSocket` on wasm and `tungstenite` on native.

Source: [cBournhonesque/lightyear](https://github.com/cBournhonesque/lightyear). The 0.27.0 dependency list is on crates.io.

### `bevy_renet` 4.0.1

License MIT OR Apache-2.0. The manifest depends on Bevy `^0.18` and on `renet` `^2`. It is a client and server message library with authentication and connection management. The viewer already has one connection and four JSON message types. Renet would replace that socket. It would not draw the ship, and it is not the owner of `World::step`. `bevy_renet` 5.0.0 is a later line. The 4.0.1 manifest is the 0.18 pin checked here.

Source: [lucaspoffo/renet](https://github.com/lucaspoffo/renet), crate `bevy_renet` 4.0.1 on crates.io.

### `bevy_tweening` 0.15.0

License MIT OR Apache-2.0. The changelog says 0.15.0 is compatible with Bevy 0.18, and the manifest depends on Bevy `^0.18`. It can tween a `Transform`. The crew segment is already a linear lerp over the 60-second step, which is the straight walk. An ease curve would change the 1.3 m/s speed. Deck changes set `CameraRig.target.z` once. WASM and WebGL2 are unstated in the 0.15.0 changelog.

Source: [djeedai/bevy_tweening](https://github.com/djeedai/bevy_tweening), changelog entry for 0.15.0.

### `bevy_mod_billboard` 0.7.0

License MIT OR Apache-2.0. The newest release depends on Bevy `^0.14`. There is no crate release that declares Bevy 0.18. World-space names stay on the hover line until a 0.18 billboard crate exists.

Source: [kulkalkul/bevy_mod_billboard](https://github.com/kulkalkul/bevy_mod_billboard), crate 0.7.0 on crates.io.

## What this pass did not verify

- Whether `MeshPickingPlugin` in 0.18.1 hits a skinned glTF in its posed shape or in bind pose. The module docs only describe a mesh raycast.
- Whether `Text2d` renders from the viewer's `Camera3d`. The 0.18.1 example uses `Camera2d`.
- A `wasm32` WebGL2 build of `bevy_mod_outline` 0.12.1, `bevy_asset_loader` 0.26.0, or `bevy-inspector-egui` 0.36.0. Those readmes do not say.
- Which pieces of the `atmosphere` example (deferred rendering, volumetric fog, bloom, screen-space reflections) run on WebGL2. The example as a whole is marked `wasm = true`.
- A wasm statement for `EasyScreenshotPlugin` or `EasyScreenRecordPlugin`.
- The compiled wasm size of `bevy_water` or `bevy_egui`. The water readme shows the WebGL2 feature flags and does not give a binary size.
