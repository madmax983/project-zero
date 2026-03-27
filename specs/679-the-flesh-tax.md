# 679 - The Flesh Tax

## 1. Overview
The Flesh Tax is a Cross-layer (3 -> 1) mechanic where a dominant Layer 3 civilization demands a regular tribute of "biomass" (Pops) to ignore the player's borders. Fulfilling the tax removes Pops from the colony and applies negative mood and memory effects to the survivors. Failing to pay triggers an immediate, overwhelming orbital bombardment from Layer 3 to Layer 1.

## 2. Dependencies
- Layer 1 `Pop` entity.
- Layer 1 `Memory` system (to permanently scar survivors).
- Layer 3 Diplomatic events/timers (for tribute demand intervals).
- `Orbital Bombardment` (from `specs/659-orbital-bombardment.md`).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::memory::Memory;
    use crate::layer3::diplomacy::FleshTaxEvent;

    #[test]
    fn test_flesh_tax_tribute_removes_pops() {
        let mut app = App::new();
        app.add_systems(Update, process_flesh_tax_payment);

        let pop1 = app.world_mut().spawn((Pop {}, Name::new("Sacrifice 1"))).id();
        let pop2 = app.world_mut().spawn((Pop {}, Name::new("Survivor"))).id();

        // Trigger a flesh tax payment
        app.world_mut().send_event(FleshTaxPaymentEvent {
            tribute_pops: vec![pop1],
        });

        app.update();

        assert!(app.world().get_entity(pop1).is_none(), "Pop 1 should be despawned as tribute");
        assert!(app.world().get_entity(pop2).is_some(), "Pop 2 should survive");
    }

    #[test]
    fn test_flesh_tax_survivors_receive_scarred_memory() {
        let mut app = App::new();
        app.add_systems(Update, process_flesh_tax_payment);

        let pop1 = app.world_mut().spawn((Pop {}, Name::new("Sacrifice 1"))).id();
        let pop2 = app.world_mut().spawn((Pop {}, Name::new("Survivor"))).id();

        app.world_mut().send_event(FleshTaxPaymentEvent {
            tribute_pops: vec![pop1],
        });

        app.update();

        let memory = app.world().get::<Memory>(pop2);
        assert!(memory.is_some());
        assert!(memory.unwrap().memories.contains(&MemoryType::FleshTaxTrauma), "Survivor should have trauma memory");
    }

    #[test]
    fn test_flesh_tax_failure_triggers_orbital_bombardment() {
        let mut app = App::new();
        app.add_event::<OrbitalBombardmentEvent>();
        app.add_systems(Update, process_flesh_tax_failure);

        app.world_mut().send_event(FleshTaxFailedEvent {});

        app.update();

        let bombardment_events = app.world().resource::<Events<OrbitalBombardmentEvent>>();
        assert_eq!(bombardment_events.get_reader().len(&bombardment_events), 1, "Failure should trigger an orbital bombardment");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop {}

#[derive(Component, Default)]
pub struct Memory {
    pub memories: Vec<MemoryType>,
}

#[derive(PartialEq, Eq)]
pub enum MemoryType {
    FleshTaxTrauma,
}

#[derive(Event)]
pub struct FleshTaxPaymentEvent {
    pub tribute_pops: Vec<Entity>,
}

#[derive(Event)]
pub struct FleshTaxFailedEvent {}

#[derive(Event)]
pub struct OrbitalBombardmentEvent {
    pub target_layer: u8,
}

pub fn process_flesh_tax_payment(
    mut commands: Commands,
    mut events: EventReader<FleshTaxPaymentEvent>,
    mut query: Query<(Entity, Option<&mut Memory>), With<Pop>>,
) {
    for event in events.read() {
        for pop_entity in &event.tribute_pops {
            commands.entity(*pop_entity).despawn();
        }

        for (entity, mut memory_opt) in query.iter_mut() {
            if !event.tribute_pops.contains(&entity) {
                if let Some(mut memory) = memory_opt {
                    memory.memories.push(MemoryType::FleshTaxTrauma);
                } else {
                    commands.entity(entity).insert(Memory {
                        memories: vec![MemoryType::FleshTaxTrauma],
                    });
                }
            }
        }
    }
}

pub fn process_flesh_tax_failure(
    mut events: EventReader<FleshTaxFailedEvent>,
    mut bombardment_writer: EventWriter<OrbitalBombardmentEvent>,
) {
    for _event in events.read() {
        bombardment_writer.send(OrbitalBombardmentEvent {
            target_layer: 1,
        });
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** The memory allocation might be better suited as an isolated system that triggers from a generic `TragedyEvent` rather than hardcoding it into the tax payment system.
- **Code Smells:** Ensure that `tribute_pops` are actually alive and valid entities before despawning.
- **Performance:** Iterating through all Pops for memory updates scales linearly `O(n)`. If populations get massive, consider applying a global `ColonyModifier` instead.
- **API Improvements:** Include a way to select the tribute Pops strategically in the UI.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Failing the tax correctly emits an `OrbitalBombardmentEvent`.

## 7. Technical Guidance
- **Integration Points:** Connect the `FleshTaxFailedEvent` to the existing Layer 3 diplomacy timers and the `OrbitalBombardmentEvent` to `src/layer2/orbital_bombardment.rs` (if specced).
- **Code Structure:** Place the tax logic inside `src/layer3/diplomacy/` and the memory reactions in `src/layer1/memory.rs`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
