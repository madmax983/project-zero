use crate::layer1::entities::pop::Pop;
use crate::layer1::social::morale::{MoodModifier, Morale};
use crate::layer2::debris::OrbitalDebris;
use crate::layer2::fleet::StationType;
use crate::layer2::station::Station;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct DebrisCultist;

pub fn evaluate_debris_cult_formation_system(
    mut commands: Commands,
    debris_query: Query<&OrbitalDebris>,
    pop_query: Query<(Entity, &Station, Option<&DebrisCultist>), With<Pop>>,
) {
    let total_debris: f32 = debris_query.iter().map(|d| d.0).sum();

    if total_debris >= 5.0 {
        for (entity, station, cultist) in pop_query.iter() {
            if matches!(
                station.station_type,
                StationType::Outpost | StationType::MiningPlatform | StationType::Derelict
            ) && cultist.is_none()
            {
                commands.entity(entity).insert(DebrisCultist);
            }
        }
    } else {
        // Lose cult status if debris is cleaned up
        for (entity, _, cultist) in pop_query.iter() {
            if cultist.is_some() {
                commands.entity(entity).remove::<DebrisCultist>();
            }
        }
    }
}

pub fn apply_debris_cult_morale_system(
    debris_query: Query<&OrbitalDebris>,
    mut cultist_query: Query<&mut Morale, With<DebrisCultist>>,
) {
    let total_debris: f32 = debris_query.iter().map(|d| d.0).sum();
    let bonus = total_debris * 0.05; // 0.05 morale modifier per debris

    for mut morale in cultist_query.iter_mut() {
        // Remove existing debris cult modifier if present
        morale
            .modifiers
            .retain(|m| m.label != "Orbital Debris Cult");

        // Add the fresh modifier that applies this tick.
        morale.add_modifier(MoodModifier {
            label: "Orbital Debris Cult".to_string(),
            value: bonus,
            duration: 2, // Lasts for 2 ticks to ensure it applies but decays if the system stops applying it.
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::social::morale::Morale;
    use crate::layer2::debris::OrbitalDebris;
    use crate::layer2::fleet::StationType;
    use crate::layer2::station::Station;
    use bevy_app::{App, Update};

    #[test]
    fn test_debris_cult_activation() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_debris_cult_formation_system);

        // Spawn a lot of debris
        for _ in 0..10 {
            app.world_mut().spawn(OrbitalDebris(1.0));
        }

        // Spawn a low-tier orbital pop
        let entity = app
            .world_mut()
            .spawn((
                Pop,
                Station {
                    station_type: StationType::Outpost,
                },
            ))
            .id();

        app.update();

        // Should gain the DebrisCultist component
        assert!(app.world().get::<DebrisCultist>(entity).is_some());
    }

    #[test]
    fn test_debris_cult_morale_boost() {
        let mut app = App::new();
        app.add_systems(Update, apply_debris_cult_morale_system);

        app.world_mut().spawn(OrbitalDebris(1.0));
        app.world_mut().spawn(OrbitalDebris(1.0));

        let entity = app
            .world_mut()
            .spawn((
                Pop,
                DebrisCultist,
                Morale {
                    value: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        app.update();

        // Morale modifier should be added based on the amount of debris (0.05 per debris)
        let morale = app.world().get::<Morale>(entity).unwrap();
        assert_eq!(morale.modifiers.len(), 1);
        assert_eq!(morale.modifiers[0].label, "Orbital Debris Cult");
        assert!((morale.modifiers[0].value - 0.1).abs() < 0.001);
    }
}
