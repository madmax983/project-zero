# Lexicon

*The vocabulary of SCALE. What mechanics are called in-game.*

---

## Population

### souls

**Replaces:** pops, colonists, population, citizens, people
**Code reference:** `Pop` component
**Usage:**
- "The colony has 47 souls."
- "12 souls lost to the Hunger."
- "A ship carrying 200 souls."

**Note:** Originated in early colonial charters that counted "souls aboard" for life support calculations. It stuck—a reminder that each number is a name.

### founder / founders

**Replaces:** initial colonists, starting population
**Code reference:** First spawned `Pop` entities
**Usage:**
- "The founders of Haven."
- "She was a founder—one of the first five."

### the lost / the fallen

**Replaces:** dead colonists, deaths
**Code reference:** Despawned `Pop` entities
**Usage:**
- "We remember the lost."
- "17 fell to the Hunger."

---

## Needs & Status

### hunger / the Hunger

**Replaces:** food need, starvation
**Code reference:** `Needs.hunger`
**Usage:**
- "Hunger grows in the colony."
- "The Hunger came in year 12." (event name)

**Note:** When capitalized, refers to famine events specifically.

### rest / weariness

**Replaces:** energy, fatigue, tiredness
**Code reference:** `Needs.rest`
**Usage:**
- "The workers need rest."
- "Weariness takes its toll."

### idle / waiting

**Replaces:** unassigned, unemployed
**Code reference:** `PopState::Idle`
**Usage:**
- "5 souls stand idle."
- "The waiting weigh on morale."

### working / at labor

**Replaces:** employed, assigned to job
**Code reference:** `PopState::Working`
**Usage:**
- "12 souls at labor in the fields."

### resting / in shelter

**Replaces:** sleeping, recovering
**Code reference:** `PopState::Resting`
**Usage:**
- "8 souls resting in shelter."

---

## Agency & AI

### the urge / the pull

**Replaces:** utility score, AI desire
**Code reference:** `UtilityAI`
**Usage:**
- "The urge to rest grows."
- "The pull of the fields is strong."

### the weight

**Replaces:** decision making, prioritization
**Code reference:** Action selection logic
**Usage:**
- "The weight of choice."
- "He feels the weight of the harvest."

### the plan

**Replaces:** action queue, HTN plan
**Code reference:** `ActionPlan`
**Usage:**
- "A plan forms."
- "The plan is broken."

---

## Buildings

### works

**Replaces:** buildings (collective)
**Code reference:** Entities with `Building` component
**Usage:**
- "The colony's works grow."
- "New works rise on the eastern ridge."

### shelter / shelters

**Replaces:** housing, homes, residences
**Code reference:** `Housing` component
**Usage:**
- "Build more shelters."
- "The shelters house 20 souls."

### fields

**Replaces:** farms, agricultural buildings
**Code reference:** `Farm` component
**Usage:**
- "The fields yield well this season."
- "We need more hands in the fields."

### beds

**Replaces:** housing capacity, sleeping spots
**Code reference:** `Housing.capacity`
**Usage:**
- "15 beds, 20 souls. The math is grim."

---

## Resources

### stores / the stores

**Replaces:** stockpile, inventory, resources
**Code reference:** `ColonyResources`
**Usage:**
- "Check the stores."
- "The stores run low."

### yield

**Replaces:** food, food supply, rations
**Code reference:** `ColonyResources.food`
**Usage:**
- "This season's yield."
- "Yield: 47 units."

---

## Mining

### the delve / delving

**Replaces:** mining, digging
**Code reference:** `DesignationType::Mine`
**Usage:**
- "The delve goes deep."
- "12 souls delving in the dark."

### stone / the bones

**Replaces:** rock, stone resource
**Code reference:** `TerrainType::Rock`
**Usage:**
- "We build from the bones of the world."
- "Stone for the walls."

### ore / vein

**Replaces:** mineral resources
**Code reference:** Resource items
**Usage:**
- "A rich vein found."
- "Ore for the smelters."

---

## Time

### day / days

**Replaces:** tick, ticks, turns
**Code reference:** `SimulationTime.tick`
**Usage:**
- "Day 142 of the colony."
- "The famine lasted 30 days."

### year

**Replaces:** N/A (larger time unit)
**Code reference:** `tick / 365` (or similar)
**Usage:**
- "In the third year..."
- "Year One: Founding."

### the silence

**Replaces:** pause, paused state
**Code reference:** `SimSpeed::Paused`
**Usage:**
- (Meta) "The silence falls." (when pausing)
- (In-world) "Before the silence..." (historical gaps)

---

## Space & Navigation

### the dark / the void

