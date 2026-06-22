1. **Understand Zoo Hypothesis Mechanics:**
`src/layer3/zoo_hypothesis.rs` tracks an `entertainment_score` in `AlienObservers`.
When it goes above `threshold`, it spawns a `DropPod` in `trigger_alien_reward_system`.
When it goes below 0, it gets clamped.
Wait, let's create a new event `AlienRewardEvent` or just detect `Added<DropPod>` in our bridge. `Added<DropPod>` is clean.

```rust
pub fn zoo_hypothesis_chronicle_bridge(
    query: Query<(), Added<DropPod>>,
    mut chronicle: EventWriter<AddChronicleEvent>,
) {
    for _ in query.iter() {
        chronicle.send(AddChronicleEvent {
            importance: EventImportance::Major,
            text: "An anomalous supply drop has fallen from orbit. Are we being watched?".to_string(),
        });
    }
}
```

Wait, `Added<DropPod>` requires importing `DropPod` from `crate::layer3::zoo_hypothesis::DropPod`.

2. **Add bridge function to `src/layer3/integration.rs`:**
```rust
use crate::layer3::zoo_hypothesis::DropPod;

pub fn zoo_hypothesis_chronicle_bridge(
    query: Query<(), Added<DropPod>>,
    mut chronicle: EventWriter<AddChronicleEvent>,
) {
    for _ in query.iter() {
        chronicle.send(AddChronicleEvent {
            importance: EventImportance::Major,
            text: "An anomalous supply drop has fallen from orbit. Are we being watched?".to_string(),
        });
    }
}
```

3. **Register systems:**
Register `evaluate_colony_entertainment_system` and `trigger_alien_reward_system` to run in `simulation.rs`. We'll also need to initialize the `AlienObservers` resource in `setup.rs`? Actually, looking at `trigger_alien_reward_system` it requires `AlienObservers`.
Oh wait, the prompt says "Integration: The Zoo Hypothesis -> Chronicle", maybe I should just wire the chronicle part, or do I also need to register the systems? "Find completed features that aren't talking to each other, write integration code and tests to connect them". So I should add `zoo_hypothesis_chronicle_bridge` and register it. Let's see if the systems are registered. No, `AlienObservers` doesn't seem to be initialized in `src/setup.rs` and the systems aren't registered in `src/simulation.rs`. I should initialize it and add the systems.

4. **Integration Test:**
Write `tests/integration/zoo_hypothesis_chronicle_bridge.rs`.
```rust
use bevy_ecs::prelude::*;
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer3::zoo_hypothesis::{AlienObservers, DropPod, evaluate_colony_entertainment_system, trigger_alien_reward_system};
use scale::layer3::integration::zoo_hypothesis_chronicle_bridge;
// Add test that simulates high entertainment, triggers reward, and checks that chronicle event is emitted.
```

5. **Update COMPLETED.md and SEAM_MAP.md**:
Mark `INT-1029` in `COMPLETED.md` and `IN_PROGRESS.md`, update `SEAM_MAP.md`.
