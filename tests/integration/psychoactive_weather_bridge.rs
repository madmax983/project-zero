use bevy::prelude::*;
use scale::layer1::nature::psychoactive_weather::{apply_weather_moodlets_system, process_euphoria_effects_system, Mood, Moodlet};
use scale::layer1::social::morale::Morale;
use scale::layer1::jobs::CurrentTask;
use scale::layer1::entities::pop::Pop;
use scale::layer1::map::GridPosition;
use scale::layer1::nature::weather::{WeatherState, WeatherType};
use scale::layer1::structural_integrity::RoofGrid;

#[test]
fn test_psychoactive_weather_effects_chain() {
    let mut app = App::new();
    app.insert_resource(WeatherState {
        current_weather: WeatherType::BlissStorm,
        duration_remaining: 100,
    });

    let roof_grid = RoofGrid::new(10, 10);
    app.insert_resource(roof_grid);

    app.add_systems(Update, (
        apply_weather_moodlets_system,
        process_euphoria_effects_system,
    ).chain());

    let pop = app.world_mut().spawn((
        Pop,
        GridPosition { x: 5, y: 5 }, // Outdoors
        Mood::default(),
        CurrentTask { efficiency: 1.0, ..Default::default() },
        Morale { value: 50.0, ..Default::default() },
    )).id();

    app.update();

    let mood = app.world().get::<Mood>(pop).unwrap();
    assert!(mood.has_moodlet(Moodlet::Euphoric));

    let task = app.world().get::<CurrentTask>(pop).unwrap();
    assert_eq!(task.efficiency, 0.0);

    let morale = app.world().get::<Morale>(pop).unwrap();
    assert!(morale.value > 50.0);
}
