//! Leader Ascension mechanics.

use bevy_ecs::prelude::*;
use crate::layer1::pop::{Pop, PopName};
use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::skills::Skills;
use crate::layer1::psychology::traits::Traits;

/// Marks a pop as eligible to become a leader.
#[derive(Component)]
pub struct AscensionCandidate;

/// Event that promotes a pop into a Leader.
#[derive(Event)]
pub struct PromotePopEvent {
    /// The pop being promoted.
    pub pop_entity: Entity,
    /// The new role assigned to the pop.
    pub new_role: LeaderRole,
}

/// Component for a promoted leader entity.
#[derive(Component)]
pub struct Leader;

/// Determines what role a promoted leader takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaderRole {
    /// Military leader role.
    Admiral,
    /// Administrative leader role.
    Governor,
}

/// Identifies highly skilled pops and marks them as ascension candidates.
#[allow(clippy::type_complexity)]
pub fn check_ascension_eligibility_system(
    mut commands: Commands,
    query: Query<(Entity, &Skills), (With<Pop>, Without<AscensionCandidate>)>,
) {
    for (entity, skills) in query.iter() {
        // Assume level 10 is max skill level for ascension eligibility
        let is_eligible = skills.xp.values().any(|&xp| xp >= 10000.0); // 10000 XP corresponds to level 10 according to Skills::get_level formula
        if is_eligible {
            commands.entity(entity).insert(AscensionCandidate);
        }
    }
}

/// Processes `PromotePopEvent` events to transform pops into leaders.
pub fn promote_pop_system(
    mut commands: Commands,
    mut events: EventReader<PromotePopEvent>,
    query: Query<(Option<&PopName>, Option<&Traits>), With<AscensionCandidate>>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for event in events.read() {
        if let Ok((name, traits)) = query.get(event.pop_entity) {
            let mut new_entity_commands = commands.spawn(Leader);

            let mut leader_name = "Unknown".to_string();

            if let Some(pop_name) = name {
                new_entity_commands.insert(PopName(pop_name.0.clone()));
                leader_name = pop_name.0.clone();
            }
            if let Some(pop_traits) = traits {
                new_entity_commands.insert(pop_traits.clone());
            }

            commands.entity(event.pop_entity).despawn();

            chronicle_events.send(AddChronicleEvent {
                text: format!("LEADER_PROMOTED: {} ascended to {:?}", leader_name, event.new_role),
                importance: EventImportance::Major,
            });
        }
    }
}

/// Plugin setting up leader ascension events and systems.
pub struct LeaderAscensionPlugin;

impl bevy_app::Plugin for LeaderAscensionPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_event::<PromotePopEvent>()
            .add_systems(
                bevy_app::Update,
                (
                    check_ascension_eligibility_system,
                    promote_pop_system,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::skills::SkillType;
    use crate::layer1::psychology::traits::Trait;

    fn setup_test_app() -> bevy_app::App {
        let mut app = bevy_app::App::new();
        app.add_event::<PromotePopEvent>();
        app.add_event::<AddChronicleEvent>();
        app.add_systems(bevy_app::Update, (check_ascension_eligibility_system, promote_pop_system));
        app
    }

    #[test]
    fn test_pop_eligible_for_ascension() {
        let mut app = setup_test_app();

        let mut skills = Skills::default();
        skills.add_xp(SkillType::Mining, 10000.0); // Level 10

        let pop = app.world_mut().spawn((Pop, skills)).id();

        app.update();

        assert!(app.world().get::<AscensionCandidate>(pop).is_some());
    }

    #[test]
    fn test_ascension_removes_from_l1_and_creates_leader() {
        let mut app = setup_test_app();

        let mut traits = Traits::default();
        traits.add(Trait::HardWorker);
        let pop = app.world_mut().spawn((Pop, PopName("Hero".into()), traits, AscensionCandidate)).id();

        app.world_mut().send_event(PromotePopEvent { pop_entity: pop, new_role: LeaderRole::Admiral });
        app.update();

        assert!(app.world().get::<Pop>(pop).is_none());

        let mut leader_query = app.world_mut().query::<(&Leader, &Traits)>();
        let mut count = 0;
        let mut has_trait = false;
        for (_, t) in leader_query.iter(app.world()) {
            count += 1;
            if t.has(Trait::HardWorker) {
                has_trait = true;
            }
        }
        assert_eq!(count, 1);
        assert!(has_trait);

        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut cursor = events.get_cursor();
        let mut chronicle_logged = false;
        for ev in cursor.read(events) {
            if ev.text.starts_with("LEADER_PROMOTED") {
                chronicle_logged = true;
            }
        }
        assert!(chronicle_logged);
    }
}
