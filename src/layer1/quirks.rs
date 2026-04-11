//! # Planetary Traits (Quirks)
//!
//! This module defines the planetary-scale physical and atmospheric modifiers that
//! alter the fundamental constants of a simulation scenario. Rather than hardcoding
//! environment variables like gravity or orbit duration, traits like [`PlanetaryTrait::HighGravity`]
//! and [`PlanetaryTrait::RapidOrbit`] dynamically shift these rules.
//!
//! This is the mechanical foundation of a planet's "Personality", affecting base entity stats like
//! pop movement, solar power output, and the total tick duration of a solar day.
//!
//! ## Examples
//!
//! Simulating a world with a thin atmosphere and a slow orbit:
//!
//! ```rust
//! use bevy_ecs::prelude::*;
//! use scale::layer1::quirks::{PlanetaryTrait, PlanetaryTraits, apply_quirk_modifiers_system};
//! use scale::layer1::day_night::DayNightCycle;
//!
//! let mut world = World::new();
//!
//! // Define the specific traits for the scenario
//! world.insert_resource(PlanetaryTraits(vec![
//!     PlanetaryTrait::ThinAtmosphere,
//!     PlanetaryTrait::SlowOrbit,
//! ]));
//!
//! // Provide the necessary default component for the cycle
//! world.insert_resource(DayNightCycle::default());
//!
//! // Run the application of modifiers
//! let mut schedule = Schedule::default();
//! schedule.add_systems(apply_quirk_modifiers_system);
//! schedule.run(&mut world);
//!
//! // The slow orbit trait lengthens the day by 100%.
//! // Standard base tick duration is 250 ticks.
//! let day_night = world.resource::<DayNightCycle>();
//! assert_eq!(day_night.ticks_per_day, 500);
//! ```

use crate::layer1::atmosphere::AtmosphereGrid;
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
    /// Retrieves the multiplier applied to the base movement speed of pops.
    ///
    /// Extreme gravitational forces directly alter traversal times across the map.
    /// High gravity worlds create slower logistical networks, making clustered base
    /// designs far more efficient than sprawling ones. Low gravity worlds allow
    /// rapid expansion but often come paired with thin atmospheres.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use scale::layer1::quirks::PlanetaryTrait;
    ///
    /// let quirk = PlanetaryTrait::HighGravity;
    /// assert_eq!(quirk.speed_modifier(), 0.8);
    /// ```
    #[must_use]
    pub const fn speed_modifier(&self) -> f32 {
        match self {
            Self::HighGravity => 0.8,
            Self::LowGravity => 1.2,
            _ => 1.0,
        }
    }

    /// Retrieves the multiplier applied to the base tick length of a solar day.
    ///
    /// Modifying the day length alters the frequency of the [`DayNightCycle`],
    /// which dictates pop sleep schedules, solar power generation windows, and
    /// nocturnal threat spawning. A rapid orbit forces frequent context switching
    /// for pops, potentially increasing stress.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use scale::layer1::quirks::PlanetaryTrait;
    ///
    /// let quirk = PlanetaryTrait::RapidOrbit;
    /// assert_eq!(quirk.day_length_modifier(), 0.5);
    /// ```
    #[must_use]
    pub const fn day_length_modifier(&self) -> f32 {
        match self {
            Self::RapidOrbit => 0.5,
            Self::SlowOrbit => 2.0,
            _ => 1.0,
        }
    }

    /// Retrieves the multiplier applied to the output of `Generator` building types.
    ///
    /// Atmospheric density directly affects light refraction and wind resistance.
    /// A dense atmosphere chokes out solar arrays, forcing colonies to rely
    /// heavily on subterranean geothermal or nuclear power sources early on.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use scale::layer1::quirks::PlanetaryTrait;
    ///
    /// let quirk = PlanetaryTrait::DenseAtmosphere;
    /// assert_eq!(quirk.power_output_modifier(), 0.8);
    /// ```
    #[must_use]
    pub const fn power_output_modifier(&self) -> f32 {
        match self {
            Self::DenseAtmosphere => 0.8,
            Self::ThinAtmosphere => 1.2,
            _ => 1.0,
        }
    }

    /// Retrieves the atmospheric diffusion modifier affecting the `AtmosphereGrid`.
    ///
    /// High diffusion rates mean pollution and toxic gases dissipate quickly.
    /// A dense atmosphere traps gases, meaning heavy industry will rapidly render
    /// a local region uninhabitable without extensive precursor scrubber technology.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use scale::layer1::quirks::PlanetaryTrait;
    ///
    /// let quirk = PlanetaryTrait::ThinAtmosphere;
    /// assert_eq!(quirk.diffusion_modifier(), 0.91);
    /// ```
    #[must_use]
    pub const fn diffusion_modifier(&self) -> f32 {
        match self {
            Self::DenseAtmosphere => 1.009, // ~0.999 retention
            Self::ThinAtmosphere => 0.91,   // ~0.90 retention
            _ => 1.0,
        }
    }

    /// Retrieves the UI-facing localization key or plain-text label for the trait.
    ///
    /// Used by the inspector UI to display the active planetary conditions.
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
/// - `AtmosphereGrid`: Sets `diffusion_rate` based on base value (0.99).
pub fn apply_quirk_modifiers_system(
    traits: Res<PlanetaryTraits>,
    mut pops: Query<&mut Speed>,
    mut day_night: ResMut<DayNightCycle>,
    mut power_sources: Query<(&mut PowerSource, &Building)>,
    atmosphere: Option<ResMut<AtmosphereGrid>>,
) {
    // Constants defined at top of scope to appease clippy
    const BASE_TICKS_PER_DAY: f32 = 250.0;
    const BASE_GENERATOR_OUTPUT: f32 = 10.0;
    const BASE_DIFFUSION_RATE: f32 = 0.99;

    if traits.0.is_empty() {
        return;
    }

    // 1. Calculate aggregate modifiers
    let mut speed_mod = 1.0;
    let mut day_mod = 1.0;
    let mut power_mod = 1.0;
    let mut diffusion_mod = 1.0;

    for trait_ in &traits.0 {
        speed_mod *= trait_.speed_modifier();
        day_mod *= trait_.day_length_modifier();
        power_mod *= trait_.power_output_modifier();
        diffusion_mod *= trait_.diffusion_modifier();
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

    // 5. Apply to AtmosphereGrid
    if let Some(mut grid) = atmosphere {
        grid.diffusion_rate = (BASE_DIFFUSION_RATE * diffusion_mod).clamp(0.0, 1.0);
    }
}
