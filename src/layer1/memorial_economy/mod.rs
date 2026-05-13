use bevy_ecs::prelude::*;
use crate::layer1::entities::pop::PopDied;
use crate::layer1::social::social_stratification::Prestige;
use crate::layer1::social::morale::{MoodModifier, Morale};

#[derive(Component)]
pub struct Relic;

#[derive(Component)]
pub struct MemorialStructure {
    pub dedicated_to: Entity,
}

#[derive(Component)]
pub struct DescendantOf(pub Entity);

pub fn process_pop_deaths_for_relics(
    mut commands: Commands,
    mut events: EventReader<PopDied>,
    query: Query<&Prestige>,
) {
    for event in events.read() {
        if let Ok(prestige) = query.get(event.entity) {
            if prestige.value >= 10 {
                commands.spawn(Relic);
            }
        }
    }
}

pub fn apply_memorial_morale_boost(
    memorials: Query<&MemorialStructure>,
    mut descendants: Query<(&mut Morale, &DescendantOf)>,
) {
    for memorial in memorials.iter() {
        for (mut morale, descendant_of) in descendants.iter_mut() {
            if descendant_of.0 == memorial.dedicated_to {
                morale.add_modifier(MoodModifier {
                    label: "Ancestral Memorial".to_string(),
                    value: 0.1, // Equivalent to morale boost
                    duration: 1, // transient duration, maintained by structure presence
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::prelude::*;
    use crate::layer1::entities::pop::Pop;

    #[test]
    fn test_high_prestige_death_generates_relic() {
        let mut app = App::new();
        app.add_event::<PopDied>();
        app.init_resource::<Events<PopDied>>();
        app.add_systems(Update, process_pop_deaths_for_relics);

        let pop = app.world_mut().spawn((
            Pop,
            Prestige { value: 10 },
        )).id();

        app.world_mut().resource_mut::<Events<PopDied>>().send(PopDied {
            entity: pop,
            name: "Hero".to_string(),
            tick: 0,
            reason: "Old Age".to_string(),
        });

        app.update();

        // Assert a Relic was generated
        let relics = app.world_mut().query::<&Relic>().iter(app.world()).count();
        assert_eq!(relics, 1);
    }

    #[test]
    fn test_memorial_structure_boosts_descendant_morale() {
        let mut app = App::new();
        app.add_systems(Update, apply_memorial_morale_boost);

        let ancestor_id = Entity::from_raw(1); // Fake ID for testing
        let _memorial = app.world_mut().spawn((
            MemorialStructure { dedicated_to: ancestor_id },
        )).id();

        let descendant = app.world_mut().spawn((
            Pop,
            Morale::default(),
            DescendantOf(ancestor_id),
        )).id();

        app.update();

        // Assert morale was boosted
        let morale = app.world_mut().get::<Morale>(descendant).unwrap();
        assert!(morale.modifiers.iter().any(|m| m.label == "Ancestral Memorial"));
    }
}
