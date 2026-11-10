# Life simulation plan

A player on the ship should see the crew work, eat, sleep, and stay dead. The engineer who next edits the minute step should change one of those behaviors without retuning the others.

The program changes crew life in the persistent sim. It does not add log tokens. Hunger, thirst, death, meals, water, medicine, sleep, and treatment already log.

A living person keeps a watch and leaves it for hunger, thirst, or rest. A dead person stays where they fell.

Land LS-1, then LS-2, then LS-3, then LS-4, then LS-5, then LS-6.

## How to read this

One box is one unit of work. Every box names the evidence that checks it. A nested box is a sub-step of the box above it. Check a box only when its evidence exists, a file, a log line, a screenshot, a test run, or a SHA. The body is a how-to. The appendices explain and record.

The program runs `pstack/skills/poteto-mode/playbooks/autopilot-stack.md`. Owners stop at merge-ready. The operator lands LS-1, LS-2, LS-3, LS-4, LS-5, and LS-6 bottom-up. No owner merges.

Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

## Program checklist

### Arm the program

- [ ] State the protocol and this plan to the operator, then stop. Start execution only on the operator's explicit go.
- [ ] Read these from trunk at program start. Re-read them at every tick.
  - [ ] `git show origin/main:pstack/skills/poteto-mode/playbooks/autopilot-stack.md`
  - [ ] `git show origin/main:pstack/skills/swarm/SKILL.md`
  - [ ] `git show origin/main:pstack/skills/control-ui/SKILL.md`
  - [ ] `git show origin/main:pstack/skills/poteto-mode/playbooks/opening-a-pr.md`
  - [ ] `git show origin/main:pstack/skills/principle-prove-it-works/SKILL.md`
  - [ ] `git show origin/main:pstack/skills/principle-sequence-verifiable-units/SKILL.md`
  - [ ] `git show origin/main:pstack/skills/principle-test-behavior-not-implementation/SKILL.md`
  - [ ] `git show origin/main:pstack/skills/show-me-your-work/SKILL.md`
  - [ ] `git show origin/main:docs/SIMULATION.md`
- [ ] On the operator's go, arm the audit tick as `/loop 1h` with the tick prompt below. Never leave the cadence to memory.
- [ ] Use this tick prompt, verbatim. "Re-read the execution playbook from trunk. Audit the operation against it and fix drift in this tick. Probe every active lane and judge progress by side effects only. Stand down a stuck lane and dispatch its replacement now. Then post a short status message to the operator in chat only when the audit found a tracked change that no earlier status message reported, such as a PR opened, a code-ready head, a round launched or closed, a verdict, a merge, a stuck agent and the action taken, a blocker added or cleared, or a decision only the operator can make. Name every such change and nothing else. Do not repeat a table, the merged list, or an unchanged blocker. If the audit found none, end the turn with no reply text. Either way, log this tick's row in your decision trail. The row names the items reported, or none."
- [ ] On the operator's hold or stand-down, send every owner a zero-writes order at once.

### Spawn owners

- [ ] Spawn one owner per PR with the full lifecycle the execution playbook names.
- [ ] Follow this dependency graph. Start dependent work only after its parent is merge-ready, and base it on the parent branch. These PRs stack because each one edits `sim/src/world.rs`.
  - [ ] LS-1 is first. Branch it from `main`.
  - [ ] LS-2 after LS-1. LS-3 after LS-2. LS-4 after LS-3. LS-5 after LS-4. LS-6 after LS-5.
- [ ] Hold the file boundaries. LS-1 through LS-5 touch only `sim/src/world.rs` and `docs/SIMULATION.md`. LS-6 also touches `server/src/db.rs` and a new file `server/migrations/0002_companionship.sql`. No PR edits `server/migrations/0001_sim_world.sql`.
- [ ] Hold the review gate. LS-1, LS-2, LS-3, LS-4, LS-5, and LS-6 change an interaction. They wait for the operator's review in chat with screenshots and a video before merge.

### PR mechanics, for every PR

- [ ] Resolve the forge once. Default to `gh`. If `command -v origin` succeeds and Origin can resolve the repository, use `origin pr` for every PR operation. Record any fallback to `gh`. Never require `gt`.
- [ ] Open the PR ready, never draft, per **Opening a PR**. Use the run's built-in PR tool when it has one, else `origin pr create --status open --base <base-branch>` or `gh pr create --base <base-branch>` according to the resolved forge. A stack child targets its parent branch.
- [ ] Run sim `cargo fmt --check` and sim `cargo clippy -- -D warnings` before the PR-facing push. For LS-6, run the same two commands in `server` as well. Push with hooks on.
- [ ] Run `/deslop` before each commit and `/no-comments` before review.
- [ ] Triage every Bugbot and security-reviewer comment per `../references/bugbot-triage.md`.
- [ ] Rebase onto the parent head, and rebase LS-1 onto current trunk, before the code-ready report and babysit. Keep that merge base in fix rounds. Rebase again only at merge prep, on a `git merge-tree` conflict with the parent, on a CI failure that comes from a change on the parent, or when a known concurrent change to the same files lands.

### Verdict and merge, for every PR

