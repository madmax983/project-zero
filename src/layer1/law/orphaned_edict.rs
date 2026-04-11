use crate::layer1::administration::edicts::{ColonyPolicies, Policy};
use crate::layer1::memetics::MemeticCarrier;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;

pub fn orphaned_edict_enforcement_system(
    mut commands: Commands,
    policies: Res<ColonyPolicies>,
    query: Query<Entity, (With<Pop>, With<MemeticCarrier>)>,
) {
    if policies.is_active(Policy::ShootInfected) {
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_orphaned_edict_enforcement() {
        // Arrange: Create app, set an active 'Orphaned' Edict (e.g., ShootInfected)
        let mut app = bevy_app::App::new();

        let mut policies = ColonyPolicies::default();
        policies.active_policies.insert(Policy::ShootInfected);
        policies.orphaned_policies.insert(Policy::ShootInfected);
        app.insert_resource(policies);

        // Spawn an 'Infected' pop
        let infected_pop = app.world_mut().spawn((Pop, MemeticCarrier)).id();

        app.add_systems(bevy_app::Update, orphaned_edict_enforcement_system);

        // Act: Process automated drone/enforcer logic
        app.update();

        // Assert: The pop is targeted/killed by the automated system
        assert!(
            app.world().get_entity(infected_pop).is_err(),
            "Infected pop should be despawned"
        );
    }
}
