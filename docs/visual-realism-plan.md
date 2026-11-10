# Make the daylight ship look solid

The player should see sky, sea, deck paint, cast shadows, balcony glass, cabin lamps, and pool water. Camera keys from pull request 12 are already on main. Room meshes and doors stay with pull request 14 until that merges. The stack is vr-sky, vr-deck, vr-sea, vr-shadow, vr-glass, vr-lamps, then vr-pools. Each one changes a single picture and proves it on the running viewer.

## How to read this

One box is one unit of work. Every box names the evidence that checks it. A nested box is a sub-step of the box above it. Check a box only when its evidence exists, a file, a log line, a screenshot, a test run, or a SHA. The body is a how-to. The appendices explain and record.

The program runs `pstack/skills/poteto-mode/playbooks/autopilot-stack.md`. The operator lands every pull request. No owner merges. vr-sky through vr-pools stop at merge-ready.

Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

## Program checklist

### Arm the program

- [ ] State the protocol and this plan to the operator, then stop. Start execution only on the operator's explicit go.
- [ ] Read these from trunk at program start. Re-read them at every tick.
  - [ ] `git show origin/main:pstack/skills/poteto-mode/playbooks/autopilot-stack.md`
  - [ ] `git show origin/main:pstack/skills/swarm/SKILL.md`
  - [ ] `git show origin/main:pstack/skills/control-ui/SKILL.md`
  - [ ] `git show origin/main:pstack/skills/poteto-mode/playbooks/opening-a-pr.md`
  - [ ] `git show origin/main:pstack/skills/poteto-mode/playbooks/shipping.md`
  - [ ] `git show origin/main:pstack/skills/principle-prove-it-works/SKILL.md`
  - [ ] `git show origin/main:pstack/skills/principle-sequence-verifiable-units/SKILL.md`
  - [ ] `git show origin/main:pstack/skills/benchmark-checklist/SKILL.md`
  - [ ] `git show origin/main:pstack/skills/how/SKILL.md`
- [ ] If a `git show` path is missing, open the same skill from the poteto-mode plugin or the cursor-team-kit plugin and record that path in the decision trail.
- [ ] Confirm pull request 14 is merged to `main` before any owner branches. Pull request 12 is already on `main`. Stop the program if pull request 14 is still open.
- [ ] On the operator's go, arm the audit tick as `/loop 1h` with the tick prompt below. Never leave the cadence to memory.
- [ ] Use this tick prompt, verbatim. "Re-read the execution playbook from trunk. Audit the operation against it and fix drift in this tick. Probe every active lane and judge progress by side effects only. Stand down a stuck lane and dispatch its replacement now. Then post a short status message to the operator in chat only when the audit found a tracked change that no earlier status message reported, such as a PR opened, a code-ready head, a round launched or closed, a verdict, a merge, a stuck agent and the action taken, a blocker added or cleared, or a decision only the operator can make. Name every such change and nothing else. Do not repeat a table, the merged list, or an unchanged blocker. If the audit found none, end the turn with no reply text. Either way, log this tick's row in your decision trail. The row names the items reported, or none."
- [ ] On the operator's hold or stand-down, send every owner a zero-writes order at once.

### Spawn owners

- [ ] Spawn one owner per pull request with the full lifecycle in `playbooks/autopilot-stack.md`.
- [ ] Follow this dependency graph. Start dependent work only after its parent is on the parent branch, and base the child on that branch.
  - [ ] vr-sky is first and branches from `main` after pull request 14 has merged.
  - [ ] vr-deck after vr-sky.
  - [ ] vr-sea after vr-deck.
  - [ ] vr-shadow after vr-sea.
  - [ ] vr-glass after vr-shadow.
  - [ ] vr-lamps after vr-glass.
  - [ ] vr-pools after vr-lamps.
- [ ] Hold the file boundaries. Shared edits stay in `ship-game/src/app_3d.rs` and the two clip shaders, and only in the symbol that pull request names. Leave `pan_binding`, `orbit_binding`, `keyboard_pan_delta`, `plan_camera_z`, `deck_and_keymap`, `CAMERA_PAN_SPEED_M_S`, `CAM_DEFAULT_DISTANCE_M`, `DoorLeaf`, `animate_doors`, `spawn_hull`, `ship_hull_plate`, `ship_bulwark_mesh`, `sim/src/layout.rs`, `sim/src/hull.rs`, and the camera sentences in `AGENTS.md` and `README.md` unchanged. vr-pools may change the color constant inside `pool` and no other builder in `ship-game/src/venue_props.rs`.
- [ ] Hold the review gate. No pull request in this program changes a key, a click, or a camera binding. Each review gate says None.

### PR mechanics, for every PR

- [ ] Resolve the forge once. Default to `gh`. If `command -v origin` succeeds and Origin can resolve the repository, use `origin pr` for every pull request operation. Record any fallback to `gh`. Never require `gt`.
- [ ] Open the pull request ready, never draft, per **Opening a PR**. Use the run's built-in pull request tool when it has one, else `origin pr create --status open --base <base-branch>` or `gh pr create --base <base-branch>` according to the resolved forge. A stack child targets its parent branch.
- [ ] Run `make lint-ship` once before the pull-request-facing push. Push with hooks on.
- [ ] Run `/deslop` before each commit and `/no-comments` before review.
- [ ] Triage every Bugbot and security-reviewer comment per `pstack/skills/poteto-mode/references/bugbot-triage.md`.
- [ ] Rebase the stack root onto current trunk before the code-ready report and babysit. Rebase each child onto its parent branch. Keep that merge base in fix rounds. Rebase again only at merge prep, on a `git merge-tree` conflict with the parent, on a CI failure that comes from a change on the parent, or when a known concurrent change to the same files lands.

### Verdict and merge, for every PR

