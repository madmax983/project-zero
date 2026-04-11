import sys

with open('src/layer2/syzygy.rs', 'r') as f:
    content = f.read()

merge_diff = """<<<<<<< SEARCH
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct PlanetaryGravity {
=======
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct SyzygyActive {
    pub duration: u32,
}

#[derive(Component)]
pub struct MutatedTrait;

use crate::layer1::chronicle::{Chronicle, EventImportance};
use crate::layer2::alignment::PlanetaryAlignment;
use crate::layer1::entities::pop::Speed;
use crate::layer1::utility_types::ActionType;
use crate::layer1::utility_types::PopAction;
use rand::Rng;

pub fn trigger_syzygy(world: &mut World, entity: Entity) {
    if let Some(alignment) = world.get::<PlanetaryAlignment>(entity) {
        if alignment.days_until == 0 {
            world.insert_resource(SyzygyActive { duration: 30 });
            let mut chronicle = world.get_resource_mut::<Chronicle>().unwrap();
            chronicle.add_event(0, "syzygy_begins".to_string(), EventImportance::Standard);
        }
    }
}

pub fn apply_syzygy_effects(world: &mut World) {
    let mut duration = 0;
    if let Some(mut active) = world.get_resource_mut::<SyzygyActive>() {
        if active.duration > 0 {
            active.duration -= 1;
            duration = active.duration;
        }
    }

    if duration == 0 {
        world.remove_resource::<SyzygyActive>();
    }

    if world.contains_resource::<SyzygyActive>() {
        for (action, mut speed) in world.query::<(&PopAction, &mut Speed)>().iter_mut(world) {
            if action.current == ActionType::Haul {
                speed.current = speed.base + 5.0; // Boost hauling speed
            } else {
                speed.current = speed.base;
            }
        }
    } else {
        // Reset speeds
        for mut speed in world.query::<&mut Speed>().iter_mut(world) {
            speed.current = speed.base;
        }
    }
}

pub fn trigger_random_mutation(world: &mut World, entity: Entity) {
    if world.contains_resource::<SyzygyActive>() {
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.1) { // 10% chance
            world.entity_mut(entity).insert(MutatedTrait);
        }
    }
}

#[derive(Resource)]
pub struct PlanetaryGravity {
>>>>>>> REPLACE"""

content = content.replace("""use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct PlanetaryGravity {""", """use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct SyzygyActive {
    pub duration: u32,
}

#[derive(Component)]
pub struct MutatedTrait;

use crate::layer1::chronicle::{Chronicle, EventImportance};
use crate::layer2::alignment::PlanetaryAlignment;
use crate::layer1::entities::pop::Speed;
use crate::layer1::utility_types::ActionType;
use crate::layer1::utility_types::PopAction;
use rand::Rng;

pub fn trigger_syzygy(world: &mut World) {
    let entities: Vec<Entity> = world.query_filtered::<Entity, With<PlanetaryAlignment>>().iter(world).collect();
    for entity in entities {
        if let Some(alignment) = world.get::<PlanetaryAlignment>(entity) {
            if alignment.days_until == 0 {
                world.insert_resource(SyzygyActive { duration: 30 });
                let mut chronicle = world.get_resource_mut::<Chronicle>().unwrap();
                chronicle.add_event(0, "syzygy_begins".to_string(), EventImportance::Standard);
            }
        }
    }
}

pub fn apply_syzygy_effects(world: &mut World) {
    let mut duration = 0;
    if let Some(mut active) = world.get_resource_mut::<SyzygyActive>() {
        if active.duration > 0 {
            active.duration -= 1;
            duration = active.duration;
        }
    }

    if duration == 0 {
        world.remove_resource::<SyzygyActive>();
    }

    if world.contains_resource::<SyzygyActive>() {
        for (action, mut speed) in world.query::<(&PopAction, &mut Speed)>().iter_mut(world) {
            if action.current == ActionType::Haul {
                speed.current = speed.base + 5.0; // Boost hauling speed
            } else {
                speed.current = speed.base;
            }
        }
    } else {
        // Reset speeds
        for mut speed in world.query::<&mut Speed>().iter_mut(world) {
            speed.current = speed.base;
        }
    }
}

pub fn trigger_random_mutation(world: &mut World) {
    let entities: Vec<Entity> = world.query_filtered::<Entity, With<crate::layer1::pop::Pop>>().iter(world).collect();
    for entity in entities {
        if world.contains_resource::<SyzygyActive>() {
            let mut rng = rand::thread_rng();
            if rng.gen_bool(0.1) { // 10% chance
                world.entity_mut(entity).insert(MutatedTrait);
            }
        }
    }
}

#[derive(Resource)]
pub struct PlanetaryGravity {""")

with open('src/layer2/syzygy.rs', 'w') as f:
    f.write(content)
