# Specification: 858 Prototyping Phase

## 1. Overview
**Layer:** 1
**Fantasy:** The first time you build a fusion reactor, it shouldn't work perfectly.
**Mechanic:** The first time a complex building is constructed, it is spawned as a "Prototype." Prototypes have lower operational stats and a higher breakdown chance. After a certain amount of runtime, the design is "Mastered," and future builds of that type become normal, reliable buildings.

## 2. Dependencies
- Base Layer 1 Building System (`src/layer1/buildings.rs`)
- Time/Runtime Tracking System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[derive(Component, Default)]
    struct AdvancedReactor;

    #[test]
    fn test_first_building_is_prototype() {
        let mut app = App::new();
        app.add_plugins(PrototypingPlugin);

        app.world_mut().insert_resource(BlueprintMastery::default());

        // Spawn first reactor
        let reactor = app.world_mut().spawn((Building, AdvancedReactor)).id();

        // Send construction complete event
        app.world_mut().send_event(BuildingConstructedEvent {
            entity: reactor,
            building_type: "AdvancedReactor".to_string(),
        });
        app.update();

        // Assert it was marked as a prototype
        assert!(app.world().get::<Prototype>(reactor).is_some());

        // Prototype stats are worse
        let stats = app.world().get::<BuildingStats>(reactor).unwrap();
        assert!(stats.efficiency < 1.0);
        assert!(stats.breakdown_chance > 0.1);
    }

    #[test]
    fn test_prototype_achieves_mastery_over_time() {
        let mut app = App::new();
        app.add_plugins(PrototypingPlugin);

        app.world_mut().insert_resource(BlueprintMastery::default());

        let reactor = app.world_mut().spawn((
            Building,
            AdvancedReactor,
            Prototype { runtime: 0.0, required_runtime: 100.0 }
        )).id();

        // Advance time by 100 seconds
        app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs(101));
        app.update();

        // Assert Mastery was achieved
        let mastery = app.world().resource::<BlueprintMastery>();
        assert!(mastery.is_mastered("AdvancedReactor"));

        // Assert prototype component is removed from the building
        assert!(app.world().get::<Prototype>(reactor).is_none());
    }

    #[test]
    fn test_subsequent_buildings_are_not_prototypes() {
        let mut app = App::new();
        app.add_plugins(PrototypingPlugin);

        let mut mastery = BlueprintMastery::default();
        mastery.mark_mastered("AdvancedReactor");
        app.world_mut().insert_resource(mastery);

        let reactor2 = app.world_mut().spawn((Building, AdvancedReactor)).id();

        app.world_mut().send_event(BuildingConstructedEvent {
            entity: reactor2,
            building_type: "AdvancedReactor".to_string(),
        });
        app.update();

        // Assert it is NOT a prototype
        assert!(app.world().get::<Prototype>(reactor2).is_none());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct Prototype {
    pub runtime: f32,
    pub required_runtime: f32,
}

#[derive(Component)]
pub struct BuildingStats {
    pub efficiency: f32,
    pub breakdown_chance: f32,
}

#[derive(Event)]
pub struct BuildingConstructedEvent {
    pub entity: Entity,
    pub building_type: String,
}

#[derive(Resource, Default)]
pub struct BlueprintMastery {
    mastered_types: HashSet<String>,
}

impl BlueprintMastery {
    pub fn is_mastered(&self, b_type: &str) -> bool {
        self.mastered_types.contains(b_type)
    }

    pub fn mark_mastered(&mut self, b_type: &str) {
        self.mastered_types.insert(b_type.to_string());
    }
}

pub struct PrototypingPlugin;

impl Plugin for PrototypingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BuildingConstructedEvent>()
           .init_resource::<BlueprintMastery>()
           .add_systems(Update, (
               apply_prototype_status_system,
               progress_prototype_runtime_system,
           ));
    }
}

fn apply_prototype_status_system(
    mut commands: Commands,
    mut events: EventReader<BuildingConstructedEvent>,
    mastery: Res<BlueprintMastery>,
) {
    for event in events.read() {
        if !mastery.is_mastered(&event.building_type) {
            commands.entity(event.entity).insert((
                Prototype { runtime: 0.0, required_runtime: 100.0 },
                BuildingStats { efficiency: 0.5, breakdown_chance: 0.5 },
            ));
            // In a real implementation we would also store the `building_type` on the entity
            commands.entity(event.entity).insert(BuildingTypeData(event.building_type.clone()));
        } else {
            commands.entity(event.entity).insert(
                BuildingStats { efficiency: 1.0, breakdown_chance: 0.01 }
            );
        }
    }
}

#[derive(Component)]
pub struct BuildingTypeData(pub String);

fn progress_prototype_runtime_system(
    mut commands: Commands,
    mut prototypes: Query<(Entity, &mut Prototype, &BuildingTypeData)>,
    mut mastery: ResMut<BlueprintMastery>,
    time: Res<Time>,
) {
    for (entity, mut proto, b_type) in prototypes.iter_mut() {
        proto.runtime += time.delta_seconds();
        if proto.runtime >= proto.required_runtime {
            // Mastered!
            mastery.mark_mastered(&b_type.0);

            // Remove prototype handicap
            commands.entity(entity)
                .remove::<Prototype>()
                .insert(BuildingStats { efficiency: 1.0, breakdown_chance: 0.01 });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Use Enums or strongly typed IDs for `building_type` rather than bare Strings to prevent typo bugs.
- Instead of outright overriding `BuildingStats`, implement a modifier system so base stats from other components aren't completely clobbered.
- Only increment `runtime` if the building is actually active and powered, not just existing.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the prototyping module.
- [ ] First building of a complex type spawns with `Prototype` component.
- [ ] Prototypes have reduced efficiency and higher breakdown chance.
- [ ] Active runtime successfully converts a Prototype to Mastered.

## 7. Technical Guidance
- Integrate with the building logic. We only want complex buildings (Reactors, Shields, Advanced Factories) to go through prototyping, not simple ones like basic storage or solar panels. You might need a `RequiresPrototyping` marker on the blueprint data.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
