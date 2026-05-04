use crate::layer1::economy::ColonyResources;
use crate::layer1::energy::PowerSource;
use bevy::prelude::*;

#[derive(Component)]
pub struct PredecessorRuin {
    pub awakened: bool,
    pub research_bonus_rate: f32,
}

#[derive(Component)]
pub struct PredecessorOrbitalShield;

pub fn predecessor_ruins_passive_bonus_system(
    mut resources: ResMut<ColonyResources>,
    query: Query<&PredecessorRuin>,
    _time: Res<Time>,
) {
    for ruin in query.iter() {
        if !ruin.awakened {
            resources.add_knowledge(ruin.research_bonus_rate);
        }
    }
}

pub fn predecessor_ruins_awakening_system(
    mut commands: Commands,
    mut ruins_query: Query<(Entity, &mut PredecessorRuin)>,
    source_query: Query<&PowerSource>,
) {
    let mut total_grid_energy = 0.0;
    for source in source_query.iter() {
        if source.active {
            total_grid_energy += source.output;
        }
    }

    if total_grid_energy > 5000.0 {
        for (entity, mut ruin) in ruins_query.iter_mut() {
            if !ruin.awakened {
                ruin.awakened = true;
                commands.entity(entity).insert(PredecessorOrbitalShield);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::economy::ColonyResources;
    use crate::layer1::energy::PowerSource;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(
            Update,
            (
                predecessor_ruins_passive_bonus_system,
                predecessor_ruins_awakening_system,
            ),
        );
        app
    }

    #[test]
    fn test_predecessor_ruins_provide_passive_research_bonus() {
        let mut app = setup_app();

        app.insert_resource(ColonyResources {
            knowledge: 0.0,
            ..Default::default()
        });
        app.world_mut().spawn(PredecessorRuin {
            awakened: false,
            research_bonus_rate: 5.0,
        });

        let mut t = Time::<Virtual>::default();
        t.advance_by(std::time::Duration::from_secs(1));
        app.insert_resource(t);
        app.insert_resource(Time::<()>::default());

        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert!(
            resources.knowledge > 0.0,
            "Passive research bonus should be applied"
        );
    }

    #[test]
    fn test_massive_energy_spike_awakens_dormant_systems() {
        let mut app = setup_app();
        app.insert_resource(ColonyResources {
            knowledge: 0.0,
            ..Default::default()
        });
        let mut t = Time::<Virtual>::default();
        t.advance_by(std::time::Duration::from_secs(1));
        app.insert_resource(t);
        app.insert_resource(Time::<()>::default());

        let ruin_entity = app
            .world_mut()
            .spawn(PredecessorRuin {
                awakened: false,
                research_bonus_rate: 5.0,
            })
            .id();

        // Create an energy grid with a massive spike
        app.world_mut().spawn(PowerSource {
            output: 10000.0,
            active: true,
        });

        app.update();

        // Assert ruin is awakened
        let ruin = app.world().get::<PredecessorRuin>(ruin_entity).unwrap();
        assert!(ruin.awakened, "Massive energy spike should awaken the ruin");
        assert!(app
            .world()
            .get::<PredecessorOrbitalShield>(ruin_entity)
            .is_some());
    }
}
