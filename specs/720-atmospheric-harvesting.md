# Atmospheric Harvesting (Spec 720)

## 1. Overview
**Layer:** 2 (System)
**Fantasy:** Skimming the clouds of giants.
**Mechanic:** Gas Giants have "Scoop" zones in orbit. Ships can fly through to collect Fuel/Rare Gases but take hull damage from turbulence/storms.
**Emergence:** You get greedy for that last tank of Helium-3 and fly too deep. The storm crushes the ship.
**Tension:** Deep dive (rich resources/high damage) vs. Shallow skim (low resources/safe).

## 2. Dependencies
- `layer2::ship::Ship` and `Fleet` mechanics.
- `layer2::system::OrbitalBody` (needs a way to identify Gas Giants or bodies with atmospheres).
- `layer1::economy::ColonyResources` (to store harvested fuel/gases).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::preHarvesting;
    use crate::layer2::ship::{Ship, ShipType};
    use crate::layer2::fleet::Fleet;
    use crate::layer1::economy::{ColonyResources, ResourceType};
    use crate::layer2::system::OrbitalBody;

    fn setup_app() -> App {
        let mut app = App::new();
        app.init_resource::<ColonyResources>();
        app.add_event::<AtmosphericHarvestEvent>();
        app.add_systems(Update, process_atmospheric_harvesting);
        app
    }

    #[test]
    fn test_harvesting_shallow_skim_yields_resources_and_deals_light_damage() {
        let mut app = setup_app();

        let gas_giant = app.world_mut().spawn((
            OrbitalBody {
                name: "Zeus".to_string(),
                radius: 3.0,
                color: Color::srgb(0.8, 0.6, 0.2),
            },
            HarvestableAtmosphere { base_yield: 10.0, base_damage: 5.0 },
        )).id();

        let mut fleet = Fleet::default();
        fleet.ships.push(Ship {
            ship_type: ShipType::Miner,
            health: 40.0,
            max_health: 40.0,
        });

        let fleet_entity = app.world_mut().spawn(fleet).id();

        // Trigger shallow skim
        app.world_mut().send_event(AtmosphericHarvestEvent {
            fleet: fleet_entity,
            target: gas_giant,
            depth: SkimDepth::Shallow,
        });

        app.update();

        // Check resources
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.get(ResourceType::Fuel), 10.0); // 1.0x multiplier

        // Check damage
        let fleet = app.world().get::<Fleet>(fleet_entity).unwrap();
        assert_eq!(fleet.ships[0].health, 35.0); // 40 - 5 (1.0x damage)
    }

    #[test]
    fn test_harvesting_deep_dive_yields_more_resources_and_deals_heavy_damage() {
        let mut app = setup_app();

        let gas_giant = app.world_mut().spawn((
            OrbitalBody {
                name: "Zeus".to_string(),
                radius: 3.0,
                color: Color::srgb(0.8, 0.6, 0.2),
            },
            HarvestableAtmosphere { base_yield: 10.0, base_damage: 5.0 },
        )).id();

        let mut fleet = Fleet::default();
        fleet.ships.push(Ship {
            ship_type: ShipType::Miner,
            health: 40.0,
            max_health: 40.0,
        });

        let fleet_entity = app.world_mut().spawn(fleet).id();

        // Trigger deep dive
        app.world_mut().send_event(AtmosphericHarvestEvent {
            fleet: fleet_entity,
            target: gas_giant,
            depth: SkimDepth::Deep,
        });

        app.update();

        // Check resources
        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.get(ResourceType::Fuel), 30.0); // 3.0x multiplier

        // Check damage
        let fleet = app.world().get::<Fleet>(fleet_entity).unwrap();
        assert_eq!(fleet.ships[0].health, 25.0); // 40 - 15 (3.0x damage)
    }

    #[test]
    fn test_deep_dive_destroys_ship_if_damage_exceeds_health() {
        let mut app = setup_app();

        let gas_giant = app.world_mut().spawn(HarvestableAtmosphere { base_yield: 10.0, base_damage: 15.0 }).id();

        let mut fleet = Fleet::default();
        fleet.ships.push(Ship {
            ship_type: ShipType::Scout,
            health: 20.0, // Scout has 20 max health
            max_health: 20.0,
        });

        let fleet_entity = app.world_mut().spawn(fleet).id();

        app.world_mut().send_event(AtmosphericHarvestEvent {
            fleet: fleet_entity,
            target: gas_giant,
            depth: SkimDepth::Deep, // 3.0x multiplier = 45 damage
        });

        app.update();

        let fleet = app.world().get::<Fleet>(fleet_entity).unwrap();
        assert!(fleet.ships.is_empty(), "Ship should be destroyed by 45 damage");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer2::fleet::Fleet;
use crate::layer1::economy::{ColonyResources, ResourceType};

#[derive(Component)]
pub struct HarvestableAtmosphere {
    pub base_yield: f32,
    pub base_damage: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SkimDepth {
    Shallow,
    Deep,
}

#[derive(Event)]
pub struct AtmosphericHarvestEvent {
    pub fleet: Entity,
    pub target: Entity,
    pub depth: SkimDepth,
}

pub fn process_atmospheric_harvesting(
    mut events: EventReader<AtmosphericHarvestEvent>,
    mut fleets: Query<&mut Fleet>,
    atmospheres: Query<&HarvestableAtmosphere>,
    mut resources: ResMut<ColonyResources>,
) {
    for event in events.read() {
        let Ok(mut fleet) = fleets.get_mut(event.fleet) else { continue };
        let Ok(atmosphere) = atmospheres.get(event.target) else { continue };

        let multiplier = match event.depth {
            SkimDepth::Shallow => 1.0,
            SkimDepth::Deep => 3.0,
        };

        let damage = atmosphere.base_damage * multiplier;
        let yield_amount = atmosphere.base_yield * multiplier;

        // Deal damage to ships
        for ship in &mut fleet.ships {
            ship.health -= damage;
        }

        // Remove destroyed ships
        fleet.ships.retain(|s| s.health > 0.0);

        // Add resources
        resources.add(ResourceType::Fuel, yield_amount);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: `AtmosphericHarvestEvent` should likely be generated by a UI or command system when a fleet reaches a `HarvestableAtmosphere` location.
- **Ship Types**: Consider letting only specific ship types (like `ShipType::Miner`) perform the harvesting, or granting them a bonus yield/reduced damage.
- **Visuals**: Trigger a visual or chronicle event if a ship is destroyed during a deep dive.
- **Yield Types**: Expand `HarvestableAtmosphere` to specify *which* resource is harvested (Fuel vs. rare gases when they are added).

## 6. Acceptance Criteria
- [ ] `HarvestableAtmosphere` component is implemented.
- [ ] `AtmosphericHarvestEvent` buffers and processes correctly.
- [ ] Skimming yields resources and deals damage based on depth.
- [ ] Destroyed ships are correctly removed from fleets.
- [ ] Test coverage for the harvesting module is >85%.

## 7. Technical Guidance
- Ensure `AtmosphericHarvestEvent` is registered in `setup.rs` and buffered in `cleanup.rs`.
- Place the logic in a new file `src/layer2/atmospheric_harvesting.rs`.
- Hook up the system in `src/simulation.rs` or the appropriate layer 2 system execution set.

## 8. Questions
- Should the damage scale randomly to add unpredictability (e.g., storms are variable)?
- Should there be a cooldown on harvesting the same atmosphere?
