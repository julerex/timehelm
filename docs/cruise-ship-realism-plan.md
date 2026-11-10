# Cruise ship architecture plan

The Timehelm viewer should match Icon of the Seas in hull width, deck structure, and the spaces a guest can stand in. PR 14 already built the neighborhood rectangles, doors, cabins, galleys, workshops, waste rooms, pools, slides, lifeboats, and the park. This program corrects the structure those rectangles still get wrong.

A space counts only when `sim` stores its deck, its plan rectangle, and its job. A mesh in `venue_props.rs` does not count. Crew schedules stay in the life simulation plan. Shaders and materials stay in the visual plan.

Work starts on branch `cursor/ship-interior-realism-92d3`. The order is TH-ARCH-1, TH-ARCH-2, TH-ARCH-3, TH-ARCH-4, TH-ARCH-5, TH-ARCH-6, TH-ARCH-7, TH-ARCH-8.

## How to read this

One box is one unit of work. Every box names the evidence that checks it. A nested box is a sub-step of the box above it. Check a box only when its evidence exists, a file, a log line, a screenshot, a test run, or a SHA. The body is a how-to. The appendices explain and record.

Paths are relative to the timehelm repo. The viewer is a fly camera. WASD pans, Page Up and Page Down change deck, and P toggles the plan view. There is no walking character in this program.

The program runs `pstack/skills/poteto-mode/playbooks/autopilot-stack.md`. Owners stop at merge-ready. The operator lands TH-ARCH-1 through TH-ARCH-8 bottom-up. No owner merges.

Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

## Program checklist

### Arm the program

- [ ] State the protocol and this plan to the operator, then stop. Start execution only on the operator's explicit go.
- [ ] Read these from trunk at program start. Re-read them at every tick.
  - [ ] `git show origin/main:pstack/skills/poteto-mode/playbooks/autopilot-stack.md`
  - [ ] `git show origin/main:pstack/skills/swarm/SKILL.md`
  - [ ] `git show origin/main:pstack/skills/control-ui/SKILL.md`
  - [ ] `git show origin/main:pstack/skills/poteto-mode/playbooks/opening-a-pr.md`
  - [ ] `git show origin/main:pstack/skills/how/SKILL.md`
  - [ ] `git show origin/main:pstack/skills/interrogate/SKILL.md`
  - [ ] `git show origin/main:pstack/skills/show-me-your-work/SKILL.md`
- [ ] On the operator's go, arm the audit tick as `/loop 1h` with the tick prompt below. Never leave the cadence to memory.
- [ ] Use this tick prompt, verbatim.

```text
Re-read the execution playbook from trunk. Audit the operation against it and fix drift in this tick. Probe every active lane and judge progress by side effects only. Stand down a stuck lane and dispatch its replacement now. Then post a short status message to the operator in chat only when the audit found a tracked change that no earlier status message reported, such as a PR opened, a code-ready head, a round launched or closed, a verdict, a merge, a stuck agent and the action taken, a blocker added or cleared, or a decision only the operator can make. Name every such change and nothing else. Do not repeat a table, the merged list, or an unchanged blocker. If the audit found none, end the turn with no reply text. Either way, log this tick's row in your decision trail. The row names the items reported, or none.
```

- [ ] On the operator's hold or stand-down, send every owner a zero-writes order at once.

### Spawn owners

- [ ] Spawn one owner per PR with the full lifecycle the execution playbook names.
- [ ] Follow this dependency graph. Base each PR on its parent branch. Autopilot-stack does not wait for a merge before the next branch exists.
  - [ ] TH-ARCH-1 branches from `cursor/ship-interior-realism-92d3`.
  - [ ] TH-ARCH-2 after TH-ARCH-1.
  - [ ] TH-ARCH-3 after TH-ARCH-2.
  - [ ] TH-ARCH-4 after TH-ARCH-3.
  - [ ] TH-ARCH-5 after TH-ARCH-4.
  - [ ] TH-ARCH-6 after TH-ARCH-5.
  - [ ] TH-ARCH-7 after TH-ARCH-6.
  - [ ] TH-ARCH-8 after TH-ARCH-7.
- [ ] Hold the file boundaries. TH-ARCH-1 touches `sim/src/hull.rs` and the band offsets in `sim/src/layout.rs`. TH-ARCH-2 touches `sim/src/hull.rs` and `ship-game/src/deck_geometry.rs`. TH-ARCH-3 touches `place_cabins` and `berth_row` in `sim/src/layout.rs`. TH-ARCH-4 touches `sim/src/zones.rs`, `sim/src/lib.rs`, `Door` in `sim/src/world.rs`, and `doors_for` in `sim/src/layout.rs`. TH-ARCH-5 touches `sim/src/cores.rs`, `sim/src/lib.rs`, and `place_stairs` in `sim/src/layout.rs`. TH-ARCH-6 touches `machinery` in `sim/src/layout.rs`. TH-ARCH-7 touches lifeboat bays and the gangway in `sim/src/layout.rs`, and `lifeboats` in `ship-game/src/venue_props.rs`. TH-ARCH-8 touches holes in `sim/src/hull.rs`.
- [ ] Hold the review gate. TH-ARCH-1, TH-ARCH-2, TH-ARCH-3, TH-ARCH-4, TH-ARCH-5, TH-ARCH-6, TH-ARCH-7, and TH-ARCH-8 change the picture in the viewer. They wait for the operator's review in chat with screenshots and a video before merge.

### PR mechanics, for every PR

- [ ] Resolve the forge once. Default to `gh`. If `command -v origin` succeeds and Origin can resolve the repository, use `origin pr` for every PR operation. Record any fallback to `gh`. Never require `gt`.
- [ ] Open the PR ready, never draft, per Opening a PR. Use the run's built-in PR tool when it has one, else `origin pr create --status open --base <base-branch>` or `gh pr create --base <base-branch>` according to the resolved forge. A stack child targets its parent branch.
- [ ] Run `make lint` and `make test` once before the PR-facing push. The Makefile clears `ARGV0` before cargo. Push with hooks on.
- [ ] Run `/deslop` before each commit and `/no-comments` before review.
- [ ] Triage every Bugbot and security-reviewer comment per `pstack/skills/poteto-mode/references/bugbot-triage.md`.
- [ ] Rebase onto the parent head before the code-ready report and babysit. TH-ARCH-1 rebases onto `cursor/ship-interior-realism-92d3` until that branch is on main, then onto main. Keep that merge base in fix rounds. Rebase again only at merge prep, on a `git merge-tree` conflict with the parent, on a CI failure that comes from a change on the parent, or when a known concurrent change to the same files lands.

### Verdict and merge, for every PR

- [ ] A full swarm runs once, at the code-ready head. Another full swarm runs only for new behavior, a redesign, or a large new diff. A rebase, a merge-conflict resolution, or a minor fix gets a single verifier at the new head, and a wording-only change gets none. Run each full swarm per `pstack/skills/swarm/SKILL.md`. One gates lane. The ten live lanes from the PR's Verify, live block. The perf lane from its Verify, perf block. Two or more audit lanes, each with its own focus, that read the diff and the receipts and distrust the PR body. The root audits the receipts in the merge-ready report before the verdict.
- [ ] Judge every finding by the execution playbook's blocking rule. The verdict is clean when none blocks. A failure on trunk behavior in a lane marked `Regression lane against trunk.` is recorded against trunk and does not block the PR. A new head gets a fresh verdict, except for results that stay valid under the patch-id rule in `playbooks/shipping.md`.
- [ ] Stop when the PR is verified and green at its current head. Hand it to the operator. After the handoff, nobody pushes without the operator's go. The operator lands the stack bottom-up. A PR whose `git patch-id` a restack leaves unchanged keeps its verdict.

