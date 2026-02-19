#[cfg(test)]
mod tests {
    use scale::layer1::access_control::{AccessControl, AccessMode, check_access};
    use scale::layer1::pop::Pop;
    use bevy_ecs::prelude::*;
    use std::collections::HashSet;

    #[test]
    fn test_access_control_dead_entity() {
        let mut world = World::new();
        let pop = world.spawn(Pop).id();

        let mut allowed = HashSet::new();
        allowed.insert(pop);

        let door = world.spawn(AccessControl {
            mode: AccessMode::Restricted,
            allowed_pops: allowed,
            ..Default::default()
        }).id();

        // Kill the pop
        world.despawn(pop);

        // Access check should return FALSE because the entity is dead.
        let result = check_access(&world, door, pop);
        assert!(!result, "Dead entity should be denied access despite ID match");
    }
}