- [ ] A full swarm runs once, at the code-ready head. Another full swarm runs only for new behavior, a redesign, or a large new diff. A rebase, a merge-conflict resolution, or a minor fix gets a single verifier at the new head, and a wording-only change gets none. Run each full swarm per `pstack/skills/swarm/SKILL.md`. One gates lane. The ten live lanes from the pull request's **Verify, live** block. The perf lane from its **Verify, perf** block. Two or more audit lanes, each with its own focus, that read the diff and the receipts and distrust the pull request body. The root audits the receipts in the merge-ready report before the verdict.
- [ ] Judge every finding by the execution playbook's blocking rule. The verdict is clean when none blocks. A failure on trunk behavior in a lane marked `Regression lane against trunk.` is recorded against trunk and does not block the pull request. A new head gets a fresh verdict, except for results that stay valid under the patch-id rule in `playbooks/shipping.md`.
- [ ] Stop when the pull request is verified and green at its current head, then hand it to the operator. Nobody merges. A restack that leaves `git patch-id` unchanged keeps its verdict.

### Boot recipe, for every live lane

Each live lane runs on its own cloud VM at the pull request head. Drive through `control-ui` from `cursor-team-kit`.

- [ ] `git fetch origin <head-branch> && git checkout <head SHA>`.
- [ ] Start Postgres with `sudo pg_ctlcluster 16 main start` when it is not already running. Put `DATABASE_URL=postgresql://postgres:password@localhost:5432/timehelm` in `server/.env`. Apply `database/timehelm_schema.sql` when the server log says the world is unavailable. From the repo root run `make run-ship`. Wait until the log prints `Server listening` and the page status text contains the word `crew`.
- [ ] Open `http://127.0.0.1:8080/3d/` in a Playwright page at 1280 by 800. Click `#ship-game-canvas` and send every later key to that canvas. `isGameKey` in `client/public/ship/run.js` does not list `KeyP`, so P is dropped when the event target is not the canvas. Page Up, Page Down, K, Z, X, and the pan and orbit letters are forwarded from the window. Press K and use the Pan, Orbit, and Dolly keys printed there. On `main`, pan is W and S, strafe is Q and D, and orbit is A and E. Dolly is Z and X unless the panel prints other keys. Follow the panel when it disagrees. Page Up and Page Down change decks. P toggles the plan view. Press K again to close the panel before a screenshot. Read-only diagnostics are the browser console, the status text, and PNG pixel reads.
- [ ] Treat the top 15 percent of rows as topBand, the bottom 20 percent as bottomBand, and the middle 40 percent of width and height as center. Clear is rgb 140, 184, 214, within 8 levels on every channel. A ship pixel is a center pixel that is not clear. A sea pixel has blue at least 20 above red and at least 5 above green, and it is not clear.
- [ ] Save every screenshot to `/tmp/swarm-<pr-id>/worker-<n>/<slug>.png` and return the paths with the report.

## Give the exterior a sky, vr-sky

**Depends on.** Pull request 14 merged to `main`. Pull request 12 is already on `main`. No earlier visual id.

**Files.**

- [ ] Create `ship-game/src/sky.rs`.
- [ ] Edit `ship-game/src/lib.rs` so the module is visible to `app_3d`.
- [ ] Edit `ship-game/src/app_3d.rs` in `setup` and `sync_clip` only.
- [ ] Edit `ship-game/src/shaders/ship_clip_forward.wgsl` to mix fog after `daylight`.
- [ ] Edit `ship-game/src/shaders/ship_clip_prepass.wgsl` so its `ShipClipMaterial` struct matches the Rust struct field for field order.

**Build.**

- [ ] Add `sky_rgba(elevation: f32) -> [f32; 4]` in `sky.rs`. Elevation 1 is zenith and elevation 0 is the horizon. Zenith blue is at least 0.15 above horizon blue. Horizon red is at least 0.05 above zenith red.
- [ ] Generate six 32 by 32 faces at startup from `sky_rgba` and attach a `Skybox` to the `GameCamera3d` entity. Leave `ClearColor` in place for the first frame before the cubemap is ready.
- [ ] Append `fog: Vec4` to `ShipClipMaterial`, with rgb from `sky_rgba(0.0)` and density `0.0008` in w. In `daylight`, mix the lit color toward that rgb by `1.0 - exp(-density * distance_to_eye)`.
- [ ] Leave `DirectionalLight`, `pan_binding`, and `deck_and_keymap` unchanged.
- [ ] Leave the `hdr` feature out of `ship-game/Cargo.toml` and do not insert `Hdr` on the camera. Bevy attaches `Tonemapping`, and that pass does not run while the view is not HDR. The live pixel checks assume that buffer.

**You see.**

- [ ] The top of the frame is a deeper blue than the band above the hull.
- [ ] A far orbit shows the hull fading toward the horizon color.
- [ ] The status line still contains a deck name and the word `crew`.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] `ship-game/src/sky.rs` gains `sky_gradient_darkens_toward_zenith`. Run `cd ship-game && unset ARGV0 && cargo test --lib sky_gradient_darkens_toward_zenith`.
- [ ] The same command filter `directional_light_sun_shades_decks_and_bulkheads` still passes.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Open the default camera on trunk and on the head. Save `sky-regression-trunk.png` and Save `sky-regression-head.png`. Pass when the head topBand blue standard deviation is at least 8, the head center has at least 500 ship pixels, and the trunk topBand deviation is recorded. If the trunk deviation is already at least 8, record that trunk already has a gradient and still require the ship pixels.
- [ ] Lane 2. Leave the default camera for one second after `crew` appears. Save `sky-default.png`. Pass when topBand blue standard deviation is at least 8 and center has at least 500 ship pixels.
- [ ] Lane 3. Orbit until the sun-facing side of the hull fills center. Save `sky-sun-side.png`. Pass when center has at least 500 ship pixels and topBand mean blue is below the mean blue of the top 5 percent of rows.
- [ ] Lane 4. Orbit the opposite way until the shaded side fills center. Save `sky-shade-side.png`. Pass when topBand blue standard deviation is at least 8 and the status line contains `crew`.
- [ ] Lane 5. Dolly in for two seconds. Save `sky-near.png`. Pass when center has at least 500 ship pixels and the console has no line containing `panic`.
- [ ] Lane 6. Dolly out for four seconds. Save `sky-far.png`. Pass when topBand blue standard deviation is at least 8 and at least 5 percent of center pixels are within 20 levels of the horizon color from `sky_rgba(0.0)` scaled to 8-bit.
- [ ] Lane 7. Press Page Up until the status deck name changes twice. Save `sky-upper-deck.png`. Pass when the status line shows the new deck name and topBand is still a gradient with blue standard deviation at least 8.
- [ ] Lane 8. Press Page Down until the status line shows the lowest deck name. Save `sky-lower-deck.png`. Pass when topBand blue standard deviation is at least 8 and center has at least 500 ship pixels.
- [ ] Lane 9. Press P to open the plan view. Save `sky-plan.png`. Pass when the status line contains `plan` and center has at least 500 ship pixels.
- [ ] Lane 10. Reload the page and read the console for five seconds after `crew` appears. Save `sky-console.png`. Pass when the console has no line containing `panic` and no line containing `AtmospherePlugin not loaded`.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Median frame delta over 180 animation frames after the status text contains `crew`, at the default camera, plus the byte size of `client/public/ship/ship_bg.wasm` after `scripts/build-ship.sh`. Trunk and head both produce both numbers. The added work is the skybox faces and the fog mix. The end state is the gradient sky with the ship still in frame.
- [ ] Probe. On one machine, measure trunk, then the head, then trunk again. For each side run `scripts/build-ship.sh`, record `wc -c client/public/ship/ship_bg.wasm`, serve that build, and record the median of 180 frame deltas in the page. Discard the pair when the two trunk medians differ by more than 1 ms and run the probe again.
- [ ] Baseline. Record the first trunk median and the first trunk wasm size before the head run.
- [ ] Rule. Fail when the head median exceeds the first trunk median by more than 2 ms, or when the head wasm size exceeds the first trunk wasm size by more than 65536 bytes. The 2 ms and 1 ms figures are budgets set before measurement, not a measured speedup.

