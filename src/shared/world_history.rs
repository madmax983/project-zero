//! Pre-history generation for the world.
//!
//! Generates a DF-style "history of the universe" at startup, populating the
//! chronicle with events that occurred before the colony was founded.

use bevy_ecs::prelude::*;
use rand::Rng;

use crate::layer1::chronicle::{Chronicle, EventImportance};
use crate::shared::narrative::{NarrativeContext, NarrativeGenerator};

/// Generate pre-history events and insert them into the chronicle.
///
/// Uses a two-phase approach to avoid simultaneous borrows:
/// 1. Borrow `&NarrativeGenerator` to generate all text
/// 2. Borrow `&mut Chronicle` to insert events
pub fn generate_world_history(world: &mut World) {
    // Phase 1: generate all event text
    let generator = world.resource::<NarrativeGenerator>();
    let events = generate_history_events(generator);

    // Phase 2: insert into chronicle
    let mut chronicle = world.resource_mut::<Chronicle>();
    for (text, importance) in events {
        chronicle.add_prehistory_event(text, importance);
    }
}

/// Generate a set of pre-history chronicle entries.
///
/// Returns `(text, importance)` pairs for insertion into the chronicle.
fn generate_history_events(generator: &NarrativeGenerator) -> Vec<(String, EventImportance)> {
    let mut rng = rand::thread_rng();
    let mut events = Vec::new();

    let civ_names = generate_civilizations(generator, &mut rng, &mut events);
    generate_wars(generator, &mut rng, &civ_names, &mut events);
    generate_catastrophe(generator, &mut rng, &mut events);
    generate_era_transitions(generator, &mut rng, &mut events);
    generate_artifacts(generator, &mut rng, &civ_names, &mut events);

    events
}

fn generate_civilizations(
    generator: &NarrativeGenerator,
    rng: &mut impl Rng,
    events: &mut Vec<(String, EventImportance)>,
) -> Vec<String> {
    let mut civ_names = Vec::new();
    let num_civs = rng.gen_range(3..=5);

    for i in 0..num_civs {
        let civ_name = generator.generate_civ_name();
        let star_name = generator.generate_star_name();
        let year = rng.gen_range(1000..=8000);

        let mut ctx = NarrativeContext::new();
        ctx.insert("CIV_NAME", &civ_name);
        ctx.insert("ORIGIN_STAR", &star_name);
        ctx.insert("YEAR", &year.to_string());

        let text = generator
            .generate("CIVILIZATION_RISE", &ctx)
            .unwrap_or_else(|_| format!("Year {year}. The {civ_name} arise from {star_name}."));

        events.push((text, EventImportance::Major));
        civ_names.push(civ_name.clone());

        // ~60% chance of falling
        if rng.gen_bool(0.6) && i < num_civs - 1 {
            let fall_year = year + rng.gen_range(200..=3000);
            let mut fall_ctx = NarrativeContext::new();
            fall_ctx.insert("CIV_NAME", &civ_name);
            fall_ctx.insert("YEAR", &fall_year.to_string());

            let fall_text = generator
                .generate("CIVILIZATION_FALL", &fall_ctx)
                .unwrap_or_else(|_| format!("Year {fall_year}. The {civ_name} fall silent."));

            events.push((fall_text, EventImportance::Standard));
        }
    }

    civ_names
}

fn generate_wars(
    generator: &NarrativeGenerator,
    rng: &mut impl Rng,
    civ_names: &[String],
    events: &mut Vec<(String, EventImportance)>,
) {
    if civ_names.len() < 2 {
        return;
    }

    let num_wars = rng.gen_range(1..=2);
    for _ in 0..num_wars {
        let a = rng.gen_range(0..civ_names.len());
        let mut b = rng.gen_range(0..civ_names.len());
        while b == a {
            b = rng.gen_range(0..civ_names.len());
        }

        let start_year = rng.gen_range(3000..=9000);
        let end_year = start_year + rng.gen_range(10..=500);

        let mut ctx = NarrativeContext::new();
        ctx.insert("CIV_A", &civ_names[a]);
        ctx.insert("CIV_B", &civ_names[b]);
        ctx.insert("START_YEAR", &start_year.to_string());
        ctx.insert("END_YEAR", &end_year.to_string());

        let war_name = format!("the {}-{} War", &civ_names[a], &civ_names[b]);
        ctx.insert("WAR_NAME", &war_name);

        let causes = ["Succession", "Resources", "Territory", "Fear", "Pride", "Survival"];
        let cause = causes[rng.gen_range(0..causes.len())];
        ctx.insert("CAUSE", cause);

        let outcomes = [
            "Mutual exhaustion",
            "Pyrrhic victory",
            "Unconditional surrender",
            "Stalemate",
            "Both sides claimed victory",
            "The records disagree",
        ];
        let outcome = outcomes[rng.gen_range(0..outcomes.len())];
        ctx.insert("OUTCOME", outcome);

        let text = generator
            .generate("WAR_RECORD", &ctx)
            .unwrap_or_else(|_| {
                format!(
                    "{war_name} ({start_year}-{end_year}). {} vs {}. {cause}. {outcome}.",
                    &civ_names[a], &civ_names[b]
                )
            });

        events.push((text, EventImportance::Major));
    }
}

