use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;
use crate::layer1::social::morale::{MoodModifier, Morale};
use crate::layer1::GridPosition;
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct XenofloraPet {
    pub resource_upkeep: f32, // Food consumed per tick
    pub mood_boost: f32,
    pub is_starving: bool,
}

impl Default for XenofloraPet {
    fn default() -> Self {
        Self {
            resource_upkeep: 0.1,
            mood_boost: 0.15,
            is_starving: false,
        }
    }
}

/// Applies mood boosts for having a pet, and debuffs if it's starving.
pub fn apply_pet_mood_boost(mut query: Query<(&mut Morale, &XenofloraPet)>) {
    for (mut morale, pet) in query.iter_mut() {
        if pet.is_starving {
            morale.add_modifier(MoodModifier {
                label: "Starving Pet".to_string(),
                value: -0.3, // Drastic mood drop
                duration: 1,
            });
        } else {
            morale.add_modifier(MoodModifier {
                label: "Xenoflora Pet".to_string(),
                value: pet.mood_boost,
                duration: 1,
            });
        }
    }
}

/// Consumes resources for pet upkeep. If resources are missing, the pet starves.
pub fn pet_resource_consumption_system(
    mut resources: ResMut<ColonyResources>,
    mut query: Query<&mut XenofloraPet>,
) {
    for mut pet in query.iter_mut() {
        let cost = ColonyResources {
            food: pet.resource_upkeep,
            ..Default::default()
        };

        pet.is_starving = !resources.try_deduct(&cost);
    }
}

/// Viral spread: pops near each other have a chance to adopt a pet.
pub fn pet_viral_spread_system(
    mut commands: Commands,
    infected_pops: Query<&GridPosition, (With<Pop>, With<XenofloraPet>)>,
    uninfected_pops: Query<(Entity, &GridPosition), (With<Pop>, Without<XenofloraPet>)>,
) {
    let mut rng = rand::thread_rng();

    // Find all pops with pets
    let mut infected_positions = Vec::new();
    for pos in infected_pops.iter() {
        infected_positions.push(*pos);
    }

    if infected_positions.is_empty() {
        return;
    }

    // Spread to pops without pets
    for (entity, pos) in uninfected_pops.iter() {
        // Check if near any infected pop (Manhattan distance <= 2)
        let is_near = infected_positions.iter().any(|p| {
            let dx = p.x.abs_diff(pos.x);
            let dy = p.y.abs_diff(pos.y);
            (dx + dy) <= 2
        });

        if is_near && rng.gen_bool(0.05) {
            // 5% chance per tick to adopt if near
            commands.entity(entity).insert(XenofloraPet::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pet_creation() {
        let mut world = World::new();

        let pop_entity = world
            .spawn((
                Pop,
                Morale::default(),
                XenofloraPet {
                    resource_upkeep: 1.0,
                    mood_boost: 0.25,
                    is_starving: false,
                },
            ))
            .id();

        let pet = world.get::<XenofloraPet>(pop_entity).unwrap();
        assert_eq!(pet.resource_upkeep, 1.0);
        assert_eq!(pet.mood_boost, 0.25);
    }

    #[test]
    fn test_pet_mood_boost() {
        let mut world = World::new();

        let pop_entity = world
            .spawn((
                Pop,
                Morale::default(),
                XenofloraPet {
                    resource_upkeep: 1.0,
                    mood_boost: 0.25,
                    is_starving: false,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_pet_mood_boost);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop_entity).unwrap();
        assert_eq!(morale.modifiers.len(), 1);
        assert_eq!(morale.modifiers[0].value, 0.25);
        assert_eq!(morale.modifiers[0].label, "Xenoflora Pet");
    }

    #[test]
    fn test_pet_starving_mood_drop() {
        let mut world = World::new();

        let pop_entity = world
            .spawn((
                Pop,
                Morale::default(),
                XenofloraPet {
                    resource_upkeep: 1.0,
                    mood_boost: 0.25,
                    is_starving: true,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_pet_mood_boost);
        schedule.run(&mut world);

        let morale = world.get::<Morale>(pop_entity).unwrap();
        assert_eq!(morale.modifiers.len(), 1);
        assert_eq!(morale.modifiers[0].value, -0.3);
        assert_eq!(morale.modifiers[0].label, "Starving Pet");
    }

    #[test]
    fn test_pet_resource_consumption() {
        let mut world = World::new();

        world.insert_resource(ColonyResources {
            food: 10.0,
            ..Default::default()
        });

        let pop_entity = world
            .spawn((
                Pop,
                XenofloraPet {
                    resource_upkeep: 2.0,
                    mood_boost: 0.15,
                    is_starving: true, // Should become false
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(pet_resource_consumption_system);
        schedule.run(&mut world);

        let pet = world.get::<XenofloraPet>(pop_entity).unwrap();
        assert!(!pet.is_starving);

        let resources = world.get_resource::<ColonyResources>().unwrap();
        assert_eq!(resources.food, 8.0);
    }

    #[test]
    fn test_pet_starvation_no_resources() {
        let mut world = World::new();

        world.insert_resource(ColonyResources {
            food: 1.0, // Not enough
            ..Default::default()
        });

        let pop_entity = world
            .spawn((
                Pop,
                XenofloraPet {
                    resource_upkeep: 2.0,
                    mood_boost: 0.15,
                    is_starving: false, // Should become true
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(pet_resource_consumption_system);
        schedule.run(&mut world);

        let pet = world.get::<XenofloraPet>(pop_entity).unwrap();
        assert!(pet.is_starving);

        let resources = world.get_resource::<ColonyResources>().unwrap();
        assert_eq!(resources.food, 1.0); // Not deducted
    }

    #[test]
    fn test_viral_spread() {
        let mut world = World::new();

        // Setup two pops close to each other, one with pet, one without
        world.spawn((Pop, GridPosition { x: 5, y: 5 }, XenofloraPet::default()));
        let uninfected = world.spawn((Pop, GridPosition { x: 6, y: 5 })).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(pet_viral_spread_system);

        // Run several times to increase probability of infection
        for _ in 0..1000 {
            schedule.run(&mut world);
            // Needs to apply deferred commands since Commands is used
            // but in tests, calling world.clear_trackers() or just another empty schedule is not enough for Commands.
            // Oh right, schedule.run will apply commands.
        }

        // Let's verify it gets added. Because of RNG, it's not 100% guaranteed in 100 ticks but highly likely (1 - (0.95)^100 ≈ 99.4%)
        let pet = world.get::<XenofloraPet>(uninfected);
        assert!(pet.is_some());
    }
}
