# Living Architecture

## 1. Overview
**Layer:** 1
**Fantasy:** Growing your colony rather than building it, relying on biological structures that bleed, heal, and occasionally get hungry.
**Mechanic:** Some advanced buildings can be "planted" using modified bio-mass instead of constructed with metal. They slowly regenerate structural damage, produce minor resources passively, but require a constant supply of organic food or water to survive. A starving building might eventually "eat" a pop that enters it to sustain itself.

## 2. Dependencies
- `004-basic-building.md`
- `034-pop-health.md`

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_living_building_heals_over_time() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, living_building_healing_system);

        let building = app.world.spawn((
            LivingBuilding {
                hunger: 0.0,
                heal_rate: 5.0,
            },
            Health {
                current: 50.0,
                max: 100.0,
            },
        )).id();

        // Act
        app.update();

        // Assert
        let health = app.world.get::<Health>(building).unwrap();
        assert_eq!(health.current, 55.0);
    }

    #[test]
    fn test_living_building_starves_and_stops_healing() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, living_building_healing_system);

        let building = app.world.spawn((
            LivingBuilding {
                hunger: 100.0, // Starving
                heal_rate: 5.0,
            },
            Health {
                current: 50.0,
                max: 100.0,
            },
        )).id();

        // Act
        app.update();

        // Assert
        let health = app.world.get::<Health>(building).unwrap();
        assert_eq!(health.current, 50.0); // No healing
    }

    #[test]
    fn test_living_building_eats_pop_when_starving() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, living_building_consume_pop_system);

        let pop = app.world.spawn((
            Pop,
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let building = app.world.spawn((
            LivingBuilding {
                hunger: 100.0, // Starving
                heal_rate: 5.0,
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        // Act
        app.update();

        // Assert
        assert!(app.world.get_entity(pop).is_none()); // Pop was consumed
        let living_building = app.world.get::<LivingBuilding>(building).unwrap();
        assert!(living_building.hunger < 100.0); // Hunger decreased
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct LivingBuilding {
    pub hunger: f32,
    pub heal_rate: f32,
}

pub fn living_building_healing_system(
    mut query: Query<(&LivingBuilding, &mut Health)>,
) {
    for (living, mut health) in query.iter_mut() {
        if living.hunger < 80.0 { // Arbitrary threshold for not starving
            health.current = (health.current + living.heal_rate).min(health.max);
        }
    }
}

pub fn living_building_consume_pop_system(
    mut commands: Commands,
    mut buildings: Query<(&Transform, &mut LivingBuilding)>,
    pops: Query<(Entity, &Transform), With<Pop>>,
) {
    for (building_transform, mut living_building) in buildings.iter_mut() {
        if living_building.hunger >= 80.0 {
            for (pop_entity, pop_transform) in pops.iter() {
                if building_transform.translation.distance(pop_transform.translation) < 2.0 {
                    // Consume pop
                    commands.entity(pop_entity).despawn();
                    living_building.hunger = 0.0;
                    break; // Only eat one per tick
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate with the existing `Health` component used by other entities.
- Add an event system (`PopConsumedEvent`) so the chronicle and UI can notify the player when a building eats a colonist.
- Hunger logic should be part of a broader need/metabolism system, utilizing existing resources (e.g., `ColonyResources.food` or `water`).
- The pop-consumption range should map to the building's actual footprint or entrance.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Living buildings heal when well-fed, and consume nearby pops when starving.

## 7. Technical Guidance
- `living_building_consume_pop_system` should probably fire an event instead of silently despawning, so other systems (like Faction Morale) can react.
- Ensure the building's food consumption is tied to the central logistics/hauling network so workers can feed it.

## 8. Questions
*Builder: add questions here if spec is unclear.*
