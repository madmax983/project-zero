import os

files_to_update = {
    "specs/218-improvised-tools.md": ("*Builder: Should \"Scrap\" be better than \"Stone\"?*", "*Architect:* Scrap should be more durable but less efficient to work with, making Improvised Tools last longer than primitive Stone ones but take more time to perform tasks."),
    "specs/257-subspace-pen-pals.md": ("*Builder: Can I date a Hive Mind?*", "*Architect:* Yes, but the relationship provides a diplomatic buff for the colony rather than personal romance mechanics."),
    "specs/254-doppelgangers.md": ("*Builder: How does the Mimic spawn?*", "*Architect:* Mimics should spawn rarely from organic waste piles that are left uncleaned for an extended duration."),
    "specs/135-trash-cannon-defense.md": ("*Builder: How much waste creates a \"mess\"?*", "*Architect:* A \"mess\" is created when more than 5 units of waste land on a single tile."),
    "specs/205-auroral-harvesting.md": ("*Builder: Should the collector take damage during the storm?*", "*Architect:* Yes, the collector takes minor durability damage each tick it is actively harvesting during an intense storm."),
    "specs/179-magnetic-storms.md": ("*Builder: Should Magnetic Storm affect Battery discharge rate?*", "*Architect:* Yes, batteries should lose 5% of their stored charge per tick during a Magnetic Storm."),
    "specs/252-tectonic-stress.md": ("*Builder: Does digging a hole reduce stress?*", "*Architect:* Digging significantly reduces tectonic stress locally, transferring a small fraction of that stress to adjacent tiles."),
    "specs/137-pop-hobbies.md": ("*Builder: How do we handle \"Gossip\" hobby mechanically?*", "*Architect:* Gossip spreads memories between pops who interact in the same tile, potentially amplifying mood modifiers."),
    "specs/258-acoustic-shadows.md": ("*Builder: Does glass block sound?*", "*Architect:* Glass reduces sound transmission by 50%, whereas solid walls block it completely."),
    "specs/244-biometric-drift.md": ("*Builder: Does drift affect non-security interactions?*", "*Architect:* No, biometric drift only affects interactions with security scanners and locked doors."),
    "specs/126-blackout-protocol.md": ("*Builder: Should Blackout disable Batteries too?*", "*Architect:* No, batteries remain active but their output is restricted to emergency systems only."),
    "specs/245-quantum-twins.md": ("*Builder: Can twins be separated by Layer?*", "*Architect:* Yes, twins can exist on different layers (e.g., Colony vs. Galaxy), and their entanglement effects persist across layers."),
    "specs/151-cybernetic-augmentation.md": ("*Builder: Should surgery require a Doctor pop to be present?*", "*Architect:* Yes, surgery has a high failure rate unless a pop with the Medical skill is actively operating the facility."),
    "specs/049-industrial-waste.md": ("*Builder: Should Waste decay naturally?*", "*Architect:* Industrial waste does not decay naturally; it must be actively processed or moved."),
    "specs/062-pop-lifecycle.md": ("*Builder: How fast should a \"year\" be?*", "*Architect:* For MVP, a year should be equivalent to 100 simulation ticks."),
    "specs/241-campaign-season.md": ("*Builder: What happens if there are no factions?*", "*Architect:* If no factions exist, campaign events still occur but generate general unrest or loyalty instead of faction-specific shifts."),
    "specs/206-orbital-crossfire.md": ("*Builder: Should impact destroy Pops?*", "*Architect:* Direct impacts instantly kill pops, while near-misses apply a severe shellshock mood penalty."),
    "specs/125-grid-instability.md": ("*Builder: How to visualize Overload?*", "*Architect:* Overloaded nodes should emit a generic warning event that the UI can catch, without needing custom particle effects for MVP."),
    "specs/240-clone-vats.md": ("*Builder: Should clones be born as adults?*", "*Architect:* Yes, clones emerge fully grown but lack any initial skills or memories."),
    "specs/255-hypno-learning.md": ("*Builder: Does the pod require power?*", "*Architect:* The hypno-learning pod consumes power at twice the rate of standard housing while in use."),
    "specs/166-xeno-gastronomy.md": ("*Builder: Should \"Mystery Meals\" stack?*", "*Architect:* No, the effect of a Mystery Meal simply resets the duration of the buff/debuff."),
    "specs/248-infinite-archive.md": ("*Builder: Does deleting a prerequisite tech lock the advanced tech?*", "*Architect:* Deleting a prerequisite tech does not lock the advanced tech if it was already researched, but prevents building new structures that rely on the prerequisite."),
    "specs/220-hygiene-squalor.md": ("*Builder: Should Showers require Power?*", "*Architect:* Showers require Water, but Power is optional; unpowered showers simply provide less hygiene satisfaction."),
    "specs/023-refining-industry.md": ("*Builder: Should the \"10 tile range\" be a constant shared with Mining?*", "*Architect:* Yes, use a shared `INDUSTRY_RANGE_LIMIT` constant for both systems."),
    "specs/181-chemical-regulation.md": ("*Builder: Should withdrawal kill the pop or just incapacitate them?*", "*Architect:* Withdrawal incapacitates the pop, reducing their movement and work speed to 10% until the need is met."),
    "specs/061-cultural-artifacts.md": ("*Builder: Should Art also emit Beauty?*", "*Architect:* Yes, cultural artifacts should emit a Beauty aura that satisfies the Leisure need of nearby pops."),
    "specs/184-orbital-debris.md": ("*Builder: Should debris eventually form a ring system if it gets high enough?*", "*Architect:* That is out of scope for this spec. Treat debris as a localized hazard for now."),
    "specs/211-gut-biome.md": ("*Builder: Should \"Synthetic\" (Rations) cause decay of natural biomes?*", "*Architect:* Yes, a diet strictly of Synthetic Rations causes the natural gut biome to decay over time, increasing vulnerability to illness."),
    "specs/246-legacy-code.md": ("*Builder: Does bloat affect life support?*", "*Architect:* Software bloat only affects automated systems; basic life support is hardwired and immune."),
    "specs/103-private-stashes.md": ("*Builder: Should pops consume their stash?*", "*Architect:* Pops will consume from their private stash first before drawing from the colony's central storage."),
    "specs/106-resource-purity.md": ("*Builder: Should Purity also affect the amount of Stone?*", "*Architect:* Yes, lower purity yields less ore and correspondingly more waste Stone."),
    "specs/247-ghost-code.md": ("*Builder: Does residue stack?*", "*Architect:* No, residue on a specific tile does not stack in intensity, but adjacent residue tiles expand the area of effect."),
    "specs/253-cultural-vandalism.md": ("*Builder: Does vandalism destroy the building?*", "*Architect:* No, it merely appends a `Vandalized` component that flips the building's aura effect until it is cleaned."),
    "specs/219-space-barnacles.md": ("*Builder: Should barnacles fall off if the fleet moves fast enough?*", "*Architect:* No, they must be manually scrubbed off or handled via specific drydock cleaning events."),
    "specs/251-the-empty-room.md": ("*Builder: Does a person standing in the room count as \"not empty\"?*", "*Architect:* Pops do not invalidate the 'Sanctuary' state; only placed buildings or dropped item clutter break the emptiness."),
    "specs/165-gene-banks.md": ("*Builder: Should we allow cloning of Pops?*", "*Architect:* No, Gene Banks only store biological templates; actual Pop cloning is handled by the `Clone Vats` system."),
    "specs/256-placebo-protocols.md": ("*Builder: Can you stack placebos?*", "*Architect:* No, only one placebo effect can be active at a time, verified by the `applied` flag.")
}

for file_path, (question_text, answer_text) in files_to_update.items():
    if not os.path.exists(file_path):
        continue
    with open(file_path, "r") as f:
        content = f.read()

    # Check if answer is already there
    if "*Architect:*" in content and answer_text in content:
        continue

    lines = content.split('\n')
    for i, line in enumerate(lines):
        if question_text in line:
            # Check if answer is already on the next line
            if i + 1 < len(lines) and "*Architect:*" in lines[i+1]:
                break
            lines.insert(i + 1, answer_text)
            break

    with open(file_path, "w") as f:
        f.write('\n'.join(lines))
