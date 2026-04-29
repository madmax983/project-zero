git checkout HEAD~1 src/simulation.rs
sed -i 's/        crate::layer2::combat::fleet_combat_system,/        crate::layer2::bombardment::execute_bombardment_system,\n        crate::layer2::combat::fleet_combat_system,/g' src/simulation.rs
sed -i 's/        world.init_resource::<Events<crate::layer1::logistics::mass_driver::BombardmentEvent>>();/        world.init_resource::<Events<crate::layer1::logistics::mass_driver::BombardmentEvent>>();\n        world.init_resource::<Events<crate::layer2::bombardment::BombardmentEvent>>();/g' src/simulation.rs
cargo check
