sed -i 's/            commands.spawn(/            let mut rng = rand::thread_rng();\n            commands.spawn((/' src/layer1/nature/solar_flare_lottery.rs
sed -i 's/            });/            , GridPosition { x: rng.gen_range(0..100), y: rng.gen_range(0..100) }));/' src/layer1/nature/solar_flare_lottery.rs
