use crate::layer2::barnacles::{calculate_speed_modifier, SpaceBarnacles};
use crate::layer2::ship::Ship;
use crate::layer2::station::StationType;
use bevy_ecs::prelude::*;

/// Factions that can own fleets.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FleetFaction {
    /// The player's faction.
    Player,
    /// Hostile pirate faction.
    Pirate,
    /// Neutral merchant faction.
    Merchant,
}

/// Component marking an entity as a Fleet.
///
/// Fleets are mobile units in the system view that can travel between orbital bodies.
#[derive(Component, Debug, Clone, Copy)]
pub struct Fleet;

/// Component indicating a fleet is stationary at an orbital body.
#[derive(Component, Debug, Clone, Copy)]
pub struct InOrbit {
    /// The entity (Star, Planet, Moon) the fleet is orbiting.
    pub parent: Entity,
}

/// Component tracking fleet health.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct FleetHealth {
    /// Current health.
    pub current: f32,
    /// Maximum health.
    pub max: f32,
}

/// Component indicating a fleet is moving between two locations.
#[derive(Component, Debug, Clone, Copy)]
pub struct InTransit {
    /// The starting point of the journey.
    pub origin: Entity,
    /// The destination of the journey.
    pub destination: Entity,
    /// Current progress of the journey (0.0 to 1.0).
    pub progress: f32,
    /// Total duration of the journey in ticks.
    pub duration: f32,
}

/// Commands issued to a fleet.
#[derive(Component, Debug, Clone, Copy)]
pub enum FleetOrder {
    /// Order to move to a specific entity.
    MoveTo(Entity),
    /// Order to build a station.
    BuildStation(StationType),
    /// Order to mine a target entity.
    Mine(Entity),
    /// Order to claim a derelict station.
    ClaimStation(Entity),
    /// Order to drop resources onto the colony.
    Drop(crate::layer1::economy::resources::ResourceType, f32),
}

/// System to process `FleetOrder`s.
///
/// Transitions fleets from `InOrbit` to `InTransit` when a `MoveTo` order is received.
pub fn fleet_order_system(
    mut commands: Commands,
    query: Query<(Entity, &FleetOrder, Option<&InOrbit>), With<Fleet>>,
) {
    for (entity, order, maybe_orbit) in &query {
        if let FleetOrder::MoveTo(target) = order {
            // Determine origin
            let origin = if let Some(orbit) = maybe_orbit {
                orbit.parent
            } else {
                // For MVP, if not in orbit, we ignore.
                continue;
            };

            commands
                .entity(entity)
                .remove::<FleetOrder>()
                .remove::<InOrbit>()
                .insert(InTransit {
                    origin,
                    destination: *target,
                    progress: 0.0,
                    duration: 100.0, // Fixed duration for MVP
                });
        }
    }
}

/// System to ensure all fleets have the `FleetHealth` component.
#[allow(clippy::type_complexity)]
pub fn ensure_fleet_health_system(
    mut commands: Commands,
    query: Query<(Entity, Option<&FleetComposition>), (With<Fleet>, Without<FleetHealth>)>,
) {
    for (entity, composition) in &query {
        let (current, max) = composition.map_or((100.0, 100.0), |comp| {
            let total_health: f32 = comp.ships.iter().map(|s| s.health).sum();
            let max_health: f32 = comp.ships.iter().map(|s| s.max_health).sum();
            (total_health, max_health)
        });

        commands.entity(entity).insert(FleetHealth { current, max });
    }
}

/// System to update fleet positions during transit.
///
/// Increments progress and handles arrival when progress >= 1.0.
pub fn fleet_movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut InTransit, Option<&SpaceBarnacles>)>,
) {
    for (entity, mut transit, maybe_barnacles) in &mut query {
        let speed_mod = maybe_barnacles.map_or(1.0, |b| calculate_speed_modifier(b.count));

        // Increment progress
        let delta = (1.0 / transit.duration) * speed_mod;
        transit.progress += delta;

        if transit.progress >= 1.0 {
            // Arrive
            let destination = transit.destination;
            commands
                .entity(entity)
                .remove::<InTransit>()
                .insert(InOrbit {
                    parent: destination,
                });
        }
    }
}

/// Component representing the composition of a fleet (ships).
#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct FleetComposition {
    /// The ships in the fleet.
    pub ships: Vec<Ship>,
}

impl FleetComposition {
    /// Adds a ship to the fleet.
    pub fn add_ship(&mut self, ship: Ship) {
        self.ships.push(ship);
    }

    /// Calculates the total cargo capacity of the fleet.
    #[must_use]
    pub fn total_cargo_capacity(&self) -> f32 {
        self.ships
            .iter()
            .map(|s| s.ship_type.cargo_capacity())
            .sum()
    }