**Review gate.** None. vr-sky does not change a key, a click, or a camera binding.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Restacked onto current `main`, with `git patch-id` unchanged.
- [ ] Hand vr-sky to the operator at merge-ready. Do not merge it.

## Paint the decks at hull scale, vr-deck

**Depends on.** vr-sky.

**Files.**

- [ ] Edit `procedural_deck_plan_texture_image` in `ship-game/src/deck_geometry.rs` and no function below it.
- [ ] Edit `ShipClipMaterial`, `setup`, and `sync_clip` in `ship-game/src/app_3d.rs`.
- [ ] Edit `ship-game/src/shaders/ship_clip_forward.wgsl` and `ship-game/src/shaders/ship_clip_prepass.wgsl`.

**Build.**

- [ ] Replace the 160 by 160 nearest checker with a 512 by 512 image. Set that image's sampler to linear. Leave `ImagePlugin::default_nearest` in place so crew glTFs stay on the default sampler.
- [ ] Append `plan_size: Vec4` to `ShipClipMaterial`. Write `SHIP_LENGTH_M` and `SHIP_BEAM_M` into x and y from `sync_clip`. In the fragment, sample with `world_position.xy` divided by that size.
- [ ] Add `deck_pattern_weight(normal_z: f32) -> f32` and use the same thresholds in the shader. Weight is 0 when `normal_z` is below 0.45 and 1 when `normal_z` is above 0.85. Walls and the hull plate then ignore the deck image.
- [ ] Keep the plank period under 0.4 m in world space so a deck at the default camera is not a two-tone block.

**You see.**

- [ ] Deck tops show a fine repeating pattern.
- [ ] Bulkheads stay a flat paint color with no plank stripes.
- [ ] The status line still contains `crew`.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] `ship-game/src/deck_geometry.rs` gains `deck_texture_is_fine_and_linear`. It asserts width 512, height 512, a linear sampler, and more than 8 brightness changes along one row. Run `cd ship-game && unset ARGV0 && cargo test --lib deck_texture_is_fine_and_linear`.
- [ ] `deck_pattern_weight_skips_walls` asserts weight 0 at normal z 0.2 and weight 1 at normal z 0.95. Run `cd ship-game && unset ARGV0 && cargo test --lib deck_pattern_weight_skips_walls`.
- [ ] `cd ship-game && unset ARGV0 && cargo test --lib` passes, including `directional_light_sun_shades_decks_and_bulkheads`.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Dolly in for two seconds on trunk and on the head. Save `deck-regression-trunk.png` and Save `deck-regression-head.png`. Pass when the head center mean absolute difference from the trunk center is at least 6 levels and the head center still has at least 500 ship pixels. Record the trunk shot as the coarse checker.
- [ ] Lane 2. Default camera. Save `deck-default.png`. Pass when center has at least 500 ship pixels and the status line contains `crew`.
- [ ] Lane 3. Orbit to the sun-facing side and dolly in for two seconds. Save `deck-sun-side.png`. Pass when an 80 pixel horizontal run across a deck top has at least 12 brightness changes.
- [ ] Lane 4. Orbit to the shaded side. Save `deck-shade-side.png`. Pass when a wall region, a vertical face, shows fewer than 4 brightness changes along an 80 pixel run.
- [ ] Lane 5. Dolly in for four seconds. Save `deck-near.png`. Pass when the deck run from lane 3 still has at least 12 brightness changes and the console has no `panic`.
- [ ] Lane 6. Dolly out for four seconds. Save `deck-far.png`. Pass when center has at least 500 ship pixels and topBand blue standard deviation is at least 8.
- [ ] Lane 7. Page Up twice. Save `deck-upper.png`. Pass when the status deck name changed and center has at least 500 ship pixels.
- [ ] Lane 8. Page Down to the lowest deck. Save `deck-lower.png`. Pass when the status line shows that deck and center has at least 500 ship pixels.
- [ ] Lane 9. Press P. Save `deck-plan.png`. Pass when the status line contains `plan` and an 80 pixel run on the plan has at least 12 brightness changes.
- [ ] Lane 10. Resize the viewport to 800 by 600 and back to 1280 by 800. Save `deck-resize.png`. Pass when center has at least 500 ship pixels and the console has no `panic`.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. The same median frame delta and wasm byte size as vr-sky, measured on trunk and on the head. The added work is the 512 image upload and the pattern weight. The end state is a fine deck under the sky from vr-sky.
- [ ] Probe. Measure trunk, then the head, then trunk again, with `scripts/build-ship.sh` and 180 frame deltas after `crew`. Discard the pair when the two trunk medians differ by more than 1 ms.
- [ ] Baseline. Record the first trunk median and the first trunk wasm size.
- [ ] Rule. Fail when the head median exceeds the first trunk median by more than 2 ms, or when the head wasm exceeds the first trunk wasm by more than 65536 bytes.

