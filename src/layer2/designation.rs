use bevy_ecs::prelude::*;
use crate::layer1::resources::ColonyResources;
use crate::layer1::social::unrest::Unrest;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanetDesignation {
    AgriWorld,
    ForgeWorld,
    FortressWorld,
}

#[derive(Component)]
pub struct PlanetDesignationChange {
    pub previous: PlanetDesignation,
    pub next: PlanetDesignation,
}

pub fn evaluate_designation_bonuses(
    query: Query<&PlanetDesignation>,
    mut resources: ResMut<ColonyResources>,
) {
    for designation in query.iter() {
        match designation {
            PlanetDesignation::AgriWorld => {
                resources.add_food(20.0);
                resources.consume(crate::layer1::resources::ResourceType::Stone, 5.0);
                resources.consume(crate::layer1::resources::ResourceType::Metal, 5.0);
            }
            PlanetDesignation::ForgeWorld => {
                resources.add_metal(20.0);
                resources.add_tools(10.0);
                resources.consume(crate::layer1::resources::ResourceType::Food, 5.0);
                resources.consume(crate::layer1::resources::ResourceType::Wood, 5.0);
            }
            PlanetDesignation::FortressWorld => {
                resources.add_building_permits(10.0);
                resources.add_scrap(5.0);
                resources.consume(crate::layer1::resources::ResourceType::Food, 10.0);
                resources.consume(crate::layer1::resources::ResourceType::Metal, 10.0);
            }
        }
    }
}

pub fn handle_designation_changes(
    mut commands: Commands,
    query: Query<(Entity, &PlanetDesignationChange)>,
    mut unrest: ResMut<Unrest>,
) {
    for (entity, change) in query.iter() {
        // Massive spike in unrest when changing designation
        unrest.level += 50.0;

        // Apply the new designation and remove the change component
        commands.entity(entity).insert(change.next);
        commands.entity(entity).remove::<PlanetDesignationChange>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planet_designation_agri_world() {
        let mut app = bevy_app::App::new();
        app.insert_resource(ColonyResources::default());
        app.world_mut().spawn(PlanetDesignation::AgriWorld);

        app.add_systems(bevy_app::Update, evaluate_designation_bonuses);
        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert!(resources.food > 10.0, "Food should be boosted by AgriWorld designation");
        assert!(resources.metal < 0.0 || resources.ore < 0.0 || resources.stone < 5.0, "Industrial resources should be penalized by AgriWorld designation");
    }

    #[test]
    fn test_planet_designation_forge_world() {
        let mut app = bevy_app::App::new();
        app.insert_resource(ColonyResources::default());
        app.world_mut().spawn(PlanetDesignation::ForgeWorld);

        app.add_systems(bevy_app::Update, evaluate_designation_bonuses);
        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert!(resources.metal > 0.0, "Metal should be boosted by ForgeWorld designation");
        assert!(resources.tools > 2.0, "Tools should be boosted by ForgeWorld designation");
        assert!(resources.food < 10.0, "Food should be penalized by ForgeWorld designation");
    }

    #[test]
    fn test_planet_designation_fortress_world() {
        let mut app = bevy_app::App::new();
        app.insert_resource(ColonyResources::default());
        app.world_mut().spawn(PlanetDesignation::FortressWorld);

        app.add_systems(bevy_app::Update, evaluate_designation_bonuses);
        app.update();

        let resources = app.world().resource::<ColonyResources>();
        assert!(resources.building_permits > 0.0, "Building Permits should be boosted by FortressWorld designation");
        assert!(resources.food < 10.0, "Food should be penalized by FortressWorld designation");
    }

    #[test]
    fn test_planet_designation_change_causes_unrest() {
        let mut app = bevy_app::App::new();
        app.insert_resource(Unrest { level: 0.0, ..Default::default() });

        app.world_mut().spawn(PlanetDesignationChange {
            previous: PlanetDesignation::FortressWorld,
            next: PlanetDesignation::AgriWorld,
        });

        app.add_systems(bevy_app::Update, handle_designation_changes);
        app.update();

        let unrest = app.world().resource::<Unrest>();
        assert!(unrest.level >= 50.0, "Changing designation should cause a massive spike in unrest");
    }
}