### Boot recipe, for every live lane

Each live lane runs on its own cloud VM at the PR head. Drive through `control-ui` from `cursor-team-kit`.

- [ ] `git fetch origin <head-branch> && git checkout <head SHA>`.
- [ ] From the repo root, run `make run-ship`. Wait until `http://127.0.0.1:8080/` returns 200 and the canvas `#ship-game-canvas` is not one flat color. Postgres is not required. The layout is compiled into the wasm.
- [ ] Drive input only through `control-ui`. Click `#ship-game-canvas`. Send Page Up, Page Down, P, and WASD. Read the browser console and the document response. Do not click guessed room coordinates.
- [ ] Save every screenshot to `/tmp/swarm-<pr-id>/worker-<n>/<slug>.png` and return the paths with the report.

## Set the three published beams (TH-ARCH-1)

**Depends on.** PR 14, branch `cursor/ship-interior-realism-92d3`.

**Files.**

- [ ] Edit `sim/src/hull.rs`.
- [ ] Edit band offsets in `sim/src/layout.rs`.
- [ ] Edit beam samples in `sim/src/hull.rs` tests and in `ship-game/src/deck_geometry.rs` tests when a sample sits on the old shell.

**Build.**

- [ ] Add `WATERLINE_BEAM_M` at 48.5, `DECKHOUSE_BEAM_M` at 57.8, and set `SHIP_BEAM_M` to 65.8 in `sim/src/hull.rs`. Add `beam_for_deck`. Decks 0 and 1 return 48.5. Deck 12 returns 65.8. Every other deck returns 57.8. A stepped outline on decks 15 through 19 may be narrower than that cap and must not be wider.
- [ ] Keep the aft-starboard origin on the widest deck. Inset each outline by half the difference between 65.8 and `beam_for_deck`. Shift room rectangles by the same inset in `sim/src/layout.rs`.
- [ ] Point tests that still treat `deck_hull_polygon` as the waterline at `deck_hull(0)`. Keep `point_on_deck(14, park_center)` true. Keep `central_park_is_open_above_its_floor` and `sun_deck_and_slide_tops_step_in` green.

**You see.**

- [ ] Plan view on deck 0 is narrower than plan view on deck 9. Plan view on deck 12 is the widest of the three.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] `three_beams` expects 48.5, 57.8, and 65.8, and `build_ship` still returns rooms inside `point_on_deck`. Run `cd sim && unset ARGV0 && cargo test --lib three_beams -- --exact`.
- [ ] Run `cd sim && unset ARGV0 && cargo test --lib` and require `icon_neighborhoods_match_the_public_plans`, `bow_is_finer_than_the_midship`, `central_park_is_open_above_its_floor`, and `sun_deck_and_slide_tops_step_in` to pass.
- [ ] Run `cd ship-game && unset ARGV0 && cargo test --lib hull_plate_reaches_the_bow_and_accepts_the_park_hole -- --exact`.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Open plan view on deck 0 at trunk and at the head. Trunk is the 318 m ship with one beam and the name Engine Deck. Record that, and gate the head waterline sitting inboard of deck 9. Save `th-arch-1-trunk.png`. Pass when the head deck 0 plate is visibly narrower than the head deck 9 plate and the trunk shot is the old single outline.
- [ ] Lane 2. Plan view, deck 0, after the rooms have shifted. Save `th-arch-1-deck0.png`. Pass when the generator hall and both service alleys sit inside the outline with a margin of deck, not clipped by it.
- [ ] Lane 3. Plan view, deck 9. Save `th-arch-1-deck9.png`. Pass when the deckhouse is wider than deck 0 and the park hole is still open.
- [ ] Lane 4. Plan view, deck 12. Save `th-arch-1-bridge.png`. Pass when this plate is wider than deck 9 and the bridge room is still at the forward end.
- [ ] Lane 5. Dolly toward the bow in the 3D view. Save `th-arch-1-bow.png`. Pass when the bow is still a fine entry, not a rectangular slab.
- [ ] Lane 6. Dolly toward the stern. Save `th-arch-1-stern.png`. Pass when the transom quarters are still rounded.
- [ ] Lane 7. Plan view, deck 19. Save `th-arch-1-slides.png`. Pass when the slide-top plate is still stepped in from the deckhouse and is not as wide as deck 12.
- [ ] Lane 8. Page Up until the deck label reads Deck 14. Save `th-arch-1-no-13.png`. Pass when the label is Deck 14 Spa and no step is named Deck 13.
- [ ] Lane 9. Plan view, deck 8. Save `th-arch-1-park-floor.png`. Pass when the park floor is solid and wider than deck 0.
- [ ] Lane 10. Plan view, deck 1. Save `th-arch-1-utility.png`. Pass when laundry, the workshops, and the waste room sit inside the 48.5 m outline.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Wall-clock seconds of `cargo test --lib --release` in `sim` on trunk and on the head, and seconds from HTTP 200 until `#ship-game-canvas` shows more than one color.
- [ ] Probe. Interleave `cd sim && unset ARGV0 && cargo test --lib --release` on trunk and on the head, recording `date +%s` before and after. Build the viewer once with `make run-ship`, then reload `http://127.0.0.1:8080/` and time the canvas. Both sides must produce both numbers.
- [ ] Baseline. Record the trunk test seconds and the trunk paint seconds first.
- [ ] Rule. Fail when the head test time is above 1.5 times the trunk test time, or above 180 seconds. Fail when head paint is more than 5 seconds slower than trunk paint, or more than 20 seconds after HTTP 200.

**Review gate.** The operator reviews before merge.

- [ ] Copy the lane 3 screenshot into `/cursor/stores/self/media/th-arch-1-review.png`.
- [ ] Record a 30 to 60 second video of plan view stepping from deck 0 to deck 9 to deck 12. Save it as `/cursor/stores/self/media/th-arch-1-review.mp4`.
- [ ] Post the screenshots and the video in chat. Stop at merge-ready. Wait for the operator.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Rebased onto the parent with patch-id unchanged.
- [ ] The owner stops. The operator lands this PR in stack order.

## Set draft and the inner bottom (TH-ARCH-2)

**Depends on.** TH-ARCH-1.

**Files.**

- [ ] Edit `sim/src/hull.rs`.
- [ ] Edit `ship-game/src/deck_geometry.rs` so the section shows the waterline and the tank top.

**Build.**

- [ ] Add `DRAFT_M` at 9.25 in `sim/src/hull.rs`. Add `INNER_BOTTOM_M` from the minimum height in SOLAS chapter II-1 regulation 9, using `WATERLINE_BEAM_M` as the breadth. Leave `DECK_COUNT` at 20. Leave `DECK_HEIGHT_M` at 3.0.
- [ ] Draw the keel, the tank top, and the loaded waterline on the side elevation in `ship-game/src/deck_geometry.rs`. The machinery floor sits on the tank top. There is no 21st deck name.

**You see.**

