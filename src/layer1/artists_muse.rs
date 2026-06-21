use crate::layer1::crafting::{CraftEvent, Quality};
use crate::layer1::entities::pop::Pop;
use crate::layer1::psychology::traits::{Trait, Traits};
use crate::layer1::social::morale::Morale;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct ArtWork {
    pub item_type: String,
    pub quality: Quality,
}

pub fn evaluate_art_quality_system(
    mut commands: Commands,
    mut events: EventReader<CraftEvent>,
    query: Query<(&Morale, &Traits), With<Pop>>,
) {
    for event in events.read() {
        if let Ok((morale, traits)) = query.get(event.crafter) {
            if traits.has(Trait::Artistic) {
                // If they have Trauma, it acts as suffering. Mood spec uses 0 to 100, our Morale uses 0.0 to 1.0.
                // Spec says < 30.0 mood -> Quality::Masterpiece
                // > 80.0 mood -> Quality::Poor
                // Let's use < 0.3 for Masterpiece, > 0.8 for Poor
                let quality = if morale.value < 0.3 || traits.has(Trait::Trauma) {
                    Quality::Masterpiece // Suffering = Great Art
                } else if morale.value > 0.8 {
                    Quality::Poor // Happy = Boring Art
                } else {
                    Quality::Normal
                };

                commands.spawn(ArtWork {
                    item_type: event.item_type.clone(),
                    quality,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_unhappy_artist_creates_masterpiece() {
        let mut app = App::new();
        app.add_event::<CraftEvent>();
        app.add_systems(Update, evaluate_art_quality_system);

        let mut traits = Traits::default();
        traits.add(Trait::Artistic);

        let tortured_artist = app
            .world_mut()
            .spawn((
                Pop,
                traits,
                Morale {
                    value: 0.1,
                    modifiers: vec![],
                }, // Very unhappy
            ))
            .id();

        // Simulate completing an art project
        app.world_mut()
            .resource_mut::<Events<CraftEvent>>()
            .send(CraftEvent {
                crafter: tortured_artist,
                item_type: "Sculpture".to_string(),
            });

        app.update();

        // Verify a masterpiece was spawned
        let mut found_masterpiece = false;
        let mut art_query = app.world_mut().query::<&ArtWork>();
        for art in art_query.iter(app.world()) {
            if art.quality == Quality::Masterpiece {
                found_masterpiece = true;
                break;
            }
        }

        assert!(
            found_masterpiece,
            "An artist with low mood should produce a Masterpiece."
        );
    }

    #[test]
    fn test_happy_artist_creates_boring_art() {
        let mut app = App::new();
        app.add_event::<CraftEvent>();
        app.add_systems(Update, evaluate_art_quality_system);

        let mut traits = Traits::default();
        traits.add(Trait::Artistic);

        let happy_artist = app
            .world_mut()
            .spawn((
                Pop,
                traits,
                Morale {
                    value: 0.95,
                    modifiers: vec![],
                }, // Very happy
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<CraftEvent>>()
            .send(CraftEvent {
                crafter: happy_artist,
                item_type: "Painting".to_string(),
            });

        app.update();

        let mut found_boring = false;
        let mut art_query = app.world_mut().query::<&ArtWork>();
        for art in art_query.iter(app.world()) {
            if art.quality == Quality::Poor || art.quality == Quality::Normal {
                found_boring = true;
                break;
            }
        }

        assert!(
            found_boring,
            "A happy artist should produce Normal or Poor quality art."
        );
    }

    #[test]
    fn test_trauma_artist_creates_masterpiece() {
        let mut app = App::new();
        app.add_event::<CraftEvent>();
        app.add_systems(Update, evaluate_art_quality_system);

        let mut traits = Traits::default();
        traits.add(Trait::Artistic);
        traits.add(Trait::Trauma);

        let happy_but_traumatized = app
            .world_mut()
            .spawn((
                Pop,
                traits,
                Morale {
                    value: 0.95,
                    modifiers: vec![],
                }, // Very happy but has trauma
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<CraftEvent>>()
            .send(CraftEvent {
                crafter: happy_but_traumatized,
                item_type: "Sculpture".to_string(),
            });

        app.update();

        let mut found_masterpiece = false;
        let mut art_query = app.world_mut().query::<&ArtWork>();
        for art in art_query.iter(app.world()) {
            if art.quality == Quality::Masterpiece {
                found_masterpiece = true;
                break;
            }
        }

        assert!(
            found_masterpiece,
            "An artist with trauma should produce a Masterpiece regardless of mood."
        );
    }
}