- [ ] A full swarm runs once, at the code-ready head. Another full swarm runs only for new behavior, a redesign, or a large new diff. A rebase, a merge-conflict resolution, or a minor fix gets a single verifier at the new head, and a wording-only change gets none. Run each full swarm per `pstack/skills/swarm/SKILL.md`. One gates lane. The ten live lanes from the PR's **Verify, live** block. The perf lane from its **Verify, perf** block. Two or more audit lanes, each with its own focus, that read the diff and the receipts and distrust the PR body. The root audits the receipts in the merge-ready report before the verdict.
- [ ] Judge every finding by the execution playbook's blocking rule. The verdict is clean when none blocks. A failure on trunk behavior in a lane marked `Regression lane against trunk.` is recorded against trunk and does not block the PR. A new head gets a fresh verdict, except for results that stay valid under the patch-id rule in `playbooks/shipping.md`.
- [ ] Stop at verified. Do not merge. After the operator review, hand the PR to the operator. The operator lands bottom-up. A verdict carries across a restack only when `git patch-id` is unchanged, per `playbooks/shipping.md`.

### Boot recipe, for every live lane

Each live lane runs on its own cloud VM at the PR head. Drive the browser through `control-ui` from `cursor-team-kit`. Read `GET /api/world` and the server log as diagnostics. Clicks, deck changes, camera keys, and room hover go through `control-ui` only. The 3D client draws positions, decks, up to four hovered names, and a HUD of crew count, food days, water days, and power. It does not draw hunger, health, thirst, fatigue, or activity. Read those from `/api/world`. `sync_crew` glides a body for 60 seconds, so a screenshot that must show final positions waits out that glide.

- [ ] `git fetch origin <head-branch> && git checkout <head SHA>`.
- [ ] Start Postgres if it is down with `sudo pg_ctlcluster 16 main start`. Put `DATABASE_URL=postgresql://postgres:password@localhost:5432/timehelm` in `server/.env`. Run `cd server && env -u ARGV0 cargo run`. Wait until the log contains `Ship world ready`.
- [ ] For a lane that names a fixture, stop the server, apply the fixture in Postgres, set `world_meta.last_tick_unix` to the current unix second, and start the server again before opening the browser.
- [ ] Open `http://localhost:8080/3d/` through `control-ui`. Read `http://localhost:8080/api/world` for crew, stocks, and totals.
- [ ] Save every screenshot to `/tmp/swarm-<pr-id>/worker-<n>/<slug>.png` and return the paths with the report.

## Make death terminal (LS-1)

**Depends on.** None.

**Files.**

- [ ] Edit `sim/src/world.rs`.
- [ ] Edit `docs/SIMULATION.md`.

**Build.**

- [ ] Add `is_alive` in `sim/src/world.rs` and use it from `tick_needs`, `tick_clinic`, `tick_movement`, `station_staffed`, `room_staffed`, and the fisher count in `tick_fishing`. A person with health at or below 0 is not alive. Do not add a second death flag.
- [ ] Leave the existing `died` log as it is. Do not add a log token. Do not remove the body from `crew`.
- [ ] Rewrite `clinic_revival_is_logged` so a person who starts the step at health 0 stays at health 0, keeps the same room and coordinates, and does not consume `MedicalKg`.

**You see.**

- [ ] A body at health 0 stays in the room where it fell, and the clinic does not raise that health, while a living patient in the clinic still gains health.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Add `a_dead_crew_member_stays_put_and_is_not_revived` in `sim/src/world.rs`. It places a health 0 person in the clinic with medical stock, steps twice, and expects health 0, the same coordinates, unchanged medical stock, and no `revived` line. Run `cd sim && env -u ARGV0 cargo test a_dead_crew_member_stays_put_and_is_not_revived clinic_revival -- --test-threads=1`.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Fixture crew id 0 at health 0 in the clinic with medical stock, then let one minute pass on trunk and on the head. Save `ls1-dead-still.png`. Pass when head health is still 0 and the lane records the trunk health.
- [ ] Lane 2. Boot a fresh world with no fixture. Save `ls1-living-boot.png`. Pass when `/api/world` lists 80 crew and every health is above 0.
- [ ] Lane 3. Use the dead fixture and wait one minute. Save `ls1-dead-room.png`. Pass when that crew member `room_id` is unchanged.
- [ ] Lane 4. On the same world, watch one living crew member for one minute. Save `ls1-living-moves.png`. Pass when that living person has a new `x` or `y`.
- [ ] Lane 5. Read the dead person activity after one minute. Save `ls1-dead-activity.png`. Pass when activity is not `eating` and not `treating`.
- [ ] Lane 6. Compare medical stock before and after one minute with only a dead patient in the clinic. Save `ls1-medicine-held.png`. Pass when `MedicalKg` is unchanged.
- [ ] Lane 7. Fixture a different living person at health 0.5 in the clinic with medical stock. Save `ls1-living-treated.png`. Pass when that health is above 0.5 after one minute.
- [ ] Lane 8. Open the HUD with the dead body still in the roster. Save `ls1-hud-eighty.png`. Pass when the on-screen crew count is 80.
- [ ] Lane 9. Read `/api/world` after the death fixture. Save `ls1-body-kept.png`. Pass when the dead id is still in the crew array.
- [ ] Lane 10. Wait a second minute on the death fixture. Save `ls1-still-dead.png`. Pass when health is still 0.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Median wall-clock seconds of three runs of `catch_up_matches_a_day_of_steps` after a warm compile, on trunk and on the head.
- [ ] Probe. Compile `sim` tests once per side. Then run `cd sim && env -u ARGV0 cargo test catch_up_matches_a_day_of_steps -- --exact --test-threads=1` three times per side. Interleave trunk and head. Record every elapsed time and the spread.
- [ ] Baseline. Record the trunk median first.
- [ ] Rule. Fail when the three-run spread on either side is larger than the gap, because that gap is noise. Otherwise fail when the head median is greater than 2 times the trunk median. The test must also pass on both sides.

