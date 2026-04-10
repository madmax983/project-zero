# 942: The Work-Shift Cartel

## Overview

Workers organizing to control labor supply and demand informally. Pops with high social influence in a specific sector (e.g., Mining) form a "Shift Cartel," demanding higher leisure time for their members while subtly sabotaging the productivity of non-members. It forces the player to choose between aggressively breaking the cartel or appeasing them at the cost of maximum efficiency.

## Dependencies

- None explicitly required, assumes existing Pop, Need, and Jobs systems.

## 1. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::{NeedTracker, Leisure};
    use crate::layer1::jobs::{Job, ProductionModifier};
    use crate::layer1::social::SocialInfluence;

    #[test]
    fn test_cartel_formation() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, evaluate_cartel_formation_system);

        let pop_id = app.world_mut().spawn((
            SocialInfluence { value: 0.9 }, // High influence
            Job::Miner,
        )).id();

        // Act
        app.update();

        // Assert
        // The highly influential pop forms a cartel in their sector
        assert!(app.world().get::<ShiftCartelMember>(pop_id).is_some());
        assert!(app.world().get_resource::<ActiveCartels>().unwrap().contains(&Job::Miner));
    }

    #[test]
    fn test_cartel_productivity_sabotage() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, cartel_sabotage_system);

        app.world_mut().insert_resource(ActiveCartels(vec![Job::Miner]));

        // Non-member in a cartel-controlled sector
        let non_member_id = app.world_mut().spawn((
            Job::Miner,
            ProductionModifier { multiplier: 1.0 },
        )).id();

        // Member in the cartel
        let member_id = app.world_mut().spawn((
            Job::Miner,
            ShiftCartelMember,
            ProductionModifier { multiplier: 1.0 },
        )).id();

        // Act
        app.update();

        // Assert
        // Non-member suffers a productivity penalty
        let non_member_modifier = app.world().get::<ProductionModifier>(non_member_id).unwrap();
        assert!(non_member_modifier.multiplier < 1.0);

        // Member remains unaffected
        let member_modifier = app.world().get::<ProductionModifier>(member_id).unwrap();
        assert_eq!(member_modifier.multiplier, 1.0);
    }

    #[test]
    fn test_cartel_leisure_demands() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, cartel_leisure_demand_system);

        let member_id = app.world_mut().spawn((
            ShiftCartelMember,
            NeedTracker::<Leisure>::new(50.0),
        )).id();

        // Act
        app.update();

        // Assert
        // The cartel member expects higher leisure time or their need degrades faster
        let leisure = app.world().get::<NeedTracker<Leisure>>(member_id).unwrap();
        assert!(leisure.decay_rate > 1.0); // Decay is accelerated due to cartel demands
    }
}
```

## 2. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Components and Resources
#[derive(Component)]
pub struct ShiftCartelMember;

#[derive(Resource, Default)]
pub struct ActiveCartels(pub Vec<Job>);

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Job {
    Miner,
    Farmer,
    // ...
}

#[derive(Component)]
pub struct SocialInfluence {
    pub value: f32,
}

#[derive(Component)]
pub struct ProductionModifier {
    pub multiplier: f32,
}

// Marker struct for NeedTracker
pub struct Leisure;

#[derive(Component)]
pub struct NeedTracker<T> {
    pub value: f32,
    pub decay_rate: f32,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> NeedTracker<T> {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            decay_rate: 1.0,
            _phantom: std::marker::PhantomData,
        }
    }
}

// Systems
pub fn evaluate_cartel_formation_system(
    mut commands: Commands,
    query: Query<(Entity, &SocialInfluence, &Job), Without<ShiftCartelMember>>,
    mut active_cartels: Local<Vec<Job>>, // Simplified to Local for now, or interact with Resource
) {
    for (entity, influence, job) in query.iter() {
        if influence.value > 0.8 {
            commands.entity(entity).insert(ShiftCartelMember);
            if !active_cartels.contains(job) {
                active_cartels.push(*job);
                // In full implementation, update the ActiveCartels resource
            }
        }
    }
}

pub fn cartel_sabotage_system(
    mut query: Query<(&Job, &mut ProductionModifier, Option<&ShiftCartelMember>)>,
    active_cartels: Option<Res<ActiveCartels>>,
) {
    if let Some(cartels) = active_cartels {
        for (job, mut modifier, member) in query.iter_mut() {
            if cartels.0.contains(job) && member.is_none() {
                modifier.multiplier = 0.5; // Sabotage effect
            }
        }
    }
}

pub fn cartel_leisure_demand_system(
    mut query: Query<&mut NeedTracker<Leisure>, With<ShiftCartelMember>>,
) {
    for mut leisure in query.iter_mut() {
        leisure.decay_rate = 2.0; // Higher leisure demand
    }
}
```

## 3. REFACTOR Phase: Quality & Design

- Ensure `ActiveCartels` integrates cleanly with global event structures (e.g., dispatching an event when a cartel forms).
- Extract magic numbers (`0.8` influence, `0.5` sabotage multiplier, `2.0` decay rate) into tunable balancing constants or a `CartelConfig` resource.
- Handle edge cases, such as Pops changing jobs (they should probably lose `ShiftCartelMember` or cause friction in their new job).

## 4. Acceptance Criteria (Testable)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.
- [ ] Highly influential Pops correctly spawn Cartels.
- [ ] Cartels negatively impact the production of non-members in the same sector.

## 5. Technical Guidance

- Place this in `src/layer1/social/cartel.rs` or similar.
- Use `Option<Res<T>>` in `cartel_sabotage_system` to ensure the system is safe to run even if the resource hasn't been properly initialized yet.
- Bevy's `With` and `Without` filters are highly efficient here for identifying cartel members vs non-members.

## 6. Questions

*Builder: add questions here if spec is unclear.*
