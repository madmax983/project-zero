# 1150 - The Doomsday Clock

## 1. Overview
**Layer:** 1
The "Doomsday Clock" is a psychological mechanic where a false rumor or delusion spreads through the colony, convincing Pops that an apocalyptic event is imminent. During the countdown to this non-existent event, panic increases productivity as Pops desperately prepare or panic-work. However, once the clock reaches zero and the event fails to occur, the resulting cognitive dissonance and exhaustion trigger a severe crash in mood, productivity, and an increase in nihilistic traits.

## 2. Dependencies
- Base Pop components (`Pop`, `NeedState`, `Job`, `Trait`)
- Time/Simulation components (`SimulationTime`)
- Event/Rumor infrastructure (to start the countdown)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use bevy::ecs::system::RunSystemOnce;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // ... add necessary simulation plugins
        app
    }

    #[test]
    fn test_doomsday_rumor_increases_productivity() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            Job { productivity_modifier: 1.0, ..default() },
        )).id();

        // Act: Apply a Doomsday Clock rumor counting down to time 100
        app.world_mut().entity_mut(pop).insert(DoomsdayBeliever {
            apocalypse_time: 100.0,
            original_productivity: 1.0,
        });

        // Advance simulation time (e.g., current time = 50.0)
        app.insert_resource(SimulationTime { current: 50.0 });
        app.world_mut().run_system_once(apply_doomsday_panic_effects).unwrap();

        // Assert: Productivity should be spiked during the countdown
        let job = app.world().entity(pop).get::<Job>().unwrap();
        assert!(job.productivity_modifier > 1.0, "Productivity should increase during panic");
    }

    #[test]
    fn test_doomsday_clock_hits_zero_causes_crash() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            Job { productivity_modifier: 1.5, ..default() }, // Boosted from panic
            DoomsdayBeliever {
                apocalypse_time: 100.0,
                original_productivity: 1.0,
            },
        )).id();

        // Act: Advance time past the apocalypse
        app.insert_resource(SimulationTime { current: 101.0 });
        app.world_mut().run_system_once(resolve_doomsday_event).unwrap();

        // Assert: The belief component is removed, and a debuff is applied
        assert!(!app.world().entity(pop).contains::<DoomsdayBeliever>());
        assert!(app.world().entity(pop).contains::<NihilismDebuff>());

        let job = app.world().entity(pop).get::<Job>().unwrap();
        assert!(job.productivity_modifier < 1.0, "Productivity should crash after false apocalypse");
    }

    #[test]
    fn test_doomsday_crash_duration() {
         let mut app = setup_app();

         let pop = app.world_mut().spawn((
             Pop,
             NihilismDebuff {
                 expires_at: 120.0,
             },
             Job { productivity_modifier: 0.5, ..default() }
         )).id();

         // Act: Time passes the expiration
         app.insert_resource(SimulationTime { current: 121.0 });
         app.world_mut().run_system_once(cleanup_nihilism_debuffs).unwrap();

         // Assert: Debuff removed, stats should eventually normalize
         assert!(!app.world().entity(pop).contains::<NihilismDebuff>());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct DoomsdayBeliever {
    pub apocalypse_time: f32,
    pub original_productivity: f32,
}

#[derive(Component)]
pub struct NihilismDebuff {
    pub expires_at: f32,
}

#[derive(Component)]
pub struct Job {
    pub productivity_modifier: f32,
}

#[derive(Resource)]
pub struct SimulationTime {
    pub current: f32,
}

pub fn apply_doomsday_panic_effects(
    time: Res<SimulationTime>,
    mut query: Query<(&DoomsdayBeliever, &mut Job)>
) {
    for (believer, mut job) in query.iter_mut() {
        if time.current < believer.apocalypse_time {
            // Panic productivity boost
            job.productivity_modifier = believer.original_productivity * 1.5;
        }
    }
}

pub fn resolve_doomsday_event(
    mut commands: Commands,
    time: Res<SimulationTime>,
    mut query: Query<(Entity, &DoomsdayBeliever, &mut Job)>
) {
    for (entity, believer, mut job) in query.iter_mut() {
        if time.current >= believer.apocalypse_time {
            // Event didn't happen! Crash productivity and apply debuff
            job.productivity_modifier = believer.original_productivity * 0.5;
            commands.entity(entity).remove::<DoomsdayBeliever>();
            commands.entity(entity).insert(NihilismDebuff {
                expires_at: time.current + 50.0, // Arbitrary duration
            });
        }
    }
}

pub fn cleanup_nihilism_debuffs(
    mut commands: Commands,
    time: Res<SimulationTime>,
    query: Query<(Entity, &NihilismDebuff)>
) {
     for (entity, debuff) in query.iter() {
         if time.current >= debuff.expires_at {
             commands.entity(entity).remove::<NihilismDebuff>();
             // Note: Needs logic to restore original productivity after debuff expires.
         }
     }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** Ensure `original_productivity` is robustly captured and restored. Directly mutating `Job.productivity_modifier` may conflict with other buffs/debuffs. Consider standardizing an `Effect` component or using Bevy's `Added`/`Removed` query filters to recalculate stat modifiers dynamically instead of hardcoding multipliers.
- **Code Smell:** Hardcoded multipliers (1.5x, 0.5x) and durations (50.0). These should be configurable via resources or component fields.
- **Performance:** `apply_doomsday_panic_effects` recalculates productivity every frame. It should ideally only trigger when the rumor is first applied or use standard ECS change detection (`Changed<DoomsdayBeliever>`).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops exhibit increased productivity during the countdown.
- [ ] Pops suffer a productivity crash and gain a negative mood/trait after the countdown ends.

## 7. Technical Guidance
- **Gotcha:** Do not blindly multiply `productivity_modifier` on every frame without an `Added` filter, or it will overflow. The minimal implementation sets it directly, which is safe, but recalculating from a base value is safer when combining multiple modifiers.
- **Integration:** The `DoomsdayBeliever` component needs to be seeded by the event/rumor system. Consider adding a small system that randomly (or based on certain low-mood conditions) spawns a "Prophet" pop that starts infecting neighbors with `DoomsdayBeliever`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