**Review gate.** None. vr-deck does not change a key, a click, or a camera binding.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Restacked onto vr-sky, with `git patch-id` unchanged.
- [ ] Hand vr-deck to the operator at merge-ready. Do not merge it.

## Put the hull in the sea, vr-sea

**Depends on.** vr-deck.

**Files.**

- [ ] Create `ship-game/src/sea.rs`.
- [ ] Create `ship-game/src/shaders/sea.wgsl`.
- [ ] Edit `ship-game/src/lib.rs` and `ship-game/src/shader_embed.rs` so the new shader is embedded the same way as `ship_clip_forward.wgsl`.
- [ ] Edit `spawn_ship_once` in `ship-game/src/app_3d.rs` to spawn the sea and the skirt. Leave `spawn_hull` unchanged.

**Build.**

- [ ] Add `SeaMaterial` with sun, eye, time, and fog color. Fog rgb comes from `sky_rgba(0.0)`. `alpha_mode` returns `AlphaMode::Opaque`.
- [ ] Add `displace_sea(x, y, time) -> f32` as `0.08 * sin(0.06 * x + 1.3 * time) + 0.05 * sin(0.11 * y - 0.9 * time)`. Build a 64 by 64 grid covering four ship lengths and four beams, rest height `z = -0.4`, and move the vertices on the CPU each frame with `displace_sea`.
- [ ] In `sea.wgsl`, color by fresnel and a sun specular using the same sun direction `sync_clip` already writes. Copy the fog mix from vr-sky.
- [ ] Add `sea_skirt_mesh` from the deck 0 outline, from `z = -0.4` to `z = 0.05`, painted with the hull plate color, using `ShipClipMaterial`. Do not edit `ship_hull_plate`.

**You see.**

- [ ] Water surrounds the hull and sits below deck 0.
- [ ] Two frames half a second apart differ in the water and match on the deck.
- [ ] The deck pattern is absent from the water.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] `sea_crests_stay_below_deck_floor` asserts every sample of `displace_sea` over a 20 by 20 grid and four times stays below 0.15, so a crest at rest `z = -0.4` stays under the deck 0 plate. Run `cd ship-game && unset ARGV0 && cargo test --lib sea_crests_stay_below_deck_floor`.
- [ ] `sea_grid_is_64_and_wider_than_the_hull` asserts 64 by 64 vertices and an extent of at least twice `SHIP_LENGTH_M`. Run `cd ship-game && unset ARGV0 && cargo test --lib sea_grid_is_64_and_wider_than_the_hull`.
- [ ] `cd ship-game && unset ARGV0 && cargo test --lib` passes.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Default camera on trunk and on the head. Save `sea-regression-trunk.png` and Save `sea-regression-head.png`. Pass when the head frame has at least 2 percent sea pixels and the trunk frame has under 0.5 percent sea pixels. If the trunk frame already has 2 percent sea pixels, record that and still require the head ship pixels.
- [ ] Lane 2. Default camera. Save `sea-default.png`. Pass when at least 2 percent of the frame is sea pixels and center has at least 500 ship pixels.
- [ ] Lane 3. Orbit to the sun-facing side. Save `sea-sun-side.png`. Pass when a sea patch of 40 by 40 pixels has a max channel at least 25 levels above its min channel.
- [ ] Lane 4. Orbit to the shaded side. Save `sea-shade-side.png`. Pass when bottomBand still contains at least 200 sea pixels.
- [ ] Lane 5. Save `sea-wave-a.png`, wait 500 ms, and Save `sea-wave-b.png`. Pass when the mean absolute difference inside a sea patch is at least 2 levels and the mean absolute difference inside a deck patch is under 2 levels.
- [ ] Lane 6. Dolly out for four seconds. Save `sea-far.png`. Pass when at least 5 percent of the frame is sea pixels and topBand blue standard deviation is at least 8.
- [ ] Lane 7. Page Up twice. Save `sea-upper.png`. Pass when sea pixels remain in the frame and the status deck name changed.
- [ ] Lane 8. Page Down to the lowest deck. Save `sea-lower.png`. Pass when the skirt meets the water with no clear-colored gap taller than 8 pixels between hull paint and sea pixels.
- [ ] Lane 9. Press P. Save `sea-plan.png`. Pass when the status line contains `plan` and the frame still has ship pixels. Sea may fill the margin.
- [ ] Lane 10. Dolly in until the sea grid fills the bottom of the frame. Save `sea-near.png`. Pass when a 32 pixel run across the water has more than 4 distinct blue values, and the console has no `panic`.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Median frame delta and wasm size, on trunk and on the head. The added work is the 64 by 64 CPU displace and the sea draw. The end state is a moving sea around a visible hull.
- [ ] Probe. Measure trunk, then the head, then trunk again. Use 180 frames after `crew` at the default camera. Discard the pair when the two trunk medians differ by more than 1 ms.
- [ ] Baseline. Record the first trunk median and the first trunk wasm size.
- [ ] Rule. Fail when the head median exceeds the first trunk median by more than 2 ms, or when the head wasm exceeds the first trunk wasm by more than 65536 bytes. A finer grid than 64 by 64 fails this rule on purpose.

**Review gate.** None. vr-sea does not change a key, a click, or a camera binding.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Restacked onto vr-deck, with `git patch-id` unchanged.
- [ ] Hand vr-sea to the operator at merge-ready. Do not merge it.

## Cast the upper decks onto the lower decks, vr-shadow

**Depends on.** vr-sea.

**Files.**

- [ ] Create `ship-game/src/sun_shadow.rs`.
- [ ] Edit `ShipClipMaterial` and `sync_clip` in `ship-game/src/app_3d.rs`.
- [ ] Edit `ship-game/src/shaders/ship_clip_forward.wgsl` and `ship-game/src/shaders/ship_clip_prepass.wgsl`.

**Build.**