**Replaces:** space, interstellar space
**Code reference:** Areas without stars/colonies
**Usage:**
- "Ships vanish into the dark."
- "Signals lost to the void."

### foldspace / the fold

**Replaces:** hyperspace, FTL dimension
**Code reference:** Ship transit state
**Usage:**
- "The ship enters fold."
- "Lost in foldspace."

### the Wound

**Replaces:** N/A (unique entity)
**Code reference:** Special region entity
**Usage:**
- "The Wound grows."
- "Too close to the Wound."

**Note:** Always capitalized. The central mystery.

---

## Technology & Objects

### the Substrate

**Replaces:** game UI, player, god-view
**Code reference:** N/A (meta-narrative)
**Usage:** Never directly named in-game. The player IS the Substrate.

**Note:** Ancient computational layer that persists in foldspace. Colonies invoke it for guidance.

### guidance / the mandate

**Replaces:** designations, orders, clicks
**Code reference:** `Designation` component
**Usage:**
- "Awaiting guidance."
- "The mandate is clear: dig here."

### relics / remnants

**Replaces:** artifacts, ancient items
**Code reference:** `Artifact` component
**Usage:**
- "A relic of the Builders."
- "Remnants of a dead age."

### the chronicle / chronicles

**Replaces:** history log, event log
**Code reference:** Chronicle data structure
**Usage:**
- "Recorded in the chronicle."
- "The chronicles speak of..."

---

## Factions & People

### the Kindred

**Replaces:** known civilizations, galactic community
**Code reference:** All `Civilization` entities
**Usage:**
- "The Kindred species."
- "Younger Kindred" (current civs vs. ancient)

### the Builders / the Silent Ones / [EPITHET]

**Replaces:** specific civilization names
**Code reference:** `Civilization.epithet`
**Usage:**
- "The Builders made this."
- "A relic of the Silent Ones."

**Note:** Epithets used more often than proper names, especially for dead civs.

---

## Events

### the Long Hunger

**Replaces:** famine event
**Code reference:** `FAMINE` template
**Usage:**
- "The Long Hunger of Year 12."

### landfall

**Replaces:** colony founding
**Code reference:** `COLONY_FOUNDED` template
**Usage:**
- "Landfall at Kepler-7."
- "The day of landfall."

### first light

**Replaces:** first building completed, colony established
**Usage:**
- "First light at Haven." (first structure raised)

### the return

**Replaces:** ship arrival (especially late)
**Code reference:** `SHIP_RETURNED` template
**Usage:**
- "The return of the Wanderer."

---

## Forestry

### the Green / the Wild

**Replaces:** forests, woods, trees
**Code reference:** `TerrainType::Tree`
**Usage:**
- "The Green encroaches on our walls."
- "Lost in the Wild."

### timber / wood

**Replaces:** wood resource, logs
**Code reference:** `ColonyResources.wood`
**Usage:**
- "Timber for the pyres."
- "We need more wood."

### felling / harvesting

**Replaces:** chopping, cutting trees
**Code reference:** `DesignationType::Chop`
**Usage:**
- "The felling begins at dawn."
- "Harvesting the ancient grove."

### grove / stand

**Replaces:** cluster of trees
**Code reference:** Adjacent `Tree` tiles
**Usage:**
- "A grove of iron-wood."
- "The northern stand must be cleared."

---

## Stockpiles & Logistics

### the hoard / the cache

**Replaces:** stockpile building, storage
**Code reference:** `Stockpile` component
**Usage:**
- "The hoard is full."
- "Hidden in a cache deep underground."

### the take

**Replaces:** resource output, production
**Code reference:** Resource delta
**Usage:**
- "The take is poor this season."
- "A rich yield."

---

## Pop Lifecycle

### arrival

**Replaces:** spawn, immigration
**Code reference:** `spawn_pop`
**Usage:**
- "The arrival of the second wave."
- "New souls arrive."

### departure / the passing

**Replaces:** death, despawn
**Code reference:** `despawn_pop`
**Usage:**
- "We mourn the passing of [NAME]."
- "His departure leaves a void."

---

## UI Text Patterns

### Status Bar
```
▶ Day 142 │ Souls: 47 │ Yield: 23 │ [B]uild
```

### Notifications
```
"The Hunger comes."           (famine starts)
"The Hunger passes."          (famine ends)
"A soul is lost."             (pop dies)
"New works rise."             (building complete)
"Landfall."                   (colony founded)
"Silence."                    (colony lost)
```

### Chronicle Entries
```
"Day 12: Landfall at Haven. 5 founders."
"Day 47: The Long Hunger begins."
"Day 52: 3 souls lost. The Hunger passes."
"Day 100: First fields yield."
```

---

*Update this lexicon as new mechanics are added. Consistency is sacred.*
