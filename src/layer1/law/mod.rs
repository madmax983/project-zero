//! Law and Justice systems.
//!
//! Contains systems for crime, punishment, policing, and penal contracts.

pub mod aesthetic_edict;
pub mod contraband;
pub mod justice;
pub mod orphaned_edict;
pub mod penal;
pub mod predictive_policing;
pub mod embassy;

#[cfg(test)]
#[allow(missing_docs)]
pub mod justice_tests;
#[allow(missing_docs)]
pub mod embassy_tests;
