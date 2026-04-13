# 1000: The Eclipsing Swarm

## 1. Overview
The sheer terror of watching the sun disappear behind a living wall. Enormous swarms of migratory, void-dwelling locusts travel through Layer 2. When they pass between your planet and its sun, they create a "Swarm Eclipse." Solar power drops to zero, temperatures plummet, and the swarm sheds highly corrosive bio-waste onto the planet's surface before moving on.

## 2. Dependencies
- Layer 1 Power/Solar grid mechanics.
- Layer 2 Void Fauna/Swarm entities.
- Layer 1 Climate/Temperature mechanics.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_swarm_eclipse_power_drop() {
        let mut app = App::new();
        app.add_systems(Update, swarm_eclipse_system);

        let planet = app.world_mut().spawn((
            SolarPowerGrid { current_output: 100.0, max_output: 100.0 },
            Temperature { current: 20.0 },
        )).id();

        app.world_mut().spawn((
            VoidSwarm { size: 1000 },
            EclipseTarget(planet),
        ));

        app.update();

        let grid = app.world().get::<SolarPowerGrid>(planet).unwrap();
        assert_eq!(grid.current_output, 0.0);
        let temp = app.world().get::<Temperature>(planet).unwrap();
        assert!(temp.current < 20.0);
    }

    #[test]
    fn test_swarm_corrosive_rain() {
        let mut app = App::new();
        app.add_systems(Update, swarm_bio_waste_system);

        let planet = app.world_mut().spawn((
            ColonyInfrastructure { integrity: 100.0 },
        )).id();

        app.world_mut().spawn((
            VoidSwarm { size: 1000 },
            EclipseTarget(planet),
        ));

        app.update();

        let infra = app.world().get::<ColonyInfrastructure>(planet).unwrap();
        assert!(infra.integrity < 100.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct SolarPowerGrid {
    pub current_output: f32,
    pub max_output: f32,
}

#[derive(Component)]
pub struct Temperature {
    pub current: f32,
}

#[derive(Component)]
pub struct ColonyInfrastructure {
    pub integrity: f32,
}

#[derive(Component)]
pub struct VoidSwarm {
    pub size: u32,
}

#[derive(Component)]
pub struct EclipseTarget(pub Entity);

pub fn swarm_eclipse_system(
    swarm_query: Query<&EclipseTarget, With<VoidSwarm>>,
    mut planet_query: Query<(&mut SolarPowerGrid, &mut Temperature)>,
) {
    for target in swarm_query.iter() {
        if let Ok((mut grid, mut temp)) = planet_query.get_mut(target.0) {
            grid.current_output = 0.0;
            temp.current -= 5.0; // Minimal implementation of temp drop
        }
    }
}

pub fn swarm_bio_waste_system(
    swarm_query: Query<&EclipseTarget, With<VoidSwarm>>,
    mut planet_query: Query<&mut ColonyInfrastructure>,
) {
    for target in swarm_query.iter() {
        if let Ok(mut infra) = planet_query.get_mut(target.0) {
            infra.integrity -= 10.0; // Minimal implementation of corrosion
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Hardcoded temp and integrity drops should scale with `VoidSwarm.size`.
- **Performance**: We iterate over swarms and then perform a random access `get_mut` on planets. This is fine unless there are thousands of swarms eclipsing the same planet, in which case we might want to aggregate effects first.
- **Design Improvements**: The eclipse state should probably be a buff/debuff component applied to the planet rather than directly modifying the power output and temp tick-by-tick, ensuring it cleanly reverts when the swarm leaves.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Swarms over a planet drop solar output to 0.
- [ ] Swarms over a planet decrease temperature and damage infrastructure.

## 7. Technical Guidance
- The `VoidSwarm` movement should be handled in `layer2::weather` or a new `layer2::fauna` module.
- Integration between L2 spatial positions and L1 planetary effects will likely need a system in `layer1::core::integration`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
