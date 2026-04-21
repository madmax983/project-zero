import re

with open("src/layer1/execution/mining.rs", "r") as f:
    content = f.read()

terrain_mock = """
        app.world_mut().insert_resource(crate::layer1::nature::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![crate::layer1::nature::terrain::TerrainType::Rock; 100],
        });

        // Also needs structural integrity RoofGrid
        app.world_mut().insert_resource(crate::layer1::physics::structural_integrity::RoofGrid::new(10, 10));
"""

content = content.replace("app.world_mut().insert_resource(map);\n", "app.world_mut().insert_resource(map);\n" + terrain_mock)

with open("src/layer1/execution/mining.rs", "w") as f:
    f.write(content)
