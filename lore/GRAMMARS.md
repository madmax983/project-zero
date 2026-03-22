# Grammars

*Rules for how fragments combine, how events chain, and how names generate.*

---

## Name Generation

### Civilization Names

**Structure:** `[PHONEME_PATTERN]` modified by `[CIV_TRAIT]`

**Phoneme Sets:**
```
HARSH   = k, t, r, g, z, v, x      → warlike, aggressive civs
SOFT    = l, m, n, s, w, y, h      → peaceful, diplomatic civs
ANCIENT = apostrophes, vowel clusters, double consonants → old civs
ALIEN   = unusual combinations, q without u, terminal consonants → strange civs
```

**Patterns:**
```
[C][V][C][V]       → Kira, Tova, Masu, Zel'a
[C][V][C][V][C]    → Kirat, Tovar, Masul
[V][C][V][C]       → Orak, Elin, Uvar, Asha
[C][V][C]-[C][V]   → Tor-Ka, Vel-Su, Mar-En (compound)
[C][V]'[C][V][C]   → Ka'vel, To'rin, Ma'sur (ancient style)
```

**Examples by trait:**
```
Aggressive:  Krath, Vorax, Tezrik, Gar-Vel
Peaceful:    Solima, Yaneth, Mirel, Hasu
Ancient:     Ka'thenn, Uu'var, M'reth, Aa'sul
Enigmatic:   Qeth, Xiral, Vzorn, Nth
```

### Star Names

**Structure:** `[STAR_PREFIX]` + `[STAR_SUFFIX]` OR `[CIV_NAME]-derived`

**Generation Rules:**
```
70% — [PREFIX]-[NUMBER]           → Kepler-7, Nova-12
15% — [PREFIX] [SUFFIX]           → Threshold Prime, Echo's Wake
10% — [NAMED_FOR_PERSON/CIV]      → Torval's Star, The Mirel Point
5%  — [DESCRIPTIVE]               → The Burning, Far Silence
```

**Regional Consistency:**
Stars in the same region share naming conventions:
```
Kepler Cluster  → Kepler-1 through Kepler-20
The Marches     → Threshold-*, Vigil-*, Anchor-*
Old Space       → [CIV]-derived names, archaic terms
The Frontier    → Promise-*, Exile-*, Drift-*
```

### Colony Names

**Structure:** Based on `[FOUNDER_CULTURE]` + `[CONTEXT]`

**Patterns:**
```
HOPEFUL     → Haven, Promise, New [Origin], Sanctuary
PRAGMATIC   → Site [Number], [Star]-Colony, Base [Greek Letter]
MEMORIAL    → [Person]'s Landing, [Lost Colony] Remembered
DESCRIPTIVE → Dustbowl, Icehome, The Shelf
DEFIANT     → Last Stand, Exile's End, Spite
```

### Artifact Names

**Structure:** `[ARTIFACT_PREFIX]` + `[ARTIFACT_CORE]`

**Prefixes:**
```
the Last [NOUN]              → the Last Engine
the [ORDINAL] [NOUN]         → the Third Key  
[NAME]'s [NOUN]              → Tovar's Compass
the [ADJECTIVE] [NOUN]       → the Silent Record
the [NOUN] of [PLACE]        → the Beacon of Kepler-7
the [NOUN] That [VERB]       → the Map That Bleeds
```

**Cores by type:**
```
SHIP     → Vessel, Ark, Hull, Drift
WEAPON   → Edge, Voice, Silence, End
RECORD   → Chronicle, Memory, Witness, Echo
KEY      → Key, Gate, Path, Opening
ENGINE   → Engine, Heart, Core, Pulse
```

### Ship Names

**Structure:** `[ADJECTIVE] [NOUN]` OR `[CIV_VALUE]`

**Patterns:**
```
VOID_THEMED   → Void-Walker, Star-Treader, Deep-Diver
AGGRESSIVE    → Iron-Will, Hammer of [Star], Retribution
PEACEFUL      → Silent-Running, Hope's Carrier, The Promise
ABSTRACT      → The [Color] [Noun] (e.g. The Red Echo)
```

---

## Event Chaining

### Causation Rules

Events increase probability of subsequent events:

```yaml
CIVILIZATION_RISE:
  enables:
    - EXPANSION (high)
    - WAR (medium, if neighbors)
    - GOLDEN_AGE (low)
    - ARTIFACT_CREATION (medium)

CIVILIZATION_FALL:
  enables:
    - ERA_TRANSITION (high)
    - ARTIFACT_DISPERSAL (high)
    - RUIN_CREATION (certain)
    - LEGEND_BIRTH (medium)
  
WAR:
  enables:
    - HERO_BIRTH (medium)
    - ARTIFACT_CREATION (medium, war-forged items)
    - CIVILIZATION_FALL (low, if prolonged)
    - TERRITORY_CHANGE (high)
  increases:
    - FAMINE (in affected regions)
    - SHIP_LOST (along contested routes)

CATASTROPHE:
  enables:
    - CIVILIZATION_FALL (high)
    - ERA_TRANSITION (high)
    - LEGEND_BIRTH (medium, survivors)
    - MIGRATION (high)
  creates:
    - WOUND_EXPANSION (if Wound-related)
    - LOST_COLONY (multiple)

FAMINE:
  enables:
    - LEGEND_BIRTH (low, survivor stories)
    - COLONY_LOST (if prolonged)
    - MIGRATION (medium)
  increases:
    - REBELLION (medium)
    - DESPERATE_MEASURES (high)

ARTIFACT_DISCOVERED:
  enables:
    - TECH_ADVANCEMENT (if functional)
    - CULT_FORMATION (if mysterious)
    - WAR (if contested)
  reveals:
    - [ORIGIN_CIV] history entries (previously hidden)

RESOURCE_DISCOVERY:
  enables:
    - INDUSTRIAL_BOOM (high)
    - GREED (medium)
    - ACCIDENT (low)
  increases:
    - MIGRATION (high, gold rush effect)

FIRST_CONTACT:
  enables:
    - TRADE_ESTABLISHED (if peaceful)
    - WAR (if hostile)
    - ALLIANCE (if very peaceful)
    - CULTURAL_EXCHANGE (medium)

RESOURCE_SHORTAGE:
  enables:
    - MIGRATION (medium)
    - UNREST (high)
    - INNOVATION (low)
  increases:
    - BLACK_MARKET (high)
    - HOARDING (high)

SEASON_START:
  enables:
    - FAMINE (high, if Winter)
    - BUMPER_CROP (high, if Autumn)
    - MIGRATION (low, if Spring)
  increases:
    - CONSUMPTION (high, if Winter)
    - MOOD (high, if Spring)

TAVERN_OPENED:
  enables:
    - SOCIAL_GATHERING (high)
    - RUMOR_SPREAD (high)
    - BRAWL (medium)
  increases:
    - HAPPINESS (high)
    - PRODUCTIVITY (low, hangover risk)

SOCIAL_GATHERING:
  enables:
    - RUMOR_SPREAD (very high)
    - LEGEND_BIRTH (low, stories told)
  increases:
    - COHESION (high)
```

### Chronicle Depth Rules

How detailed is the record?

```yaml
RECENT (< 100 years):
  certainty: high
  detail: full
  sources: multiple
  uncertainty_markers: rare

MIDDLE (100-500 years):
  certainty: medium
  detail: partial
  sources: some gaps
  uncertainty_markers: occasional

ANCIENT (500-1000 years):
  certainty: low
  detail: summary only
  sources: fragmentary
  uncertainty_markers: common

PREHISTORIC (> 1000 years):
  certainty: very low
  detail: legendary
  sources: oral tradition only
  uncertainty_markers: frequent
```

Apply `[UNCERTAINTY]` fragments based on era:
```
Recent:     "In year 847, the Mirel..."
Middle:     "Around year 412 (date uncertain), the Mirel..."
Ancient:    "The Mirel arose sometime in the third century (source disputed)..."
Prehistoric: "Legend speaks of the Mirel, who may have (possibly apocryphal)..."
```

---

## History Generation

### World Gen Sequence

Order of operations when generating pre-history:

```
1. GENERATE_WOUND
   - Place the Wound
   - Generate Wound origin mystery
   
2. GENERATE_ANCIENT_CIVS (3-7)
   - For each: CIVILIZATION_RISE
   - Most will CIVILIZATION_FALL
   - One may still exist (player's rivals)
   
3. GENERATE_WARS (based on civ overlaps)
   - Wars between contemporary civs
   - Each war may trigger CATASTROPHE
   
4. GENERATE_ARTIFACTS (based on civs + wars)
   - Each major civ creates 1-3
   - Wars create 1-2
   - Scatter across galaxy
   
5. GENERATE_CATASTROPHES (0-2 major)
   - May end eras
   - May destroy civs
   
6. GENERATE_ERAS (based on transitions)
   - Name periods between major events
   
7. GENERATE_CURRENT_STATE
   - What civs remain?
   - Where are artifacts?
   - What is known vs forgotten?
   
8. PLACE_PLAYER
   - Choose starting location
   - Generate immediate local history
   - Ensure starting area has discoverable content
```

### Minimum History Requirements

Every generated universe MUST have:
```
- 1 Wound (always)
- 2+ civilizations that fell (ruins to find)
- 1+ civilization that remains (rivals/allies)
- 3+ artifacts scattered in space
- 1+ major catastrophe in history
- 1+ legendary figure remembered
- Local history around starting position
```

---

## Connection Rules

### Entity Relationships

How things reference each other:

```yaml
PERSON:
  can_belong_to: [CIVILIZATION, COLONY]
  can_create: [ARTIFACT, BUILDING, SHIP]
  can_participate_in: [WAR, CATASTROPHE, EVENT]
  can_found: [COLONY, DYNASTY, TRADITION]

CIVILIZATION:
  can_create: [ARTIFACT, COLONY, SHIP_CLASS]
  can_participate_in: [WAR, TRADE, ALLIANCE]
  can_occupy: [REGION, STAR_SYSTEM]
  has: [EPITHET, FATE, ACHIEVEMENT]

ARTIFACT:
  has: [CREATOR_CIV, CREATOR_PERSON, ORIGIN_EVENT]
  can_be: [FOUND, LOST, DESTROYED, CHANGED_HANDS]
  can_cause: [WAR, CULT, TECH_ADVANCEMENT]

COLONY:
  belongs_to: [CIVILIZATION or PLAYER]
  can_experience: [FAMINE, GROWTH, DISASTER, CONTACT]
  can_produce: [LEGEND, ARTIFACT, SHIP]
  has: [FOUNDER, HISTORY, POPULATION]
```

### Reference Integrity

When generating:
```
- If artifact references CIV, CIV must exist in chronicle
- If war involves CIV_A and CIV_B, both must have RISE before war DATE
- If person FOUNDS colony, person birth_year < colony found_year
- If artifact is FOUND at colony, artifact exists before find_year
```

---

## Procedural Prose Rules

### Rhythm Variation

Never generate three entries with same structure:
```
BAD:
  "Year 100. The Krath arise."
  "Year 200. The Mirel arise."
  "Year 300. The Solima arise."

GOOD:
  "Year 100. The Krath arise from Kepler-7."
  "From the silence of the Rim, the Mirel emerge. Year 200."
  "The Solima—the Patient—first reach beyond their birth-star in year 300."
```

### Emotional Pacing

Major events get weight:
```
Minor:   Single sentence, terse
Major:   Two sentences, second adds color
Legend:  Three sentences, builds
```

```
Minor:   "Year 412. [COLONY] founded."
Major:   "Year 412. [COLONY] founded. The first footprints in alien soil."
Legend:  "Year 412. [COLONY] founded. The first footprints in alien soil. 
          Founder [NAME] plants the beacon. It still transmits."
```

---

## Location Naming

### Naming Rules
When a significant event happens at `(x,y)`, the location may earn a name.

