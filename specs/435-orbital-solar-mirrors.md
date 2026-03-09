# Spec 435: Orbital Solar Mirrors

## 1. Overview
Massive mirror arrays in Layer 2 orbit reflect sunlight onto specific Layer 1 tiles, providing 24/7 solar power and heat. However, misaligning them or a hacking event can focus the beam too intensely, turning the mirror into a devastating orbital laser that scorches the colony.

## 2. Dependencies
- `042-energy-system`
- `065-day-night-cycle`
- `140-thermal-management`

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_orbital_mirror_provides_energy() {
        let mut app = App::new();
        app.insert_resource(EnergyGrid { available: 0.0 });
        app.add_systems(Update, process_solar_mirrors_system);

        let entity = app.world_mut().spawn((
            OrbitalSolarMirror { intensity: 100.0, hacked: false },
        )).id();

        app.update();

        let grid = app.world().resource::<EnergyGrid>();
        assert_eq!(grid.available, 100.0); // Safe intensity provides power
    }

    #[test]
    fn test_hacked_mirror_causes_fire() {
        let mut app = App::new();
        app.insert_resource(EnergyGrid { available: 0.0 });
        app.add_event::<SpawnFireEvent>();
        app.add_systems(Update, process_solar_mirrors_system);

        let entity = app.world_mut().spawn((
            OrbitalSolarMirror { intensity: 500.0, hacked: true },
            TargetTile { x: 10, y: 10 },
        )).id();

        app.update();

        let grid = app.world().resource::<EnergyGrid>();
        assert_eq!(grid.available, 0.0); // Hacked mirrors don't provide safe power

        let fire_events = app.world().resource::<Events<SpawnFireEvent>>();
        let mut reader = fire_events.get_reader();
        let events: Vec<_> = reader.read(fire_events).collect();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].x, 10);
        assert_eq!(events[0].y, 10);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct EnergyGrid {
    pub available: f32,
}

#[derive(Component)]
pub struct OrbitalSolarMirror {
    pub intensity: f32,
    pub hacked: bool,
}

#[derive(Component)]
pub struct TargetTile {
    pub x: i32,
    pub y: i32,
}

#[derive(Event)]
pub struct SpawnFireEvent {
    pub x: i32,
    pub y: i32,
}

pub fn process_solar_mirrors_system(
    mut grid: ResMut<EnergyGrid>,
    mut fire_events: EventWriter<SpawnFireEvent>,
    q_mirrors: Query<(&OrbitalSolarMirror, Option<&TargetTile>)>,
) {
    for (mirror, target) in q_mirrors.iter() {
        if !mirror.hacked {
            grid.available += mirror.intensity;
        } else {
            if let Some(target) = target {
                if mirror.intensity > 200.0 {
                    fire_events.send(SpawnFireEvent {
                        x: target.x,
                        y: target.y,
                    });
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create an `Orbiting` component for Layer 2 entities that link down to the specific tile coords they target.
- Include a thermal transfer system where unhacked mirrors increase ambient heat on their targeted tiles over time, useful for ice planets.
- Randomize hacking events based on colony Unrest or External Threats.

## 6. Acceptance Criteria
- [ ] Safe `OrbitalSolarMirror`s provide energy continuously regardless of the Layer 1 day/night cycle.
- [ ] Hacked `OrbitalSolarMirror`s provide no energy and spawn fires at their `TargetTile`.
- [ ] Tests pass with >= 85% coverage.

## 7. Technical Guidance
- Hook the `SpawnFireEvent` into the existing Fire Propagation system (033).
- Ensure mirrors correctly apply heat using the Thermal Management system (140) if they are focused but safe.

## 8. Questions
- How easily can a hacked mirror be shut down or realigned? Does it require a spacewalk?
- *Architect:* Hacked mirrors can be disabled from the Command Center if the player has sufficient `Admin` resources, or by manually launching a spacewalk repair mission which costs time and fuel.