**Review gate.** The operator reviews before merge.

- [ ] Copy the lane 1 screenshot into `/cursor/stores/self/media/LS-1-review-dead-still.png`.
- [ ] Record a 30 to 60 second video of the dead body staying still while a living neighbor moves. Save it as `/cursor/stores/self/media/LS-1-review.mp4`.
- [ ] Post the screenshots and the video in chat. Stop at merge-ready. Wait for the operator.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Rebased onto current trunk after the verdict, patch-id unchanged.
- [ ] The owner does not merge. The operator lands LS-1 first.

## Interrupt work for hunger and thirst (LS-2)

**Depends on.** LS-1.

**Files.**

- [ ] Edit `sim/src/world.rs`.
- [ ] Edit `docs/SIMULATION.md`.

**Build.**

- [ ] In `muster`, set `watch_offset_min` to the index within the department modulo 3, times 480. Stop using `dept_index(department) * 45`. Leave `on_shift` as an 8 hour block.
- [ ] In `goal_room`, for a living person, go to the clinic when health is below 0.55. Otherwise go to a water store when thirst is above 0.7. Otherwise go to a dining room that holds at least one meal portion when hunger is above 0.7, or to the first mess when no portion exists. Otherwise go to the assigned station while on shift. Otherwise, when fatigue is above 0.6, go to a head unless already in a head, in which case go to the berth. Otherwise go to a mess or restaurant.
- [ ] In `tick_needs`, drink the existing 0.12 L only when the person is standing in a water store. Eat the existing 0.45 kg meal only from `MealKg` stock in the room they are standing in.
- [ ] In `finish_recipe`, add `CookMeal` output to a mess room. Leave every other recipe output in the station room.
- [ ] Do not change `advance_station`. Do not add a log token. Update `eating_drinking_and_shortages_are_logged` so the drinker stands in the water store. Keep the existing `drank` and `ate` lines.

**You see.**

- [ ] Galley, clinic, and bridge are not empty for the same 16 hours, a hungry watch stander walks to food, and a thirsty person drinks only at a water store.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Add `watches_cover_galley_clinic_and_bridge` and `hunger_and_thirst_pull_a_living_person_off_station` in `sim/src/world.rs`. The first expects every department of 3 or more to have someone on shift at minutes 0, 480, and 960. The second expects a hungry on-shift person to target food and a thirsty person outside a water store to leave water stock unchanged. Run `cd sim && env -u ARGV0 cargo test watches_cover hunger_and_thirst_pull eating_drinking_and_shortages -- --test-threads=1`.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Read `watch_offset_min` for the galley department on trunk and on the head. Save `ls2-watch-split.png`. Pass when the head has more than one distinct galley offset and the lane records the trunk offsets.
- [ ] Lane 2. At the current minute, split galley crew into on shift and off shift. Save `ls2-galley-covered.png`. Pass when both groups exist.
- [ ] Lane 3. Fixture hunger 0.95 on an on-shift person standing at their station. Save `ls2-leave-post.png`. Pass when after one minute their `room_id` changed or their activity is `walking`.
- [ ] Lane 4. Fixture thirst 0.95 outside a water store. Save `ls2-seek-water.png`. Pass when after one minute they are walking or their room is a water store.
- [ ] Lane 5. Fixture thirst 0.95 in a fishing room. Save `ls2-no-remote-drink.png`. Pass when water litres are unchanged after one minute.
- [ ] Lane 6. Fixture thirst 0.50 while standing in the water store that holds the water. Save `ls2-drink-at-store.png`. Pass when water litres fell by 0.12.
- [ ] Lane 7. Fixture an off-shift person with fatigue 0.9 in a workshop. Save `ls2-to-bed.png`. Pass when after one minute they are in a head, in their berth, or walking.
- [ ] Lane 8. Fixture an off-shift person with fatigue 0.1 and hunger 0.2 in a workshop. Save `ls2-evening-out.png`. Pass when after one minute they are in a mess, in a restaurant, or walking, and they are not sent straight to a berth.
- [ ] Lane 9. Put two hungry people in one mess that holds 0.50 kg of meals. Save `ls2-one-portion.png`. Pass when one hunger value fell, the other stayed above 0.9, and meal kg fell below 0.45.
- [ ] Lane 10. Leave a thirsty person in a berth. Save `ls2-berth-no-drink.png`. Pass when water litres are unchanged after one minute.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Median wall-clock seconds of three runs of `catch_up_matches_a_day_of_steps` after a warm compile, on trunk and on the head.
- [ ] Probe. Compile `sim` tests once per side. Then run `cd sim && env -u ARGV0 cargo test catch_up_matches_a_day_of_steps -- --exact --test-threads=1` three times per side. Interleave trunk and head. Record every elapsed time and the spread.
- [ ] Baseline. Record the trunk median first.
- [ ] Rule. Fail when the three-run spread on either side is larger than the gap, because that gap is noise. Otherwise fail when the head median is greater than 2 times the trunk median. The test must also pass on both sides.

**Review gate.** The operator reviews before merge.

- [ ] Copy the lane 3 screenshot into `/cursor/stores/self/media/LS-2-review-leave-post.png`.
- [ ] Record a 30 to 60 second video of a watch stander leaving for food and a thirsty person moving toward water. Save it as `/cursor/stores/self/media/LS-2-review.mp4`.
- [ ] Post the screenshots and the video in chat. Stop at merge-ready. Wait for the operator.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Rebased onto the LS-1 head after the verdict, patch-id unchanged.
- [ ] The owner does not merge. The operator lands LS-2 only after LS-1 has landed, then retargets LS-3 onto trunk.