- [ ] A side view shows blue waterline, a tank top under the machinery floor, and the same 20 deck names as before.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] `draft_and_inner_bottom` expects `DRAFT_M` of 9.25 and an inner bottom strictly between the keel and the machinery floor. Run `cd sim && unset ARGV0 && cargo test --lib draft_and_inner_bottom -- --exact`.
- [ ] Run `cd sim && unset ARGV0 && cargo test --lib` and require `three_beams` and `icon_neighborhoods_match_the_public_plans` to pass.
- [ ] Run `cd ship-game && unset ARGV0 && cargo test --lib hull_plate_reaches_the_bow_and_accepts_the_park_hole -- --exact`.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Show the side elevation at trunk and at the head. Trunk has no draft mark. Record that, and gate the head waterline and tank top. Save `th-arch-2-trunk.png`. Pass when the head shows three horizontal lines the trunk shot lacks.
- [ ] Lane 2. Side view at the bow. Save `th-arch-2-bow.png`. Pass when the waterline crosses the bow below the gangway deck.
- [ ] Lane 3. Side view at the stern. Save `th-arch-2-stern.png`. Pass when the waterline meets the transom and the tank top continues aft.
- [ ] Lane 4. Plan view, deck 0. Save `th-arch-2-deck0.png`. Pass when the machinery rooms are unchanged in plan and still inside the narrow hull.
- [ ] Lane 5. Page Down until the deck no longer changes. Save `th-arch-2-lowest.png`. Pass when the lowest label is Machinery and the tank top is visible below that floor.
- [ ] Lane 6. Plan view, deck 12. Save `th-arch-2-bridge.png`. Pass when the bridge is still the widest deck.
- [ ] Lane 7. Plan view, deck 9. Save `th-arch-2-park.png`. Pass when the park hole is still open.
- [ ] Lane 8. Side view amidships. Save `th-arch-2-section.png`. Pass when the machinery floor is above the tank top and below the deck 2 floor.
- [ ] Lane 9. Dolly below the keel. Save `th-arch-2-keel.png`. Pass when the shell is closed and there is no extra walkable deck under Machinery.
- [ ] Lane 10. Plan view, deck 19. Save `th-arch-2-slides.png`. Pass when the slide tops are still stepped in.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Wall-clock seconds of `cargo test --lib --release` in `sim` on trunk and on the head, and seconds from HTTP 200 until `#ship-game-canvas` shows more than one color.
- [ ] Probe. Interleave `cd sim && unset ARGV0 && cargo test --lib --release` on trunk and on the head, recording `date +%s` before and after. Build the viewer once with `make run-ship`, then reload `http://127.0.0.1:8080/` and time the canvas. Both sides must produce both numbers.
- [ ] Baseline. Record the trunk test seconds and the trunk paint seconds first.
- [ ] Rule. Fail when the head test time is above 1.5 times the trunk test time, or above 180 seconds. Fail when head paint is more than 5 seconds slower than trunk paint, or more than 20 seconds after HTTP 200.

**Review gate.** The operator reviews before merge.

- [ ] Copy the lane 8 screenshot into `/cursor/stores/self/media/th-arch-2-review.png`.
- [ ] Record a 30 to 60 second video of the side elevation from bow to stern. Save it as `/cursor/stores/self/media/th-arch-2-review.mp4`.
- [ ] Post the screenshots and the video in chat. Stop at merge-ready. Wait for the operator.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Rebased onto the TH-ARCH-1 head with patch-id unchanged.
- [ ] The owner stops. The operator lands this PR in stack order.

## Size the cabin bays (TH-ARCH-3)

**Depends on.** TH-ARCH-2.

**Files.**

- [ ] Edit `place_cabins` and `berth_row` in `sim/src/layout.rs`.

**Build.**

- [ ] Split each outboard berth into an interior of 18.9 square metres and a balcony of 4.6 square metres. Set each park-side berth to 17.4 square metres. Keep the alley between them. Keep the shared alley edge longer than 1.2 m so `cabin_marks` still draws a door leaf.
- [ ] Leave the sampled heads where they are. `place_heads` puts two heads on an outboard side, not one head per berth. The deck 9 head whose polygon reaches x 45.6 must stay, because `saved_layout_matches` in `server/src/db.rs` requires it.
- [ ] Stack one suite on the same plan footprint across three suite decks that already use the name Suite. Keep decks 5 and 6 free of berths. Keep deck index 18 free of berths. Every new rectangle needs a door onto a room that can already reach a stair. `every_kind_sits_inside_its_hull` fails when any room is disconnected.
- [ ] Print the berth count from `build_ship`. Do not fail the test for missing 2805. Fail it when a standard balcony interior is more than 0.5 square metres off 18.9.

**You see.**

- [ ] Deck 9 plan view shows a thin balcony strip outboard of a smaller berth, and a second band of smaller rooms beside the park hole.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] `balcony_berth_area` checks the 18.9 and 4.6 split, the 17.4 park-side berth, zero berths on decks 5 and 6, and one three-deck suite footprint. Run `cd sim && unset ARGV0 && cargo test --lib balcony_berth_area -- --exact`.
- [ ] Run `cd sim && unset ARGV0 && cargo test --lib every_kind_sits_inside_its_hull -- --exact`.
- [ ] Run `cd server && unset ARGV0 && cargo test --lib a_saved_world_must_match_the_icon_deck_plan -- --exact`.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Open deck 9 at trunk and at the head. Trunk berths are the old working-ship grid. Record that, and gate the balcony strip. Save `th-arch-3-trunk.png`. Pass when the head shows two bands and a balcony strip the trunk shot does not.
- [ ] Lane 2. Plan view, deck 9, starboard side. Save `th-arch-3-balcony.png`. Pass when a balcony cell sits outboard of the berth and inboard of the shell.
- [ ] Lane 3. Plan view, deck 9, beside the park hole. Save `th-arch-3-park-cabins.png`. Pass when those berths face the hole and are smaller than the balcony interiors.
- [ ] Lane 4. Plan view, deck 5. Save `th-arch-3-promenade.png`. Pass when the promenade has no berth rectangles.
- [ ] Lane 5. Plan view, deck 6. Save `th-arch-3-promenade-upper.png`. Pass when this deck also has no berths.
- [ ] Lane 6. Plan view, deck index 18. Save `th-arch-3-grove.png`. Pass when that deck has no berths.
- [ ] Lane 7. Plan view on the three suite decks, same camera x and y. Save `th-arch-3-suite.png`. Pass when the suite footprint lands on the same plan point on each of the three decks.
- [ ] Lane 8. Plan view, deck 9, the alley. Save `th-arch-3-alley.png`. Pass when a corridor still separates the outboard berths from the park-side berths.
- [ ] Lane 9. Plan view, deck 3. Save `th-arch-3-medical.png`. Pass when the room named Medical Center is still on this deck.
- [ ] Lane 10. Zoom on one outboard door. Save `th-arch-3-door.png`. Pass when the door is on the alley edge, not on the shell.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Wall-clock seconds of `cargo test --lib --release` in `sim` on trunk and on the head, and seconds from HTTP 200 until `#ship-game-canvas` shows more than one color.
- [ ] Probe. Interleave `cd sim && unset ARGV0 && cargo test --lib --release` on trunk and on the head, recording `date +%s` before and after. Build the viewer once with `make run-ship`, then reload `http://127.0.0.1:8080/` and time the canvas. Both sides must produce both numbers.
- [ ] Baseline. Record the trunk test seconds and the trunk paint seconds first.
- [ ] Rule. Fail when the head test time is above 1.5 times the trunk test time, or above 180 seconds. Fail when head paint is more than 5 seconds slower than trunk paint, or more than 20 seconds after HTTP 200.

**Review gate.** The operator reviews before merge.

- [ ] Copy the lane 2 screenshot into `/cursor/stores/self/media/th-arch-3-review.png`.
- [ ] Record a 30 to 60 second video panning along the deck 9 balcony strip. Save it as `/cursor/stores/self/media/th-arch-3-review.mp4`.
- [ ] Post the screenshots and the video in chat. Stop at merge-ready. Wait for the operator.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Rebased onto the TH-ARCH-2 head with patch-id unchanged.
- [ ] The owner stops. The operator lands this PR in stack order.