- [ ] Add `occlusion_at(point_xy, deck_index, hulls, deck_height_m, to_sun) -> f32` in `sun_shadow.rs`. A point is in shadow when a higher deck's polygon contains `point_xy + to_sun.xy / to_sun.z * height_delta`. Return 0.45 in shadow and 1.0 in the open.
- [ ] Rasterize one 256 by 256 layer per deck into a `texture_2d_array` and bind it as the next free texture on `ShipClipMaterial`. Sample it in `daylight` for fragments with `normal_z` above 0.85. Leave `DirectionalLight.shadows_enabled` unset.
- [ ] Rebuild the grid when the ship spawns and when `to_sun` changes. Do not rebuild it per frame.

**You see.**

- [ ] An open top deck stays bright.
- [ ] A lower deck directly down-sun from an upper deck is darker than an open patch of that same lower deck.
- [ ] Vertical bulkheads keep the sun term already shipped in pull request 11.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] `open_deck_is_lit_and_covered_deck_is_dark` builds two square hulls. The upper square covers x and y from 100 to 140. The lower deck is the same square plus a pad. A point at 120, 120 on the lower deck is at most 0.45. A point 30 m up-sun of the upper square is 1.0. Run `cd ship-game && unset ARGV0 && cargo test --lib open_deck_is_lit_and_covered_deck_is_dark`.
- [ ] `cd ship-game && unset ARGV0 && cargo test --lib` passes.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Dolly in for two seconds on trunk and on the head, framed on a lower deck beside an upper deck. Save `shadow-regression-trunk.png` and Save `shadow-regression-head.png`. Pass when the head center mean absolute difference from trunk is at least 4 levels, the head center mean luminance stays at least 60 percent of the trunk center, and center has at least 500 ship pixels.
- [ ] Lane 2. Default camera. Save `shadow-default.png`. Pass when center has at least 500 ship pixels and at least 2 percent of the frame is sea pixels.
- [ ] Lane 3. Orbit and dolly until an upper deck and the deck below it are both in center. Save `shadow-pair.png`. Pass when the darker half of center is at least 12 percent darker than the brighter half, and the status line contains `crew`.
- [ ] Lane 4. Orbit about 180 degrees and frame the same two decks. Save `shadow-opposite.png`. Pass when a dark patch remains on the down-sun side of the upper deck, judged against lane 3, and center mean luminance stays at least 60 percent of the trunk shot from lane 1.
- [ ] Lane 5. Dolly in for three seconds on an open top deck. Save `shadow-open-top.png`. Pass when that deck patch mean luminance is at least 15 percent higher than the darker half from lane 3.
- [ ] Lane 6. Dolly out for four seconds. Save `shadow-far.png`. Pass when center has at least 500 ship pixels and topBand blue standard deviation is at least 8.
- [ ] Lane 7. Page Up to a high deck. Save `shadow-upper.png`. Pass when the status deck name changed and center has at least 500 ship pixels.
- [ ] Lane 8. Page Down to the lowest deck. Save `shadow-lower.png`. Pass when center mean luminance stays at least 50 percent of the lane 5 patch.
- [ ] Lane 9. Press P. Save `shadow-plan.png`. Pass when the status line contains `plan` and the plan image is not a single flat color. Its center blue standard deviation is at least 4.
- [ ] Lane 10. Leave the camera still for two seconds and compare two screenshots. Save `shadow-still-a.png` and Save `shadow-still-b.png`. Pass when their center mean absolute difference is under 2 levels and the console has no `panic`.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Median frame delta and wasm size, on trunk and on the head. The added work is one shadow sample in `daylight` and a spawn-time raster. The end state is a lit ship with a darker covered deck.
- [ ] Probe. Measure trunk, then the head, then trunk again, at the default camera, 180 frames after `crew`. Discard the pair when the two trunk medians differ by more than 1 ms.
- [ ] Baseline. Record the first trunk median and the first trunk wasm size.
- [ ] Rule. Fail when the head median exceeds the first trunk median by more than 2 ms, or when the head wasm exceeds the first trunk wasm by more than 65536 bytes.

**Review gate.** None. vr-shadow does not change a key, a click, or a camera binding.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Restacked onto vr-sea, with `git patch-id` unchanged.
- [ ] Hand vr-shadow to the operator at merge-ready. Do not merge it.

## Add balcony glass along the hull, vr-glass

**Depends on.** vr-shadow.

**Files.**

- [ ] Create `ship-game/src/balcony_glass.rs`.
- [ ] Create `ship-game/src/shaders/glass.wgsl`.
- [ ] Edit `ship-game/src/lib.rs`, `ship-game/src/shader_embed.rs`, and `spawn_ship_once` in `ship-game/src/app_3d.rs`.

**Build.**

- [ ] Add `balcony_spans(outline) -> Vec<Span>` for the port edge and the starboard edge only. Skip bow and stern edges, skip edges shorter than 4 m, and skip edges that bound the park hole. Each span is a quad from 0.9 m to 2.2 m above that deck's floor, pushed 0.05 m outboard.
- [ ] Spawn those quads on decks 5 through 14 with a `GlassMaterial` whose `alpha_mode` is `AlphaMode::Blend`. The shader reflects the sky color from `sky_rgba` and keeps the pane transparent enough that the bulkhead behind it darkens the pixel.
- [ ] Leave `push_wall`, `DoorLeaf`, and `animate_doors` unchanged.

**You see.**

