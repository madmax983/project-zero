//! Administration module.
/// General administration mechanics.
pub mod admin;
/// Policies controlling sleep cycles.
pub mod bureaucracy_of_sleep;
/// Map designations and zones.
pub mod designation;
/// Laws and colony edicts.
pub mod edicts;
/// Inspections and rule enforcement.
pub mod inspector;
/// Malicious compliance AI mechanics.
pub mod malicious_compliance;
/// Permitting system for actions.
pub mod permit;
/// Zones and their definitions.
pub mod zone;

pub use admin::*;
pub use bureaucracy_of_sleep::*;
pub use designation::*;
pub use edicts::*;
pub use inspector::*;
pub use malicious_compliance::*;
pub use permit::*;
pub use zone::*;
pub mod invasive_bureaucracy;
pub use invasive_bureaucracy::*;

pub mod bureaucratic_redlining;
pub use bureaucratic_redlining::*;
pub mod bureaucratic_black_hole;
pub use bureaucratic_black_hole::*;
pub mod feral_administration;
pub use feral_administration::*;
pub mod sentient_bureaucracy;
pub use sentient_bureaucracy::*;
