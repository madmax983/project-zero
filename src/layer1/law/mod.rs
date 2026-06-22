//! Law and Justice systems.
//!
//! Contains systems for crime, punishment, policing, and penal contracts.

pub mod aesthetic_edict;
pub mod contraband;
pub mod embassy;
pub mod justice;
pub mod orphaned_edict;
pub mod penal;
pub mod predictive_policing;
pub mod sanctuary;

#[cfg(test)]
#[allow(missing_docs)]
pub mod justice_tests;
pub mod rogue_ai_arbitration;
pub use rogue_ai_arbitration::{Edict, Infraction};
pub mod prohibition;