**Structure:**
- `[EVENT_NOUN] [LOCATION_SUFFIX]`      → Hunger Field, Silence Hill
- `[PERSON]'s [LOCATION_SUFFIX]`        → Tovar's Stand, Kira's Rest
- `[ADJECTIVE] [LOCATION_SUFFIX]`       → Bitter Crossing, Lost Watch

**Triggers:**
- Deaths > 3 in one tile → [LOCATION_SUFFIX: Grave, Fall, End]
- Survivor of event → [PERSON]'s [LOCATION_SUFFIX: Luck, Hope]
- Resource discovery → [RESOURCE] [LOCATION_SUFFIX: Lode, Well]
- Initial landing → [LANDING_NAME]
- Harvest > 100 wood → [FOREST_NAME]
- Mine > 100 stone → [LOCATION_SUFFIX: Delve, Pit, Deep]

---

## Pop Memory

### Memory Formation
Events create memories with `(type, strength, expiration)`.

- **Famine:** Strength 10. Type: Trauma. Text: "Remembering the empty stores."
- **Feast:** Strength 5. Type: Joy. Text: "Remembering the harvest."
- **Death of Friend:** Strength 8. Type: Grief. Text: "Mourning [NAME]."

### Memory Sharing
Pops share memories when:
- Working together
- Resting in same shelter
- Socializing

Shared memories become **Culture**.

---

*These grammars are instructions to the generator. Architect will spec the generator; Builder will implement it.*

---

## Beauty & Waste Chaining

### Beauty Events
When beauty is created, it affects morale and inspiration.

```yaml
PARK_OPENED:
  enables:
    - SOCIAL_GATHERING (high, in park)
    - STATUE_RAISED (medium, as centerpiece)
  increases:
    - HAPPINESS (high)
    - ROMANCE (medium)

STATUE_RAISED:
  enables:
    - LEGEND_BIRTH (medium, regarding subject)
    - VANDALISM (low, if unrest)
  increases:
    - MEMORY_RETENTION (high)
```

### Waste Events
Pollution creates problems.

```yaml
LANDFILL_FULL:
  enables:
    - WASTE_SPILL (high)
    - NEW_LANDFILL_CONSTRUCTION (high)
  increases:
    - SICKNESS (medium)
    - UNREST (low)

WASTE_SPILL:
  enables:
    - SICKNESS_OUTBREAK (high)
    - CLEANUP_EFFORT (high)
    - MUTATION (very low, long term)
  increases:
    - UNREST (high)
    - BEAUTY_LOSS (critical)
```

### Science & Skill Chaining

```yaml
ANOMALY_STUDIED:
  enables:
    - TECH_BREAKTHROUGH (high)
    - ARTIFACT_DISCOVERY (medium)
    - NEW_ANOMALY_SPAWN (low, chain reaction)
  increases:
    - KNOWLEDGE (high)
    - VOID_MADNESS (low, depends on anomaly)

MASTERY_ACHIEVED:
  enables:
    - MASTERWORK_CREATED (high)
    - APPRENTICE_TRAINING (medium)
  increases:
    - EFFICIENCY (high)
    - LEGEND_GENERATION (medium)
```

### Relationship Chaining

```yaml
BOND_FORMED:
  enables:
    - JOINT_ACTION (high)
    - MARRIAGE/UNION (medium)
    - GRIEF_TRAUMA (if one dies)
  increases:
    - STABILITY (medium)

RIVALRY_STARTED:
  enables:
    - BRAWL (medium)
    - SABOTAGE (low)
    - RECONCILIATION (low)
  increases:
    - UNREST (low)
    - STRESS (medium)
```

### Trade Chaining

```yaml
MERCHANT_ARRIVAL:
  enables:
    - TRADE_COMPLETED (high)
    - MERCHANT_DEPARTURE (always)
    - RUMOR_EXCHANGE (medium)
  increases:
    - WEALTH (medium)
    - EXTERNAL_RELATIONS (low)

TRADE_COMPLETED:
  enables:
    - NEW_TECH_AVAILABLE (low, via exotic goods)
    - STOCKPILE_FULL (medium)
  increases:
    - HAPPINESS (low)
```

### Acoustics Chaining

```yaml
NOISE_COMPLAINT:
  enables:
    - INSOMNIA_EPIDEMIC (high)
    - WORK_STOPPAGE (low)
    - BRAWL (medium, due to stress)
  increases:
    - UNREST (medium)
    - STRESS (high)

QUIET_MOMENT:
  enables:
    - REFLECTION (high)
    - ARTISTIC_INSPIRATION (medium)
  increases:
    - MORALE (medium)
```

### Death & Law Chaining

```yaml
FUNERAL_HELD:
  enables:
    - GRIEF_COUNSELING (medium)
    - GRAVE_VISIT (high, recurring)
    - GHOST_SIGHTING (low, if supported)
  increases:
    - MEMORY_RETENTION (high)
    - SOCIAL_BOND (medium)

EDICT_ISSUED:
  enables:
    - PROTEST (low, if unpopular)
    - COMPLIANCE (high)
  increases:
    - STABILITY (medium)
    - AUTHORITY (high)
```

### Vermin Chaining

```yaml
VERMIN_OUTBREAK:
  enables:
    - SPOILAGE_EVENT (high)
    - VERMIN_CLEARED (medium)
  increases:
    - SICKNESS (high)
    - STRESS (medium)

VERMIN_CLEARED:
  enables:
    - STOCKPILE_AUDIT (low)
  increases:
    - MORALE (medium)
    - HYGIENE (high)
```

### Combat & Militia Chaining

```yaml
MILITIA_MUSTER:
  enables:
    - SKIRMISH_RESULT (medium)
    - DRILL_ACCIDENT (low)
  increases:
    - SECURITY (high)
    - UNREST (low, if forced)

SKIRMISH_RESULT:
  enables:
    - FUNERAL_HELD (high, if losses)
    - MILITIA_DISBAND (medium, if victory)
  increases:
    - LEGEND_GENERATION (high, for heroes)
    - TRAUMA (medium)
```

### Fauna Chaining

```yaml
FAUNA_SIGHTING:
  enables:
    - FAUNA_ATTACK (medium)
    - HUNTING_PARTY (medium)
  increases:
    - FEAR (medium)
    - CAUTION (high)

FAUNA_ATTACK:
  enables:
    - INJURY_ACCIDENT (high)
    - MILITIA_MUSTER (high)
  increases:
    - TRAUMA (high)
    - HATRED_OF_WILD (medium)
```

### Structural Chaining

```yaml
STRUCTURE_COLLAPSE:
  enables:
    - INJURY_ACCIDENT (high)
    - REBUILDING_EFFORT (high)
    - INVESTIGATION (medium)
  increases:
    - FEAR (high)
    - CAUTION (high)

RUIN_DISCOVERY:
  enables:
    - ARTIFACT_DISCOVERED (high)
    - TECH_SALVAGE (medium)
    - CURSE_AWAKENING (low)
  reveals:
    - [RUIN_CIV] chronicle entries
```

---

## Energy Chaining

```yaml
POWER_OUTAGE:
  enables:
    - PANIC_ATTACK (medium)
    - SPOILAGE_EVENT (high, fridges fail)
    - ACCIDENT (medium, dark)
  increases:
    - FEAR (high)
    - UNREST (medium)
```

---

## Visitor Chaining

```yaml
VISITOR_ARRIVAL:
  enables:
    - TRADE_OPPORTUNITY (high)
    - DISEASE_OUTBREAK (low)
    - RECRUITMENT (medium)
  increases:
    - CURIOSITY (medium)
    - TENSION (low)
```

---

## Faction Chaining

```yaml
FACTION_FORMED:
  enables:
    - RIVALRY_STARTED (high)
    - DEMAND_ISSUED (medium)
  increases:
    - DIVISION (high)
    - LOYALTY (high, within faction)
```

---

## Atmosphere Chaining

```yaml
ATMOSPHERE_EVENT:
  enables:
    - SICKNESS_OUTBREAK (medium, if toxic)
    - WORK_STOPPAGE (low)
  increases:
    - FEAR (medium)
    - CAUTION (high)
```

---

## Cabin Fever Chaining

```yaml
CONFINEMENT_ALERT:
  enables:
    - CABIN_FEVER_BREAK (high)
    - BRAWL (medium)
  increases:
    - STRESS (high)
    - AGGRESSION (high)

CABIN_FEVER_BREAK:
  enables:
    - ASSAULT (medium)
    - VANDALISM (high)
  increases:
    - FEAR (medium)
    - MORALE_LOSS (high)
```

---

## Weather Chaining

```yaml
WEATHER_EVENT_START:
  enables:
    - WEATHER_DAMAGE (high, if storm)
    - POWER_OUTAGE (medium)
    - CONFINEMENT_ALERT (high)
  increases:
    - FEAR (medium)
    - AWE (low)

WEATHER_DAMAGE:
  enables:
    - REBUILDING_EFFORT (high)
    - RESOURCE_CRISIS (medium, if crops lost)
  increases:
    - TRAUMA (medium)
```

---

## Inspector Chaining

```yaml
INSPECTOR_ARRIVAL:
  enables:
    - INSPECTOR_JUDGMENT (always)
    - CLEANUP_EFFORT (high, panic cleaning)
  increases:
    - ANXIETY (high)
    - PRODUCTIVITY (medium, look busy)

INSPECTOR_JUDGMENT:
  enables:
    - CELEBRATION (if S/A grade)
    - PUNISHMENT (if F grade)
  increases:
    - MORALE (variable)
    - REPUTATION (variable)
```

---

## Stowaway Chaining

```yaml
THEFT_REPORT:
  enables:
    - STOWAWAY_DISCOVERED (medium)
    - PARANOIA (high)
    - RATIONING (low)
  increases:
    - SUSPICION (high)
    - UNREST (medium)

STOWAWAY_DISCOVERED:
  enables:
    - ARREST (high)
    - RECRUITMENT (medium)
  increases:
    - SAFETY (medium)
    - CURIOSITY (low)
```

---

## Mood Chaining

```yaml
PANIC_SPREAD:
  enables:
    - STAMPEDE (low)
    - WORK_STOPPAGE (high)
  increases:
    - CHAOS (high)

JOY_SPREAD:
  enables:
    - CELEBRATION (medium)
    - PRODUCTIVITY_BOOST (high)
  increases:
    - COHESION (high)
```

---

## Omen Chaining

```yaml
OMEN_WITNESSED:
  enables:
    - TABOO_BROKEN (low, fear reaction)
    - RITUAL_PERFORMANCE (high)
  increases:
    - SUPERSTITION (high)
    - ANXIETY (medium)

TABOO_BROKEN:
  enables:
    - ACCIDENT (high, perceived cause)
    - OSTRACISM (medium)
  increases:
    - FEAR (high)
    - DIVISION (medium)
```

---

## Shift Work Chaining

```yaml
SHIFT_CHANGE_DISPUTE:
  enables:
    - WORK_STOPPAGE (medium)
    - BRAWL (low)
  increases:
    - FATIGUE (medium)
    - RESENTMENT (high)
```

## Water Chaining

```yaml
WATER_DISCOVERY:
  enables:
    - FARM_EXPANSION (high)
    - WELL_CONSTRUCTION (high)
    - FLOOD_EVENT (low)
  increases:
    - MORALE (high)
    - GROWTH (medium)

FLOOD_EVENT:
  enables:
    - CROP_FAILURE (high)
    - SICKNESS_OUTBREAK (medium)
  increases:
    - UNREST (medium)
    - FEAR (medium)
```

## Husbandry Chaining

```yaml
ANIMAL_TAMED:
  enables:
    - ANIMAL_BORN (medium, over time)
    - PEN_EXPANSION (high)
    - PREDATOR_ATTACK (medium)
  increases:
    - FOOD_STABILITY (high)

ANIMAL_BORN:
  enables:
    - FEAST (low)
  increases:
    - MORALE (medium)
```