## Scale work by who is present (LS-3)

**Depends on.** LS-2.

**Files.**

- [ ] Edit `sim/src/world.rs`.
- [ ] Edit `docs/SIMULATION.md`.

**Build.**

- [ ] In `advance_station`, add elapsed time only while `station_staffed` is true for that minute. A job that already exists pauses when the room is empty. It does not lose its elapsed time.
- [ ] Add `skill_matches` beside `advance_station`. `BakeBread` matches `Baking`. `CookMeal` matches `Cooking`. `Desalinate`, `RecycleMetal`, and `CompostNutrients` match `Engineering`. `SpinCloth` and `SewGarment` match `Sewing`. `SawPlanks` and `RepairTools` match `Carpentry`. `ForgeFitting` matches `Metalwork`. `WashClothes` matches `General`.
- [ ] Scale the elapsed gain by the number of living on-shift assigned people in the room whose skill matches, plus half a person for each living on-shift assigned person whose skill does not match. At fatigue 1, halve that gain. Below health 0.55, halve it again.
- [ ] Set activity to `cooking` when the staffed recipe is `CookMeal` or `BakeBread`. Set `working` for any other staffed recipe.
- [ ] Apply the same fatigue halving to the kilograms added in `tick_fishing`. A dead fisher still does not count. Do not add a log token.

**You see.**

- [ ] A galley batch stops when the cook leaves, two matched cooks advance it faster than one, and a tired or hurt cook is slower.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Add `an_unstaffed_job_does_not_advance` and `two_cooks_finish_faster_than_one` in `sim/src/world.rs`. The first expects elapsed time to hold when the cook is outside the room. The second expects one minute with two matching cooks to add more elapsed time than one minute with one matching cook, and a fatigue of 1 to add half of the rested gain. Run `cd sim && env -u ARGV0 cargo test an_unstaffed_job two_cooks_finish -- --test-threads=1`.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Start a `CookMeal` job close to its duration, move the cook out, and let one minute pass on trunk and on the head. Save `ls3-paused-job.png`. Pass when head meal kg does not rise and the lane records whether trunk meal kg rose.
- [ ] Lane 2. Run one minute with one matching cook and one minute with two matching cooks in the same galley. Save `ls3-two-cooks.png`. Pass when the two-cook elapsed gain is greater than the one-cook elapsed gain.
- [ ] Lane 3. Move the cook out of a running job and step one minute. Save `ls3-cook-left.png`. Pass when `elapsed_seconds` is unchanged.
- [ ] Lane 4. Compare one minute of a `Cooking` cook with one minute of a `General` cook on the same recipe. Save `ls3-relief-slower.png`. Pass when the unmatched gain is half the matched gain, within 1 percent.
- [ ] Lane 5. Compare fatigue 0 with fatigue 1 on the same staffed station. Save `ls3-fatigue-slows.png`. Pass when the tired gain is half the rested gain, within 1 percent.
- [ ] Lane 6. Compare health 1 with health 0.4 on the same staffed station. Save `ls3-health-slows.png`. Pass when the hurt gain is half the healthy gain, within 1 percent.
- [ ] Lane 7. Staff a `CookMeal` station for one minute. Save `ls3-cooking-label.png`. Pass when `/api/world` activity for that cook is `cooking`.
- [ ] Lane 8. Set the only assigned cook to health 0 inside the galley. Save `ls3-dead-not-staff.png`. Pass when `elapsed_seconds` does not advance.
- [ ] Lane 9. Move every fisher out of the fishing rooms. Save `ls3-no-fish.png`. Pass when fish kg is unchanged after one minute.
- [ ] Lane 10. Read power demand with the cook in the galley and with the cook outside it. Save `ls3-power-drop.png`. Pass when demand is lower with the cook outside.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Median wall-clock seconds of three runs of `catch_up_matches_a_day_of_steps` after a warm compile, on trunk and on the head.
- [ ] Probe. Compile `sim` tests once per side. Then run `cd sim && env -u ARGV0 cargo test catch_up_matches_a_day_of_steps -- --exact --test-threads=1` three times per side. Interleave trunk and head. Record every elapsed time and the spread.
- [ ] Baseline. Record the trunk median first.
- [ ] Rule. Fail when the three-run spread on either side is larger than the gap, because that gap is noise. Otherwise fail when the head median is greater than 2 times the trunk median. The test must also pass on both sides.

**Review gate.** The operator reviews before merge.

- [ ] Copy the lane 1 screenshot into `/cursor/stores/self/media/LS-3-review-paused-job.png`.
- [ ] Record a 30 to 60 second video of meal stock holding still after the cook leaves and moving again when the cook returns. Save it as `/cursor/stores/self/media/LS-3-review.mp4`.
- [ ] Post the screenshots and the video in chat. Stop at merge-ready. Wait for the operator.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Rebased onto the LS-2 head after the verdict, patch-id unchanged.
- [ ] The owner does not merge. The operator lands LS-3 only after LS-2 has landed.

## Make food and medicine run out (LS-4)

**Depends on.** LS-3.

**Files.**

- [ ] Edit `sim/src/world.rs`.
- [ ] Edit `docs/SIMULATION.md`.

**Build.**

