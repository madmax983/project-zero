pub mod health;
pub mod medical;
pub mod genetics;
pub mod addiction;
pub mod rust_lung;
pub mod contagion;
pub mod cryo_shock;
pub mod cybernetics;
pub mod gene_bank;
pub mod grafting;
pub mod gut_biome;
pub mod biocompatibility;
pub mod clone_vat;
pub mod agony_extract;

#[cfg(test)]
#[allow(missing_docs)]
pub mod medical_triage_tests;

pub use addiction::*;
pub use agony_extract::*;
pub use biocompatibility::*;
pub use clone_vat::*;
pub use contagion::*;
pub use cryo_shock::*;
pub use cybernetics::*;
pub use gene_bank::*;
pub use genetics::*;
pub use grafting::*;
pub use gut_biome::*;
pub use health::*;
pub use medical::*;