fn generate_catastrophe(
    generator: &NarrativeGenerator,
    rng: &mut impl Rng,
    events: &mut Vec<(String, EventImportance)>,
) {
    let year = rng.gen_range(5000..=9500);
    let mut ctx = NarrativeContext::new();
    ctx.insert("YEAR", &year.to_string());

    let regions = [
        "the Reaching",
        "the Old Marches",
        "the Kindred Space",
        "the Core",
        "the Rim",
    ];
    let region = regions[rng.gen_range(0..regions.len())];
    ctx.insert("AFFECTED_REGION", region);

    let consequences = [
        "Trade routes collapse",
        "Three civilizations fall silent",
        "The maps are redrawn",
        "Survivors scatter to the dark",
        "Communication breaks down for centuries",
    ];
    let consequence = consequences[rng.gen_range(0..consequences.len())];
    ctx.insert("CONSEQUENCE", consequence);

    let text = generator
        .generate("CATASTROPHE", &ctx)
        .unwrap_or_else(|_| format!("Year {year}. Catastrophe strikes {region}. {consequence}."));

    events.push((text, EventImportance::Legendary));
}

fn generate_era_transitions(
    generator: &NarrativeGenerator,
    rng: &mut impl Rng,
    events: &mut Vec<(String, EventImportance)>,
) {
    let eras = [
        "the Silence",
        "the Expansion",
        "the Long Peace",
        "the Burning",
        "the Scattering",
        "the Forgetting",
        "the Golden Age",
        "the Interregnum",
    ];

    let num_eras = rng.gen_range(2..=3);
    for _ in 0..num_eras {
        let year = rng.gen_range(2000..=9800);
        let mut ctx = NarrativeContext::new();
        ctx.insert("YEAR", &year.to_string());

        let old_idx = rng.gen_range(0..eras.len());
        let mut new_idx = rng.gen_range(0..eras.len());
        while new_idx == old_idx {
            new_idx = rng.gen_range(0..eras.len());
        }
        ctx.insert("OLD_ERA", eras[old_idx]);
        ctx.insert("NEW_ERA", eras[new_idx]);

        let causes = [
            "a war that broke boundaries",
            "the death of the last Kindred",
            "a discovery that changed everything",
            "the collapse of trade",
            "a signal from the deep",
        ];
        let cause = causes[rng.gen_range(0..causes.len())];
        ctx.insert("CAUSE", cause);

        let text = generator.generate("ERA_TRANSITION", &ctx).unwrap_or_else(|_| {
            format!("Year {year}. {} ends. {} begins.", eras[old_idx], eras[new_idx])
        });

        events.push((text, EventImportance::Standard));
    }
}

fn generate_artifacts(
    generator: &NarrativeGenerator,
    rng: &mut impl Rng,
    civ_names: &[String],
    events: &mut Vec<(String, EventImportance)>,
) {
    let names = [
        "the Shard of Knowing",
        "the Silence Engine",
        "the First Map",
        "the Burning Codex",
        "the Void Key",
        "the Last Record",
    ];

    let num_artifacts = rng.gen_range(1..=2);
    for _ in 0..num_artifacts {
        let year = rng.gen_range(2000..=9000);
        let mut ctx = NarrativeContext::new();
        ctx.insert("YEAR", &year.to_string());

        let name = names[rng.gen_range(0..names.len())];
        ctx.insert("ARTIFACT_NAME", name);

        if !civ_names.is_empty() {
            let creator = &civ_names[rng.gen_range(0..civ_names.len())];
            ctx.insert("CREATOR_CIV", creator);
        }

        let text = generator
            .generate("ARTIFACT_CREATION", &ctx)
            .unwrap_or_else(|_| format!("Year {year}. {name} is created."));

        events.push((text, EventImportance::Standard));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_history_events_produces_events() {
        let narrator = NarrativeGenerator::from_embedded();
        let events = generate_history_events(&narrator);
        assert!(
            events.len() >= 8,
            "Should generate at least 8 events, got {}",
            events.len()
        );
        assert!(
            events.len() <= 20,
            "Should generate at most 20 events, got {}",
            events.len()
        );
    }

    #[test]
    fn test_generate_history_events_has_legendary() {
        let narrator = NarrativeGenerator::from_embedded();
        let events = generate_history_events(&narrator);
        let legendary_count = events
            .iter()
            .filter(|(_, imp)| *imp == EventImportance::Legendary)
            .count();
        assert!(
            legendary_count >= 1,
            "Should have at least one legendary event"
        );
    }

    #[test]
    fn test_generate_history_events_text_not_empty() {
        let narrator = NarrativeGenerator::from_embedded();
        let events = generate_history_events(&narrator);
        for (text, _) in &events {
            assert!(!text.is_empty(), "Event text should not be empty");
        }
    }

    #[test]
    fn test_generate_world_history_inserts_into_chronicle() {
        let mut world = World::new();
        world.insert_resource(NarrativeGenerator::from_embedded());
        world.insert_resource(Chronicle::default());

        generate_world_history(&mut world);

        let chronicle = world.resource::<Chronicle>();
        assert!(
            !chronicle.events.is_empty(),
            "Chronicle should have pre-history events"
        );
        // All pre-history events should have year=0
        for evt in &chronicle.events {
            assert_eq!(evt.year, 0, "Pre-history events should have year=0");
            assert_eq!(evt.tick, 0, "Pre-history events should have tick=0");
        }
    }
}
