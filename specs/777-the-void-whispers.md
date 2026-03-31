# 777: The Void Whispers

## 1. Overview
**Layer:** Cross-layer (2 -> 1)
**Fantasy:** Deep space exploration isn't just about finding resources; it's about what the darkness does to the people who go there.
**Mechanic:** Exploratory fleets returning from uncharted nodes bring back "Void Whispers"—subtle memetic traits. When they return, they spread these traits through the Rumor Web, manifesting as bizarre religious cults or strange aversions in the colony.

## 2. Dependencies
- Layer 2 exploration mechanics
- Layer 1 Pop/Memetic contagion system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_explorers_accumulate_void_whispers() {
        let mut app = App::new();
        app.add_systems(Update, accumulate_void_whispers_in_deep_space);

        let fleet_entity = app.world_mut().spawn((
            ExplorerFleet,
            DeepSpaceExposure { ticks: 100 },
        )).id();

        app.update();

        let whispers = app.world().get::<VoidWhispers>(fleet_entity);
        assert!(whispers.is_some(), "Fleet exposed to deep space should accumulate Void Whispers");
        assert_eq!(whispers.unwrap().intensity, 10.0);
    }

    #[test]
    fn test_returning_fleet_infects_colony() {
        let mut app = App::new();
        app.add_event::<FleetReturnedEvent>();
        app.add_systems(Update, spread_whispers_to_colony);

        let fleet_entity = app.world_mut().spawn((
            VoidWhispers { intensity: 50.0 },
        )).id();

        let colony_pop_entity = app.world_mut().spawn((
            ColonyPop,
        )).id();

        app.world_mut().send_event(FleetReturnedEvent {
            fleet: fleet_entity,
            colony: Entity::PLACEHOLDER, // Generic target for test
        });

        app.update();

        // Colony pop should now have an infection or meme component
        let meme = app.world().get::<MemeticInfection>(colony_pop_entity);
        assert!(meme.is_some(), "Colony pop should receive MemeticInfection from returning fleet");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct ExplorerFleet;

#[derive(Component)]
pub struct DeepSpaceExposure {
    pub ticks: u32,
}

#[derive(Component)]
pub struct VoidWhispers {
    pub intensity: f32,
}

#[derive(Component)]
pub struct ColonyPop;

#[derive(Component)]
pub struct MemeticInfection;

#[derive(Event)]
pub struct FleetReturnedEvent {
    pub fleet: Entity,
    pub colony: Entity,
}

pub fn accumulate_void_whispers_in_deep_space(
    mut commands: Commands,
    mut fleets: Query<(Entity, &DeepSpaceExposure), With<ExplorerFleet>>,
) {
    for (entity, exposure) in fleets.iter_mut() {
        if exposure.ticks >= 100 {
            commands.entity(entity).insert(VoidWhispers { intensity: 10.0 });
        }
    }
}

pub fn spread_whispers_to_colony(
    mut commands: Commands,
    mut events: EventReader<FleetReturnedEvent>,
    fleets_with_whispers: Query<&VoidWhispers>,
    pops: Query<Entity, With<ColonyPop>>,
) {
    for event in events.read() {
        if fleets_with_whispers.get(event.fleet).is_ok() {
            // Infect all pops in the colony
            for pop_entity in pops.iter() {
                commands.entity(pop_entity).insert(MemeticInfection);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**: The contagion should use the existing Rumor Web or proximity spread (e.g., `MemeCarrier`) instead of instantly infecting all `ColonyPop`s globally.
- **Performance**: Replace global `Query` on pops with an event that a memetic hub processes to begin a slow contagion.
- **API Improvements**: Differentiate whisper effects (e.g. geometric carving vs religious chanting) by giving `MemeticInfection` variants or data fields.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified: Deep space exposure creates VoidWhispers, returning fleets spread MemeticInfection.

## 7. Technical Guidance
- The "MemeticInfection" should tie into pop productivity or morale mechanics, representing the disruptive behavior (carving non-Euclidean patterns).
- Utilize the `nova` feature memetics system if applicable.

## 8. Questions
*Builder: add questions here if spec is unclear.*