- [ ] In `tick_needs`, when a living person is in a dining room, hunger is at least 0.5, and that room has no 0.45 kg meal, eat 0.25 kg of bread from that room, else 0.20 kg of fish, else 0.20 kg of vegetables. Take fish and vegetables from that room or from a cold store. Relief for each fallback bite is 0.25. Keep the meal portion at 0.45 kg and the meal relief at 0.4. Prefer the meal when it is present.
- [ ] In `tick_wear`, at minute 0, reduce `MealKg`, `BreadKg`, `FishKg`, and `VegetablesKg` by 2 percent when the stock room is not a cold store. Leave cold-store quantities unchanged. Keep the existing garment wear.
- [ ] In `tick_clinic`, treat only when health is below 0.55 and above 0, and a living on-shift person with `Skill::Medicine` is in that same clinic. Keep the 0.02 kg cost and the 0.05 health gain. Do not treat in the infirmary. Do not add a medicine recipe. Do not add a log token.
- [ ] Leave `food_days` in `World::totals` as edible kilograms divided by the product of the crew count and 0.6. Edible kilograms already include meals, bread, vegetables, and fish.

**You see.**

- [ ] With meals gone, a hungry person in the mess eats bread, fish, or vegetables, the HUD food days fall for that bite, and medicine is spent only on a badly hurt living person while a medic is there.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Add `a_hungry_person_eats_fish_when_meals_are_gone`, `perishables_spoil_outside_the_cold_store`, and `clinic_treats_only_with_a_medic` in `sim/src/world.rs`. They expect hunger to fall without `MealKg`, cold-store fish to hold across a minute-0 step, galley meals to drop, and medical stock to hold when no medic is present. Run `cd sim && env -u ARGV0 cargo test a_hungry_person_eats_fish perishables_spoil clinic_treats_only -- --test-threads=1`.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Remove meals, put fish in the mess, set hunger to 0.95, and step one minute on trunk and on the head. Save `ls4-eats-fish.png`. Pass when head hunger fell and the lane records that trunk hunger did not.
- [ ] Lane 2. Remove meals and fish, put bread in the mess, and step one minute. Save `ls4-eats-bread.png`. Pass when bread kg fell and hunger fell.
- [ ] Lane 3. Remove meals, fish, and bread, put vegetables in the mess, and step one minute. Save `ls4-eats-veg.png`. Pass when vegetable kg fell and hunger fell.
- [ ] Lane 4. Put both a meal portion and fish in the mess. Save `ls4-prefers-meal.png`. Pass when meal kg fell and fish kg did not.
- [ ] Lane 5. Set the next step to minute 0. Put meals in a galley and the same quantity in a cold store. Save `ls4-spoil.png`. Pass when galley meals fell by 2 percent and cold-store meals did not.
- [ ] Lane 6. Put a living person at health 0.5 in the clinic with medical stock and no medic in the room. Save `ls4-no-medic.png`. Pass when health and medical kg are unchanged after one minute.
- [ ] Lane 7. Add an on-shift `Medicine` crew member in that clinic. Save `ls4-medic.png`. Pass when health rose and medical kg fell by 0.02.
- [ ] Lane 8. Set the patient health to 0.7 with the medic still present. Save `ls4-no-bruise.png`. Pass when medical kg is unchanged.
- [ ] Lane 9. Set the patient health to 0 with the medic present. Save `ls4-medic-ignores-dead.png`. Pass when health stays 0 and medical kg is unchanged.
- [ ] Lane 10. Eat a fish bite and read the HUD food days before and after. Save `ls4-hud-food-days.png`. Pass when the on-screen food days number is lower after the bite.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Median wall-clock seconds of three runs of `catch_up_matches_a_day_of_steps` after a warm compile, on trunk and on the head.
- [ ] Probe. Compile `sim` tests once per side. Then run `cd sim && env -u ARGV0 cargo test catch_up_matches_a_day_of_steps -- --exact --test-threads=1` three times per side. Interleave trunk and head. Record every elapsed time and the spread.
- [ ] Baseline. Record the trunk median first.
- [ ] Rule. Fail when the three-run spread on either side is larger than the gap, because that gap is noise. Otherwise fail when the head median is greater than 2 times the trunk median. The test must also pass on both sides.

**Review gate.** The operator reviews before merge.

- [ ] Copy the lane 1 screenshot and the lane 10 screenshot into `/cursor/stores/self/media/LS-4-review-eats-fish.png` and `/cursor/stores/self/media/LS-4-review-food-days.png`.
- [ ] Record a 30 to 60 second video of a meal-less mess where hunger falls and the HUD food days move. Save it as `/cursor/stores/self/media/LS-4-review.mp4`.
- [ ] Post the screenshots and the video in chat. Stop at merge-ready. Wait for the operator.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Rebased onto the LS-3 head after the verdict, patch-id unchanged.
- [ ] The owner does not merge. The operator lands LS-4 only after LS-3 has landed.

## Spread crew through the ship (LS-5)

**Depends on.** LS-4.

**Files.**

- [ ] Edit `sim/src/world.rs`.
- [ ] Edit `docs/SIMULATION.md`.

**Build.**

- [ ] In `goal_room`, choose the nearest room of the needed kind by door-graph distance. Stop using the lowest room id from `first_room` for clinic, mess, water store, and head goals.
- [ ] Scale the existing 1.3 metres per second by health, and again by fatigue. Health 1 and fatigue 0 keep 1.3 metres per second. Health 0.55 or fatigue 1 is slower. A dead person still does not move.
- [ ] `move_crew` returns without moving the point when `room_id` is already the goal, and `muster` starts every berth mate on the room centroid. Assign a unique point inside the polygon both on arrival and when the person is already there. In a berth, use a free bed from `cabin_fittings` when one exists. Two living people must not share `x` and `y`.
- [ ] Do not add door capacity. A room still accepts the person. Do not add a log token.

