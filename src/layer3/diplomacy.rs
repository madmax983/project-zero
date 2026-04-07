use crate::layer1::spore_diplomat::SporeInfection;
use bevy::prelude::*;

pub mod succession;

#[derive(Component)]
pub struct Envoy;

#[derive(Component)]
pub struct Treaty {
    pub has_spore_propagation: bool,
}

#[derive(Resource)]
pub struct ActiveNegotiation {
    pub envoy: Entity,
    pub treaty: Entity,
}

pub fn diplomatic_negotiation_system(
    negotiation_opt: Option<Res<ActiveNegotiation>>,
    envoys: Query<&SporeInfection, With<Envoy>>,
    mut treaties: Query<&mut Treaty>,
) {
    if let Some(negotiation) = negotiation_opt {
        if let Ok(infection) = envoys.get(negotiation.envoy) {
            if infection.severity > 0.0 {
                if let Ok(mut treaty) = treaties.get_mut(negotiation.treaty) {
                    treaty.has_spore_propagation = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_spore_diplomat_inserts_hidden_clause() {
        let mut app = App::new();
        let pop = app
            .world_mut()
            .spawn((Pop, Envoy, SporeInfection { severity: 1.0 }))
            .id();
        let treaty = app.world_mut().spawn(Treaty { has_spore_propagation: false }).id();
        app.world_mut()
            .insert_resource(ActiveNegotiation { envoy: pop, treaty });

        app.add_systems(Update, diplomatic_negotiation_system);
        app.update();

        let treaty_data = app.world().get::<Treaty>(treaty).unwrap();
        assert!(treaty_data.has_spore_propagation);
    }
}
