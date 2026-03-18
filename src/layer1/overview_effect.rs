use bevy_ecs::prelude::*;

use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::traits::Trait;

#[derive(Event)]
pub struct ObserveEvent {
    pub pop: Entity,
}

#[derive(Resource, Default)]
pub struct Layer2State {
    pub hostile_fleets_in_orbit: bool,
}

pub fn overview_effect_system(
    mut events: EventReader<ObserveEvent>,
    mut query: Query<(&mut Skills, &crate::layer1::traits::Traits, &mut Morale)>,
    l2_state: Option<Res<Layer2State>>,
) {
    let hostile_orbit = l2_state.map(|s| s.hostile_fleets_in_orbit).unwrap_or(false);

    for ev in events.read() {
        if let Ok((mut skills, traits, mut morale)) = query.get_mut(ev.pop) {
            skills.add_xp(SkillType::Crafting, 10.0);

            if traits.has(Trait::Anxious) || hostile_orbit {
                morale.modifiers.push(MoodModifier {
                    label: "Existential Dread".to_string(),
                    value: -0.2,
                    duration: 1000,
                });
            } else if traits.has(Trait::Optimist) {
                morale.modifiers.push(MoodModifier {
                    label: "Inspired".to_string(),
                    value: 0.2,
                    duration: 1000,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer1::morale::Morale;
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::{SkillType, Skills};
    use crate::layer1::traits::{Trait, Traits};

    fn setup_app() -> World {
        let mut world = World::new();
        world.init_resource::<Events<ObserveEvent>>();
        world
    }

    #[test]
    fn test_observatory_grants_knowledge() {
        let mut world = setup_app();

        let pop_entity = world
            .spawn((
                Pop,
                Skills::default(),
                Traits::default(),
                Morale::default(),
            ))
            .id();

        world.send_event(ObserveEvent { pop: pop_entity });

        let mut schedule = Schedule::default();
        schedule.add_systems(overview_effect_system);
        schedule.run(&mut world);

        let skills = world.get::<Skills>(pop_entity).unwrap();
        assert!(
            skills.get_xp(SkillType::Crafting) > 0.0,
            "Observing must grant XP"
        );
    }

    #[test]
    fn test_existential_dread_trait_reaction() {
        let mut world = setup_app();

        let mut traits = Traits::default();
        traits.add(Trait::Anxious);

        let pop_entity = world
            .spawn((Pop, Skills::default(), traits, Morale::default()))
            .id();

        world.send_event(ObserveEvent { pop: pop_entity });

        let mut schedule = Schedule::default();
        schedule.add_systems(overview_effect_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop_entity).unwrap();
        assert!(
            morale
                .modifiers
                .iter()
                .any(|m| m.label == "Existential Dread"),
            "Anxious pop should feel Dread when looking at the stars"
        );
    }

    #[test]
    fn test_orbital_fleet_reaction() {
        let mut world = setup_app();

        world.insert_resource(Layer2State {
            hostile_fleets_in_orbit: true,
        });

        let pop_entity = world
            .spawn((
                Pop,
                Skills::default(),
                Traits::default(),
                Morale::default(),
            ))
            .id();

        world.send_event(ObserveEvent { pop: pop_entity });

        let mut schedule = Schedule::default();
        schedule.add_systems(overview_effect_system);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop_entity).unwrap();
        assert!(
            morale
                .modifiers
                .iter()
                .any(|m| m.label == "Existential Dread"),
            "Seeing a hostile fleet must cause Dread"
        );
    }
}
