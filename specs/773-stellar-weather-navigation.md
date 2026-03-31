# 773: Stellar Weather Navigation

## 1. Overview
**Layer:** 2
**Fantasy:** Sailing the dangerous, unpredictable oceans of space where the "wind" is solar radiation and the "storms" are coronal mass ejections.
**Mechanic:** Fleet movement between nodes is altered by stellar weather. Ships equipped with "Solar Sails" can ride solar winds for massive speed boosts, but risk being blown off course or damaged by sudden solar flares.

## 2. Dependencies
- Core Layer 2 Fleet movement
- Map/Node graph system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_solar_sails_boost_speed_in_favorable_wind() {
        let mut app = App::new();
        app.add_systems(Update, apply_stellar_weather_effects);

        let fleet_entity = app.world_mut().spawn((
            Fleet { base_speed: 10.0, current_speed: 10.0 },
            SolarSails { deployed: true, efficiency: 2.0 },
            CurrentSector { sector_id: 1 },
        )).id();

        app.world_mut().spawn((
            Sector { id: 1 },
            StellarWeather { wind_strength: 5.0, flare_active: false },
        ));

        app.update();

        let fleet = app.world().get::<Fleet>(fleet_entity).unwrap();
        assert!(fleet.current_speed > fleet.base_speed, "Speed should be boosted by favorable wind");
        assert_eq!(fleet.current_speed, 10.0 + (5.0 * 2.0));
    }

    #[test]
    fn test_solar_flare_damages_deployed_sails() {
        let mut app = App::new();
        app.add_event::<FleetDamagedEvent>();
        app.add_systems(Update, apply_stellar_weather_effects);

        let fleet_entity = app.world_mut().spawn((
            Fleet { base_speed: 10.0, current_speed: 10.0 },
            SolarSails { deployed: true, efficiency: 2.0 },
            CurrentSector { sector_id: 2 },
        )).id();

        app.world_mut().spawn((
            Sector { id: 2 },
            StellarWeather { wind_strength: 5.0, flare_active: true },
        ));

        app.update();

        let damage_events = app.world().resource::<Events<FleetDamagedEvent>>();
        let mut reader = damage_events.get_cursor();
        assert!(reader.read(damage_events).any(|e| e.fleet == fleet_entity), "Fleet should be damaged by flare when sails are deployed");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Fleet {
    pub base_speed: f32,
    pub current_speed: f32,
}

#[derive(Component)]
pub struct SolarSails {
    pub deployed: bool,
    pub efficiency: f32,
}

#[derive(Component)]
pub struct CurrentSector {
    pub sector_id: u32,
}

#[derive(Component)]
pub struct Sector {
    pub id: u32,
}

#[derive(Component)]
pub struct StellarWeather {
    pub wind_strength: f32,
    pub flare_active: bool,
}

#[derive(Event)]
pub struct FleetDamagedEvent {
    pub fleet: Entity,
    pub amount: f32,
}

pub fn apply_stellar_weather_effects(
    mut fleets: Query<(Entity, &mut Fleet, &SolarSails, &CurrentSector)>,
    sectors: Query<(&Sector, &StellarWeather)>,
    mut damage_events: EventWriter<FleetDamagedEvent>,
) {
    for (fleet_entity, mut fleet, sails, current_sector) in fleets.iter_mut() {
        if let Some((_, weather)) = sectors.iter().find(|(s, _)| s.id == current_sector.sector_id) {
            if sails.deployed {
                fleet.current_speed = fleet.base_speed + (weather.wind_strength * sails.efficiency);
                if weather.flare_active {
                    damage_events.send(FleetDamagedEvent {
                        fleet: fleet_entity,
                        amount: 50.0,
                    });
                }
            } else {
                fleet.current_speed = fleet.base_speed;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**: Finding the sector by iterating all sectors is `O(N)`. We should either use a HashMap resource mapping sector IDs to entities, or have the `CurrentSector` store the actual sector `Entity` instead of an ID.
- **Performance**: Transition to Entity relationships (using a component like `Parent` or a custom relation) instead of ID lookups for sector locations.
- **API Improvements**: Differentiate between hull damage and sail damage.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified: Speed increases with wind + sails, flares damage fleets with deployed sails.

## 7. Technical Guidance
- Consider how weather is generated or moves between sectors.
- Use `Events` for damage to maintain decoupling.

## 8. Questions
*Builder: add questions here if spec is unclear.*