- [ ] A cool band runs along the outboard side and is missing from the bow tip.
- [ ] Door leaves still move. The band does not seal a door gap.
- [ ] The sea and the sky are still in the frame.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] `balcony_spans_are_the_long_sides` uses a 200 m by 40 m rectangle. It returns two spans, none of them on the short ends, each with z from 0.9 to 2.2. Run `cd ship-game && unset ARGV0 && cargo test --lib balcony_spans_are_the_long_sides`.
- [ ] `cd ship-game && unset ARGV0 && cargo test --lib` passes.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Orbit to a long side and dolly in for two seconds, on trunk and on the head. Save `glass-regression-trunk.png` and Save `glass-regression-head.png`. Pass when the head frame has at least 200 edge pixels with blue greater than red plus 15, the trunk frame of that pose has fewer than 40 such pixels, and center has at least 500 ship pixels. If trunk already has 200 such pixels, record that and still require the ship pixels.
- [ ] Lane 2. Default camera. Save `glass-default.png`. Pass when center has at least 500 ship pixels and at least 2 percent of the frame is sea pixels.
- [ ] Lane 3. Orbit to the port side and dolly in. Save `glass-port.png`. Pass when at least 200 edge pixels have blue greater than red plus 15.
- [ ] Lane 4. Orbit to the starboard side and dolly in. Save `glass-starboard.png`. Pass when at least 200 edge pixels have blue greater than red plus 15.
- [ ] Lane 5. Orbit until the bow tip fills center. Save `glass-bow.png`. Pass when fewer than 40 edge pixels have blue greater than red plus 15.
- [ ] Lane 6. Dolly out for four seconds. Save `glass-far.png`. Pass when the cool band is still visible as at least 80 such edge pixels and topBand blue standard deviation is at least 8.
- [ ] Lane 7. Page Up onto a deck in the 5 to 14 range. Save `glass-upper.png`. Pass when the status deck name changed and the long side still shows the cool band.
- [ ] Lane 8. Page Down to the lowest deck. Save `glass-lower.png`. Pass when that deck shows fewer than 40 cool edge pixels, because balcony spans start at deck 5.
- [ ] Lane 9. Press P. Save `glass-plan.png`. Pass when the status line contains `plan` and center has at least 500 ship pixels.
- [ ] Lane 10. Watch a door for one second on a cabin deck. Save `glass-door-a.png` and Save `glass-door-b.png`. Pass when the two frames are not identical and the console has no `panic`.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Median frame delta and wasm size, on trunk and on the head. The added work is the balcony quads and one blended shader. The end state is a long side with a cool band and a moving door.
- [ ] Probe. Measure trunk, then the head, then trunk again, 180 frames after `crew` at the default camera. Discard the pair when the two trunk medians differ by more than 1 ms.
- [ ] Baseline. Record the first trunk median and the first trunk wasm size.
- [ ] Rule. Fail when the head median exceeds the first trunk median by more than 2 ms, or when the head wasm exceeds the first trunk wasm by more than 65536 bytes.

**Review gate.** None. vr-glass does not change a key, a click, or a camera binding.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Restacked onto vr-shadow, with `git patch-id` unchanged.
- [ ] Hand vr-glass to the operator at merge-ready. Do not merge it.

## Light the cabins from lamps, vr-lamps

**Depends on.** vr-glass.

**Files.**

- [ ] Create `ship-game/src/lamps.rs`.
- [ ] Edit `ShipClipMaterial`, `spawn_ship_once`, and `sync_clip` in `ship-game/src/app_3d.rs`.
- [ ] Edit `ship-game/src/shaders/ship_clip_forward.wgsl` and `ship-game/src/shaders/ship_clip_prepass.wgsl`.

**Build.**

- [ ] Add `place_lamps(rooms) -> Vec<Vec4>` with at most 8 entries. One lamp at the centroid of a cabin, a galley, or a dining room, with xyz the position and w the radius 6 m. Drop later rooms after 8.
- [ ] Add `lamp_gain(world_pos, lamps) -> f32`. Gain is above 0.5 at the lamp and below 0.05 at 20 m.
- [ ] Append `lamps: array<vec4<f32>, 8>` to the clip uniform and add `lamp_gain` inside `daylight` as a warm term `vec3(1.0, 0.78, 0.45)`. Leave the sun direction term in place.
- [ ] Do not spawn `PointLight` entities. The clip shader does not read Bevy point lights.

**You see.**

- [ ] A cabin shows a warm patch around the lamp.
- [ ] An open top deck does not turn orange.
- [ ] The covered deck from vr-shadow stays darker away from the lamp.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] `lamp_gain_falls_off_by_20_m` asserts gain above 0.5 at the lamp and below 0.05 at 20 m. Run `cd ship-game && unset ARGV0 && cargo test --lib lamp_gain_falls_off_by_20_m`.
- [ ] `place_lamps_caps_at_eight` feeds 10 cabin rooms and asserts 8 lamps, each at a room centroid. Run `cd ship-game && unset ARGV0 && cargo test --lib place_lamps_caps_at_eight`.
- [ ] `cd ship-game && unset ARGV0 && cargo test --lib` passes.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Dolly into a cabin on trunk and on the head. Save `lamp-regression-trunk.png` and Save `lamp-regression-head.png`. Pass when the head center has at least 80 pixels with red greater than green, green greater than blue, and red above 170, and the trunk center has fewer than 20 such pixels. If trunk already has 80, record that and still require at least 500 ship pixels.
- [ ] Lane 2. Default camera. Save `lamp-default.png`. Pass when center has at least 500 ship pixels and fewer than 80 warm pixels, because the default view is outside.
- [ ] Lane 3. Dolly into a cabin on the sun side. Save `lamp-cabin-sun.png`. Pass when center has at least 80 warm pixels as defined in lane 1.
- [ ] Lane 4. Dolly into a cabin on the shaded side. Save `lamp-cabin-shade.png`. Pass when center has at least 80 warm pixels and mean luminance stays above 25 percent of the lane 3 cabin.
- [ ] Lane 5. Dolly to an open top deck. Save `lamp-open-deck.png`. Pass when warm pixels in center are fewer than 20.
- [ ] Lane 6. Dolly out to the default distance. Save `lamp-far.png`. Pass when sea pixels are at least 2 percent of the frame and topBand blue standard deviation is at least 8.
- [ ] Lane 7. Page Up through two decks and dolly into a room. Save `lamp-upper-room.png`. Pass when the status deck name changed and the console has no `panic`.
- [ ] Lane 8. Page Down to the lowest deck and dolly in. Save `lamp-lower.png`. Pass when center has at least 500 ship pixels.
- [ ] Lane 9. Press P over a cabin deck. Save `lamp-plan.png`. Pass when the status line contains `plan` and center has at least 500 ship pixels.
- [ ] Lane 10. Open the keymap with K and close it. Save `lamp-keymap.png`. Pass when the keymap panel is gone, the canvas still shows the ship, and the console has no `panic`.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Median frame delta and wasm size, on trunk and on the head. The added work is an 8-wide loop in `daylight`. The end state is a warm cabin patch with the exterior still sunlit.
- [ ] Probe. Measure trunk, then the head, then trunk again, 180 frames after `crew` at the default camera. Discard the pair when the two trunk medians differ by more than 1 ms.
- [ ] Baseline. Record the first trunk median and the first trunk wasm size.
- [ ] Rule. Fail when the head median exceeds the first trunk median by more than 2 ms, or when the head wasm exceeds the first trunk wasm by more than 32768 bytes.

