use crate::layer1::disasters::MegaEvent;
use crate::layer2::planet::{PlanetNode, PlanetTexture};
use bevy::prelude::*;

pub fn process_planetary_scars_system(
    mut events: EventReader<MegaEvent>,
    mut query: Query<&mut PlanetTexture, With<PlanetNode>>,
) {
    for event in events.read() {
        if event.intensity >= 50.0 {
            if let Ok(mut texture) = query.get_mut(event.planet_entity) {
                match event.event_type.as_str() {
                    "NuclearBlast" => texture.scars.push("Crater".to_string()),
                    "MegaFire" => texture.scars.push("AshCloud".to_string()),
                    _ => {} // Other events might not cause visible macro scars
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::disasters::MegaEvent;
    use crate::layer2::planet::{PlanetNode, PlanetTexture};

    #[test]
    fn test_nuclear_blast_applies_crater_scar_to_planet() {
        let mut app = App::new();
        app.add_event::<MegaEvent>();
        app.add_systems(Update, process_planetary_scars_system);

        let planet = app
            .world_mut()
            .spawn((
                PlanetNode,
                PlanetTexture {
                    id: "pristine_earth".to_string(),
                    scars: vec![],
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<MegaEvent>>()
            .send(MegaEvent {
                planet_entity: planet,
                event_type: "NuclearBlast".to_string(),
                intensity: 100.0,
            });

        app.update();

        let texture = app.world().get::<PlanetTexture>(planet).unwrap();
        assert!(
            texture.scars.contains(&"Crater".to_string()),
            "A nuclear blast should add a Crater scar to the planet texture."
        );
    }

    #[test]
    fn test_forest_fire_applies_ash_scar_to_planet() {
        let mut app = App::new();
        app.add_event::<MegaEvent>();
        app.add_systems(Update, process_planetary_scars_system);

        let planet = app
            .world_mut()
            .spawn((
                PlanetNode,
                PlanetTexture {
                    id: "pristine_earth".to_string(),
                    scars: vec![],
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<MegaEvent>>()
            .send(MegaEvent {
                planet_entity: planet,
                event_type: "MegaFire".to_string(),
                intensity: 80.0,
            });

        app.update();

        let texture = app.world().get::<PlanetTexture>(planet).unwrap();
        assert!(
            texture.scars.contains(&"AshCloud".to_string()),
            "A mega-fire should add an Ash Cloud scar to the planet texture."
        );
    }

    #[test]
    fn test_event_below_intensity_threshold_does_not_apply_scar() {
        let mut app = App::new();
        app.add_event::<MegaEvent>();
        app.add_systems(Update, process_planetary_scars_system);

        let planet = app
            .world_mut()
            .spawn((
                PlanetNode,
                PlanetTexture {
                    id: "pristine_earth".to_string(),
                    scars: vec![],
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<MegaEvent>>()
            .send(MegaEvent {
                planet_entity: planet,
                event_type: "MegaFire".to_string(),
                intensity: 49.9,
            });

        app.update();

        let texture = app.world().get::<PlanetTexture>(planet).unwrap();
        assert!(
            !texture.scars.contains(&"AshCloud".to_string()),
            "An event below intensity 50 should not apply a scar."
        );
    }
}
