# The Attrition of Immortality

## 1. Overview
Pops that undergo extreme life-extension therapies no longer die of old age, becoming immortal specialists with hyper-productivity. However, their brains eventually run out of storage space for `Memories`. To continue functioning, they must systematically delete early memories—such as family bonds, empathy, and leisure desires—degrading into unfeeling sociopaths that radicalize the mortal population and cause massive social unrest.

## 2. Dependencies
- `019-pop-needs.md` (for `Needs` like empathy and leisure)
- `112-pop-memory.md` (for the `Memory` system)
- `198-unrest-mechanics.md` (for unrest triggers and radicalization)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_immortal_pop_does_not_die_of_old_age() {
    let mut app = App::new();
    // Setup standard Pop and immortal Pop (with LifeExtension therapy)
    // Advance time beyond normal lifespan
    // Assert standard Pop dies
    // Assert immortal Pop remains alive
}

#[test]
fn test_immortal_pop_reaches_memory_capacity_and_deletes() {
    let mut app = App::new();
    // Setup immortal Pop with maximum memory capacity
    // Attempt to add new Memory
    // Assert oldest or emotionally-tagged memory (e.g., family bond) is deleted
    // Assert new memory is added
}

#[test]
fn test_deleted_empathy_triggers_sociopathic_action() {
    let mut app = App::new();
    // Setup immortal Pop with deleted "Empathy" memory/need
    // Trigger localized disaster affecting Pop's lineage
    // Assert Pop performs hyper-efficient, ruthless action (e.g., bulldoze rubble immediately)
}

#[test]
fn test_sociopathic_action_radicalizes_mortals() {
    let mut app = App::new();
    // Trigger sociopathic action by immortal Pop
    // Assert nearby mortal Pops gain significant Stress/Unrest
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct Immortal;

#[derive(Component)]
pub struct MemoryCapacity(pub usize);

#[derive(Component)]
pub struct Memories {
    pub logs: Vec<String>,
}

#[derive(Component)]
pub struct SociopathyLevel(pub f32);

#[derive(Event)]
pub struct SociopathicActionEvent {
    pub perpetrator: Entity,
    pub location: Vec3,
}

pub fn immortal_aging_system(
    mut query: Query<(&mut Age, Option<&Immortal>)>,
) {
    // Normal aging logic applies, but Immortals won't trigger death based on Age
}

pub fn memory_attrition_system(
    mut query: Query<(&mut Memories, &MemoryCapacity, &mut SociopathyLevel), With<Immortal>>,
) {
    for (mut memories, capacity, mut sociopathy) in query.iter_mut() {
        if memories.logs.len() > capacity.0 {
            // Delete the oldest memory
            memories.logs.remove(0);
            // Increase sociopathy as empathy/bonds are forgotten
            sociopathy.0 += 10.0;
        }
    }
}

pub fn trigger_sociopathic_action_system(
    mut events: EventWriter<SociopathicActionEvent>,
    query: Query<(Entity, &SociopathyLevel, &Transform)>,
) {
    for (entity, sociopathy, transform) in query.iter() {
        if sociopathy.0 > 50.0 {
            // Trigger ruthless action
            events.send(SociopathicActionEvent {
                perpetrator: entity,
                location: transform.translation,
            });
        }
    }
}

pub fn radicalize_mortals_system(
    mut events: EventReader<SociopathicActionEvent>,
    mut mortals: Query<(&mut Unrest, &Transform), Without<Immortal>>,
) {
    for event in events.read() {
        for (mut unrest, transform) in mortals.iter_mut() {
            if transform.translation.distance(event.location) < 50.0 {
                unrest.0 += 25.0; // Significant unrest spike
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: `Memories.logs` as a `Vec<String>` is inefficient. It should use actual `Memory` structs with timestamps and emotional weights.
- **Refactoring Opportunities**: Integrate the memory deletion process into the main `MemorySystem` so standard Pops also have memory limits, but Immortals hit it frequently over centuries.
- **Integration Points**: Connect `SociopathyLevel` to the Utility AI to naturally drive the Pop toward hyper-efficient, unfeeling actions instead of hardcoding an event trigger based purely on a threshold.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Immortal Pops survive beyond normal lifespans
- [ ] Memories are deleted when capacity is reached, increasing Sociopathy
- [ ] Sociopathic actions cause unrest in nearby mortal Pops

## 7. Technical Guidance
- **Code Structure**: The `MemoryCapacity` component should be a config value, possibly scaling with specific cybernetic augmentations.
- **Gotchas**: Ensure deleting a "family bond" actually severs the logical link between Pop entities if a family graph system exists.

## 8. Questions
*Builder: add questions here if spec is unclear.*
