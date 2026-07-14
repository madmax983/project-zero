# 1274: Memory Smugglers

## 1. Overview
Pops with high stress can buy "synthesized memories" from a black market building (Memory Den) to temporarily overwrite their trauma. While this rapidly reduces stress, artificial memories can clash with their actual histories, causing severe dissociative traits or making them temporarily adopt unrelated job behaviors (like an engineer trying to cook).

## 2. Dependencies
- Job System (`specs/009-job-system.md`)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::psychology::stress::Stress;
    use crate::layer1::memory::{Memories, MemoryType, Memory};
    use crate::layer1::jobs::{Job, JobType};
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::entities::pop::Pop;
    use std::collections::HashSet;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MemorySmugglersPlugin);
        app.init_resource::<crate::shared::time::SimulationTime>();
        app
    }

    #[test]
    fn test_buy_synthesized_memory_reduces_stress() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            Stress { current: 80.0, max: 100.0, ..Default::default() },
            Memories::default(),
            Job::new(JobType::Engineer),
        )).id();

        let den = app.world_mut().spawn(MemoryDen).id();

        app.world_mut().resource_mut::<Events<BuySynthesizedMemory>>().send(
            BuySynthesizedMemory {
                pop_entity: pop,
                den_entity: den,
            }
        );

        app.update();

        let stress = app.world().get::<Stress>(pop).unwrap();
        assert!(stress.current < 80.0, "Stress should be reduced after buying memory");

        let memories = app.world().get::<Memories>(pop).unwrap();
        assert!(memories.items.iter().any(|m| matches!(m.memory_type, MemoryType::Synthesized(_))), "Pop should have acquired a synthesized memory");
    }

    #[test]
    fn test_synthesized_memory_causes_dissociation() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            Memories {
                items: vec![
                    Memory { memory_type: MemoryType::StarvationTrauma, intensity: 1.0, added_at: 0 },
                    Memory { memory_type: MemoryType::Synthesized(JobType::Chef), intensity: 1.0, added_at: 0 },
                ],
            },
            Traits(HashSet::new()),
        )).id();

        app.update();

        let traits = app.world().get::<Traits>(pop).unwrap();
        assert!(traits.has(Trait::Dissociative), "Pop should become dissociative when real trauma clashes with synthesized memories");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;
use crate::layer1::psychology::stress::Stress;
use crate::layer1::memory::{Memories, MemoryType, Memory};
use crate::layer1::jobs::JobType;
use crate::layer1::traits::{Traits, Trait};
use crate::layer1::entities::pop::Pop;
use crate::shared::time::SimulationTime;

#[derive(Component)]
pub struct MemoryDen;

#[derive(Event)]
pub struct BuySynthesizedMemory {
    pub pop_entity: Entity,
    pub den_entity: Entity,
}

pub struct MemorySmugglersPlugin;

impl Plugin for MemorySmugglersPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BuySynthesizedMemory>()
           .add_systems(Update, (
               process_memory_purchases,
               check_memory_dissociation,
           ));
    }
}

fn process_memory_purchases(
    mut events: EventReader<BuySynthesizedMemory>,
    mut query: Query<(&mut Stress, &mut Memories)>,
    time: Res<SimulationTime>,
) {
    for event in events.read() {
        if let Ok((mut stress, mut memories)) = query.get_mut(event.pop_entity) {
            stress.current = (stress.current - 40.0).max(0.0);
            memories.items.push(Memory {
                memory_type: MemoryType::Synthesized(JobType::Chef),
                intensity: 1.0,
                added_at: time.tick,
            });
        }
    }
}

fn check_memory_dissociation(
    mut query: Query<(Entity, &Memories, &mut Traits), With<Pop>>,
) {
    for (_entity, memories, mut traits) in query.iter_mut() {
        let has_trauma = memories.items.iter().any(|m| matches!(m.memory_type, MemoryType::StarvationTrauma));
        let has_synth = memories.items.iter().any(|m| matches!(m.memory_type, MemoryType::Synthesized(_)));

        if has_trauma && has_synth {
            if !traits.has(Trait::Dissociative) {
                traits.add(Trait::Dissociative);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: The purchased memory type (`JobType::Chef`) is hardcoded in the minimal implementation. This should be driven by the Memory Den's inventory or randomly selected based on the Pop's missing needs/desires.
- **Integration**: `MemoryType::Synthesized` and `Trait::Dissociative` need to be officially added to their respective enums in the psychology/traits modules if they don't exist yet.
- **Performance**: The dissociation check currently iterates over all pops and all their memories on every update. This could be slow. We should only check when a new memory is added (e.g. by reacting to a `MemoryAdded` event) rather than polling constantly.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops correctly reduce stress and gain synthesized memories when buying from a Memory Den.
- [ ] Clashing memories correctly apply the Dissociative trait.

## 7. Technical Guidance
- Make sure to extend the existing `MemoryType` and `Trait` enums gracefully. You may need to update `Memories::default()` or other initialization code if these enums change.
- In a full implementation, `MemoryDen` would probably need a system where Pops automatically path to it when their stress is high, similar to how they seek out food when hungry.

## 8. Questions
*Builder: add questions here if spec is unclear.*