**You see.**

- [ ] People in the same mess stand on different points. A hurt person covers less ground in one minute. A person walks to the nearer mess.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Add `two_crew_in_one_room_do_not_share_a_point` and `low_health_walks_less_far` in `sim/src/world.rs`. They expect distinct coordinates after a shared-room step, and a shorter distance for health 0.4 than for health 1 from the same start. Run `cd sim && env -u ARGV0 cargo test two_crew_in_one_room low_health_walks -- --test-threads=1`.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Send two hungry living people into the same mess on trunk and on the head. Save `ls5-unstacked.png`. Pass when the head coordinates differ and the lane records whether the trunk coordinates match.
- [ ] Lane 2. Start two living people at the same point, one at health 1 and one at health 0.4, with the same goal. Save `ls5-slow-walk.png`. Pass when after one minute the hurt person is closer to the start.
- [ ] Lane 3. Repeat with fatigue 0 and fatigue 1 at full health. Save `ls5-tired-walk.png`. Pass when after one minute the tired person is closer to the start.
- [ ] Lane 4. Pick a start whose nearest mess is not the lowest mess id. Save `ls5-nearest-mess.png`. Pass when the next room is the nearer mess.
- [ ] Lane 5. Send one off-shift tired person into a berth that has a bed. Save `ls5-bed.png`. Pass when their point lies on a bed mark and is not the room centroid when those differ.
- [ ] Lane 6. Send a second living person into that berth. Save `ls5-second-bed.png`. Pass when the two points differ.
- [ ] Lane 7. Fixture a dead person under the new speed rule. Save `ls5-dead-still.png`. Pass when `x` and `y` are unchanged after one minute.
- [ ] Lane 8. Place a healthy rested person one short corridor from their station. Save `ls5-still-walks.png`. Pass when they change room or reduce the distance to the doorway within one minute.
- [ ] Lane 9. Place a person one stair away from their goal. Save `ls5-stair.png`. Pass when their deck changes within five minutes.
- [ ] Lane 10. Fixture eight living off-watch people in one mess and wait 60 seconds for the position glide. Save `ls5-crowd.png`. Pass when `/api/world` has at least four distinct positions and the screenshot shows those bodies apart.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Median wall-clock seconds of three runs of `catch_up_matches_a_day_of_steps` after a warm compile, on trunk and on the head.
- [ ] Probe. Compile `sim` tests once per side. Then run `cd sim && env -u ARGV0 cargo test catch_up_matches_a_day_of_steps -- --exact --test-threads=1` three times per side. Interleave trunk and head. Record every elapsed time and the spread.
- [ ] Baseline. Record the trunk median first.
- [ ] Rule. Fail when the three-run spread on either side is larger than the gap, because that gap is noise. Otherwise fail when the head median is greater than 2 times the trunk median. The test must also pass on both sides.

**Review gate.** The operator reviews before merge.

- [ ] Copy the lane 10 screenshot into `/cursor/stores/self/media/LS-5-review-crowd.png`.
- [ ] Record a 30 to 60 second video of the mess with people on separate points, including one slower hurt walker. Save it as `/cursor/stores/self/media/LS-5-review.mp4`.
- [ ] Post the screenshots and the video in chat. Stop at merge-ready. Wait for the operator.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Rebased onto the LS-4 head after the verdict, patch-id unchanged.
- [ ] The owner does not merge. The operator lands LS-5 only after LS-4 has landed.

## Send lonely crew to company (LS-6)

**Depends on.** LS-5.

**Files.**

- [ ] Edit `sim/src/world.rs`.
- [ ] Edit `docs/SIMULATION.md`.
- [ ] Edit `server/src/db.rs`.
- [ ] Create `server/migrations/0002_companionship.sql`.

**Build.**

- [ ] Add `companionship` on `CrewMember`, a `f64` starting at 0.5 in `muster`. Each minute alone among living people, add 1 divided by 1440, capped at 1.5. Each minute sharing a room with another living person, subtract 1 divided by 480, floored at 0. A dead roommate does not count.
- [ ] When off watch, fed, and rested, a person with companionship above 0.6 prefers a mess or restaurant that already holds a living person over an empty one.
- [ ] While sleeping, companionship above 0.8 halves fatigue recovery.
- [ ] Store the column as `DOUBLE PRECISION NOT NULL DEFAULT 0.5` in the new migration. Update the crew `SELECT`, `INSERT`, row struct, and both maps in `server/src/db.rs`. Do not edit `0001_sim_world.sql`. Do not add the field to `CrewView`. Do not add a log token.

**You see.**

- [ ] A lonely rested person joins a mess that already has people, a night alone recovers less fatigue, and the value is still there after a server restart.

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Add `a_lonely_rested_person_joins_an_occupied_mess` and `isolation_slows_sleep_recovery` in `sim/src/world.rs`. They expect the occupied mess over an empty one, and a smaller fatigue drop when companionship is above 0.8. Run `cd sim && env -u ARGV0 cargo test a_lonely_rested_person isolation_slows_sleep -- --test-threads=1`. Also run `cd server && env -u ARGV0 cargo test embedded_migration_checksum -- --test-threads=1`.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes on `grok-4.7-xhigh-fast` at the PR head, per the boot recipe.

