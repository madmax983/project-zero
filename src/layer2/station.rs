use crate::layer1::resources::ResourceType;
use crate::layer2::fleet::{Fleet, FleetOrder, InOrbit};
use crate::layer2::mining::FleetCargo;
use crate::layer2::system::{Orbit, OrbitalBody};
use bevy_ecs::prelude::*;
use ratatui::style::Color;

/// Types of stations that can be built by a fleet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StationType {
    /// A small outpost for basic operations.
    Outpost,
    /// A platform for mining operations.
    MiningPlatform,
    /// A facility for ship construction and repair.
    Shipyard,
}

impl StationType {
    /// Returns the resource cost to build this station.
    #[must_use]
    pub fn cost(&self) -> Vec<(ResourceType, f32)> {
        match self {
            Self::Outpost => vec![(ResourceType::Metal, 50.0)],
            Self::MiningPlatform => vec![(ResourceType::Metal, 100.0), (ResourceType::Fuel, 10.0)],
            Self::Shipyard => vec![(ResourceType::Metal, 200.0), (ResourceType::Fuel, 50.0)],
        }
    }

    /// Returns the display label for the station type.
    #[must_use]
    pub const fn label(&self) -> &str {
        match self {
            Self::Outpost => "Outpost",
            Self::MiningPlatform => "Mining Platform",
            Self::Shipyard => "Shipyard",
        }
    }

    /// Returns the character representation for the station type.
    #[must_use]
    pub const fn char(&self) -> char {
        match self {
            Self::Outpost => '+',
            Self::MiningPlatform => '⚒',
            Self::Shipyard => '⚓',
        }
    }
}

/// Component marking an entity as a space station.
#[derive(Component, Debug, Clone)]
pub struct Station {
    /// The type of station.
    pub station_type: StationType,
}

/// System to handle `FleetOrder::BuildStation`.
///
/// Checks if the fleet has sufficient resources, deducts them, and spawns the station.
pub fn build_station_system(
    mut commands: Commands,
    mut query: Query<(Entity, &FleetOrder, &InOrbit, &mut FleetCargo), With<Fleet>>,
) {
    for (entity, order, orbit, mut cargo) in &mut query {
        if let FleetOrder::BuildStation(station_type) = *order {
            let cost = station_type.cost();

            // Check affordability (summing all stacks of required type)
            let can_afford = cost.iter().all(|(res, amt)| {
                let total_available: f32 = cargo
                    .contents
                    .iter()
                    .filter(|s| s.resource_type == *res)
                    .map(|s| s.amount)
                    .sum();
                total_available >= *amt
            });

            if can_afford {
                // Deduct resources
                for (res, amt) in &cost {
                    let mut remaining_to_deduct = *amt;
                    for stack in cargo
                        .contents
                        .iter_mut()
                        .filter(|s| s.resource_type == *res)
                    {
                        if remaining_to_deduct <= 0.0 {
                            break;
                        }
                        let deduction = stack.amount.min(remaining_to_deduct);
                        stack.amount -= deduction;
                        remaining_to_deduct -= deduction;
                    }
                }

                // Cleanup empty stacks
                cargo.contents.retain(|s| s.amount > 0.0);

                // Spawn Station
                commands.spawn((
                    Station { station_type },
                    OrbitalBody {
                        name: format!("{} {}", station_type.label(), entity.index()), // Unique-ish name
                        radius: 0.5,
                        color: Color::Cyan,
                        char: station_type.char(),
                    },
                    Orbit {
                        parent: orbit.parent,
                        radius: 10.0, // Fixed radius for now as per MVP
                        speed: 0.05,
                        angle: 0.0,
                    },
                ));

                // Consume Order
                commands.entity(entity).remove::<FleetOrder>();
            }
        }
    }
}
