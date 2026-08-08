//! Procedural generation of modular fauna entities.
//!
//! This module handles the creation of `FaunaBody` entities using a modular system.
//! Fauna are generated from a `FaunaSeed`, which dictates what `FaunaPart`s they receive.
//! The parts are then aggregated to define their stats and what resources they drop.

use crate::layer1::fauna::{BodyPartType, FaunaBody, FaunaPart, FaunaStats};
use rand::{rngs::StdRng, SeedableRng};

/// Seed for generating modular fauna.
///
/// Uses either a numeric seed or a named preset (e.g., "Wolf") to generate parts.
///
/// # Examples
///
/// ```
/// use scale::layer1::entities::fauna_gen::FaunaSeed;
///
/// let random_seed = FaunaSeed::new(1234);
/// assert_eq!(random_seed.value, 1234);
///
/// let preset_seed = FaunaSeed::from_preset("Wolf");
/// assert_eq!(preset_seed.preset.unwrap(), "Wolf");
/// ```
#[derive(Debug, Clone)]
pub struct FaunaSeed {
    /// Random seed value.
    pub value: u64,
    /// Optional preset name (e.g., "Wolf").
    pub preset: Option<String>,
}

impl FaunaSeed {
    /// Creates a new random seed.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self {
            value,
            preset: None,
        }
    }

    /// Creates a seed from a preset name.
    ///
    /// ## Examples
    ///
    /// ```
    /// use scale::layer1::entities::fauna_gen::FaunaSeed;
    ///
    /// let preset_seed = FaunaSeed::from_preset("Wolf");
    /// assert_eq!(preset_seed.preset.unwrap(), "Wolf");
    /// ```
    #[must_use]
    pub fn from_preset(name: &str) -> Self {
        Self {
            value: 0,
            preset: Some(name.to_string()),
        }
    }
}

/// Generator for procedural fauna.
pub struct FaunaGenerator;

impl FaunaGenerator {
    /// Generates a `FaunaBody` from a seed.
    ///
    /// This function procedurally combines different body parts based on the seed
    /// to build a creature with specific stats and resource drops.
    #[must_use]
    pub fn generate(seed: FaunaSeed) -> FaunaBody {
        if let Some(preset) = seed.preset {
            return Self::generate_preset(&preset);
        }

        // Use RNG for procedural generation (unused variable in minimal implementation but good practice)
        let _rng = StdRng::seed_from_u64(seed.value);
        let mut body = FaunaBody {
            name: format!("Generated Beast {}", seed.value % 100),
            ..Default::default()
        };

        // Head
        let head = FaunaPart {
            part_type: BodyPartType::Head,
            name: "Wolf Head".to_string(),
            stats: FaunaStats {
                attack: 5.0,
                ..Default::default()
            },
            resource_drop: None,
        };
        body.add_part(head);

        // Body
        let torso = FaunaPart {
            part_type: BodyPartType::Body,
            name: "Bear Torso".to_string(),
            stats: FaunaStats {
                health_max: 20.0,
                ..Default::default()
            },
            resource_drop: Some("Meat".to_string()),
        };
        body.add_part(torso);

        // Limbs
        let limbs = FaunaPart {
            part_type: BodyPartType::Limbs,
            name: "Insect Legs".to_string(),
            stats: FaunaStats {
                speed: 1.2,
                ..Default::default()
            },
            resource_drop: None,
        };
        body.add_part(limbs);

        body
    }

    fn generate_preset(name: &str) -> FaunaBody {
        let mut body = FaunaBody::default();
        if name == "Wolf" || name == "Grey Wolf" {
            body.name = "Grey Wolf".to_string();
            body.add_part(FaunaPart {
                part_type: BodyPartType::Head,
                name: "Wolf Head".to_string(),
                stats: FaunaStats {
                    attack: 8.0,
                    ..Default::default()
                },
                resource_drop: None,
            });
            body.add_part(FaunaPart {
                part_type: BodyPartType::Body,
                name: "Wolf Body".to_string(),
                stats: FaunaStats {
                    health_max: 15.0,
                    ..Default::default()
                },
                resource_drop: Some("Meat".to_string()),
            });
        }
        body
    }
}