- [ ] Lane 1. Regression lane against trunk. Read the crew columns on trunk and on the head. Save `ls6-column.png`. Pass when the head column defaults to 0.5 and the lane records that trunk has no companionship column.
- [ ] Lane 2. Fixture companionship at 0.59 for a person alone, then step one minute. Save `ls6-lonelier.png`. Pass when the stored value rose.
- [ ] Lane 3. Put two living people in one room and step one minute. Save `ls6-company.png`. Pass when companionship fell.
- [ ] Lane 4. Fixture an off-shift person with hunger 0.2, fatigue 0.1, and companionship 0.9, with one occupied mess and one empty mess. Save `ls6-seeks-mess.png`. Pass when they walk toward or enter the occupied mess.
- [ ] Lane 5. Repeat lane 4 with companionship 0.1. Save `ls6-not-desperate.png`. Pass when companionship stays at or below 0.6 and the person is in a mess, a restaurant, or walking toward one.
- [ ] Lane 6. Sleep one minute in a berth at companionship 0 and at companionship 0.9. Save `ls6-restless.png`. Pass when the lonely fatigue drop is half the company fatigue drop, within 1 percent.
- [ ] Lane 7. Put a living person alone with a dead roommate. Save `ls6-dead-company.png`. Pass when companionship still rises.
- [ ] Lane 8. Set companionship to 0.77, restart the server, and load the world. Save `ls6-persisted.png`. Pass when the loaded value is 0.77.
- [ ] Lane 9. Boot the migrated database. Save `ls6-roster-loads.png`. Pass when `/api/world` lists 80 crew.
- [ ] Lane 10. Hover the occupied mess, then the empty mess. Save `ls6-gather.png`. Pass when the hovered occupied room lists at least two living names and `/api/world` shows the lonely person in that room.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. Median wall-clock seconds of three runs of `catch_up_matches_a_day_of_steps` after a warm compile, on trunk and on the head.
- [ ] Probe. Compile `sim` tests once per side. Then run `cd sim && env -u ARGV0 cargo test catch_up_matches_a_day_of_steps -- --exact --test-threads=1` three times per side. Interleave trunk and head. Record every elapsed time and the spread.
- [ ] Baseline. Record the trunk median first.
- [ ] Rule. Fail when the three-run spread on either side is larger than the gap, because that gap is noise. Otherwise fail when the head median is greater than 2 times the trunk median. The test must also pass on both sides.

**Review gate.** The operator reviews before merge.

- [ ] Copy the lane 10 screenshot into `/cursor/stores/self/media/LS-6-review-gather.png`.
- [ ] Record a 30 to 60 second video of a lonely person entering a mess that already has crew. Save it as `/cursor/stores/self/media/LS-6-review.mp4`.
- [ ] Post the screenshots and the video in chat. Stop at merge-ready. Wait for the operator.

**Merge.**

- [ ] Root's clean verdict at the exact head SHA.
- [ ] Bugbot triage done.
- [ ] Rebased onto the LS-5 head after the verdict, patch-id unchanged.
- [ ] The owner does not merge. The operator lands LS-6 last.

## Close the program

- [ ] Every box above is checked with its evidence.
- [ ] Reply to the operator with each PR, its verdict, and the merge-ready SHA. Do not report a merge unless the operator landed it.

## Appendix A. Prototype evidence

No prototype branch and no prototype SHA. The operator forbade repo writes, so open questions that are facts were read from `origin/main` at `eb43f1bd2f6655874cb019cef4741779371967b5`.

These facts are settled by that commit.

- `World::step` in `sim/src/world.rs` already raises hunger and thirst, moves crew, cooks from `sim/src/recipes.rs`, and logs the life events named in the intro. Fatigue is stored and never read back except to log it. `crew.skill` is stored and never read by the step.
- A running job keeps advancing when the crew leave. `advance_station` adds elapsed time for an existing job without calling `station_staffed` again. `docs/SIMULATION.md` says the opposite.
- Health 0 stays in `crew` and `tick_clinic` can raise it. `clinic_revival_is_logged` expects that.
- Drinking does not check the room. `eating_drinking_and_shortages_are_logged` drinks from a water store while the person stands elsewhere.
- `food_days` counts meals, bread, vegetables, and fish. `tick_needs` eats only `MealKg`.
- `goal_room` sends the whole ship to `first_room`. There is no social field, no spoilage field, and no use of the head rooms.
- Deck 5 requires one galley and then fills with more galleys, plus two bakeries. Deck 6 requires one restaurant and two messes. Deck 7 requires one clinic and one infirmary. The packed galley count is whatever `place_kind` fits. Do not hard-code it.
- `move_crew` returns without writing a point when the person is already in the goal room. `muster` places every person in a shared berth on that room centroid.
- The 3D client draws bodies, hovered names, and HUD totals. It does not read hunger, health, or activity. `CrewView` still sends hunger, health, and activity on `/api/world`.
- `RecycleMetal` outputs `MetalKg`. The doc sentence that calls metal finite does not match that recipe.
- `tick_needs` can set `Sleeping` and emit `asleep` before `tick_movement` sends that same person walking in the same minute. Jobs read the room from the start of the minute, before movement.

These product calls stay unproven. The defaults below are what the PRs implement. The operator can replace any default in their own words.

- Corpses stay aboard and do not get back up. The operator could say to remove them, or to allow revival.
- Departments split across the day on the existing offset, and hunger or thirst pulls someone off a post. The operator could say to keep one shared watch and to eat only off duty.
- Crew eat bread, fish, and vegetables when meals are gone. Medicine stays finite and is used only below health 0.55, and only with a medic present. The operator could say to manufacture medicine, or to keep meals as the only food.
- Company is one number. The operator could say to add named relationships.
- Perishables outside a cold store lose 2 percent per day. The operator could name a different rate.

