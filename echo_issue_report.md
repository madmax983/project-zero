# Echo's Report

## 🗣️ Echo: Missing `NarrativeContext` import in README example
🤦 **The Confusion:** Tried to run the "Procedural Generation (Narrative)" code from `README.md`. The compiler said `NarrativeContext` not found! But the example has `use scale::shared::narrative::NarrativeGenerator;`!
🕵️ **The Reality:** The example code uses `NarrativeContext` but does not import it. `use scale::shared::narrative::NarrativeGenerator;` only imports the generator.
💡 **The Fix:** Change the import in `README.md` to `use scale::shared::narrative::{NarrativeContext, NarrativeGenerator};`.
## 🗣️ Echo: "Origin Tick" in Story structure leaks simulation jargon to players
🤦 **The Confusion:** When reading the API for `Story`, it has `origin_tick: u64`. Wait, "tick"? Does the legend of "The Great Frost" happen at "Tick 100"? That feels like simulation jargon, not a story concept!
🕵️ **The Reality:** The internal simulation uses `tick` for time. While accurate, calling it `origin_tick` in a narrative structure is jarring.
💡 **The Fix:** Since `Chronicle` events also use `tick`, it's consistent within the engine, but for UI/Narrative purposes, something like "timestamp" or "historical_date" might be softer, or it should at least be abstracted in the output. Just flagging this as a minor slang check!
## 🗣️ Echo: Jargon Check ("bitemporal adjacency" vs "history") in Oral Tradition API
🤦 **The Confusion:** The `Oral Tradition` example says `world.resource_mut::<Chronicle>().add_event(...)` and `collect_chronicles_system` to generate a `Story`. It feels a bit like "SimulationTick vs LastProcessedTick" internal mechanics leaking into the user API.
🕵️ **The Reality:** The internal logic requires manually advancing `SimulationTime` to make `collect_chronicles_system` process the event properly if running minimal API, since `collect_chronicles_system` checks if an event's `tick > tradition.last_processed_tick`.
💡 **The Fix:** Provide a more straightforward `Storyteller` or `Tavern` setup snippet without making me manage `SimulationTime` loops just to verify a story generation.

## 🗣️ Echo: `NarrativeGenerator` error for missing context doesn't trigger Rust error type, just outputs text with `[ERROR: VAR]`
🤦 **The Confusion:** I ran the example code from `README.md` intentionally missing a context variable, and expected it to return an `Err` from the `generate` function because of `?`. Instead, it returned `Ok` with `[ERROR: KEY]` injected into the text! "Result is Ok but there is an error in my string?"
🕵️ **The Reality:** The generation algorithm replaces missing context variables with a literal string `[ERROR: KEY]` and returns an `Ok(String)`. `NarrativeSegment::Error` implements `Display` by formatting as `[ERROR: {s}]`. It does not fail the generation `Result`.
💡 **The Fix:** Document clearly in the API and examples that missing context variables won't fail the Result but will produce formatted placeholders in the string, OR change `generate()` to return an `Err` if required context is missing. (README currently mentions it: "Note: Missing context variables will appear as [ERROR: KEY] in the text.", but it doesn't fail the `?`).