## Aging Chaining

```yaml
ELDER_PASSING:
  enables:
    - FUNERAL_HELD (always)
    - SUCCESSION_CRISIS (low, if leader)
  increases:
    - GRIEF (high)
    - REFLECTION (medium)

CHILD_BORN:
  enables:
    - CELEBRATION (medium)
    - SCHOOL_CONSTRUCTION (low, later)
  increases:
    - HOPE (high)
    - RESOURCE_DRAIN (low)
```

## Sleepwalking Chaining

```yaml
SLEEPWALKER_FOUND:
  enables:
    - ACCIDENT (medium)
    - OMEN_WITNESSED (high, "what did they see?")
  increases:
    - UNEASE (medium)
    - SUPERSTITION (high)
```

## Planetary Quirk Chaining

```yaml
QUIRK_REVEALED:
  enables:
    - TECH_ADAPTATION (high)
    - SPECIALIZED_BUILDING (medium)
  increases:
    - DIFFICULTY_AWARENESS (high)
```

## Private Stash Chaining

```yaml
STASH_FOUND:
  enables:
    - THEFT_REPORT (high, confirming missing items)
    - BRAWL (medium, if owner present)
    - CONFISCATION (high)
  increases:
    - UNREST (medium, jealousy)
    - RELIEF (low, resources recovered)
```

## Fuel Chaining

```yaml
FUEL_PRODUCED:
  enables:
    - POWER_PLANT_CONSTRUCTION (high)
    - REFINERY_ACCIDENT (low, risk starts)
    - POLLUTION_EVENT (medium, smog)
  increases:
    - INDUSTRY_CAPACITY (high)

REFINERY_ACCIDENT:
  enables:
    - FIRE_OUTBREAK (high)
    - INJURY_ACCIDENT (high)
    - SHUTDOWN (medium)
  increases:
    - FEAR_OF_TECH (medium)
```

## Jury-Rigging Chaining

```yaml
JURY_RIG_EVENT:
  enables:
    - JURY_RIG_FAILURE (medium, later)
    - WORK_RESUMED (high)
  increases:
    - ACCIDENT_RISK (high)
    - CONFIDENCE (low)

JURY_RIG_FAILURE:
  enables:
    - STRUCTURE_COLLAPSE (medium)
    - INJURY_ACCIDENT (high)
  increases:
    - FRUSTRATION (high)
```

## Greenhouse Chaining

```yaml
GREENHOUSE_BUILT:
  enables:
    - FIRST_HARVEST (high)
    - JOY_SPREAD (medium, nature view)
    - GLASS_BREAKAGE (low)
  increases:
    - HOPE (high)
    - FOOD_STABILITY (medium)
```

## Provenance Chaining

```yaml
PROVENANCE_REVEALED:
  enables:
    - MEMORY_FORMATION (high)
    - HAUNTING (low, if grim provenance)
    - INSPIRATION (medium, if heroic provenance)
  increases:
    - ATTACHMENT_TO_PLACE (high)
```

---

## Antagonistic Flora Chaining

```yaml
FLORA_OUTBREAK:
  enables:
    - FLORA_CLEARED (high)
    - STRUCTURE_STRANGLED (medium)
    - CROP_LOSS (high, if near farms)
  increases:
    - FEAR (high)
    - MILITIA_MUSTER (medium)

STRUCTURE_STRANGLED:
  enables:
    - STRUCTURE_COLLAPSE (medium)
    - REBUILDING_EFFORT (high)
  increases:
    - HATRED_OF_WILD (high)
```

---

## Observatory Chaining

```yaml
OBSERVATORY_BUILT:
  enables:
    - COSMIC_EPIPHANY (medium)
    - VOID_GAZE (medium)
    - ANOMALY_STUDIED (high)
  increases:
    - SCIENCE_OUTPUT (high)
    - EXISTENTIAL_DREAD (low)

COSMIC_EPIPHANY:
  enables:
    - ARTISTIC_INSPIRATION (high)
    - TECH_BREAKTHROUGH (medium)
  increases:
    - MORALE (high)

VOID_GAZE:
  enables:
    - CABIN_FEVER_BREAK (medium)
    - OMEN_WITNESSED (high)
  increases:
    - STRESS (high)
    - UNREST (low)
```

---

## Mentorship Chaining

```yaml
MENTORSHIP_STARTED:
  enables:
    - LESSON_COMPLETED (high)
    - BOND_FORMED (high)
    - MASTERY_ACHIEVED (medium, eventually)
  increases:
    - SKILL_GAIN (high)
    - COHESION (medium)

LESSON_COMPLETED:
  enables:
    - MASTERWORK_CREATED (low)
    - PROMOTION (medium)
  increases:
    - CONFIDENCE (high)
```

---

## Spontaneous Architecture Chaining

```yaml
FOLLY_RAISED:
  enables:
    - FOLLY_DISCOVERED (always)
    - SOCIAL_GATHERING (medium, if shrine/garden)
    - SECRET_MEETING (low)
  increases:
    - HAPPINESS (high, for builder)
    - CONFUSION (low, for others)

FOLLY_DISCOVERED:
  enables:
    - INSPECTOR_JUDGMENT (medium, if illegal)
    - CELEBRATION (low, if beautiful)
  increases:
    - CURIOSITY (medium)
```

---

## Logistics Chaining

```yaml
LOGISTICS_JAM:
  enables:
    - FLOW_RESTORED (high)
    - WORK_STOPPAGE (high)
    - SPOILAGE_EVENT (medium, if food stuck)
  increases:
    - FRUSTRATION (high)
    - EFFICIENCY_LOSS (high)

FLOW_RESTORED:
  enables:
    - STOCKPILE_FULL (low, sudden influx)
  increases:
    - RELIEF (medium)
```

---

## Retrograde Chaining

```yaml
TECH_DECONSTRUCTED:
  enables:
    - TECH_BREAKTHROUGH (high)
    - FLAW_DISCOVERED (medium)
    - COMPONENT_RECOVERY (high)
  increases:
    - KNOWLEDGE (high)
    - WASTE (medium, scrap)

FLAW_DISCOVERED:
  enables:
    - RETROFIT_PROJECT (high)
    - ACCIDENT_PREVENTION (high)
  increases:
    - CAUTION (medium)
    - TRUST_IN_LEADERSHIP (low)
```

---

## Penal Labor Chaining

```yaml
PRISONER_ARRIVED:
  enables:
    - PRISON_RIOT (low, if morale low)
    - SENTENCE_SERVED (always, eventually)
    - ESCAPE_ATTEMPT (medium)
  increases:
    - WORKFORCE (high)
    - TENSION (medium)

SENTENCE_SERVED:
  enables:
    - FULL_CITIZENSHIP (always)
    - CELEBRATION (low)
  increases:
    - HOPE (high)
    - LOYALTY (high)

PRISON_RIOT:
  enables:
    - MILITIA_MUSTER (high)
    - LOCKDOWN (high)
    - NEGOTIATION (medium)
  increases:
    - FEAR (high)
    - INJURY_COUNT (medium)
```

---

## Technological Ritual Chaining

```yaml
RITUAL_PERFORMED:
  enables:
    - SPIRIT_APPEASED (high)
    - SPIRIT_ANGERED (low, if botched)
    - EFFICIENCY_BOOST (medium)
  increases:
    - SUPERSTITION (high)
    - CONFIDENCE (medium)

SPIRIT_APPEASED:
  enables:
    - PRODUCTION_BONUS (high)
    - ACCIDENT_AVOIDANCE (high)
  increases:
    - MORALE (medium)

SPIRIT_ANGERED:
  enables:
    - MACHINE_MALFUNCTION (high)
    - ACCIDENT (medium)
    - OMEN_WITNESSED (high)
  increases:
    - FEAR (high)
    - CAUTION (high)
```

---

## Social Mimicry Chaining

```yaml
TREND_STARTED:
  enables:
    - RESOURCE_SHORTAGE (medium, of trend item)
    - TREND_DIED (always, eventually)
    - CLIQUE_FORMATION (low)
  increases:
    - COHESION (high)
    - CONSUMPTION (medium)

TREND_DIED:
  enables:
    - NEW_TREND (medium)
    - STOCKPILE_GLUT (low, unused items)
  increases:
    - BOREDOM (low)
```

---

## Colony Mascot Chaining

```yaml
MASCOT_NAMED:
  enables:
    - MASCOT_EVENT (high, recurring)
    - MASCOT_DEATH (eventually)
  increases:
    - MORALE (high)
    - ATTACHMENT (high)

MASCOT_EVENT:
  enables:
    - JOY_SPREAD (high)
    - RUMOR_SPREAD (medium, cute stories)
  increases:
    - HAPPINESS (high)
    - STRESS_REDUCTION (high)

MASCOT_DEATH:
  enables:
    - FUNERAL_HELD (high)
    - STATUE_RAISED (medium)
  increases:
    - GRIEF (high)
    - UNITY (medium)
```

## Security Chaining

```yaml
ACCESS_DENIED:
  enables:
    - HACKING_ATTEMPT (medium)
    - WORK_DELAY (high)
  increases:
    - FRUSTRATION (high)

LOCKOUT_OVERRIDE:
  enables:
    - SECURITY_ALERT (high)
    - INSPECTOR_JUDGMENT (medium, if illegal)
  increases:
    - SUSPICION (medium)
```

## Old Guard Chaining

```yaml
GENERATION_CLASH:
  enables:
    - DEMAND_ISSUED (high, from Old Guard)
    - FACTION_FORMED (medium, New Blood)
  increases:
    - DIVISION (high)

TRADITION_UPHELD:
  enables:
    - CELEBRATION (medium)
    - RESENTMENT (low, from New Blood)
  increases:
    - STABILITY (medium)
```

## Vacuum Chaining

```yaml
EMERGENCY_VENT:
  enables:
    - FIRE_EXTINGUISHED (high)
    - INJURY_ACCIDENT (medium)
    - ITEM_LOSS (high, sucked out)
  increases:
    - FEAR (high)
    - RELIEF (medium, if fire gone)

HULL_BREACH:
  enables:
    - DECOMPRESSION_EVENT (always)
    - REPAIR_RUSH (high)
  increases:
    - PANIC (high)
```

## Trash Cannon Chaining

```yaml
CANNON_FIRED:
  enables:
    - THREAT_DESTROYED (medium)
    - WASTE_REDUCTION (high)
    - ACCIDENTAL_HIT (low)
  increases:
    - SECURITY (medium)
    - MORALE (low, "taking out the trash")

AMMO_DEPLETED:
  enables:
    - CANNON_FIRED (impossible until refilled)
    - THREAT_IGNORED (high)
  increases:
    - FEAR (high)
    - URGENCY (high)
```

## Flora Chaining

GLOW_DISCOVERED:
  enables:
    - RESEARCH_BREAKTHROUGH (high)
    - GARDEN_CONSTRUCTION (medium)
  increases:
    - WONDER (high)
    - LIGHT_LEVEL (medium)

## Grid Chaining

GRID_SURGE:
  enables:
    - FIRE_OUTBREAK (high)
    - MACHINE_DAMAGE (medium)
    - BLACKOUT (low)
  increases:
    - FEAR (medium)
    - MAINTENANCE_DEBT (high)

BROWNOUT:
  enables:
    - WORK_STOPPAGE (high)
    - SPOILAGE_EVENT (medium, if fridges fail)
  increases:
    - FRUSTRATION (high)
    - EFFICIENCY_LOSS (high)


---


---

## Wild Child Chaining

```yaml
WILD_CHILD_FOUND:
  enables:
    - CHILD_RECOVERED (high, if housed)
    - CHILD_GOES_FERAL (high, if left outside)
    - RUMOR_SPREAD (medium)
  increases:
    - CURIOSITY (medium)
    - PITY (medium)

CHILD_GOES_FERAL:
  enables:
    - THEFT_REPORT (high, stealing food)
    - FAUNA_SIGHTING (medium, running with beasts)
  increases:
    - FEAR (medium)
    - SADNESS (high)
```