## Cut main vertical zones (TH-ARCH-4)

**Depends on.** TH-ARCH-3.

**Files.**

- [ ] Create `sim/src/zones.rs`.
- [ ] Edit `sim/src/lib.rs`.
- [ ] Edit `Door` in `sim/src/world.rs`.
- [ ] Edit `doors_for` in `sim/src/layout.rs`.

**Build.**

- [ ] Add zone cuts in `sim/src/zones.rs`. Each zone is at most 48 m in length and at most 48 m in width, and at most 1600 square metres on any one deck. Align a cut with a cabin-module edge from TH-ARCH-3. Below the bulkhead deck, use the same x positions.
- [ ] Add a class field on `Door`. A door whose rooms sit in different zones is class A-60. Other doors stay class B. Draw the A-60 cuts as lines on the plan view.
- [ ] Leave `RoomKind` unchanged. The Royal Theater may occupy a whole zone. A berth may not straddle a cut.

**You see.**

- [ ] Plan view shows transverse lines from shell to shell, and those lines land on corridor edges rather than through a berth.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] `zone_limits` checks length, width, and area, and checks that no berth polygon crosses a cut. Run `cd sim && unset ARGV0 && cargo test --lib zone_limits -- --exact`.
- [ ] Run `cd sim && unset ARGV0 && cargo test --lib`.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Open the promenade plan at trunk and at the head. Trunk has no zone lines. Record that, and gate the head lines. Save `th-arch-4-trunk.png`. Pass when the head promenade shows more than one transverse line and the trunk shot shows none.
- [ ] Lane 2. Plan view, deck 5, full length. Save `th-arch-4-promenade.png`. Pass when every zone along the promenade is shorter than the deckhouse beam is wide, and the lines reach both shells.
- [ ] Lane 3. Zoom on one cut at the shell. Save `th-arch-4-shell.png`. Pass when the line meets the shell and does not stop in open floor.
- [ ] Lane 4. Plan view, deck 9, one cabin bay. Save `th-arch-4-module.png`. Pass when the cut runs along a module gap and the berth rectangle is entirely on one side.
- [ ] Lane 5. Plan view, deck 0, then deck 5, same x. Save `th-arch-4-aligned.png`. Pass when a cut on deck 0 lands on the same x as a cut on deck 5.
- [ ] Lane 6. Plan view, deck 4, the Royal Theater. Save `th-arch-4-theater.png`. Pass when that room sits inside a single zone.
- [ ] Lane 7. Plan view, deck 9, the park hole. Save `th-arch-4-park.png`. Pass when zone lines remain on the cabin bands beside the hole.
- [ ] Lane 8. Zoom on an A-60 door. Save `th-arch-4-door.png`. Pass when the door on the cut is drawn heavier than a cabin door off the cut.
- [ ] Lane 9. Page to Deck 14. Save `th-arch-4-no-13.png`. Pass when the label is still Deck 14 and Deck 13 is absent.
- [ ] Lane 10. Plan view, deck 0, the generator hall. Save `th-arch-4-hall.png`. Pass when the hall does not contain a zone line through its interior.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Wall-clock seconds of `cargo test --lib --release` in `sim` on trunk and on the head, and seconds from HTTP 200 until `#ship-game-canvas` shows more than one color.
- [ ] Probe. Interleave `cd sim && unset ARGV0 && cargo test --lib --release` on trunk and on the head, recording `date +%s` before and after. Build the viewer once with `make run-ship`, then reload `http://127.0.0.1:8080/` and time the canvas. Both sides must produce both numbers.
- [ ] Baseline. Record the trunk test seconds and the trunk paint seconds first.
- [ ] Rule. Fail when the head test time is above 1.5 times the trunk test time, or above 180 seconds. Fail when head paint is more than 5 seconds slower than trunk paint, or more than 20 seconds after HTTP 200.

**Review gate.** The operator reviews before merge.

- [ ] Copy the lane 4 screenshot into `/cursor/stores/self/media/th-arch-4-review.png`.
- [ ] Record a 30 to 60 second video panning the promenade zone lines. Save it as `/cursor/stores/self/media/th-arch-4-review.mp4`.
- [ ] Post the screenshots and the video in chat. Stop at merge-ready. Wait for the operator.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Rebased onto the TH-ARCH-3 head with patch-id unchanged.
- [ ] The owner stops. The operator lands this PR in stack order.

## Separate stairs, elevators, and the escalator (TH-ARCH-5)

**Depends on.** TH-ARCH-4.

**Files.**

- [ ] Create `sim/src/cores.rs`.
- [ ] Edit `sim/src/lib.rs`.
- [ ] Edit `place_stairs` in `sim/src/layout.rs` so it reads `sim/src/cores.rs`.

**Build.**

- [ ] Define the aft core and the forward core in `sim/src/cores.rs`. Each core has a stair shaft and an elevator shaft on the same plan point through the decks that already have the atrium stairs. The forward shaft still stops before deck index 19.
- [ ] Add an escalator only on decks 2, 4, and 5. Leave `RoomKind` unchanged. A stair remains the escape. An elevator is not an escape.
- [ ] Keep one source. `place_stairs` copies the core footprints. It does not keep a second pair of magic x values.

**You see.**

- [ ] The promenade plan shows two stair shafts, an elevator beside each stair, and an escalator. Deck 8 shows the stairs and the elevators without the escalator.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] `cores_stack` checks that stair footprints match across decks, that elevators do not count in the escape set, and that the escalator exists only on decks 2, 4, and 5. Run `cd sim && unset ARGV0 && cargo test --lib cores_stack -- --exact`.
- [ ] Run `cd sim && unset ARGV0 && cargo test --lib zone_limits -- --exact`.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Open deck 5 at trunk and at the head. Trunk has a single stair footprint copied up the old ship. Record that, and gate two shafts plus elevators. Save `th-arch-5-trunk.png`. Pass when the head shows two stair shafts and the trunk shot shows the old single stair.
- [ ] Lane 2. Plan view, deck 5, aft shaft. Save `th-arch-5-aft.png`. Pass when the stair and a neighboring elevator are both drawn, and they are not the same rectangle.
- [ ] Lane 3. Plan view, deck 5, forward shaft. Save `th-arch-5-fwd.png`. Pass when the forward stair and its elevator are both drawn.
- [ ] Lane 4. Plan view, deck 2, then deck 5, same camera. Save `th-arch-5-stack.png`. Pass when the aft stair occupies the same plan point on both decks.
- [ ] Lane 5. Plan view, deck index 19. Save `th-arch-5-top.png`. Pass when the aft shaft is present and the forward shaft is absent.
- [ ] Lane 6. Plan view, deck 2. Save `th-arch-5-esc-2.png`. Pass when the escalator is visible on the gangway deck.
- [ ] Lane 7. Plan view, deck 4. Save `th-arch-5-esc-4.png`. Pass when the escalator is visible on the casino deck.
- [ ] Lane 8. Plan view, deck 5. Save `th-arch-5-esc-5.png`. Pass when the escalator meets the promenade.
- [ ] Lane 9. Plan view, deck 8. Save `th-arch-5-park.png`. Pass when stairs and elevators are present and the escalator is absent.
- [ ] Lane 10. Plan view, deck 3. Save `th-arch-5-medical.png`. Pass when Medical Center is still present and is not relabeled as a core.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Wall-clock seconds of `cargo test --lib --release` in `sim` on trunk and on the head, and seconds from HTTP 200 until `#ship-game-canvas` shows more than one color.
- [ ] Probe. Interleave `cd sim && unset ARGV0 && cargo test --lib --release` on trunk and on the head, recording `date +%s` before and after. Build the viewer once with `make run-ship`, then reload `http://127.0.0.1:8080/` and time the canvas. Both sides must produce both numbers.
- [ ] Baseline. Record the trunk test seconds and the trunk paint seconds first.
- [ ] Rule. Fail when the head test time is above 1.5 times the trunk test time, or above 180 seconds. Fail when head paint is more than 5 seconds slower than trunk paint, or more than 20 seconds after HTTP 200.