**Review gate.** None. vr-lamps does not change a key, a click, or a camera binding.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Restacked onto vr-glass, with `git patch-id` unchanged.
- [ ] Hand vr-lamps to the operator at merge-ready. Do not merge it.

## Shade pool water with the sea shader, vr-pools

**Depends on.** vr-lamps.

**Files.**

- [ ] Edit the color passed by `pool` in `ship-game/src/venue_props.rs` from `WATER` to a new `POOL` constant. Leave `waste_plant`, `slides`, `trees`, and every mesh size unchanged.
- [ ] Create `split_pool_mesh` in `ship-game/src/sea.rs`.
- [ ] Edit the `venue_mesh` spawn arm in `spawn_ship_once` so pool triangles use `SeaMaterial`.

**Build.**

- [ ] Set `POOL` to rgba `0.05, 0.55, 0.62, 1.0`. Keep `WATER` at rgba `0.12, 0.48, 0.78, 1.0` for the wastewater tank tops.
- [ ] `split_pool_mesh` returns the original triangles plus a second mesh of triangles whose vertex colors match `POOL`. Positions of both meshes match the input.
- [ ] Spawn the `POOL` mesh with `SeaMaterial` and the rest with `ShipClipMaterial`. The sea time uniform drives the pool highlight.

**You see.**

- [ ] Pool water has a moving highlight.
- [ ] Wastewater tank tops stay flat.
- [ ] Slide flumes stay where the room work put them.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] `split_pool_mesh_keeps_tank_water` builds one mesh with a `POOL` quad and a `WATER` quad. The sea mesh contains the `POOL` vertices and the clip mesh contains the `WATER` vertices. Positions match the input. Run `cd ship-game && unset ARGV0 && cargo test --lib split_pool_mesh_keeps_tank_water`.
- [ ] `cd ship-game && unset ARGV0 && cargo test --lib` passes.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Page to the deck whose status name contains `Pool`, on trunk and on the head, and dolly in. Save `pool-regression-trunk.png` and Save `pool-regression-head.png`. Pass when a pool patch on the trunk has blue standard deviation under 4 and the same patch on the head has blue standard deviation at least 8. If the status line never contains `Pool`, fail the lane and record the deck names that did appear.
- [ ] Lane 2. That same pool framing. Save `pool-default.png`. Pass when the pool patch blue standard deviation is at least 8 and center has at least 500 ship pixels.
- [ ] Lane 3. Orbit so the pool reflects toward the sun. Save `pool-sun.png`. Pass when the pool patch max luminance exceeds its min luminance by at least 15 percent.
- [ ] Lane 4. Orbit to the opposite side of the pool. Save `pool-shade.png`. Pass when the pool patch is still not flat. Blue standard deviation is at least 6.
- [ ] Lane 5. Save `pool-wave-a.png`, wait 500 ms, and Save `pool-wave-b.png`. Pass when the pool patch mean absolute difference is at least 2 levels.
- [ ] Lane 6. Dolly out until the whole pool deck is in frame. Save `pool-far.png`. Pass when the pool patch still has blue standard deviation at least 6 and sea pixels outside the hull are at least 1 percent of the frame.
- [ ] Lane 7. Page Up to the park deck. Save `pool-park.png`. Pass when tree blocks are still in frame and the console has no `panic`.
- [ ] Lane 8. Page to the deck that holds the wastewater plant and frame a tank top. Save `pool-tank.png`. Pass when that tank patch has blue standard deviation under 4.
- [ ] Lane 9. Page to the water park and frame the flumes. Save `pool-slides.png`. Pass when the flume pixels are still present and their centroids moved by under 4 pixels relative to the trunk shot of the same pose.
- [ ] Lane 10. Press P on the pool deck. Save `pool-plan.png`. Pass when the status line contains `plan` and the pool area is not the flat trunk color.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Median frame delta and wasm size, on trunk and on the head. The added work is a second draw of the pool triangles with `SeaMaterial`. The end state is a pool with a highlight and a flat tank top.
- [ ] Probe. Measure trunk, then the head, then trunk again, 180 frames after `crew` while the pool deck fills the frame. Discard the pair when the two trunk medians differ by more than 1 ms.
- [ ] Baseline. Record the first trunk median and the first trunk wasm size.
- [ ] Rule. Fail when the head median exceeds the first trunk median by more than 2 ms, or when the head wasm exceeds the first trunk wasm by more than 32768 bytes.

**Review gate.** None. vr-pools does not change a key, a click, or a camera binding.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Restacked onto vr-lamps, with `git patch-id` unchanged.
- [ ] Hand vr-pools to the operator at merge-ready. Do not merge it.

## Close the program

- [ ] Every box above is checked with its evidence.
- [ ] Reply to the operator with each pull request id, its verdict, and the head SHA. List nothing as merged.

## Appendix A. Prototype evidence

No throwaway prototype was built. The operator asked for a plan and forbade repo code changes. The open rendering questions were facts in the current client and in Bevy 0.18.1, so a scratch scene would not have changed the choice.

Sky. `Atmosphere` in `bevy_pbr-0.18.1` `src/atmosphere/mod.rs` returns before it loads when `DownlevelFlags::COMPUTE_SHADERS` is missing. `ship-game/Cargo.toml` enables the `webgl2` feature, and `scripts/build-ship.sh` builds `wasm32-unknown-unknown`. The verified surface is that wasm page. vr-sky therefore builds a cubemap with `Skybox`.

Fog. `DistanceFog` in `bevy_pbr-0.18.1` `src/fog.rs` affects `StandardMaterial`. The hull uses `ShipClipMaterial`. vr-sky mixes fog inside `daylight`.

Water material. `spawn_ship_once` on `origin/main` clones one `SharedClipMaterial` onto every ship mesh. `ship_clip_forward.wgsl` multiplies a deck texture and fades fragments above the deck cut. A sea that shares that handle would wear the deck checker. vr-sea adds `SeaMaterial`.

