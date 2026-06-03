# 1288: The Memorial Revolt

## 1. Overview
**Layer:** 1

**Fantasy:** Pops form emotional bonds with native fauna and demand permanent memorial sites when they die.

**Mechanic:** Sometimes, Pops adopt local fauna as "Colony Pets." If a Pet dies, the Pop demands a permanent "Memorial Site" be built. These sites consume valuable building space and resources. If refused, the Pop's mood plummets and they might go on strike.

## 2. Dependencies
- Fauna / Pet System
- Death Event System
- Grid / Building System
- Pop Mood & Strike System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // RED Phase Test Setup
    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<PetDeathEvent>();
        app.add_systems(Update, (
            handle_pet_death_system,
            process_memorial_demand_system,
        ));
        app
    }

    #[test]
    fn test_pet_death_triggers_memorial_demand() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            Mood { level: 100.0 },
        )).id();

        let pet = app.world_mut().spawn(ColonyPet { owner: pop }).id();

        app.world_mut().send_event(PetDeathEvent { pet_entity: pet, owner_entity: pop });
        app.update();

        // Check if Pop now has a MemorialDemand component
        assert!(app.world().get::<MemorialDemand>(pop).is_some(), "Pop should demand a memorial when their pet dies");
    }

    #[test]
    fn test_unfulfilled_memorial_demand_lowers_mood() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            Mood { level: 100.0 },
            MemorialDemand { timer: 10 }, // Expiring soon
        )).id();

        // Simulate time passing causing the demand to expire unfulfilled
        let mut demand = app.world_mut().get_mut::<MemorialDemand>(pop).unwrap();
        demand.timer = 0;

        app.update();

        let mood = app.world().get::<Mood>(pop).unwrap();
        assert!(mood.level < 100.0, "Pop mood should drop if memorial demand is not fulfilled in time");
        assert!(app.world().get::<OnStrike>(pop).is_some(), "Pop should go on strike if memorial is denied");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Mood {
    pub level: f32,
}

#[derive(Component)]
pub struct ColonyPet {
    pub owner: Entity,
}

#[derive(Event)]
pub struct PetDeathEvent {
    pub pet_entity: Entity,
    pub owner_entity: Entity,
}

#[derive(Component)]
pub struct MemorialDemand {
    pub timer: u32,
}

#[derive(Component)]
pub struct OnStrike;

pub fn handle_pet_death_system(
    mut commands: Commands,
    mut events: EventReader<PetDeathEvent>,
) {
    for event in events.read() {
        // Give them 100 ticks to build a memorial
        commands.entity(event.owner_entity).insert(MemorialDemand { timer: 100 });
    }
}

pub fn process_memorial_demand_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut MemorialDemand, &mut Mood)>,
) {
    for (entity, mut demand, mut mood) in pops.iter_mut() {
        if demand.timer > 0 {
            demand.timer -= 1;
        } else {
            // Demand expired unfulfilled
            mood.level -= 50.0;
            commands.entity(entity).insert(OnStrike);
            commands.entity(entity).remove::<MemorialDemand>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Memorial Fulfillment**: The GREEN phase only handles the failure case. Needs integration with the building placement system so that building a `MemorialSite` component targets the specific `Pop` and removes the `MemorialDemand` component.
- **Strike Nuance**: `OnStrike` currently just slaps a component on. It should likely hook into a broader Utility AI or job scheduling system to prevent work.
- **Timer Scaling**: Replace the hardcoded `100` tick timer with a proper configuration resource based on game speed.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- **Grid Placement**: The main tension is *space*. The memorial building should require a `GridPosition` and actually take up a physical tile on the Layer 1 map, denying it for industrial use.

## 8. Questions
*Builder: add questions here if spec is unclear.*