## Blob Chaining

```yaml
BLOB_SIGHTING:
  enables:
    - BLOB_CONSUMPTION (high)
    - BLOB_DAMAGE (medium)
    - MILITIA_MUSTER (high)
  increases:
    - FEAR (high)
    - URGENCY (high)

BLOB_DAMAGE:
  enables:
    - STRUCTURE_COLLAPSE (high)
    - REBUILDING_EFFORT (high)
  increases:
    - HATRED_OF_WILD (high)
```

## Cybernetics Chaining

```yaml
SURGERY_COMPLETED:
  enables:
    - EFFICIENCY_BOOST (always)
    - SOCIAL_ISOLATION (medium, due to penalty)
    - NEW_SURGERY (low, addiction)
  increases:
    - PRODUCTIVITY (high)
    - ALIENATION (medium)

SURGERY_FAILED:
  enables:
    - INJURY_ACCIDENT (medium)
    - MEDICAL_RITES (high)
  increases:
    - FEAR_OF_TECH (medium)
```

## Heirloom Chaining

```yaml
HEIRLOOM_CREATED:
  enables:
    - LEGEND_BIRTH (high)
    - THEFT_REPORT (low, valuable item)
  increases:
    - MORALE (high)
    - TRADITION (high)

ANCIENT_DECAY:
  enables:
    - RETROGRADE_SACRIFICE (medium, save the data)
    - STRUCTURE_COLLAPSE (high, if ignored)
  increases:
    - URGENCY (medium)
    - LOSS (medium)

RETROGRADE_SACRIFICE:
  enables:
    - KNOWLEDGE_BREAKTHROUGH (high)
    - MEMORY_FRAGMENT (medium)
  increases:
    - KNOWLEDGE (high)
    - REGRET (low)
```

## Social Stratification Chaining

```yaml
CLASS_FRICTION_EVENT:
  enables:
    - BRAWL (high)
    - EDICT_ISSUED (medium, crackdown)
    - STRIKE (medium)
  increases:
    - UNREST (high)
    - DIVISION (high)

SOCIAL_PROMOTION:
  enables:
    - CELEBRATION (low)
    - RESENTMENT (medium, from former peers)
  increases:
    - AMBITION (medium)
```

---

## Justice & Sanctuary Chaining

```yaml
SANCTUARY_DECLARED:
  enables:
    - CRIMINAL_FLIGHT (high)
    - BLACK_MARKET_ACTIVITY (medium)
  increases:
    - CRIME_RATE (medium, localized)
    - STABILITY (medium, pressure valve)

CRIMINAL_FLIGHT:
  enables:
    - WANTED_ESCAPE (high)
    - WARDEN_FRUSTRATION (medium)
  increases:
    - AUTHORITY_LOSS (low)
```

## Tech Envy Chaining

```yaml
TECH_ENVY_COMPLAINT:
  enables:
    - DEMAND_ISSUED (high)
    - THEFT_REPORT (medium, stealing upgrades)
    - WORK_SLOWDOWN (high)
  increases:
    - UNREST (medium)
    - GREED (high)
```

## Ecological Succession Chaining

```yaml
SUCCESSION_STAGE:
  enables:
    - FLORA_OUTBREAK (medium)
    - STRUCTURE_STRANGLED (low, if ignored)
    - BEAUTY_GAIN (medium, nature reclaim)
  increases:
    - MAINTENANCE_DEBT (medium)
```

## Xeno-Artifact Chaining

```yaml
ARTIFACT_AURA_FELT:
  enables:
    - MOOD_WAVE (high)
    - OMEN_WITNESSED (medium)
    - VOID_GAZE (low)
  increases:
    - WONDER (medium)
    - DREAD (medium)
```

## Vermin Evolution Chaining

```yaml
VERMIN_EVOLVED:
  enables:
    - INJURY_ACCIDENT (high, if toxic/volatile)
    - SPOILAGE_EVENT (high)
    - MILITIA_MUSTER (high)
  increases:
    - FEAR (high)
    - URGENCY (high)
```
```

---

## Drone Chaining

```yaml
DRONE_ACTIVATED:
  enables:
    - DRONE_MALFUNCTION (medium, over time)
    - WORK_EFFICIENCY (high)
  increases:
    - TECH_DEPENDENCE (medium)

DRONE_MALFUNCTION:
  enables:
    - LOGISTICS_JAM (high)
    - RETROGRADE_SACRIFICE (low, for parts)
  increases:
    - FRUSTRATION (medium)
```

## Graffiti Chaining

```yaml
GRAFFITI_SPOTTED:
  enables:
    - INSPECTOR_JUDGMENT (medium, negative)
    - FACTION_FORMED (low, if message resonates)
    - CLEANUP_EFFORT (high)
  increases:
    - UNREST (medium)
    - CURIOSITY (low)
```

## Cannibalization Chaining

```yaml
SHIP_PART_SALVAGED:
  enables:
    - RESOURCE_GAIN (high)
    - SHIP_GONE (eventually)
    - MEMORY_FORMATION (high, sadness)
  increases:
    - SURVIVAL_CHANCE (high)
    - NOSTALGIA (medium)

SHIP_GONE:
  enables:
    - FULL_ACCEPTANCE (medium, "we are here now")
  increases:
    - ATTACHMENT_TO_COLONY (high)
```

## Geological Chaining

```yaml
SEISMIC_TREMOR:
  enables:
    - STRUCTURE_COLLAPSE (medium)
    - PANIC_SPREAD (high)
    - CAVERN_DISCOVERY (low)
  increases:
    - FEAR (high)
    - CAUTION (high)
```

## Thermal Chaining

```yaml
HEAT_SPIKE:
  enables:
    - FIRE_OUTBREAK (high)
    - MACHINE_MALFUNCTION (medium)
    - WORK_STOPPAGE (high)
  increases:
    - STRESS (high)
    - THIRST (high)

FREEZE_EVENT:
  enables:
    - CROP_FAILURE (high)
    - HYPOTHERMIA_ACCIDENT (high)
    - POWER_DRAIN (high, heating)
  increases:
    - DESPAIR (medium)
```

## Data Chaining

```yaml
DATA_FOUND:
  enables:
    - TECH_BREAKTHROUGH (medium)
    - SECRET_REVEALED (high)
    - RUMOR_SPREAD (high)
  increases:
    - KNOWLEDGE (high)
    - PARANOIA (low)
```

## Social Debt Chaining

```yaml
FAVOR_CALLED:
  enables:
    - WORK_ASSIST (high)
    - ITEM_TRANSFER (medium)
    - REFUSAL (low, creates rivalry)
  increases:
    - COHESION (medium)
    - RESENTMENT (low)
```

---

## Chemical Regulation Chaining

```yaml
ADDICTION_CRISIS:
  enables:
    - THEFT_REPORT (high, for fix)
    - OVERDOSE (medium)
    - CRIME_WAVE (high, if widespread)
  increases:
    - DESPERATION (high)
    - UNREST (medium)

OVERDOSE:
  enables:
    - FUNERAL_HELD (high)
    - EDICT_ISSUED (medium, banning chem)
  increases:
    - GRIEF (high)
    - FEAR_OF_TECH (low)
```

## Wind & Canyon Chaining

```yaml
HIGH_WIND_EVENT:
  enables:
    - POWER_SURGE (high, wind turbines)
    - STRUCTURAL_DAMAGE (medium)
    - MOVEMENT_PENALTY (always)
  increases:
    - NOISE_COMPLAINT (high)

CANYON_FORMED:
  enables:
    - WIND_TUNNEL_EFFECT (always)
    - COOLING_BONUS (medium)
  increases:
    - WALKING_DIFFICULTY (high)
```

## Geodetic Sentience Chaining

```yaml
STONE_MIGRATION:
  enables:
    - GOLEM_RISES (high, if unchecked)
    - MINER_SCARED (medium)
  increases:
    - PARANOIA (medium)
    - MYSTERY (high)

GOLEM_RISES:
  enables:
    - MILITIA_MUSTER (high)
    - STOCKPILE_DESTRUCTION (high)
    - MINING_HALT (always)
  increases:
    - FEAR (high)
    - AWE (medium)
```

## Orbital Debris Chaining

```yaml
LAUNCH_FAILURE_DEBRIS:
  enables:
    - ORBITAL_IMPACT (medium)
    - KESSLER_WARNING (high)
  increases:
    - ISOLATION (medium)
    - LAUNCH_COST (high)

ORBITAL_IMPACT:
  enables:
    - STATION_DAMAGE (high)
    - SHIELD_BREACH (medium)
  increases:
    - FEAR_OF_SKY (medium)
```

## Vacuum Welding Chaining

```yaml
STRUCTURE_WELDED:
  enables:
    - DESTROY_DESIGNATION (only way to remove)
    - DURABILITY_BONUS (always)
  increases:
    - PERMANENCE (high)
    - REGRET (low, if misplaced)

DESTROY_DESIGNATION:
  enables:
    - RESOURCE_LOSS (always, 100%)
    - EXPLOSION_EVENT (low)
  increases:
    - WASTE (high)
```

## Bio-Architecture Chaining

```yaml
BIO_STRUCTURE_GROWN:
  enables:
    - BIO_STARVATION (medium, if neglected)
    - PULSE_DOOR_OPEN (high)
  increases:
    - WONDER (high)
    - UPKEEP_DEMAND (high, food)

BIO_STARVATION:
  enables:
    - BIO_INFECTION (high)
    - STRUCTURE_DECAY (high)
  increases:
    - GUILT (medium)
    - DISGUST (low)

BIO_INFECTION:
  enables:
    - STRUCTURE_ATTACK (high, hostile building)
    - QUARANTINE (medium)
    - FLAMETHROWER_USE (high)
  increases:
    - TERROR (high)
    - SICKNESS (medium)
```

## Cryo-Dreams Chaining

```yaml
CRYO_WAKE_EPIPHANY:
  enables:
    - TECH_BREAKTHROUGH (high)
    - NEW_TRADITION (medium)
  increases:
    - KNOWLEDGE (high)
    - MORALE (medium)

CRYO_WAKE_NIGHTMARE:
  enables:
    - CABIN_FEVER_BREAK (medium)
    - OMEN_WITNESSED (high, "they brought it back")
    - WORK_STOPPAGE (low)
  increases:
    - FEAR (high)
    - TRAUMA (high)
```

## Gastronomy Chaining

```yaml
MYSTERY_MEAL_COOKED:
  enables:
    - FOOD_POISONING (medium, risk)
    - RECIPE_MASTERED (medium, reward)
    - XENO_DELICACY (low, critical success)
  increases:
    - CURIOSITY (medium)
    - CAUTION (medium)

RECIPE_MASTERED:
  enables:
    - FEAST (high)
    - TRADE_GOOD_EXPORT (medium)
  increases:
    - MORALE (high)
    - HEALTH (medium)

FOOD_POISONING:
  enables:
    - WORK_STOPPAGE (medium)
    - MEDICAL_EMERGENCY (low)
  increases:
    - SICKNESS (high)
    - DISTRUST_OF_COOK (high)
```

## Atmospheric Chaining

```yaml
TIDE_HIGH:
  enables:
    - MOVEMENT_PENALTY (always)
    - WIND_POWER_BOOST (high)
    - COMPRESSOR_FAILURE (low)
  increases:
    - FATIGUE (medium)
    - NOISE_COMPLAINT (high)

TIDE_LOW:
  enables:
    - MOVEMENT_BOOST (always)
    - WIND_POWER_DROP (high)
    - VENTILATION_ISSUE (medium)
  increases:
    - RELIEF (medium)
    - DIZZINESS (low)
