# 337 - Symbiotic Pests

## 1. Overview
The **Symbiotic Pests** feature introduces a dynamic where food storage and organic waste naturally generate "Vermin". Vermin are not strictly bad: while they eat food and can spread disease, certain species can be harvested for resources (meat, silk) or act as predators to keep other, more dangerous pests in check. This mechanic requires players to balance eradication, control, and symbiosis rather than treating all non-domesticated life as a pure negative.

## 2. Dependencies
- `008` Farm Building and Food Production
- `032` Entropy and Spoilage
- `164` Modular Fauna

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_vermin_spawn_near_unprotected_food() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, spawn_vermin_system);

        // Unprotected food pile
        app.world_mut().spawn((
            ResourceItem { item_type: ResourceType::Food },
            Stockpile { capacity: 100.0, current: 50.0, protected: false },
            Transform::from_xyz(10.0, 10.0, 0.0),
        ));

        // Act
        app.update(); // Initial tick may or may not spawn depending on probability
        // Advance time and force a spawn check
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(60));
        app.update();

        // Assert
        let vermin_count = app.world_mut().query::<&Vermin>().iter(app.world()).count();
        assert!(vermin_count > 0, "Vermin should spawn near unprotected food over time.");
    }

    #[test]
    fn test_predator_vermin_consume_prey_vermin() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, vermin_predation_system);

        let prey = app.world_mut().spawn((
            Vermin { species: VerminSpecies::GrainRat, health: 10.0 },
            Transform::from_xyz(5.0, 5.0, 0.0),
        )).id();

        let predator = app.world_mut().spawn((
            Vermin { species: VerminSpecies::BrainBorerBeetle, health: 20.0 },
            Predator { target_species: vec![VerminSpecies::GrainRat], hunt_radius: 5.0 },
            Transform::from_xyz(6.0, 6.0, 0.0),
        )).id();

        // Act
        app.update();

        // Assert
        // The prey should be dead (health <= 0 or despawned)
        assert!(app.world().get_entity(prey).is_none() || app.world().get::<Vermin>(prey).unwrap().health <= 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, Clone, PartialEq, Eq)]
pub enum VerminSpecies {
    GrainRat,
    BrainBorerBeetle,
    SilkWeaverSpider,
}

#[derive(Component)]
pub struct Vermin {
    pub species: VerminSpecies,
    pub health: f32,
}

#[derive(Component)]
pub struct Stockpile {
    pub capacity: f32,
    pub current: f32,
    pub protected: bool,
}

#[derive(Component)]
pub struct Predator {
    pub target_species: Vec<VerminSpecies>,
    pub hunt_radius: f32,
}

pub fn spawn_vermin_system(
    mut commands: Commands,
    time: Res<Time>,
    stockpiles: Query<(&Stockpile, &Transform)>,
) {
    // Simple 10% chance per second per unprotected stockpile with food
    if time.delta_seconds() > 0.0 {
        for (stockpile, transform) in stockpiles.iter() {
            if !stockpile.protected && stockpile.current > 0.0 && rand::random::<f32>() < 0.10 {
                commands.spawn((
                    Vermin { species: VerminSpecies::GrainRat, health: 10.0 },
                    *transform,
                ));
            }
        }
    }
}

pub fn vermin_predation_system(
    mut commands: Commands,
    predators: Query<(&Predator, &Transform)>,
    mut prey_query: Query<(Entity, &Vermin, &Transform)>,
) {
    for (predator, pred_transform) in predators.iter() {
        for (prey_entity, prey, prey_transform) in prey_query.iter_mut() {
            if predator.target_species.contains(&prey.species) {
                let distance = pred_transform.translation.distance(prey_transform.translation);
                if distance <= predator.hunt_radius {
                    commands.entity(prey_entity).despawn();
                    // In a full implementation, predator might gain health/reproduce
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**:
  - Instead of a flat probability for vermin spawning, scale the probability based on the amount of food (`stockpile.current`), ambient temperature, and cleanliness.
  - Make `VerminSpecies` an extensible resource or struct rather than an enum to allow easy modding or procedurally generated pests.
- **Code Smells**:
  - `spawn_vermin_system` uses `rand::random`. We should inject a deterministic RNG resource for testing consistency.
  - The predation system checks `N x M` entities. Use spatial hashing (e.g., the `TerrainGrid`) if the number of vermin gets large.
- **Performance**:
  - Iterating over all stockpiles and all vermin every tick could be expensive. Limit spawn checks to a timer (e.g., every 5 seconds).

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Vermin spawn around unprotected food.
- [ ] Predator vermin hunt and consume prey vermin within their radius.

## 7. Technical Guidance
- Integrate vermin behavior with the `Utility AI` if they are meant to move around intelligently (e.g., pathing towards food sources).
- Add an action for colonists to "Exterminate" vermin, allowing players to actively control populations.
- Provide a way to tag certain stockpiles as "Pest Safe" (e.g., refrigerated or sealed silos).

## 8. Questions
*Builder: add questions here if spec is unclear.*
