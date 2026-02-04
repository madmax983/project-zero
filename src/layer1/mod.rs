//! Layer 1: Colony Simulation.
//!
//! This is the "Dwarf Fortress" layer of SCALE. It simulates the daily lives of
//! individual Pops on a 2D tile grid.
//!
//! # The Simulation Loop
//!
//! Layer 1 runs on a deterministic tick system. Every tick (approx 100ms real time):
//!
//! 1. **Production**: Farms grow food, miners extract stone.
//! 2. **Metabolism**: Pops consume food and get tired.
//! 3. **Life & Death**: Starvation is checked, dead entities are cleaned up.
//! 4. **Milestones**: The Chronicle records history if thresholds are met.
//!
//! # Key Modules
//!
//! * **Agents**: [`pop`], [`needs`] - The simulation of living beings.
//! * **World**: [`terrain`], [`resources`] - The physical map and economy.
//! * **Player Will**: [`designation`], [`building`] - How the player influences the world.

/// Building placement and types.
pub mod building;
/// Chronicle system and historical records.
pub mod chronicle;
/// Designation system for player tools.
pub mod designation;
/// Farm building and food production.
pub mod farm;
/// Housing and rest mechanics.
pub mod housing;
/// Pop needs (hunger, rest).
pub mod needs;
/// Pop entity and management.
pub mod pop;
/// Colony resources and mining.
pub mod resources;
/// Resource storage limits and stockpile buildings.
pub mod stockpile;
/// Terrain generation and grid management.
pub mod terrain;

pub use building::*;
pub use chronicle::*;
pub use designation::*;
pub use farm::*;
pub use housing::*;
pub use needs::*;
pub use pop::*;
pub use resources::*;
pub use stockpile::*;
pub use terrain::*;
