use crate::layer1::map::GridPosition;
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::pop::Pop;
use crate::layer1::structural_integrity::RoofGrid;
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;
use rand::Rng;

/// Tag component to ensure pops don't mutate infinitely during a single storm.
#[derive(Component, Default)]
pub struct Mutated;

/// Applies random mutant traits to pops exposed to mutagenic rain.
type MutagenicRainQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static GridPosition, &'static mut Traits),
    (With<Pop>, Without<Mutated>),
>;
pub fn apply_mutagenic_rain_system(
    mut commands: Commands,
    weather: Res<WeatherState>,
    grid: Res<RoofGrid>,
    mut query: MutagenicRainQuery<'_, '_>,
) {
    if weather.current_weather != WeatherType::MutagenicRain {
        return;
    }

    let mut rng = rand::thread_rng();

    for (entity, pos, mut traits) in query.iter_mut() {
        let is_under_roof = grid.has_roof(pos.x, pos.y);

        if !is_under_roof {
            // High chance to mutate (e.g., 5% per tick for testing, but in reality maybe 0.1%)
            if rng.gen_bool(0.05) {
                let possible_traits = [
                    Trait::Photosynthesis,
                    Trait::ThickSkin,
                    Trait::BrittleBones,
                    Trait::ExtremeHunger,
                ];

                let new_trait = possible_traits[rng.gen_range(0..possible_traits.len())];

                if !traits.has(new_trait) {
                    traits.add(new_trait);
                    // Add Mutated tag to prevent infinite mutations in one storm
                    commands.entity(entity).insert(Mutated);
                }
            }
        }
    }
}

/// Removes the Mutated tag when the weather clears, allowing mutation in the next storm.
pub fn clear_mutation_immunity_system(
    mut commands: Commands,
    weather: Res<WeatherState>,
    query: Query<Entity, With<Mutated>>,
) {
    if weather.current_weather != WeatherType::MutagenicRain {
        for entity in query.iter() {
            commands.entity(entity).remove::<Mutated>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::nature::weather::{WeatherState, WeatherType};
    use crate::layer1::pop::Pop;
    use crate::layer1::structural_integrity::RoofGrid;
    use crate::layer1::traits::{Trait, Traits};
    use bevy_app::{App, Update};

    fn setup_app() -> App {
        let mut app = App::new();

        let mut grid = RoofGrid::new(10, 10);
        // Add a roof at (5, 5)
        grid.has_roof[5 * 10 + 5] = true;
        app.insert_resource(grid);

        app.insert_resource(WeatherState {
            current_weather: WeatherType::MutagenicRain,
            duration_remaining: 100,
        });

        app.add_systems(Update, apply_mutagenic_rain_system);

        app
    }

    #[test]
    fn test_indoor_pop_not_mutated() {
        let mut app = setup_app();

        let safe_pop = app
            .world_mut()
            .spawn((
                GridPosition { x: 5, y: 5 }, // Under roof
                Pop,
                Traits::default(),
            ))
            .id();

        // Run system enough times to statistically guarantee mutation if not protected
        for _ in 0..100 {
            app.update();
        }

        let traits = app.world().get::<Traits>(safe_pop).unwrap();
        assert_eq!(traits.0, 0, "Pop under roof should not mutate.");
    }

    #[test]
    fn test_outdoor_pop_mutates() {
        let mut app = setup_app();

        let exposed_pop = app
            .world_mut()
            .spawn((
                GridPosition { x: 2, y: 2 }, // No roof above
                Pop,
                Traits::default(),
            ))
            .id();

        // Run system enough times to statistically guarantee mutation (1000 iterations to avoid flakiness)
        for _ in 0..1000 {
            app.update();
        }

        let traits = app.world().get::<Traits>(exposed_pop).unwrap();
        assert_ne!(
            traits.0, 0,
            "Pop outside during mutagenic rain should mutate."
        );

        // Ensure trait is a mutant trait
        let has_mutant_trait = traits.has(Trait::Photosynthesis)
            || traits.has(Trait::ThickSkin)
            || traits.has(Trait::BrittleBones)
            || traits.has(Trait::ExtremeHunger);
        assert!(has_mutant_trait, "Gained trait must be a mutant variant.");

        // Ensure they got tagged so they don't mutate infinitely
        assert!(
            app.world().get::<Mutated>(exposed_pop).is_some(),
            "Pop should be tagged as Mutated"
        );
    }

    #[test]
    fn test_mutated_tag_prevents_multiple_mutations() {
        let mut app = setup_app();

        let exposed_pop = app
            .world_mut()
            .spawn((
                GridPosition { x: 2, y: 2 }, // No roof above
                Pop,
                Traits::default(),
                Mutated, // Already mutated
            ))
            .id();

        for _ in 0..100 {
            app.update();
        }

        let traits = app.world().get::<Traits>(exposed_pop).unwrap();
        assert_eq!(
            traits.0, 0,
            "Pop with Mutated tag should not gain further traits."
        );
    }
}
