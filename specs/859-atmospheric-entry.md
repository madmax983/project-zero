# Specification: 859 Atmospheric Entry

## 1. Overview
**Layer:** 2 -> 1
**Fantasy:** The 7 minutes of terror. Landing on a planet isn't a simple loading screen.
**Mechanic:** Dropships transitioning from Layer 2 space to Layer 1 planetary surface take heat and structural damage based on entry angle, speed, and atmospheric density. Safe, slow descents are vulnerable to anti-air, while hot drops risk crashing and scattering cargo/mechs across the map.

## 2. Dependencies
- Layer 2 to Layer 1 Transition System (`src/layer2/planetary_landing.rs` or similar)
- Atmospheric Density data on planets

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_slow_entry_takes_no_heat_damage() {
        let mut app = App::new();
        app.add_plugins(AtmosphericEntryPlugin);

        let ship = app.world_mut().spawn((
            Dropship,
            Hull(100.0),
        )).id();

        app.world_mut().send_event(AtmosphericEntryEvent {
            ship_entity: ship,
            entry_speed: 10.0, // Slow
            entry_angle: 45.0, // Optimal
            atmosphere_density: 1.0,
        });
        app.update();

        // Hull should be intact
        let hull = app.world().get::<Hull>(ship).unwrap();
        assert_eq!(hull.0, 100.0, "Slow entry should not damage hull");
    }

    #[test]
    fn test_hot_drop_takes_heat_damage() {
        let mut app = App::new();
        app.add_plugins(AtmosphericEntryPlugin);

        let ship = app.world_mut().spawn((
            Dropship,
            Hull(100.0),
        )).id();

        app.world_mut().send_event(AtmosphericEntryEvent {
            ship_entity: ship,
            entry_speed: 100.0, // Fast hot drop
            entry_angle: 90.0, // Steep angle
            atmosphere_density: 1.5, // Thick atmosphere
        });
        app.update();

        // Hull should take damage based on speed and density
        let hull = app.world().get::<Hull>(ship).unwrap();
        assert!(hull.0 < 100.0, "Hot drop should damage hull");
    }

    #[test]
    fn test_fatal_entry_scatters_cargo() {
        let mut app = App::new();
        app.add_plugins(AtmosphericEntryPlugin);

        let ship = app.world_mut().spawn((
            Dropship,
            Hull(10.0), // Weak hull
            Cargo(vec!["Mech_A".to_string(), "Mech_B".to_string()]),
        )).id();

        app.world_mut().send_event(AtmosphericEntryEvent {
            ship_entity: ship,
            entry_speed: 200.0, // Extremely fast
            entry_angle: 90.0,
            atmosphere_density: 2.0,
        });
        app.update();

        // Ship should be destroyed
        assert!(app.world().get_entity(ship).is_err());

        // Cargo scattering event should be emitted
        let scatter_events = app.world().resource::<Events<CargoScatteredEvent>>();
        let mut reader = scatter_events.get_cursor();
        let events: Vec<_> = reader.read(scatter_events).collect();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].items.len(), 2);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Dropship;

#[derive(Component)]
pub struct Hull(pub f32);

#[derive(Component)]
pub struct Cargo(pub Vec<String>);

#[derive(Event)]
pub struct AtmosphericEntryEvent {
    pub ship_entity: Entity,
    pub entry_speed: f32,
    pub entry_angle: f32,
    pub atmosphere_density: f32,
}

#[derive(Event)]
pub struct CargoScatteredEvent {
    pub items: Vec<String>,
}

pub struct AtmosphericEntryPlugin;

impl Plugin for AtmosphericEntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AtmosphericEntryEvent>()
           .add_event::<CargoScatteredEvent>()
           .add_systems(Update, handle_atmospheric_entry_system);
    }
}

fn handle_atmospheric_entry_system(
    mut commands: Commands,
    mut events: EventReader<AtmosphericEntryEvent>,
    mut ships: Query<(&mut Hull, Option<&Cargo>)>,
    mut scatter_events: EventWriter<CargoScatteredEvent>,
) {
    for event in events.read() {
        if let Ok((mut hull, cargo)) = ships.get_mut(event.ship_entity) {

            // Basic heat calculation: speed * density * angle_penalty
            // angle 45 is optimal (penalty 0), 90 is steep (penalty 1)
            let angle_penalty = (event.entry_angle - 45.0).abs() / 45.0;
            let speed_threshold = 20.0;

            if event.entry_speed > speed_threshold {
                let heat_damage = (event.entry_speed - speed_threshold) * event.atmosphere_density * (0.5 + angle_penalty * 0.5);
                hull.0 -= heat_damage;

                if hull.0 <= 0.0 {
                    if let Some(c) = cargo {
                        scatter_events.send(CargoScatteredEvent { items: c.0.clone() });
                    }
                    commands.entity(event.ship_entity).despawn();
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Improve the heat formula to be non-linear (e.g. speed squared) to better represent kinetic energy converted to heat.
- Instead of instantly despawning, spawn a "Crashing" component that creates a visual fireball and resolves the actual crash impact (with area damage) on the Layer 1 surface on a subsequent tick.
- Hook into the anti-air targetting systems to apply the vulnerability to slow ships (this is conceptualized in the overview, but should be integrated).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the atmospheric entry module.
- [ ] High speed drops into dense atmospheres deal damage to the hull.
- [ ] If hull drops below zero, the ship is destroyed and a `CargoScatteredEvent` is fired.

## 7. Technical Guidance
- The `CargoScatteredEvent` should be picked up by a Layer 1 system to actually place the mechs or cargo pods at randomized coordinates on the map surface.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
