use bevy::prelude::*;

#[derive(Component)]
pub struct Planet {
    pub name: String,
}

#[derive(Component)]
pub struct Biome {
    pub biome_type: BiomeType,
}

#[derive(PartialEq, Debug)]
pub enum BiomeType {
    Barren,
    Hybrid(u32),
}

#[derive(Component)]
pub struct BlindTerraformer {
    pub target: Entity,
    pub active: bool,
}

#[derive(Component)]
pub struct Infrastructure {
    pub hp: i32,
}

#[derive(Component)]
pub struct Location {
    pub planet_id: Entity,
}

#[derive(Component)]
pub struct ExoticResourceDeposit;

#[derive(Event)]
pub struct EnvironmentalDamageEvent {
    pub target: Entity,
    pub amount: i32,
}

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnvironmentalDamageEvent>();
        app.add_systems(
            Update,
            (
                apply_blind_terraformer_system,
                apply_environmental_damage_system,
            ),
        );
    }
}

pub fn apply_blind_terraformer_system(
    mut commands: Commands,
    mut terraformers: Query<&mut BlindTerraformer>,
    mut biomes: Query<&mut Biome>,
    infrastructure: Query<(Entity, &Location, &Infrastructure)>,
    mut damage_events: EventWriter<EnvironmentalDamageEvent>,
) {
    for mut terraformer in terraformers.iter_mut() {
        if !terraformer.active {
            continue;
        }

        terraformer.active = false;

        // Change biome
        if let Ok(mut biome) = biomes.get_mut(terraformer.target) {
            biome.biome_type = BiomeType::Hybrid(1);
        }

        // Spawn exotic resources
        commands.spawn((
            ExoticResourceDeposit,
            Location {
                planet_id: terraformer.target,
            },
        ));

        // Damage infrastructure
        for (entity, location, _) in infrastructure.iter() {
            if location.planet_id == terraformer.target {
                damage_events.send(EnvironmentalDamageEvent {
                    target: entity,
                    amount: 50,
                });
            }
        }
    }
}

pub fn apply_environmental_damage_system(
    mut events: EventReader<EnvironmentalDamageEvent>,
    mut infrastructure: Query<&mut Infrastructure>,
) {
    for event in events.read() {
        if let Ok(mut infra) = infrastructure.get_mut(event.target) {
            infra.hp -= event.amount;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blind_terraformer_changes_biome() {
        let mut app = App::new();
        app.add_plugins(EnvironmentPlugin);

        let planet = app
            .world_mut()
            .spawn((
                Planet {
                    name: "Barren Rock".to_string(),
                },
                Biome {
                    biome_type: BiomeType::Barren,
                },
            ))
            .id();

        app.world_mut().spawn(BlindTerraformer {
            target: planet,
            active: true,
        });

        app.update();

        let new_biome = app.world().get::<Biome>(planet).unwrap();
        assert_ne!(
            new_biome.biome_type,
            BiomeType::Barren,
            "The terraformer should alter the biome"
        );
        assert!(
            matches!(new_biome.biome_type, BiomeType::Hybrid(_)),
            "The resulting biome should be a bizarre hybrid type"
        );
    }

    #[test]
    fn test_blind_terraformer_causes_infrastructure_damage() {
        let mut app = App::new();
        app.add_plugins(EnvironmentPlugin);

        let planet = app
            .world_mut()
            .spawn((
                Planet {
                    name: "Barren Rock".to_string(),
                },
                Biome {
                    biome_type: BiomeType::Barren,
                },
            ))
            .id();

        let solar_array = app
            .world_mut()
            .spawn((Infrastructure { hp: 100 }, Location { planet_id: planet }))
            .id();

        app.world_mut().spawn(BlindTerraformer {
            target: planet,
            active: true,
        });

        app.update();
        app.update();

        let damaged_infrastructure = app.world().get::<Infrastructure>(solar_array).unwrap();
        assert!(
            damaged_infrastructure.hp < 100,
            "Existing infrastructure should take damage due to extreme environmental shifts"
        );
    }

    #[test]
    fn test_blind_terraformer_spawns_exotic_resources() {
        let mut app = App::new();
        app.add_plugins(EnvironmentPlugin);

        let planet = app
            .world_mut()
            .spawn((
                Planet {
                    name: "Barren Rock".to_string(),
                },
                Biome {
                    biome_type: BiomeType::Barren,
                },
            ))
            .id();

        app.world_mut().spawn(BlindTerraformer {
            target: planet,
            active: true,
        });

        app.update();

        let mut query = app.world_mut().query::<&ExoticResourceDeposit>();
        let deposits_count = query.iter(app.world()).count();
        assert!(
            deposits_count > 0,
            "The terraformed hybrid biome should spawn exotic resources"
        );
    }
}
