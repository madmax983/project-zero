pub mod building;
#[cfg(test)]
mod building_gate_test;
#[cfg(test)]
mod work_building_tests;

pub mod housing;
pub mod structure;
#[cfg(test)]
mod structure_fragile_tests;
#[cfg(test)]
mod structure_jury_rig_tests;
#[cfg(test)]
mod structure_maintenance_tests;

pub mod parasitic_architecture;
pub mod spontaneous_architecture;
pub mod symbiotic_infrastructure;
pub mod room_quality;
pub mod ruins;
pub mod turret;
pub mod window;

pub use building::*;
pub use housing::*;
pub use structure::*;
pub use parasitic_architecture::*;
pub use spontaneous_architecture::*;
pub use symbiotic_infrastructure::*;
pub use room_quality::*;
pub use ruins::*;
pub use turret::*;
pub use window::*;
