import re

with open("src/layer1/core/integration.rs", "r") as f:
    content = f.read()

# Replace `use crate::layer1::culture::nostalgia::{Nostalgia, Rumor, RumorSpreadEvent};`
# with `use crate::layer1::culture::nostalgia::{Nostalgia, Rumor as NostalgiaRumor, RumorSpreadEvent};`
content = content.replace("use crate::layer1::culture::nostalgia::{Nostalgia, Rumor, RumorSpreadEvent};", "use crate::layer1::culture::nostalgia::{Nostalgia, Rumor as NostalgiaRumor, RumorSpreadEvent};")
content = content.replace("rumor: Rumor::PastGlory,", "rumor: NostalgiaRumor::PastGlory,")

with open("src/layer1/core/integration.rs", "w") as f:
    f.write(content)
