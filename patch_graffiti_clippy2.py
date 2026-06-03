with open("src/layer1/graffiti.rs", "r") as f:
    content = f.read()

import re
content = re.sub(r"pub fn graffiti_placement_system\([\s\S]*?pops: Query<\(&GridPosition, &Morale, Option<&crate::layer1::psychology::stress::StressTracker>, Option<&crate::layer1::traits::Traits>\), With<crate::layer1::pop::Pop>>,\n", "type GraffitiPlacementQuery<'a> = (\n    &'a GridPosition,\n    &'a Morale,\n    Option<&'a crate::layer1::psychology::stress::StressTracker>,\n    Option<&'a crate::layer1::traits::Traits>,\n);\n\npub fn graffiti_placement_system(\n    mut graffiti_map: ResMut<GraffitiMap>,\n    pops: Query<GraffitiPlacementQuery, With<crate::layer1::pop::Pop>>,\n", content)

with open("src/layer1/graffiti.rs", "w") as f:
    f.write(content)
