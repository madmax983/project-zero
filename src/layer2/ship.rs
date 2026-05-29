use crate::layer1::resources::ResourceType;

/// Defines the class of a ship, determining its stats and capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShipType {
    /// Fast, low cargo, cheap. Used for exploration.
    Scout,
    /// Slow, high cargo, expensive. Used for hauling.
    Transport,
    /// Medium speed, medium cargo. Equipped for mining operations.
    Miner,
    /// Combat vessel. High speed, low cargo, expensive.
    Frigate,
    /// Massive morale boosting ship built from scrap.
    Memorial,
}

impl ShipType {
    /// Returns the base movement speed of the ship type.
    /// Higher is faster.
    #[must_use]
    pub const fn base_speed(&self) -> f32 {
        match self {
            Self::Scout => 2.0,
            Self::Frigate => 1.5,
            Self::Miner => 0.8,
            Self::Transport => 0.5,
            Self::Memorial => 0.3,
        }
    }

    /// Returns the cargo capacity of the ship type.
    #[must_use]
    pub const fn cargo_capacity(&self) -> f32 {
        match self {
            Self::Scout => 10.0,
            Self::Frigate => 50.0,
            Self::Miner => 200.0,
            Self::Transport => 1000.0,
            Self::Memorial => 0.0,
        }
    }

    /// Returns the resource cost to construct this ship.
    #[must_use]
    pub fn construction_cost(&self) -> Vec<(ResourceType, f32)> {
        match self {
            Self::Scout => vec![(ResourceType::Metal, 50.0), (ResourceType::Fuel, 20.0)],
            Self::Transport => vec![(ResourceType::Metal, 200.0), (ResourceType::Fuel, 50.0)],
            Self::Miner => vec![(ResourceType::Metal, 100.0), (ResourceType::Fuel, 30.0)],
            Self::Frigate => vec![(ResourceType::Metal, 150.0), (ResourceType::Fuel, 40.0)],
            Self::Memorial => vec![(ResourceType::Scrap, 500.0), (ResourceType::Fuel, 100.0)],
        }
    }

    /// Returns the attack power of the ship.
    #[must_use]
    pub const fn attack_power(&self) -> f32 {
        match self {
            Self::Frigate => 50.0,
            Self::Miner => 10.0,
            Self::Transport => 5.0,
            Self::Scout => 2.0,
            Self::Memorial => 10.0,
        }
    }

    /// Returns the max health of the ship.
    #[must_use]
    pub const fn max_health(&self) -> f32 {
        match self {
            Self::Frigate => 100.0,
            Self::Miner => 40.0,
            Self::Transport => 50.0,
            Self::Scout => 20.0,
            Self::Memorial => 500.0,
        }
    }
}

/// Represents a Pop assigned to off-world duties.
#[derive(bevy_ecs::prelude::Component, Debug, Clone, PartialEq, Default)]
pub struct OffWorldDuty;

/// Represents an individual ship instance within a fleet.
#[derive(bevy_ecs::prelude::Component, Debug, Clone, PartialEq)]
pub struct Ship {
    /// The class of the ship.
    pub ship_type: ShipType,
    /// Current structural integrity.
    pub health: f32,
    /// Maximum structural integrity.
    pub max_health: f32,
}

impl Ship {
    /// Creates a new ship of the given type with full health.
    #[must_use]
    pub const fn new(ship_type: ShipType) -> Self {
        Self {
            ship_type,
            health: ship_type.max_health(),
            max_health: ship_type.max_health(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::resources::ResourceType;
    use crate::layer2::fleet::FleetComposition;

    #[test]
    fn test_ship_type_stats() {
        // Scout: Fast, Low Cargo
        let scout = ShipType::Scout;
        assert_eq!(scout.base_speed(), 2.0);
        assert_eq!(scout.cargo_capacity(), 10.0);

        // Transport: Slow, High Cargo
        let transport = ShipType::Transport;
        assert_eq!(transport.base_speed(), 0.5);
        assert_eq!(transport.cargo_capacity(), 1000.0);

        // Miner: Medium Speed, Medium Cargo, Mining Ability
        let miner = ShipType::Miner;
        assert_eq!(miner.base_speed(), 0.8);
        assert_eq!(miner.cargo_capacity(), 200.0);
    }

    #[test]
    fn test_fleet_composition_calculates_total_cargo() {
        let mut comp = FleetComposition::default();
        comp.add_ship(Ship::new(ShipType::Scout)); // 10
        comp.add_ship(Ship::new(ShipType::Transport)); // 1000

        assert_eq!(comp.total_cargo_capacity(), 1010.0);
    }

    #[test]
    fn test_fleet_composition_calculates_min_speed() {
        let mut comp = FleetComposition::default();
        comp.add_ship(Ship::new(ShipType::Scout)); // 2.0
        comp.add_ship(Ship::new(ShipType::Transport)); // 0.5

        // Fleet moves at speed of slowest ship
        assert_eq!(comp.speed(), 0.5);
    }

    #[test]
    fn test_fleet_composition_empty_has_default_speed() {
        let comp = FleetComposition::default();
        // Empty fleet (e.g., just a probe?) or invalid?
        // Let's say 0.0 or 1.0.
        // Logic: An empty fleet shouldn't exist, but if it does, it has no speed constraints?
        // Or 0.0 to prevent movement.
        assert_eq!(comp.speed(), 0.0);
    }

    #[test]
    fn test_ship_construction_cost() {
        let scout = ShipType::Scout;
        let cost = scout.construction_cost();

        // Check contents
        assert!(cost.contains(&(ResourceType::Metal, 50.0)));
        assert!(cost.contains(&(ResourceType::Fuel, 20.0))); // Initial fueling? Or construction energy?
    }
}

pub mod logistics;
