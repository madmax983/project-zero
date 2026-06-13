//! Galactic Market
//!
//! Handles the galaxy-wide trading system, fluctuating prices based on supply and demand,
//! ephemeral shadow markets, and famine events.
pub mod galactic_market;
pub use galactic_market::*;
pub mod quantum_famine;
pub use quantum_famine::*;
pub mod ephemeral_market;
pub use ephemeral_market::*;
pub mod orbital_debt_collection;