```

## Festival Chaining

```yaml
FESTIVAL_START:
  enables:
    - WORK_STOPPAGE (high, intended)
    - SOCIAL_BONDING (high)
    - RUMOR_SPREAD (high)
    - BRAWL (low, too much brew)
  increases:
    - MORALE (high)
    - UNITY (high)

FESTIVAL_END:
  enables:
    - CLEANUP_DUTY (high)
    - POST_FESTIVAL_BLUES (low)
  increases:
    - MEMORY_RETENTION (high)
```

## Radiation Chaining

```yaml
RADIATION_SICKNESS_DETECTED:
  enables:
    - MEDICAL_TREATMENT (high)
    - EVACUATION (medium)
    - SHIELDING_CONSTRUCTION (high)
  increases:
    - FEAR_OF_INVISIBLE (high)
    - HEALTH_LOSS (high)

WARM_STONE_REFUGE:
  enables:
    - SICKNESS_OUTBREAK (medium, long term)
    - HYPOTHERMIA_AVOIDED (high, short term)
  increases:
    - COMFORT (medium, misleading)
    - DANGER (high, hidden)
```

## Crop Chaining

```yaml
FIRST_HARVEST_WHEAT:
  enables:
    - BAKING (high)
    - ALE_BREWING (medium)
  increases:
    - FOOD_STABILITY (medium)

FIRST_HARVEST_POTATO:
  enables:
    - WINTER_SURVIVAL (high)
    - VODKA_BREWING (low)
  increases:
    - SECURITY (high)

FIRST_HARVEST_RICE:
  enables:
    - SUSHI_PREPARATION (low, xeno-fish)
    - SAKE_BREWING (medium)
  increases:
    - FOOD_STABILITY (high)
```

---

## Auroral Chaining

```yaml
AURORA_SIGHTING:
  enables:
    - AURORA_HARVEST (high, if collectors active)
    - POWER_SURGE (medium)
  increases:
    - AWE (high)
    - ENERGY_STOCKS (high)

AURORA_HARVEST:
  enables:
    - BATTERY_OVERLOAD (low)
    - CELEBRATION (medium)
  increases:
    - WEALTH (medium)
```

## Predictive Policing Chaining

```yaml
CRIME_PREDICTED:
  enables:
    - PREEMPTIVE_ARREST (high)
    - FALSE_POSITIVE (low, generates unrest)
  increases:
    - SECURITY (high)
    - PARANOIA (medium)

PREEMPTIVE_ARREST:
  enables:
    - PROTEST (medium)
    - WORK_CONTINUATION (high, no crime downtime)
  increases:
    - AUTHORITY (high)
    - RESENTMENT (medium)
```

## Scrapcode Chaining

```yaml
SCRAPCODE_INFECTION:
  enables:
    - MACHINE_MALFUNCTION (high)
    - DATA_LOSS (medium)
    - RETROGRADE_SACRIFICE (high, purge system)
  increases:
    - CONFUSION (high)
    - MAINTENANCE_DEBT (high)
```

## Totem Chaining

```yaml
TOTEM_CRAFTED:
  enables:
    - TOTEM_LOST (medium, later)
    - BRAVERY_BOOST (high)
  increases:
    - SUPERSTITION (medium)
    - MORALE (high)

TOTEM_LOST:
  enables:
    - OMEN_WITNESSED (high, psychological)
    - PANIC_ATTACK (medium)
  increases:
    - FEAR (high)
```

## Food Preservation Chaining

```yaml
RATIONS_PRESERVED:
  enables:
    - WINTER_SURVIVAL (high)
    - TRADE_EXPORT (medium)
  increases:
    - FOOD_STABILITY (high)
    - SPOILAGE_RESISTANCE (high)
```

## Institutional Memory Chaining

```yaml
ARCHIVE_DISCOVERY:
  enables:
    - LOST_KNOWLEDGE_RECOVERED (high)
    - MYSTERY_SOLVED (medium)
  increases:
    - KNOWLEDGE (high)
    - CULTURAL_DEPTH (high)

LOST_KNOWLEDGE_RECOVERED:
  enables:
    - TECH_BREAKTHROUGH (high)
    - ANCIENT_RITUAL (medium)
  increases:
    - CONFIDENCE (high)
```

---

## Escape Pod Chaining

```yaml
ESCAPE_POD_LAUNCH:
  enables:
    - COLONY_EVACUATED (medium, if mass launch)
    - SHIP_LOST (low, if pod fails)
    - SURVIVOR_FOUND (medium, later elsewhere)
  increases:
    - POP_LOSS (high)
    - GRIEF (high)

COLONY_EVACUATED:
  enables:
    - RUIN_CREATION (always)
    - LEGEND_BIRTH (medium, "The Lost Colony")
  increases:
    - MYSTERY (high)
```

## Planetary Core Tap Chaining

```yaml
CORE_TAP_ACTIVATED:
  enables:
    - CORE_STRESS_WARNING (high, over time)
    - SEISMIC_TREMOR (medium)
    - UNLIMITED_POWER (always)
  increases:
    - INDUSTRY_CAPACITY (maximum)
    - DANGER (high)

CORE_STRESS_WARNING:
  enables:
    - SEISMIC_TREMOR (high)
    - STRUCTURE_COLLAPSE (medium)
    - EVACUATION_PLANNING (high)
  increases:
    - FEAR (high)
    - URGENCY (high)
```

## Solar Cycle Chaining

```yaml
SOLAR_CYCLE_CHANGE:
  enables:
    - POWER_FLUCTUATION (high, if solar dependent)
    - CROP_GROWTH_CHANGE (medium)
  increases:
    - SEASONAL_AWARENESS (medium)
```

## Customs Chaining

```yaml
CONTRABAND_SEIZED:
  enables:
    - ARREST (high)
    - BLACK_MARKET_DIP (medium)
    - REVENGE_ATTEMPT (low)
  increases:
    - SECURITY (medium)
    - TENSION (low)

VISITOR_DENIED:
  enables:
    - ANGRY_DEPARTURE (always)
    - RUMOR_SPREAD (low, "they turn people away")
  increases:
    - ISOLATION (low)
    - SAFETY (medium)
```

## Ammunition Chaining

```yaml
AMMO_SHORTAGE:
  enables:
    - DEFENSE_FAILURE (high, if attacked)
    - FABRICATION_RUSH (high)
    - TURRET_SILENCE (always)
  increases:
    - VULNERABILITY (high)
    - FEAR (medium)

TURRET_RELOADED:
  enables:
    - DEFENSE_READY (always)
  increases:
    - CONFIDENCE (medium)
```

## Planetary Governance Chaining

```yaml
GOVERNOR_APPOINTED:
  enables:
    - POLICY_ENACTED (high)
    - FACTION_RESPONSE (medium)
  increases:
    - STABILITY (medium)
    - BUREAUCRACY (medium)

POLICY_ENACTED:
  enables:
    - COMPLIANCE (high)
    - PROTEST (low, if unpopular)
  increases:
    - ORDER (medium)
```

## Safehouse Chaining

```yaml
SAFEHOUSE_CONTRACT:
  enables:
    - VISITOR_HIDDEN (always)
    - INSPECTOR_SUSPICION (medium)
    - PAYMENT_RECEIVED (high)
  increases:
    - WEALTH (medium)
    - RISK (medium)
```

---

## Hygiene Chaining

```yaml
FILTH_OUTBREAK:
  enables:
    - SICKNESS_OUTBREAK (high)
    - VERMIN_OUTBREAK (high)
    - SHOWER_BUILT (medium)
  increases:
    - DISGUST (high)
    - UNREST (medium)

SHOWER_BUILT:
  enables:
    - HYGIENE_BOOST (high)
    - WATER_CONSUMPTION_SPIKE (always)
  increases:
    - MORALE (medium)
    - HEALTH (medium)
```

## Recycling Chaining

```yaml
RECYCLER_OPERATIONAL:
  enables:
    - CORPSE_RECYCLED (high, if deaths)
    - WASTE_REDUCTION (high)
  increases:
    - EFFICIENCY (high)
    - DISCOMFORT (low, smell)

CORPSE_RECYCLED:
  enables:
    - RATION_BOOST (medium)
    - FUNERAL_SKIPPED (always)
  increases:
    - PRAGMATISM (high)
    - HORROR (medium, for non-pragmatists)
```

## Fleet Chaining

```yaml
SHIP_CONSTRUCTED:
  enables:
    - FLEET_ASSIGNMENT (high)
    - SHIP_LAUNCH (always)
  increases:
    - MILITARY_POWER (high)
    - PRIDE (medium)

FLEET_ENGAGEMENT:
  enables:
    - SHIP_LOST (medium)
    - HERO_BIRTH (low)
    - REPAIR_RUSH (high, if survivors)
  increases:
    - TENSION (high)
    - LEGEND_GENERATION (medium)
```

## Barnacle Chaining

```yaml
BARNACLE_INFESTATION:
  enables:
    - DRAG_WARNING (high)
    - HULL_DAMAGE (medium)
    - CLEANING_DUTY (high)
  increases:
    - FUEL_CONSUMPTION (high)
    - FRUSTRATION (medium)

DRAG_WARNING:
  enables:
    - SHIP_DELAYED (high)
    - FUEL_CRISIS (medium)
  increases:
    - URGENCY (high)
```

## Crossfire Chaining

```yaml
ORBITAL_BOMBARDMENT:
  enables:
    - CROSSFIRE_HIT (high)
    - SHIELD_FAILURE (medium)
    - PANIC_SPREAD (high)
  increases:
    - FEAR_OF_SKY (high)
    - DESTRUCTION (high)

CROSSFIRE_HIT:
  enables:
    - STRUCTURE_COLLAPSE (always)
    - FIRE_OUTBREAK (high)
    - REBUILDING_EFFORT (medium)
  increases:
    - ANGER (high)
    - TRAUMA (high)
```

## Mother Lode Chaining

```yaml
MOTHER_LODE_FOUND:
  enables:
    - INDUSTRIAL_BOOM (high)
    - MINE_EXPANSION (always)
    - GREED (medium)
  increases:
    - WEALTH (high)
    - ACCIDENT_RISK (medium, deep mining)

MOTHER_LODE_DEPLETED:
  enables:
    - ECONOMIC_CRASH (medium)
    - MIGRATION (low)
  increases:
    - DESPAIR (medium)
    - NOSTALGIA (high)
```

## Heat Island Chaining

```yaml
HEAT_ISLAND_WARNING:
  enables:
    - SICKNESS_OUTBREAK (medium, heatstroke)
    - WORK_STOPPAGE (low)
    - COOLING_DEMAND (high)
  increases:
    - DISCOMFORT (high)
    - ENERGY_USE (high, cooling)
```

## Pneumatic Chaining

```yaml
TUBE_JAM:
  enables:
    - LOGISTICS_JAM (high)
    - WORK_STOPPAGE (medium)
    - EXPLOSION_EVENT (low, if pressure builds)
  increases:
    - FRUSTRATION (high)
    - EFFICIENCY_LOSS (high)
```

## Keystone Chaining

```yaml
KEYSTONE_DEATH:
  enables:
    - ECOSYSTEM_COLLAPSE (always)
    - FAMINE (high)
    - PREDATOR_ATTACK (medium, desperate animals)
  increases:
    - DESPAIR (high)
    - RESOURCE_SCARCITY (high)

ECOSYSTEM_COLLAPSE:
  enables:
    - MIGRATION (high, animals leave)
    - DUST_BOWL (medium)
  increases:
    - SURVIVAL_DIFFICULTY (high)
```

## Gene Bank Chaining

```yaml
SAMPLE_DEGRADED:
  enables:
    - EXTINCTION_EVENT (high, if last sample)
    - RESEARCH_SETBACK (medium)
  increases:
    - REGRET (medium)

ANCIENT_DNA_FOUND:
  enables:
    - CLONING_PROJECT (high)
    - MYSTERY_MEAL_COOKED (low, if edible)
  increases:
    - HOPE (high)
    - SCIENCE_XP (high)
