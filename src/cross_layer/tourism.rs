use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct WeaponizedTourist {
    pub origin_faction: Entity,
    pub annoyance_level: f32,
    pub sabotage_chance: f32,
}

#[derive(Component)]
pub struct FactionMarker;

#[derive(Event)]
pub struct TouristHarmedEvent {
    pub tourist: Entity,
    pub origin_faction: Entity,
    pub offending_faction: Entity,
}

#[derive(Component)]
pub struct CasusBelli {
    pub target: Entity,
    pub reason: String,
}

pub fn weaponized_tourism_sabotage_system(mut tourists: Query<&mut WeaponizedTourist>) {
    for mut tourist in tourists.iter_mut() {
        tourist.annoyance_level += 0.1;
        // In a real game, this would randomly sabotage things based on sabotage_chance.
    }
}

pub fn weaponized_tourism_harm_system(
    mut commands: Commands,
    mut events: EventReader<TouristHarmedEvent>,
) {
    for event in events.read() {
        commands.entity(event.origin_faction).insert(CasusBelli {
            target: event.offending_faction,
            reason: "Tourist Harmed".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_weaponized_tourism_basic_behavior() {
        let mut app = App::new();
        app.add_event::<TouristHarmedEvent>();
        app.add_systems(
            Update,
            (
                weaponized_tourism_sabotage_system,
                weaponized_tourism_harm_system,
            ),
        );

        let origin_faction = app.world_mut().spawn(FactionMarker).id();
        let offending_faction = app.world_mut().spawn(FactionMarker).id();

        let tourist = app
            .world_mut()
            .spawn(WeaponizedTourist {
                origin_faction,
                annoyance_level: 0.0,
                sabotage_chance: 0.1,
            })
            .id();

        app.update();

        // Verify annoyance level increases
        let tourist_cmp = app.world().get::<WeaponizedTourist>(tourist).unwrap();
        assert!(tourist_cmp.annoyance_level > 0.0);

        // Simulate harm
        app.world_mut().send_event(TouristHarmedEvent {
            tourist,
            origin_faction,
            offending_faction,
        });

        app.update();

        // Verify Casus Belli
        let cb = app.world().get::<CasusBelli>(origin_faction).unwrap();
        assert_eq!(cb.target, offending_faction);
        assert_eq!(cb.reason, "Tourist Harmed");
    }
}
