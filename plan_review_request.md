# Requesting Plan Review

Okay, I have examined spec `768` (Dynastic Succession) which was recently completed but lacks integration.
The requirement says:
`Integrate the succession event into the Chronicle system (src/layer1/chronicle.rs) using a new template.`

But `src/layer3/diplomacy/succession.rs` does not emit any events currently, only mutating components (`Dead`, `Leader`, `SuccessionCrisis`).
The Integrator should write integration code without modifying the builder's code too much. So I should add an event like `SuccessionEvent` or `SuccessionCrisisEvent` that gets fired, or perhaps a system in `src/layer3/integration.rs` (or similar) that queries for `Added<SuccessionCrisis>` and `Added<Leader>` to send an `AddChronicleEvent`.

Let's check `tests/integration/` for `dynastic_succession_chronicle.rs` or similar. We should create `tests/integration/dynastic_succession_chronicle.rs`.
Wait, in `src/layer3/integration.rs` does it exist? Or `src/layer2/integration.rs`? I will create the bridge and the test.
The problem is: how do I know the succession actually occurred across frames without an event?
Wait! `Added<Leader>` is perfect for tracking a new leader taking over, and `Added<SuccessionCrisis>` is perfect for a crisis!

Wait, `Added<Leader>` could match newly spawned leaders at the start of the game. I could use `Added<Leader>, Without<Age>` perhaps, or something specific, but actually, checking `Added<SuccessionCrisis>` and maybe `Changed<CurrentLeader>` is better!
But `Changed<CurrentLeader>` requires `CurrentLeader`. Yes, `CurrentLeader` is changed!

So the plan:
1. Write RED phase tests in `tests/integration/dynastic_succession_chronicle.rs` for:
    - New leader succession sending `AddChronicleEvent`
    - Succession crisis sending `AddChronicleEvent`
2. Add a bridge system in `src/layer3/integration.rs` (or create it if not exist, or `src/layer1/integration.rs` depending on where it belongs. Layer 3 -> 1, probably `src/layer1/integration.rs` or `src/layer3/integration.rs`). I will use `layer1/integration.rs` to emit `AddChronicleEvent`. Wait, `layer2/integration.rs` bridges L2 to L1. I'll check where other layer3 bridges are.
3. Update `src/simulation.rs` to register the new bridge system.
4. Update `design/SEAM_MAP.md` and `design/COMPLETED.md`.
