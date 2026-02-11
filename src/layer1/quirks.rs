use crate::layer1::building::{Building, BuildingType};
use crate::layer1::day_night::DayNightCycle;
use crate::layer1::energy::PowerSource;
use crate::layer1::pop::Speed;
use bevy_ecs::prelude::*;

/// Traits that modify planetary conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanetaryTrait {
    /// High gravity slows movement (-20%).
    HighGravity,
    /// Low gravity speeds movement (+20%).
    LowGravity,
    /// Rapid orbit shortens day length (-50%).
    RapidOrbit,
    /// Slow orbit lengthens day length (+100%).
    SlowOrbit,
    /// Dense atmosphere reduces solar power output (-20%).
    DenseAtmosphere,
    /// Thin atmosphere increases solar power output (+20%).
    ThinAtmosphere,
}

impl PlanetaryTrait {
    /// Returns the speed modifier for this trait.
    #[must_use]
    pub const fn speed_modifier(&self) -> f32 {
        match self {
            Self::HighGravity => 0.8,
            Self::LowGravity => 1.2,
            _ => 1.0,
        }
    }

    /// Returns the day length modifier for this trait.
    #[must_use]
    pub const fn day_length_modifier(&self) -> f32 {
        match self {
            Self::RapidOrbit => 0.5,
            Self::SlowOrbit => 2.0,
            _ => 1.0,
        }
    }

    /// Returns the power output modifier for this trait.
    #[must_use]
    pub const fn power_output_modifier(&self) -> f32 {
        match self {
            Self::DenseAtmosphere => 0.8,
            Self::ThinAtmosphere => 1.2,
            _ => 1.0,
        }
    }

    /// Returns the human-readable label of the trait.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::HighGravity => "High Gravity",
            Self::LowGravity => "Low Gravity",
            Self::RapidOrbit => "Rapid Orbit",
            Self::SlowOrbit => "Slow Orbit",
            Self::DenseAtmosphere => "Dense Atmosphere",
            Self::ThinAtmosphere => "Thin Atmosphere",
        }
    }
}

/// Resource containing the active planetary traits for the current game.
#[derive(Resource, Default, Debug)]
pub struct PlanetaryTraits(pub Vec<PlanetaryTrait>);

/// System that applies modifiers from planetary traits to simulation components.
///
/// This affects:
/// - `Speed`: Multiplies `current` speed.
/// - `DayNightCycle`: Sets `ticks_per_day` based on base value (250).
/// - `PowerSource`: Sets `output` for Generators based on base value (10.0).
pub fn apply_quirk_modifiers_system(
    traits: Res<PlanetaryTraits>,
    mut pops: Query<&mut Speed>,
    mut day_night: ResMut<DayNightCycle>,
    mut power_sources: Query<(&mut PowerSource, &Building)>,
) {
    // Constants defined at top of scope to appease clippy
    const BASE_TICKS_PER_DAY: f32 = 250.0;
    const BASE_GENERATOR_OUTPUT: f32 = 10.0;

    if traits.0.is_empty() {
        return;
    }

    // 1. Calculate aggregate modifiers
    let mut speed_mod = 1.0;
    let mut day_mod = 1.0;
    let mut power_mod = 1.0;

    for trait_ in &traits.0 {
        speed_mod *= trait_.speed_modifier();
        day_mod *= trait_.day_length_modifier();
        power_mod *= trait_.power_output_modifier();
    }

    // 2. Apply to Pops (Speed)
    // We multiply `current` instead of setting from `base` to respect other modifiers (like Lighting).
    for mut speed in &mut pops {
        speed.current *= speed_mod;
    }

    // 3. Apply to DayNightCycle
    // We use a hardcoded base value because DayNightCycle does not store one.
    // Default is 250 ticks per day.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    {
        day_night.ticks_per_day = (BASE_TICKS_PER_DAY * day_mod) as u64;
    }

    // 4. Apply to Power Sources
    // Only affect Generators.
    // We use a hardcoded base value (10.0) because PowerSource does not store one.
    for (mut source, building) in &mut power_sources {
        if matches!(building.building_type, BuildingType::Generator) {
            source.output = BASE_GENERATOR_OUTPUT * power_mod;
        }
    }
}
