use crate::layer1::inventory::Inventory;
use crate::layer1::items::ItemType;
use bevy::prelude::*;

/// Component indicating the station is a Stellar Forge and holds its production state.
#[derive(Component, Default)]
pub struct StellarForge {
    /// Rate at which the forge produces alloys.
    pub production_rate: f32,
    /// Whether the forge is currently active.
    pub active: bool,
}

/// Component tracking the thermal stress of a Stellar Forge.
#[derive(Component, Default)]
pub struct ThermalStress {
    /// The current thermal stress level.
    pub current_stress: f32,
    /// The maximum thermal stress before meltdown.
    pub max_stress: f32,
    /// Rate at which stress increases when uncooled, or decreases when cooled.
    pub stress_rate: f32,
}

/// Event emitted when a Stellar Forge undergoes a catastrophic meltdown.
#[derive(Event)]
pub struct SolarFlareEvent {
    /// The entity of the Stellar Forge that melted down.
    pub source_entity: Entity,
    /// The severity of the solar flare (typically max stress).
    pub severity: f32,
}

/// System that processes the conversion of RawMass and Coolant into StellarAlloy.
pub fn stellar_forge_production_system(mut query: Query<(&StellarForge, &mut Inventory)>) {
    for (forge, mut inventory) in query.iter_mut() {
        if forge.active {
            // Simplified recipe: 10 RawMass + 1 Coolant = 1 StellarAlloy
            if inventory.get_count(ItemType::RawMass) >= 10
                && inventory.get_count(ItemType::Coolant) >= 1
            {
                inventory.remove(ItemType::RawMass, 10);
                inventory.remove(ItemType::Coolant, 1);
                inventory.add_count(ItemType::StellarAlloy, 1);
            }
        }
    }
}

/// System that manages thermal stress, cooling it if coolant is available, or causing a meltdown otherwise.
pub fn thermal_stress_management_system(
    mut commands: Commands,
    mut query: Query<(Entity, &StellarForge, &mut ThermalStress, &Inventory)>,
    mut flare_events: EventWriter<SolarFlareEvent>,
) {
    for (entity, forge, mut stress, inventory) in query.iter_mut() {
        if forge.active {
            // If we have coolant, we reduce stress. Otherwise, stress increases.
            if inventory.get_count(ItemType::Coolant) > 0 {
                stress.current_stress = (stress.current_stress - stress.stress_rate).max(0.0);
            } else {
                stress.current_stress += stress.stress_rate;
            }

            if stress.current_stress >= stress.max_stress {
                // Meltdown!
                flare_events.send(SolarFlareEvent {
                    source_entity: entity,
                    severity: stress.current_stress,
                });

                // Destroy the forge to prevent endless flares
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::station::{Station, StationType};
    use bevy_app::{App, Update};

    #[test]
    fn test_stellar_forge_produces_without_fuel_when_cooled() {
        let mut app = App::new();
        app.add_systems(Update, stellar_forge_production_system);

        // Arrange: Forge with coolant
        let mut inventory = Inventory::new();
        inventory.capacity = 1000; // ensure capacity is enough to hold 70 items
        inventory.add_count(ItemType::RawMass, 50); // Raw material
        inventory.add_count(ItemType::Coolant, 20); // Required to prevent stress/allow production

        let forge_entity = app
            .world_mut()
            .spawn((
                Station {
                    station_type: StationType::StellarForge,
                },
                StellarForge {
                    production_rate: 1.0,
                    active: true,
                },
                ThermalStress {
                    current_stress: 0.0,
                    max_stress: 100.0,
                    stress_rate: 5.0,
                },
                inventory,
            ))
            .id();

        // Act
        app.update();

        // Assert: RawMass converted to StellarAlloy, Coolant consumed
        let inventory_after = app.world().get::<Inventory>(forge_entity).unwrap();
        assert!(inventory_after.has_item(ItemType::StellarAlloy));
        assert!(inventory_after.get_count(ItemType::Coolant) < 20);
    }

    #[test]
    fn test_stellar_forge_accumulates_stress_without_coolant() {
        let mut app = App::new();
        app.add_event::<SolarFlareEvent>();
        app.add_systems(Update, thermal_stress_management_system);

        // Arrange: Forge without coolant
        let forge_entity = app
            .world_mut()
            .spawn((
                Station {
                    station_type: StationType::StellarForge,
                },
                StellarForge {
                    production_rate: 1.0,
                    active: true,
                },
                ThermalStress {
                    current_stress: 0.0,
                    max_stress: 100.0,
                    stress_rate: 10.0,
                },
                Inventory::new(), // No Coolant
            ))
            .id();

        // Act
        app.update();

        // Assert: Stress increased
        let stress_after = app.world().get::<ThermalStress>(forge_entity).unwrap();
        assert!(
            stress_after.current_stress > 0.0,
            "Thermal stress should increase when no coolant is available"
        );
    }

    #[test]
    fn test_stellar_forge_meltdown_causes_solar_flare() {
        let mut app = App::new();
        app.add_event::<SolarFlareEvent>();
        app.add_systems(Update, thermal_stress_management_system);

        // Arrange: Forge at max stress
        let forge_entity = app
            .world_mut()
            .spawn((
                Station {
                    station_type: StationType::StellarForge,
                },
                StellarForge {
                    production_rate: 1.0,
                    active: true,
                },
                ThermalStress {
                    current_stress: 100.0,
                    max_stress: 100.0,
                    stress_rate: 10.0,
                },
                Inventory::new(),
            ))
            .id();

        // Act
        app.update();

        // Assert: SolarFlareEvent fired
        let events = app.world().resource::<Events<SolarFlareEvent>>();
        let mut reader = events.get_cursor();
        let flare_events: Vec<_> = reader.read(events).collect();

        assert_eq!(
            flare_events.len(),
            1,
            "SolarFlareEvent should trigger on meltdown"
        );
        assert_eq!(flare_events[0].source_entity, forge_entity);

        // Assert: Entity despawned
        assert!(
            app.world().get_entity(forge_entity).is_err(),
            "Entity should be despawned after meltdown"
        );
    }
}
