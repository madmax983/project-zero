use bevy_ecs::prelude::*;

use crate::layer1::entities::pop::Pop;
use crate::layer1::jobs::CurrentTask;
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::social::morale::Morale;
use crate::layer1::structural_integrity::RoofGrid;

use crate::layer1::map::GridPosition;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Moodlet {
    Euphoric,
    Paranoid,
}

#[derive(Component, Default, Debug)]
pub struct Mood {
    pub active_moodlets: Vec<Moodlet>,
}

impl Mood {
    pub fn has_moodlet(&self, moodlet: Moodlet) -> bool {
        self.active_moodlets.contains(&moodlet)
    }
    pub fn add(&mut self, moodlet: Moodlet) {
        if !self.has_moodlet(moodlet) {
            self.active_moodlets.push(moodlet);
        }
    }
}

pub fn apply_weather_moodlets_system(
    weather: Option<Res<WeatherState>>,
    roof_grid: Option<Res<RoofGrid>>,
    mut pops: Query<(&mut Mood, &GridPosition), With<Pop>>,
) {
    if let Some(w) = weather {
        let moodlet_to_apply = match w.current_weather {
            WeatherType::BlissStorm => Some(Moodlet::Euphoric),
            WeatherType::MutagenicRain | WeatherType::SporeStorm => Some(Moodlet::Paranoid), // Including MutagenicRain as well since we modified weather
            _ => None,
        };

        if let Some(moodlet) = moodlet_to_apply {
            let grid = roof_grid.as_deref();

            for (mut mood, pos) in pops.iter_mut() {
                // If there's no roof grid, we assume outdoors.
                // If there is, we check if the pop is under a roof (indoors).
                let indoors = grid.is_some_and(|g| g.has_roof(pos.x, pos.y));

                if !indoors {
                    mood.add(moodlet);
                }
            }
        }
    }
}

pub fn process_euphoria_effects_system(
    mut pop_query: Query<(&Mood, &mut CurrentTask, &mut Morale), With<Pop>>,
) {
    for (mood, mut task, mut morale) in pop_query.iter_mut() {
        if mood.has_moodlet(Moodlet::Euphoric) {
            // Work halts
            task.efficiency = 0.0;
            // Morale boost (simplified, spec: +10)
            morale.value += 10.0;
        } else {
            task.efficiency = 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_psychoactive_weather_applies_moodlets() {
        let mut world = setup_world();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::BlissStorm,
            duration_remaining: 100,
        });

        // Outdoors pop (no RoofGrid)
        let pop = world
            .spawn((Pop, GridPosition { x: 5, y: 5 }, Mood::default()))
            .id();

        world
            .run_system_once(apply_weather_moodlets_system)
            .unwrap();

        let mood = world.get::<Mood>(pop).unwrap();
        assert!(mood.has_moodlet(Moodlet::Euphoric));
    }

    #[test]
    fn test_indoors_prevents_psychoactive_effects() {
        let mut world = setup_world();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::SporeStorm,
            duration_remaining: 100,
        });

        let mut roof_grid = RoofGrid::new(10, 10);
        roof_grid.set(5, 5, true); // Pop is indoors
        world.insert_resource(roof_grid);

        let pop = world
            .spawn((Pop, GridPosition { x: 5, y: 5 }, Mood::default()))
            .id();

        world
            .run_system_once(apply_weather_moodlets_system)
            .unwrap();

        let mood = world.get::<Mood>(pop).unwrap();
        assert!(!mood.has_moodlet(Moodlet::Paranoid));
    }

    #[test]
    fn test_euphoria_halts_work_but_boosts_morale() {
        let mut world = setup_world();
        let mut starting_mood = Mood::default();
        starting_mood.add(Moodlet::Euphoric);

        let starting_morale = Morale {
            value: 50.0,
            ..Default::default()
        };

        let pop = world
            .spawn((
                Pop,
                starting_mood,
                CurrentTask {
                    efficiency: 1.0,
                    ..Default::default()
                },
                starting_morale,
            ))
            .id();

        world
            .run_system_once(process_euphoria_effects_system)
            .unwrap();

        let task = world.get::<CurrentTask>(pop).unwrap();
        let morale = world.get::<Morale>(pop).unwrap();

        // Work halts (efficiency 0)
        assert_eq!(task.efficiency, 0.0);
        // Morale boosted
        assert!(morale.value > 50.0);
    }
}
