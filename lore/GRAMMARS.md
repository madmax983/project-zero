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
