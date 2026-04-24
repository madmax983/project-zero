//! Religion and Belief Systems
//!
//! This module handles the spiritual and religious mechanics of the colony.
//! When pops face extreme hardship, low morale, or specific triggering events,
//! they may develop new traits and beliefs, such as forming cults.
//!
//! # Mechanics
//! - **The Prophet of the Engine:** Pops with dangerously low morale who are near
//!   certain buildings may experience a revelation, gaining the `Prophet` trait.
//! - **Cult Conversion:** Prophets actively convert nearby susceptible pops into
//!   `EngineCultist`s, spreading the belief system.
//! - **Protests:** Cultists will react negatively (generating unrest) if the objects
//!   of their worship (buildings) are dismantled.

pub mod astrological_beliefs;
pub mod prophet_of_the_engine;