## Appendix B. Alternatives rejected

A new life-event vocabulary loses because PR 13 already logs hunger, thirst, death, meals, water, medicine, sleep, and treatment. More lines would not make the crew behave.

A planner that scores every room every minute loses because `goal_room` is already the decision point. LS-2 replaces the order of that decision. A second brain beside it would split the rule.

A new ration good loses because `World::totals` already treats bread, fish, and vegetables as edible, and the HUD shows that number. LS-4 makes the bite match the number.

Removing bodies loses because the existing `died` line says the crew member stays aboard. LS-1 makes that sentence true by freezing the body.

A new pathfinder loses because `next_hop` and the 1.3 metres per second walk already cross doors. The visible failure is a shared point, which LS-5 fixes inside the room.

Named pairs and dialogue lose because nothing in the client renders speech. One companionship number changes where people stand, which the 3D view already shows.

A medicine recipe loses because `MedicalKg` has no producer and the endowment is 25 kg. Rationing the stock is the failure mode. A factory would hide it.

Door capacity loses because rooms have no occupancy limit. A blocked medic is worse than two people who do not overlap.

A porter job that hauls meals into the mess loses because `finish_recipe` can deposit `CookMeal` output in a mess directly. Hauling can wait until someone watches empty messes and full galleys.

Keeping revival as a compatibility path loses because the end state is that health 0 stays 0. LS-1 rewrites `clinic_revival_is_logged` in the same change.

## Appendix C. Risks

LS-1 rewrites `clinic_revival_is_logged`. LS-2 rewrites the drinker setup in `eating_drinking_and_shortages_are_logged`. An owner who leaves those tests green has not shipped the new rule.

`catch_up_to` will run the new rules across any downtime. The first boot after LS-1 leaves any stored health 0 crew at health 0. The owner watches that boot and treats a still body as the new rule.

Every PR edits `sim/src/world.rs`. A concurrent change to that file restacks the chain. Do not split a behavior across two PRs to dodge the conflict.

`git show origin/main:pstack/...` fails on this repo. The timehelm tree does not vendor those skills. The owner reads the same files from the local poteto-mode and cursor-team-kit plugins, and records the path in the decision trail.

Live lanes cannot wait out an 8 hour watch. Multi-hour rules are the unit tests that call `World::step`. Live lanes use one wall-clock minute, a fixture, and `/api/world`. Lane 9 of LS-5 is the exception and waits up to five minutes for one stair.

`water_days` still divides litres by crew times 3. Actual drinks are 0.12 L. This program does not retune that HUD number.

`make lint` also checks `ship-game` formatting. These PRs do not edit `ship-game`. If that check fails on trunk, record it and do not reformat `ship-game` here.

LS-6 is the only schema change. Editing `0001_sim_world.sql` changes its checksum and `classify_migration_history` will refuse the server. The new file is the only safe path. Production apply still needs a database user who can change the schema.

The day-step probe includes `World::generate` plus 1440 steps plus catch-up. Both sides include generate. The owner still compiles before timing so the number is not compile time. If the spread exceeds the gap, the owner reruns. They do not treat a single noisy sample as a regression.

A lane that screenshots bodies on the first frame, or reads names without hovering the room, is not evidence. Wait out the 60 second glide. Hover the room for names.

Editing `docs/SIMULATION.md` must leave `RecycleMetal` in place. The doc calls metal finite. The recipe produces it.

Assert end-of-step position, activity, and stock. An `asleep` line can belong to a person who is `Walking` at the end of that minute. A cook who must pause a job has to be outside the room before the step starts.

## Appendix D. Links and reading list

Read these before editing. Lines are from `eb43f1bd2f6655874cb019cef4741779371967b5`.

- `docs/SIMULATION.md` crew, goods, power, and authority sections.
- `sim/src/world.rs` `CrewMember`, `World::step`, `tick_needs`, `tick_clinic`, `tick_power_and_jobs`, `advance_station`, `tick_fishing`, `tick_wear`, `goal_room`, `move_crew`, `on_shift`, and `muster`.
- `sim/src/recipes.rs` `recipe` and the mass tests.
- `sim/src/kinds.rs` `Activity`, `Skill`, `Department`, `Good`, and `PowerClass`.
- `sim/src/layout.rs` `required`, `filler_kind`, and `default_recipe`.
- `sim/src/proto.rs` `CrewView` and `World::totals`.
- `server/src/db.rs` `migrate`, the crew `SELECT`, and the crew `INSERT`.
- `server/migrations/0001_sim_world.sql` crew columns.
- `ship-game/src/app_3d.rs` `sync_crew` and the HUD string that prints crew, food days, water days, and power.

LS-2 and LS-4 get `pstack/skills/interrogate/SKILL.md` only if the operator rejects the default in Appendix A. The plan already picked the watch split and the fallback foods. LS-3 gets `pstack/skills/how/SKILL.md` if the owner wants to change the skill multiplier. They do not reopen it by default.

The execution trail is a local TSV per `pstack/skills/show-me-your-work/SKILL.md`. Keep it at `/cursor/stores/self/internal/life-sim-decision-log.tsv`. Do not commit it unless the operator asks. The perf lane follows `pstack/skills/benchmark-checklist/SKILL.md` before anyone acts on the day-step number.

Test commands use `env -u ARGV0 cargo` because a Cursor shell can pass a cargo proxy name that cargo rejects. `make test` on this commit runs `sim`, `server`, and `ship-game` tests. Focused checks above are enough for the unit box. Run `make test` once on the final head if the owner wants the whole suite.
