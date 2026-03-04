import os

with open("src/ui/status.rs", "r") as f:
    content = f.read()

# Add DetectionRisk import
import1 = "use crate::layer1::traits::Traits;"
import2 = "use crate::layer1::traits::Traits;\nuse crate::layer3::silence::DetectionRisk;"
content = content.replace(import1, import2)

# Read it in render_status_bar
read1 = "    let admin_stats = world.get_resource::<AdminStats>();"
read2 = """    let admin_stats = world.get_resource::<AdminStats>();
    let detection_risk = world.get_resource::<DetectionRisk>();
    let risk_pct = detection_risk.map_or(0.0, |r| {
        if r.threshold > 0.0 {
            (r.current_risk / r.threshold) * 100.0
        } else {
            0.0
        }
    });"""
content = content.replace(read1, read2)

# Pass it to get_status_line
call1 = "        efficiency,\n        season,\n        solar_cycle,\n    );"
call2 = "        efficiency,\n        season,\n        solar_cycle,\n        risk_pct,\n    );"
content = content.replace(call1, call2)

# Update function signature
sig1 = "    efficiency: f32,\n    season: Option<Season>,\n    solar_cycle: Option<SolarCycle>,\n) -> Line<'a> {"
sig2 = "    efficiency: f32,\n    season: Option<Season>,\n    solar_cycle: Option<SolarCycle>,\n    risk_pct: f32,\n) -> Line<'a> {"
content = content.replace(sig1, sig2)

# Add output to the spans
out1 = "    // 7. Speed"
out2 = """    // 6.5 Detection Risk
    let risk_color = if risk_pct > 80.0 {
        Color::Red
    } else if risk_pct > 50.0 {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    spans.push(Span::styled("Risk: ", Style::default().fg(risk_color)));
    spans.push(Span::styled(
        format!("{risk_pct:.0}% │ "),
        Style::default().fg(Color::White),
    ));

    // 7. Speed"""
content = content.replace(out1, out2)

with open("src/ui/status.rs", "w") as f:
    f.write(content)
