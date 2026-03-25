# 583: The Cryo-Plague Carrier

## 1. Overview
A Pop wakes up from an ancient cryo-pod carrying a benign (to them) precursor virus. To the modern colony, it is a devastating plague. However, this "Carrier" is the only Pop possessing the unique "Precursor Engineering" skill required to repair failing ancient terraforming machinery keeping the colony's atmosphere breathable.

## 2. Dependencies
- Layer 1: Disease & Health System
- Layer 1: Pop Skills & Professions
- Layer 1: Environmental Systems (Atmospherics)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_carrier_spreads_plague_to_nearby_pops() {
        let mut app = App::new();
        app.add_systems(Update, spread_cryo_plague_system);

        let carrier = app.world_mut().spawn((Position { x: 5, y: 5 }, CryoPlagueCarrier)).id();
        let victim = app.world_mut().spawn((Position { x: 6, y: 5 }, Pop)).id();

        app.update();

        // Victim should now have the plague
        assert!(app.world().get::<CryoPlagueInfection>(victim).is_some());
    }

    #[test]
    fn test_carrier_repairs_precursor_machinery() {
        let mut app = App::new();
        app.add_systems(Update, repair_precursor_machinery_system);

        let machine = app.world_mut().spawn(PrecursorAtmosphereScrubber { health: 50.0 }).id();

        // Only a pop with PrecursorEngineering can repair
        let carrier = app.world_mut().spawn((
            CryoPlagueCarrier,
            PrecursorEngineeringSkill,
            JobAssignment { target: machine }
        )).id();

        app.update();

        let scrubber = app.world().get::<PrecursorAtmosphereScrubber>(machine).unwrap();
        assert!(scrubber.health > 50.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct CryoPlagueCarrier;

#[derive(Component)]
pub struct CryoPlagueInfection;

#[derive(Component)]
pub struct PrecursorEngineeringSkill;

#[derive(Component)]
pub struct PrecursorAtmosphereScrubber {
    pub health: f32,
}

#[derive(Component)]
pub struct JobAssignment {
    pub target: Entity,
}

pub fn spread_cryo_plague_system(
    mut commands: Commands,
    carrier_query: Query<&Position, With<CryoPlagueCarrier>>,
    pop_query: Query<(Entity, &Position), (With<Pop>, Without<CryoPlagueInfection>, Without<CryoPlagueCarrier>)>,
) {
    for carrier_pos in carrier_query.iter() {
        for (pop_entity, pop_pos) in pop_query.iter() {
            // Simple Manhattan distance check
            if (carrier_pos.x - pop_pos.x).abs() + (carrier_pos.y - pop_pos.y).abs() <= 2 {
                commands.entity(pop_entity).insert(CryoPlagueInfection);
            }
        }
    }
}

pub fn repair_precursor_machinery_system(
    worker_query: Query<&JobAssignment, With<PrecursorEngineeringSkill>>,
    mut machine_query: Query<&mut PrecursorAtmosphereScrubber>,
) {
    for job in worker_query.iter() {
        if let Ok(mut machine) = machine_query.get_mut(job.target) {
            machine.health += 10.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spatial Queries:** Use a spatial hash grid instead of an O(N^2) loop for disease spreading.
- **Infection Severity:** Add stages to `CryoPlagueInfection` (incubation, symptomatic, terminal).
- **Quarantine Logic:** Ensure Carrier routing respects isolation zones if the player tries to minimize contact.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified (Carrier spreads disease, Carrier can repair specific machinery)

## 7. Technical Guidance
- The tension relies on the `PrecursorAtmosphereScrubber` degrading over time. Ensure an existing system constantly ticks its health down.
- Consider an event when the Carrier dies, triggering a "Atmosphere Failure Imminent" alert.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