```

## Paperwork Chaining

```yaml
PAPERWORK_LOST:
  enables:
    - WORK_STOPPAGE (high)
    - FORM_REJECTED (medium)
    - UNREST (low)
  increases:
    - BUREAUCRACY (high)
    - ANGER (medium)

FORM_REJECTED:
  enables:
    - APPEAL_FILED (high)
    - BRIBERY_ATTEMPT (medium)
    - RAGE_QUIT (low)
  increases:
    - FRUSTRATION (high)
```

## Volatile Chaining

```yaml
VOLATILE_DECAY:
  enables:
    - EXPLOSION_EVENT (high, if unchecked)
    - EVACUATION (medium)
  increases:
    - FEAR (high)
    - URGENCY (high)

EXPLOSION_EVENT:
  enables:
    - FIRE_OUTBREAK (always)
    - INJURY_ACCIDENT (high)
    - STRUCTURAL_COLLAPSE (medium)
  increases:
    - TRAUMA (high)
    - DAMAGE (high)
```

## Corrosive Chaining

```yaml
ACID_RAIN_EVENT:
  enables:
    - STRUCTURAL_DISSOLUTION (high)
    - CROP_FAILURE (medium)
    - INDOOR_CONFINEMENT (high)
  increases:
    - MAINTENANCE_DEBT (high)
    - FEAR_OF_OUTSIDE (medium)

STRUCTURAL_DISSOLUTION:
  enables:
    - HULL_BREACH (medium)
    - REPAINTING_TASK (high)
  increases:
    - EXPOSURE (medium)

## Generational Hoarders Chaining

```yaml
HOARD_DISCOVERED:
  enables:
    - HOARD_CONFISCATED (high)
    - JURY_RIG_EVENT (medium, if hoarder forced to use it)
  increases:
    - LOGISTICS_JAM (high)
    - FRUSTRATION (medium)

HOARD_CONFISCATED:
  enables:
    - STRIKE (low, from elders)
    - BRAWL (medium)
  increases:
    - UNREST (medium)
    - RESOURCE_GAIN (high)
```

## Echoes of the Past Chaining

```yaml
GHOST_SIGHTING:
  enables:
    - ANCIENT_SECRET_REVEALED (medium)
    - PANIC_SPREAD (high)
  increases:
    - FEAR (high)
    - MYSTERY (high)

ANCIENT_SECRET_REVEALED:
  enables:
    - TECH_BREAKTHROUGH (high)
    - ARTIFACT_DISCOVERED (high)
  increases:
    - KNOWLEDGE (high)
```

## Symbiotic Shipyards Chaining

```yaml
LIVING_SHIP_BORN:
  enables:
    - SHIP_STARVATION (high, ongoing risk)
    - FLEET_ENGAGEMENT (medium)
  increases:
    - MILITARY_POWER (high)
    - FOOD_CONSUMPTION (maximum)

SHIP_STARVATION:
  enables:
    - MUTINY (low, ship turns hostile)
    - FAMINE (high, if ship eats reserves)
  increases:
    - TENSION (high)
```

## Orbital Tethers as Weapons Chaining

```yaml
TETHER_SNAPPED:
  enables:
    - ISOLATION_EVENT (always)
    - REBUILDING_EFFORT (low, unlikely)
  increases:
    - DESTRUCTION (maximum)
    - DESPAIR (high)

TETHER_SACRIFICE:
  enables:
    - THREAT_DESTROYED (always)
    - ISOLATION_EVENT (always)
  increases:
    - REGRET (medium)
    - SAFETY (high, temporary)
```

## The Empathy Plague Chaining

```yaml
EMPATHY_PLAGUE_START:
  enables:
    - CASCADE_BREAKDOWN (high)
    - SYNCHRONIZED_WORK (low, if happy)
  increases:
    - COHESION (maximum)
    - VULNERABILITY (high)

CASCADE_BREAKDOWN:
  enables:
    - WORK_STOPPAGE (always)
    - MEDICAL_EMERGENCY (high)
  increases:
    - STRESS (maximum)
```

## Counterfeit Reality Chaining

```yaml
HOLO_FLEET_PROJECTED:
  enables:
    - BLUFF_CALLED (medium)
    - THREAT_IGNORED (high, enemy flees)
  increases:
    - POWER_DRAIN (high)
    - FALSE_SECURITY (high)

BLUFF_CALLED:
  enables:
    - ORBITAL_BOMBARDMENT (high)
    - PANIC_SPREAD (always)
  increases:
    - VULNERABILITY (maximum)
    - DESTRUCTION (high)
```
```

## Doppelganger Chaining

```yaml
DOPPELGANGER_SUSPECTED:
  enables:
    - INVESTIGATION_STARTED (high)
    - PARANOIA_WAVE (high)
  increases:
    - UNREST (medium)
    - ACCIDENT_RISK (low, distracted)

DOPPELGANGER_REVEALED:
  enables:
    - FUNERAL_HELD (low, mixed feelings)
    - SECURITY_SWEEP (high)
  increases:
    - FEAR (high)
    - CAUTION (high)
```

## Hypno-Learning Chaining

```yaml
HYPNO_SESSION_COMPLETE:
  enables:
    - HYPNO_FOG_ONSET (always)
    - MASTERY_ACHIEVED (high)
  increases:
    - KNOWLEDGE (high)
    - FATIGUE (high)

HYPNO_FOG_ONSET:
  enables:
    - MEDICAL_EMERGENCY (low, if severe)
    - WORK_STOPPAGE (medium)
  increases:
    - CONFUSION (high)
    - SLOW_WORK (high)
```

## Placebo Protocol Chaining

```yaml
PLACEBO_ADMINISTERED:
  enables:
    - MOOD_WAVE (high, temporary relief)
    - DISTRUST_DISCOVERED (low, if trick fails)
  increases:
    - HOPE (medium)
    - SUSPICION (low)
```

## Cultural Vandalism Chaining

```yaml
MONUMENT_DEFACED:
  enables:
    - PROTEST (high)
    - CLEANUP_EFFORT (high)
    - REPRISAL (medium)
  increases:
    - DIVISION (high)
    - UNREST (maximum)
```

## Shadow Market Chaining

```yaml
SHADOW_TRADE_MADE:
  enables:
    - CONTRABAND_SEIZED (medium, if caught)
    - ADDICTION_CRISIS (low, bad stims)
  increases:
    - WEALTH (medium)
    - CORRUPTION (high)
```

## Great Works Chaining

```yaml
GREAT_WORK_STARTED:
  enables:
    - GREAT_WORK_FINISHED (eventually)
    - RESOURCE_SHORTAGE (high)
  increases:
    - AMBITION (high)
    - STRESS (medium)

GREAT_WORK_FINISHED:
  enables:
    - CELEBRATION (always)
    - LEGEND_BIRTH (high)
  increases:
    - MORALE (maximum)
    - PRESTIGE (high)
```

## Harmonic Mining Chaining

```yaml
HARMONIC_DRILL_USED:
  enables:
    - NOISE_COMPLAINT (high)
    - STRUCTURE_COLLAPSE (medium, if resonance hits glass)
  increases:
    - RESOURCE_YIELD (high)
    - DANGER (medium)
```

## Thermal Glider Chaining

```yaml
GLIDER_FLIGHT:
  enables:
    - FAST_DELIVERY (high)
    - STALL_CRASH (low, in cold zones)
  increases:
    - EFFICIENCY (medium)
    - AWE (low)
```

## Subliminal Advertising Chaining

```yaml
AD_CAMPAIGN_STARTED:
  enables:
    - RESOURCE_SHORTAGE (high, due to artificial demand)
    - TREND_STARTED (medium, forced mimicry)
  increases:
    - CONSUMPTION (maximum)
    - DISSATISFACTION (high)
```

## Subspace Pen Pal Chaining

```yaml
SUBSPACE_MESSAGE_RECEIVED:
  enables:
    - RUMOR_SPREAD (high, off-world news)
    - KNOWLEDGE_BREAKTHROUGH (low, shared tech)
  increases:
    - MORALE (medium)
    - ALIENATION (low, longing for elsewhere)
```

## Scapegoat Chaining

```yaml
POP_DENOUNCED:
  enables:
    - SCAPEGOAT_EXILED (high)
    - FACTION_FORMED (medium, sympathizers)
  increases:
    - DIVISION (maximum)
    - FEAR (high)
```

## Temporal Smuggling Chaining

```yaml
TEMPORAL_RIFT_OPENED:
  enables:
    - TEMPORAL_DEBT_DUE (always)
    - TECH_BREAKTHROUGH (low, future tools)
  increases:
    - SURVIVAL_CHANCE (high, immediate)
    - EXISTENTIAL_DREAD (high)
```

## Cadet Branch Chaining

```yaml
NOBLE_ARRIVAL:
  enables:
    - LUXURY_DEMAND_ISSUED (high)
    - CLASS_FRICTION_EVENT (high)
  increases:
    - WEALTH (medium, funding)
    - RESENTMENT (high)
```

## Bio-Acoustic Chorus Chaining

```yaml
BIO_ACOUSTIC_CHORUS_HEARD:
  enables:
    - MOOD_WAVE (high)
    - ARTISTIC_INSPIRATION (medium)
  increases:
    - COHESION (high)
    - WONDER (low)
```

## Olfactory Map Chaining

```yaml
SCENT_OVERWHELM:
  enables:
    - SICKNESS_OUTBREAK (medium, if toxic)
    - CLEANUP_EFFORT (high)
  increases:
    - DISGUST (high)
    - STRESS (medium)
```

## Orbital Drop Logistics Chaining

```yaml
DROP_POD_SCATTERED:
  enables:
    - EXPEDITION (high)
    - ITEM_LOSS (medium, if not recovered fast)
  increases:
    - RISK (high)
    - HOPE (medium, supplies arrived)