**Review gate.** The operator reviews before merge.

- [ ] Copy the lane 2 screenshot into `/cursor/stores/self/media/th-arch-5-review.png`.
- [ ] Record a 30 to 60 second video of Page Up from deck 2 through deck 8, holding the camera on the aft core. Save it as `/cursor/stores/self/media/th-arch-5-review.mp4`.
- [ ] Post the screenshots and the video in chat. Stop at merge-ready. Wait for the operator.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Rebased onto the TH-ARCH-4 head with patch-id unchanged.
- [ ] The owner stops. The operator lands this PR in stack order.

## Place the machinery, tanks, and pods (TH-ARCH-6)

**Depends on.** TH-ARCH-5.

**Files.**

- [ ] Edit `machinery` in `sim/src/layout.rs`.

**Build.**

- [ ] Replace the single gas-plant rectangle with two LNG tank footprints, each 27 m long and 8 m tall, so each tank rises through more than one 3 m deck. Keep them inside the deck 0 and deck 1 hull.
- [ ] Place six engine footprints inside the generator hall, three pod footprints under the transom and below the waterline, and five thruster footprints in the bow on deck 0.
- [ ] Keep Switchgear, Desalination, Wastewater and incinerator, and the workshops. Add one shore-connection recess on the starboard shell of deck 0. Leave `RoomKind` unchanged. Use the existing Engineering, Electrical, and FuelStore kinds.
- [ ] Leave `SOLAR_KW` at 90.0. `supply_kwh` still multiplies that constant. The new engine footprints do not feed it. Hotel power stays with the life simulation plan.

**You see.**

- [ ] Deck 0 plan view shows six engines, two long tanks, five bow thrusters, and three pods aft of the transom. A side view shows each tank taller than one deck.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] `machinery_block` expects six engines, two tanks of length 27, three pods, and five thrusters, and still finds Switchgear and the waste room. Run `cd sim && unset ARGV0 && cargo test --lib machinery_block -- --exact`.
- [ ] Run `cd sim && unset ARGV0 && cargo test --lib icon_neighborhoods_match_the_public_plans -- --exact`.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Open deck 0 at trunk and at the head. Trunk engineering is one room on the 318 m ship. Record that, and gate six engines and two tanks. Save `th-arch-6-trunk.png`. Pass when the head shows six separate engine footprints and two tanks.
- [ ] Lane 2. Plan view, deck 0, the hall. Save `th-arch-6-engines.png`. Pass when six engine footprints sit inside the old hall bounds.
- [ ] Lane 3. Plan view, deck 0, the tanks. Save `th-arch-6-tanks.png`. Pass when two footprints are each about 27 m along x and both are inside the hull.
- [ ] Lane 4. Side view through a tank. Save `th-arch-6-tank-height.png`. Pass when the tank extends above the machinery floor into the next deck.
- [ ] Lane 5. Stern view below the waterline. Save `th-arch-6-pods.png`. Pass when three pods sit under the transom and below the draft line from TH-ARCH-2.
- [ ] Lane 6. Plan view, the bow of deck 0. Save `th-arch-6-thrusters.png`. Pass when five thruster footprints sit forward of the generator hall and inside the bow.
- [ ] Lane 7. Plan view, Switchgear. Save `th-arch-6-switchgear.png`. Pass when the Switchgear room is still present beside the hall.
- [ ] Lane 8. Plan view, deck 1. Save `th-arch-6-waste.png`. Pass when Wastewater and incinerator, Desalination, laundry, and the three workshops are still on this deck.
- [ ] Lane 9. Plan view, starboard shell of deck 0. Save `th-arch-6-shore.png`. Pass when a recess breaks that shell for the shore connection.
- [ ] Lane 10. Plan view, deck 9. Save `th-arch-6-cabins.png`. Pass when the balcony strip from TH-ARCH-3 is unchanged.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Wall-clock seconds of `cargo test --lib --release` in `sim` on trunk and on the head, and seconds from HTTP 200 until `#ship-game-canvas` shows more than one color.
- [ ] Probe. Interleave `cd sim && unset ARGV0 && cargo test --lib --release` on trunk and on the head, recording `date +%s` before and after. Build the viewer once with `make run-ship`, then reload `http://127.0.0.1:8080/` and time the canvas. Both sides must produce both numbers.
- [ ] Baseline. Record the trunk test seconds and the trunk paint seconds first.
- [ ] Rule. Fail when the head test time is above 1.5 times the trunk test time, or above 180 seconds. Fail when head paint is more than 5 seconds slower than trunk paint, or more than 20 seconds after HTTP 200.

**Review gate.** The operator reviews before merge.

- [ ] Copy the lane 2 screenshot into `/cursor/stores/self/media/th-arch-6-review.png`.
- [ ] Record a 30 to 60 second video from the bow thrusters, across the tanks, to the pods. Save it as `/cursor/stores/self/media/th-arch-6-review.mp4`.
- [ ] Post the screenshots and the video in chat. Stop at merge-ready. Wait for the operator.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Rebased onto the TH-ARCH-5 head with patch-id unchanged.
- [ ] The owner stops. The operator lands this PR in stack order.

## Seat the lifeboats and open the gangway (TH-ARCH-7)

**Depends on.** TH-ARCH-6.

**Files.**

- [ ] Edit lifeboat bays and the gangway in `sim/src/layout.rs`.
- [ ] Edit `lifeboats` in `ship-game/src/venue_props.rs` so it reads those bays.

**Build.**

- [ ] Store 17 lifeboat bays in `sim/src/layout.rs`, on deck index 6, with their davits on the shell. Delete the hardcoded loop of 8 boats per side in `lifeboats`. Keep the existing boat colors. Index 6 is Icon deck 6, Royal Promenade, in the deck table in `docs/SIMULATION.md`. Do not move the boats to index 7. The sentence that says they hang off Icon deck 7 disagrees with that table.
- [ ] Update `workshops_plant_kitchens_and_park_leave_a_readable_mesh`. It currently requires a boat vertex with y below 0 and a vertex with y above 65. After this PR those vertices sit on the shell. Correct the Icon deck 7 sentence in `docs/SIMULATION.md` and the comment on `LIFEBOAT_DECK` in the same commit.
- [ ] Cut one gangway opening through the shell on deck index 2. Leave a second embarkation opening off this PR until the deck-plan PDF shows its deck.
- [ ] Keep Medical Center on deck 3, the workshops on deck 1, Category 6 on the top deck, and the bridge forward on deck 12.

**You see.**

