use crate::layer1::balance::TICKS_PER_YEAR;
use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;

/// The number of years after which new arrivals are considered Immigrants instead of Founders.
pub const FOUNDER_CUTOFF_YEAR: u64 = 5;

/// The generation of a Pop based on their arrival time.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Component)]
pub enum Generation {
    /// Arrived early in the colony's history.
    Founder,
    /// Arrived later.
    Immigrant,
}

/// Component tracking when a Pop arrived at the colony.
#[derive(Component)]
pub struct Arrival {
    /// The simulation tick when the Pop arrived.
    pub tick: u64,
}

impl Arrival {
    /// Determines the generation of the Pop based on arrival tick.
    #[must_use]
    pub const fn generation(&self) -> Generation {
        if self.tick < FOUNDER_CUTOFF_YEAR * TICKS_PER_YEAR {
            Generation::Founder
        } else {
            Generation::Immigrant
        }
    }
}

/// System to apply the effects of mood modifiers to Pop needs.
///
/// Modifiers act as a regeneration or decay acceleration on Leisure.
/// Scale: 1.0 modifier value ~= 0.0005 leisure change per tick.
/// A +5.0 buff provides +0.0025/tick, overcoming natural decay (0.0015).
pub fn apply_mood_modifiers_system(mut query: Query<(&MoodModifiers, &mut Needs)>) {
    const MODIFIER_SCALE: f32 = 0.0005;

    for (modifiers, mut needs) in &mut query {
        let mut total_change = 0.0;
        for entry in &modifiers.entries {
            total_change += entry.value;
        }

        if total_change.abs() > f32::EPSILON {
            let change = total_change * MODIFIER_SCALE;
            needs.leisure = (needs.leisure + change).clamp(0.0, 1.0);
        }
    }
}

/// Marker component for Founders receiving the legacy buff.
#[derive(Component)]
pub struct FounderBuff;

/// A single mood modifier entry.
#[derive(Debug, Clone, PartialEq)]
pub struct MoodModifierEntry {
    /// The value to add to morale (e.g., +5.0 or -5.0).
    pub value: f32,
    /// The description of the source of this modifier.
    pub source: String,
    /// How long this modifier lasts in ticks (or `f32::MAX` for permanent).
    pub duration: f32,
}

/// Component holding a list of active mood modifiers.
#[derive(Component, Default)]
pub struct MoodModifiers {
    /// The list of active modifiers.
    pub entries: Vec<MoodModifierEntry>,
}

/// Resource tracking population demographics.
#[derive(Resource, Default)]
pub struct Demographics {
    /// Number of founders.
    pub founders: usize,
    /// Number of immigrants.
    pub immigrants: usize,
    /// Whether the "Turning Point" event has triggered.
    pub has_triggered_turning_point: bool,
}

/// System to assign Generation and apply initial Founder benefits.
///
/// Runs only for Pops that have an Arrival component but no Generation yet.
#[allow(clippy::type_complexity)]
pub fn apply_founder_benefits_system(
    mut commands: Commands,
    mut query: Query<
        (Entity, &Arrival, Option<&mut MoodModifiers>),
        (With<Pop>, Without<Generation>),
    >,
) {
    for (entity, arrival, mut modifiers_opt) in &mut query {
        let generation = arrival.generation();
        commands.entity(entity).insert(generation);

        if generation == Generation::Founder {
            commands.entity(entity).insert(FounderBuff);

            let entry = MoodModifierEntry {
                value: 5.0,
                source: "Legacy of the First".to_string(),
                duration: f32::MAX,
            };

            if let Some(ref mut mods) = modifiers_opt {
                mods.entries.push(entry);
            } else {
                commands.entity(entity).insert(MoodModifiers {
                    entries: vec![entry],
                });
            }
        }
    }
}

/// System to check for social friction between generations.
///
/// Applies negative mood modifiers if one group feels overwhelmed or excluded.
#[allow(clippy::type_complexity)]
pub fn check_generational_friction_system(
    mut commands: Commands,
    count_query: Query<&Generation, With<Pop>>,
    mut pop_query: Query<(Entity, &Generation, Option<&mut MoodModifiers>), With<Pop>>,
    mut demographics: ResMut<Demographics>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    let mut founders = 0;
    let mut immigrants = 0;

    for generation in count_query.iter() {
        match generation {
            Generation::Founder => founders += 1,
            Generation::Immigrant => immigrants += 1,
        }
    }

    // Update resource
    demographics.founders = founders;
    demographics.immigrants = immigrants;

    // Check for Turning Point
    if !demographics.has_triggered_turning_point && founders > 0 && immigrants > founders {
        demographics.has_triggered_turning_point = true;
        chronicle_events.send(AddChronicleEvent {
            text: "The Turning Point. For the first time, new arrivals outnumber the founders."
                .to_string(),
            importance: EventImportance::Major,
        });
    }

    if founders == 0 && immigrants == 0 {
        return;
    }

    let overwhelmed = founders > 0 && immigrants > founders * 2;
    let excluded = immigrants > 0 && founders > immigrants;

    for (entity, generation, mut modifiers_opt) in &mut pop_query {
        let (value, source) = match generation {
            Generation::Founder if overwhelmed => (-5.0, "Overwhelmed by Strangers"),
            Generation::Immigrant if excluded => (-2.0, "Excluded by Clique"),
            _ => continue,
        };

        let duration = 10.0;

        // Helper to check if source already exists
        let mut updated = false;
        if let Some(ref mut mods) = modifiers_opt {
            for entry in &mut mods.entries {
                if entry.source == source {
                    entry.duration = duration; // Refresh duration
                    updated = true;
                    break;
                }
            }
            if !updated {
                mods.entries.push(MoodModifierEntry {
                    value,
                    source: source.to_string(),
                    duration,
                });
            }
        } else {
            commands.entity(entity).insert(MoodModifiers {
                entries: vec![MoodModifierEntry {
                    value,
                    source: source.to_string(),
                    duration,
                }],
            });
        }
    }
}

/// System to decrement mood modifier durations and remove expired ones.
pub fn mood_lifecycle_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MoodModifiers)>,
) {
    for (entity, mut modifiers) in &mut query {
        modifiers.entries.retain_mut(|entry| {
            if (entry.duration - f32::MAX).abs() > f32::EPSILON {
                entry.duration -= 1.0;
            }
            entry.duration > 0.0
        });

        if modifiers.entries.is_empty() {
            commands.entity(entity).remove::<MoodModifiers>();
        }
    }
}