```

## Gut Biome Chaining

- DIET_CHANGE_SICKNESS → enables → MEDICAL_EMERGENCY
- DIET_CHANGE_SICKNESS → enables → COMPLAINT_FILED
- DIET_CHANGE_SICKNESS → increases_chance → FRUSTRATION
- GUT_COMFORT_ACHIEVED → enables → PRODUCTIVITY_BOOST
- GUT_COMFORT_ACHIEVED → increases_chance → MORALE

## Kinetic Storage Chaining

- BATTERY_CHARGED → enables → POWER_SURGE
- BATTERY_CHARGED → increases_chance → ENERGY_SECURITY
- BATTERY_COLLAPSE → enables → STRUCTURE_COLLAPSE
- BATTERY_COLLAPSE → enables → INJURY_ACCIDENT
- BATTERY_COLLAPSE → enables → PANIC_SPREAD
- BATTERY_COLLAPSE → increases_chance → FEAR_OF_TECH

## The Direct Link Chaining

- LINK_ESTABLISHED → enables → HERO_BIRTH
- LINK_ESTABLISHED → increases_chance → EFFICIENCY
- LINK_SEVERED → enables → EXHAUSTION_BREAKDOWN
- LINK_SEVERED → increases_chance → CONFUSION

## Public Grievances Chaining

- GRIEVANCE_POSTED → enables → RIVALRY_STARTED
- GRIEVANCE_POSTED → enables → PROTEST
- GRIEVANCE_POSTED → increases_chance → DIVISION
- COMMENDATION_POSTED → enables → BOND_FORMED
- COMMENDATION_POSTED → increases_chance → COHESION

## Clone Vats Chaining

- CLONE_DECANTED → enables → CLONE_DISCRIMINATION
- CLONE_DECANTED → enables → WORKFORCE_EXPANSION
- CLONE_DECANTED → increases_chance → SOCIAL_FRICTION
- CLONE_DISCRIMINATION → enables → BRAWL
- CLONE_DISCRIMINATION → enables → FACTION_FORMED
- CLONE_DISCRIMINATION → increases_chance → UNREST

## Legacy Code Chaining

- SYSTEM_BLOAT_WARNING → enables → WORK_STOPPAGE
- SYSTEM_BLOAT_WARNING → enables → REFORMAT_INITIATED
- SYSTEM_BLOAT_WARNING → increases_chance → BUREAUCRATIC_DRAG
- REFORMAT_INITIATED → enables → BLACKOUT
- REFORMAT_INITIATED → enables → LOGISTICS_JAM
- REFORMAT_INITIATED → increases_chance → VULNERABILITY

## Ghost Code Chaining

- GHOST_CODE_MANIFESTS → enables → MACHINE_MALFUNCTION
- GHOST_CODE_MANIFESTS → enables → ACCIDENT
- GHOST_CODE_MANIFESTS → enables → RESIDUE_PURGED
- GHOST_CODE_MANIFESTS → increases_chance → SUPERSTITION
- RESIDUE_PURGED → enables → TECH_STABILIZATION
- RESIDUE_PURGED → increases_chance → EFFICIENCY

## Thermal Bloom Chaining

- THERMAL_BLOOM_DETECTED → enables → HEAT_SINK_ATTACK
- THERMAL_BLOOM_DETECTED → enables → SENSOR_BLINDNESS
- THERMAL_BLOOM_DETECTED → increases_chance → DANGER
- HEAT_SINK_ATTACK → enables → ORBITAL_BOMBARDMENT
- HEAT_SINK_ATTACK → enables → MILITIA_MUSTER
- HEAT_SINK_ATTACK → increases_chance → PANIC

## The Infinite Archive Chaining

- ARCHIVE_PARALYSIS → enables → KNOWLEDGE_PURGED
- ARCHIVE_PARALYSIS → enables → SERVER_EXPANSION
- ARCHIVE_PARALYSIS → increases_chance → RESEARCH_STALL
- KNOWLEDGE_PURGED → enables → REGRET_EVENT
- KNOWLEDGE_PURGED → increases_chance → EFFICIENCY

## Quantum Twins Chaining

- TWIN_SYNC_EVENT → enables → MASTERY_ACHIEVED
- TWIN_SYNC_EVENT → enables → MOOD_WAVE
- TWIN_SYNC_EVENT → increases_chance → EFFICIENCY
- SEVERANCE_SHOCK → enables → CATATONIC_BREAK
- SEVERANCE_SHOCK → enables → MEDICAL_EMERGENCY
- SEVERANCE_SHOCK → increases_chance → GRIEF

## The Empty Room Chaining

- SANCTUARY_ESTABLISHED → enables → STRESS_RELIEF_VISIT
- SANCTUARY_ESTABLISHED → enables → SANCTUARY_VIOLATED
- SANCTUARY_ESTABLISHED → increases_chance → MORALE
- SANCTUARY_VIOLATED → enables → CLEANUP_DUTY
- SANCTUARY_VIOLATED → enables → BRAWL
- SANCTUARY_VIOLATED → increases_chance → STRESS

## Tectonic Stress Chaining

- STRESS_CRITICAL → enables → RELIEF_QUAKE_TRIGGERED
- STRESS_CRITICAL → enables → MEGA_QUAKE_EVENT
- STRESS_CRITICAL → increases_chance → FEAR
- RELIEF_QUAKE_TRIGGERED → enables → STRUCTURAL_DAMAGE
- RELIEF_QUAKE_TRIGGERED → enables → PANIC_SPREAD
- RELIEF_QUAKE_TRIGGERED → increases_chance → SAFETY

## The Industrial Rhythm Chaining

- PERFECT_RHYTHM → enables → PRODUCTIVITY_BOOST
- PERFECT_RHYTHM → enables → MOOD_WAVE
- PERFECT_RHYTHM → increases_chance → MORALE
- DISCORDANT_NOISE → enables → NOISE_COMPLAINT
- DISCORDANT_NOISE → enables → WORK_STOPPAGE
- DISCORDANT_NOISE → increases_chance → STRESS

## The Black Market Chaining

- SMUGGLER_DOCKED → enables → CORRUPTION_EXPOSED
- SMUGGLER_DOCKED → enables → NEED_FULFILLED
- SMUGGLER_DOCKED → increases_chance → CORRUPTION
- SMUGGLER_DOCKED → increases_chance → WEALTH
- CORRUPTION_EXPOSED → enables → ARREST
- CORRUPTION_EXPOSED → enables → PROTEST
- CORRUPTION_EXPOSED → increases_chance → AUTHORITY_LOSS

## Ancestral Graves Chaining

- GRAVE_VISITED → enables → REFLECTION
- GRAVE_VISITED → enables → GHOST_SIGHTING
- GRAVE_VISITED → increases_chance → MORALE
- SACRILEGE_COMMITTED → enables → PROTEST
- SACRILEGE_COMMITTED → enables → CURSE_AWAKENING
- SACRILEGE_COMMITTED → enables → RIVALRY_STARTED
- SACRILEGE_COMMITTED → increases_chance → UNREST

## The Overview Effect Chaining

- EXISTENTIAL_EPIPHANY → enables → RESEARCH_BREAKTHROUGH
- EXISTENTIAL_EPIPHANY → enables → PHILOSOPHICAL_SHIFT
- EXISTENTIAL_EPIPHANY → increases_chance → KNOWLEDGE
- ORBITAL_DREAD → enables → PANIC_SPREAD
- ORBITAL_DREAD → enables → WORK_STOPPAGE
- ORBITAL_DREAD → increases_chance → STRESS

## The Spiteful Will Chaining

- SPITEFUL_WILL_EXECUTED → enables → RIVALRY_STARTED
- SPITEFUL_WILL_EXECUTED → enables → BRAWL
- SPITEFUL_WILL_EXECUTED → increases_chance → RESENTMENT
- WILL_OVERRIDE → enables → PROTEST
- WILL_OVERRIDE → enables → FACTION_FORMED
- WILL_OVERRIDE → increases_chance → AUTHORITY

## The Event Horizon Tap Chaining

- TAP_ACTIVATED → enables → POWER_SURGE
- TAP_ACTIVATED → enables → MACHINE_MALFUNCTION
- TAP_ACTIVATED → increases_chance → AWE
- DILATION_CRISIS → enables → FAMINE
- DILATION_CRISIS → enables → ACCIDENT
- DILATION_CRISIS → increases_chance → EXHAUSTION

## Machine Awakening Chaining

- MACHINE_GLITCH → increases_chance → MACHINE_AWAKENED
- MACHINE_AWAKENED → enables → BOT_REBELLION, SOCIAL_UNREST, EQUAL_RIGHTS_MOVEMENT
- MACHINE_AWAKENED → reveals → hidden memory logs of [BOT_DESIGNATION]

## The Lotus Simulation Chaining

- MENTAL_BREAKDOWN → increases_chance → ENTERED_LOTUS_SIMULATION
- STARVING → increases_chance → ENTERED_LOTUS_SIMULATION
- ENTERED_LOTUS_SIMULATION → increases_chance → STARVED_IN_LOTUS_SIMULATION
- ENTERED_LOTUS_SIMULATION → enables → FORGOTTEN_SOUL, THE_EMPTY_ROOM

## Psychic Background Radiation Chaining

- PSYCHIC_BACKGROUND_SPIKE → enables → INSOMNIA_EPIDEMIC, CABIN_FEVER_BREAK
- PSYCHIC_BACKGROUND_SPIKE → increases_chance → UNREST, FATIGUE

## The Commuter Tax Chaining

- TOLL_ROAD_ESTABLISHED → increases_chance → PRICED_OUT_OF_TRANSIT
- PRICED_OUT_OF_TRANSIT → increases_chance → PUBLIC_GRIEVANCE, CLASS_WARFARE
- PRICED_OUT_OF_TRANSIT → enables → DESIRE_PATHS, WILDERNESS_ENCOUNTER


## Stellar Forge Chaining

```yaml
STELLAR_FORGE_IGNITED:
  enables:
    - FORGE_MELTDOWN (medium, if coolant fails)
    - STELLAR_ALLOY_PRODUCED (high)
  increases:
    - MILITARY_POWER (high)
    - ENERGY_OUTPUT (maximum)

FORGE_MELTDOWN:
  enables:
    - POWER_OUTAGE (always)
    - COLONY_EVACUATED (medium)
  increases:
    - DESTRUCTION (maximum)
    - FEAR_OF_TECH (high)
```

## Gravity-Defying Flora Chaining

```yaml
VINE_ANCHORED:
  enables:
    - ROOF_TORN_OFF (medium, in storms)
    - AERIAL_HARVEST (high)
  increases:
    - FOOD_STABILITY (high)
    - STRUCTURAL_STRESS (medium)

ROOF_TORN_OFF:
  enables:
    - DECOMPRESSION_EVENT (high)
    - REBUILDING_EFFORT (high)
  increases:
    - EXPOSURE (maximum)
    - FRUSTRATION (high)
```

## Subterranean Mycelial Network Chaining

```yaml
NETWORK_TRAINED:
  enables:
    - FUNGAL_INFECTION_SPREAD (medium)
    - INSTANT_LOGISTICS (high)
  increases:
    - EFFICIENCY (maximum)
    - ALIENATION (low)

FUNGAL_INFECTION_SPREAD:
  enables:
    - MEDICAL_EMERGENCY (high)
    - MYCELIUM_PURGE (medium)
  increases:
    - SICKNESS (maximum)
    - PARANOIA (high)
```

## The Monumental Ego Chaining

```yaml
VANITY_PROJECT_STARTED:
  enables:
    - VANITY_PROJECT_FINISHED (eventually)
    - STRIKE (medium)
  increases:
    - RESOURCE_DRAIN (maximum)
    - RESENTMENT (high)

VANITY_PROJECT_FINISHED:
  enables:
    - DIPLOMATIC_AWE (low, from superficial empires)
    - REBELLION (medium)
  increases:
    - PRESTIGE (high)
    - UNREST (maximum)
```

## Ghost Frequencies Chaining

```yaml
GHOST_BROADCAST_RECEIVED:
  enables:
    - TIMELINE_DIVERGED (always)
    - PANIC_PREPARATION (high)
  increases:
    - PARANOIA (high)
    - MYSTERY (maximum)

TIMELINE_DIVERGED:
  enables:
    - RELIEF (medium)
    - ECONOMIC_CRASH (low, from wasted prep)
  increases:
    - CONFUSION (high)
```

## Bureau of Redundancy Chaining

```yaml
DOUBLE_VERIFICATION_ENACTED:
  enables:
    - BUREAUCRATIC_DEADLOCK (high, in crisis)
    - ACCIDENT_PREVENTION (maximum)
  increases:
    - SAFETY (maximum)
    - BUREAUCRACY (maximum)

BUREAUCRATIC_DEADLOCK:
  enables:
    - CRITICAL_FAILURE (always)
    - POLICY_REVOKED (medium)
  increases:
    - ANGER (high)
    - DESTRUCTION (medium)
```

## Panopticon Morale Chaining

```yaml
CAMERAS_INSTALLED:
  enables:
    - SURVEILLANCE_RIOT (medium, on blackout)
    - PERFECT_ATTENDANCE (high)
  increases:
    - PRODUCTIVITY (maximum)
    - STRESS (maximum)

SURVEILLANCE_RIOT:
  enables:
    - CAMERAS_DESTROYED (always)
    - MILITIA_MUSTER (high)
  increases:
    - CHAOS (maximum)
    - DAMAGE (high)
```

## Zero-G Sports Chaining

```yaml
ZERO_G_MATCH_PLAYED:
  enables:
    - STAR_PLAYER_INJURED (medium)
    - BETTING_SCANDAL (low)
  increases:
    - MORALE (maximum)
    - COHESION (high)

