# Specification: The Subspace Contagion

## 1. Overview
The "Subspace Contagion" is a cross-layer mechanic where ancient data caches or derelict ships salvaged in Layer 2 contain a cybernetic virus. When salvaged, this virus jumps into the cybernetic implants of Layer 1 Pops. Infected Pops begin acting erratically (sabotaging grids, overriding airlocks, building bizarre structures) and can spread the contagion to other augmented Pops via proximity. The player must choose between EMPing the colony (destroying implants) or individually quarantining and rebooting the infected.

## 2. Dependencies
- **Layer 2 Salvage/Exploration Mechanics:** Must be able to encounter and salvage derelict ships.
- **Cybernetic Implants:** Pops must have a component or trait indicating cybernetic augmentation.
- **Job/Task System:** To allow infected Pops to exhibit erratic behavior or override regular jobs.
- **Pathfinding/Grid Proximity:** To calculate proximity for spreading the contagion.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::{Pop, CyberneticImplants};
    use crate::layer2::salvage::SalvageEvent;

    #[test]
    fn test_salvage_event_triggers_contagion() {
        // Arrange
        let mut app = App::new();
        app.add_event::<SalvageEvent>();
        app.add_systems(Update, handle_salvage_contagion_system);

        let pop_id = app.world_mut().spawn((Pop, CyberneticImplants)).id();

        // Act
        app.world_mut().send_event(SalvageEvent {
            contains_contagion: true,
            involved_pops: vec![pop_id],
        });
        app.update();

        // Assert
        assert!(app.world().entity(pop_id).has::<SubspaceContagion>());
    }

    #[test]
    fn test_contagion_spreads_via_proximity() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, spread_contagion_system);

        let infected_pop = app.world_mut().spawn((Pop, CyberneticImplants, SubspaceContagion, GridPosition { x: 5, y: 5 })).id();
        let uninfected_pop = app.world_mut().spawn((Pop, CyberneticImplants, GridPosition { x: 5, y: 6 })).id();

        // Act
        app.update();

        // Assert
        assert!(app.world().entity(uninfected_pop).has::<SubspaceContagion>());
    }

    #[test]
    fn test_emp_cures_contagion_but_destroys_implants() {
        // Arrange
        let mut app = App::new();
        app.add_event::<EmpDetonationEvent>();
        app.add_systems(Update, handle_emp_detonation_system);

        let infected_pop = app.world_mut().spawn((Pop, CyberneticImplants, SubspaceContagion)).id();

        // Act
        app.world_mut().send_event(EmpDetonationEvent);
        app.update();

        // Assert
        let entity = app.world().entity(infected_pop);
        assert!(!entity.has::<SubspaceContagion>());
        assert!(!entity.has::<CyberneticImplants>());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pop::{Pop, CyberneticImplants};
use crate::layer2::salvage::SalvageEvent;

#[derive(Component)]
pub struct SubspaceContagion;

#[derive(Event)]
pub struct EmpDetonationEvent;

pub fn handle_salvage_contagion_system(
    mut commands: Commands,
    mut events: EventReader<SalvageEvent>,
    query: Query<Entity, With<CyberneticImplants>>,
) {
    for event in events.read() {
        if event.contains_contagion {
            for pop_id in &event.involved_pops {
                if query.contains(*pop_id) {
                    commands.entity(*pop_id).insert(SubspaceContagion);
                }
            }
        }
    }
}

pub fn spread_contagion_system(
    mut commands: Commands,
    infected_query: Query<&GridPosition, (With<Pop>, With<SubspaceContagion>)>,
    uninfected_query: Query<(Entity, &GridPosition), (With<Pop>, With<CyberneticImplants>, Without<SubspaceContagion>)>,
) {
    for infected_pos in infected_query.iter() {
        for (entity, uninfected_pos) in uninfected_query.iter() {
            if infected_pos.distance_chebyshev(*uninfected_pos) <= 1 {
                commands.entity(entity).insert(SubspaceContagion);
            }
        }
    }
}

pub fn handle_emp_detonation_system(
    mut commands: Commands,
    mut events: EventReader<EmpDetonationEvent>,
    query: Query<Entity, With<CyberneticImplants>>,
) {
    for _ in events.read() {
        for entity in query.iter() {
            commands.entity(entity).remove::<CyberneticImplants>();
            commands.entity(entity).remove::<SubspaceContagion>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Spread Rate Limiting:** Introduce a probability for contagion spread or a timer to prevent instantaneous colony-wide infection.
- **Job Sabotage:** Implement logic to inject erratic jobs (e.g., `SabotageJob`, `BuildBizarreStructureJob`) into infected Pops' job queues, potentially intercepting their utility AI evaluations.
- **Quarantine Logic:** Build out the individual "quarantine and reboot" mechanics via the medical or security systems, giving the player an alternative to a full EMP.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Subspace Contagion transfers from salvage events to cybernetically augmented Pops.
- [ ] Subspace Contagion spreads via proximity to other augmented Pops.
- [ ] EMP event successfully removes both the contagion and the cybernetic implants.

## 7. Technical Guidance
- **System Placement:** Register observation and spread systems in `Layer1SystemSet::Observation` or a specific contagion set.
- **Event Handling:** Ensure `SalvageEvent` and `EmpDetonationEvent` are properly registered and cleared during setup/cleanup.
- **Performance Constraints:** Optimizing the `spread_contagion_system` is crucial. Instead of an O(N*M) iteration, consider a spatial hash or grid-based lookup to find uninfected neighbors near infected Pops.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
