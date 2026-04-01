use crate::layer1::edicts::{ColonyPolicies, Policy};
use crate::layer1::health::Dead;
use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;
use crate::layer1::traits::{Trait, Traits};
use bevy::prelude::DespawnRecursiveExt;
use bevy_ecs::prelude::*;

#[derive(Event, Debug)]
pub struct OrganHarvestedEvent;

pub fn process_dead_pops_for_organs(
    mut commands: Commands,
    policies: Res<ColonyPolicies>,
    mut resources: ResMut<ColonyResources>,
    dead_pops: Query<Entity, (With<Dead>, With<Pop>)>,
    mut harvest_events: EventWriter<OrganHarvestedEvent>,
) {
    if !policies.is_active(Policy::MandatoryOrganHarvesting) {
        return;
    }

    for entity in dead_pops.iter() {
        resources.add_vital_organs(1.0);
        commands.entity(entity).despawn_recursive();
        harvest_events.send(OrganHarvestedEvent);
    }
}

pub fn apply_harvesting_horror_system(
    mut events: EventReader<OrganHarvestedEvent>,
    mut pops: Query<(&mut Morale, Option<&Traits>), With<Pop>>,
) {
    for _ in events.read() {
        for (mut morale, traits) in pops.iter_mut() {
            let is_psycho = traits.is_some_and(|t| t.has(Trait::Psychopath));

            if !is_psycho {
                morale.modifiers.push(MoodModifier {
                    label: "Organ Harvesting Horror".to_string(),
                    value: -0.2, // Heavy flat penalty
                    duration: 200,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organ_harvesting_edict_produces_organs() {
        let mut world = World::new();

        // Arrange
        let mut policies = ColonyPolicies::default();
        policies
            .active_policies
            .insert(Policy::MandatoryOrganHarvesting);
        world.insert_resource(policies);

        let resources = ColonyResources { vital_organs: 0.0, ..Default::default() };
        world.insert_resource(resources);

        world.init_resource::<Events<OrganHarvestedEvent>>();

        let dead_pop = world.spawn((Pop, Dead)).id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(process_dead_pops_for_organs);
        schedule.run(&mut world);

        // Assert
        let current_resources = world.resource::<ColonyResources>();
        assert_eq!(current_resources.vital_organs, 1.0);
        assert!(world.get_entity(dead_pop).is_err()); // Corpse should be consumed

        let events = world.resource::<Events<OrganHarvestedEvent>>();
        assert!(!events.is_empty(), "Expected OrganHarvestedEvent");
    }

    #[test]
    fn test_organ_harvesting_causes_horror() {
        let mut world = World::new();

        world.init_resource::<Events<OrganHarvestedEvent>>();

        let normal_pop = world
            .spawn((
                Pop,
                Morale {
                    value: 1.0,
                    modifiers: vec![],
                },
            ))
            .id();

        let mut psycho_traits = Traits::default();
        psycho_traits.add(Trait::Psychopath);

        let psycho_pop = world
            .spawn((
                Pop,
                psycho_traits,
                Morale {
                    value: 1.0,
                    modifiers: vec![],
                },
            ))
            .id();

        // Trigger harvest
        world
            .resource_mut::<Events<OrganHarvestedEvent>>()
            .send(OrganHarvestedEvent);

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_harvesting_horror_system);
        schedule.run(&mut world);

        let normal_morale = world.get::<Morale>(normal_pop).unwrap();
        let psycho_morale = world.get::<Morale>(psycho_pop).unwrap();

        assert!(
            normal_morale.modifiers.iter().any(|m| m.value == -0.2),
            "Normal pop should receive horror modifier"
        );
        assert!(
            psycho_morale.modifiers.is_empty(),
            "Psychopath should be unaffected"
        );
    }
}
