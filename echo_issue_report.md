# Echo's Report

## 🗣️ Echo: Missing `NarrativeContext` import in README example
🤦 **The Confusion:** Tried to run the "Procedural Generation (Narrative)" code from `README.md`. The compiler said `NarrativeContext` not found! But the example has `use scale::shared::narrative::NarrativeGenerator;`!
🕵️ **The Reality:** The example code uses `NarrativeContext` but does not import it. `use scale::shared::narrative::NarrativeGenerator;` only imports the generator.
💡 **The Fix:** Change the import in `README.md` to `use scale::shared::narrative::{NarrativeContext, NarrativeGenerator};`.
✅ **Resolution:** Verified that this was already fixed. The `README.md` now correctly imports `NarrativeContext`.

## 🗣️ Echo: "Origin Tick" in Story structure leaks simulation jargon to players
🤦 **The Confusion:** When reading the API for `Story`, it has `origin_tick: u64`. Wait, "tick"? Does the legend of "The Great Frost" happen at "Tick 100"? That feels like simulation jargon, not a story concept!
🕵️ **The Reality:** The internal simulation uses `tick` for time. While accurate, calling it `origin_tick` in a narrative structure is jarring.
💡 **The Fix:** Since `Chronicle` events also use `tick`, it's consistent within the engine, but for UI/Narrative purposes, something like "timestamp" or "historical_date" might be softer, or it should at least be abstracted in the output. Just flagging this as a minor slang check!
✅ **Resolution:** Verified that this was already fixed. The `Story` struct in `src/layer1/oral_tradition.rs` and the examples in `README.md` now use `historical_date` instead of `origin_tick`.

## 🗣️ Echo: Jargon Check ("bitemporal adjacency" vs "history") in Oral Tradition API
🤦 **The Confusion:** The `Oral Tradition` example says `world.resource_mut::<Chronicle>().add_event(...)` and `collect_chronicles_system` to generate a `Story`. It feels a bit like "SimulationTick vs LastProcessedTick" internal mechanics leaking into the user API.
🕵️ **The Reality:** The internal logic requires manually advancing `SimulationTime` to make `collect_chronicles_system` process the event properly if running minimal API, since `collect_chronicles_system` checks if an event's `tick > tradition.last_processed_tick`.
💡 **The Fix:** Provide a more straightforward `Storyteller` or `Tavern` setup snippet without making me manage `SimulationTime` loops just to verify a story generation.
✅ **Resolution:** Verified that this was already fixed. The `README.md` now provides a simple snippet to add a story via `tradition.add_story(story);` without requiring Bevy ECS boilerplate.

## 🗣️ Echo: `NarrativeGenerator` error for missing context doesn't trigger Rust error type, just outputs text with `[ERROR: VAR]`
🤦 **The Confusion:** I ran the example code from `README.md` intentionally missing a context variable, and expected it to return an `Err` from the `generate` function because of `?`. Instead, it returned `Ok` with `[ERROR: KEY]` injected into the text! "Result is Ok but there is an error in my string?"
🕵️ **The Reality:** The generation algorithm replaces missing context variables with a literal string `[ERROR: KEY]` and returns an `Ok(String)`. `NarrativeSegment::Error` implements `Display` by formatting as `[ERROR: {s}]`. It does not fail the generation `Result`.
💡 **The Fix:** Document clearly in the API and examples that missing context variables won't fail the Result but will produce formatted placeholders in the string, OR change `generate()` to return an `Err` if required context is missing. (README currently mentions it: "Note: Missing context variables will appear as [ERROR: KEY] in the text.", but it doesn't fail the `?`).
✅ **Resolution:** Fixed. Modified `NarrativeGenerator` to return an explicit `Err` on missing context variables, with actionable instructions like: `Missing required context variable or fragment: 'CIV_EPITHET'. Please add it using context.insert("CIV_EPITHET", <value>).`

## 🗣️ Echo: Getting Started example is broken
🤦 **The Confusion:** Tried to run the "Procedural Generation (Narrative)" code from `README.md`. The docs literally start with a huge blockquote: `> **⚠️ REQUIRES FEATURE NOVA**`. So I tried running it with `--features nova` which works, but then I read the text below the code block which says: "Note: This is the **base narrative system** ... It is available in the default build." Wait, what? So I don't need the `nova` feature? I tried running my code again without `--features nova` and it compiled and worked perfectly. I wasted time trying to figure out feature flags for no reason!
🕵️ **The Reality:** The `NarrativeGenerator` and `NarrativeContext` are part of `scale::shared::narrative` which is compiled in the base project. The massive warning banner at the top of the "Procedural Generation (Narrative)" section was either copy-pasted from the "Oral Tradition (Nova Feature)" section below it, or mistakenly put there. The feature flag is only required for the "Oral Tradition" module.
💡 **The Fix:** Delete the `> **⚠️ REQUIRES FEATURE NOVA**` warning banner entirely from the `Procedural Generation (Narrative)` section in `README.md`. It's lying to the user.
✅ **Resolution:** Verified that this was already fixed. The `README.md` no longer has this warning in the "Procedural Generation (Narrative)" section.

## Final Result of Audit
I've updated `echo_issue_report.md` with resolutions. The error text fix for `NarrativeGenerator` is implemented and actionable.

## 🗣️ Echo: The Import Scan Failure - Too many nested imports
🤦 **The Confusion:** When I try to run the basic examples, I have to import multiple different things from deep nested modules like `scale::setup::{setup_world_with_config, SetupConfig}`, `scale::simulation::run_simulation_tick`, and `scale::shared::time::SimulationTime`. I just want to write the code and not worry about internal structure!
🕵️ **The Reality:** The library did not have a `prelude` module, so users were forced to learn the internal directory structure to get the basic pieces required for a headless loop or a narrative generation.
💡 **The Fix:** Created a `scale::prelude` module that exports the most commonly used types and functions (like `SetupConfig`, `SimulationTime`, `run_simulation_tick`, `NarrativeGenerator`, and `OralTradition`). Updated the README and examples to just use `use scale::prelude::*;`.
✅ **Resolution:** Implemented. `src/prelude.rs` added and docs updated.

## 🗣️ Echo: Headless example crashes on startup
🤦 **The Confusion:** I copied the basic headless simulation example from the README and ran it, but it immediately crashed with a weird error: `scale::layer1::social::ghost_shift_strike::evaluate_ghost_shifts could not access system parameter ResMut<'_, Events<GhostShiftStartedEvent>>`. I just wanted to run 10 ticks!
🕵️ **The Reality:** The internal engine forgot to initialize an event required by a background system.
💡 **The Fix:** The developers need to register `GhostShiftStartedEvent` properly in the setup so the simulation doesn't crash for basic users.
