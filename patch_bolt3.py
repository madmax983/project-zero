import re

with open("src/ui/inspector.rs", "r") as f:
    content = f.read()

# Replace 3
search3 = """fn render_personality(frame: &mut Frame, area: Rect, weights: UtilityWeights) {
    if area.height < 2 {
        return;
    }

    let mut traits = Vec::new();

    // Distance
    if weights.distance_weight > 1.2 {"""

replace3 = """fn render_personality(frame: &mut Frame, area: Rect, weights: UtilityWeights) {
    if area.height < 2 {
        return;
    }

    // ⚡ Bolt Optimization: Pre-allocate vector for traits
    let mut traits = Vec::with_capacity(3);

    // Distance
    if weights.distance_weight > 1.2 {"""

content = content.replace(search3, replace3)

with open("src/ui/inspector.rs", "w") as f:
    f.write(content)
