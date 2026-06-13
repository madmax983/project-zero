with open("src/layer1/social/echoes.rs", "r") as f:
    content = f.read()

content = content.replace("use bevy_ecs::prelude::*;\n    use crate::layer1::core::map::GridPosition;", "use crate::layer1::core::map::GridPosition;")
content = content.replace("use crate::layer1::psychology::stress::StressTracker;\n    use std::collections::HashMap;", "use crate::layer1::psychology::stress::StressTracker;")

with open("src/layer1/social/echoes.rs", "w") as f:
    f.write(content)