- [ ] Seventeen boats sit on the shell of deck 6, not outside a 65 m box. Deck 2 has a hole in the shell at the gangway.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] `lifeboat_bays` expects 17 bays on deck 6, every bay on the shell, and one shell opening on deck 2. Run `cd sim && unset ARGV0 && cargo test --lib lifeboat_bays -- --exact`.
- [ ] Run `cd ship-game && unset ARGV0 && cargo test --lib` for the venue mesh test that counts boat blocks.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Open deck 6 at trunk and at the head. Trunk has no lifeboat mesh. Record that, and gate 17 bays on the shell. Save `th-arch-7-trunk.png`. Pass when the head shows boats on the shell and the trunk shot shows none.
- [ ] Lane 2. Plan view, deck 6, full length. Save `th-arch-7-count.png`. Pass when the boats number 17, not 16 and not 18.
- [ ] Lane 3. Zoom on one davit. Save `th-arch-7-davit.png`. Pass when the boat and the davit meet the shell, and the boat y is inside the frame rather than past y 0 or past the far shell.
- [ ] Lane 4. Side view of deck 6. Save `th-arch-7-side.png`. Pass when the boats hang at the promenade upper deck, not on the slide tops.
- [ ] Lane 5. Plan view, deck 2. Save `th-arch-7-gangway.png`. Pass when the shell is broken at the gangway and the corridor still reaches that opening.
- [ ] Lane 6. Plan view, deck 3. Save `th-arch-7-medical.png`. Pass when Medical Center is still on deck 3.
- [ ] Lane 7. Plan view, deck 1. Save `th-arch-7-shops.png`. Pass when Carpentry, Metalwork, Sewing workshop, and Laundry are still on deck 1.
- [ ] Lane 8. Plan view, deck index 19. Save `th-arch-7-slides.png`. Pass when Category 6 is still the top deck.
- [ ] Lane 9. Plan view, deck index 14, aft end. Save `th-arch-7-hideaway.png`. Pass when The Hideaway is still aft.
- [ ] Lane 10. Plan view, deck 12, forward end. Save `th-arch-7-bridge.png`. Pass when the Bridge room is still forward of the cabin bays.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Wall-clock seconds of `cargo test --lib --release` in `sim` on trunk and on the head, and seconds from HTTP 200 until `#ship-game-canvas` shows more than one color.
- [ ] Probe. Interleave `cd sim && unset ARGV0 && cargo test --lib --release` on trunk and on the head, recording `date +%s` before and after. Build the viewer once with `make run-ship`, then reload `http://127.0.0.1:8080/` and time the canvas. Both sides must produce both numbers.
- [ ] Baseline. Record the trunk test seconds and the trunk paint seconds first.
- [ ] Rule. Fail when the head test time is above 1.5 times the trunk test time, or above 180 seconds. Fail when head paint is more than 5 seconds slower than trunk paint, or more than 20 seconds after HTTP 200.

**Review gate.** The operator reviews before merge.

- [ ] Copy the lane 2 screenshot into `/cursor/stores/self/media/th-arch-7-review.png`.
- [ ] Record a 30 to 60 second video panning the deck 6 boats and then the deck 2 gangway. Save it as `/cursor/stores/self/media/th-arch-7-review.mp4`.
- [ ] Post the screenshots and the video in chat. Stop at merge-ready. Wait for the operator.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Rebased onto the TH-ARCH-6 head with patch-id unchanged.
- [ ] The owner stops. The operator lands this PR in stack order.

## Cut the Pearl, the pool tanks, and the AquaDome (TH-ARCH-8)

**Depends on.** TH-ARCH-7.

**Files.**

- [ ] Edit holes and the bow outline in `sim/src/hull.rs`.

**Build.**

- [ ] Add a circular hole of diameter 15 m on decks 5 and 6, centered on the Royal Promenade rectangle. Do not cut that hole on deck 4.
- [ ] Add a pool-tank hole only inside rooms that already draw a pool. Those names are The Hideaway, Water's Edge, Cloud 17, and the basin inside Category 6. Water's Edge is on index 7. Cloud 17 is on index 16. Hideaway is on index 14. Do not cut Windjammer. It is a restaurant and `pool` is not called for it. Keep `point_on_deck(14, park_center)` true.
- [ ] Add a bow outline for the AquaDome on deck index 14, forward of the cabin block. Keep the room AquaDome Market inside it. Leave glass shading and water shading untouched.

**You see.**

- [ ] The promenade has a round hole about 15 m across. The pool deck has tank openings. The bow of deck 15 carries a volume forward of the cabins.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] `public_voids` expects the 15 m hole on decks 5 and 6 only, pool holes only inside pool rooms, and the AquaDome outline forward. Run `cd sim && unset ARGV0 && cargo test --lib public_voids -- --exact`.
- [ ] Run `cd sim && unset ARGV0 && cargo test --lib central_park_is_open_above_its_floor -- --exact`.
- [ ] Run `cd ship-game && unset ARGV0 && cargo test --lib hull_plate_reaches_the_bow_and_accepts_the_park_hole -- --exact`.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Open deck 5 at trunk and at the head. Trunk's open void is the old upper-deck courtyard on a 318 m hull. Record that, and gate the 15 m round hole. Save `th-arch-8-trunk.png`. Pass when the head hole is round and about 15 m across, and the trunk shot is the old courtyard.
- [ ] Lane 2. Plan view, deck 5. Save `th-arch-8-pearl.png`. Pass when the hole sits inside the Royal Promenade and the promenade floor remains around it.
- [ ] Lane 3. Plan view, deck 6, same x and y. Save `th-arch-8-pearl-upper.png`. Pass when the hole matches the deck 5 hole in plan.
- [ ] Lane 4. Plan view, deck 4. Save `th-arch-8-theater.png`. Pass when the Royal Theater floor is solid where the Pearl would land.
- [ ] Lane 5. Plan view, deck index 14. Save `th-arch-8-pools.png`. Pass when each pool room contains an opening and the rest of the plate is solid.
- [ ] Lane 6. Bow of deck index 14. Save `th-arch-8-dome.png`. Pass when a volume projects forward of the cabin block and AquaDome Market sits inside it.
- [ ] Lane 7. Plan view, deck 9. Save `th-arch-8-park.png`. Pass when the Central Park hole on the cabin decks is still open and is not the Pearl.
- [ ] Lane 8. Plan view, deck 8. Save `th-arch-8-park-floor.png`. Pass when the park floor is solid.
- [ ] Lane 9. Side view of the bow. Save `th-arch-8-bow.png`. Pass when the AquaDome volume reads as structure above the bow, not as a flat restaurant rectangle alone.
- [ ] Lane 10. Plan view, deck index 19. Save `th-arch-8-slides.png`. Pass when Category 6 and the stepped slide tops are still there.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Wall-clock seconds of `cargo test --lib --release` in `sim` on trunk and on the head, and seconds from HTTP 200 until `#ship-game-canvas` shows more than one color.
- [ ] Probe. Interleave `cd sim && unset ARGV0 && cargo test --lib --release` on trunk and on the head, recording `date +%s` before and after. Build the viewer once with `make run-ship`, then reload `http://127.0.0.1:8080/` and time the canvas. Both sides must produce both numbers.
- [ ] Baseline. Record the trunk test seconds and the trunk paint seconds first.
- [ ] Rule. Fail when the head test time is above 1.5 times the trunk test time, or above 180 seconds. Fail when head paint is more than 5 seconds slower than trunk paint, or more than 20 seconds after HTTP 200.

**Review gate.** The operator reviews before merge.

- [ ] Copy the lane 2 screenshot into `/cursor/stores/self/media/th-arch-8-review.png`.
- [ ] Record a 30 to 60 second video from the Pearl down through the promenade and forward to the AquaDome. Save it as `/cursor/stores/self/media/th-arch-8-review.mp4`.
- [ ] Post the screenshots and the video in chat. Stop at merge-ready. Wait for the operator.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Rebased onto the TH-ARCH-7 head with patch-id unchanged.
- [ ] The owner stops. The operator lands this PR in stack order.