Shadows. `daylight` has no shadow sample. The prepass fragment in `ship_clip_prepass.wgsl` returns an empty `FragmentOutput`. The app never inserts `DepthPrepass`, so that shader is not requested. `DirectionalLight` at `app_3d.rs` sets illuminance and does not set `shadows_enabled`. vr-shadow rasterizes deck polygons on the CPU.

What stays unproven. The exact horizon colors, the wave amplitudes 0.08 and 0.05, and whether a 12 percent luminance gap reads as a shadow at the merged camera distance. The live lanes measure those pictures. The operator can ask for a paler sky, a flatter sea, or a darker shadow, in their own words.

## Appendix B. Alternatives rejected

Bevy `Atmosphere` stays out. The plugin needs compute shaders, and the page the player loads is WebGL2. A native-only sky would pass on `make run-ship-native` and miss on `https://timehelm.net/`.

`DistanceFog` on the camera stays out for the hull. Crew glTFs may pick it up later. This program does not retune crew lighting.

Bevy directional shadow maps stay out. The clip prepass does not write a usable depth, and `daylight` ignores illuminance. Turning on `shadows_enabled` would light the crew and leave the decks flat.

Loading `bedDouble.glb`, plant glTFs, or `assets/Textures/texture-*.png` stays out. Pull request 14 already builds beds, toilets, trees, and workshops as meshes. Swapping those meshes repeats that work. The character glTFs stay the only scene loads.

A bow wake stays out. The ship is fixed in the world. A wake would show motion the ship does not have.

Night, weather, and funnel smoke stay out. Pull request 11 shipped one sun. These seven pull requests finish that daylight picture.

Rebuilding slide flumes, doors, or the hull plate stays out. vr-pools shades pool triangles and leaves `slides` and `ship_hull_plate` alone.

## Appendix C. Risks

vr-sky. The cubemap can pop in after the first frame while `ClearColor` still shows. The owner watches lane 10 for a panic and for the atmosphere warning. Leave HDR off. Turning it on would run the tonemap pass trunk does not run, and the pixel predicates would stop matching.

Plan view. P is bound in `viewer_input_map` and missing from `isGameKey`. A lane that sends P to the window fails even when the plan view works. Focus the canvas first.

The ship material is `AlphaMode::Blend`. Fragments above the deck cut stay submitted at alpha 0. Leave that blend mode unchanged in these pull requests. The frame-time budget sits on top of that cost.

vr-deck. `deck_geometry.rs` also holds `ship_hull_plate` on the interior branch. The owner edits `procedural_deck_plan_texture_image` and stops before that function. Walls use uv 0, 0 today, so the pattern weight is what keeps planks off the bulkheads.

vr-sea. The hull plate is a thin slab around each deck, about 0.18 m thick, not a shell down to the water. `sea_skirt_mesh` covers the gap under deck 0. If the merged outline function moves, the skirt uses that function and does not copy the plate builder. Wave amplitudes that exceed the unit-test cap will intersect the deck 0 plate.

vr-shadow. The grid is spawn-time. Animated doors and walking crew do not move the shadow. A wrong `to_sun.z` near 0 would explode the offset. The existing sun test requires `to_sun.z > 0.45`, and the owner keeps that test green.

vr-glass. Balcony quads can z-fight the bulwark. The 0.05 m outboard push is the fix the owner checks in lane 3 and lane 4. Decks below 5 stay solid on purpose.

vr-lamps. Eight lamps cannot cover every cabin. The cap is the WebGL uniform size the owner will not raise without a new perf rule. Rooms past the cap stay on daylight plus the shadow term.

vr-pools. `WATER` is also the tank-top color inside `waste_plant`. The split keys off `POOL` so tanks stay flat. If `pool` stops using a unique color, `split_pool_mesh_keeps_tank_water` fails. Slide flumes are not water. Lane 9 checks they did not move.

Every pull request touches `ship-game/src/app_3d.rs`. The stack order is what keeps those edits from landing side by side. A child that branches from `main` will conflict with pull request 14 while that pull request is open. Branch that child from its parent.

Frame-time numbers in the perf rules are budgets. This plan did not measure a frame. The owner throws out a comparison when the two trunk medians differ by more than 1 ms, then measures again. Seven pull requests at a 2 ms budget can consume most of a 16 ms frame. That tightness is intentional. A 512 sea grid should fail vr-sea.

`origin/main` does not contain the poteto-mode playbooks. The arm step already says to open the plugin copy when `git show` misses.

The live lanes need a database world. Without `crew` in the status text the ship never spawns, and a screenshot of clear color is not evidence.

## Appendix D. Links and reading list

Read these on `origin/main` at `6dbf853f72b638b7bfeedeffeb1982aeb1e17724` before editing.

`ship-game/src/app_3d.rs` holds `ShipClipMaterial`, `setup`, `spawn_ship_once`, `sync_clip`, and `directional_light_sun_shades_decks_and_bulkheads`.

`ship-game/src/shaders/ship_clip_forward.wgsl` holds `daylight` and the world-space deck sample.

`ship-game/src/shaders/ship_clip_prepass.wgsl` must stay in struct lockstep with the forward shader.

`ship-game/src/deck_geometry.rs` holds `procedural_deck_plan_texture_image`.

`ship-game/src/shader_embed.rs` is where a new shader is registered.

`sim/src/hull.rs` holds `SHIP_LENGTH_M` and `SHIP_BEAM_M`. On this SHA they are 318 and 60. Pull request 14 changes the outline. Read the merged values at program start and pass them through `plan_size`.

Pull request 11, merged, is the sun term in `daylight`. Pull request 12, merged, keeps the camera level on deck changes. Pan is W and S, strafe is Q and D, and orbit is A and E. Pull request 14 is still `cursor/ship-interior-realism-92d3`.

vr-sea and vr-shadow read `pstack/skills/how/SKILL.md` before editing. No pull request in this program requires `pstack/skills/interrogate/SKILL.md`.

The execution trail follows `pstack/skills/show-me-your-work/SKILL.md`. The hourly tick writes one row. Perf probes follow `pstack/skills/benchmark-checklist/SKILL.md` and `pstack/skills/principle-explain-the-number/SKILL.md`.

Boot the viewer with `make run-ship` and open `http://127.0.0.1:8080/3d/`. The canvas id is `ship-game-canvas`.
