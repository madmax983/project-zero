use crate::layer1::social::morale::MoodModifier;
use bevy::prelude::*;

use crate::layer1::funeral::Corpse;
use crate::layer1::social::morale::Morale;

#[derive(Resource)]
pub enum BurialPolicy {
    Land,
    Orbital,
}

#[derive(Resource, Default)]
pub struct OrbitalCemetery {
    pub coffin_count: u32,
}

pub fn process_corpses_system(
    mut commands: Commands,
    policy: Option<Res<BurialPolicy>>,
    mut cemetery: ResMut<OrbitalCemetery>,
    corpse_query: Query<Entity, With<Corpse>>,
) {
    if let Some(p) = policy {
        if matches!(*p, BurialPolicy::Orbital) {
            for entity in corpse_query.iter() {
                commands.entity(entity).despawn();
                cemetery.coffin_count += 1;
            }
        }
    }
}

#[derive(Component)]
pub struct LaunchSequence {
    pub base_risk: f32,
}

pub fn calculate_launch_risk_system(
    cemetery: Res<OrbitalCemetery>,
    mut launch_query: Query<&mut LaunchSequence>,
) {
    let extra_risk = (cemetery.coffin_count as f32 / 100.0) * 0.01;
    for mut launch in launch_query.iter_mut() {
        launch.base_risk += extra_risk;
    }
}

#[derive(Event)]
pub struct ClearCemeteryEvent {
    pub coffins_destroyed: u32,
}

pub fn clear_cemetery_system(
    mut events: EventReader<ClearCemeteryEvent>,
    mut cemetery: ResMut<OrbitalCemetery>,
    mut pop_query: Query<&mut Morale>,
) {
    for event in events.read() {
        cemetery.coffin_count = cemetery
            .coffin_count
            .saturating_sub(event.coffins_destroyed);

        let penalty = -(event.coffins_destroyed as f32) * 0.01; // Scale down for MoodModifier
        for mut morale in pop_query.iter_mut() {
            morale.add_modifier(MoodModifier {
                label: "Desecration".to_string(),
                value: penalty.clamp(-1.0, 0.0),
                duration: 100, // Timed moodlet
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer1::funeral::Corpse;
    use crate::layer1::social::morale::Morale;

    #[test]
    fn test_orbital_burial_policy_increases_coffin_density() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<OrbitalCemetery>()
            .add_systems(Update, process_corpses_system);

        // Arrange: Enable policy and spawn a corpse
        app.world_mut().insert_resource(BurialPolicy::Orbital);
        app.world_mut().spawn(Corpse {
            name: "Bob".to_string(),
            decay: 0.0,
        });

        app.update();

        // Assert: Corpse despawned from land, added to orbit
        assert_eq!(
            app.world_mut().query::<&Corpse>().iter(app.world()).count(),
            0
        );
        assert_eq!(app.world().resource::<OrbitalCemetery>().coffin_count, 1);
    }

    #[test]
    fn test_coffin_density_causes_launch_risk() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(OrbitalCemetery { coffin_count: 500 }) // High density
            .add_systems(Update, calculate_launch_risk_system);

        // Arrange: A ship preparing to launch
        let ship_entity = app
            .world_mut()
            .spawn(LaunchSequence { base_risk: 0.05 })
            .id();

        app.update();

        // Assert: Risk is increased due to coffin density
        let risk = app
            .world()
            .get::<LaunchSequence>(ship_entity)
            .unwrap()
            .base_risk;
        assert!(
            risk > 0.05,
            "Launch risk should increase with coffin density"
        );
    }

    #[test]
    fn test_clearing_the_cemetery() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(OrbitalCemetery { coffin_count: 100 })
            .add_systems(Update, clear_cemetery_system);
        app.add_event::<ClearCemeteryEvent>();

        // Arrange: Fire the clearing laser
        app.world_mut()
            .resource_mut::<Events<ClearCemeteryEvent>>()
            .send(ClearCemeteryEvent {
                coffins_destroyed: 50,
            });

        app.update();

        // Assert: Density reduced
        assert_eq!(app.world().resource::<OrbitalCemetery>().coffin_count, 50);
    }

    #[test]
    fn test_clearing_causes_morale_penalty() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(OrbitalCemetery { coffin_count: 100 })
            .add_systems(Update, clear_cemetery_system);
        app.add_event::<ClearCemeteryEvent>();

        // Arrange: A pop observing the desecration
        let pop = app.world_mut().spawn(Morale::default()).id();

        app.world_mut()
            .resource_mut::<Events<ClearCemeteryEvent>>()
            .send(ClearCemeteryEvent {
                coffins_destroyed: 10,
            });

        app.update();

        // Assert: Morale is penalized
        let morale = app.world().get::<Morale>(pop).unwrap();
        assert!(!morale.modifiers.is_empty());
        assert!(
            morale.modifiers[0].value < 0.0,
            "Destroying ancestors should add negative modifier"
        );
    }
}
