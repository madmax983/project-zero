import re

with open("src/layer1/core/integration.rs", "r") as f:
    content = f.read()

target = """                        if speaker != listener && nostalgia_query.get(listener).is_err() {
                            if rng.gen_bool(0.1) { // 10% chance to proselytize
                                spread_events.send(RumorSpreadEvent {
                                    source: speaker,
                                    target: listener,
                                    rumor: NostalgiaRumor::PastGlory,
                                });
                            }
                        }"""
replacement = """                        if speaker != listener && nostalgia_query.get(listener).is_err() && rng.gen_bool(0.1) {
                            // 10% chance to proselytize
                            spread_events.send(RumorSpreadEvent {
                                source: speaker,
                                target: listener,
                                rumor: NostalgiaRumor::PastGlory,
                            });
                        }"""
content = content.replace(target, replacement)
with open("src/layer1/core/integration.rs", "w") as f:
    f.write(content)
