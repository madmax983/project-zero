backlog_file = "design/BACKLOG.md"
with open(backlog_file, "r") as f:
    backlog_content = f.read()

new_entries = """
- [ ] `1276` Signal Decay — `specs/1276-signal-decay.md`
- [ ] `1277` Ghost Code — `specs/1277-ghost-code.md`
- [ ] `1278` Asteroid Hollowing — `specs/1278-asteroid-hollowing.md`
- [ ] `1279` Weaponized Tourism — `specs/1279-weaponized-tourism.md`
- [ ] `1280` Tectonic Stress — `specs/1280-tectonic-stress.md`
"""

with open(backlog_file, "w") as f:
    f.write(backlog_content + new_entries)