## Close the program

- [ ] Every box above is checked with its evidence.
- [ ] Reply to the operator with the verdict of each PR, in place of a merge list. Name the head SHA and the unresolved review gates.

## Appendix A. Prototype evidence

No prototype branch exists. The operator forbade repo edits and pull requests, so there is no SHA and no screenshot for a layout trial.

These questions stay unproven.

The DNV vessel page `https://vesselregister.dnv.com/vesselregister/details/38545` did not render. Do not encode a Wartsila model string until that page is read. Meyer Turku's delivery note of 26 November 2023 gives length 365 m, waterline width 48.5 m, deck-building width 57.8 m, bridge width 65.8 m, gross tonnage 248655, 7600 passengers, 2350 crew, and flag Panama. Baird Maritime's vessel review gives length 364.75 m, beam 48.47 m, draught 9.25 m, gross tonnage 248663, flag Bahamas, three engines of 16030 kW, three engines of 13740 kW, three ABB units of 20 MW, and five thrusters of 4800 kW. The program uses Meyer's three widths and Baird's draught. It does not pick a flag.

Deck height is unknown. `DECK_HEIGHT_M` stays 3.0.

The inner-bottom formula was not copied in full from SOLAS chapter II-1 regulation 9. TH-ARCH-2 reads it before setting `INNER_BOTTOM_M`.

Icon's as-built zone drawing is not in the sources below. TH-ARCH-4 enforces the regulatory limits. It does not claim the yard's cut positions.

A Cruise and Ferry article says two engine rooms and also says two tanks of 3.7 tonnes. LNG Prime, citing the cruise line, says the first tank weighs 307 tonnes and is about 27 m long and 8 m tall. The 3.7 tonne sentence is rejected. The two-engine-room split is rejected with it.

CruiseMapper says 22 elevators. That count is not in the Royal Caribbean deck-plan PDF text extracted here. TH-ARCH-5 places an elevator shaft at each atrium and does not hardcode 22.

The funnel's top deck is unknown. No PR builds a funnel height.

Which side of the ship has the shore connection is unknown. TH-ARCH-6 uses starboard until a plan shows otherwise.

A second embarkation opening on deck 5 is unproven. The PDF text contains the words PORT GANGWAY, and the extraction scrambled the deck sheets. TH-ARCH-7 cuts only the deck 2 shell. The owner adds a second opening only after the PDF page shows it.

Whether a rectangular module can reach 2805 staterooms without entering the park is unproven. TH-ARCH-3 prints the count and fails on area, not on 2805.

Baird says some family suites cover three decks and include an indoor slide. The deck-plan PDF was not read as a clean sheet. TH-ARCH-3 stacks one footprint across three suite decks and does not invent an indoor slide.

The 670 m jogging track is not used. It was not confirmed from the deck plan.

Oasis-class inboard lifeboats are not copied. No source read here says Icon stows boats inboard. The current mesh puts them outboard.

## Appendix B. Alternatives rejected

Restarting from `origin/main` was rejected. That tree is 318 m by 60 m, with one stair and deck names such as Engine Deck and Berth Deck A. PR 14 already replaced that arrangement with the Icon neighborhood list, length 365 m, and a single beam of 65 m.

Adding a `RoomKind` for every new venue was rejected. `missing_kinds` requires every variant somewhere on the ship, and every match in layout, stations, and recipes would grow. Elevators, escalators, tanks, pods, and zones are new structs. Fishing stays on deck 7 and hydroponics stays in the park, because the life simulation still needs those stations.

Inserting an empty deck 13 was rejected. Royal Caribbean's own deck-plan navigator lists Deck Two through Deck Twenty and skips Deck Thirteen. `decks[13].name` is already Deck 14 Spa. Keep that skip.

Adding a 21st deck index for the double bottom was rejected. It would desynchronize the 20 names from the 18 guest decks. The tank top is a height under Machinery.

Using 66 m as the only beam was rejected. The marketing sheet rounds the width to 66 m and 219 ft. Meyer's note is the one that separates waterline, deckhouse, and bridge. One 65 m waterline is the bug TH-ARCH-1 removes.

A full safe-return-to-port casualty model was rejected. The zone cuts are the architectural part. System redundancy and crew drills belong with the life simulation plan.

A walking character was rejected. The viewer is a fly camera. The pictures in the live lanes are the check. Pathfinding for people is not this program.

Recoloring `venue_props.rs` was rejected. That work belongs to the visual realism plan. TH-ARCH-7 may move boat positions and must leave the color constants.

## Appendix C. Risks

TH-ARCH-1 can push a room outside `point_on_deck` when the waterline narrows to 48.5 m. The owner shifts bands with `beam_for_deck` in the same change. `build_ship` returning Ok is the guard. Decks 0 and 1 must keep their alleys inside the narrow hull. If a cabin band remains on those decks, move the cabins to deckhouse decks rather than widening the waterline.

TH-ARCH-1 changes `SHIP_BEAM_M` from 65 to 65.8. Tests that sample `SHIP_BEAM_M * 0.5` move by 0.4 m. Update the sample. Do not freeze the old 65 as a second constant.

PR 14 is open and can move. TH-ARCH-1 rebases onto `cursor/ship-interior-realism-92d3` until that branch lands, then onto main. The rest of the stack restacks onto TH-ARCH-1.

`sim/src/layout.rs` is on TH-ARCH-1, TH-ARCH-3, TH-ARCH-4, TH-ARCH-5, TH-ARCH-6, and TH-ARCH-7. The stack is strictly linear so two owners do not edit it at once.

`every_kind_sits_inside_its_hull` requires every room to be reachable from the first stair. A balcony cell or a tank with no door fails that test. `saved_layout_matches` discards a stored world whose deck names or landmark rooms drift. Do not rename Central Park, Gas generator hall, Wastewater and incinerator, or Casino Royale.

Indexes 17 and 18 are two hull islands. A same-deck walk from the aft island to the Grove uses a stair down to index 16 and back up. `point_on_deck` is false in the gap. Leave that gap. The spine also has no segment from x 118 to 130. That strip is bare plate. Do not invent a room there.

TH-ARCH-7 fails `workshops_plant_kitchens_and_park_leave_a_readable_mesh` until the outboard y assertions are updated. The failure is the point of the PR.

TH-ARCH-4 can fail `rect_on_deck` if a cut splits a berth. Move the cut to a module edge. Do not delete the berth to silence the test.

TH-ARCH-8 can break `central_park_is_open_above_its_floor` if a pool hole covers the park center on deck index 14. That test says the pool plate covers the park footprint. Keep that point solid, or move the sample in the same commit.

The timehelm repo has no `pstack/` tree. If `git show origin/main:pstack/skills/...` fails, read the installed copies. Poteto playbooks are under `/home/ubuntu/.cursor/plugins/cache/cursor-public/9717366/de8ee15dd1194627d21eb5fcd41cb1873208dabb/skills/`. `control-ui` is under `/home/ubuntu/.cursor/plugins/cache/cursor-public/677/de8ee15dd1194627d21eb5fcd41cb1873208dabb/skills/control-ui/SKILL.md`.

Wasm cold builds are slow. The perf paint clock starts at HTTP 200, after `make run-ship` has already built.

`Door` gains a field in TH-ARCH-4. Saved worlds that deserialize `Door` need a default class of B so old saves still load. Check `sim/src/world.rs` and `sim/src/proto.rs` in that PR.

