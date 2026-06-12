#[cfg(feature = "nova")]
use crate::layer1::culture::ancestral_graves::SacrilegeEvent;
#[cfg(feature = "nova")]
use crate::layer1::graffiti::{Graffiti, GraffitiMap, GraffitiType};
#[cfg(feature = "nova")]
use bevy_ecs::prelude::*;

/// Listens for `SacrilegeEvent` and creates memetic sigils around the desecrated site.
#[cfg(feature = "nova")]
pub fn cursed_foundations_system(
    mut events: EventReader<SacrilegeEvent>,
    mut graffiti_map: ResMut<GraffitiMap>,
) {
    for event in events.read() {
        let pos = event.pos;

        // Curse the tile itself and all 8 adjacent tiles
        for dx in -1..=1 {
            for dy in -1..=1 {
                let target_x = pos.x + dx;
                let target_y = pos.y + dy;

                let sigil = Graffiti {
                    graffiti_type: GraffitiType::MemeticSigil,
                    decay: 1000.0,  // Long lasting curse
                    modifier: -0.2, // Severe impact
                };

                graffiti_map.markings.insert((target_x, target_y), sigil);
            }
        }
    }
}

#[cfg(feature = "nova")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_sacrilege_creates_memetic_sigils() {
        let mut world = World::new();
        world.insert_resource(GraffitiMap::default());
        world.insert_resource(Events::<SacrilegeEvent>::default());

        world
            .resource_mut::<Events<SacrilegeEvent>>()
            .send(SacrilegeEvent {
                pos: GridPosition { x: 5, y: 5 },
            });

        let _ = world.run_system_once(cursed_foundations_system);

        let graffiti_map = world.get_resource::<GraffitiMap>().unwrap();

        // Check the center tile
        let center_mark = graffiti_map.markings.get(&(5, 5)).unwrap();
        assert!(matches!(
            center_mark.graffiti_type,
            GraffitiType::MemeticSigil
        ));

        // Check an adjacent tile
        let adjacent_mark = graffiti_map.markings.get(&(6, 4)).unwrap();
        assert!(matches!(
            adjacent_mark.graffiti_type,
            GraffitiType::MemeticSigil
        ));
    }
}
