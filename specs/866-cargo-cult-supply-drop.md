# 866: The Cargo Cult of the Supply Drop

## 1. Overview
When a colony's communication array fails, it relies on automated blind supply drops from Layer 2. If the communication is down for too long, low-intellect or high-stress Pops begin worshipping the supply drones. They form a "Cargo Cult," stopping all meaningful labor to build massive, useless effigies of the drones in hopes of attracting more food.

## 2. Dependencies
- Pop components (`Stress`, `Traits` / `Intellect`, `Job` / `Work`)
- Colony infrastructure (`ColonyResources`, communications status)
- Cargo / Supply delivery events
- `TraumaTracker` for integration upon comms restoration

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_cargo_cult_formation_on_extended_comms_loss() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_plugins(CargoCultPlugin);

        // Setup colony with down communications
        let colony = app.world_mut().spawn(ColonyCore {
            comms_online: false,
            comms_downtime_ticks: 1500, // Very long time
        }).id();

        // Spawn highly stressed pop
        let pop = app.world_mut().spawn((
            Pop,
            ColonyLocation(colony),
            StressTracker { current: 95.0, ..default() },
            Job::Miner,
        )).id();

        // Act
        app.update();

        // Assert: Pop should have joined the cult and changed jobs
        assert!(app.world().get::<CargoCultist>(pop).is_some());
        assert_eq!(app.world().get::<Job>(pop).unwrap(), &Job::EffigyBuilder);
    }

    #[test]
    fn test_comms_restoration_causes_trauma() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(CargoCultPlugin);
        app.add_event::<CommsRestoredEvent>();
        app.add_event::<TraumaEvent>();

        let pop = app.world_mut().spawn((
            Pop,
            CargoCultist,
            StressTracker { current: 50.0, ..default() },
        )).id();

        // Act: Restore communications
        app.world_mut().send_event(CommsRestoredEvent);
        app.update();

        // Assert: Pop loses cult status and gains trauma
        assert!(app.world().get::<CargoCultist>(pop).is_none());

        let trauma_events = app.world().resource::<Events<TraumaEvent>>();
        let mut reader = trauma_events.get_reader(); // Bevy 0.14 compat (use get_cursor in 0.15+)
        let events: Vec<_> = reader.read(trauma_events).collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].target, pop);
        assert_eq!(events[0].cause, TraumaCause::WorldviewShattered);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct CargoCultist;

#[derive(Event)]
pub struct CommsRestoredEvent;

#[derive(Event)]
pub struct TraumaEvent {
    pub target: Entity,
    pub cause: TraumaCause,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TraumaCause {
    WorldviewShattered,
}

pub struct CargoCultPlugin;

impl Plugin for CargoCultPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CommsRestoredEvent>()
           .add_event::<TraumaEvent>()
           .add_systems(Update, (
               cargo_cult_formation_system,
               cargo_cult_disillusionment_system,
           ));
    }
}

fn cargo_cult_formation_system(
    mut commands: Commands,
    colonies: Query<(Entity, &ColonyCore)>,
    mut pops: Query<(Entity, &ColonyLocation, &StressTracker, &mut Job), Without<CargoCultist>>,
) {
    for (pop_entity, colony_loc, stress, mut job) in pops.iter_mut() {
        if let Ok((_, colony)) = colonies.get(colony_loc.0) {
            if !colony.comms_online && colony.comms_downtime_ticks > 1000 {
                if stress.current > 80.0 {
                    commands.entity(pop_entity).insert(CargoCultist);
                    *job = Job::EffigyBuilder;
                }
            }
        }
    }
}

fn cargo_cult_disillusionment_system(
    mut commands: Commands,
    mut comms_restored: EventReader<CommsRestoredEvent>,
    cultists: Query<Entity, With<CargoCultist>>,
    mut trauma_events: EventWriter<TraumaEvent>,
) {
    for _ in comms_restored.read() {
        for entity in cultists.iter() {
            commands.entity(entity).remove::<CargoCultist>();
            trauma_events.send(TraumaEvent {
                target: entity,
                cause: TraumaCause::WorldviewShattered,
            });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Magic Numbers:** Extract threshold values (1000 ticks, 80.0 stress) into a `CargoCultConfig` resource.
- **Resource Drain:** Effigy building should actively drain `ColonyResources`, converting useful metal into useless `Effigy` structures.
- **Event Updates:** Use the latest Bevy patterns (e.g. `EventReader` vs cursor handling).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes without issue.
- [ ] Test coverage ≥85% for the cargo cult logic.
- [ ] Cultists automatically transition their jobs and stop producing standard resources.
- [ ] Comms restoration triggers proper trauma integration.

## 7. Technical Guidance
- Ensure `CargoCultPlugin` is properly registered in the app schedule, likely in `Layer1SystemSet::Observation` or similar.
- Consider what happens if a cultist dies while building an effigy; ensure no resource leaks occur.

## 8. Questions
*Builder: add questions here if spec is unclear.*
