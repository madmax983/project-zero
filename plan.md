1. *Add fragments to `lore/FRAGMENTS.md`*
   - `cat << 'EOF' >> lore/FRAGMENTS.md

## Petrification Sickness Fragments (Spec 634)

## Fragment Type: [PETRIFICATION_STAGE]
- a slow stiffening of the joints
- graying of the skin
- turning cold as stone
- an unbreakable stillness
- the final hardening

## Fragment Type: [STONE_DESCRIPTORS]
- resembling crude granite
- veins of exotic ore
- polished and silent
- unnaturally heavy
- cold and rigid
EOF`

2. *Verify `lore/FRAGMENTS.md`*
   - `tail -n 20 lore/FRAGMENTS.md`

3. *Add templates to `lore/TEMPLATES.md`*
   - `cat << 'EOF' >> lore/TEMPLATES.md

## Petrification Sickness Templates (Spec 634)

### Template: PETRIFICATION_EXPOSURE
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [PETRIFICATION_STAGE]

**Patterns:**
- "[YEAR]: [NAME] returned from the deep crust mines. They complain of [PETRIFICATION_STAGE]."
- "The sickness begins in [COLONY]. [NAME] is showing signs of [PETRIFICATION_STAGE]. [YEAR]."
- "[NAME] was exposed to the resonant ore. The first symptom is [PETRIFICATION_STAGE]. [YEAR]."

### Template: PETRIFICATION_COMPLETE
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [STONE_DESCRIPTORS]

**Patterns:**
- "[NAME] is gone. They are now just a statue, [STONE_DESCRIPTORS]. [YEAR]."
- "[YEAR]: The transformation of [NAME] is complete. They stand in [COLONY], [STONE_DESCRIPTORS]."
- "We lost [NAME] to the stone. Their form is [STONE_DESCRIPTORS]. [YEAR]."
EOF`

4. *Verify `lore/TEMPLATES.md`*
   - `tail -n 20 lore/TEMPLATES.md`

5. *Add grammars to `lore/GRAMMARS.md`*
   - `cat << 'EOF' >> lore/GRAMMARS.md

## Petrification Sickness Chaining (Spec 634)

- PETRIFICATION_EXPOSURE → enables → MEDICAL_EMERGENCY
- PETRIFICATION_EXPOSURE → increases_chance → UNREST_SPIKE
- PETRIFICATION_COMPLETE → enables → ARTIFACT_CREATION
- PETRIFICATION_COMPLETE → increases_chance → UNREST

## Monuments of Failure Chaining (Spec 167)

- RUIN_CREATED → enables → RUIN_SCAVENGED
- RUIN_CREATED → increases_chance → UNREST
- RUIN_SCAVENGED → enables → NEW_RESOURCE
- RUIN_SCAVENGED → increases_chance → ACCIDENT

## Crop Diversity Chaining (Spec 120)

- FIRST_HARVEST_WHEAT → enables → FOOD_SURPLUS
- FIRST_HARVEST_WHEAT → increases_chance → MORALE_BOOST
- FIRST_HARVEST_POTATO → enables → FAMINE_PREVENTION
- FIRST_HARVEST_POTATO → increases_chance → WINTER_SURVIVAL

## Atmospheric Processors Chaining (Spec 207)

- PROCESSOR_ONLINE → enables → PLANETARY_HEALING
- PROCESSOR_ONLINE → increases_chance → MORALE_BOOST
EOF`

6. *Verify `lore/GRAMMARS.md`*
   - `tail -n 30 lore/GRAMMARS.md`

7. *Add lexicons to `lore/LEXICON.md`*
   - `cat << 'EOF' >> lore/LEXICON.md

## The Petrification Sickness (Spec 634)

### Petrification Sickness
**Replaces:** Disease, turning to stone
**Code reference:** `PetrificationSickness`
**Usage:**
- "The Petrification Sickness claimed three miners today."
- "There is no cure for the Petrification Sickness."

### Living Statue
**Replaces:** Petrified colonist, stone artifact
**Code reference:** `Artifact` from `PetrificationSickness`
**Usage:**
- "We placed the Living Statue in the plaza."
- "The Living Statue still looks like him."

## Monuments of Failure (Spec 167)

### Ruin
**Replaces:** Destroyed building, rubble
**Code reference:** `Ruin`
**Usage:**
- "The fire left behind a Ruin."
- "We cannot build over the Ruin until it is cleared."

### Scavenge
**Replaces:** Demolish ruin, reclaim resources
**Code reference:** `DesignationType::Demolish`
**Usage:**
- "We must Scavenge the old reactor for parts."
- "Scavenging yielded some usable steel."

## Crop Diversity (Spec 120)

### Sun-Grain
**Replaces:** Wheat, standard crop
**Code reference:** `ItemType::Wheat`
**Usage:**
- "The Sun-Grain harvest was plentiful."

### Earth-Apple
**Replaces:** Potato, root crop
**Code reference:** `ItemType::Potato`
**Usage:**
- "We survive on Earth-Apples this winter."

## Atmospheric Processors (Spec 207)

### Air-Forge / The Lung
**Replaces:** Atmospheric processor building
**Code reference:** `BuildingType::AtmosphericProcessor`
**Usage:**
- "The Lung failed today. The air tastes like ash."
- "Build another Air-Forge before we suffocate."
EOF`

8. *Verify `lore/LEXICON.md`*
   - `tail -n 40 lore/LEXICON.md`

9. *Run tests in the background*
   - `cargo test &> test_output.log &`

10. *Monitor the results*
    - `sleep 10 && tail -n 50 test_output.log`

11. *Remove the test artifact*
    - `rm test_output.log`

12. *Prepare git submission*
    - `git checkout -b lore-mechanics-updates`
    - `git add lore/`
    - `git commit -m "lore: add petrification sickness, monuments, crop, and atmospheric processor lore"`

13. *Complete pre commit steps*
    - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

14. *Submit the changes using the submit tool*
    - I will call the `submit` tool to finalize the changes.