## Appendix D. Links and reading list

Read these before editing.

Meyer Turku, 26 November 2023, delivery figures, the Pearl as a 15 m load-bearing stair, and the AquaDome as a lifted glass-steel structure. `https://www.meyerturku.fi/en/news-publications/news/meyer-turku-shipyard-delivered-the-worlds-most-advanced-cruise-ship-icon-of-the-seas/`

ABB, 11 December 2024, six LNG-ready dual-fuel engines, Azipod propulsion, shore connection, length 365 m, 5610 guests. `https://new.abb.com/news/detail/121936/cruise-sector-breaks-fresh-ground-as-newest-ships-adopt-innovative-abb-control-technology`

Baird Maritime vessel review, draught 9.25 m, engine kilowatts, three 20 MW ABB units, five 4800 kW thrusters. `https://www.bairdmaritime.com/passenger/cruise/vessel-review-icon-of-the-seas-royal-caribbeans-newest-lng-powered-ship-can-house-5600-guests`

LNG Prime, 307 tonne tank, about 27 m long and 8 m tall, installed at Neptun Werft. `https://lngprime.com/europe/royal-caribbeans-newbuild-gets-lng-fuel-tank/32629/`

Royal Caribbean deck-plan navigator, decks 2 through 20 skipping 13. `https://www.royalcaribbean.com/cruise-ships/icon-of-the-seas/deck-plans/2377/02`

Royal Caribbean deck-plan PDF. `https://www.royalcaribbean.com/content/dam/royal/ships/deckplans-hub/Icon_2250_v2023.pdf`

Royal Caribbean Japan cabin areas. Standard inside 14.5 square metres, superior inside 16.5, ocean view 14.8 to 17.4, Central Park inside 17.4, balcony 18.9 plus 4.6, large balcony 18.9 plus 6.0 to 6.5. `https://www.royalcaribbean.jp/ships/IC/`

Tschudi, 17 lifeboats of 450 people delivered to Turku for this ship. `https://www.projectcargojournal.com/project-logistics/2023/05/10/tschudi-handles-lifeboats-delivery-for-icon-of-the-seas/`

Palfinger's contract for 18 boats is the fourth Icon-class ship. Do not use 18 for this hull. `https://www.palfinger.com/worldwide/en/insights-news-and-events/news/2025/palfinger-marine--fourth-major-contract-for-royal-caribbeans-ico.html`

SOLAS chapter II-2 regulation 9, main vertical zones, UK MCA copy. `https://assets.publishing.service.gov.uk/media/5a7f48cae5274a2e8ab4b3e4/Regulation_9_-_Containment_of_Fire.pdf`

SOLAS chapter II-2 regulation 13, two means of escape, lifts excluded. `https://imorules.com/GUID-FD648BB1-3991-407E-AD5E-1263AA7F3AD4.html`

SOLAS chapter II-1 regulation 9, double bottom. `https://www.imorules.com/GUID-568B838E-4F96-405B-A010-19DDA7FB2852.html`

TH-ARCH-1 and TH-ARCH-6 read `pstack/skills/how/SKILL.md` before editing. The beam split and the machinery block are the arrangement choices.

TH-ARCH-4 and TH-ARCH-5 read `pstack/skills/interrogate/SKILL.md` before editing. Zone cuts and the escape set are the contested choices.

Every owner keeps a decision trail per `pstack/skills/show-me-your-work/SKILL.md`. The hourly tick writes one row.

Leave `/cursor/stores/self/docs/visual-realism-plan.md` and `/cursor/stores/self/docs/life-simulation-plan.md` as the other two programs. This file does not take their work.

## Appendix E. What the code already models

`origin/main` at the time of this plan has `SHIP_LENGTH_M` 318 and `SHIP_BEAM_M` 60 in `sim/src/hull.rs`. `DECK_COUNT` is 20 and `DECK_HEIGHT_M` is 3 in `sim/src/layout.rs`. The first deck name is Engine Deck. One stair footprint is copied onto every deck. `RoomKind` has 28 variants, including Berth, Galley, Clinic, Engineering, FuelStore, and Bridge. `Door` has `id`, `from_room`, and `to_room`.

PR 14 sets `SHIP_LENGTH_M` to 365 and `SHIP_BEAM_M` to 65, and the comment calls 65 the maximum beam. That single number is used as the waterline. Meyer's waterline is 48.5 m, the deck building is 57.8 m, and the bridge is 65.8 m. There is no `DRAFT_M`.

PR 14 deck names, in index order, are Machinery, Utility, Deck 2 Gangway, Deck 3 Dining, Deck 4 Casino, Deck 5 Promenade, Deck 6 Promenade, Deck 7 Surfside, Deck 8 Central Park, Deck 9 Cabins, Deck 10 Cabins, Deck 11 Cabins, Deck 12 Bridge, Deck 14 Spa, Deck 15 Chill Island, Deck 16 Thrill Island, Deck 17 Cloud 17, Deck 18 Grove, Deck 19 Grove, and Deck 20 Slides. Index 13 is named Deck 14 because the number 13 is skipped. That matches the 18 guest decks plus two non-guest decks on the Royal Caribbean fact sheet, with Machinery and Utility standing in for the two decks the guest navigator does not list.

Outboard cabins use `CABIN_W` 4.4 m and `CABIN_STEP` 5.6 m. The starboard outboard band runs from y 1.8 to y 9.0, so the band is 7.2 m deep. The product of 4.4 and 7.2 is 31.68 square metres. A published balcony cabin is 18.9 square metres plus a 4.6 square metre balcony. The park-side published inside cabin is 17.4 square metres. The test `icon_neighborhoods_match_the_public_plans` only requires more than 20 berths on deck 9.

Stairs are two 10 m by 10 m rectangles, Aft atrium from x 72 to 82 and Forward atrium from x 258 to 268, both from y 26 to 36. The forward stair is omitted on deck index 19. They are `RoomKind::Stair`. There is no elevator room. The camera action Lift moves the view. It is not a passenger elevator. Page Up and Page Down change the displayed deck without passing through the stair.

`lifeboats` in `ship-game/src/venue_props.rs` draws 8 boats on each side, 16 in total, on `LIFEBOAT_DECK` 6, at y -2.6 and y 67.6. Those y values lie outside a 0 to 65 frame. The mesh test requires one of those outside vertices. Tschudi delivered 17 boats. The mesh is 16, and the boats are not bays in `sim`. The deck table maps index 6 to Icon deck 6. A comment and one sentence in `docs/SIMULATION.md` call the same index Icon deck 7. The table wins.

`place_heads` samples two heads on an outboard side. It does not give every berth a head. `SOLAR_KW` is 90.0 in `sim/src/world.rs`, and `supply_kwh` uses that constant. The generator rectangles do not change it. Pools are drawn for The Hideaway, Water's Edge, Cloud 17, and a basin in Category 6. There is no physics collider. Crew walk the door graph. The Bevy app is a camera.

The generator hall, Switchgear, and one Gas plant space are rectangles on deck 0. Waste, desalination, laundry, and the workshops are rectangles on deck 1. Medical Center is a named room on deck 3. The park is a hole in `deck_hull` for decks 9 through 13, with hydroponics rooms on the deck 8 floor. Pools and slides are prisms in `venue_props.rs`. The Pearl is not a 15 m hole. The AquaDome is the room AquaDome Market.

`build_ship` fails if any `RoomKind` is missing. That is why Fishing remains on deck 7. Do not delete it in this program.