    /// Calculates the speed of the fleet (determined by the slowest ship).
    #[must_use]
    pub fn speed(&self) -> f32 {
        if self.ships.is_empty() {
            return 0.0;
        }
        // Minimal speed of all ships
        self.ships
            .iter()
            .map(|s| s.ship_type.base_speed())
            .fold(f32::INFINITY, f32::min)
    }

    /// Applies damage to the fleet, destroying ships if necessary.
    /// Returns a list of destroyed ship types.
    pub fn take_damage(&mut self, mut damage: f32) -> Vec<crate::layer2::ship::ShipType> {
        let mut destroyed = Vec::new();
        let mut survivors = Vec::new();

        for mut ship in self.ships.drain(..) {
            if damage >= ship.health {
                damage -= ship.health;
                destroyed.push(ship.ship_type);
            } else {
                ship.health -= damage;
                survivors.push(ship);
                damage = 0.0;
            }
        }
        self.ships = survivors;
        destroyed
    }
}

#[cfg(test)]
mod tests {
    use crate::layer2::fleet::{
        fleet_movement_system, fleet_order_system, Fleet, FleetOrder, InOrbit, InTransit,
    };
    use crate::layer2::system::OrbitalBody;
    use bevy_ecs::prelude::*;
    use ratatui::style::Color;

    fn setup_world() -> World {
        // Register components if needed
        World::new()
    }

    #[test]
    fn test_fleet_spawn_in_orbit() {
        let mut world = setup_world();
        let planet = world.spawn_empty().id();

        let fleet = world
            .spawn((
                Fleet,
                InOrbit { parent: planet },
                OrbitalBody {
                    name: "Scout 1".to_string(),
                    radius: 0.0, // Fleets are points usually, or very small
                    color: Color::White,
                    char: '▲',
                },
            ))
            .id();

        let in_orbit = world
            .get::<InOrbit>(fleet)
            .expect("Fleet should be in orbit");
        assert_eq!(in_orbit.parent, planet);
    }

    #[test]
    fn test_order_fleet_movement() {
        let mut world = setup_world();
        let planet_a = world.spawn_empty().id();
        let planet_b = world.spawn_empty().id();

        let fleet = world.spawn((Fleet, InOrbit { parent: planet_a })).id();

        // Issue Move Order
        world.entity_mut(fleet).insert(FleetOrder::MoveTo(planet_b));

        // Run Order System
        let mut schedule = Schedule::default();
        schedule.add_systems(fleet_order_system);
        schedule.run(&mut world);

        // Verify Fleet is now InTransit
        assert!(
            world.get::<InOrbit>(fleet).is_none(),
            "Fleet should leave orbit"
        );
        assert!(
            world.get::<FleetOrder>(fleet).is_none(),
            "Order should be consumed"
        );

        let transit = world
            .get::<InTransit>(fleet)
            .expect("Fleet should be in transit");
        assert_eq!(transit.origin, planet_a);
        assert_eq!(transit.destination, planet_b);
        assert!(transit.progress.abs() < f32::EPSILON);
        assert!(transit.duration > 0.0);
    }

    #[test]
    fn test_fleet_transit_progress() {
        let mut world = setup_world();
        let planet_a = world.spawn_empty().id();
        let planet_b = world.spawn_empty().id();

        let fleet = world
            .spawn((
                Fleet,
                InTransit {
                    origin: planet_a,
                    destination: planet_b,
                    progress: 0.5,
                    duration: 10.0, // 10 ticks
                },
            ))
            .id();

        // Run Movement System
        let mut schedule = Schedule::default();
        schedule.add_systems(fleet_movement_system);
        schedule.run(&mut world);

        let transit = world.get::<InTransit>(fleet).unwrap();
        // Progress should increase by 1.0 / duration
        // 0.5 + (1.0/10.0) = 0.6
        assert!((transit.progress - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fleet_arrival() {
        let mut world = setup_world();
        let planet_a = world.spawn_empty().id();
        let planet_b = world.spawn_empty().id();

        let fleet = world
            .spawn((
                Fleet,
                InTransit {
                    origin: planet_a,
                    destination: planet_b,
                    progress: 0.95,
                    duration: 10.0,
                },
            ))
            .id();

        // Run Movement System (should complete transit)
        let mut schedule = Schedule::default();
        schedule.add_systems(fleet_movement_system);
        schedule.run(&mut world);

        // Verify Fleet is InOrbit at destination
        assert!(
            world.get::<InTransit>(fleet).is_none(),
            "Fleet should arrive"
        );
        let in_orbit = world
            .get::<InOrbit>(fleet)
            .expect("Fleet should be in orbit");
        assert_eq!(in_orbit.parent, planet_b);
    }
}