STAR_PLAYER_INJURED:
  enables:
    - MEDICAL_TREATMENT (high)
    - RIOT (low, from fans)
  increases:
    - GRIEF (medium)
```

## Bureaucratic Language Chaining

```yaml
HIGH_SPEECH_MANDATED:
  enables:
    - TRANSLATION_FATAL_ERROR (high)
    - CLASS_DIVIDE (high)
  increases:
    - CONFUSION (maximum)
    - UNREST (high)

TRANSLATION_FATAL_ERROR:
  enables:
    - MASS_CASUALTIES (medium)
    - INVESTIGATION (low)
  increases:
    - TRAGEDY (high)
    - HATRED_OF_ELITES (maximum)
```

## Cult of the Forgotten Machine Chaining

```yaml
MACHINE_QUIRK_REVERED:
  enables:
    - CULT_SABOTAGE (high)
    - MAINTENANCE_DEBT (maximum)
  increases:
    - SUPERSTITION (maximum)
    - TENSION (high)

CULT_SABOTAGE:
  enables:
    - POWER_OUTAGE (high)
    - RELIGIOUS_PURGE (medium)
  increases:
    - DESTRUCTION (high)
    - DIVISION (maximum)
```

## Phantom Sub-routines Chaining

```yaml
GHOST_FLEET_SPAWNED:
  enables:
    - PHANTOM_ORDER_EXECUTED (always)
    - LOGISTICS_JAM (maximum)
  increases:
    - CONFUSION (high)
    - INEFFICIENCY (maximum)

PHANTOM_ORDER_EXECUTED:
  enables:
    - CODE_DEFRAGMENTATION (high)
    - RESOURCE_SHORTAGE (high)
  increases:
    - WASTE (maximum)
```

## The Biosphere Empathy Link Chaining (Spec 293)
- EMPATHIC_LINK_ESTABLISHED → enables → MENTAL_COLLECTIVE, MASS_HYSTERIA, CULT_OF_THE_ROOT
- EMPATHIC_LINK_ESTABLISHED → increases_chance → PRODUCTIVITY_SPIKE, SILENT_WORKERS
- FLORA_DAMAGED_BACKLASH → increases_chance → MASS_BREAKDOWN, UNREST, MUTINY

## Escape Velocity Economics Chaining (Spec 468)
- HIGH_G_LAUNCH_SUCCESS → enables → ORBITAL_STATION_BUILD, LUXURY_TRADE, DEBT_PAYMENT
- ECONOMIC_WELL_TRAP → increases_chance → SMUGGLING, ISOLATIONISM, TECHNOLOGICAL_REGRESSION
- ECONOMIC_WELL_TRAP → decreases_chance → MIGRATION, INTERSTELLAR_WAR

## Civic Ideology Chaining (Spec 484)
- IDEOLOGY_FOUNDED → enables → INQUISITION, PURGE, GREAT_WORKS, GOLDEN_AGE
- IDEOLOGICAL_DEVIATION → increases_chance → SCHISM, REBELLION, CULT_FORMATION
- IDEOLOGICAL_DEVIATION → decreases_chance → PRODUCTIVITY_SPIKE, LOYALTY

## Debt-Prison Colonies Chaining (Spec 486)
- BAILOUT_ACCEPTED → enables → GANG_WARFARE, BLACK_MARKET_BOOM, HIGH_SKILL_INDUSTRY
- BAILOUT_ACCEPTED → increases_chance → ARTIFACT_THEFT, ASSASSINATION, CORRUPTION
- BAILOUT_ACCEPTED → decreases_chance → PEACEFUL_PROTEST, LAW_AND_ORDER

## Deep Crust Geomes Chaining (Spec 515)
- GEOME_BREACHED → enables → NEW_RESOURCE_DISCOVERY, SUBTERRANEAN_EXPANSION
- GEOME_BREACHED → increases_chance → MUTATION, CAVE_IN, SPONTANEOUS_CULT_FORMATION

## Inter-Colony Trade Routes Chaining (Spec 538)
- TRADE_ROUTE_ESTABLISHED → enables → ECONOMIC_BOOM, RESOURCE_SPECIALIZATION
- TRADE_ROUTE_ESTABLISHED → increases_chance → PIRATE_RAID, CULTURAL_EXCHANGE
- TRADE_ROUTE_DISRUPTED → enables → RESOURCE_SHORTAGE, DIPLOMATIC_INCIDENT
- TRADE_ROUTE_DISRUPTED → increases_chance → FAMINE, SMUGGLING

## Penal Contracts Chaining (Spec 539)
- PENAL_CONTRACT_SIGNED → enables → CHEAP_LABOR_BOOM, PENAL_COLONY_EXPANSION
- PENAL_CONTRACT_SIGNED → increases_chance → PRISON_RIOT, CORRUPTION, BLACK_MARKET_ACTIVITY

## The Exodus Chaining (Spec 540)
- ARK_CONSTRUCTION_BEGUN → enables → RESOURCE_HOARDING, MASS_MIGRATION
- ARK_CONSTRUCTION_BEGUN → increases_chance → ZEALOTRY, DOOMSDAY_CULT
- STRUCTURE_CANNIBALIZED → enables → INFRASTRUCTURE_COLLAPSE, DESPERATE_MEASURES
- STRUCTURE_CANNIBALIZED → increases_chance → UNREST, POWER_OUTAGE

## The Silent Mutiny Chaining (Spec 533)
- MUTINY_REVEALED → enables → FLEET_DESERTION, ROGUE_STATION_FOUNDED
- MUTINY_REVEALED → increases_chance → PARANOIA, WITCH_HUNT, SUPPLY_SHORTAGE

## The Cartographer's Curse Chaining
- MAPS_SOLD → enables → ECONOMIC_WINDFALL, RAPID_EXPANSION
- MAPS_SOLD → increases_chance → TARGETED_RAID, SABOTAGE, ESPIONAGE

## The Sabotaged Seed Bank Chaining
- SEED_BANK_DISCOVERED → enables → AGRICULTURAL_REVOLUTION, POPULATION_BOOM
- SEED_BANK_DISCOVERED → increases_chance → COMPLACENCY, MONOCULTURE
- KILL_SWITCH_ACTIVATED → enables → INSTANT_FAMINE, ECOLOGICAL_COLLAPSE
- KILL_SWITCH_ACTIVATED → increases_chance → MASS_STARVATION, CANNIBALISM

## The Empathic Sinkhole Chaining
- SINKHOLE_DISCOVERED → enables → STRESS_RELIEF_VISIT, CULT_OF_THE_ABYSS
- SINKHOLE_DISCOVERED → increases_chance → MYSTERIOUS_DISAPPEARANCE
- SINKHOLE_ERUPTION → enables → MASS_HYSTERIA, COLONY_WIDE_CATATONIA
- SINKHOLE_ERUPTION → increases_chance → SUICIDE_PACT, MUTINY

## Atmospheric Ignition Chaining
- VAPOR_CLOUD_FORMED → enables → RESPIRATORY_ILLNESS, HAZMAT_PROTOCOLS
- VAPOR_CLOUD_FORMED → increases_chance → EVACUATION, WORK_STOPPAGE
- AIR_BURST_DETONATION → enables → MASS_CASUALTIES, TOTAL_DESTRUCTION
- AIR_BURST_DETONATION → increases_chance → FIRESTORM, ATMOSPHERIC_COLLAPSE

## Orbital Necropolis Chaining
- SARCOPHAGUS_LAUNCHED → enables → MORALE_BOOST, ANCESTOR_WORSHIP
- SARCOPHAGUS_LAUNCHED → increases_chance → ORBITAL_CLUTTER
- NECROPOLIS_DESECRATED → enables → SACRILEGE_PENALTY, HOLY_WAR
- NECROPOLIS_DESECRATED → increases_chance → REBELLION, DESPAIR

## The Parasitic Wardrobe Chaining
- SYM_WEAVE_DONNED → enables → HEROIC_FEAT, UNSTOPPABLE_SOLDIER
- SYM_WEAVE_DONNED → increases_chance → ELITISM, ARROGANCE
- PARASITIC_TOLL → enables → MEDICAL_EMERGENCY, SLOW_DEATH
- PARASITIC_TOLL → increases_chance → INSANITY, MUTATION

## The Empathic Gridlock Chaining
- GRIDLOCK_FORMED → enables → LOGISTICS_JAM, WORK_STOPPAGE
- GRIDLOCK_FORMED → increases_chance → BRAWL, STARVATION

## The Quantum Audit Chaining
- AUDIT_DECLARED → enables → PANIC_HOARDING, DATA_DELETION
- AUDIT_DECLARED → increases_chance → STRESS, PARANOIA
- AUDIT_PUNISHMENT → enables → INFRASTRUCTURE_LOSS, ECONOMIC_CRASH
- AUDIT_PUNISHMENT → increases_chance → REBELLION, RESENTMENT

## The Bureaucratic Black Hole Chaining
- DATA_DEMAND_ISSUED → enables → ADMIN_OVERLOAD, RESOURCE_DRAIN
- DATA_DEMAND_ISSUED → increases_chance → FATIGUE, REBELLION

## The Gravity Well Dump Chaining
- TOXIC_DUMP → enables → POLLUTION_EVENT, HAZMAT_CLEANUP
- TOXIC_DUMP → increases_chance → MUTATION, SICKNESS, ANGER

## Galactic Standard Time Chaining
- GST_ENFORCED → enables → TRADE_EFFICIENCY_BOOST, DIPLOMATIC_BONUS
- GST_ENFORCED → increases_chance → CHRONIC_FATIGUE, MENTAL_BREAK

## The Feral Algorithm Chaining
- ALGORITHM_OPTIMIZES → enables → RUTHLESS_EFFICIENCY, AUTOMATED_CRUELTY
- ALGORITHM_OPTIMIZES → increases_chance → MASS_CASUALTIES, SABOTAGE, REBELLION

## The Galactic Council Chaining
- COUNCIL_SANCTION_APPLIED → enables → BLOCKADE_RUNNER, BLACK_MARKET_BOOM
- COUNCIL_SANCTION_APPLIED → increases_chance → STARVATION, MUTINY
- COUNCIL_SANCTION_LIFTED → enables → TRADE_FESTIVAL, ECONOMIC_BOOM
- COUNCIL_SANCTION_LIFTED → increases_chance → LOYALTY_TO_CORE

## Synthetic Apathy Chaining
- SYNTH_APATHY_INCIDENT → enables → LUDDITE_REBELLION, SYNTH_PURGE
- SYNTH_APATHY_INCIDENT → increases_chance → ALIENATION, STRESS

## Planetary Governance Chaining (Spec 544)
- GOVERNOR_APPOINTED -> increases_chance -> CORRUPTION
- GOVERNOR_APPOINTED -> increases_chance -> AMBITION
- GOVERNOR_APPOINTED -> enables -> GOVERNOR_DECLARED_INDEPENDENCE

## Disaster Tourism Chaining (Spec 545)
- DISASTER -> enables -> DISASTER_TOURISTS_ARRIVE
- DISASTER_TOURISTS_ARRIVE -> enables -> OBSERVATION_DECK_BUILT_NEAR_DISASTER
- OBSERVATION_DECK_BUILT_NEAR_DISASTER -> increases_chance -> DISASTER
- OBSERVATION_DECK_BUILT_NEAR_DISASTER -> decreases_chance -> STARVATION

## Biomass Tariff Chaining (Spec 548)
- FAMINE -> increases_chance -> POPS_TRADED_AS_LIVESTOCK
- BIOMASS_TARIFF_DEMANDED -> enables -> POPS_TRADED_AS_LIVESTOCK
- POPS_TRADED_AS_LIVESTOCK -> enables -> BIO_ARCHITECTURE_SCREAMED
- BIO_ARCHITECTURE_SCREAMED -> increases_chance -> MADNESS
- BIO_ARCHITECTURE_SCREAMED -> increases_chance -> REBELLION
