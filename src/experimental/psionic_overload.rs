use crate::layer1::economy::resources::ColonyResources;
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

pub fn psionic_overload_system(
    weather: Res<WeatherState>,
    mut colony_resources: ResMut<ColonyResources>,
    mut query: Query<(&Traits, &mut Needs)>,
) {
    if weather.current_weather == WeatherType::BlissStorm {
        for (traits, mut needs) in query.iter_mut() {
            if traits.has(Trait::Sensitive) {
                colony_resources.knowledge += 0.5;
                needs.rest -= 0.02;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::resources::ColonyResources;
    use crate::layer1::nature::weather::{WeatherState, WeatherType};
    use crate::layer1::psychology::needs::Needs;
    use crate::layer1::psychology::traits::{Trait, Traits};
    use bevy::prelude::*;
    use bevy::MinimalPlugins;

    #[test]
    fn test_psionic_overload() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(WeatherState {
            current_weather: WeatherType::BlissStorm,
            duration_remaining: 100,
        });
        app.insert_resource(ColonyResources::default());

        app.add_systems(Update, psionic_overload_system);

        let mut traits = Traits::default();
        traits.add(Trait::Sensitive);

        let pop = app.world_mut().spawn((traits, Needs::default())).id();

        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert!(resources.knowledge > 0.0);

        let needs = app.world().get::<Needs>(pop).unwrap();
        assert!(needs.rest < 0.8);
    }
}
