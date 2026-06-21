1. **Identify the missing seam for INT-1026: The Lottery -> Chronicle:** The `execute_lottery_system` despawns `Pop`s and adds `Traumatized` traits. It does not send any indication to the chronicle that a lottery happened.
2. **Create the bridge:** Add a system `lottery_chronicle_bridge` to `src/layer1/core/integration.rs` which observes `Policy::TheLottery` being removed from active policies when `execute_lottery_system` completes (this might be tricky since it's just removed. We need to maybe emit an event when the lottery executes). Let's see what `execute_lottery_system` does. It despawns pops and adds trauma, then removes the policy.
Wait, `execute_lottery_system` doesn't emit any event. It just mutates things. This makes it hard to bridge without changing the system or using `Added<Traumatized>` on multiple pops, which isn't robust.
Let's modify `execute_lottery_system` to emit a `LotteryExecutedEvent` or just change the code in `src/layer1/core/integration.rs` to detect the event. Actually, we should try not to modify the builder's code too much. Maybe `execute_lottery_system` could emit an event?
Let's see if we can just emit the `AddChronicleEvent` directly inside `execute_lottery_system`, or add a `LotteryExecutedEvent` to `src/layer1/the_lottery.rs`.
Wait, "Integrators prefer events over tight coupling. If system A needs to notify system B, use Bevy events. Don't add gameplay features, minimal additions."
I will add a `LotteryExecutedEvent { sacrifice_count: usize }` to `src/layer1/the_lottery.rs` and send it from `execute_lottery_system`.
Then I will write `lottery_chronicle_bridge` in `src/layer1/core/integration.rs` that reads `LotteryExecutedEvent` and writes `AddChronicleEvent`.
3. **Register systems:** Add the event to `src/setup.rs`? Actually, where are events registered? `app.add_event::<LotteryExecutedEvent>()` in `src/simulation.rs` or `src/setup.rs` or `src/layer1/mod.rs`? Let's check `src/setup.rs` and `src/simulation.rs`.
4. **Schedule ordering:** The bridge system `lottery_chronicle_bridge` goes into `Layer1SystemSet::Observation` in `src/layer1/systems/observation.rs`.
5. **Write tests:** `tests/integration/lottery_chronicle_bridge.rs`.
6. **Update SEAM_MAP.md.**
7. **Complete INT-1026 in design/IN_PROGRESS.md to COMPLETED.md.**
