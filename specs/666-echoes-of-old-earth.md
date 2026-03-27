# The Echoes of Old Earth (Spec 666)

## 1. Overview
Nostalgia weaponized into a memetic virus that paralyzes progress. A Layer 3 probe from Old Earth (or a highly convincing simulation) arrives, broadcasting perfectly preserved media, art, and idealized historical records. Pops who consume this media contract "Old Earth Melancholy." They become deeply dissatisfied with their current lives, demanding impossible, archaic luxuries (like "real coffee" or "paper books") and refusing to use "unnatural" xeno-tech or cybernetics.

## 2. Dependencies
- `src/layer1/pop.rs` (Pop entity, traits, needs, and task assignment)
- `src/layer1/items.rs` (Items, luxuries, and their tags)
- `src/layer3/probes.rs` (Interstellar probes and events)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_melancholy_contraction() {
        let mut app = App::new();
        app.add_systems(Update, process_probe_broadcast_system);

        let pop = app.world_mut().spawn((
            Pop::default(),
            PopTraits::default(),
            Needs { leisure: 10.0, ..default() }, // High need for leisure makes them susceptible
        )).id();

        // Act: Old Earth Probe arrives and broadcasts
        app.world_mut().send_event(OldEarthBroadcastEvent {
            intensity: 100.0,
        });
        app.update();

        // Assert: Pop contracts the trait
        let traits = app.world().get::<PopTraits>(pop).unwrap();
        assert!(traits.has_trait(TraitType::OldEarthMelancholy));
    }

    #[test]
    fn test_melancholy_needs_shift() {
        let mut app = App::new();
        app.add_systems(Update, apply_melancholy_needs_system);

        let pop = app.world_mut().spawn((
            Pop::default(),
            PopTraits::new(vec![TraitType::OldEarthMelancholy]),
            Needs { luxury: 0.0, ..default() },
            DemandState::default(),
        )).id();

        // Act: System updates demands based on traits
        app.world_mut().send_event(UpdateDemandsEvent { pop });
        app.update();

        // Assert: Pop now demands specific Old Earth luxuries and rejects Xeno tech
        let demands = app.world().get::<DemandState>(pop).unwrap();
        assert!(demands.requires_item_tag(ItemTag::OldEarthArtifact));
        assert!(demands.rejects_item_tag(ItemTag::XenoTech));
        assert!(demands.rejects_item_tag(ItemTag::Cybernetics));
    }

    #[test]
    fn test_melancholy_work_refusal() {
        let mut app = App::new();
        app.add_systems(Update, check_work_refusal_system);

        let melancholy_pop = app.world_mut().spawn((
            Pop::default(),
            PopTraits::new(vec![TraitType::OldEarthMelancholy]),
            CurrentTask { task_type: TaskType::OperateXenoMachine },
        )).id();

        // Act: Check if the pop will perform the task
        app.world_mut().send_event(TaskEvaluationEvent { pop: melancholy_pop });
        app.update();

        // Assert: The pop refuses the task because it involves Xeno tech
        let task = app.world().get::<CurrentTask>(melancholy_pop);
        assert!(task.is_none() || task.unwrap().status == TaskStatus::Refused);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Pop {}

#[derive(Component, Default)]
pub struct Needs {
    pub leisure: f32,
    pub luxury: f32,
}

#[derive(Component, Default)]
pub struct PopTraits {
    pub traits: Vec<TraitType>,
}

impl PopTraits {
    pub fn new(traits: Vec<TraitType>) -> Self {
        Self { traits }
    }

    pub fn has_trait(&self, trait_type: TraitType) -> bool {
        self.traits.contains(&trait_type)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum TraitType {
    OldEarthMelancholy,
}

#[derive(Event)]
pub struct OldEarthBroadcastEvent {
    pub intensity: f32,
}

#[derive(Component, Default)]
pub struct DemandState {
    pub required_tags: Vec<ItemTag>,
    pub rejected_tags: Vec<ItemTag>,
}

impl DemandState {
    pub fn requires_item_tag(&self, tag: ItemTag) -> bool {
        self.required_tags.contains(&tag)
    }
    pub fn rejects_item_tag(&self, tag: ItemTag) -> bool {
        self.rejected_tags.contains(&tag)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ItemTag {
    OldEarthArtifact,
    XenoTech,
    Cybernetics,
}

#[derive(Event)]
pub struct UpdateDemandsEvent {
    pub pop: Entity,
}

#[derive(Component)]
pub struct CurrentTask {
    pub task_type: TaskType,
    pub status: TaskStatus,
}

#[derive(Clone, PartialEq, Debug)]
pub enum TaskType {
    OperateXenoMachine,
    NormalWork,
}

#[derive(Clone, PartialEq, Debug)]
pub enum TaskStatus {
    Assigned,
    Refused,
}

#[derive(Event)]
pub struct TaskEvaluationEvent {
    pub pop: Entity,
}

pub fn process_probe_broadcast_system(
    mut events: EventReader<OldEarthBroadcastEvent>,
    mut query: Query<(&mut PopTraits, &Needs)>,
) {
    for event in events.read() {
        for (mut traits, needs) in query.iter_mut() {
            // Simplified condition: high leisure need makes them consume the media and get infected
            if needs.leisure > 5.0 && event.intensity > 50.0 {
                if !traits.has_trait(TraitType::OldEarthMelancholy) {
                    traits.traits.push(TraitType::OldEarthMelancholy);
                }
            }
        }
    }
}

pub fn apply_melancholy_needs_system(
    mut events: EventReader<UpdateDemandsEvent>,
    mut query: Query<(&PopTraits, &mut DemandState)>,
) {
    for event in events.read() {
        if let Ok((traits, mut demands)) = query.get_mut(event.pop) {
            if traits.has_trait(TraitType::OldEarthMelancholy) {
                demands.required_tags.push(ItemTag::OldEarthArtifact);
                demands.rejected_tags.push(ItemTag::XenoTech);
                demands.rejected_tags.push(ItemTag::Cybernetics);
            }
        }
    }
}

pub fn check_work_refusal_system(
    mut events: EventReader<TaskEvaluationEvent>,
    mut query: Query<(&PopTraits, &mut CurrentTask)>,
) {
    for event in events.read() {
        if let Ok((traits, mut task)) = query.get_mut(event.pop) {
            if traits.has_trait(TraitType::OldEarthMelancholy) && task.task_type == TaskType::OperateXenoMachine {
                task.status = TaskStatus::Refused;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration with Utility AI:** Refusing tasks shouldn't just set a status; it needs to affect the `UtilityAI` scoring system so the pop actively avoids tasks involving Xeno tech or cybernetics.
- **Item Demand System:** The `DemandState` needs to integrate with the colony's supply and logistics system, driving unhappiness if `OldEarthArtifact` items are not available.
- **Infection Spread:** How does the media spread? Does it jump from person to person (like gossip), or only via direct exposure to the probe's broadcast? Consider tying it into the existing `Rumor` or memory systems.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops exposed to the Old Earth Broadcast contract the Melancholy trait based on existing needs.
- [ ] Infected pops alter their demand states, requesting old artifacts and rejecting alien/cyber tech.
- [ ] Infected pops refuse tasks that conflict with their new ideology.

## 7. Technical Guidance
- **Layer 3 Probe Event:** The broadcast event (`OldEarthBroadcastEvent`) should originate from Layer 3, triggered by the arrival of a specific object in the system. Ensure the event correctly bridges down to Layer 1.
- **Tag Integration:** Check the existing item system in `src/layer1/items.rs` or `src/layer1/economy/items.rs` to ensure the new tags (`OldEarthArtifact`, `XenoTech`, `Cybernetics`) are properly integrated and supported by existing systems.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
