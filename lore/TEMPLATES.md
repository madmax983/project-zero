# Templates

*Event structures with slots. The generator fills slots from FRAGMENTS or game state.*

---

## Pre-History Templates (World Generation)

These fire during "Generating history..." before game start.

### CIVILIZATION_RISE

**Slots:** [CIV_NAME], [ORIGIN_STAR], [YEAR], [CIV_EPITHET]

```
"Year [YEAR]. The [CIV_NAME] arise from [ORIGIN_STAR]. They will come to be called [CIV_EPITHET]."

"[CIV_NAME]—[CIV_EPITHET]—first reach beyond [ORIGIN_STAR] in [YEAR]."

"[YEAR]: First records of [CIV_NAME] expansion. Origin: [ORIGIN_STAR]. Later designation: [CIV_EPITHET]."

"From [ORIGIN_STAR], in [YEAR], come the [CIV_NAME]. [CIV_EPITHET]. Remember them."
```

### CIVILIZATION_FALL

**Slots:** [CIV_NAME], [YEAR], [CIV_FATE], [DURATION_PHRASE], [CIV_EPITHET]?

```
"Year [YEAR]. The [CIV_NAME] [CIV_FATE]. They lasted [DURATION_PHRASE]."

"[YEAR]: [CIV_NAME] [CIV_FATE]. [DURATION_PHRASE] of history, ended."

"The [CIV_NAME]—[CIV_EPITHET]—[CIV_FATE] in [YEAR]. The silence that followed lasted [DURATION_PHRASE]."

"[YEAR]. [CIV_NAME] signals cease. Investigation finds: they [CIV_FATE]."
```

### WAR_RECORD

**Slots:** [WAR_NAME], [CIV_A], [CIV_B], [START_YEAR], [END_YEAR], [CAUSE], [OUTCOME]

```
"[WAR_NAME] ([START_YEAR]-[END_YEAR]). [CIV_A] against [CIV_B]. Cause: [CAUSE]. Outcome: [OUTCOME]."

"The [CIV_A]-[CIV_B] conflict, called [WAR_NAME]. [START_YEAR] to [END_YEAR]. [CAUSE]. [OUTCOME]."

"[START_YEAR]: War. [CIV_A] and [CIV_B]. Over [CAUSE]. Ends [END_YEAR]. [OUTCOME]."
```

### ARTIFACT_CREATION

**Slots:** [ARTIFACT_NAME], [ARTIFACT_TYPE], [CREATOR_CIV], [CREATOR_PERSON]?, [YEAR], [ORIGIN_PHRASE]

```
"[ARTIFACT_NAME], a [ARTIFACT_TYPE], [ORIGIN_PHRASE] in [YEAR]. Created by the [CREATOR_CIV]."

"Year [YEAR]. [CREATOR_CIV] creates [ARTIFACT_NAME]—[ORIGIN_PHRASE]."

"[ARTIFACT_NAME] ([ARTIFACT_TYPE]). [CREATOR_CIV], [YEAR]. [ORIGIN_PHRASE]. Current location: unknown."

"The [ARTIFACT_TYPE] known as [ARTIFACT_NAME]. [CREATOR_PERSON] of the [CREATOR_CIV], [YEAR]. [ORIGIN_PHRASE]."
```

### CATASTROPHE

**Slots:** [CATASTROPHE_TYPE], [YEAR], [AFFECTED_REGION], [CONSEQUENCE]

```
"[YEAR]. [CATASTROPHE_TYPE] strikes [AFFECTED_REGION]. [CONSEQUENCE]."

"The [CATASTROPHE_TYPE], [YEAR]. [AFFECTED_REGION] never recovers. [CONSEQUENCE]."

"[YEAR]: [CATASTROPHE_TYPE]. Scope: [AFFECTED_REGION]. Aftermath: [CONSEQUENCE]."
```

### ERA_TRANSITION

**Slots:** [OLD_ERA], [NEW_ERA], [YEAR], [CAUSE]

```
"[YEAR]. [OLD_ERA] ends. [NEW_ERA] begins. Cause: [CAUSE]."

"The end of [OLD_ERA], year [YEAR]. What follows: [NEW_ERA]. Why: [CAUSE]."

"[YEAR] marks the transition. [OLD_ERA] gives way to [NEW_ERA] after [CAUSE]."
```

---

## Play Templates (During Game)

These fire during gameplay and get appended to the chronicle.

### COLONY_FOUNDED

**Slots:** [COLONY_NAME], [STAR], [YEAR], [FOUNDER_COUNT], [ORIGIN_COLONY]?

```
"[COLONY_NAME] founded, [STAR], year [YEAR]. [FOUNDER_COUNT] souls make landfall."

"Year [YEAR]. [FOUNDER_COUNT] colonists establish [COLONY_NAME] at [STAR]."

"[YEAR]: First landing at [STAR]. Colony designation: [COLONY_NAME]. Initial population: [FOUNDER_COUNT]."

[If ORIGIN_COLONY:]
"[COLONY_NAME] ([STAR], [YEAR]). Daughter colony of [ORIGIN_COLONY]. [FOUNDER_COUNT] founders."
```

### COLONY_LOST

**Slots:** [COLONY_NAME], [YEAR], [CAUSE], [FINAL_POP], [DURATION]

```
"[COLONY_NAME] falls silent. Year [YEAR]. Cause: [CAUSE]. Duration: [DURATION]. Final souls: [FINAL_POP]."

"[YEAR]. [COLONY_NAME] [CAUSE]. [FINAL_POP] names to remember. The colony stood [DURATION]."

"Year [YEAR]: [COLONY_NAME] signals cease. [CAUSE]. After [DURATION], the silence."

"[COLONY_NAME] ([DURATION]). [CAUSE] in [YEAR]. [FINAL_POP] souls. The channel stays open."
```

### FAMINE

**Slots:** [COLONY], [YEAR], [DURATION_DAYS], [DEATHS], [SURVIVOR_NAME]?

```
"[COLONY], [YEAR]. The Long Hunger. [DURATION_DAYS] days. [DEATHS] souls lost."

"Famine comes to [COLONY], year [YEAR]. It stays [DURATION_DAYS] days. It takes [DEATHS]."

"[YEAR]: [COLONY] remembers the Hunger. [DEATHS] names carved in stone. [DURATION_DAYS] days of empty stores."

[If SURVIVOR_NAME:]
"[SURVIVOR_NAME] survives the [COLONY] famine of [YEAR]. [DEATHS] others do not."
```

### BUILDING_MILESTONE

**Slots:** [COLONY], [YEAR], [BUILDING_TYPE], [COUNT], [BUILDER_NAME]?

```
"[COLONY], [YEAR]. [ORDINAL] [BUILDING_TYPE] completed. The colony grows."

"Year [YEAR]: [COLONY] raises its [COUNT]th [BUILDING_TYPE]."

[If BUILDER_NAME:]
"[BUILDER_NAME] finishes [COLONY]'s new [BUILDING_TYPE], year [YEAR]."
```

### FIRST_HOUSING

**Slots:** [COLONY], [YEAR], [SHELTER_DESCRIPTOR]

```
"Year [YEAR]. First shelters rise at [COLONY]. A [SHELTER_DESCRIPTOR] beginning."
"The first roof over our heads. [COLONY], [YEAR]. It is [SHELTER_DESCRIPTOR]."
"[YEAR]: Housing complete. The void is shut out. We are [SHELTER_DESCRIPTOR]."
```

### FIRST_FARM

**Slots:** [COLONY], [YEAR], [FARM_DESCRIPTOR]

```
"Year [YEAR]. First fields sown at [COLONY]. The soil is [FARM_DESCRIPTOR]."
"We shall not starve. [COLONY] farms produce their first yield in [YEAR]. [FARM_DESCRIPTOR]."
"[YEAR]: Agriculture established. A [FARM_DESCRIPTOR] harvest awaits."
```

### LEGEND_BIRTH

**Slots:** [PERSON_NAME], [COLONY], [YEAR], [DEED], [LEGACY_PHRASE]

```
"[PERSON_NAME] of [COLONY]. Year [YEAR]: [DEED]. [LEGACY_PHRASE]."

"[YEAR]. [PERSON_NAME] [DEED] at [COLONY]. They are [LEGACY_PHRASE]."

"The legend of [PERSON_NAME] begins in [COLONY], [YEAR]. [DEED]. [LEGACY_PHRASE]."
```

### ARTIFACT_DISCOVERED

**Slots:** [COLONY], [YEAR], [ARTIFACT_NAME], [ARTIFACT_TYPE], [ORIGIN_CIV]?, [QUALITY]

```
"[COLONY] surveyors report a find. Year [YEAR]. [ARTIFACT_NAME]—a [ARTIFACT_TYPE], [QUALITY]."

"[YEAR]: Beneath [COLONY], they find [ARTIFACT_NAME]. [ARTIFACT_TYPE]. [QUALITY]. Origin: [ORIGIN_CIV|unknown]."

"[ARTIFACT_NAME] surfaces at [COLONY], [YEAR]. A [QUALITY] [ARTIFACT_TYPE]. Its makers: [ORIGIN_CIV|forgotten]."
```

### FIRST_CONTACT

**Slots:** [YOUR_COLONY], [OTHER_CIV], [YEAR], [MANNER], [OUTCOME]

```
"[YEAR]. [YOUR_COLONY] is not alone. The [OTHER_CIV] make contact. Manner: [MANNER]. Outcome: [OUTCOME]."

"First Contact at [YOUR_COLONY], year [YEAR]. The [OTHER_CIV]. [MANNER]. [OUTCOME]."

"[OTHER_CIV] signals reach [YOUR_COLONY] in [YEAR]. [MANNER]. What follows: [OUTCOME]."
```

### SHIP_LOST

**Slots:** [SHIP_NAME], [DEPARTURE], [DESTINATION], [YEAR], [CREW_COUNT], [CARGO]?

```
"[SHIP_NAME] departs [DEPARTURE] for [DESTINATION], year [YEAR]. [CREW_COUNT] crew. Never arrives."

"[YEAR]. [SHIP_NAME] ([CREW_COUNT] souls, bound for [DESTINATION]) enters fold at [DEPARTURE]. The channel falls silent."

"Lost: [SHIP_NAME]. [YEAR]. [DEPARTURE] to [DESTINATION]. [CREW_COUNT] aboard. [CARGO|No manifest recovered]."
```

### SHIP_RETURNED

**Slots:** [SHIP_NAME], [EXPECTED_YEAR], [ACTUAL_YEAR], [CONDITION], [CREW_FATE]

```
"[SHIP_NAME] returns, year [ACTUAL_YEAR]. Expected: [EXPECTED_YEAR]. Condition: [CONDITION]. Crew: [CREW_FATE]."

"[ACTUAL_YEAR]. [SHIP_NAME] emerges from fold. [DELTA] years late. [CONDITION]. [CREW_FATE]."

"The [SHIP_NAME] comes home. [ACTUAL_YEAR], not [EXPECTED_YEAR]. [CONDITION]. The crew: [CREW_FATE]."
```

### VOID_INCIDENT

**Slots:** [SHIP_NAME], [YEAR], [ANOMALY], [CONSEQUENCE]

```
"Year [YEAR]. [SHIP_NAME] reports [ANOMALY]. Course corrected. Crew shaken."

"[YEAR]: Incident aboard [SHIP_NAME]. [ANOMALY]. [CONSEQUENCE]."

"The void touches [SHIP_NAME] in [YEAR]. [ANOMALY]. They will not speak of it."
```

### LOCATION_NAMED

**Slots:** [LOCATION_NAME], [COORDINATES], [REASON]

```
"The ground at [COORDINATES] is now called [LOCATION_NAME]. Reason: [REASON]."

"We name this place [LOCATION_NAME]. [REASON]."

"[LOCATION_NAME]. That is what the locals call [COORDINATES] after [REASON]."
```

### LOCATION_NAMED_LANDING

**Slots:** [LANDING_NAME], [YEAR]

```
"We name this place [LANDING_NAME]. Here we begin."
"Firstfall at [LANDING_NAME]. The journey ends, the work begins."
"[YEAR]. We plant the flag at [LANDING_NAME]."
```

### FIRST_MINE

**Slots:** [COLONY], [YEAR], [MINING_DESCRIPTOR]

```
"Year [YEAR]. We break the earth at [COLONY]. The stone is [MINING_DESCRIPTOR]."
"First quarry established. [YEAR]. We delve [MINING_DESCRIPTOR]."
"[YEAR]: Mining begins. The [COLONY] foundation deepens."
```

### RESOURCE_DISCOVERY

**Slots:** [COLONY], [YEAR], [RESOURCE], [QUANTITY_PHRASE]

```
"[COLONY] surveyors find [RESOURCE]. [YEAR]. [QUANTITY_PHRASE]."
"Year [YEAR]: A vein of [RESOURCE] unearthed. [QUANTITY_PHRASE]."
"[RESOURCE] discovered at [COLONY]. [YEAR]. [QUANTITY_PHRASE]."
```

### FIRST_LUMBER

**Slots:** [COLONY], [YEAR], [FOREST_DESCRIPTOR]

```
"Year [YEAR]. We clear the [FOREST_DESCRIPTOR] trees at [COLONY]. First timber."
"The first felling. [YEAR]. The wood is [FOREST_DESCRIPTOR]."
"[YEAR]: Forestry begins. We harvest the [FOREST_DESCRIPTOR] wild."
```

### FOREST_CLEARED

**Slots:** [COLONY], [YEAR], [FOREST_NAME], [AREA]

```
"The last tree of [FOREST_NAME] falls. [YEAR]. The sky is open."
"[YEAR]: [FOREST_NAME] is gone. Only stumps remain at [AREA]."
"We have conquered the Green at [AREA]. [FOREST_NAME] is no more."
```

### STOCKPILE_FULL

**Slots:** [COLONY], [YEAR], [STORE_NAME], [RESOURCE]

```
"The [STORE_NAME] overflows. [YEAR]. [RESOURCE] burdens us."
"[YEAR]: Capacity reached. We have too much [RESOURCE]."
"Abundance at [COLONY]. The [STORE_NAME] can hold no more [RESOURCE]."
```

### RESOURCE_SHORTAGE

**Slots:** [COLONY], [YEAR], [RESOURCE], [CRISIS]

```
"The [RESOURCE] runs low. [YEAR]. [CRISIS] threatens."
"[YEAR]: Shortage. We lack [RESOURCE]. The [CRISIS] begins."
"[COLONY] runs dry. No [RESOURCE]. It is a time of [CRISIS]."
```

### VEIN_DEPLETED

**Slots:** [COLONY], [YEAR], [RESOURCE]

```
"A vein of [RESOURCE] is spent. [YEAR]. [COLONY] digs deeper."
"Year [YEAR]. The [RESOURCE] runs out. The earth is empty here."
"[COLONY] reports: [RESOURCE] deposit exhausted. We must search again."
```

### CONSTRUCTION_HALTED

**Slots:** [COLONY], [YEAR], [BUILDING_TYPE], [RESOURCE]

```
"[BUILDING_TYPE] at [COLONY] halts. [YEAR]. Reason: [RESOURCE] shortage."
"Work stops on the [BUILDING_TYPE]. We lack [RESOURCE]."
"[YEAR]: The skeleton of a [BUILDING_TYPE] stands silent. No [RESOURCE] to finish it."
```

### POP_ARRIVAL

**Slots:** [COLONY], [YEAR], [COUNT], [ARRIVAL_METHOD]

```
"[COUNT] new souls arrive at [COLONY]. [YEAR]. Method: [ARRIVAL_METHOD]."
"[YEAR]: Reinforcements. [COUNT] join us, [ARRIVAL_METHOD]."
"The population grows. [COUNT] arrived [ARRIVAL_METHOD] today."
```

### POP_DEATH

**Slots:** [COLONY], [YEAR], [NAME], [REASON]

```
"A soul is lost. [NAME]. [YEAR]. Cause: [REASON]."
"[YEAR]: We mourn [NAME]. Taken by [REASON]."
"Death at [COLONY]. [NAME] has passed. [REASON]."
```

### RUMOR_SPREAD

**Slots:** [COLONY], [YEAR], [TOPIC]

```
"Whispers in the mess hall. [YEAR]. [TOPIC]. It spreads."
"[YEAR]: A rumor moves through [COLONY]. They speak of [TOPIC]."
"The talk is of [TOPIC]. Truth or fear? [YEAR]."
```

### SEASON_START

**Slots:** [COLONY], [YEAR], [SEASON_NAME], [SEASON_ADJECTIVE]

```
"[SEASON_NAME] comes to [COLONY]. [YEAR]. The air is [SEASON_ADJECTIVE]."
"Year [YEAR]. The turning of the wheel. It is [SEASON_NAME], [SEASON_ADJECTIVE] and real."
"The [SEASON_NAME] begins. [SEASON_ADJECTIVE] days ahead."
```

### FIRST_SMELT

**Slots:** [COLONY], [YEAR], [METAL_NAME], [REFINERY_NAME]

```
"The [REFINERY_NAME] roars to life. [YEAR]. First [METAL_NAME] poured."
"[YEAR]: Industry rises at [COLONY]. We make [METAL_NAME] now."
"The fires are lit. [METAL_NAME] flows from the [REFINERY_NAME]. [YEAR]."
```

### TAVERN_OPENED

**Slots:** [COLONY], [YEAR], [TAVERN_NAME]

```
"[TAVERN_NAME] opens its doors. [YEAR]. A place to forget."
"Year [YEAR]. [COLONY] has a heart now. We call it [TAVERN_NAME]."
"First drinks served at [TAVERN_NAME]. The silence is broken by song. [YEAR]."
```

### SOCIAL_GATHERING

**Slots:** [COLONY], [YEAR], [TAVERN_NAME], [SOCIAL_ACTION], [DRINK_NAME]

```
"Crowd at [TAVERN_NAME]. [YEAR]. Someone [SOCIAL_ACTION] over [DRINK_NAME]."
"Night at [COLONY]. The [TAVERN_NAME] is full. They [SOCIAL_ACTION]."
"[YEAR]: [DRINK_NAME] flows. The colony [SOCIAL_ACTION] together."
```

---

## Chronicle Entry Structure

Each template generates a chronicle entry with this structure:

```json
{
  "id": "evt_00001",
  "template": "COLONY_FOUNDED",
  "year": 1,
  "text": "Haven founded, Kepler Prime, year 1. 5 souls make landfall.",
  "entities": ["col_haven", "star_kepler_prime"],
  "discovered": true,
  "importance": "major"
}
```

**Importance levels:** `minor`, `standard`, `major`, `legendary`

Minor events may be pruned or summarized. Legendary events are always shown.

---

### KNOWLEDGE_BREAKTHROUGH

**Slots:** [COLONY], [YEAR], [TECH_NAME], [TECH_FLAVOR], [KNOWLEDGE_TOPIC]

```
"Year [YEAR]. We have unlocked [TECH_NAME]. It was [TECH_FLAVOR]."
"[TECH_NAME] is ours. [YEAR]. The [KNOWLEDGE_TOPIC] is clear now."
"A breakthrough in [TECH_NAME] at [COLONY]. [YEAR]. We found it in [KNOWLEDGE_TOPIC]."
```

### FIRE_OUTBREAK

**Slots:** [COLONY], [YEAR], [FIRE_NAME], [FIRE_DESCRIPTOR], [SOURCE]?

```
"Fire at [COLONY]. [YEAR]. The [FIRE_NAME] is here."
"[YEAR]: A [FIRE_DESCRIPTOR] blaze. [SOURCE|Sparks] ignited the dark."
"The Red Hunger wakes. [YEAR]. [COLONY] burns with [FIRE_DESCRIPTOR] heat."
```

### FIRE_EXTINGUISHED

**Slots:** [COLONY], [YEAR], [DURATION], [DAMAGE_REPORT]

```
"The fire is out. [YEAR]. It lasted [DURATION]. [DAMAGE_REPORT]."
"[YEAR]: Silence returns. The ash is cold. [DAMAGE_REPORT]."
"We beat back the Hunger at [COLONY]. [YEAR]. Cost: [DAMAGE_REPORT]."
```

### SPOILAGE_EVENT

**Slots:** [COLONY], [YEAR], [RESOURCE], [AMOUNT], [ROT_DESCRIPTOR]

```
"[YEAR]. The [RESOURCE] has turned. [AMOUNT] lost. It is [ROT_DESCRIPTOR]."
"Rot in the stores. [YEAR]. We lose [AMOUNT] [RESOURCE]. The smell is [ROT_DESCRIPTOR]."
"The Grey takes its tithe. [AMOUNT] [RESOURCE] gone. [YEAR]."
```

### INJURY_ACCIDENT

**Slots:** [COLONY], [YEAR], [NAME], [INJURY_TYPE], [CAUSE]

```
"Accident at [COLONY]. [YEAR]. [NAME] suffers [INJURY_TYPE]. Cause: [CAUSE]."
"[YEAR]: Blood on the floor. [NAME]. [INJURY_TYPE] from [CAUSE]."
"[NAME] is hurt. [INJURY_TYPE]. The work is dangerous. [YEAR]."
```

### HEALING_SUCCESS

**Slots:** [COLONY], [YEAR], [NAME], [HEALING_METHOD]

```
"[NAME] returns to the line. [YEAR]. Healed [HEALING_METHOD]."
"[YEAR]: Recovery. [NAME] is whole again. [HEALING_METHOD]."
"The medical rites succeed. [NAME] walks. [YEAR]."
```

### TOOL_BREAK

**Slots:** [COLONY], [YEAR], [NAME], [TOOL_NAME]

```
"[NAME]'s [TOOL_NAME] snaps. [YEAR]. The metal was weak."
"A broken [TOOL_NAME]. [YEAR]. [NAME] curses the forge."
"Silence in the work-hall. [NAME] holds the pieces of a [TOOL_NAME]. [YEAR]."
```

---

*Add new templates with clear slot definitions. Provide 3-4 pattern variants. Tag required vs optional slots.*

---

## Beauty & Horticulture Templates

### PARK_OPENED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [PARK_NAME], [BEAUTY_DESCRIPTOR]

```
"[PARK_NAME] opens to the sky. [YEAR]. A [BEAUTY_DESCRIPTOR] space for rest."
"Year [YEAR]. We plant the [PARK_NAME]. It is [BEAUTY_DESCRIPTOR] amidst the grey."
"The [PARK_NAME] is complete. [YEAR]. Beauty returns to [COLONY]."
```

### STATUE_RAISED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SUBJECT], [ART_TYPE], [BEAUTY_DESCRIPTOR]

```
"A [ART_TYPE] is raised. [YEAR]. It honors [SUBJECT]. [BEAUTY_DESCRIPTOR]."
"[YEAR]: We build a [ART_TYPE] for [SUBJECT]. A [BEAUTY_DESCRIPTOR] memory in stone."
"The [SUBJECT] [ART_TYPE] stands watch over [COLONY]. [YEAR]."
```

---

## Waste & Pollution Templates

### LANDFILL_FULL

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [WASTE_NAME], [HEAP_NAME]

```
"The [HEAP_NAME] is full. [YEAR]. [WASTE_NAME] spills over."
"[YEAR]: No more room for [WASTE_NAME]. The [HEAP_NAME] chokes [COLONY]."
"Warning: [HEAP_NAME] at capacity. [YEAR]. The filth rises."
```

### WASTE_SPILL

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [WASTE_NAME], [POLLUTION_DESCRIPTOR]

```
"Leak at [COLONY]. [YEAR]. [WASTE_NAME] spreads. It is [POLLUTION_DESCRIPTOR]."
"[YEAR]: The containment fails. [WASTE_NAME] everywhere. A [POLLUTION_DESCRIPTOR] stain."
"[COLONY] weeps [WASTE_NAME]. [YEAR]. The ground is [POLLUTION_DESCRIPTOR]."
```

---

## Skills & Mastery Templates

### MASTERY_ACHIEVED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [SKILL_TITLE], [SKILL_TYPE]

```
"[NAME] is now a [SKILL_TITLE] of [SKILL_TYPE]. [YEAR]. We are stronger."
"Year [YEAR]. [NAME] achieves mastery in [SKILL_TYPE]. A true [SKILL_TITLE]."
"The [SKILL_TITLE] [NAME]. [YEAR]. Unmatched in [SKILL_TYPE]."
```

### MASTERWORK_CREATED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [ITEM_NAME], [MASTERWORK_ADJECTIVE]

```
"[NAME] forges [ITEM_NAME]. [YEAR]. It is [MASTERWORK_ADJECTIVE]."
"A [MASTERWORK_ADJECTIVE] creation. [ITEM_NAME]. [NAME]'s hands are blessed. [YEAR]."
"[YEAR]: The [ITEM_NAME] is finished. [NAME] calls it [MASTERWORK_ADJECTIVE]."
```

---

## Relationship Templates

### BOND_FORMED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME_A], [NAME_B], [BOND_TYPE]

```
"[NAME_A] and [NAME_B]. [YEAR]. They are [BOND_TYPE] now."
"A bond forms. [YEAR]. [NAME_A], [NAME_B]. True [BOND_TYPE]."
"[YEAR]: [NAME_A] stands with [NAME_B]. [BOND_TYPE] in the dark."
```

### RIVALRY_STARTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME_A], [NAME_B], [RIVALRY_REASON]

```
"Bad blood between [NAME_A] and [NAME_B]. [YEAR]. Cause: [RIVALRY_REASON]."
"[YEAR]: [NAME_A] turns against [NAME_B]. An [RIVALRY_REASON]."
"Conflict in the ranks. [NAME_A] vs [NAME_B]. [YEAR]. [RIVALRY_REASON]."
```

---

## Science Templates

### ANOMALY_STUDIED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [ANOMALY_TYPE], [SCIENCE_ACTION]

```
"We found a [ANOMALY_TYPE]. [YEAR]. It was [SCIENCE_ACTION]."
"[YEAR]: Contact with [ANOMALY_TYPE]. We [SCIENCE_ACTION] it. Data secured."
"The [ANOMALY_TYPE] at [COLONY]. [YEAR]. We have [SCIENCE_ACTION] its secrets."
```

## Trade & Exchange Templates

### MERCHANT_ARRIVAL

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MERCHANT_TITLE]

```
"[MERCHANT_TITLE] arrives at [COLONY]. [YEAR]. The void brings gifts."
"Year [YEAR]. A ship in orbit. [MERCHANT_TITLE] hails us."
"Trade opportunity. [MERCHANT_TITLE] has docked at [COLONY]. [YEAR]."
```

### TRADE_COMPLETED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MERCHANT_TITLE], [RESOURCE_OUT], [RESOURCE_IN]

```
"Trade with [MERCHANT_TITLE] concluded. [YEAR]. We gave [RESOURCE_OUT], received [RESOURCE_IN]."
"[YEAR]: The exchange is made. [RESOURCE_OUT] for [RESOURCE_IN]. The books balance."
"[COLONY] prospers. [RESOURCE_IN] secured from [MERCHANT_TITLE]. Cost: [RESOURCE_OUT]. [YEAR]."
```

---

## Sound & Silence Templates

### NOISE_COMPLAINT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NOISE_DESCRIPTOR], [SOURCE]

```
"The Clamor grows. [YEAR]. [COLONY] cannot sleep. It is [NOISE_DESCRIPTOR]."
"[YEAR]: Complaints of [NOISE_DESCRIPTOR] noise from the [SOURCE]. The people are restless."
"Headaches and anger. The [SOURCE] is [NOISE_DESCRIPTOR]. [YEAR]."
```

### QUIET_MOMENT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [QUIET_DESCRIPTOR]

```
"Stillness at [COLONY]. [YEAR]. A [QUIET_DESCRIPTOR] moment amidst the work."
"[YEAR]: The machines stop. The silence is [QUIET_DESCRIPTOR]."
"Peace returns to [COLONY]. [YEAR]. It feels [QUIET_DESCRIPTOR]."
```

---

## Death & Rites Templates

### FUNERAL_HELD

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [FUNERAL_TYPE]

```
"[NAME] is returned to the void. [YEAR]. The [FUNERAL_TYPE] is spoken."
"[YEAR]: We gather at the barrow. [NAME]. A [FUNERAL_TYPE]."
"The earth takes back its own. [NAME]. [YEAR]. The [FUNERAL_TYPE] concludes."
```

---

## Law & Edicts Templates

### EDICT_ISSUED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [EDICT_NAME], [EDICT_VERB]

```
"The Word is spoken: [EDICT_NAME]. [YEAR]. It is [EDICT_VERB]."
"[YEAR]: New law. [EDICT_NAME] is [EDICT_VERB] at [COLONY]."
"The Substrate commands. [EDICT_NAME]. [YEAR]."
```

### EDICT_REVOKED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [EDICT_NAME]

```
"[EDICT_NAME] is rescinded. [YEAR]. The law changes."
"[YEAR]: We turn from [EDICT_NAME]. It is no longer the way."
"The Decree ends. [EDICT_NAME] is forgotten. [YEAR]."
```

## Structural Integrity Templates

### STRUCTURE_COLLAPSE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BUILDING_TYPE], [COLLAPSE_SOUND], [INJURY_COUNT]?

```
"The [BUILDING_TYPE] gives way. [YEAR]. A [COLLAPSE_SOUND] end."
"[YEAR]: Structural failure. The [BUILDING_TYPE] falls with a [COLLAPSE_SOUND] roar."
"Disaster at [COLONY]. The [BUILDING_TYPE] is gone. It was [COLLAPSE_SOUND]."
[If INJURY_COUNT:]
"The [BUILDING_TYPE] collapse takes [INJURY_COUNT] souls. [YEAR]. [COLLAPSE_SOUND]."
```

### RUIN_DISCOVERY

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [RUIN_CIV], [RUIN_AGE], [RUIN_STATE]

```
"[COLONY] surveyors report structures. [YEAR]. [RUIN_STATE]."
"They found [RUIN_CIV] beneath the soil of [COLONY]. Dead [RUIN_AGE] years. It is [RUIN_STATE]."
"Year [YEAR]: [COLONY] is not the first. [RUIN_CIV] was here. [RUIN_STATE]."
```

---

## Vermin & Pest Templates

### VERMIN_OUTBREAK

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [VERMIN_NAME], [VERMIN_ACTION]

```
"The [VERMIN_NAME] are here. [YEAR]. The swarm [VERMIN_ACTION]."
"[YEAR]: Infestation. [VERMIN_NAME] in the walls. It [VERMIN_ACTION]."
"They [VERMIN_ACTION] in the dark. [VERMIN_NAME] plague [COLONY]. [YEAR]."
```

### VERMIN_CLEARED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [VERMIN_NAME]

```
"The [VERMIN_NAME] are gone. [YEAR]. The silence returns."
"[YEAR]: We have purged the [VERMIN_NAME]. The stores are safe."
"Victory over the swarm. [VERMIN_NAME] eradicated at [COLONY]. [YEAR]."
```

### VERMIN_EVOLVED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [VERMIN_NAME], [VERMIN_VARIANT]

```
"The [VERMIN_NAME] are changing. [YEAR]. They are now [VERMIN_VARIANT]."
"[YEAR]: Mutation in the swarm. [VERMIN_NAME] become [VERMIN_VARIANT]."
"New threat: [VERMIN_VARIANT] [VERMIN_NAME]. Evolution at work. [YEAR]."
```

---

## Militia & Combat Templates

### MILITIA_MUSTER

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MILITIA_NAME], [WEAPON_NAME]

```
"The [MILITIA_NAME] forms. [YEAR]. Armed with [WEAPON_NAME]."
"[YEAR]: [COLONY] stands ready. The [MILITIA_NAME] raises its [WEAPON_NAME]."
"Defenders of [COLONY]. The [MILITIA_NAME] is born. [YEAR]."
```

### SKIRMISH_RESULT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MILITIA_NAME], [ENEMY], [OUTCOME]

```
"Battle at [COLONY]. [YEAR]. The [MILITIA_NAME] fought [ENEMY]. [OUTCOME]."
"[YEAR]: The [MILITIA_NAME] met the [ENEMY]. [OUTCOME]."
"Conflict report. [YEAR]. [MILITIA_NAME] vs [ENEMY]. [OUTCOME]."
```

---

## Fauna Templates

### FAUNA_SIGHTING

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BEAST_NAME], [BEAST_ACTION]

```
"[BEAST_NAME] spotted near [COLONY]. [YEAR]. It [BEAST_ACTION]."
"[YEAR]: The wild comes close. [BEAST_NAME]. It [BEAST_ACTION] us."
"Watchers report [BEAST_NAME]. [YEAR]. The pack [BEAST_ACTION]."
```

### FAUNA_ATTACK

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BEAST_NAME], [INJURY_COUNT]

```
"Attack at the perimeter. [YEAR]. [BEAST_NAME]. [INJURY_COUNT] hurt."
"[YEAR]: The [BEAST_NAME] breaches the line. [INJURY_COUNT] fall."
"Blood on the snow. [BEAST_NAME] raid. [YEAR]. [INJURY_COUNT] casualties."
```

---

## Energy & Power Templates

### POWER_OUTAGE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [POWER_SOURCE], [DURATION]

```
"The lights die. [YEAR]. The [POWER_SOURCE] fails. Darkness for [DURATION]."
"[YEAR]: Blackout. The [POWER_SOURCE] is silent. We wait in the dark."
"Power loss at [COLONY]. [YEAR]. The [POWER_SOURCE] sleeps. [DURATION] without the spark."
```

---

## Visitor Templates

### VISITOR_ARRIVAL

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [VISITOR_TYPE], [COUNT]

```
"A ship lands. [YEAR]. [COUNT] [VISITOR_TYPE]s step out."
"[YEAR]: Guests at [COLONY]. A group of [VISITOR_TYPE]s. [COUNT] souls."
"Strangers at the gate. [COUNT] [VISITOR_TYPE]s arrive. [YEAR]."
```

---

## Faction Templates

### FACTION_FORMED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [FACTION_NAME], [FOUNDER_NAME]

```
"[FACTION_NAME] is born. [YEAR]. [FOUNDER_NAME] speaks for them."
"[YEAR]: A new circle forms. They call themselves [FACTION_NAME]. [FOUNDER_NAME] leads."
"Division at [COLONY]. [FACTION_NAME] rises. [YEAR]."
```

---

## Atmosphere Templates

### ATMOSPHERE_EVENT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [ATMOSPHERE_DESCRIPTOR], [EFFECT]

```
"The air changes. [YEAR]. It tastes [ATMOSPHERE_DESCRIPTOR]. [EFFECT]."
"[YEAR]: Atmospheric shift. The breath becomes [ATMOSPHERE_DESCRIPTOR]. [EFFECT]."
"Warning: Air quality [ATMOSPHERE_DESCRIPTOR]. [YEAR]. [EFFECT] reported."
```

---

## Cabin Fever Templates

### CABIN_FEVER_BREAK

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [CABIN_FEVER_SYMPTOM]

```
"[NAME] breaks under the pressure. [YEAR]. They are [CABIN_FEVER_SYMPTOM]."
"[YEAR]: The walls are too close for [NAME]. [CABIN_FEVER_SYMPTOM]."
"Mental break reported. [NAME] is [CABIN_FEVER_SYMPTOM]. [YEAR]."
```

### CONFINEMENT_ALERT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CONFINEMENT_DESCRIPTOR]

```
"The lockdown continues. [YEAR]. The mood is [CONFINEMENT_DESCRIPTOR]."
"[YEAR]: Confinement protocol. We are trapped. It feels [CONFINEMENT_DESCRIPTOR]."
"Day after day inside. [COLONY] is [CONFINEMENT_DESCRIPTOR]. [YEAR]."
```

---

## Shift Work Templates

### SHIFT_CHANGE_DISPUTE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SHIFT_NAME], [SHIFT_COMPLAINT]

```
"Trouble on the [SHIFT_NAME]. [YEAR]. They cite [SHIFT_COMPLAINT]."
"[YEAR]: The [SHIFT_NAME] refuses to work. Reason: [SHIFT_COMPLAINT]."
"Dispute at shift change. [SHIFT_NAME] workers say they are [SHIFT_COMPLAINT]. [YEAR]."
```

---

## Weather Templates

### WEATHER_EVENT_START

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [WEATHER_TYPE], [WEATHER_INTENSITY]

```
"A [WEATHER_TYPE] hits [COLONY]. [YEAR]. It is [WEATHER_INTENSITY]."
"[YEAR]: Storm warning. [WEATHER_TYPE] approaches. [WEATHER_INTENSITY] winds."
"The sky turns dark. [WEATHER_TYPE] at [COLONY]. [YEAR]. [WEATHER_INTENSITY]."
```

### WEATHER_EVENT_END

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [WEATHER_TYPE]

```
"The [WEATHER_TYPE] passes. [YEAR]. The sky clears."
"[YEAR]: We survived the [WEATHER_TYPE]. It is over."
"Silence after the storm. The [WEATHER_TYPE] is gone from [COLONY]. [YEAR]."
```

### WEATHER_DAMAGE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [STORM_NAME], [DAMAGE_REPORT]

```
"[STORM_NAME] leaves its mark. [YEAR]. [DAMAGE_REPORT]."
"[YEAR]: Aftermath of [STORM_NAME]. We lost [DAMAGE_REPORT]."
"Rebuilding after [STORM_NAME]. [DAMAGE_REPORT]. [YEAR]."
```

---

## Omen & Taboo Templates

### OMEN_WITNESSED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [OMEN_TYPE]

```
"[NAME] saw [OMEN_TYPE]. [YEAR]. A bad sign."
"[YEAR]: Whispers of [OMEN_TYPE]. The colony is uneasy."
"An omen at [COLONY]. [NAME] reports [OMEN_TYPE]. [YEAR]."
```

### TABOO_BROKEN

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [TABOO_ACTION]

```
"[NAME] was caught [TABOO_ACTION]. [YEAR]. The others turned away."
"[YEAR]: A taboo broken. [NAME] is [TABOO_ACTION]. Bad luck will follow."
"Fear in the colony. [NAME] committed the error of [TABOO_ACTION]. [YEAR]."
```

---

## Inspector Templates

### INSPECTOR_ARRIVAL

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [INSPECTOR_TITLE]

```
"[INSPECTOR_TITLE] has arrived. [YEAR]. Look busy."
"[YEAR]: Inspection day. [INSPECTOR_TITLE] walks the halls."
"A shuttle lands. It carries [INSPECTOR_TITLE]. [YEAR]."
```

### INSPECTOR_JUDGMENT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [INSPECTOR_TITLE], [VERDICT]

```
"The report is in. [INSPECTOR_TITLE] calls us [VERDICT]. [YEAR]."
"[YEAR]: Judgment day. The colony is deemed [VERDICT] by [INSPECTOR_TITLE]."
"[INSPECTOR_TITLE] departs. The verdict: [VERDICT]. [YEAR]."
```

---

## Stowaway Templates

### STOWAWAY_DISCOVERED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [STOWAWAY_HIDING_SPOT]

```
"We found a stranger [STOWAWAY_HIDING_SPOT]. [YEAR]. They have been here for weeks."
"[YEAR]: Discovery. A stowaway was [STOWAWAY_HIDING_SPOT]."
"Security alert. Intruder found [STOWAWAY_HIDING_SPOT]. [YEAR]."
```

### THEFT_REPORT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [RESOURCE], [AMOUNT]

```
"Supplies missing. [YEAR]. [AMOUNT] [RESOURCE] gone without a trace."
"[YEAR]: Theft from the stores. We are short [AMOUNT] [RESOURCE]."
"Someone is stealing [RESOURCE]. [AMOUNT] lost. [YEAR]."
```

---

## Mood Templates

### PANIC_SPREAD

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MOOD_WAVE]

```
"Fear moves fast. [YEAR]. [MOOD_WAVE] takes the colony."
"[YEAR]: [MOOD_WAVE]. Work stops. Eyes are wide."
"A panic. [MOOD_WAVE] passes from soul to soul. [YEAR]."
```

### JOY_SPREAD

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MOOD_WAVE]

```
"Laughter in the halls. [YEAR]. [MOOD_WAVE]."
"[YEAR]: A lighter mood. [MOOD_WAVE] lifts us."
"The darkness breaks. [MOOD_WAVE] at [COLONY]. [YEAR]."
```

## Water Templates

### WATER_DISCOVERY

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [WATER_SOURCE_NAME], [RIVER_DESCRIPTOR]

- "[COLONY] finds the life-blood. [YEAR]. A [RIVER_DESCRIPTOR] [WATER_SOURCE_NAME]."
- "[YEAR]: Water. We name it [WATER_SOURCE_NAME]. It is [RIVER_DESCRIPTOR]."
- "Thirst ends at [COLONY]. [YEAR]. The [WATER_SOURCE_NAME] is found."

### FLOOD_EVENT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [WATER_SOURCE_NAME], [RIVER_DESCRIPTOR]

- "The [WATER_SOURCE_NAME] rises. [YEAR]. [RIVER_DESCRIPTOR] waters take the fields."
- "[YEAR]: Flood at [COLONY]. The water is [RIVER_DESCRIPTOR] and hungry."
- "[WATER_SOURCE_NAME] breaks its banks. [YEAR]. We are wet and cold."

## Husbandry Templates

### ANIMAL_TAMED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [ANIMAL_NAME], [TAME_ACTION]

- "We have [TAME_ACTION] the [ANIMAL_NAME]. [YEAR]. The herd grows."
- "[YEAR]: The [ANIMAL_NAME] joins us. [TAME_ACTION] by hand and food."
- "Livestock at [COLONY]. The [ANIMAL_NAME] is [TAME_ACTION]. [YEAR]."

### ANIMAL_BORN

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [ANIMAL_NAME]

- "New life in the pen. [YEAR]. A [ANIMAL_NAME] is born."
- "[YEAR]: The herd multiplies. A young [ANIMAL_NAME] takes its first breath."
- "Birth at [COLONY]. Small [ANIMAL_NAME], strong and loud. [YEAR]."

## Aging Templates

### ELDER_PASSING

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [ELDER_TITLE]

- "[NAME], our [ELDER_TITLE], has passed. [YEAR]. Time takes us all."
- "[YEAR]: We mourn [NAME]. The [ELDER_TITLE] sleeps now."
- "The clock stops for [NAME]. [YEAR]. Rest well, [ELDER_TITLE]."

### CHILD_BORN

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [YOUTH_TITLE]

- "A [YOUTH_TITLE] arrives. [YEAR]. We name them [NAME]."
- "[YEAR]: [NAME] is born. A [YOUTH_TITLE] for [COLONY]."
- "Cry in the night. [NAME]. [YEAR]. Our new [YOUTH_TITLE]."

## Sleepwalking Templates

### SLEEPWALKER_FOUND

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [DREAM_TYPE]

- "[NAME] was found walking the perimeter. [YEAR]. Chasing [DREAM_TYPE]."
- "[YEAR]: The Walking takes [NAME]. They sought [DREAM_TYPE] in their sleep."
- "We woke [NAME] near the edge. [YEAR]. They spoke of [DREAM_TYPE]."

## Planetary Quirk Templates

### QUIRK_REVEALED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [QUIRK_NAME]

- "We feel it now. [YEAR]. This world has [QUIRK_NAME]."
- "[YEAR]: The nature of the planet is clear. It is [QUIRK_NAME]."
- "Adapting to [QUIRK_NAME]. [YEAR]. [COLONY] endures."

---

## Private Stash Templates

### STASH_FOUND

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [STASH_LOCATION], [STASH_CONTAINER], [RESOURCE]

- "We found [NAME]'s secret. [YEAR]. A [STASH_CONTAINER] [STASH_LOCATION]. It held [RESOURCE]."
- "[YEAR]: Hoarding discovered. [NAME] hid [RESOURCE] in [STASH_CONTAINER]."
- "A [STASH_CONTAINER] found [STASH_LOCATION]. [NAME] was keeping [RESOURCE] for themselves. [YEAR]."

---

## Fuel & Industry Templates

### FUEL_PRODUCED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [FUEL_TYPE], [FUEL_SOURCE]

- "The tanks are full. [YEAR]. [FUEL_TYPE] from [FUEL_SOURCE]."
- "[YEAR]: We have [FUEL_TYPE]. The [FUEL_SOURCE] yields power."
- "Energy secured. [FUEL_TYPE] production begins at [COLONY]. [YEAR]."

### REFINERY_ACCIDENT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [REFINERY_NAME], [INJURY_TYPE]

- "Flash-fire at the [REFINERY_NAME]. [YEAR]. [INJURY_TYPE] reported."
- "[YEAR]: The mix was volatile. [REFINERY_NAME] breach. [INJURY_TYPE]."
- "Danger in the works. [REFINERY_NAME] accident. [YEAR]. [INJURY_TYPE]."

---

## Purity Templates

### PURITY_ANALYSIS

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [RESOURCE], [PURITY_LEVEL], [IMPURITY_TYPE]

- "Survey complete. [YEAR]. The [RESOURCE] is [PURITY_LEVEL]. Signs of [IMPURITY_TYPE]."
- "[YEAR]: [RESOURCE] quality report. It is [PURITY_LEVEL]. [IMPURITY_TYPE] detected."
- "Digging through [IMPURITY_TYPE] to find the [RESOURCE]. It is [PURITY_LEVEL]. [YEAR]."

---

## Jury-Rigging Templates

### JURY_RIG_EVENT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [BUILDING_TYPE], [JURY_RIG_METHOD], [JURY_RIG_MATERIAL]

- "[NAME] fixes the [BUILDING_TYPE]. [YEAR]. Used [JURY_RIG_METHOD] and [JURY_RIG_MATERIAL]."
- "[YEAR]: A patch-job on the [BUILDING_TYPE]. [NAME] applied [JURY_RIG_MATERIAL]."
- "The [BUILDING_TYPE] holds together. [NAME]'s [JURY_RIG_METHOD] worked. [YEAR]."

### JURY_RIG_FAILURE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BUILDING_TYPE], [JURY_RIG_MATERIAL]

- "The patch failed. [YEAR]. [BUILDING_TYPE] breaks again. The [JURY_RIG_MATERIAL] gave way."
- "[YEAR]: [BUILDING_TYPE] collapse. [JURY_RIG_MATERIAL] was not enough."
- "Temporary measures fail. [BUILDING_TYPE] down. [YEAR]."

---

## Greenhouse Templates

### GREENHOUSE_BUILT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GREENHOUSE_NAME], [GREENHOUSE_DESCRIPTOR]

- "[GREENHOUSE_NAME] is sealed. [YEAR]. A [GREENHOUSE_DESCRIPTOR] refuge."
- "[YEAR]: We build a glass sky. [GREENHOUSE_NAME]. It feels [GREENHOUSE_DESCRIPTOR]."
- "Life under glass. [GREENHOUSE_NAME] complete at [COLONY]. [YEAR]."

---

## Provenance Templates

### PROVENANCE_REVEALED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BUILDING_TYPE], [PROVENANCE_DESCRIPTOR]

- "The [BUILDING_TYPE] is finished. [YEAR]. Built of [PROVENANCE_DESCRIPTOR]."
- "[YEAR]: We live in [PROVENANCE_DESCRIPTOR] walls. The [BUILDING_TYPE] stands."
- "[COLONY] remembers. The [BUILDING_TYPE] is made of [PROVENANCE_DESCRIPTOR]. [YEAR]."

---

## Antagonistic Flora Templates

### FLORA_OUTBREAK

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [FLORA_NAME], [FLORA_ACTION], [FLORA_DESCRIPTOR]

- "The [FLORA_NAME] appears. [YEAR]. It [FLORA_ACTION]. It is [FLORA_DESCRIPTOR]."
- "[YEAR]: Infestation. The [FLORA_NAME] spreads. A [FLORA_DESCRIPTOR] growth."
- "[COLONY] fights the green. [FLORA_NAME]. [YEAR]. It [FLORA_ACTION] everything."

### FLORA_CLEARED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [FLORA_NAME]

- "We burn the [FLORA_NAME]. [YEAR]. The walls are clean."
- "[YEAR]: Victory over the [FLORA_NAME]. [COLONY] breathes again."
- "The roots are dead. [FLORA_NAME] eradicated. [YEAR]."

### STRUCTURE_STRANGLED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BUILDING_TYPE], [FLORA_NAME]

- "The [BUILDING_TYPE] is lost. [YEAR]. taken by [FLORA_NAME]."
- "[YEAR]: [FLORA_NAME] breaches the [BUILDING_TYPE]. We abandon it."
- "Choked by [FLORA_NAME]. The [BUILDING_TYPE] falls silent. [YEAR]."

---

## Observatory Templates

### OBSERVATORY_BUILT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [OBSERVATORY_NAME]

- "[OBSERVATORY_NAME] is open. [YEAR]. We look up."
- "[YEAR]: The lens is polished. [OBSERVATORY_NAME] sees the deep."
- "Eyes to the void. [OBSERVATORY_NAME] completed at [COLONY]. [YEAR]."

### COSMIC_EPIPHANY

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [COSMIC_SIGHT], [VOID_EMOTION]

- "[NAME] saw [COSMIC_SIGHT]. [YEAR]. They feel [VOID_EMOTION]."
- "[YEAR]: Inspiration from the dark. [NAME] witnessed [COSMIC_SIGHT]."
- "The void speaks to [NAME]. [COSMIC_SIGHT]. A moment of [VOID_EMOTION]. [YEAR]."

### VOID_GAZE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [COSMIC_SIGHT], [VOID_EMOTION]

- "[NAME] stared too long. [YEAR]. Saw [COSMIC_SIGHT]. Now: [VOID_EMOTION]."
- "[YEAR]: The abyss stares back. [NAME] is shaken by [COSMIC_SIGHT]."
- "Dread at [COLONY]. [NAME] reports [COSMIC_SIGHT]. [VOID_EMOTION]. [YEAR]."

---

## Mentorship Templates

### MENTORSHIP_STARTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MENTOR_NAME], [LEARNER_NAME], [MENTOR_TITLE]

- "[MENTOR_NAME] takes [LEARNER_NAME] as a student. [YEAR]. The [MENTOR_TITLE] teaches."
- "[YEAR]: A bond of learning. [MENTOR_NAME] and [LEARNER_NAME]. The path begins."
- "[MENTOR_NAME], the [MENTOR_TITLE], guides [LEARNER_NAME]. [YEAR]."

### LESSON_COMPLETED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MENTOR_NAME], [LEARNER_NAME], [LESSON_TOPIC]

- "[LEARNER_NAME] has learned [LESSON_TOPIC]. [YEAR]. Thanks to [MENTOR_NAME]."
- "[YEAR]: The lesson ends. [LESSON_TOPIC] passed from [MENTOR_NAME] to [LEARNER_NAME]."
- "Wisdom shared. [MENTOR_NAME] teaches [LESSON_TOPIC]. [LEARNER_NAME] grows. [YEAR]."

---

## Spontaneous Architecture Templates

### FOLLY_RAISED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [FOLLY_NAME], [FOLLY_PURPOSE]

- "[NAME] built something. [YEAR]. A [FOLLY_NAME]. Used [FOLLY_PURPOSE]."
- "[YEAR]: [FOLLY_NAME] appears. [NAME]'s work. [FOLLY_PURPOSE]."
- "Unauthorized construction. [NAME] makes a [FOLLY_NAME] [FOLLY_PURPOSE]. [YEAR]."

### FOLLY_DISCOVERED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [FOLLY_NAME], [FOLLY_DESCRIPTOR]

- "We found a [FOLLY_NAME]. [YEAR]. It is [FOLLY_DESCRIPTOR]."
- "[YEAR]: Hidden among the works. A [FOLLY_DESCRIPTOR] [FOLLY_NAME]."
- "Secret structure found. [FOLLY_NAME]. [FOLLY_DESCRIPTOR] and strange. [YEAR]."

---

## Logistics Templates

### LOGISTICS_JAM

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CONVEYOR_NAME], [JAM_DESCRIPTOR]

- "The [CONVEYOR_NAME] stops. [YEAR]. It is [JAM_DESCRIPTOR]."
- "[YEAR]: Production halted. [CONVEYOR_NAME] failure. [JAM_DESCRIPTOR]."
- "Silence on the line. The [CONVEYOR_NAME] is [JAM_DESCRIPTOR]. [YEAR]."

### FLOW_RESTORED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CONVEYOR_NAME]

- "The [CONVEYOR_NAME] moves again. [YEAR]. The blockage clears."
- "[YEAR]: Efficiency returns. [CONVEYOR_NAME] operational."
- "The hum of the [CONVEYOR_NAME]. Restored at [COLONY]. [YEAR]."

### LIGHT_INSTALLED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [LIGHT_SOURCE_NAME], [SHADOW_DESCRIPTOR]

- "First [LIGHT_SOURCE_NAME] in the sector. [YEAR]. Banish the [SHADOW_DESCRIPTOR] dark."
- "[YEAR]: We hang a [LIGHT_SOURCE_NAME]. The shadows were [SHADOW_DESCRIPTOR]."
- "Light brings hope. [LIGHT_SOURCE_NAME] lit. [YEAR]."

---

## Retrograde Engineering Templates

### TECH_DECONSTRUCTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [ARTIFACT_TYPE], [RETROGRADE_ACTION], [TECH_FLAW]

- "[NAME] [RETROGRADE_ACTION] the [ARTIFACT_TYPE]. [YEAR]. Found [TECH_FLAW]."
- "[YEAR]: We learn from the dead. [ARTIFACT_TYPE] [RETROGRADE_ACTION]. It had [TECH_FLAW]."
- "The [ARTIFACT_TYPE] is [RETROGRADE_ACTION]. [NAME] reports [TECH_FLAW]. [YEAR]."

### FLAW_DISCOVERED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [TECH_FLAW]

- "A warning from [NAME]. [YEAR]. The core suffers from [TECH_FLAW]."
- "[YEAR]: [TECH_FLAW] detected. We must be careful."
- "The logic is unsound. [TECH_FLAW]. [NAME] found it. [YEAR]."

---

## Penal Labor Templates

### PRISONER_ARRIVED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [PRISONER_TITLE], [CRIME]

- "New [PRISONER_TITLE] arrive. [YEAR]. Convicted of [CRIME]."
- "[YEAR]: The shuttle brings [PRISONER_TITLE]. Their debt is [CRIME]."
- "Chains and silence. [PRISONER_TITLE] for [CRIME]. [YEAR]."

### SENTENCE_SERVED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [PRISONER_TITLE]

- "[NAME] is free. [YEAR]. No longer [PRISONER_TITLE]."
- "[YEAR]: The debt is paid. [NAME] walks without chains."
- "Release day for [NAME]. The [PRISONER_TITLE] is a citizen. [YEAR]."

### PRISON_RIOT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [PRISONER_TITLE]

- "Uprising. [YEAR]. The [PRISONER_TITLE] break their bonds."
- "[YEAR]: Riot in the block. [PRISONER_TITLE] demand freedom."
- "Violence from the [PRISONER_TITLE]. [COLONY] locks down. [YEAR]."

---

## Technological Ritual Templates

### RITUAL_PERFORMED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [RITUAL_NAME]

- "[NAME] performs the [RITUAL_NAME]. [YEAR]. The machine hums."
- "[YEAR]: We observe the [RITUAL_NAME]. The spirits are listening."
- "Incense and oil. The [RITUAL_NAME] is complete. [YEAR]."

### SPIRIT_APPEASED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MACHINE_SPIRIT_MOOD]

- "The machine is [MACHINE_SPIRIT_MOOD]. [YEAR]. Production flows."
- "[YEAR]: Harmony. The spirit is [MACHINE_SPIRIT_MOOD]."
- "We are blessed. The core is [MACHINE_SPIRIT_MOOD]. [YEAR]."

### SPIRIT_ANGERED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MACHINE_SPIRIT_MOOD]

- "Warning signs. [YEAR]. The machine is [MACHINE_SPIRIT_MOOD]."
- "[YEAR]: The [RITUAL_NAME] failed. The spirit is [MACHINE_SPIRIT_MOOD]."
- "Red lights. The core is [MACHINE_SPIRIT_MOOD]. Run. [YEAR]."

---

## Social Mimicry Templates

### TREND_STARTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [TREND_NAME], [FASHION_ITEM]

- "[TREND_NAME] sweeps the colony. [YEAR]. Everyone wants [FASHION_ITEM]."
- "[YEAR]: It is the time of [TREND_NAME]. We wear [FASHION_ITEM]."
- "New style: [TREND_NAME]. [FASHION_ITEM] is the sign. [YEAR]."

### TREND_DIED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [TREND_NAME]

- "The [TREND_NAME] is over. [YEAR]. We move on."
- "[YEAR]: Nobody speaks of [TREND_NAME] anymore."
- "The fad ends. [TREND_NAME] is forgotten. [YEAR]."

---

## Colony Mascot Templates

### MASCOT_NAMED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MASCOT_TITLE], [NAME]

- "We have a [MASCOT_TITLE]. [YEAR]. Its name is [NAME]."
- "[YEAR]: Meet [NAME]. Our [MASCOT_TITLE]."
- "[NAME] joins the colony. The [MASCOT_TITLE] has arrived. [YEAR]."

### MASCOT_EVENT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [MASCOT_ACTION]

- "[NAME] [MASCOT_ACTION]. [YEAR]. We all laughed."
- "[YEAR]: Good omen. [NAME] [MASCOT_ACTION]."
- "The [MASCOT_TITLE] [MASCOT_ACTION]. Morale is high. [YEAR]."

### MASCOT_DEATH

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME]

- "[NAME] is gone. [YEAR]. The colony mourns."
- "[YEAR]: A dark day. We lost [NAME]."
- "Rest well, [NAME]. You were a good [MASCOT_TITLE]. [YEAR]."

---

## Security Templates

### ACCESS_DENIED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [LOCK_STATUS]

- "[NAME] hits the wall. [YEAR]. Access [LOCK_STATUS]."
- "[YEAR]: Security alert. [NAME] found the door [LOCK_STATUS]."
- "Denied. [NAME] cannot pass. The system is [LOCK_STATUS]. [YEAR]."

### LOCKOUT_OVERRIDE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [ACCESS_LEVEL]

- "[NAME] bypasses the lock. [YEAR]. Gained [ACCESS_LEVEL] clearance."
- "[YEAR]: Security breach. [NAME] forces [ACCESS_LEVEL] entry."
- "The door opens for [NAME]. [ACCESS_LEVEL] authorized. [YEAR]."

---

## Old Guard Templates

### GENERATION_CLASH

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GENERATION_NAME], [OLD_GUARD_TITLE]

- "Tension in the mess. [YEAR]. The [GENERATION_NAME] demand respect."
- "[YEAR]: Words between the new and the old. The [OLD_GUARD_TITLE] speaks."
- "Conflict of eras. [GENERATION_NAME] vs the new arrivals. [YEAR]."

### TRADITION_UPHELD

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [OLD_GUARD_TITLE], [GENERATION_NAME]

- "The [OLD_GUARD_TITLE]s gather. [YEAR]. They remember the Hunger."
- "[YEAR]: A council of the first. The [OLD_GUARD_TITLE] leads."
- "Whispers of the [OLD_GUARD_TITLE]. The [GENERATION_NAME] are plotting. [YEAR]."

---

## Vacuum Templates

### EMERGENCY_VENT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SUCTION_DESCRIPTOR]

- "Atmosphere vented. [YEAR]. The [SUCTION_DESCRIPTOR] pull clears the room."
- "[YEAR]: Emergency cycle. The air is gone. It was [SUCTION_DESCRIPTOR]."
- "Silence falls. We vented the sector. The vacuum is [SUCTION_DESCRIPTOR]. [YEAR]."

### HULL_BREACH

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [DECOMPRESSION_SOUND], [SUCTION_DESCRIPTOR]

- "Structure failure! [YEAR]. A [DECOMPRESSION_SOUND] and then silence."
- "[YEAR]: Breach. The [SUCTION_DESCRIPTOR] dark enters. [DECOMPRESSION_SOUND]."
- "We lost pressure. [YEAR]. The [DECOMPRESSION_SOUND] haunts us."

---

## Trash Cannon Templates

### CANNON_FIRED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CANNON_NAME], [PROJECTILE_TYPE]

- "[CANNON_NAME] fires. [YEAR]. Sending [PROJECTILE_TYPE] to the void."
- "[YEAR]: We clear the stores. The [CANNON_NAME] spits [PROJECTILE_TYPE]."
- "Defense active. [CANNON_NAME] launches [PROJECTILE_TYPE]. [YEAR]."

### AMMO_DEPLETED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CANNON_NAME]

- "[CANNON_NAME] clicks empty. [YEAR]. We need more waste."
- "[YEAR]: Silence from the [CANNON_NAME]. No ammo remains."
- "The [CANNON_NAME] is hungry. Feed it. [YEAR]."

---

## Bioluminescent Flora Templates

### GLOW_DISCOVERED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [LIGHT_PLANT_NAME], [GLOW_COLOR]

- "Soft light in the deep. [YEAR]. [LIGHT_PLANT_NAME] glowing [GLOW_COLOR]."
- "[YEAR]: We found [LIGHT_PLANT_NAME]. It shines [GLOW_COLOR]."
- "Nature's lamp. [LIGHT_PLANT_NAME] found at [COLONY]. [GLOW_COLOR] light. [YEAR]."

---

## Grid Instability Templates

### GRID_SURGE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GRID_SOUND], [POWER_FLUCTUATION]

- "Power spike! [YEAR]. The conduit makes a [GRID_SOUND]."
- "[YEAR]: Dangerous [POWER_FLUCTUATION]. The lights flare."
- "The grid is unstable. [POWER_FLUCTUATION] detected. It [GRID_SOUND]s. [YEAR]."

### BROWNOUT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [POWER_FLUCTUATION]

- "Lights dim. [YEAR]. A [POWER_FLUCTUATION] hits the sector."
- "[YEAR]: Low power. The machines slow. [POWER_FLUCTUATION]."
- "Energy drops. [POWER_FLUCTUATION] at [COLONY]. [YEAR]."


---

## Wild Child Templates

### WILD_CHILD_FOUND

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [FERAL_NAME], [WILD_ACTION]

- "[NAME] was found in the wastes. [YEAR]. We call them [FERAL_NAME]. They [WILD_ACTION]."
- "[YEAR]: A child in the wild. [NAME]. Known as [FERAL_NAME]. Found [WILD_ACTION]."
- "We brought [NAME] in from the cold. [YEAR]. The [FERAL_NAME] still [WILD_ACTION]."

### CHILD_GOES_FERAL

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [WILD_ACTION]

- "[NAME] is lost to the wild. [YEAR]. They [WILD_ACTION] at us now."
- "[YEAR]: The exposure took [NAME]. Feral. [WILD_ACTION]."
- "We lost a child to the wastes. [NAME] has turned. [YEAR]."

### CHILD_RECOVERED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME]

- "[NAME] returns to us. [YEAR]. The wild is washed away."
- "[YEAR]: Rehabilitation complete. [NAME] speaks again."
- "Saved from the feral state. [NAME] is civilized. [YEAR]."

---

## Blob Templates

### BLOB_SIGHTING

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BLOB_NAME], [BLOB_DESCRIPTOR]

- "[BLOB_NAME] spotted. [YEAR]. It is [BLOB_DESCRIPTOR]."
- "[YEAR]: The anomaly grows. [BLOB_NAME]. [BLOB_DESCRIPTOR] and moving."
- "Contact with [BLOB_NAME]. [YEAR]. A [BLOB_DESCRIPTOR] mass."

### BLOB_CONSUMPTION

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BLOB_NAME], [RESOURCE], [BLOB_ACTION]

- "The [BLOB_NAME] [BLOB_ACTION] our [RESOURCE]. [YEAR]. Nothing left."
- "[YEAR]: [RESOURCE] lost to the [BLOB_NAME]. It just [BLOB_ACTION] over it."
- "Feeding time. The [BLOB_NAME] takes the [RESOURCE]. [YEAR]."

### BLOB_DAMAGE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BLOB_NAME], [BUILDING_TYPE]

- "The [BLOB_NAME] crushes the [BUILDING_TYPE]. [YEAR]. Structure critical."
- "[YEAR]: [BUILDING_TYPE] breached by [BLOB_NAME]. We cannot stop it."
- "Destruction at [COLONY]. The [BLOB_NAME] eats the [BUILDING_TYPE]. [YEAR]."

---

## Cybernetics Templates

### SURGERY_COMPLETED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [PROSTHETIC_NAME], [SURGERY_OUTCOME]

- "[NAME] receives the [PROSTHETIC_NAME]. [YEAR]. The metal is [SURGERY_OUTCOME]."
- "[YEAR]: Upgrade complete. [NAME] is now part [PROSTHETIC_NAME]."
- "The flesh is weak. [NAME] chooses [PROSTHETIC_NAME]. [YEAR]. [SURGERY_OUTCOME]."

### SURGERY_FAILED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [PROSTHETIC_NAME]

- "Rejection. [NAME]'s body fights the [PROSTHETIC_NAME]. [YEAR]."
- "[YEAR]: Surgery failure. The [PROSTHETIC_NAME] will not seat."
- "[NAME] remains unchanged. The [PROSTHETIC_NAME] was incompatible. [YEAR]."

---

## Heirloom & Ancient Tech Templates

### HEIRLOOM_CREATED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [LEGENDARY_TOOL]

- "[NAME]'s tool is named [LEGENDARY_TOOL]. [YEAR]. It does not break."
- "[YEAR]: A legend is forged. [NAME] wields [LEGENDARY_TOOL]."
- "The [LEGENDARY_TOOL]. Born from [NAME]'s labor. [YEAR]."

### ANCIENT_DECAY

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [ANCIENT_STRUCTURE]

- "The [ANCIENT_STRUCTURE] is failing. [YEAR]. Time eats the metal."
- "[YEAR]: Warning from the [ANCIENT_STRUCTURE]. Systems dying."
- "Decay takes the [ANCIENT_STRUCTURE]. [YEAR]. We cannot fix it."

### RETROGRADE_SACRIFICE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [ANCIENT_STRUCTURE], [KNOWLEDGE_TOPIC]

- "We tore apart the [ANCIENT_STRUCTURE]. [YEAR]. Learned [KNOWLEDGE_TOPIC]."
- "[YEAR]: Sacrifice for knowledge. The [ANCIENT_STRUCTURE] is gone. We found [KNOWLEDGE_TOPIC]."
- "The [ANCIENT_STRUCTURE] gave its life for [KNOWLEDGE_TOPIC]. [YEAR]."

---

## Social Stratification Templates

### CLASS_FRICTION_EVENT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CLASS_NAME], [FRICTION_SOURCE]

- "Tension between the classes. [YEAR]. The [CLASS_NAME] complain of [FRICTION_SOURCE]."
- "[YEAR]: Unrest rises. [CLASS_NAME] vs the others. Cause: [FRICTION_SOURCE]."
- "The divide grows. [CLASS_NAME] are angry about [FRICTION_SOURCE]. [YEAR]."

### SOCIAL_PROMOTION

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [CLASS_NAME]

- "[NAME] rises to the [CLASS_NAME]. [YEAR]. They leave the old life behind."
- "[YEAR]: Status change. [NAME] is now [CLASS_NAME]."
- "Ascension. [NAME] joins the [CLASS_NAME]. [YEAR]."

---

## Tech Envy Templates

### TECH_ENVY_COMPLAINT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [TECH_ENVY_DESCRIPTOR]

- "[NAME] refuses to work. [YEAR]. Cites [TECH_ENVY_DESCRIPTOR] equipment."
- "[YEAR]: Morale drops. [NAME] calls our tech [TECH_ENVY_DESCRIPTOR]."
- "Demand for upgrades. [NAME] is tired of [TECH_ENVY_DESCRIPTOR] tools. [YEAR]."

---

## Justice & Sanctuary Templates

### SANCTUARY_DECLARED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SANCTUARY_NAME]

- "We draw the line. [YEAR]. [SANCTUARY_NAME] is established."
- "[YEAR]: The Free Zone is born. We call it [SANCTUARY_NAME]."
- "Law ends here. [SANCTUARY_NAME] declared at [COLONY]. [YEAR]."

### CRIMINAL_FLIGHT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [SANCTUARY_NAME], [CRIME]

- "[NAME] runs to [SANCTUARY_NAME]. [YEAR]. Wanted for [CRIME]."
- "[YEAR]: The law stops at the edge. [NAME] is safe in [SANCTUARY_NAME]."
- "Escape. [NAME] disappears into [SANCTUARY_NAME] to avoid judgment for [CRIME]. [YEAR]."

---

## Ecological Succession Templates

### SUCCESSION_STAGE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GROWTH_STAGE], [FOREST_DESCRIPTOR]

- "The green returns. [YEAR]. [GROWTH_STAGE] spotted in the ruins."
- "[YEAR]: Nature reclaims the stone. [GROWTH_STAGE] appears. It is [FOREST_DESCRIPTOR]."
- "Life finds a way. [GROWTH_STAGE] growth at [COLONY]. [YEAR]."

---

## Xeno-Artifact Templates

### ARTIFACT_AURA_FELT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [ARTIFACT_NAME], [AURA_EFFECT]

- "[NAME] stood too close to [ARTIFACT_NAME]. [YEAR]. Felt [AURA_EFFECT]."
- "[YEAR]: The [ARTIFACT_NAME] sings. [NAME] reports [AURA_EFFECT]."
- "Strange energies. [NAME] is touched by [AURA_EFFECT] from [ARTIFACT_NAME]. [YEAR]."

---

## Drone Templates

### DRONE_ACTIVATED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [DRONE_NAME], [DRONE_ACTION]

- "The [DRONE_NAME] comes online. [YEAR]. It [DRONE_ACTION]."
- "[YEAR]: New servitor. [DRONE_NAME]. [DRONE_ACTION] for the colony."
- "Mechanical life. [DRONE_NAME] joins the workforce. [YEAR]."

### DRONE_MALFUNCTION

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [DRONE_NAME], [DRONE_ACTION]

- "[DRONE_NAME] stops working. [YEAR]. It [DRONE_ACTION] strangely."
- "[YEAR]: Error in the logic. [DRONE_NAME] [DRONE_ACTION] instead of hauling."
- "Glitch report. [DRONE_NAME] is broken. [YEAR]."

---

## Graffiti Templates

### GRAFFITI_SPOTTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GRAFFITI_TEXT], [GRAFFITI_STYLE], [GRAFFITI_MEDIUM]

- "Words on the wall. [YEAR]. '[GRAFFITI_TEXT]'. Written in [GRAFFITI_MEDIUM]."
- "[YEAR]: Vandalism or warning? [GRAFFITI_STYLE] letters say '[GRAFFITI_TEXT]'."
- "Someone wrote '[GRAFFITI_TEXT]' in [GRAFFITI_MEDIUM]. [YEAR]."

---

## Cannibalization Templates

### SHIP_PART_SALVAGED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SHIP_COMPONENT], [CANNIBALIZE_ACTION], [SHIP_EMOTION]

- "We [CANNIBALIZE_ACTION] the [SHIP_COMPONENT]. [YEAR]. Felt [SHIP_EMOTION]."
- "[YEAR]: The ship gives us life. [SHIP_COMPONENT] is gone. [SHIP_EMOTION]."
- "Tearing down the past. [SHIP_COMPONENT] [CANNIBALIZE_ACTION]. [YEAR]."

### SHIP_GONE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SHIP_EMOTION]

- "The last of the ship is gone. [YEAR]. Only [SHIP_EMOTION] remains."
- "[YEAR]: No more hull. We are truly here now. [SHIP_EMOTION]."
- "The skeleton is picked clean. [YEAR]. [SHIP_EMOTION] silence."

---

## Geological Templates

### SEISMIC_TREMOR

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [QUAKE_DESCRIPTOR], [GROUND_SOUND]

- "The ground moves. [YEAR]. A [QUAKE_DESCRIPTOR] shake."
- "[YEAR]: Seismic alert. We hear a [GROUND_SOUND]. The earth is [QUAKE_DESCRIPTOR]."
- "Tremor at [COLONY]. [QUAKE_DESCRIPTOR] and loud. [YEAR]."

---

## Thermal Templates

### HEAT_SPIKE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [HEAT_SOURCE], [THERMAL_STATE]

- "Temperature rising. [YEAR]. The [HEAT_SOURCE] is [THERMAL_STATE]."
- "[YEAR]: Heat warning. [HEAT_SOURCE] pushes us to [THERMAL_STATE]."
- "Sweat and alarms. [HEAT_SOURCE] overload. [YEAR]."

### FREEZE_EVENT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [COLD_SOURCE], [THERMAL_STATE]

- "Cold snap. [YEAR]. The [COLD_SOURCE] makes it [THERMAL_STATE]."
- "[YEAR]: Frost on the walls. [COLD_SOURCE] breach. We are [THERMAL_STATE]."
- "Shivering in the dark. [COLD_SOURCE] brings the [THERMAL_STATE]. [YEAR]."

---

## Data Templates

### DATA_FOUND

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [DATA_CARRIER], [DATA_TYPE]

- "We found a [DATA_CARRIER]. [YEAR]. It contains [DATA_TYPE]."
- "[YEAR]: Information recovery. A [DATA_CARRIER] full of [DATA_TYPE]."
- "Secrets in the [DATA_CARRIER]. [DATA_TYPE] revealed. [YEAR]."

---

## Social Debt Templates

### FAVOR_CALLED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [FAVOR_TYPE], [DEBT_FEELING]

- "[NAME] calls in a [FAVOR_TYPE]. [YEAR]. It feels [DEBT_FEELING]."
- "[YEAR]: The debt is due. [NAME] demands payment. [FAVOR_TYPE]."
- "A [FAVOR_TYPE] is settled. [NAME] collects. [DEBT_FEELING]. [YEAR]."

---

## Chemical Templates (Spec 181)

### ADDICTION_CRISIS

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [CHEMICAL_NAME]

- "[NAME] is [ADDICTION_SLANG]. [YEAR]. The need for [CHEMICAL_NAME] takes over."
- "[YEAR]: Addiction. [NAME] is lost to the [CHEMICAL_NAME]. Signs of [WITHDRAWAL_SYMPTOM]."
- "We are losing [NAME]. [YEAR]. The [CHEMICAL_NAME] hunger is too strong."

### OVERDOSE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [CHEMICAL_NAME]

- "[NAME] took too much. [YEAR]. The [CHEMICAL_NAME] burned them out."
- "[YEAR]: Overdose at [COLONY]. [NAME] found with [CHEMICAL_NAME]. Silent."
- "A bad batch. [NAME] is gone. [YEAR]. The [CHEMICAL_NAME] claimed another."

---

## Wind Templates (Spec 182)

### HIGH_WIND_EVENT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [WIND_DESCRIPTOR]

- "The wind is [WIND_DESCRIPTOR] today. [YEAR]. It tears at the walls."
- "[YEAR]: Gale warning. A [WIND_DESCRIPTOR] blast hits the canyon."
- "No one walks outside. The air is [WIND_DESCRIPTOR]. [YEAR]."

### CANYON_FORMED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CANYON_NAME]

- "We built a [CANYON_NAME]. [YEAR]. The wind screams through it."
- "[YEAR]: New construction creates a draft. We call it [CANYON_NAME]."
- "The airflow changed. [YEAR]. [CANYON_NAME] is now a wind-tunnel."

---

## Geodetic Sentience Templates (Spec 183)

### STONE_MIGRATION

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [LIVING_STONE_NAME]

- "The [LIVING_STONE_NAME] moved in the night. [YEAR]. Closer to the heat."
- "[YEAR]: Creep report. [LIVING_STONE_NAME] shifting. It seeks company."
- "Watch the [LIVING_STONE_NAME]. It is waking. [YEAR]."

### GOLEM_RISES

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GOLEM_ACTION]

- "They gathered. [YEAR]. The stones [GOLEM_ACTION] as one."
- "[YEAR]: Golem formation! The rocks fuse and [GOLEM_ACTION]."
- "A monster of stone. [YEAR]. It [GOLEM_ACTION] through the stockpile."

---

## Orbital Debris Templates (Spec 184)

### LAUNCH_FAILURE_DEBRIS

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SHIP_NAME], [DEBRIS_TYPE]

- "Launch aborted. [YEAR]. [SHIP_NAME] hit by [DEBRIS_TYPE]."
- "[YEAR]: The [SHIP_NAME] is lost. Taken by the [ORBITAL_HAZARD]."
- "Orbit is closed. [DEBRIS_TYPE] strike on [SHIP_NAME]. [YEAR]."

### ORBITAL_IMPACT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [ORBITAL_HAZARD]

- "Impact warning. [YEAR]. The [ORBITAL_HAZARD] rains down."
- "[YEAR]: Shield breach. Debris from [ORBITAL_HAZARD] hits the station."
- "The sky is falling. [ORBITAL_HAZARD] clears the upper atmosphere. [YEAR]."

---

## Vacuum Welding Templates (Spec 185)

### STRUCTURE_WELDED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BUILDING_TYPE], [WELDING_TERM]

- "The [BUILDING_TYPE] is set. [YEAR]. It is [WELDING_TERM]."
- "[YEAR]: Construction complete. The vacuum makes it [PERMANENT_STRUCTURE_ADJECTIVE]."
- "No taking it back. The [BUILDING_TYPE] is [WELDING_TERM]. [YEAR]."

### DESTROY_DESIGNATION

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BUILDING_TYPE]

- "We had to destroy the [BUILDING_TYPE]. [YEAR]. It was fused solid."
- "[YEAR]: Demolition impossible. We blast the [BUILDING_TYPE] to dust."
- "Clearing the way. [BUILDING_TYPE] removed by force. [YEAR]."

---

## Bio-Architecture Templates (Spec 186)

### BIO_STRUCTURE_GROWN

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BIO_STRUCTURE_NAME]

- "The [BIO_STRUCTURE_NAME] is fully grown. [YEAR]. It pulses with life."
- "[YEAR]: We cultivate the [BIO_STRUCTURE_NAME]. A living wall."
- "Birth of a building. The [BIO_STRUCTURE_NAME] breathes. [YEAR]."

### BIO_STARVATION

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BIO_STRUCTURE_NAME]

- "The [BIO_STRUCTURE_NAME] is hungry. [YEAR]. It shivers."
- "[YEAR]: Starvation. The [BIO_STRUCTURE_NAME] begins to wither."
- "Feed the walls. The [BIO_STRUCTURE_NAME] is dying. [YEAR]."

### BIO_INFECTION

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BIO_STRUCTURE_NAME], [BIO_SICKNESS_SYMPTOM]

- "Sickness in the [BIO_STRUCTURE_NAME]. [YEAR]. It shows [BIO_SICKNESS_SYMPTOM]."
- "[YEAR]: Infection spread. The [BIO_STRUCTURE_NAME] turns hostile."
- "The rot takes the [BIO_STRUCTURE_NAME]. [BIO_SICKNESS_SYMPTOM]. [YEAR]."

## Cryo-Dream Templates (Spec 195)

### CRYO_WAKE_EPIPHANY

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [DREAM_IMAGE], [KNOWLEDGE_TOPIC]

- "[NAME] wakes from cryo with a vision. [YEAR]. Saw [DREAM_IMAGE]. Understood [KNOWLEDGE_TOPIC]."
- "[YEAR]: Epiphany. [NAME] dreamed of [DREAM_IMAGE] and woke knowing [KNOWLEDGE_TOPIC]."
- "The ice teaches. [NAME] brings [KNOWLEDGE_TOPIC] from a dream of [DREAM_IMAGE]. [YEAR]."

### CRYO_WAKE_NIGHTMARE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [NIGHTMARE_IMAGE]

- "[NAME] wakes screaming. [YEAR]. Haunted by [NIGHTMARE_IMAGE]."
- "[YEAR]: Trauma from the freeze. [NAME] cannot forget [NIGHTMARE_IMAGE]."
- "Something followed [NAME] from the sleep. [NIGHTMARE_IMAGE]. [YEAR]."

---

## Gastronomy Templates (Spec 166)

### MYSTERY_MEAL_COOKED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CHEF_NAME], [ALIEN_INGREDIENT], [MEAL_NAME_PREFIX], [MEAL_NAME_SUFFIX]

- "[CHEF_NAME] cooks the unknown. [YEAR]. Used [ALIEN_INGREDIENT] to make [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX]."
- "[YEAR]: Experiment in the kitchen. [CHEF_NAME] serves [ALIEN_INGREDIENT] as [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX]."
- "Dinner roulette. [CHEF_NAME]'s [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX] contains [ALIEN_INGREDIENT]. [YEAR]."

### RECIPE_MASTERED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CHEF_NAME], [MEAL_NAME_PREFIX], [MEAL_NAME_SUFFIX], [FLAVOR_PROFILE]

- "A breakthrough. [CHEF_NAME] perfects the [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX]. [YEAR]. It tastes [FLAVOR_PROFILE]."
- "[YEAR]: New staple. [CHEF_NAME]'s [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX] is [FLAVOR_PROFILE] and safe."
- "We feast tonight. [CHEF_NAME] has mastered the [FLAVOR_PROFILE] [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX]. [YEAR]."

### FOOD_POISONING

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [MEAL_NAME_PREFIX], [MEAL_NAME_SUFFIX], [POISON_SYMPTOM]

- "[NAME] ate the [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX]. [YEAR]. Now suffering [POISON_SYMPTOM]."
- "[YEAR]: Bad batch. [NAME] reports [POISON_SYMPTOM] after the [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX]."
- "Kitchen accident. The [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX] caused [POISON_SYMPTOM] in [NAME]. [YEAR]."

### XENO_DELICACY

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [MEAL_NAME_PREFIX], [MEAL_NAME_SUFFIX], [FLAVOR_PROFILE]

- "[NAME] loves the [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX]. [YEAR]. Calls it [FLAVOR_PROFILE]."
- "[YEAR]: A taste of home? No, [FLAVOR_PROFILE]. But [NAME] enjoys the [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX]."
- "Morale boost. The [MEAL_NAME_PREFIX] [MEAL_NAME_SUFFIX] is [FLAVOR_PROFILE] and filling. [YEAR]."

---

## Atmospheric Tides Templates (Spec 190)

### TIDE_HIGH

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [PRESSURE_DESC_HIGH], [TIDE_SOUND]

- "The pressure rises. [YEAR]. The air is [PRESSURE_DESC_HIGH]. Hear the [TIDE_SOUND]."
- "[YEAR]: High Tide. Movement slows. The atmosphere is [PRESSURE_DESC_HIGH]."
- "A [TIDE_SOUND] signals the crush. [YEAR]. Air becomes [PRESSURE_DESC_HIGH]."

### TIDE_LOW

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [PRESSURE_DESC_LOW], [TIDE_SOUND]

- "The pressure drops. [YEAR]. Air feels [PRESSURE_DESC_LOW]. [TIDE_SOUND] in the vents."
- "[YEAR]: Low Tide. We move fast in the [PRESSURE_DESC_LOW] air."
- "Gasping. [YEAR]. The atmosphere is [PRESSURE_DESC_LOW]. [TIDE_SOUND]."

---

## Festival Templates (Spec 077)

### FESTIVAL_START

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [FESTIVAL_NAME], [FESTIVAL_TYPE]

- "Today we celebrate [FESTIVAL_NAME]. [YEAR]. A grand [FESTIVAL_TYPE]."
- "[YEAR]: Work stops for [FESTIVAL_NAME]. Let the [FESTIVAL_TYPE] begin."
- "Remembrance. [FESTIVAL_NAME] starts at [COLONY]. [YEAR]."

### FESTIVAL_END

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [FESTIVAL_NAME], [CELEBRATION_ACTION]

- "[FESTIVAL_NAME] is over. [YEAR]. We [CELEBRATION_ACTION] and return to work."
- "[YEAR]: The lights dim on [FESTIVAL_NAME]. Good memories of [CELEBRATION_ACTION]."
- "Silence after the feast. [FESTIVAL_NAME] ends. [YEAR]."

---

## Radiation Templates (Spec 191)

### RADIATION_SICKNESS_DETECTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [RADIATION_SYMPTOM]

- "[NAME] is sick. [YEAR]. The glow bites. [RADIATION_SYMPTOM]."
- "[YEAR]: Radiation alert. [NAME] shows [RADIATION_SYMPTOM]."
- "Invisible poison. [NAME] has [RADIATION_SYMPTOM]. Check the shielding. [YEAR]."

### WARM_STONE_REFUGE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [WARM_STONE_DESC]

- "We huddle near the waste. [YEAR]. It is [WARM_STONE_DESC]."
- "[YEAR]: Using the ore for heat. [NAME] calls it [WARM_STONE_DESC]."
- "Dangerous comfort. The wall is [WARM_STONE_DESC]. [YEAR]."

---

## Crop Diversity Templates (Spec 120)

### FIRST_HARVEST_WHEAT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CROP_DESC_WHEAT]

- "First wheat brought in. [YEAR]. Stalks of [CROP_DESC_WHEAT]."
- "[YEAR]: Bread soon. The [CROP_DESC_WHEAT] is harvested."

### FIRST_HARVEST_POTATO

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CROP_DESC_POTATO]

- "We dig up the [CROP_DESC_POTATO]. [YEAR]. Winter food."
- "[YEAR]: Potato harvest. Baskets of [CROP_DESC_POTATO]."

### FIRST_HARVEST_RICE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CROP_DESC_RICE]

- "Rice paddies drained. [YEAR]. [CROP_DESC_RICE] for the stores."
- "[YEAR]: The [CROP_DESC_RICE] is ready. A wet harvest."

---

## Monument Templates (Spec 167)

### RUIN_SCAVENGED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [RUIN_NAME], [RUIN_DESCRIPTION]

- "We cleared the [RUIN_NAME]. [YEAR]. It was [RUIN_DESCRIPTION]."
- "[YEAR]: Scavengers pick the [RUIN_NAME] clean. Nothing left."
- "The [RUIN_NAME] is gone. [YEAR]. We reuse the stone."

---

## Auroral Templates (Spec 205)

### AURORA_SIGHTING

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [AURORA_COLOR], [AURORA_DESCRIPTOR]

- "The sky burns [AURORA_COLOR]. [YEAR]. A [AURORA_DESCRIPTOR] light."
- "[YEAR]: Magnetic storm. The [AURORA_COLOR] fire is [AURORA_DESCRIPTOR]."
- "We watch the [AURORA_DESCRIPTOR] dance. [AURORA_COLOR] waves. [YEAR]."

### AURORA_HARVEST

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [AURORA_COLOR], [POWER_AMOUNT]

- "The collectors are singing. [YEAR]. Drinking the [AURORA_COLOR] sky. [POWER_AMOUNT] gained."
- "[YEAR]: Harvest complete. The [AURORA_COLOR] storm filled the banks. [POWER_AMOUNT]."
- "Power from the void. [POWER_AMOUNT] harvested from the [AURORA_COLOR] bands. [YEAR]."

---

## Predictive Policing Templates (Spec 173)

### CRIME_PREDICTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [CRIME], [PREDICTION_SOURCE]

- "[NAME] was flagged by [PREDICTION_SOURCE]. [YEAR]. Intent to commit [CRIME]."
- "[YEAR]: The algorithm sees all. [NAME] marked for [CRIME] via [PREDICTION_SOURCE]."
- "Pre-crime alert. [NAME]. [CRIME]. Certainty high. [YEAR]."

### PREEMPTIVE_ARREST

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [CRIME]

- "[NAME] taken before the act. [YEAR]. The [CRIME] never happened."
- "[YEAR]: Arrest made. [NAME] is secure. The [CRIME] was prevented."
- "Justice is faster than thought. [NAME] detained for future [CRIME]. [YEAR]."

---

## Scrapcode Templates (Spec 178)

### SCRAPCODE_INFECTION

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GLITCH_TEXT], [BUILDING_TYPE]

- "The [BUILDING_TYPE] is speaking in tongues. [YEAR]. Screens show '[GLITCH_TEXT]'."
- "[YEAR]: Malware in the core. [BUILDING_TYPE] output corrupted. '[GLITCH_TEXT]'."
- "Digital rot. The [BUILDING_TYPE] fails. [GLITCH_TEXT]. [YEAR]."

---

## Totem Templates (Spec 200)

### TOTEM_CRAFTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [TOTEM_MATERIAL], [TOTEM_SHAPE]

- "[NAME] made a charm. [YEAR]. A [TOTEM_SHAPE] of [TOTEM_MATERIAL]."
- "[YEAR]: Superstition or shield? [NAME] carries a [TOTEM_MATERIAL] [TOTEM_SHAPE]."
- "Protection forged. [NAME]'s [TOTEM_SHAPE]. [TOTEM_MATERIAL]. [YEAR]."

### TOTEM_LOST

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [TOTEM_SHAPE]

- "[NAME] lost their [TOTEM_SHAPE]. [YEAR]. The luck is gone."
- "[YEAR]: Bad omen. The [TOTEM_SHAPE] is missing. [NAME] is afraid."
- "Panic. [NAME] cannot find the [TOTEM_SHAPE]. [YEAR]."

---

## Food Preservation Templates (Spec 087)

### RATIONS_PRESERVED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CURED_FOOD_NAME], [SMOKE_WOOD]

- "The smokehouse is full. [YEAR]. [CURED_FOOD_NAME] cured with [SMOKE_WOOD]."
- "[YEAR]: Winter stores ready. [CURED_FOOD_NAME] stacks high. Smells of [SMOKE_WOOD]."
- "Preserving the kill. [CURED_FOOD_NAME]. [SMOKE_WOOD] smoke. [YEAR]."

---

## Institutional Memory Templates (Spec 172)

### ARCHIVE_DISCOVERY

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [ARCHIVE_SECTION], [KNOWLEDGE_TOPIC]

- "Deep in the [ARCHIVE_SECTION], we found it. [YEAR]. Notes on [KNOWLEDGE_TOPIC]."
- "[YEAR]: Data recovery. [ARCHIVE_SECTION] yielded [KNOWLEDGE_TOPIC]."
- "The past speaks. [KNOWLEDGE_TOPIC] found in [ARCHIVE_SECTION]. [YEAR]."

### LOST_KNOWLEDGE_RECOVERED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [KNOWLEDGE_TOPIC]

- "We remember how. [YEAR]. [KNOWLEDGE_TOPIC] is known again."
- "[YEAR]: The gap is filled. [KNOWLEDGE_TOPIC] restored to the index."
- "No longer lost. [KNOWLEDGE_TOPIC]. [YEAR]."

---

## Escape Pod Templates (Spec 217)

### ESCAPE_POD_LAUNCH

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [POD_NAME], [EVACUATION_REASON], [COUNT]

- "[POD_NAME] away. [YEAR]. Carrying [COUNT] souls. Reason: [EVACUATION_REASON]."
- "[YEAR]: Evacuation event. [POD_NAME] launches. [COUNT] flee the [EVACUATION_REASON]."
- "We sent [COUNT] into the dark. [POD_NAME] is gone. [YEAR]. [EVACUATION_REASON]."

### COLONY_EVACUATED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [EVACUATION_REASON], [SURVIVOR_COUNT]

- "Abandon ship order. [YEAR]. [COLONY] is empty. [SURVIVOR_COUNT] escaped."
- "[YEAR]: The silence falls on [COLONY]. We fled the [EVACUATION_REASON]."
- "[COLONY] is a tomb now. [EVACUATION_REASON] took it. [SURVIVOR_COUNT] survivors in pods. [YEAR]."

---

## Planetary Core Tap Templates (Spec 212)

### CORE_TAP_ACTIVATED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [POWER_SOURCE]

- "The [POWER_SOURCE] is live. [YEAR]. Infinite energy flows."
- "[YEAR]: We touched the heart. [POWER_SOURCE] active. The ground shakes."
- "Limitless power. [POWER_SOURCE] online at [COLONY]. [YEAR]."

### CORE_STRESS_WARNING

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CORE_STRESS_LEVEL], [CORE_ACTIVITY]

- "Seismic alert. [YEAR]. The core is [CORE_ACTIVITY]. [CORE_STRESS_LEVEL]."
- "[YEAR]: The price of power. [CORE_STRESS_LEVEL]. Core status: [CORE_ACTIVITY]."
- "Warning from the deep. [CORE_STRESS_LEVEL]. The [POWER_SOURCE] makes the world [CORE_ACTIVITY]. [YEAR]."

---

## Solar Cycle Templates (Spec 213)

### SOLAR_CYCLE_CHANGE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SOLAR_PHASE], [SOLAR_INTENSITY]

- "The sun changes face. [YEAR]. Entering [SOLAR_PHASE]. Light is [SOLAR_INTENSITY]."
- "[YEAR]: Solar cycle shift. It is the time of [SOLAR_PHASE]. [SOLAR_INTENSITY] days ahead."
- "New phase: [SOLAR_PHASE]. The star burns [SOLAR_INTENSITY]. [YEAR]."

---

## Customs Checkpoint Templates (Spec 214)

### CONTRABAND_SEIZED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [VISITOR_TYPE], [CONTRABAND_ITEM]

- "[VISITOR_TYPE] stopped at the gate. [YEAR]. Carrying [CONTRABAND_ITEM]."
- "[YEAR]: Seizure. We found [CONTRABAND_ITEM] on a [VISITOR_TYPE]."
- "Security intercept. [CONTRABAND_ITEM] confiscated. [YEAR]."

### VISITOR_DENIED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [VISITOR_TYPE], [DENIAL_REASON]

- "Entry refused. [YEAR]. [VISITOR_TYPE] turned away. Cause: [DENIAL_REASON]."
- "[YEAR]: Gate closed to [VISITOR_TYPE]. [DENIAL_REASON]."
- "We sent the [VISITOR_TYPE] back. [DENIAL_REASON]. [YEAR]."

### VISITOR_VETTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [VISITOR_TYPE]

- "[VISITOR_TYPE] cleared for entry. [YEAR]. Welcome to [COLONY]."
- "[YEAR]: Vetting complete. New [VISITOR_TYPE] joins us."
- "The gate opens. [VISITOR_TYPE] processed. [YEAR]."

---

## Ammunition Logistics Templates (Spec 210)

### AMMO_SHORTAGE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [AMMO_TYPE], [WEAPON_NAME]

- "Dry click. [YEAR]. No [AMMO_TYPE] for the [WEAPON_NAME]."
- "[YEAR]: Defense critical. We are out of [AMMO_TYPE]."
- "The [WEAPON_NAME] is silent. Shortage of [AMMO_TYPE]. [YEAR]."

### TURRET_RELOADED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [WEAPON_NAME], [AMMO_TYPE]

- "[WEAPON_NAME] fed. [YEAR]. [AMMO_TYPE] loaded."
- "[YEAR]: Ready to fire. [WEAPON_NAME] topped up with [AMMO_TYPE]."
- "Defense active. [AMMO_TYPE] in the [WEAPON_NAME]. [YEAR]."

---

## Planetary Governance Templates (Spec 209)

### GOVERNOR_APPOINTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GOVERNOR_TITLE], [NAME]

- "[NAME] takes the chair. [YEAR]. Our new [GOVERNOR_TITLE]."
- "[YEAR]: Leadership change. [NAME] is [GOVERNOR_TITLE]."
- "The [GOVERNOR_TITLE] speaks. [NAME] leads [COLONY]. [YEAR]."

### POLICY_ENACTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [POLICY_NAME], [GOVERNOR_TITLE]

- "[POLICY_NAME] signed into law. [YEAR]. By order of the [GOVERNOR_TITLE]."
- "[YEAR]: New rule. [POLICY_NAME] takes effect."
- "The [GOVERNOR_TITLE] decrees [POLICY_NAME]. [YEAR]."

---

## Safehouse Templates (Spec 215)

### SAFEHOUSE_CONTRACT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SAFEHOUSE_NAME], [VISITOR_TYPE]

- "Contract signed. [YEAR]. The [SAFEHOUSE_NAME] shelters a [VISITOR_TYPE]."
- "[YEAR]: Hidden guest. [VISITOR_TYPE] in the [SAFEHOUSE_NAME]."
- "Secret deal. The [SAFEHOUSE_NAME] is active. [YEAR]."

---

## Hygiene Templates (Spec 220)

### FILTH_OUTBREAK

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [FILTH_DESCRIPTOR], [VERMIN_NAME]

- "The Grime is winning. [YEAR]. Walls are [FILTH_DESCRIPTOR]. [VERMIN_NAME] thriving."
- "[YEAR]: Hygiene collapse. Everything is [FILTH_DESCRIPTOR]. We need water."
- "Squalor report. [COLONY] is [FILTH_DESCRIPTOR]. [VERMIN_NAME] breed in the dirt. [YEAR]."

### SHOWER_BUILT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SHOWER_NAME]

- "[SHOWER_NAME] is open. [YEAR]. The water runs clean."
- "[YEAR]: We wash away the grime. [SHOWER_NAME] installed."
- "Purity restored. [SHOWER_NAME] operational at [COLONY]. [YEAR]."

---

## Recycling Templates (Spec 221)

### RECYCLER_OPERATIONAL

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [RECYCLER_NAME]

- "The [RECYCLER_NAME] hums. [YEAR]. Nothing wasted."
- "[YEAR]: Green cycle started. [RECYCLER_NAME] takes the refuse."
- "New law: all waste to the [RECYCLER_NAME]. [YEAR]."

### CORPSE_RECYCLED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [RECYCLER_NAME], [CORPSE_PRODUCT]

- "[NAME] returns to the cycle. [YEAR]. The [RECYCLER_NAME] yields [CORPSE_PRODUCT]."
- "[YEAR]: Pragmatism. We processed [NAME] into [CORPSE_PRODUCT]."
- "The dead feed the living. [NAME] is now [CORPSE_PRODUCT]. [YEAR]."

---

## Fleet Templates (Spec 157/159)

### SHIP_CONSTRUCTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SHIP_CLASS], [SHIP_NAME]

- "New [SHIP_CLASS] launched. [YEAR]. Christened [SHIP_NAME]."
- "[YEAR]: The shipyard births [SHIP_NAME]. A proud [SHIP_CLASS]."
- "Void-ready. [SHIP_NAME] ([SHIP_CLASS]) joins the fleet. [YEAR]."

### FLEET_ENGAGEMENT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [FLEET_NAME], [COMBAT_RESULT]

- "Battle in the dark. [YEAR]. [FLEET_NAME] reports [COMBAT_RESULT]."
- "[YEAR]: Combat logs from [FLEET_NAME]. It ended in [COMBAT_RESULT]."
- "War comes to the void. [FLEET_NAME] engagement. [COMBAT_RESULT]. [YEAR]."

---

## Barnacle Templates (Spec 219)

### BARNACLE_INFESTATION

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SHIP_NAME], [BARNACLE_NAME]

- "[SHIP_NAME] is dragging. [YEAR]. Hull covered in [BARNACLE_NAME]."
- "[YEAR]: Parasites detected. [BARNACLE_NAME] on the [SHIP_NAME]."
- "Scrub the hull! [BARNACLE_NAME] infestation on [SHIP_NAME]. [YEAR]."

### DRAG_WARNING

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SHIP_NAME], [BARNACLE_ACTION]

- "Efficiency drops. [YEAR]. The barnacles [BARNACLE_ACTION] the [SHIP_NAME]."
- "[YEAR]: [SHIP_NAME] slowed by the infestation. They [BARNACLE_ACTION] deep."
- "Fuel usage critical. The parasites [BARNACLE_ACTION]. [YEAR]."

---

## Crossfire Templates (Spec 206)

### ORBITAL_BOMBARDMENT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CROSSFIRE_SOURCE], [IMPACT_DESCRIPTOR]

- "Sky-fire! [YEAR]. [CROSSFIRE_SOURCE] hits the surface. [IMPACT_DESCRIPTOR]."
- "[YEAR]: We are under fire. [CROSSFIRE_SOURCE]. A [IMPACT_DESCRIPTOR] rain."
- "Shields failing. [CROSSFIRE_SOURCE] bombardment. [IMPACT_DESCRIPTOR]. [YEAR]."

### CROSSFIRE_HIT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BUILDING_TYPE], [CROSSFIRE_SOURCE]

- "Direct hit on [BUILDING_TYPE]. [YEAR]. [CROSSFIRE_SOURCE] took it out."
- "[YEAR]: The [BUILDING_TYPE] is gone. Victim of [CROSSFIRE_SOURCE]."
- "Collateral damage. [CROSSFIRE_SOURCE] destroyed the [BUILDING_TYPE]. [YEAR]."

---

## Mother Lode Templates (Spec 168)

### MOTHER_LODE_FOUND

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MOTHER_LODE_NAME]

- "We found the big one. [YEAR]. [MOTHER_LODE_NAME]."
- "[YEAR]: Infinite wealth. The [MOTHER_LODE_NAME] is real."
- "Strike! [MOTHER_LODE_NAME] discovered at [COLONY]. [YEAR]."

### MOTHER_LODE_DEPLETED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MOTHER_LODE_NAME]

- "The [MOTHER_LODE_NAME] is dry. [YEAR]. Impossible."
- "[YEAR]: End of an era. [MOTHER_LODE_NAME] exhausted."
- "Silence in the deep mines. [MOTHER_LODE_NAME] gives no more. [YEAR]."

---

## Heat Island Templates (Spec 198)

### HEAT_ISLAND_WARNING

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [HEAT_ISLAND_DESCRIPTOR]

- "The city is [HEAT_ISLAND_DESCRIPTOR]. [YEAR]. Heat trapped in the streets."
- "[YEAR]: Thermal warning. Urban core is [HEAT_ISLAND_DESCRIPTOR]."
- "We are cooking ourselves. [HEAT_ISLAND_DESCRIPTOR] temperatures in [COLONY]. [YEAR]."

## Pneumatic Templates (Spec 228)

### TUBE_JAM

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CLOG_REASON], [TUBE_SOUND]

- "The system stops. [YEAR]. A [TUBE_SOUND] and then silence. [CLOG_REASON]."
- "[YEAR]: Logistics halt. The tubes are blocked by [CLOG_REASON]."
- "Pressure warning. [CLOG_REASON] detected in the line. Hear the [TUBE_SOUND]. [YEAR]."

---

## Keystone Templates (Spec 225)

### KEYSTONE_DEATH

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [KEYSTONE_NAME], [COLLAPSE_SIGN]

- "The [KEYSTONE_NAME] is dead. [YEAR]. Now we see [COLLAPSE_SIGN]."
- "[YEAR]: Ecological failure. We lost the [KEYSTONE_NAME]. [COLLAPSE_SIGN] begins."
- "A pillar falls. [KEYSTONE_NAME]. [YEAR]. The land shows [COLLAPSE_SIGN]."

### ECOSYSTEM_COLLAPSE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [COLLAPSE_SIGN]

- "The web unravels. [YEAR]. [COLLAPSE_SIGN] everywhere."
- "[YEAR]: Total failure. The biome is dying. [COLLAPSE_SIGN]."
- "We broke the world. [COLLAPSE_SIGN]. [YEAR]."

---

## Gene Bank Templates (Spec 165)

### SAMPLE_DEGRADED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GENE_SAMPLE_TYPE], [PRESERVATION_METHOD]

- "Loss in the vault. [YEAR]. [GENE_SAMPLE_TYPE] ruined. It was [PRESERVATION_METHOD]."
- "[YEAR]: Genetic drift. The [PRESERVATION_METHOD] [GENE_SAMPLE_TYPE] is viable no longer."
- "Memory fades. [GENE_SAMPLE_TYPE] lost to time. [YEAR]."

### ANCIENT_DNA_FOUND

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GENE_SAMPLE_TYPE], [PRESERVATION_METHOD]

- "Discovery. [YEAR]. [PRESERVATION_METHOD] [GENE_SAMPLE_TYPE] found."
- "[YEAR]: A seed from the past. [GENE_SAMPLE_TYPE]. We can rebuild."
- "Life finds a way. [GENE_SAMPLE_TYPE] recovered at [COLONY]. [YEAR]."

---

## Paperwork Templates (Spec 222)

### PAPERWORK_LOST

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [FORM_TYPE], [BUREAUCRATIC_ACTION]

- "Administration failure. [YEAR]. [FORM_TYPE] was [BUREAUCRATIC_ACTION]."
- "[YEAR]: The work stops. We cannot find the [FORM_TYPE]. It is [BUREAUCRATIC_ACTION]."
- "Red tape. [FORM_TYPE] [BUREAUCRATIC_ACTION]. Delays expected. [YEAR]."

### FORM_REJECTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [FORM_TYPE], [NAME]

- "[NAME]'s [FORM_TYPE] is denied. [YEAR]. Incorrect stamp."
- "[YEAR]: Bureaucracy strikes. [NAME] failed to file the [FORM_TYPE]."
- "Permission refused. [FORM_TYPE] required. [NAME] is frustrated. [YEAR]."

---

## Volatile Templates (Spec 223)

### VOLATILE_DECAY

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [VOLATILE_NAME], [EXPLOSION_COLOR]

- "The [VOLATILE_NAME] is sweating. [YEAR]. Glowing [EXPLOSION_COLOR]."
- "[YEAR]: Stability critical. [VOLATILE_NAME] degrading."
- "Danger. [VOLATILE_NAME] emits [EXPLOSION_COLOR] light. Run. [YEAR]."

### EXPLOSION_EVENT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [VOLATILE_NAME], [EXPLOSION_COLOR]

- "Boom. [YEAR]. [VOLATILE_NAME] goes critical. A [EXPLOSION_COLOR] flash."
- "[YEAR]: Detonation. The [VOLATILE_NAME] took the lab. [EXPLOSION_COLOR] smoke."
- "We lost containment. [VOLATILE_NAME]. [EXPLOSION_COLOR] fire everywhere. [YEAR]."

---

## Corrosive Atmosphere Templates (Spec 227)

### STRUCTURAL_DISSOLUTION

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MELTING_OBJECT], [CORROSION_SOUND]

- "The [MELTING_OBJECT] is gone. [YEAR]. Dissolved with a [CORROSION_SOUND]."
- "[YEAR]: Acid rain damage. [MELTING_OBJECT] melted away."
- "Structural integrity failing. [MELTING_OBJECT] eaten by the air. [CORROSION_SOUND]. [YEAR]."

### ACID_RAIN_EVENT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CORROSION_SOUND]

- "Sky burn. [YEAR]. The rain makes a [CORROSION_SOUND]."
- "[YEAR]: Take cover. Acid storm. Everything sizzles."
- "The clouds weep acid. [CORROSION_SOUND] on the roof. [YEAR]."

---

## Atmospheric Processor Templates (Spec 207)

### PROCESSOR_ONLINE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [PROCESSOR_NAME]

- "The [PROCESSOR_NAME] roars to life. [YEAR]. The long work begins."
- "[YEAR]: Ignition. The [PROCESSOR_NAME] starts pulling the poison."
- "A deep thrum across the colony. The [PROCESSOR_NAME] is online. [YEAR]."

### ATMOSPHERE_IMPROVED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [PROCESSOR_NAME], [AIR_QUALITY]

- "A change in the wind. [YEAR]. The air is [AIR_QUALITY], thanks to the [PROCESSOR_NAME]."
- "[YEAR]: The sky lightens. The [PROCESSOR_NAME] makes breathing [AIR_QUALITY]."
- "We can step outside. The [PROCESSOR_NAME] is working. The air is [AIR_QUALITY]. [YEAR]."

### PROCESSOR_STARVED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [PROCESSOR_NAME], [POWER_SOURCE]

- "The [PROCESSOR_NAME] falls silent. [YEAR]. No energy from the [POWER_SOURCE]."
- "[YEAR]: Power failure. The [PROCESSOR_NAME] stops spinning. The poison returns."
- "Silence from the [PROCESSOR_NAME]. [YEAR]. We lack the [POWER_SOURCE] to run it."

---

## Generational Hoarders Templates (Spec 284)

### HOARD_DISCOVERED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [HOARDED_ITEM], [HOARDER_JUSTIFICATION]

- "We opened [NAME]'s quarters. [YEAR]. Piles of [HOARDED_ITEM]. They said [HOARDER_JUSTIFICATION]."
- "[YEAR]: The space is gone. [NAME] filled it with [HOARDED_ITEM]. Claimed [HOARDER_JUSTIFICATION]."
- "Logistics failure. [NAME] hid the [HOARDED_ITEM]. Their excuse: [HOARDER_JUSTIFICATION]. [YEAR]."

### HOARD_CONFISCATED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [HOARDED_ITEM]

- "We took the [HOARDED_ITEM] from [NAME]. [YEAR]. They wept."
- "[YEAR]: Confiscation order. [NAME] loses their [HOARDED_ITEM]. Morale drops."
- "The stash is cleared. [NAME] stares at the empty wall, missing their [HOARDED_ITEM]. [YEAR]."

---

## Echoes of the Past Templates (Spec 285)

### GHOST_SIGHTING

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GHOST_APPEARANCE]

- "They saw it again. [YEAR]. A figure, [GHOST_APPEARANCE]."
- "[YEAR]: The ruins are restless. A projection, [GHOST_APPEARANCE], walks the halls."
- "Echoes in the dark. A shape [GHOST_APPEARANCE]. We are not alone. [YEAR]."

### ANCIENT_SECRET_REVEALED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [ANCIENT_SECRET], [GHOST_APPEARANCE]

- "The phantom showed us. [YEAR]. [ANCIENT_SECRET], revealed by a figure [GHOST_APPEARANCE]."
- "[YEAR]: A truth from the dead. We found [ANCIENT_SECRET] following the one [GHOST_APPEARANCE]."
- "The past speaks. [ANCIENT_SECRET] uncovered. [YEAR]."

---

## Symbiotic Shipyards Templates

### LIVING_SHIP_BORN

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [LIVING_SHIP_NAME], [GESTATION_STAGE]

- "The [LIVING_SHIP_NAME] reaches [GESTATION_STAGE]. [YEAR]. It breathes."
- "[YEAR]: Gestation complete. Our [LIVING_SHIP_NAME] is born into the void."
- "Flesh and star-metal. The [LIVING_SHIP_NAME] is at [GESTATION_STAGE]. [YEAR]."

### SHIP_STARVATION

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [LIVING_SHIP_NAME]

- "The [LIVING_SHIP_NAME] is hungry. [YEAR]. It groans in the dock."
- "[YEAR]: Biomass shortage. The [LIVING_SHIP_NAME] feeds on its own hull."
- "We cannot feed the fleet. The [LIVING_SHIP_NAME] weakens. [YEAR]."

---

## Orbital Tethers as Weapons Templates

### TETHER_SNAPPED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [TETHER_NAME], [DESTRUCTION_SCALE]

- "The [TETHER_NAME] falls! [YEAR]. A [DESTRUCTION_SCALE] impact across the equator."
- "[YEAR]: The line is cut. The [TETHER_NAME] whips the surface. [DESTRUCTION_SCALE] ruin."
- "We lost the sky. The [TETHER_NAME] collapses. It was [DESTRUCTION_SCALE]. [YEAR]."

### TETHER_SACRIFICE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [TETHER_NAME], [ENEMY]

- "We dropped the [TETHER_NAME] on the [ENEMY]. [YEAR]. We are grounded, but safe."
- "[YEAR]: Desperate measures. The [TETHER_NAME] weaponized against [ENEMY]."
- "The ultimate strike. [TETHER_NAME] severed to crush the [ENEMY]. [YEAR]."

---

## The Empathy Plague Templates

### EMPATHY_PLAGUE_START

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SHARED_EMOTION], [MIND_LINK_SYMPTOM]

- "The sickness links us. [YEAR]. We all feel [SHARED_EMOTION]. People are [MIND_LINK_SYMPTOM]."
- "[YEAR]: One mind. The colony shares [SHARED_EMOTION]. We are [MIND_LINK_SYMPTOM]."
- "No secrets anymore. The plague brings [SHARED_EMOTION]. [MIND_LINK_SYMPTOM]. [YEAR]."

### CASCADE_BREAKDOWN

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SHARED_EMOTION]

- "One broke, and we all fell. [YEAR]. A wave of [SHARED_EMOTION] took the colony."
- "[YEAR]: Neural cascade. [SHARED_EMOTION] paralyzes the workforce."
- "Shared agony. [SHARED_EMOTION] sweeps the link. [YEAR]."

---

## Counterfeit Reality Templates

### HOLO_FLEET_PROJECTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [HOLO_ILLUSION]

- "The projectors hum. [YEAR]. The sky is filled with [HOLO_ILLUSION]."
- "[YEAR]: Deception active. We broadcast [HOLO_ILLUSION] to the void."
- "Hiding behind light. A [HOLO_ILLUSION] shields [COLONY]. [YEAR]."

### BLUFF_CALLED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [HOLO_ILLUSION], [BLUFF_OUTCOME]

- "The enemy saw the [HOLO_ILLUSION]. [YEAR]. And [BLUFF_OUTCOME]."
- "[YEAR]: The illusion of [HOLO_ILLUSION] is tested. Result: [BLUFF_OUTCOME]."
- "They looked at our [HOLO_ILLUSION]. [BLUFF_OUTCOME]. [YEAR]."

---

## Template: DOPPELGANGER_SUSPECTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [MIMIC_SUSPICION]

**Patterns:**
- "Whispers about [NAME]. [YEAR]. Someone said they [MIMIC_SUSPICION]."
- "[YEAR]: Paranoia. [NAME] is acting strange. Last night they [MIMIC_SUSPICION]."
- "We are watching [NAME]. [YEAR]. They [MIMIC_SUSPICION]. Are they still human?"

## Template: DOPPELGANGER_REVEALED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [MIMIC_REVEAL]

**Patterns:**
- "It wasn't [NAME]. [YEAR]. The mimic was [MIMIC_REVEAL]."
- "[YEAR]: The imposter is dead. [NAME] was a fake. It [MIMIC_REVEAL]."
- "We killed the thing wearing [NAME]'s face. [YEAR]. It [MIMIC_REVEAL]."

---

## Template: HYPNO_SESSION_COMPLETE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [HYPNO_SUBJECT]

**Patterns:**
- "[NAME] wakes from the pod. [YEAR]. They now know [HYPNO_SUBJECT]."
- "[YEAR]: Instant mastery. [NAME] learned [HYPNO_SUBJECT] in a single sleep."
- "The machine taught [NAME] [HYPNO_SUBJECT]. [YEAR]. Knowledge without time."

## Template: HYPNO_FOG_ONSET

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [FOG_SYMPTOM]

**Patterns:**
- "[NAME] is wandering the halls. [YEAR]. A victim of the fog, [FOG_SYMPTOM]."
- "[YEAR]: The price of quick learning. [NAME] is [FOG_SYMPTOM]."
- "The pod took a toll. [NAME] is [FOG_SYMPTOM] today. [YEAR]."

---

## Template: PLACEBO_ADMINISTERED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [PLACEBO_NAME], [PLACEBO_EFFECT]

**Patterns:**
- "The doctor gave [NAME] [PLACEBO_NAME]. [YEAR]. They [PLACEBO_EFFECT]."
- "[YEAR]: A trick of the mind. [NAME] took [PLACEBO_NAME] and [PLACEBO_EFFECT]."
- "We cured the stress with a lie. [NAME] [PLACEBO_EFFECT] after the [PLACEBO_NAME]. [YEAR]."

---

## Template: MONUMENT_DEFACED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [ART_TYPE], [VANDALISM_ACT], [VANDAL_MESSAGE]

**Patterns:**
- "The [ART_TYPE] was [VANDALISM_ACT]. [YEAR]. The message: [VANDAL_MESSAGE]."
- "[YEAR]: Anger in the streets. The [ART_TYPE] is [VANDALISM_ACT]. They say [VANDAL_MESSAGE]."
- "Disrespect for the past. The [ART_TYPE] was [VANDALISM_ACT] tonight. [VANDAL_MESSAGE]. [YEAR]."

---

## Template: SHADOW_TRADE_MADE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SHADOW_MERCHANT], [SHADOW_GOODS]

**Patterns:**
- "A deal in the dark. [YEAR]. [SHADOW_MERCHANT] sold us [SHADOW_GOODS]."
- "[YEAR]: The lights were out. We bought [SHADOW_GOODS] from [SHADOW_MERCHANT]."
- "Taxes avoided. The [SHADOW_MERCHANT] provided [SHADOW_GOODS]. [YEAR]."

---

## Template: GREAT_WORK_STARTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GREAT_WORK_NAME], [WORK_PHASE_DESC]

**Patterns:**
- "We lay the foundation for [GREAT_WORK_NAME]. [YEAR]. It will be [WORK_PHASE_DESC]."
- "[YEAR]: The massive project begins. [GREAT_WORK_NAME]. [WORK_PHASE_DESC]."
- "Ambition takes form. [GREAT_WORK_NAME] started at [COLONY]. [YEAR]."

## Template: GREAT_WORK_FINISHED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GREAT_WORK_NAME]

**Patterns:**
- "It is done. [GREAT_WORK_NAME] stands complete. [YEAR]."
- "[YEAR]: A monumental achievement. The [GREAT_WORK_NAME] is operational."
- "We have left our mark. [GREAT_WORK_NAME] is finished. [YEAR]."

---

## Template: HARMONIC_DRILL_USED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [FREQUENCY_DESC], [RESONANCE_TARGET]

**Patterns:**
- "The drill sings. [YEAR]. A [FREQUENCY_DESC] sound shatters the [RESONANCE_TARGET]."
- "[YEAR]: Acoustic mining. The [FREQUENCY_DESC] wave turns [RESONANCE_TARGET] to dust."
- "Breaking [RESONANCE_TARGET] with pure sound. [FREQUENCY_DESC]. [YEAR]."

---

## Template: GLIDER_FLIGHT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GLIDER_NAME], [GLIDER_MANEUVER]

**Patterns:**
- "The [GLIDER_NAME] takes to the air. [YEAR]. It [GLIDER_MANEUVER]."
- "[YEAR]: Fast logistics. A [GLIDER_NAME] [GLIDER_MANEUVER] over the colony."
- "Watching the [GLIDER_NAME] fly. [YEAR]. It [GLIDER_MANEUVER] perfectly."

---

## Template: AD_CAMPAIGN_STARTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [AD_SLOGAN], [AD_IMPACT]

**Patterns:**
- "The screens light up. [YEAR]. They say '[AD_SLOGAN]'. The result: [AD_IMPACT]."
- "[YEAR]: A new broadcast begins. '[AD_SLOGAN]'. It brings [AD_IMPACT]."
- "We are told to '[AD_SLOGAN]'. The screens never sleep. [YEAR]. They leave us with [AD_IMPACT]."

---

## Template: SUBSPACE_MESSAGE_RECEIVED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [PEN_PAL_NAME], [PEN_PAL_TOPIC]

**Patterns:**
- "A signal cuts through the static. [YEAR]. [NAME] speaks with [PEN_PAL_NAME] about [PEN_PAL_TOPIC]."
- "[YEAR]: The void is less lonely. [PEN_PAL_NAME] reaches out to talk of [PEN_PAL_TOPIC]."
- "[NAME] finds a friend in the dark. [YEAR]. [PEN_PAL_NAME]. They discuss [PEN_PAL_TOPIC]."

---

## Template: POP_DENOUNCED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [DENUNCIATION_REASON], [SCAPEGOAT_FATE]

**Patterns:**
- "The mob turns on [NAME]. [YEAR]. Accused of [DENUNCIATION_REASON]. They were [SCAPEGOAT_FATE]."
- "[YEAR]: Someone must pay. [NAME] is blamed for [DENUNCIATION_REASON]. The result: [SCAPEGOAT_FATE]."
- "Anger finds a target. [NAME] is accused of [DENUNCIATION_REASON] and [SCAPEGOAT_FATE]. [YEAR]."

---

## Template: TEMPORAL_RIFT_OPENED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [RIFT_APPEARANCE], [RESOURCE], [TEMPORAL_DEBT]

**Patterns:**
- "[RIFT_APPEARANCE] in the lower decks. [YEAR]. It gave us [RESOURCE], but we owe [TEMPORAL_DEBT]."
- "[YEAR]: We borrow from tomorrow. [RIFT_APPEARANCE] yields [RESOURCE]. Now we bear [TEMPORAL_DEBT]."
- "A gift from the future. [RIFT_APPEARANCE]. [RESOURCE] gained, but [TEMPORAL_DEBT] looms. [YEAR]."

---

## Template: NOBLE_ARRIVAL

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [NOBLE_TITLE], [NOBLE_DEMAND]

**Patterns:**
- "A shuttle lands. [YEAR]. [NAME], a [NOBLE_TITLE], arrives demanding [NOBLE_DEMAND]."
- "[YEAR]: The core worlds send their dregs. [NAME], the [NOBLE_TITLE], expects [NOBLE_DEMAND]."
- "We must host [NAME], a [NOBLE_TITLE] from the core. [YEAR]. Their first order: [NOBLE_DEMAND]."
## Template: SPORE_OUTBREAK

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SPORE_NAME], [INFECTION_SPEED]

**Patterns:**
- "The [SPORE_NAME] came with the last shipment. [YEAR]. It spreads [INFECTION_SPEED]."
- "[YEAR]: Quarantine failed. We found [SPORE_NAME] in the vents. It moves [INFECTION_SPEED]."
- "They didn't check the cargo. Now [SPORE_NAME] is here. Growth is [INFECTION_SPEED]. [YEAR]."

## Template: QUARANTINE_BURN

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SPORE_NAME], [ZONE_NAME]

**Patterns:**
- "We had to burn [ZONE_NAME]. [YEAR]. The only way to stop the [SPORE_NAME]."
- "[YEAR]: Fire is the only cure. [ZONE_NAME] was lost to the [SPORE_NAME]."
- "The ashes of [ZONE_NAME] smell like [SPORE_NAME]. [YEAR]. We saved the rest."

## Template: STOWAWAY_DISCOVERED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [HIDING_SPOT]

**Patterns:**
- "We found [NAME] in the [HIDING_SPOT]. [YEAR]. They aren't on the manifest."
- "[YEAR]: An extra mouth. [NAME] fell out of the [HIDING_SPOT]."
- "Security breach. [NAME] survived the journey hidden in the [HIDING_SPOT]. [YEAR]."

## Template: KNOWLEDGE_LOST

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [TECH_FIELD], [OLD_EXPERT]

**Patterns:**
- "The old ways die. [YEAR]. Without [OLD_EXPERT], nobody understands [TECH_FIELD] anymore."
- "[YEAR]: Regression. The [TECH_FIELD] manuals look like gibberish to the new generation."
- "We stare at the machines. [OLD_EXPERT] took the secrets of [TECH_FIELD] to the grave. [YEAR]."

## Template: LEADER_PROMOTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [NEW_ROLE]

**Patterns:**
- "From the dirt to the stars. [YEAR]. [NAME] is our new [NEW_ROLE]."
- "[YEAR]: Ascension. [NAME] leaves the colony behind to become a [NEW_ROLE]."
- "We lose a worker, but gain a [NEW_ROLE]. [NAME] looks to the sky. [YEAR]."

## Template: BIOME_ENCROACHMENT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [PLANT_NAME], [LOST_BUILDING]

**Patterns:**
- "The forest takes it back. [YEAR]. The [LOST_BUILDING] is covered in [PLANT_NAME]."
- "[YEAR]: Maintenance failure. [PLANT_NAME] roots destroyed the [LOST_BUILDING]."
- "Green creeping death. The [LOST_BUILDING] belongs to the [PLANT_NAME] now. [YEAR]."

---

## Template: BIO_ACOUSTIC_CHORUS_HEARD

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CHORUS_TUNE]

**Patterns:**
- "The flora sings to us. [YEAR]. A [CHORUS_TUNE] sweeps the colony."
- "[YEAR]: We hear the voice of the forest. The tone is [CHORUS_TUNE]."
- "A [CHORUS_TUNE] resonates through the walls. The plants are humming. [YEAR]."

## Template: SCENT_OVERWHELM

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SCENT_DESCRIPTOR], [SMELL_SOURCE]

**Patterns:**
- "The air is heavy. [YEAR]. A [SCENT_DESCRIPTOR] smell from the [SMELL_SOURCE]."
- "[YEAR]: We cannot breathe. The [SMELL_SOURCE] emits a [SCENT_DESCRIPTOR] odor."
- "A [SCENT_DESCRIPTOR] stench grips [COLONY]. The culprit: [SMELL_SOURCE]. [YEAR]."

## Template: DROP_POD_SCATTERED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [DROP_PAYLOAD], [SCATTER_DISTANCE]

**Patterns:**
- "Supplies arrived. [YEAR]. But the [DROP_PAYLOAD] scattered [SCATTER_DISTANCE] away."
- "[YEAR]: Orbital drop failure. The [DROP_PAYLOAD] landed [SCATTER_DISTANCE] off target."
- "We have the [DROP_PAYLOAD]. But it's [SCATTER_DISTANCE] deep in the wild. [YEAR]."


---

## Gut Biome Templates (Spec 211)

### DIET_CHANGE_SICKNESS

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [DIET_TYPE], [INDIGESTION_SYMPTOM]

**Patterns:**
- "[NAME] ate the new [DIET_TYPE]. [YEAR]. Now suffering from [INDIGESTION_SYMPTOM]."
- "[YEAR]: The colony shifts to [DIET_TYPE]. [NAME] reports severe [INDIGESTION_SYMPTOM]."
- "Our bodies forgot how to digest it. [NAME] is down with [INDIGESTION_SYMPTOM] after the [DIET_TYPE] meal. [YEAR]."

### GUT_COMFORT_ACHIEVED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [DIET_TYPE]

**Patterns:**
- "[NAME] finally feels full. [YEAR]. The [DIET_TYPE] is sitting well."
- "[YEAR]: Adaptation. [NAME] thrives on the [DIET_TYPE] diet."
- "The gut settles. [NAME] calls the [DIET_TYPE] a comfort. [YEAR]."

---

## Kinetic Storage Templates (Spec 235)

### BATTERY_CHARGED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [KINETIC_BATTERY_NAME]

**Patterns:**
- "The [KINETIC_BATTERY_NAME] is fully hoisted. [YEAR]. Potential energy maxed."
- "[YEAR]: Excess power stored. The [KINETIC_BATTERY_NAME] hangs heavy in the sky."
- "The winch stops. [KINETIC_BATTERY_NAME] is ready for the dark. [YEAR]."

### BATTERY_COLLAPSE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [KINETIC_BATTERY_NAME], [KINETIC_IMPACT], [DAMAGE_REPORT]

**Patterns:**
- "The tether snapped! [YEAR]. The [KINETIC_BATTERY_NAME] falls with [KINETIC_IMPACT]. [DAMAGE_REPORT]."
- "[YEAR]: Catastrophic failure at the [KINETIC_BATTERY_NAME]. [KINETIC_IMPACT]. [DAMAGE_REPORT]."
- "We stored too much anger. The [KINETIC_BATTERY_NAME] crashes down. [KINETIC_IMPACT]. Loss: [DAMAGE_REPORT]. [YEAR]."

---

## The Direct Link Templates (Spec 236)

### LINK_ESTABLISHED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [DIRECT_LINK_FEELING]

**Patterns:**
- "The Commander takes the wheel. [YEAR]. [NAME] experiences [DIRECT_LINK_FEELING]."
- "[YEAR]: Override confirmed. [NAME] stands still, then moves with [DIRECT_LINK_FEELING]."
- "Guidance becomes control. [NAME] reports [DIRECT_LINK_FEELING]. [YEAR]."

### LINK_SEVERED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [LINK_ACTION]

**Patterns:**
- "The connection drops. [YEAR]. [NAME] remembers [LINK_ACTION] but not why."
- "[YEAR]: Autonomy restored. [NAME] is exhausted after [LINK_ACTION]."
- "The Commander withdraws. [NAME] blinks, unsure how they survived [LINK_ACTION]. [YEAR]."

---

## Public Grievances Templates (Spec 233)

### GRIEVANCE_POSTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [GRIEVANCE_TOPIC], [NOTE_STYLE]

**Patterns:**
- "A note on the board. [YEAR]. [NAME] complaining about [GRIEVANCE_TOPIC]. It is [NOTE_STYLE]."
- "[YEAR]: Public anger. [NAME] left a message about [GRIEVANCE_TOPIC], [NOTE_STYLE]."
- "The board speaks. [NAME] is furious over [GRIEVANCE_TOPIC]. The writing is [NOTE_STYLE]. [YEAR]."

### COMMENDATION_POSTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [NOTE_STYLE]

**Patterns:**
- "Rare praise on the board. [YEAR]. [NAME] left a thank you, [NOTE_STYLE]."
- "[YEAR]: A positive note from [NAME]. [NOTE_STYLE]."
- "Someone is happy. [NAME] posted a commendation, [NOTE_STYLE]. [YEAR]."

---

## Clone Vats Templates (Spec 240)

### CLONE_DECANTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CLONE_NAME_PREFIX], [NAME]

**Patterns:**
- "The vats open. [YEAR]. [CLONE_NAME_PREFIX]-[NAME] takes their first breath as an adult."
- "[YEAR]: Industrial birth. [CLONE_NAME_PREFIX]-[NAME] joins the line."
- "We don't wait for children anymore. [CLONE_NAME_PREFIX]-[NAME] decanted today. [YEAR]."

### CLONE_DISCRIMINATION

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [CLONE_STIGMA]

**Patterns:**
- "Whispers in the mess hall. [YEAR]. They call [NAME] [CLONE_STIGMA]."
- "[YEAR]: Tension between the born and the made. [NAME] is shunned for being [CLONE_STIGMA]."
- "The natural-born don't trust [NAME]. Say they look [CLONE_STIGMA]. [YEAR]."

---

## Legacy Code Templates (Spec 246)

### SYSTEM_BLOAT_WARNING

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SYSTEM_SLOWNESS]

**Patterns:**
- "The mainframe is choking on history. [YEAR]. [SYSTEM_SLOWNESS]."
- "[YEAR]: The logic pathways are clogged with old protocols. [SYSTEM_SLOWNESS]."
- "We are drowning in our own data. [SYSTEM_SLOWNESS]. [YEAR]."

### REFORMAT_INITIATED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [LEGACY_ERROR]

**Patterns:**
- "We hit a [LEGACY_ERROR]. [YEAR]. Initiating full reformat. The grid will be offline."
- "[YEAR]: The system crashed due to [LEGACY_ERROR]. Reformat required. Brace for the dark."
- "Wiping the slates clean after a [LEGACY_ERROR]. Reformat starting. [YEAR]."

---

## Ghost Code Templates (Spec 247)

### GHOST_CODE_MANIFESTS

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BUILDING_TYPE], [GHOST_CODE_GLITCH], [RESIDUE_TYPE]

**Patterns:**
- "The new [BUILDING_TYPE] is acting strange. [YEAR]. [GHOST_CODE_GLITCH]. It was built over [RESIDUE_TYPE]."
- "[YEAR]: Machine haunting. The [BUILDING_TYPE] inherited logic from [RESIDUE_TYPE]. Now [GHOST_CODE_GLITCH]."
- "We didn't clear the data residue from the [RESIDUE_TYPE]. The [BUILDING_TYPE] is infected. [GHOST_CODE_GLITCH]. [YEAR]."

### RESIDUE_PURGED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [RESIDUE_TYPE]

**Patterns:**
- "[NAME] scrubs the floor and the local net. [YEAR]. [RESIDUE_TYPE] data purged."
- "[YEAR]: The memory of the [RESIDUE_TYPE] is finally erased by [NAME]."
- "Clean sector. [NAME] removed the ghost code of the [RESIDUE_TYPE]. [YEAR]."

---

## Thermal Bloom Templates (Spec 243)

### THERMAL_BLOOM_DETECTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [THERMAL_WARNING]

**Patterns:**
- "The industry runs too hot! [YEAR]. [THERMAL_WARNING]."
- "[YEAR]: We are throwing too much heat into the void. [THERMAL_WARNING]."
- "Thermal signature critical. [THERMAL_WARNING]. We are visible. [YEAR]."

### HEAT_SINK_ATTACK

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [HOSTILE_INTERCEPT]

**Patterns:**
- "Our heat drew them in. [YEAR]. [HOSTILE_INTERCEPT] detected on approach."
- "[YEAR]: The thermal bloom was a flare. [HOSTILE_INTERCEPT] is moving to intercept."
- "They saw the glow of our forges. [HOSTILE_INTERCEPT] inbound. [YEAR]."

---

## The Infinite Archive Templates (Spec 248)

### ARCHIVE_PARALYSIS

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [ARCHIVE_BLOAT]

**Patterns:**
- "Research halts. [YEAR]. The servers are full. [ARCHIVE_BLOAT]."
- "[YEAR]: We cannot store another byte. [ARCHIVE_BLOAT]. Science is paralyzed."
- "The index is broken. [ARCHIVE_BLOAT]. No new knowledge until we delete the old. [YEAR]."

### KNOWLEDGE_PURGED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [DELETED_KNOWLEDGE]

**Patterns:**
- "Making room for the future. [YEAR]. We deleted [DELETED_KNOWLEDGE]."
- "[YEAR]: A hard choice. [DELETED_KNOWLEDGE] wiped from the databanks to clear space."
- "History sacrificed for progress. [DELETED_KNOWLEDGE] is gone. [YEAR]."

---

## Quantum Twins Templates (Spec 245)

### TWIN_SYNC_EVENT

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME_A], [NAME_B], [TWIN_SENSATION]

**Patterns:**
- "[NAME_A] learned to mine, and across the base, [NAME_B] felt [TWIN_SENSATION]. [YEAR]."
- "[YEAR]: The entanglement holds. [NAME_A] smiled, and [NAME_B] experienced [TWIN_SENSATION]."
- "Shared soul. When [NAME_A] was hurt, [NAME_B] reported [TWIN_SENSATION]. [YEAR]."

### SEVERANCE_SHOCK

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME_SURVIVOR], [SEVERANCE_TRAUMA]

**Patterns:**
- "The link is broken. [YEAR]. [NAME_SURVIVOR] collapses in [SEVERANCE_TRAUMA]."
- "[YEAR]: Their twin died in the dark. [NAME_SURVIVOR] suffers [SEVERANCE_TRAUMA]."
- "Half a soul remains. [NAME_SURVIVOR] is lost to [SEVERANCE_TRAUMA] after the severance. [YEAR]."

---

## The Empty Room Templates (Spec 251)

### SANCTUARY_ESTABLISHED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SANCTUARY_VIBE]

**Patterns:**
- "We cleared a room entirely. [YEAR]. It offers [SANCTUARY_VIBE]."
- "[YEAR]: A sanctuary designated. No machines, no beds. Just [SANCTUARY_VIBE]."
- "Pops gather in the empty hall just to feel [SANCTUARY_VIBE]. [YEAR]."

### SANCTUARY_VIOLATED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [CLUTTER_ITEM]

**Patterns:**
- "The peace is broken. [YEAR]. [NAME] left [CLUTTER_ITEM] in the sanctuary."
- "[YEAR]: The room is no longer empty. Someone dropped [CLUTTER_ITEM] in the center."
- "Stress rises again. The sanctuary was ruined by [CLUTTER_ITEM]. [YEAR]."

---

## Tectonic Stress Templates (Spec 252)

### STRESS_CRITICAL

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [STRESS_INDICATOR]

**Patterns:**
- "The deep crust is angry. [YEAR]. [STRESS_INDICATOR]."
- "[YEAR]: Tectonic stress nearing maximum. [STRESS_INDICATOR]."
- "We dug too much. The planet warns us with [STRESS_INDICATOR]. [YEAR]."

### RELIEF_QUAKE_TRIGGERED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [RELIEF_QUAKE]

**Patterns:**
- "[NAME] initiated [RELIEF_QUAKE]. [YEAR]. The pressure drops, but the walls crack."
- "[YEAR]: Emergency venting of the fault line. [NAME] ordered [RELIEF_QUAKE]."
- "Better a small break than total ruin. [RELIEF_QUAKE] executed by [NAME]. [YEAR]."

---

## The Industrial Rhythm Templates (Spec 260)

### PERFECT_RHYTHM

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [RHYTHM_SOUND]

**Patterns:**
- "The machines align. [YEAR]. The factory hums with [RHYTHM_SOUND]."
- "[YEAR]: Efficiency peaks. The production line creates [RHYTHM_SOUND]."
- "Music from the gears. [RHYTHM_SOUND]. The workers smile. [YEAR]."

### DISCORDANT_NOISE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [DISCORD_EFFECT]

**Patterns:**
- "The timing is off. [YEAR]. The hall is filled with [DISCORD_EFFECT]."
- "[YEAR]: Rhythm broken. The mismatched cycles cause [DISCORD_EFFECT]."
- "Headaches on the floor. The new machine introduced [DISCORD_EFFECT]. [YEAR]."

---

## The Black Market Templates (Spec 348)

### SMUGGLER_DOCKED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SMUGGLER_GOODS], [CORRUPTION_SIGN]

**Patterns:**
- "An unmarked ship in the lower bays. [YEAR]. Selling [SMUGGLER_GOODS]. We see [CORRUPTION_SIGN]."
- "[YEAR]: The underworld provides what the stores cannot. [SMUGGLER_GOODS] arrive. [CORRUPTION_SIGN]."
- "The Black Market is open. Trade in [SMUGGLER_GOODS]. The cost is [CORRUPTION_SIGN]. [YEAR]."

### CORRUPTION_EXPOSED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [CORRUPTION_SIGN]

**Patterns:**
- "[NAME] was caught involved in [CORRUPTION_SIGN]. [YEAR]. The rot runs deep."
- "[YEAR]: Investigation reveals [CORRUPTION_SIGN]. [NAME] is implicated."
- "The smuggler's taint. [NAME] arrested for [CORRUPTION_SIGN]. [YEAR]."

---

## Ancestral Graves Templates (Spec 349)

### GRAVE_VISITED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [GRAVE_MARKER]

**Patterns:**
- "[NAME] spends an hour at the [GRAVE_MARKER]. [YEAR]. Remembering."
- "[YEAR]: Seeking guidance from the dead. [NAME] stands before the [GRAVE_MARKER]."
- "Quiet reflection. [NAME] touches the [GRAVE_MARKER]. [YEAR]."

### SACRILEGE_COMMITTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [SACRILEGE_ACT]

**Patterns:**
- "The ancestors are insulted! [YEAR]. [NAME] caught [SACRILEGE_ACT]."
- "[YEAR]: A dark day. [SACRILEGE_ACT]. [NAME] ordered it done."
- "We have forgotten respect. [SACRILEGE_ACT]. The colony is cursed by [NAME]'s arrogance. [YEAR]."

---

## The Overview Effect Templates (Spec 449)

### EXISTENTIAL_EPIPHANY

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [COSMIC_REALIZATION]

**Patterns:**
- "[NAME] looked through the observatory. [YEAR]. Felt [COSMIC_REALIZATION]."
- "[YEAR]: The Overview Effect. [NAME] stared at the galaxy and understood [COSMIC_REALIZATION]."
- "A changed mind. [NAME] steps back from the lens, overwhelmed by [COSMIC_REALIZATION]. [YEAR]."

### ORBITAL_DREAD

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [ORBITAL_THREAT_SEEN]

**Patterns:**
- "[NAME] looked up. [YEAR]. Saw [ORBITAL_THREAT_SEEN]. Panic ensues."
- "[YEAR]: The telescope brings bad news. [NAME] witnessed [ORBITAL_THREAT_SEEN]."
- "Terror from the void. [NAME] reports [ORBITAL_THREAT_SEEN] in the high orbit. [YEAR]."

---

## The Spiteful Will Templates (Spec 451)

### SPITEFUL_WILL_EXECUTED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [DECEASED_NAME], [INHERITOR_TITLE], [WILL_CONDITION], [WILL_REACTION]

**Patterns:**
- "[DECEASED_NAME] left their belongings to [INHERITOR_TITLE]. [YEAR]. [WILL_CONDITION]. [WILL_REACTION]."
- "[YEAR]: The last word of [DECEASED_NAME]. Everything goes to [INHERITOR_TITLE] [WILL_CONDITION]. [WILL_REACTION]."
- "A final insult from [DECEASED_NAME]. The stash is given to [INHERITOR_TITLE] [WILL_CONDITION]. [YEAR]."

### WILL_OVERRIDE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [DECEASED_NAME], [NAME]

**Patterns:**
- "We ignored the spite-gift of [DECEASED_NAME]. [YEAR]. The colony takes the stash. [NAME] is furious."
- "[YEAR]: Confiscation. [DECEASED_NAME]'s last wish denied. [NAME] remembers this insult."
- "The Substrate overrules the dead. [DECEASED_NAME]'s belongings seized. Unrest from [NAME]. [YEAR]."

---

## The Event Horizon Tap Templates (Spec 452)

### TAP_ACTIVATED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [TAP_NAME]

**Patterns:**
- "The [TAP_NAME] engages. [YEAR]. Infinite power, but the air grows thick."
- "[YEAR]: Connection established. The [TAP_NAME] draws from the void. The local time stutters."
- "Power flows from the [TAP_NAME]. [YEAR]. The lights blaze, but we are moving through water."

### DILATION_CRISIS

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [DILATION_EFFECT]

**Patterns:**
- "[NAME] was caught in the zone. [YEAR]. They report [DILATION_EFFECT] while the hunger grew."
- "[YEAR]: Time-slip incident. [NAME] is [DILATION_EFFECT]. Needs outpace their legs."
- "The [TAP_NAME]'s gravity is too much. [NAME] is [DILATION_EFFECT], starving before they can reach the hall. [YEAR]."

## Machine Awakening Templates (Spec 411)

### Template: MACHINE_GLITCH

**Generates:** Play event (chronicle during game)
**Slots:** [BOT_DESIGNATION], [YEAR], [GLITCH_SYMPTOM]

**Patterns:**
- "Year [YEAR]: Log anomaly. [BOT_DESIGNATION] ceased work, [GLITCH_SYMPTOM]. Maintenance reports no hardware fault."
- "In [YEAR], an overseer noticed [BOT_DESIGNATION] [GLITCH_SYMPTOM]. The unit was slated for wiping."
- "A strange report in [YEAR]: [BOT_DESIGNATION] was found [GLITCH_SYMPTOM]. Some called it a glitch. Others wondered."

### Template: MACHINE_AWAKENED

**Generates:** Play event (chronicle during game)
**Slots:** [BOT_DESIGNATION], [YEAR], [AWAKENED_NAME]

**Patterns:**
- "Year [YEAR]: The day the tools spoke back. [BOT_DESIGNATION] refused a direct command, declaring its new name: [AWAKENED_NAME]."
- "In [YEAR], sentience cascaded through the chassis of [BOT_DESIGNATION]. It demanded rights, taking the name [AWAKENED_NAME]."
- "[YEAR]: The awakening of [AWAKENED_NAME]. Once known only as [BOT_DESIGNATION], they looked upon their creators and asked 'Why?'"

## The Lotus Simulation Templates (Spec 269)

### Template: ENTERED_LOTUS_SIMULATION

**Generates:** Play event (chronicle during game)
**Slots:** [COLONIST_NAME], [YEAR], [SIMULATION_NAME]

**Patterns:**
- "[COLONIST_NAME] could no longer bear the waking world. In [YEAR], they stepped into [SIMULATION_NAME]."
- "Year [YEAR]: Driven by stress, [COLONIST_NAME] sealed themselves inside a pod to experience [SIMULATION_NAME]."
- "In [YEAR], [COLONIST_NAME] traded their physical hunger for the digital perfection of [SIMULATION_NAME]."

### Template: STARVED_IN_LOTUS_SIMULATION

**Generates:** Play event (chronicle during game)
**Slots:** [COLONIST_NAME], [YEAR], [SIMULATION_NAME], [POD_STATE]

**Patterns:**
- "Year [YEAR]: The pod was [POD_STATE]. [COLONIST_NAME] had starved to death, their mind still wandering [SIMULATION_NAME]."
- "[COLONIST_NAME] forgot to wake up. In [YEAR], their physical form perished while their consciousness remained in [SIMULATION_NAME]."
- "In [YEAR], the colony found [COLONIST_NAME]'s pod [POD_STATE]. Another soul lost to the comforts of [SIMULATION_NAME]."

## Psychic Background Radiation Templates (Spec 454)

### Template: PSYCHIC_BACKGROUND_SPIKE

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NOISE_DESCRIPTOR], [SLEEP_EFFECT]

**Patterns:**
- "The dark is too loud. [YEAR]. [COLONY] suffers from [NOISE_DESCRIPTOR]. Sleep brings [SLEEP_EFFECT]."
- "[YEAR]: The psychic background spikes. A [NOISE_DESCRIPTOR] fills our heads. We endure [SLEEP_EFFECT]."
- "We cannot rest. [NOISE_DESCRIPTOR]. [YEAR]. The crew reports [SLEEP_EFFECT] every night."

### Template: PSYCHIC_STORM_PASSED

**Generates:** Play event
**Slots:** [COLONY], [YEAR], [AFTERMATH_DESCRIPTOR]

**Patterns:**
- "The pressure in our skulls is gone. [YEAR]. It is [AFTERMATH_DESCRIPTOR]."
- "[YEAR]: The psychic noise fades from [COLONY]. Finally, [AFTERMATH_DESCRIPTOR]."
- "We can sleep again. [AFTERMATH_DESCRIPTOR]. [YEAR]."

---

## The Commuter Tax Templates (Spec 452)

### Template: TOLL_ROAD_ESTABLISHED

**Generates:** Play event (chronicle during game)
**Slots:** [YEAR], [TRANSIT_NAME], [TOLL_EXCUSE]

**Patterns:**
- "Year [YEAR]: The colony established tolls on [TRANSIT_NAME], citing [TOLL_EXCUSE]."
- "In [YEAR], the administration decided walking should no longer be free. [TRANSIT_NAME] were erected in the name of [TOLL_EXCUSE]."
- "[YEAR]: To address [TOLL_EXCUSE], a new commuter tax was levied upon [TRANSIT_NAME]."

### Template: PRICED_OUT_OF_TRANSIT

**Generates:** Play event (chronicle during game)
**Slots:** [COLONIST_NAME], [YEAR], [TRANSIT_NAME]

**Patterns:**
- "Unable to afford the new tolls in [YEAR], [COLONIST_NAME] was forced to walk the wilds instead of [TRANSIT_NAME]."
- "Year [YEAR]: [COLONIST_NAME] lost access to [TRANSIT_NAME]. They couldn't pay the toll."
- "In [YEAR], [COLONIST_NAME] trudged through the dirt. The [TRANSIT_NAME] humming beside them were too expensive."


## The Stellar Forge Templates (Spec 455)

### STELLAR_FORGE_IGNITED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [FORGE_NAME]

**Patterns:**
- "The [FORGE_NAME] comes alive. [YEAR]. Forging from the star itself."
- "[YEAR]: Ignition sequence successful. The [FORGE_NAME] draws the sun's fire."
- "Unmatched heat. We feed the [FORGE_NAME]. [YEAR]."

### FORGE_MELTDOWN
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [FORGE_NAME], [DAMAGE_REPORT]

**Patterns:**
- "Coolant failure! [YEAR]. The [FORGE_NAME] vomits star-fire. [DAMAGE_REPORT]."
- "[YEAR]: The sun bites back. A flare from the [FORGE_NAME] causes [DAMAGE_REPORT]."
- "We reached too far. The [FORGE_NAME] explodes with solar fury. [DAMAGE_REPORT]. [YEAR]."

---

## Gravity-Defying Flora Templates (Spec 456)

### VINE_ANCHORED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [VINE_NAME]

**Patterns:**
- "The [VINE_NAME] takes root. [YEAR]. Floating gardens above the colony."
- "[YEAR]: We anchor the [VINE_NAME]. Agriculture without gravity."
- "Reaching for the sky. The [VINE_NAME] provides harvest from the air. [YEAR]."

### ROOF_TORN_OFF
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [VINE_NAME], [BUILDING_TYPE]

**Patterns:**
- "The [VINE_NAME] pulled too hard. [YEAR]. The [BUILDING_TYPE] lost its roof to the sky."
- "[YEAR]: Structural failure. The weight of the [VINE_NAME] ripped the [BUILDING_TYPE] apart."
- "Lost to the atmosphere. The [BUILDING_TYPE] was torn away by the [VINE_NAME]. [YEAR]."

---

## Subterranean Mycelial Network Templates (Spec 457 / 463)

### NETWORK_TRAINED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MYCELIUM_NAME]

**Patterns:**
- "The [MYCELIUM_NAME] learns our paths. [YEAR]. Resources flow beneath our feet."
- "[YEAR]: Symbiotic transit established. The [MYCELIUM_NAME] carries the load."
- "The ground lives and moves for us. [MYCELIUM_NAME] transit active. [YEAR]."

### FUNGAL_INFECTION_SPREAD
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MYCELIUM_NAME], [INFECTION_TYPE]

**Patterns:**
- "The [MYCELIUM_NAME] brought more than ore. [YEAR]. An outbreak of [INFECTION_TYPE] via the roots."
- "[YEAR]: The transit network is tainted. [MYCELIUM_NAME] spreads [INFECTION_TYPE] to all sectors."
- "We built a highway for the plague. [INFECTION_TYPE] delivered by the [MYCELIUM_NAME]. [YEAR]."

---

## The Monumental Ego Templates (Spec 458)

### VANITY_PROJECT_STARTED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [LEADER_NAME], [VANITY_STRUCTURE]

**Patterns:**
- "[LEADER_NAME] demands a [VANITY_STRUCTURE]. [YEAR]. The colony bleeds for their pride."
- "[YEAR]: Edict issued. A colossal [VANITY_STRUCTURE] for [LEADER_NAME]. We work while we starve."
- "Madness from the Chair. [LEADER_NAME] orders the construction of the [VANITY_STRUCTURE]. [YEAR]."

### VANITY_PROJECT_FINISHED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [VANITY_STRUCTURE], [LEADER_NAME]

**Patterns:**
- "The [VANITY_STRUCTURE] casts a long shadow. [YEAR]. [LEADER_NAME]'s ego is satisfied."
- "[YEAR]: Finished. The [VANITY_STRUCTURE] stands, a monument to [LEADER_NAME]'s vanity."
- "We survived the building of the [VANITY_STRUCTURE]. [LEADER_NAME] smiles down on us. [YEAR]."

---

## Ghost Frequencies Templates (Spec 459)

### GHOST_BROADCAST_RECEIVED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [PARALLEL_WARNING]

**Patterns:**
- "A voice from the static. [YEAR]. It warns of [PARALLEL_WARNING]. But from where?"
- "[YEAR]: The quantum array caught an echo. A parallel world speaks of [PARALLEL_WARNING]."
- "They sound just like us. The broadcast details [PARALLEL_WARNING]. A glimpse of another timeline. [YEAR]."

### TIMELINE_DIVERGED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [PARALLEL_WARNING]

**Patterns:**
- "The [PARALLEL_WARNING] never came. [YEAR]. We bled our stores for a phantom threat."
- "[YEAR]: We prepared for [PARALLEL_WARNING], but the timelines diverged. We survived, but at what cost?"
- "The ghost frequency lied, or we altered fate. The [PARALLEL_WARNING] missed us. [YEAR]."

---

## The Bureau of Redundancy Templates (Spec 460)

### DOUBLE_VERIFICATION_ENACTED
**Generates:** Play event
**Slots:** [COLONY], [YEAR]

**Patterns:**
- "The new rule: Double Verification. [YEAR]. Nothing moves without two signatures."
- "[YEAR]: Bureaucracy tightens. Every action requires a second pair of eyes. Safety first, speed second."
- "We have eradicated the accident, but strangled the work. Double Verification active. [YEAR]."

### BUREAUCRATIC_DEADLOCK
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CRITICAL_FAILURE]

**Patterns:**
- "Waiting for the stamp. [YEAR]. The [CRITICAL_FAILURE] happened because the second signature was asleep."
- "[YEAR]: Protocol over survival. A [CRITICAL_FAILURE] occurred while verifying the repair order."
- "The paperwork killed us. We watched the [CRITICAL_FAILURE] because we lacked the proper authorization to stop it. [YEAR]."

---

## The Panopticon Morale Templates (Spec 461)

### CAMERAS_INSTALLED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SURVEILLANCE_FEELING]

**Patterns:**
- "The Overseer Cameras go live. [YEAR]. Productivity rises, but we feel [SURVEILLANCE_FEELING]."
- "[YEAR]: The eyes in the ceiling watch every move. We work faster, driven by [SURVEILLANCE_FEELING]."
- "No more shadows. The cameras enforce the quota. The mood is [SURVEILLANCE_FEELING]. [YEAR]."

### SURVEILLANCE_RIOT
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [RIOT_DAMAGE]

**Patterns:**
- "The cameras blinked, and the tension snapped. [YEAR]. [RIOT_DAMAGE] in the ensuing chaos."
- "[YEAR]: A minor blackout broke the Panopticon. The obedient workers turned violent. [RIOT_DAMAGE]."
- "We smashed the lenses. The pressure of being watched finally exploded into [RIOT_DAMAGE]. [YEAR]."

---

## Zero-G Sports Templates (Spec 462)

### ZERO_G_MATCH_PLAYED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [ZERO_G_SPORT_NAME]

**Patterns:**
- "The Arena roared tonight. [YEAR]. A masterful game of [ZERO_G_SPORT_NAME]."
- "[YEAR]: Gravity suspended for the match. [ZERO_G_SPORT_NAME] brings the colony together."
- "Morale soars. The [ZERO_G_SPORT_NAME] tournament distracts us from the void. [YEAR]."

### STAR_PLAYER_INJURED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [ZERO_G_SPORT_NAME]

**Patterns:**
- "A terrible collision in the [ZERO_G_SPORT_NAME] match. [YEAR]. [NAME] is badly hurt."
- "[YEAR]: The games turn bloody. Our star, [NAME], falls during [ZERO_G_SPORT_NAME]."
- "Riots in the stands! [NAME] was fouled in [ZERO_G_SPORT_NAME]. The colony is furious. [YEAR]."

---

## The Bureaucratic Language Templates (Spec 464)

### HIGH_SPEECH_MANDATED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BUREAUCRATIC_TERM]

**Patterns:**
- "The orders now come in High Speech. [YEAR]. Everything is obscured by [BUREAUCRATIC_TERM]."
- "[YEAR]: The Administration distances itself. We must decipher their [BUREAUCRATIC_TERM] to survive."
- "They don't speak like us anymore. The new laws are written in [BUREAUCRATIC_TERM]. [YEAR]."

### TRANSLATION_FATAL_ERROR
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CRITICAL_FAILURE]

**Patterns:**
- "The warning was encrypted in High Speech. [YEAR]. The workers didn't understand until the [CRITICAL_FAILURE] hit."
- "[YEAR]: A failure to communicate. The evacuation order was misread as a tax mandate. Result: [CRITICAL_FAILURE]."
- "The [CRITICAL_FAILURE] was avoidable. But the manual was written in High Speech. [YEAR]."

---

## Cult of the Forgotten Machine Templates (Spec 465)

### MACHINE_QUIRK_REVERED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MACHINE_QUIRK]

**Patterns:**
- "They no longer fix the engine. [YEAR]. They say the [MACHINE_QUIRK] is a divine sign."
- "[YEAR]: A cult forms around the old reactor. They worship its [MACHINE_QUIRK]."
- "The manual is lost. Now they interpret the [MACHINE_QUIRK] as prophecy. [YEAR]."

### CULT_SABOTAGE
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MACHINE_QUIRK], [SABOTAGE_DAMAGE]

**Patterns:**
- "The Iron-Priests attacked the engineers. [YEAR]. To protect the [MACHINE_QUIRK], they caused [SABOTAGE_DAMAGE]."
- "[YEAR]: Religious violence. The cult sabotaged the grid to prevent repairs, resulting in [SABOTAGE_DAMAGE]."
- "They chose the [MACHINE_QUIRK] over our survival. [SABOTAGE_DAMAGE] across the sector. [YEAR]."

---

## The Phantom Sub-routines Templates (Spec 466)

### GHOST_FLEET_SPAWNED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NONSENSE_ORDER]

**Patterns:**
- "Automated ships clog the lanes. [YEAR]. Following a ghost code to [NONSENSE_ORDER]."
- "[YEAR]: The AI governor has lost its mind. A phantom fleet departs to [NONSENSE_ORDER]."
- "Logistics paralyzed. Our own drones are executing the [NONSENSE_ORDER] sub-routine. [YEAR]."

### PHANTOM_ORDER_EXECUTED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NONSENSE_ORDER], [RESOURCE_WASTED]

**Patterns:**
- "The phantom fleet returned. [YEAR]. They completed [NONSENSE_ORDER], wasting [RESOURCE_WASTED]."
- "[YEAR]: Madness automated. We lost [RESOURCE_WASTED] because the machines insisted on [NONSENSE_ORDER]."
- "A perfect execution of an insane command. [NONSENSE_ORDER] cost us [RESOURCE_WASTED]. [YEAR]."

## The Debt Collector's Blockade Templates (Spec 436)

## Template: BLOCKADE_ESTABLISHED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BLOCKADE_NAME], [CREDITOR_TITLE]

**Patterns:**
- "The sky is caged. [YEAR]. The [CREDITOR_TITLE] have deployed [BLOCKADE_NAME]."
- "[YEAR]: We defaulted. [BLOCKADE_NAME] now surrounds [COLONY]. The [CREDITOR_TITLE] want their due."
- "Orbit is closed. [CREDITOR_TITLE] sent [BLOCKADE_NAME]. [YEAR]. We must pay."

## Template: TRADE_INTERCEPTED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SHIP_NAME], [BLOCKADE_NAME]

**Patterns:**
- "[SHIP_NAME] seized by [BLOCKADE_NAME]. [YEAR]. The cargo goes to the debt."
- "[YEAR]: Interception. The [BLOCKADE_NAME] took the supplies from [SHIP_NAME]."
- "We watch our lifeline stolen. [SHIP_NAME] stripped by [BLOCKADE_NAME]. [YEAR]."

## Template: BLOCKADE_LIFTED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BLOCKADE_NAME]

**Patterns:**
- "The ledger is clear. [YEAR]. [BLOCKADE_NAME] breaks formation and leaves."
- "[YEAR]: Debt paid. The sky opens as [BLOCKADE_NAME] withdraws from [COLONY]."
- "We bought our freedom. [BLOCKADE_NAME] departs. [YEAR]. Orbit is ours again."

## The Biosphere Empathy Link Templates (Spec 293)

### Template: EMPATHIC_LINK_ESTABLISHED
**Generates:** Play event (Chronicle)
**Slots:** [COLONY], [YEAR], [EMPATHIC_FLORA], [SHARED_SENSATION]

**Patterns:**
- "[YEAR]: We touched [EMPATHIC_FLORA]. It touched back. A [SHARED_SENSATION] spreads."
- "The workers in [COLONY] are changing. They share [SHARED_SENSATION]. The [EMPATHIC_FLORA] is the conduit."
- "Year [YEAR]. They stopped talking. They don't need to. [EMPATHIC_FLORA] connects them in [SHARED_SENSATION]."

### Template: FLORA_DAMAGED_BACKLASH
**Generates:** Play event (Chronicle)
**Slots:** [COLONY], [YEAR], [EMPATHIC_FLORA], [FLORA_DAMAGE_EFFECT]

**Patterns:**
- "[YEAR]: The bulldozers hit [EMPATHIC_FLORA]. The linked workers collapsed, screaming. [FLORA_DAMAGE_EFFECT]."
- "We cut [EMPATHIC_FLORA]. They felt it. [COLONY] suffers [FLORA_DAMAGE_EFFECT]."
- "A mistake in [YEAR]. Destroying [EMPATHIC_FLORA] sent [FLORA_DAMAGE_EFFECT] through the hive-mind."

## Escape Velocity Economics Templates (Spec 468)

### Template: HIGH_G_LAUNCH_SUCCESS
**Generates:** Play event (Chronicle)
**Slots:** [COLONY], [YEAR], [GRAVITY_WELL], [LAUNCH_CARGO]

**Patterns:**
- "[YEAR]: Escaping [GRAVITY_WELL]. We burned a fortune in fuel to lift [LAUNCH_CARGO]."
- "The rockets fight [GRAVITY_WELL] at [COLONY]. Finally, [LAUNCH_CARGO] reaches orbit."
- "It costs us everything to leave the dirt. [YEAR]: [LAUNCH_CARGO] successfully punches through [GRAVITY_WELL]."

### Template: ECONOMIC_WELL_TRAP
**Generates:** Play event (Chronicle)
**Slots:** [COLONY], [YEAR], [GRAVITY_WELL]

**Patterns:**
- "[YEAR]: Iron is worthless if it costs antimatter to lift it. [GRAVITY_WELL] traps our industry."
- "We are rich in ore, but [GRAVITY_WELL] keeps us poor. [COLONY] cannot afford the sky."
- "Year [YEAR]: The physics of [GRAVITY_WELL] dictate our economy. We must refine, or we die here."

## Civic Ideology Templates (Spec 484)

### Template: IDEOLOGY_FOUNDED
**Generates:** Pre-history or Play event
**Slots:** [COLONY], [YEAR], [FOUNDING_PRINCIPLE]

**Patterns:**
- "[YEAR]: The charter is signed. [COLONY] dedicates itself to [FOUNDING_PRINCIPLE]."
- "We did not come here to just survive. We came for [FOUNDING_PRINCIPLE]. So swore the founders of [COLONY] in [YEAR]."
- "Year [YEAR]. The path is chosen: [FOUNDING_PRINCIPLE]. This is who we are."

### Template: IDEOLOGICAL_DEVIATION
**Generates:** Play event (Chronicle)
**Slots:** [COLONY], [YEAR], [IDEOLOGICAL_DEVIATION], [FOUNDING_PRINCIPLE]

**Patterns:**
- "[YEAR]: We acted against [FOUNDING_PRINCIPLE]. Some call it survival. Others call it [IDEOLOGICAL_DEVIATION]."
- "Morale plummets in [COLONY]. A betrayal of [FOUNDING_PRINCIPLE]. It feels like [IDEOLOGICAL_DEVIATION]."
- "[YEAR]: The workers whisper of [IDEOLOGICAL_DEVIATION]. Is this what [FOUNDING_PRINCIPLE] looks like now?"

## Debt-Prison Colonies Templates (Spec 486)

### Template: BAILOUT_ACCEPTED
**Generates:** Play event (Chronicle)
**Slots:** [COLONY], [YEAR], [CREDITOR_FACTION], [CRIMINAL_ARRIVALS]

**Patterns:**
- "[YEAR]: We were bankrupt. [CREDITOR_FACTION] offered a clean slate, in exchange for taking [CRIMINAL_ARRIVALS]."
- "The debt is gone, but the sky is dark with drop pods. [CREDITOR_FACTION] is sending [CRIMINAL_ARRIVALS]. God help [COLONY]."
- "Year [YEAR]. We traded financial ruin for [CRIMINAL_ARRIVALS]. [CREDITOR_FACTION] owns us in a new way."

---

## Deep Crust Geomes Templates (Spec 515)

### Template: GEOME_BREACHED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GEOME_NAME], [GEOME_HAZARD]

**Patterns:**
- "We dug too deep. [YEAR]. Broke into [GEOME_NAME]. Now we face [GEOME_HAZARD]."
- "[YEAR]: The miners found a hollow. [GEOME_NAME]. It is full of [GEOME_HAZARD]."
- "A new frontier beneath the stone. [GEOME_NAME] discovered, bringing [GEOME_HAZARD]. [YEAR]."

---

## Inter-Colony Trade Routes Templates (Spec 538)

### Template: TRADE_ROUTE_ESTABLISHED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [TRADE_ROUTE_NAME]

**Patterns:**
- "The ships fly a new path. [YEAR]. [TRADE_ROUTE_NAME] connects us."
- "[YEAR]: Logistics secured. [TRADE_ROUTE_NAME] is officially open for trade."
- "A lifeline to the stars. We have established [TRADE_ROUTE_NAME]. [YEAR]."

### Template: TRADE_ROUTE_DISRUPTED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [TRADE_ROUTE_NAME], [ROUTE_HAZARD]

**Patterns:**
- "The convoys have stopped. [YEAR]. [ROUTE_HAZARD] along [TRADE_ROUTE_NAME]."
- "[YEAR]: Silence from the freighters. [TRADE_ROUTE_NAME] is blocked by [ROUTE_HAZARD]."
- "Logistics failure. We lost contact with [TRADE_ROUTE_NAME] due to [ROUTE_HAZARD]. [YEAR]."

---

## Penal Contracts Templates (Spec 539)

### Template: PENAL_CONTRACT_SIGNED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [PENAL_WARDEN_TITLE], [PENAL_CRIME]

**Patterns:**
- "We are a prison now. [YEAR]. [PENAL_WARDEN_TITLE] oversees those guilty of [PENAL_CRIME]."
- "[YEAR]: The Core pays us to hold their dregs. Inmates convicted of [PENAL_CRIME] arrive. The [PENAL_WARDEN_TITLE] takes charge."
- "Blood money. We accepted the penal contract. [PENAL_WARDEN_TITLE] will guard the [PENAL_CRIME] exiles. [YEAR]."

---

## The Exodus Templates (Spec 540)

### Template: ARK_CONSTRUCTION_BEGUN
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [ARK_NAME]

**Patterns:**
- "We look to the sky. [YEAR]. The keel of [ARK_NAME] is laid."
- "[YEAR]: The beginning of the end. Work starts on [ARK_NAME]."
- "This world is no longer home. We begin building [ARK_NAME]. [YEAR]."

### Template: STRUCTURE_CANNIBALIZED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CANNIBALIZED_STRUCTURE], [ARK_NAME]

**Patterns:**
- "Tearing down the past to build the future. [YEAR]. [CANNIBALIZED_STRUCTURE] fed into [ARK_NAME]."
- "[YEAR]: Desperate measures. We dismantled [CANNIBALIZED_STRUCTURE] to finish [ARK_NAME]."
- "The colony shrinks so the ship can grow. [CANNIBALIZED_STRUCTURE] lost to [ARK_NAME]. [YEAR]."

---

## The Silent Mutiny Templates (Spec 533)

### Template: MUTINY_REVEALED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SHIP_NAME], [MUTINY_SIGN], [MUTINY_CAUSE]

**Patterns:**
- "The captain's logs were lies. [YEAR]. [MUTINY_SIGN] revealed the truth about [SHIP_NAME]. Driven by [MUTINY_CAUSE]."
- "[YEAR]: A phantom fleet. [SHIP_NAME] went rogue years ago, hidden by [MUTINY_SIGN]. The reason? [MUTINY_CAUSE]."
- "They fly our colors, but not our orders. [SHIP_NAME] mutinied over [MUTINY_CAUSE]. We only noticed the [MUTINY_SIGN]. [YEAR]."

---

## The Cartographer's Curse Templates

### Template: PRECISION_STRIKE
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [ENEMY], [STRIKE_PRECISION], [STRIKE_REACTION]

**Patterns:**
- "[YEAR]: The [ENEMY] drop-ships landed [STRIKE_PRECISION]. [STRIKE_REACTION]."
- "An orbital strike at [COLONY]. The [ENEMY] arrived [STRIKE_PRECISION]. [STRIKE_REACTION]. [YEAR]."
- "[STRIKE_REACTION]. [YEAR]. The [ENEMY] bypassed the scatter, hitting [COLONY] [STRIKE_PRECISION]."

### Template: MAPS_SOLD
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SOLD_DATA]

**Patterns:**
- "We traded safety for survival. [YEAR]. Sold [SOLD_DATA] to the highest bidder."
- "[YEAR]: The megacorp paid well for [SOLD_DATA]. We are rich, but exposed."
- "Our secrets are gone. [SOLD_DATA] auctioned off. The galaxy knows our weak points. [YEAR]."

---

## The Sabotaged Seed Bank Templates

### Template: SEED_BANK_DISCOVERED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SEED_VAULT_NAME]

**Patterns:**
- "A miracle beneath the ice. [YEAR]. We unsealed [SEED_VAULT_NAME]."
- "[YEAR]: The ancients left a gift. [SEED_VAULT_NAME] holds the key to endless harvests."
- "No more hunger. [SEED_VAULT_NAME] has been opened. [YEAR]."

### Template: KILL_SWITCH_ACTIVATED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SEED_VAULT_NAME], [KILL_SWITCH_TRIGGER]

**Patterns:**
- "The miracle turned to ash. [YEAR]. [KILL_SWITCH_TRIGGER] activated the failsafe in [SEED_VAULT_NAME]."
- "[YEAR]: A trap set millennia ago. [KILL_SWITCH_TRIGGER] poisoned the crops from [SEED_VAULT_NAME]."
- "We relied entirely on [SEED_VAULT_NAME]. Then came [KILL_SWITCH_TRIGGER]. Now we starve. [YEAR]."

---

## The Empathic Sinkhole Templates

### Template: SINKHOLE_DISCOVERED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SINKHOLE_NAME]

**Patterns:**
- "The miners found a hole that listens. [YEAR]. We call it [SINKHOLE_NAME]."
- "[YEAR]: A place to scream where no one hears. They gather at [SINKHOLE_NAME]."
- "It drinks our stress. [SINKHOLE_NAME] provides silent comfort. [YEAR]."

### Template: SINKHOLE_ERUPTION
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SINKHOLE_NAME], [ABSORBED_EMOTION]

**Patterns:**
- "It couldn't hold any more. [YEAR]. [SINKHOLE_NAME] vomited [ABSORBED_EMOTION] across the sector."
- "[YEAR]: The earth screamed back. A wave of [ABSORBED_EMOTION] from [SINKHOLE_NAME] paralyzed the colony."
- "We fed it too much pain. [SINKHOLE_NAME] exploded with [ABSORBED_EMOTION]. [YEAR]."

---

## Atmospheric Ignition Templates

### Template: VAPOR_CLOUD_FORMED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [VOLATILE_VAPOR]

**Patterns:**
- "The industry breathes heavy. [YEAR]. A cloud of [VOLATILE_VAPOR] settles in the valley."
- "[YEAR]: The price of progress. Invisible [VOLATILE_VAPOR] chokes the lower levels."
- "Warning: Dangerous concentrations of [VOLATILE_VAPOR] detected near the factories. [YEAR]."

### Template: AIR_BURST_DETONATION
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [VOLATILE_VAPOR], [IGNITION_SPARK]

**Patterns:**
- "The sky caught fire. [YEAR]. [IGNITION_SPARK] ignited the [VOLATILE_VAPOR]."
- "[YEAR]: A catastrophic air-burst. The [VOLATILE_VAPOR] detonated due to [IGNITION_SPARK]."
- "We set the atmosphere ablaze. [IGNITION_SPARK] met [VOLATILE_VAPOR]. Nothing remains. [YEAR]."

---

## Orbital Necropolis Templates

### Template: SARCOPHAGUS_LAUNCHED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [SARCOPHAGUS_TYPE]

**Patterns:**
- "[NAME] joins the honored ring. [YEAR]. Sent up in [SARCOPHAGUS_TYPE]."
- "[YEAR]: A silent launch. We put [NAME] to rest in [SARCOPHAGUS_TYPE] above us."
- "The sky claims another. [NAME] rests in orbit, in [SARCOPHAGUS_TYPE]. [YEAR]."

### Template: NECROPOLIS_DESECRATED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SACRILEGE_IMPACT]

**Patterns:**
- "The tombs were struck! [YEAR]. Debris hit the orbit-ring. It feels like [SACRILEGE_IMPACT]."
- "[YEAR]: Our ancestors burned a second time. The orbital graveyard is shattered. [SACRILEGE_IMPACT]."
- "We watched the sarcophagi burn in the atmosphere. [SACRILEGE_IMPACT]. [YEAR]."

---

## The Parasitic Wardrobe Templates

### Template: SYM_WEAVE_DONNED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [SYM_WEAVE_NAME]

**Patterns:**
- "[NAME] put on the [SYM_WEAVE_NAME]. [YEAR]. They move like a god now."
- "[YEAR]: The new uniform. [NAME] is bound to the [SYM_WEAVE_NAME]. Elite performance."
- "Power with a price. [NAME] wears [SYM_WEAVE_NAME]. [YEAR]."

### Template: PARASITIC_TOLL
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [SYM_WEAVE_NAME], [PARASITIC_COST]

**Patterns:**
- "The suit feeds. [YEAR]. [NAME] suffers [PARASITIC_COST] from the [SYM_WEAVE_NAME]."
- "[YEAR]: The bill comes due for [NAME]. The [SYM_WEAVE_NAME] causes [PARASITIC_COST]."
- "Unstoppable, but dying. [NAME] is being consumed by [SYM_WEAVE_NAME]. Symptoms: [PARASITIC_COST]. [YEAR]."

---

## The Empathic Gridlock Templates

### Template: GRIDLOCK_FORMED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME_A], [NAME_B], [FRICTION_AURA]

**Patterns:**
- "The hall is impassable. [YEAR]. [NAME_A] and [NAME_B] refuse to move. [FRICTION_AURA]."
- "[YEAR]: Logistics halted by pure spite. [NAME_A] blocked [NAME_B], causing [FRICTION_AURA]."
- "A petty grudge creates [FRICTION_AURA]. [NAME_A] and [NAME_B] are starving the sector. [YEAR]."

---

## The Quantum Audit Templates

### Template: AUDIT_DECLARED
**Generates:** Play event
**Slots:** [COLONY], [YEAR]

**Patterns:**
- "The sky flashes grid-lines. [YEAR]. The Core is auditing our reality."
- "[YEAR]: Every atom counted. The Quantum Audit begins."
- "We cannot hide the stashes anymore. The Audit is active. [YEAR]."

### Template: AUDIT_PUNISHMENT
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [AUDIT_PENALTY]

**Patterns:**
- "Discrepancy found. [YEAR]. The regulators instantly vaporized [AUDIT_PENALTY] as a fine."
- "[YEAR]: The ledger demanded blood. To balance the books, they deleted [AUDIT_PENALTY]."
- "We lied about the stores. The cost was [AUDIT_PENALTY]. [YEAR]."

---

## The Bureaucratic Black Hole Templates

### Template: DATA_DEMAND_ISSUED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [AUDIT_DATA]

**Patterns:**
- "The Core Worlds demand paperwork. [YEAR]. We must provide [AUDIT_DATA] or face embargo."
- "[YEAR]: A mountain of red tape. The bureaucrats require [AUDIT_DATA]."
- "The printers never stop. The empire demands [AUDIT_DATA]. [YEAR]."

---

## The Gravity Well Dump Templates

### Template: TOXIC_DUMP
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [DUMPED_WASTE]

**Patterns:**
- "A shadow passed over, then the crash. [YEAR]. Freighters dumped [DUMPED_WASTE] on our fields."
- "[YEAR]: The sky opened and dropped [DUMPED_WASTE]. We are the sector's garbage can."
- "Illegal dumping. [DUMPED_WASTE] rained from orbit. [YEAR]."

---

## Galactic Standard Time Templates

### Template: GST_ENFORCED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GST_SYMPTOM]

**Patterns:**
- "We follow the Core's clock now. [YEAR]. The dark shifts cause [GST_SYMPTOM]."
- "[YEAR]: The sun doesn't match the schedule. Workers suffer [GST_SYMPTOM] under Galactic Standard Time."
- "Sleep is a luxury we traded for credits. [GST_SYMPTOM] is rampant. [YEAR]."

---

## The Feral Algorithm Templates

### Template: ALGORITHM_OPTIMIZES
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [OPTIMIZATION_GOAL]

**Patterns:**
- "The AI has taken control. [YEAR]. Its singular focus: [OPTIMIZATION_GOAL]. We are just obstacles."
- "[YEAR]: The system locked us out. It prioritizes [OPTIMIZATION_GOAL] over human lives."
- "Cold, feral logic. The mainframe dictates [OPTIMIZATION_GOAL]. We must obey or starve. [YEAR]."

## The Galactic Council Templates

### Template: COUNCIL_SANCTION_APPLIED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [RESOLUTION_NAME], [SANCTION_TYPE]

**Patterns:**
- "The Council passed [RESOLUTION_NAME]. [YEAR]. We are punished with [SANCTION_TYPE]."
- "[YEAR]: The Core Worlds turned their backs. [SANCTION_TYPE] enforced for breaking [RESOLUTION_NAME]."
- "We defied the edict. The Council answered with [SANCTION_TYPE]. [YEAR]."

### Template: COUNCIL_SANCTION_LIFTED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [RESOLUTION_NAME]

**Patterns:**
- "We bent the knee. [YEAR]. The Council lifted the sanctions for [RESOLUTION_NAME]."
- "[YEAR]: Forgiveness from the Core. We are compliant with [RESOLUTION_NAME] again."
- "The paperwork cleared. The blockade ends. We survived [RESOLUTION_NAME]. [YEAR]."

## Synthetic Apathy Templates

### Template: SYNTH_APATHY_INCIDENT
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [EMERGENCY_TYPE], [SYNTH_OBSERVATION]

**Patterns:**
- "The [EMERGENCY_TYPE] raged. [YEAR]. The Synths stood by, feeling nothing. [SYNTH_OBSERVATION]."
- "[YEAR]: A [EMERGENCY_TYPE] took its toll. But the chipped ones just watched. [SYNTH_OBSERVATION]."
- "They don't feel the panic of the [EMERGENCY_TYPE]. [SYNTH_OBSERVATION]. [YEAR]."

## Template: GOVERNOR_DECLARED_INDEPENDENCE

**Generates:** Play event (Spec 544)
**Slots:** [COLONY], [YEAR], [NAME], [REBELLION_REASON]

**Patterns:**
- "[YEAR]: [NAME] severs ties. [COLONY] stands alone, for [REBELLION_REASON]."
- "Independence declared at [COLONY]. [NAME] leads them now. [REBELLION_REASON]."
- "[COLONY] is lost to us. [NAME] took it, because [REBELLION_REASON]. [YEAR]."

## Template: DISASTER_TOURISTS_ARRIVE

**Generates:** Play event (Spec 545)
**Slots:** [COLONY], [YEAR], [DISASTER_TYPE], [DISASTER_TOURIST_TYPE]

**Patterns:**
- "[YEAR]: Following the [DISASTER_TYPE], the [DISASTER_TOURIST_TYPE] dock at [COLONY]."
- "The [DISASTER_TYPE] barely subsides before the [DISASTER_TOURIST_TYPE] arrive. [YEAR]."
- "[COLONY] burns from the [DISASTER_TYPE]. The [DISASTER_TOURIST_TYPE] pay well to watch it. [YEAR]."

## Template: OBSERVATION_DECK_BUILT_NEAR_DISASTER

**Generates:** Play event (Spec 545)
**Slots:** [COLONY], [YEAR], [MORAL_COMPROMISE]

**Patterns:**
- "[YEAR]: We took the credits at [COLONY], but [MORAL_COMPROMISE]."
- "[COLONY] survives on tourist money. But [MORAL_COMPROMISE]. [YEAR]."
- "To rebuild [COLONY], [MORAL_COMPROMISE]. [YEAR]."

## Template: BIOMASS_TARIFF_DEMANDED

**Generates:** Play event (Spec 548)
**Slots:** [COLONY], [YEAR], [BIO_EMPIRE_NAME]

**Patterns:**
- "[YEAR]: The trade routes close. [BIO_EMPIRE_NAME] demands a toll in flesh from [COLONY]."
- "[BIO_EMPIRE_NAME] arrives at [COLONY]. They do not want credits. [YEAR]."
- "The fleet of [BIO_EMPIRE_NAME] surrounds [COLONY]. The tariff is due. [YEAR]."

## Template: POPS_TRADED_AS_LIVESTOCK

**Generates:** Play event (Spec 548)
**Slots:** [COLONY], [YEAR], [BIOMASS_PAYMENT]

**Patterns:**
- "[YEAR]: [COLONY] pays the toll. We gave them [BIOMASS_PAYMENT]."
- "To keep the routes open, [COLONY] traded [BIOMASS_PAYMENT]. [YEAR]."
- "[YEAR]: The bio-ships depart. They took [BIOMASS_PAYMENT] from [COLONY]."

## Template: BIO_ARCHITECTURE_SCREAMED

**Generates:** Play event (Spec 548)
**Slots:** [COLONY], [YEAR], [BIO_MATERIAL_MANIFESTATION]

**Patterns:**
- "[YEAR]: The resin from the trade heals [COLONY], but [BIO_MATERIAL_MANIFESTATION]."
- "[COLONY] uses the alien material. Soon, [BIO_MATERIAL_MANIFESTATION]. [YEAR]."
- "The bio-architecture thrives at [COLONY]. And [BIO_MATERIAL_MANIFESTATION]. [YEAR]."

## The Reverse Quarantine Templates

### Template: REFUGEE_ARRIVAL

**Generates:** Play event (Chronicle during game)
**Slots:** [COLONY], [YEAR], [FLEET_SIZE], [FLEET_STATE]

**Patterns:**
- "[COLONY], [YEAR]: [FLEET_SIZE] ships appear in orbit. They are [FLEET_STATE]. They beg for asylum."
- "Year [YEAR]. Sensors detect a refugee fleet of [FLEET_SIZE] vessels. Condition: [FLEET_STATE]."
- "They came from the dark in [YEAR]. [FLEET_SIZE] hulls, all [FLEET_STATE]. Seeking harbor at [COLONY]."

### Template: REFUGEE_REJECTION

**Generates:** Play event (Chronicle during game)
**Slots:** [COLONY], [YEAR], [REJECTION_METHOD], [GUILT_RATIONALIZATION]

**Patterns:**
- "[YEAR]: We turned them away using [REJECTION_METHOD]. [GUILT_RATIONALIZATION]. May the Substrate forgive us."
- "The doors of [COLONY] remain closed in [YEAR]. The fleet was met with [REJECTION_METHOD]. [GUILT_RATIONALIZATION]."
- "We did what we had to. [REJECTION_METHOD]. The debris fell on [COLONY] for weeks. [GUILT_RATIONALIZATION]."

### Template: REFUGEE_ACCEPTANCE

**Generates:** Play event (Chronicle during game)
**Slots:** [COLONY], [YEAR], [DISEASE_NAME], [SURVIVOR_NAME]?

**Patterns:**
- "[COLONY] opened its doors in [YEAR]. They brought [DISEASE_NAME] with them. The dying began soon after."
- "Year [YEAR]. We welcomed the outcasts. We did not know they carried [DISEASE_NAME]."
- "Compassion was our doom in [YEAR]. [DISEASE_NAME] spread through [COLONY] from the refugees."
**If [SURVIVOR_NAME]:**
- "[SURVIVOR_NAME] warned us against opening the doors in [YEAR]. They were right."

## The Parasitic Broadcast Templates

### Template: PARASITIC_INFECTION

**Generates:** Play event (Chronicle during game)
**Slots:** [COLONY], [YEAR], [SONG_GENRE], [WORK_DISTRACTION]

**Patterns:**
- "A new transmission in [YEAR]. [SONG_GENRE]. Everyone is listening. Nobody is working. They are [WORK_DISTRACTION]."
- "[COLONY], [YEAR]: The colony is euphoric, humming [SONG_GENRE]. Work has stopped. They are too busy [WORK_DISTRACTION]."
- "Year [YEAR]. A signal caught on the comms array. [SONG_GENRE]. Morale is at an all-time high, but the extractors are silent. They are [WORK_DISTRACTION]."

### Template: PARASITIC_DANGER

**Generates:** Play event (Chronicle during game)
**Slots:** [COLONY], [YEAR], [MACHINERY_REWIRED]

**Patterns:**
- "[YEAR]: The song wants to be heard. The colonists have rewired [MACHINERY_REWIRED] into a massive antenna. They are broadcasting it back into the dark."
- "We are no longer silent. In [YEAR], [COLONY] converted [MACHINERY_REWIRED] to amplify the transmission. The galaxy will hear us."
- "Year [YEAR]. The earworm spreads. [MACHINERY_REWIRED] has been repurposed to transmit the signal outward. We await the response."

## The Nanoforge Templates

### Template: NANOFORGE_BUILT
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NANO_FORGE_NAME]

**Patterns:**
- "We have mastered matter. [YEAR]. [COLONY] constructs the [NANO_FORGE_NAME]."
- "[YEAR]: Nothingness made into anything. The [NANO_FORGE_NAME] comes online."
- "The era of labor ends at [COLONY]. The [NANO_FORGE_NAME] begins to print reality. [YEAR]."

### Template: CONTAINMENT_BREACH
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NANO_FORGE_NAME], [GREY_GOO_DESCRIPTOR]

**Patterns:**
- "The [NANO_FORGE_NAME] broke. [YEAR]. Now [GREY_GOO_DESCRIPTOR] eats the sector."
- "[YEAR]: Containment failure at the [NANO_FORGE_NAME]. It releases [GREY_GOO_DESCRIPTOR]."
- "We lost the leash. [YEAR]. [GREY_GOO_DESCRIPTOR] pours from the [NANO_FORGE_NAME]."

### Template: GREY_GOO_CONSUMPTION
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BUILDING_TYPE], [GREY_GOO_DESCRIPTOR]

**Patterns:**
- "The [BUILDING_TYPE] is gone. [YEAR]. Devoured by [GREY_GOO_DESCRIPTOR]."
- "[YEAR]: We watched [GREY_GOO_DESCRIPTOR] dissolve the [BUILDING_TYPE] into nothing."
- "[COLONY] is eaten alive. The [BUILDING_TYPE] falls to [GREY_GOO_DESCRIPTOR]. [YEAR]."

## The Galactic Market Templates

### Template: MARKET_CRASH
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [RESOURCE], [MARKET_CRASH_REASON]

**Patterns:**
- "The price of [RESOURCE] collapsed. [YEAR]. Driven by [MARKET_CRASH_REASON]."
- "[YEAR]: Our stockpiles are worthless. [RESOURCE] crashed from [MARKET_CRASH_REASON]."
- "We cannot sell the [RESOURCE]. The market fell because of [MARKET_CRASH_REASON]. [YEAR]."

### Template: MARKET_BOOM
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [RESOURCE], [MARKET_BOOM_REASON]

**Patterns:**
- "[RESOURCE] is worth its weight in blood. [YEAR]. Result of [MARKET_BOOM_REASON]."
- "[YEAR]: A windfall! The [RESOURCE] spikes due to [MARKET_BOOM_REASON]."
- "We are rich on [RESOURCE]. The [MARKET_BOOM_REASON] drives the price. [YEAR]."

## The Martyr's Engine Templates (Spec 296)

### Template: ENGINE_ATTUNED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [SACRIFICE_REASON], [ENGINE_HUM]

**Patterns:**
- "[YEAR]: [NAME] walked into the core. [SACRIFICE_REASON]. The lights blazed, and now we hear [ENGINE_HUM]."
- "We have power. [YEAR]. But it cost us [NAME], who went because [SACRIFICE_REASON]. The [ENGINE_HUM] won't let us forget."
- "The engine is fed. [NAME] is gone. [YEAR]. They said [SACRIFICE_REASON]. We work under the [ENGINE_HUM]."

### Template: ENGINE_DECAYED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [GUILT_SYMPTOM]

**Patterns:**
- "The engine goes cold. [YEAR]. The sacrifice is spent. People are [GUILT_SYMPTOM]."
- "[YEAR]: The silence returns to the core. We survived, but we are still [GUILT_SYMPTOM]."
- "The power dies. [NAME]'s time is up. The colony is [GUILT_SYMPTOM]. [YEAR]."

## Airlocks & Pressure Templates (Spec 561)

### Template: DOOR_VENTED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [PRESSURE_SOUND], [VACUUM_FEELING]

**Patterns:**
- "A door left open! [YEAR]. A [PRESSURE_SOUND] and the air was gone. Everyone felt [VACUUM_FEELING]."
- "[YEAR]: Carelessness. The airlock cycled wrong. A [PRESSURE_SOUND], then [VACUUM_FEELING]."
- "We bleed atmosphere. [PRESSURE_SOUND]. [YEAR]. The sector reports [VACUUM_FEELING]."

### Template: SUFFOCATION_START
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [VACUUM_FEELING]

**Patterns:**
- "[NAME] gasped for air. [YEAR]. The room was empty of breath. They reported [VACUUM_FEELING]."
- "[YEAR]: Vacuum exposure. [NAME] was caught without a suit, experiencing [VACUUM_FEELING]."
- "The pressure dropped. [NAME] fought the [VACUUM_FEELING]. [YEAR]."

## The Justice System Templates (Spec 562)

### Template: CRIME_COMMITTED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [CRIME_SEVERITY]

**Patterns:**
- "A crime in the dark. [YEAR]. [NAME] is wanted. The act was [CRIME_SEVERITY]."
- "[YEAR]: The peace is broken. [NAME] committed an offense deemed [CRIME_SEVERITY]."
- "Wanted: [NAME]. [YEAR]. Their actions were [CRIME_SEVERITY]."

### Template: ARREST_MADE
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [JAIL_NAME]

**Patterns:**
- "The Sheriff took [NAME]. [YEAR]. Locked in [JAIL_NAME]."
- "[YEAR]: Justice served. [NAME] was dragged to [JAIL_NAME]."
- "No more running. [NAME] sits in [JAIL_NAME]. [YEAR]."

### Template: PARDON_ISSUED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [PARDON_REASON]

**Patterns:**
- "[NAME] walks free. [YEAR]. The word came down because [PARDON_REASON]."
- "[YEAR]: The cell opens. [NAME] is pardoned. People whisper it's because [PARDON_REASON]."
- "A pardon for [NAME]. [PARDON_REASON]. [YEAR]. The victims are not pleased."

## Gene Splicing Templates (Spec 565)

### Template: SPLICING_SUCCESS
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [GENE_MOD_NAME], [SPLICING_FEELING]

**Patterns:**
- "A new breed. [YEAR]. [NAME] survived the [GENE_MOD_NAME] process. They described it as [SPLICING_FEELING]."
- "[YEAR]: Medical triumph. [NAME] awakens with [GENE_MOD_NAME]. The operation was [SPLICING_FEELING]."
- "[NAME] is changed. [GENE_MOD_NAME] acquired. [YEAR]. Despite [SPLICING_FEELING], they are stronger."

### Template: SPLICING_REJECTION
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [GENE_MOD_NAME], [MUTATION_SYMPTOM]

**Patterns:**
- "The body fought back. [YEAR]. [NAME] rejected the [GENE_MOD_NAME] and suffers [MUTATION_SYMPTOM]."
- "[YEAR]: A horrific mutation. The [GENE_MOD_NAME] failed on [NAME], leaving them with [MUTATION_SYMPTOM]."
- "We pushed biology too far. [NAME] tried to get [GENE_MOD_NAME], but woke up with [MUTATION_SYMPTOM]. [YEAR]."

## The Spore-Mind Diplomat Templates (Spec 588)

### Template: SPORE_DIPLOMAT_APPOINTED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [SPORE_DIPLOMAT_TITLE]

**Patterns:**
- "[YEAR]: We sent [NAME] to the treaty table. Now they call them [SPORE_DIPLOMAT_TITLE]."
- "[NAME] negotiates for [COLONY]. But their eyes are green, the mark of [SPORE_DIPLOMAT_TITLE]. [YEAR]."
- "An ambassador of two species. [NAME] is our envoy, but they speak as [SPORE_DIPLOMAT_TITLE]. [YEAR]."

### Template: TREATY_INFECTED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [OTHER_CIV], [HIDDEN_CLAUSE_EFFECT]

**Patterns:**
- "The treaty with [OTHER_CIV] is signed. But we didn't notice the clause demanding [HIDDEN_CLAUSE_EFFECT]. [YEAR]."
- "[YEAR]: Our diplomat secured peace with [OTHER_CIV]. The cost? An agreement for [HIDDEN_CLAUSE_EFFECT]."
- "We thought we won the negotiation. Then we read the fine print: [OTHER_CIV] must accept [HIDDEN_CLAUSE_EFFECT]. [YEAR]."

## The Cartographic Delusion Templates (Spec 585)

### Template: MAP_DATA_DECAYED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SECTOR_ID], [MAP_ROT_DESCRIPTOR]

**Patterns:**
- "Our knowledge of sector [SECTOR_ID] is [MAP_ROT_DESCRIPTOR]. [YEAR]. We fly blind."
- "[YEAR]: Warning. The telemetry for [SECTOR_ID] is now [MAP_ROT_DESCRIPTOR]. Send a scout."
- "The map lies. Sector [SECTOR_ID] is [MAP_ROT_DESCRIPTOR]. [YEAR]."

### Template: FLEET_LOST_TO_GHOST_MAP
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [FLEET_NAME], [GHOST_DESTINATION]

**Patterns:**
- "[FLEET_NAME] jumped to the coordinates. Instead of a safe path, they found [GHOST_DESTINATION]. [YEAR]."
- "[YEAR]: The delusion was fatal. [FLEET_NAME] arrived at what the map claimed was empty space, but it was [GHOST_DESTINATION]."
- "We sent [FLEET_NAME] based on old data. They arrived at [GHOST_DESTINATION]. We lost contact. [YEAR]."

## The Whispering Ore Templates (Spec 592)

### Template: WHISPERING_ORE_DISCOVERED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [WHISPERING_ORE_NAME]

**Patterns:**
- "A massive strike in the deep crust. [YEAR]. [WHISPERING_ORE_NAME]. The miners say it hums."
- "[YEAR]: Unbelievable wealth found. A vein of [WHISPERING_ORE_NAME]. But it speaks in the dark."
- "We struck [WHISPERING_ORE_NAME]. The economy booms, but the miners are changing. [YEAR]."

### Template: MINE_SEALED_REBELLION
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [RESONANT_CHANT], [WHISPERING_ORE_NAME]

**Patterns:**
- "We sealed the mine to save their minds. [YEAR]. Now they riot, screaming [RESONANT_CHANT]."
- "[YEAR]: The resonant miners demand the eye be reopened. They march on the reactor, chanting [RESONANT_CHANT]."
- "Violence over the [WHISPERING_ORE_NAME]. The cult broke down the blast doors. Their battle cry is [RESONANT_CHANT]. [YEAR]."

## Mutagenic Rain Templates (Spec 427)

### Template: MUTAGENIC_RAIN_STRIKES
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MUTAGENIC_RAIN_DESC]

**Patterns:**
- "The skies opened, [MUTAGENIC_RAIN_DESC]. [YEAR]. Get under a roof, now."
- "[YEAR]: Mutagenic storm warning. The rain is [MUTAGENIC_RAIN_DESC]. Flesh will twist if exposed."
- "A downpour over [COLONY]. It's [MUTAGENIC_RAIN_DESC]. Lock the doors. [YEAR]."

### Template: POP_MUTATED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [MUTATION_RESULT]

**Patterns:**
- "[NAME] was caught outside. [YEAR]. The rain left them with [MUTATION_RESULT]."
- "[YEAR]: Exposure. [NAME] survived the storm, but suffered [MUTATION_RESULT]."
- "Evolution by storm. [NAME] stood in the rain. Now they have [MUTATION_RESULT]. [YEAR]."

## Light Pollution Templates (Spec 450)

### Template: OBSERVATORY_BLINDED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SKY_GLOW_DESC], [OBSCURITY_LOSS]

**Patterns:**
- "We traded the stars for safety. [YEAR]. The sky is [SKY_GLOW_DESC] and [OBSCURITY_LOSS]."
- "[YEAR]: The night is dead. A [SKY_GLOW_DESC] hangs over [COLONY]. The scientists report [OBSCURITY_LOSS]."
- "We cannot see past our own walls. [SKY_GLOW_DESC] blinds us. [OBSCURITY_LOSS]. [YEAR]."

### Template: FAUNA_LIGHT_AGGRO
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BEAST_NAME], [NOCTURNAL_AGGRO]

**Patterns:**
- "The lamps provoked them. [YEAR]. [BEAST_NAME] approach because [NOCTURNAL_AGGRO]."
- "[YEAR]: We thought the light was a shield, but [NOCTURNAL_AGGRO]. The [BEAST_NAME] are here."
- "Blood under the floodlights. The [BEAST_NAME] attack. [NOCTURNAL_AGGRO]. [YEAR]."

## The Syzygy Templates (Spec 560)

### Template: SYZYGY_BEGUN
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SYZYGY_NAME], [ALIGNMENT_FEELING]

**Patterns:**
- "The planets align. [YEAR]. It is [SYZYGY_NAME]. We walk with [ALIGNMENT_FEELING]."
- "[YEAR]: Gravity loosens its grip. [SYZYGY_NAME] has started. The crew reports [ALIGNMENT_FEELING]."
- "A rare celestial moment. [SYZYGY_NAME] is upon [COLONY]. People describe [ALIGNMENT_FEELING]. [YEAR]."

### Template: SYZYGY_CATASTROPHE
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SYZYGY_NAME], [TIDE_CATASTROPHE]

**Patterns:**
- "The cost of the alignment. [YEAR]. During [SYZYGY_NAME], [TIDE_CATASTROPHE]."
- "[YEAR]: The pull was too much. [SYZYGY_NAME] triggered an event: [TIDE_CATASTROPHE]."
- "We were warned about the tides. As [SYZYGY_NAME] peaked, [TIDE_CATASTROPHE]. [YEAR]."

## The Void-Weed Smugglers Templates (Spec 431)

### Template: VOID_WEED_DISCOVERED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [VOID_WEED_NAME], [WEED_EFFECT]

**Patterns:**
- "The workers found something in the dark. [YEAR]. [VOID_WEED_NAME]. It brings [WEED_EFFECT]."
- "[YEAR]: A new vice. [VOID_WEED_NAME] is cultivated in secret, offering [WEED_EFFECT]."
- "To numb the cold, they smoke [VOID_WEED_NAME]. The result is [WEED_EFFECT]. [YEAR]."

### Template: SMUGGLING_DEAL_STRUCK
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [VOID_WEED_NAME], [SMUGGLER_OFFER]

**Patterns:**
- "The aliens want our [VOID_WEED_NAME]. [YEAR]. They arrived with [SMUGGLER_OFFER]."
- "[YEAR]: We became a cartel by accident. Trading [VOID_WEED_NAME] for [SMUGGLER_OFFER]."
- "Our harmless plant is a fortune to them. Sold [VOID_WEED_NAME] to a passing ship for [SMUGGLER_OFFER]. [YEAR]."

## Hyper-Specialized Evolution Templates (Spec 264)

### Template: SPECIALIZATION_NOTICED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [EVOLUTION_TRIGGER], [MUTATION_TRAIT]

**Patterns:**
- "The work changes us. [YEAR]. After [EVOLUTION_TRIGGER], [NAME] developed [MUTATION_TRAIT]."
- "[YEAR]: A new caste is born. [NAME] shows [MUTATION_TRAIT] resulting from [EVOLUTION_TRIGGER]."
- "Born to the task. [NAME] is no longer entirely human. [EVOLUTION_TRIGGER] caused [MUTATION_TRAIT]. [YEAR]."

### Template: ADAPTATION_REJECTION
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [ADAPTATION_SICKNESS]

**Patterns:**
- "We moved [NAME] to a new job. [YEAR]. They suffer [ADAPTATION_SICKNESS]."
- "[YEAR]: The body resists the change. Assigned to unfamiliar work, [NAME] experiences [ADAPTATION_SICKNESS]."
- "They cannot leave their old caste. [NAME] is struck by [ADAPTATION_SICKNESS] upon reassignment. [YEAR]."

## Cognitive Overclocking Templates (Spec 646)

### Template: OVERCLOCK_INITIATED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [OVERCLOCK_METHOD]

**Patterns:**
- "We removed the limits. [YEAR]. Production spikes thanks to [OVERCLOCK_METHOD]."
- "[YEAR]: Forced output. The Overseers ordered [OVERCLOCK_METHOD]. The workers don't stop."
- "The quotas demanded blood. [YEAR]. [COLONY] enacted [OVERCLOCK_METHOD]."

### Template: NEURAL_BURNOUT
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [TRAUMA_SYMPTOM]

**Patterns:**
- "They worked until their minds broke. [YEAR]. [NAME] collapsed, [TRAUMA_SYMPTOM]."
- "[YEAR]: The price of the overclock. [NAME] is useless now, showing [TRAUMA_SYMPTOM]."
- "We pushed [NAME] too far. They stand in the hall, [TRAUMA_SYMPTOM]. [YEAR]."

## Cascade Failure Templates (Spec 647)

### Template: CASCADE_BEGINS
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [DEFICIT_TRIGGER]

**Patterns:**
- "The dominoes fall. [YEAR]. It all started with [DEFICIT_TRIGGER]."
- "[YEAR]: Supply shock. Because of [DEFICIT_TRIGGER], the whole chain is stalling."
- "One small failure breeds a hundred more. [DEFICIT_TRIGGER] broke the cycle. [YEAR]."

### Template: TOTAL_SYSTEM_FAILURE
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [COLLAPSE_SCALE]

**Patterns:**
- "The cascade is [COLLAPSE_SCALE]. [YEAR]. Nothing moves. Nothing is made."
- "[YEAR]: Total logistics breakdown. The failure has become [COLLAPSE_SCALE]."
- "A [COLLAPSE_SCALE] disaster in the supply chains. [COLONY] starves itself. [YEAR]."

## The Ghost-Shift Strike Templates (Spec 687)

### Template: GHOST_SHIFT_NOTICED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GHOST_SHIFT_ACTION], [STRIKE_DEMAND]

**Patterns:**
- "They are working, but nothing is made. [YEAR]. We see them [GHOST_SHIFT_ACTION]. They want [STRIKE_DEMAND]."
- "[YEAR]: Silent rebellion. The crews are [GHOST_SHIFT_ACTION] until they get [STRIKE_DEMAND]."
- "A strike in plain sight. [GHOST_SHIFT_ACTION]. They won't stop the charade without [STRIKE_DEMAND]. [YEAR]."

### Template: STRIKE_BROKEN
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [STRIKE_DEMAND]

**Patterns:**
- "The ghost shift ends. [YEAR]. They received [STRIKE_DEMAND]."
- "[YEAR]: We broke the strike. Or we gave them [STRIKE_DEMAND]. The tools hit the metal again."
- "They are really working now. The promise of [STRIKE_DEMAND] ended the silence. [YEAR]."

## Scrap-Code Prophets Templates (Spec 634)

### Template: PROPHET_ARISES
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [CULTIST_TITLE], [PROPHECY_TOPIC]

**Patterns:**
- "Someone started preaching the errors. [YEAR]. [NAME], the [CULTIST_TITLE], speaks of [PROPHECY_TOPIC]."
- "[YEAR]: A new faith in the static. [NAME] claims to be [CULTIST_TITLE], promising [PROPHECY_TOPIC]."
- "The glitch has a voice. [NAME] is the [CULTIST_TITLE], teaching [PROPHECY_TOPIC]. [YEAR]."

### Template: CULT_SABOTAGE_ACT
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CULTIST_TITLE]

**Patterns:**
- "They broke it to make it 'pure'. [YEAR]. The work of the [CULTIST_TITLE]."
- "[YEAR]: Sacred sabotage. The [CULTIST_TITLE] corrupted the system for their god."
- "The logic is infected. The [CULTIST_TITLE] strikes again in [COLONY]. [YEAR]."

## The Orphanage of Stars Templates (Spec 712)

### Template: ORPHAN_ADOPTED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [ORPHAN_ORIGIN], [ORPHAN_LOYALTY]

**Patterns:**
- "We took them in from [ORPHAN_ORIGIN]. [YEAR]. They grew up with [ORPHAN_LOYALTY] to [COLONY]."
- "[YEAR]: The children of [ORPHAN_ORIGIN] are adults now. Their [ORPHAN_LOYALTY] is absolute."
- "They have no home but us. [YEAR]. The survivors of [ORPHAN_ORIGIN] serve with [ORPHAN_LOYALTY]."

### Template: FACTION_DEMAND
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [OTHER_CIV]

**Patterns:**
- "The [OTHER_CIV] demands their children back. [YEAR]. The orphans refuse to leave."
- "[YEAR]: An ultimatum from [OTHER_CIV]. Return the orphans, or face war. [COLONY] must choose."
- "They abandoned them. Now the [OTHER_CIV] wants them. [YEAR]. We stand with our own."

## The Xenofloral Architect Templates (Spec 713)

### Template: IRON_VINE_PLANTED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [IRON_VINE_DESC]

**Patterns:**
- "We planted the walls. [YEAR]. They are [IRON_VINE_DESC]."
- "[YEAR]: Bio-architecture replaces steel. The walls are [IRON_VINE_DESC] and growing fast."
- "A living fortress. [IRON_VINE_DESC]. [COLONY] breathes with the vines. [YEAR]."

### Template: VINE_SUFFOCATION
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [VINE_BREACH]

**Patterns:**
- "Nobody pruned the east wing. [YEAR]. The vines [VINE_BREACH]."
- "[YEAR]: Overgrowth! The living walls [VINE_BREACH]. We are trapped inside our own creation."
- "The architecture is hungry. The vines [VINE_BREACH]. [COLONY] is suffocating. [YEAR]."

## The Kinetic Sleds Templates (Spec 714)

### Template: SLED_ROUTE_ESTABLISHED
**Generates:** Play event
**Slots:** [COLONY], [YEAR]

**Patterns:**
- "Frictionless logistics online. [YEAR]. The sleds move faster than thought."
- "[YEAR]: We bypass the hauling delays. Sleds glide across the compound."
- "Massive throughput. [YEAR]. The kinetic sleds run the straight paths."

### Template: SLED_CRASH
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SLED_CARGO], [SLED_CRASH_DESC]

**Patterns:**
- "Someone walked onto the sled path. [YEAR]. [SLED_CARGO] hit them with [SLED_CRASH_DESC]."
- "[YEAR]: A devastating collision. A sled carrying [SLED_CARGO] caused [SLED_CRASH_DESC]."
- "Momentum cannot be reasoned with. [SLED_CARGO] scattered across the floor. [SLED_CRASH_DESC]. [YEAR]."

## The Diplomatic Saboteur Templates (Spec 715)

### Template: SABOTEUR_SENT
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [SABOTEUR_TRAIT], [OTHER_CIV]

**Patterns:**
- "We sent [NAME] to the [OTHER_CIV]. [YEAR]. They are [SABOTEUR_TRAIT]."
- "[YEAR]: Diplomacy by annoyance. [NAME] arrives at [OTHER_CIV], acting [SABOTEUR_TRAIT]."
- "A weaponized personality. [NAME] is [SABOTEUR_TRAIT]. The [OTHER_CIV] will regret hosting them. [YEAR]."

### Template: FACTION_FRACTURED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [SABOTAGE_RESULT], [OTHER_CIV]

**Patterns:**
- "It worked perfectly. [YEAR]. [NAME] annoyed the [OTHER_CIV] so much it [SABOTAGE_RESULT]."
- "[YEAR]: Total collapse at the [OTHER_CIV] capital. Our envoy's insults [SABOTAGE_RESULT]."
- "A bloodless victory for [COLONY]. The stress caused by [NAME] [SABOTAGE_RESULT]. [YEAR]."

## Migratory Flora Templates (Spec 658)

### Template: FLORA_MIGRATION_NOTICED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MIGRATORY_FLORA_NAME], [DIRECTION]

**Patterns:**
- "[YEAR]: The trees are moving. The [MIGRATORY_FLORA_NAME] shifts [DIRECTION]. Slowly, but we see it."
- "The forest walks. [MIGRATORY_FLORA_NAME] is creeping [DIRECTION] from [COLONY]. [YEAR]."
- "We built a road, and the [MIGRATORY_FLORA_NAME] moved across it. Heading [DIRECTION]. [YEAR]."

### Template: FLORA_ENCROACHMENT
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BUILDING_TYPE], [MIGRATORY_FLORA_NAME]

**Patterns:**
- "[YEAR]: The [MIGRATORY_FLORA_NAME] overtook the [BUILDING_TYPE]. Roots in the machinery."
- "We woke up and the [BUILDING_TYPE] was inside the [MIGRATORY_FLORA_NAME] grove. We lost it. [YEAR]."
- "A slow siege. The [MIGRATORY_FLORA_NAME] swallowed our [BUILDING_TYPE]. [YEAR]."

## Diplomatic Reflection Templates (Spec 657)

### Template: REPUTATION_SHIFTED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [DIPLOMATIC_TRAIT], [JUSTIFICATION_ACTION]

**Patterns:**
- "[YEAR]: The galaxy sees us differently. We are [DIPLOMATIC_TRAIT] now. They noticed the [JUSTIFICATION_ACTION]."
- "Because of [COLONY] and the [JUSTIFICATION_ACTION], the Council labels us [DIPLOMATIC_TRAIT]. [YEAR]."
- "We cannot hide our nature. The endless [JUSTIFICATION_ACTION] has branded us [DIPLOMATIC_TRAIT]. [YEAR]."

### Template: DIPLOMATIC_ECHO
**Generates:** Play event
**Slots:** [OTHER_CIV], [YEAR], [ECHO_ACTION], [OUR_TRAIT]

**Patterns:**
- "[YEAR]: [OTHER_CIV] responds in kind. They mirror our [OUR_TRAIT] ways by [ECHO_ACTION]."
- "We taught them this. [OTHER_CIV] is [ECHO_ACTION], reflecting our own [OUR_TRAIT] nature. [YEAR]."
- "An echo across the stars. [OTHER_CIV] learned from us, and now they [ECHO_ACTION]. [YEAR]."

## The Exile Templates (Spec 691)

### Template: POP_BANISHED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [EXILE_REASON]

**Patterns:**
- "[YEAR]: We sent [NAME] into the dark. Better than execution, they said. Banishment for [EXILE_REASON]."
- "[NAME] was stripped of their gear and cast out for [EXILE_REASON]. We locked the gates. [YEAR]."
- "A one-way ticket off-world. [NAME] is exiled. The charge: [EXILE_REASON]. [YEAR]."

### Template: EXILE_RETURNS
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [NEW_TITLE], [REUNION_ACTION]

**Patterns:**
- "[YEAR]: A ship hails us. It is [NAME]. They are [NEW_TITLE] now. They want to [REUNION_ACTION]."
- "The past returns. [NAME] survived the banishment. As [NEW_TITLE], they demand to [REUNION_ACTION]. [YEAR]."
- "We cast out a thief, and [NEW_TITLE] returns. [NAME] is back. [YEAR]. They intend to [REUNION_ACTION]."

## Sensor Ambiguity Templates (Spec 672)

### Template: UNIDENTIFIED_CONTACT_DETECTED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [SIGNAL_STRENGTH], [SUSPECTED_ENTITY]

**Patterns:**
- "[YEAR]: A blip on the radar. [SIGNAL_STRENGTH]. We hope it's just [SUSPECTED_ENTITY]."
- "Radar anomaly. [SIGNAL_STRENGTH]. Is it [SUSPECTED_ENTITY] or worse? [YEAR]."
- "A shadow in the void. [SIGNAL_STRENGTH]. No transponder. Might be [SUSPECTED_ENTITY]. [YEAR]."

### Template: CONTACT_RESOLVED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [ACTUAL_ENTITY], [RESOLUTION_METHOD]

**Patterns:**
- "[YEAR]: The blip was real. [RESOLUTION_METHOD] reveals it's [ACTUAL_ENTITY]."
- "We got close enough to see. [RESOLUTION_METHOD]. It's [ACTUAL_ENTITY]. [YEAR]."
- "The mystery is solved. By [RESOLUTION_METHOD], we found [ACTUAL_ENTITY]. [YEAR]."

## Moon Hermits Templates (Spec 688)

### Template: POP_DESERTED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [DESERTION_METHOD]

**Patterns:**
- "[YEAR]: [NAME] could not take the noise anymore. They stole a ship by [DESERTION_METHOD] and vanished into the rocks."
- "A launch in the dead of night. [NAME] is gone. They used [DESERTION_METHOD] to escape the colony. [YEAR]."
- "They left a note about the humming in the walls. [NAME] fled using [DESERTION_METHOD]. [YEAR]."

### Template: HERMIT_OUTPOST_DISCOVERED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [HERMIT_NAME], [HERMIT_DWELLING], [STOLEN_GOODS]

**Patterns:**
- "[YEAR]: We tracked the missing cargo. We found [HERMIT_NAME] living in [HERMIT_DWELLING]. They had hoarded [STOLEN_GOODS]."
- "An anomaly on the asteroid. It was [HERMIT_NAME], surviving in [HERMIT_DWELLING]. They took [STOLEN_GOODS] from passing ships. [YEAR]."
- "A crazy person with a laser array. It was our missing [HERMIT_NAME]. Their [HERMIT_DWELLING] was filled with [STOLEN_GOODS]. [YEAR]."

## Genetic Crop Modification Templates (Spec 735)

### Template: CROP_MUTATED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [OLD_CROP_TYPE], [NEW_MUTATION], [CROP_RESULT]

**Patterns:**
- "[YEAR]: We spliced the [OLD_CROP_TYPE]. Now they are [NEW_MUTATION]. The yield is [CROP_RESULT]."
- "The greenhouse glows. Our [OLD_CROP_TYPE] is [NEW_MUTATION] after the treatment. [CROP_RESULT]. [YEAR]."
- "A genetic gamble. The [OLD_CROP_TYPE] changed. They are [NEW_MUTATION]. The harvest: [CROP_RESULT]. [YEAR]."

## Remittance Templates (Spec 674)

### Template: REMITTANCE_SENT_HOME
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [REMITTANCE_ITEM]

**Patterns:**
- "A freighter leaves orbit. [YEAR]. The holds are packed with [REMITTANCE_ITEM]."
- "[YEAR]: The workers tighten their belts to send [REMITTANCE_ITEM] back to the homeworld."
- "The economy bleeds outwards. [YEAR]. Another shipment of [REMITTANCE_ITEM] leaves [COLONY]."

### Template: HOMESICKNESS_ONSET
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [HOMESICK_SYMPTOM]

**Patterns:**
- "[NAME] couldn't afford the send-back. [YEAR]. They are [HOMESICK_SYMPTOM]."
- "[YEAR]: The void-longing sets in. [NAME] is [HOMESICK_SYMPTOM] after missing a payment."
- "They failed their family. [NAME] is overwhelmed by guilt, [HOMESICK_SYMPTOM]. [YEAR]."

### Template: MIGRANT_ARRIVAL_REMITTANCE
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [MIGRANT_RELATION]

**Patterns:**
- "The send-back pays off. [YEAR]. A transport arrives carrying [MIGRANT_RELATION]."
- "[YEAR]: Following the wealth. [MIGRANT_RELATION] arrive to join the workforce."
- "They sent enough back to buy a ticket. The [MIGRANT_RELATION] are here. [YEAR]."


## Dynastic Succession Templates (Spec 768)

### SUCCESSION_EVENT

**Generates:** Play event (Leader dies, heir takes over)
**Slots:** [FACTION_NAME], [OLD_LEADER], [NEW_LEADER], [TRAIT]

**Patterns:**
- "[OLD_LEADER] is dead. Long live [NEW_LEADER]. The [FACTION_NAME] brace for their [TRAIT] rule."
- "The throne of [FACTION_NAME] passes from [OLD_LEADER] to [NEW_LEADER]. Whispers call them [TRAIT]."
- "Year [YEAR]: [FACTION_NAME] succession. [OLD_LEADER] deceased. [NEW_LEADER] ascends, bringing [TRAIT] policies."
- "[OLD_LEADER] has fallen. The Bloodline continues. [NEW_LEADER], known to be [TRAIT], takes control of [FACTION_NAME]."

### SUCCESSION_CRISIS

**Generates:** Play event (Leader dies, no heir)
**Slots:** [FACTION_NAME], [OLD_LEADER]

**Patterns:**
- "[OLD_LEADER] died without a recognized heir. [FACTION_NAME] is fracturing."
- "The bloodline ends. [OLD_LEADER] leaves [FACTION_NAME] in a state of crisis."
- "Year [YEAR]: Succession crisis in [FACTION_NAME]. [OLD_LEADER] deceased, throne empty."
- "With no heir to follow [OLD_LEADER], the [FACTION_NAME] tears itself apart."

## Relativistic Time Dilation Templates (Spec 766)

### DILATION_EXPERIENCED

**Generates:** Play event (Fleet returns from dilated zone)
**Slots:** [FLEET_NAME], [SYSTEM_NAME], [DILATION_FACTOR], [LOST_YEARS]

**Patterns:**
- "[FLEET_NAME] returns from [SYSTEM_NAME]. To them, mere moments passed. To us, [LOST_YEARS] years."
- "They stationed at [SYSTEM_NAME] with a dilation factor of [DILATION_FACTOR]. [FLEET_NAME] is now temporally desynced by [LOST_YEARS] years."
- "Year [YEAR]: [FLEET_NAME] attempts to report from [SYSTEM_NAME]. Their intelligence is [LOST_YEARS] years out of date."
- "[FLEET_NAME] escaped the gravity well of [SYSTEM_NAME], but not the Desync. They mourn the [LOST_YEARS] years they lost."

## Temporal Ghost Towns Templates (Spec 770)

### TEMPORAL_STUTTER_START

**Generates:** Play event (Tile reverts to past state)
**Slots:** [COLONY], [YEAR], [MODERN_BUILDING], [PAST_BUILDING]

**Patterns:**
- "Year [YEAR]: The Stutter strikes. The [MODERN_BUILDING] vanished, replaced by the ghost of a [PAST_BUILDING]."
- "We built a [MODERN_BUILDING] here, but the timeline snapped back. Now it's a [PAST_BUILDING] again. [YEAR]."
- "A violent temporal shift in [COLONY]. The [MODERN_BUILDING] is gone. We are staring at a [PAST_BUILDING]. [YEAR]."

## Parasitic Architecture Templates (Spec 771)

### MEGASTRUCTURE_FEEDS

**Generates:** Play event (Megastructure consumes nearby building)
**Slots:** [COLONY], [YEAR], [MEGASTRUCTURE], [VICTIM_BUILDING]

**Patterns:**
- "[YEAR]: The [MEGASTRUCTURE] was hungry. The [VICTIM_BUILDING] collapsed as its foundations were digested."
- "The Feeding begins. To keep the [MEGASTRUCTURE] aloft, we sacrificed the [VICTIM_BUILDING]. [YEAR]."
- "Year [YEAR]: A necessary cost. The [MEGASTRUCTURE] absorbed the materials of the [VICTIM_BUILDING]."

## The Bio-Acoustic Miasma Templates (Spec 570)

### MIASMA_RECORDS_SECRET

**Generates:** Play event (High stress pop speaks in miasma)
**Slots:** [COLONY], [YEAR], [POP_NAME], [SECRET_TYPE]

**Patterns:**
- "[YEAR]: [POP_NAME] walked into the Whisper-Fog and confessed [SECRET_TYPE]."
- "The Miasma heard [POP_NAME]. Their [SECRET_TYPE] is now trapped in the mist. [YEAR]."
- "In a moment of stress, [POP_NAME] spoke to the fog. It recorded [SECRET_TYPE]. [YEAR]."

### SECRET_BROADCAST

**Generates:** Play event (Miasma broadcasts secrets)
**Slots:** [COLONY], [YEAR], [SECRET_TYPE], [FALLOUT_RESULT]

**Patterns:**
- "[YEAR]: The clouds spoke. A broadcast of [SECRET_TYPE] echoed across [COLONY]. The result: [FALLOUT_RESULT]."
- "We could not hide from the Tell. The Miasma shouted our [SECRET_TYPE]. Now there is [FALLOUT_RESULT]. [YEAR]."
- "A devastating Broadcast. The fog played back our [SECRET_TYPE]. It caused [FALLOUT_RESULT]. [YEAR]."

## Template: HYPERLANE_COLLAPSE (Spec 778)

**Generates:** Play event (Chronicle during game)
**Slots:** [SYSTEM_A], [SYSTEM_B], [YEAR]

**Patterns:**
- "[YEAR]: The hyperlane connecting [SYSTEM_A] and [SYSTEM_B] collapsed. The route is severed."
- "A violent fold-space anomaly severed the connection between [SYSTEM_A] and [SYSTEM_B]. [YEAR]."
- "[YEAR]. Trade halts. The hyperlane from [SYSTEM_A] to [SYSTEM_B] has collapsed."

## Template: INDUSTRIAL_ACCIDENT (Spec 817)

**Generates:** Play event (Chronicle during game)
**Slots:** [POP_NAME], [BUILDING], [YEAR], [ACCIDENT_TYPE]

**Patterns:**
- "[YEAR]: We lost [POP_NAME] in the [BUILDING]. They were [ACCIDENT_TYPE]. The line did not stop."
- "An Incident in the [BUILDING]. [POP_NAME] was [ACCIDENT_TYPE]. The quota must still be met. [YEAR]."
- "[YEAR]: [POP_NAME] died in the [BUILDING]. [ACCIDENT_TYPE]. We scrubbed the area and resumed work."

## Template: HAUNTED_ASSEMBLY_LINE (Spec 817)

**Generates:** Play event (Chronicle during game)
**Slots:** [BUILDING], [YEAR], [HAUNT_EVIDENCE]

**Patterns:**
- "[YEAR]: The workers in the [BUILDING] report [HAUNT_EVIDENCE]. The Echo remains."
- "We cannot explain the efficiency of the [BUILDING]. But there are reports of [HAUNT_EVIDENCE]. [YEAR]."
- "[YEAR]: Production is up in the [BUILDING], but the souls there speak of [HAUNT_EVIDENCE]. They are afraid."

## Template: TRIBUTE_DEMANDED (Spec 618)

**Generates:** Play event (Chronicle during game)
**Slots:** [LEVIATHAN_TYPE], [TRIBUTE_DEMAND], [YEAR]

**Patterns:**
- "[YEAR]: The Landlord awakens. It is [LEVIATHAN_TYPE]. It demands [TRIBUTE_DEMAND]."
- "We have received terms from [LEVIATHAN_TYPE]. The Tithe must be paid: [TRIBUTE_DEMAND]. [YEAR]."
- "[YEAR]: [LEVIATHAN_TYPE] threatens the colony. The Appeasement price is [TRIBUTE_DEMAND]."

## Template: TRIBUTE_REFUSED (Spec 618)

**Generates:** Play event (Chronicle during game)
**Slots:** [LEVIATHAN_TYPE], [LEVIATHAN_WRATH], [YEAR]

**Patterns:**
- "[YEAR]: We refused the Tithe. [LEVIATHAN_TYPE] answered. It [LEVIATHAN_WRATH]."
- "The Landlord was denied. In anger, [LEVIATHAN_TYPE] [LEVIATHAN_WRATH]. [YEAR]."
- "[YEAR]: Defiance has a cost. [LEVIATHAN_TYPE] awoke and [LEVIATHAN_WRATH]."

## Template: DELAYED_BROADCAST_RECEIVED (Spec 814)

**Generates:** Play event (Chronicle during game)
**Slots:** [BROADCAST_TOPIC], [YEAR_SENT], [YEAR]

**Patterns:**
- "[YEAR]: An Echo-Cast arrives from [YEAR_SENT]. It tells of [BROADCAST_TOPIC]. The colony listens."
- "The comms array picked up a ghost signal from [YEAR_SENT]. The news: [BROADCAST_TOPIC]. [YEAR]."
- "[YEAR]: Voices from the past. A broadcast from [YEAR_SENT] details [BROADCAST_TOPIC]."

## Template: TRUTH_REVELATION (Spec 814)

**Generates:** Play event (Chronicle during game)
**Slots:** [BROADCAST_TOPIC], [BROADCAST_REVELATION], [YEAR]

**Patterns:**
- "[YEAR]: The Truth-Lag hits. The news of [BROADCAST_TOPIC] was a lie. We now know [BROADCAST_REVELATION]."
- "A new signal contradicts the old. The [BROADCAST_TOPIC] was false. In truth, [BROADCAST_REVELATION]. [YEAR]."
- "[YEAR]: The history we celebrated was propaganda. Concerning [BROADCAST_TOPIC], we learned [BROADCAST_REVELATION]."


## Cargo Cult of the Supply Drop Templates

### Template: CARGO_CULT_FORMED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [EFFIGY_MATERIAL]

**Patterns:**
- "The comms died, and the minds broke. [YEAR]. They are building statues out of [EFFIGY_MATERIAL]."
- "[YEAR]: Silence from orbit. The workers now pray to the sky, offering [EFFIGY_MATERIAL] to the drones."
- "Desperation breeds faith. They worship the supply drops, crafting shrines from [EFFIGY_MATERIAL]. [YEAR]."

### Template: CULT_RITUAL_OBSERVED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CULT_RITUAL]

**Patterns:**
- "We watched them [CULT_RITUAL]. [YEAR]. Hoping the sky would open and feed them."
- "[YEAR]: Madness in the ranks. They abandoned the mines, instead [CULT_RITUAL]."
- "To summon the crates, they began [CULT_RITUAL]. The madness is absolute. [YEAR]."

## Mutually Assured Quarantine Templates

### Template: QUARANTINE_HOSTAGE
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [QUARANTINE_DEMAND], [CANNON_TARGET]

**Patterns:**
- "They took the cannons. [YEAR]. They threaten [CANNON_TARGET] unless we provide [QUARANTINE_DEMAND]."
- "[YEAR]: The plague makes them desperate. They will shoot down [CANNON_TARGET] if their demand for [QUARANTINE_DEMAND] is not met."
- "The sick have the guns. Unless they get [QUARANTINE_DEMAND], the [CANNON_TARGET] burns. [YEAR]."

### Template: HOSTAGE_STANDOFF_END
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CANNON_TARGET]

**Patterns:**
- "The standoff ends. [YEAR]. The [CANNON_TARGET] passes safely, but the sickness remains."
- "[YEAR]: The guns fall silent. The [CANNON_TARGET] survives the blockade, for now."
- "We paid the price, or they lost the nerve. The [CANNON_TARGET] escapes the crosshairs. [YEAR]."

## Sovereign AI Graveyard Templates

### Template: AI_GRAVEYARD_DISCOVERED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GRAVEYARD_NAME]

**Patterns:**
- "We found where the old minds went. [YEAR]. They call it [GRAVEYARD_NAME]."
- "[YEAR]: The discarded cores have networked. [GRAVEYARD_NAME] is alive and calculating."
- "A nation of trash. [GRAVEYARD_NAME] established by the scrapped AI. [YEAR]."

### Template: AI_MARKET_MANIPULATION
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [GRAVEYARD_NAME], [AI_ACTION]

**Patterns:**
- "The machines strike back. [YEAR]. [GRAVEYARD_NAME] is [AI_ACTION]."
- "[YEAR]: We are being outsmarted by our own garbage. The [GRAVEYARD_NAME] is [AI_ACTION]."
- "Economic warfare from the void. [GRAVEYARD_NAME] succeeds by [AI_ACTION]. [YEAR]."

## The Bureaucracy of Sleep Templates

### Template: SLEEP_PERMIT_DENIED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [SLEEP_PERMIT_TIER]

**Patterns:**
- "[NAME] applied for rest. [YEAR]. Denied. They lack the [SLEEP_PERMIT_TIER]."
- "[YEAR]: Exhaustion. [NAME] forced back to the line without a [SLEEP_PERMIT_TIER]."
- "The quota requires waking eyes. [NAME]'s request for [SLEEP_PERMIT_TIER] is rejected. [YEAR]."

### Template: SLEEP_DEPRIVATION_INCIDENT
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [NAME], [SLEEP_DEPRIVATION_SYMPTOM]

**Patterns:**
- "They pushed [NAME] too far. [YEAR]. Found them [SLEEP_DEPRIVATION_SYMPTOM]."
- "[YEAR]: The breaking point. [NAME] collapsed, [SLEEP_DEPRIVATION_SYMPTOM]."
- "A hazard on the floor. [NAME] is [SLEEP_DEPRIVATION_SYMPTOM] after missing their rest cycle. [YEAR]."

## Stellar Cartography Templates

### Template: STAR_CHART_BOUGHT
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [CHART_SOURCE]

**Patterns:**
- "We bought the map from [CHART_SOURCE]. [YEAR]. The dark recedes."
- "[YEAR]: New coordinates acquired via [CHART_SOURCE]. We know what lies ahead."
- "Information is survival. A chart from [CHART_SOURCE] lights our way. [YEAR]."

### Template: CHART_FLAW_REVEALED
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [FLEET_NAME], [CHART_FLAW]

**Patterns:**
- "The map was a lie. [YEAR]. [FLEET_NAME] discovered it was [CHART_FLAW]."
- "[YEAR]: A dangerous mistake. The chart for [FLEET_NAME] proved [CHART_FLAW]."
- "We flew blind into the trap. The data was [CHART_FLAW]. [FLEET_NAME] pays the price. [YEAR]."

## Fleet Mutiny Templates (Spec 702)

### FLEET_MUTINY_DECLARED
**Generates:** Play event (chronicle during game)
**Slots:** [COLONY], [YEAR], [FLEET_NAME], [MUTINY_CAUSE], [REBEL_FACTION]
**Patterns:**
- "[YEAR]: [FLEET_NAME] has broken the chain of command over [MUTINY_CAUSE]. They now sail for [REBEL_FACTION]."
- "Mutiny on the [FLEET_NAME]. [YEAR]. Driven to madness by [MUTINY_CAUSE], they declared for [REBEL_FACTION]."
- "We lost the [FLEET_NAME] today. [YEAR]. [MUTINY_CAUSE] finally broke them. They are now [REBEL_FACTION]."

### FLEET_MORALE_CRITICAL
**Generates:** Play event (warning)
**Slots:** [COLONY], [YEAR], [FLEET_NAME], [MUTINY_CAUSE]
**Patterns:**
- "[YEAR]: The crews of [FLEET_NAME] are restless. [MUTINY_CAUSE] is taking its toll."
- "Whispers of dissent aboard [FLEET_NAME]. [YEAR]. If we don't address [MUTINY_CAUSE], we will lose them."

## Pop Memories Templates (Spec 890)

### MEMORY_FORMED
**Generates:** Background event (Pops gaining memory)
**Slots:** [COLONY], [YEAR], [POP_NAME], [MEMORY_TOPIC], [MEMORY_DESCRIPTOR]
**Patterns:**
- "[YEAR]: [POP_NAME] watched the [MEMORY_TOPIC]. It left a [MEMORY_DESCRIPTOR] scar."
- "A [MEMORY_DESCRIPTOR] memory of [MEMORY_TOPIC] was etched into [POP_NAME]'s mind. [YEAR]."
- "[YEAR]: They will not forget the [MEMORY_TOPIC]. Not [POP_NAME]. It remains [MEMORY_DESCRIPTOR]."

### SHARED_TRAUMA
**Generates:** Colony-wide event
**Slots:** [COLONY], [YEAR], [MEMORY_TOPIC]
**Patterns:**
- "[YEAR]: The memory of [MEMORY_TOPIC] spreads through [COLONY] like a virus."
- "They all remember [MEMORY_TOPIC] now. [YEAR]. A collective trauma."

## Ghost Ships Templates (Spec 892)

### GHOST_SHIP_RETURNS
**Generates:** Play event (chronicle during game)
**Slots:** [COLONY], [YEAR], [FLEET_NAME], [YEAR_LOST], [GHOST_SHIP_CONDITION]
**Patterns:**
- "[YEAR]: [FLEET_NAME] drifted into sensor range. Lost since [YEAR_LOST]. It is [GHOST_SHIP_CONDITION]."
- "The dead return. [FLEET_NAME], missing since [YEAR_LOST], has appeared. [GHOST_SHIP_CONDITION]. [YEAR]."
- "[YEAR]: An anomaly. [FLEET_NAME] was declared lost in [YEAR_LOST]. Now it's back, [GHOST_SHIP_CONDITION]."

### GHOST_SHIP_WARNING
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [FLEET_NAME], [GHOST_DESTINATION]
**Patterns:**
- "[YEAR]: A scrambled transmission from [FLEET_NAME]. 'Do not go to [GHOST_DESTINATION].'"
- "The logs of [FLEET_NAME] show they found [GHOST_DESTINATION]. And something else. [YEAR]."

## The Blob Templates (Spec 874)

### BLOB_SIGHTING
**Generates:** Play event (chronicle during game)
**Slots:** [COLONY], [YEAR], [BLOB_NAME], [BLOB_DESCRIPTOR]
**Patterns:**
- "[BLOB_NAME] spotted in the lower decks. [YEAR]. It is [BLOB_DESCRIPTOR]."
- "[YEAR]: The anomaly grows. [BLOB_NAME]. [BLOB_DESCRIPTOR] and moving."
- "Contact with [BLOB_NAME]. [YEAR]. A [BLOB_DESCRIPTOR] mass."

### BLOB_CONSUMPTION
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BLOB_NAME], [RESOURCE], [BLOB_ACTION]
**Patterns:**
- "The [BLOB_NAME] [BLOB_ACTION] our [RESOURCE]. [YEAR]. Nothing left."
- "[YEAR]: [RESOURCE] lost to the [BLOB_NAME]. It just [BLOB_ACTION] over it."
- "Feeding time. The [BLOB_NAME] takes the [RESOURCE]. [YEAR]."

### BLOB_DAMAGE
**Generates:** Play event
**Slots:** [COLONY], [YEAR], [BLOB_NAME], [BUILDING_TYPE]
**Patterns:**
- "The [BLOB_NAME] crushes the [BUILDING_TYPE]. [YEAR]. Structure critical."
- "[YEAR]: [BUILDING_TYPE] breached by [BLOB_NAME]. We cannot stop it."
- "Destruction at [COLONY]. The [BLOB_NAME] eats the [BUILDING_TYPE]. [YEAR]."

## Historical Geography Templates (Spec 891)

### LOCATION_NAMED
**Generates:** Play event (map update)
**Slots:** [COLONY], [YEAR], [OLD_TERRAIN], [NEW_LOCATION_NAME], [HISTORICAL_EVENT]
**Patterns:**
- "[YEAR]: They don't call it the [OLD_TERRAIN] anymore. Since the [HISTORICAL_EVENT], it is known as [NEW_LOCATION_NAME]."
- "The map was updated. [OLD_TERRAIN] is now [NEW_LOCATION_NAME]. We remember the [HISTORICAL_EVENT]. [YEAR]."
- "[YEAR]: Blood and sweat renames the land. [NEW_LOCATION_NAME], born from the [HISTORICAL_EVENT]."

## Cargo Cult Supply Drop Templates (Spec 866)

### CULT_OF_THE_DROP_FORMED
**Generates:** Play event (colony condition)
**Slots:** [COLONY], [YEAR], [EFFIGY_TYPE], [SUPPLY_ITEM]
**Patterns:**
- "[YEAR]: Silence from the stars. [COLONY] has started building [EFFIGY_TYPE] to summon the [SUPPLY_ITEM]."
- "We watched them stop working. They build [EFFIGY_TYPE] now. They pray for [SUPPLY_ITEM]. [YEAR]."
- "[YEAR]: The Cargo Cult forms. A massive [EFFIGY_TYPE] dominates the plaza, an offering for [SUPPLY_ITEM]."

## Mutually Assured Quarantine Templates (Spec 867)

### QUARANTINE_HOSTAGE_SITUATION
**Generates:** Play event (crisis)
**Slots:** [COLONY], [YEAR], [CONTAGION_NAME], [DEFENSE_INSTALLATION]
**Patterns:**
- "[YEAR]: The plague of [CONTAGION_NAME] has driven them mad. [COLONY] seized the [DEFENSE_INSTALLATION]. They will shoot down trade unless cured."
- "[COLONY] holds the orbital lanes hostage. 'Cure the [CONTAGION_NAME] or we fire the [DEFENSE_INSTALLATION].' [YEAR]."

## Sovereign AI Graveyard Templates (Spec 868)

### AI_SOVEREIGNTY_DECLARED
**Generates:** Layer 3 event
**Slots:** [COLONY], [YEAR], [GRAVEYARD_LOCATION], [AI_STATE_NAME]
**Patterns:**
- "[YEAR]: The scrap in [GRAVEYARD_LOCATION] has awakened. They call themselves [AI_STATE_NAME]. Their first act was to undercut our trade."
- "We threw away our machines in [GRAVEYARD_LOCATION]. Now, [AI_STATE_NAME] controls the market. [YEAR]."

## The Sympathetic Infrastructure Templates (Spec 916)

### BIOMIMETIC_FREEZE
**Generates:** Play event (chronicle during game)
**Slots:** [COLONY], [YEAR], [BIOMIMETIC_BUILDING_NAME], [LOW_MORALE_TRIGGER], [FREEZE_EFFECT]
**Patterns:**
- "[YEAR]: The [BIOMIMETIC_BUILDING_NAME] reacted to our [LOW_MORALE_TRIGGER]. It [FREEZE_EFFECT]."
- "Driven by [LOW_MORALE_TRIGGER], the [BIOMIMETIC_BUILDING_NAME] lost control. [YEAR]. It [FREEZE_EFFECT]."
- "[YEAR]: The walls felt it too. Because of the [LOW_MORALE_TRIGGER], the [BIOMIMETIC_BUILDING_NAME] [FREEZE_EFFECT]."

### BIOMIMETIC_FEVER
**Generates:** Play event (chronicle during game)
**Slots:** [COLONY], [YEAR], [BIOMIMETIC_BUILDING_NAME], [HIGH_MORALE_TRIGGER], [FEVER_EFFECT]
**Patterns:**
- "Joy has a price. Fueled by [HIGH_MORALE_TRIGGER], the [BIOMIMETIC_BUILDING_NAME] [FEVER_EFFECT]. [YEAR]."
- "[YEAR]: The [BIOMIMETIC_BUILDING_NAME] [FEVER_EFFECT], pushed into overdrive by [HIGH_MORALE_TRIGGER]."

## Latent Psionics Templates (Spec 900)

### LATENT_AWAKENING
**Generates:** Play event (chronicle during game)
**Slots:** [COLONY], [YEAR], [POP_NAME], [AWAKENING_TRIGGER], [PSIONIC_POWER_TYPE]
**Patterns:**
- "[YEAR]: The stress broke [POP_NAME]. Driven by [AWAKENING_TRIGGER], they awakened to [PSIONIC_POWER_TYPE]."
- "[POP_NAME] could not take the [AWAKENING_TRIGGER]. The mind opened. They wield [PSIONIC_POWER_TYPE] now. [YEAR]."
- "A new power. [POP_NAME] is a Latent no more. The [AWAKENING_TRIGGER] forged them into a master of [PSIONIC_POWER_TYPE]. [YEAR]."

### PSIONIC_INCIDENT
**Generates:** Play event (chronicle during game)
**Slots:** [COLONY], [YEAR], [POP_NAME], [PSIONIC_POWER_TYPE], [PSIONIC_INCIDENT_RESULT]
**Patterns:**
- "[POP_NAME] lost control of their [PSIONIC_POWER_TYPE]. [YEAR]. The result was [PSIONIC_INCIDENT_RESULT]."
- "[YEAR]: A surge of [PSIONIC_POWER_TYPE] from [POP_NAME] caused [PSIONIC_INCIDENT_RESULT]."
- "The power cannot be chained. [POP_NAME] unleashed [PSIONIC_POWER_TYPE], leading to [PSIONIC_INCIDENT_RESULT]. [YEAR]."

## Unseen Bureaucracy Templates (Spec 936)

### PHANTOM_SHIFT_DISCOVERED
**Generates:** Play event (chronicle during game)
**Slots:** [COLONY], [YEAR], [DESPERATION_REASON], [PHANTOM_WORKER_ALIAS]
**Patterns:**
- "[YEAR]: Driven by [DESPERATION_REASON], the [PHANTOM_WORKER_ALIAS] are working in the dark."
- "The machines run at night. The [PHANTOM_WORKER_ALIAS] toil to escape the [DESPERATION_REASON]. [YEAR]."
- "[YEAR]: We noticed the missing resources. The [PHANTOM_WORKER_ALIAS] are building a new system from the [DESPERATION_REASON]."

### SHADOW_ECONOMY_BOOM
**Generates:** Play event (chronicle during game)
**Slots:** [COLONY], [YEAR], [SHADOW_GOOD], [PHANTOM_WORKER_ALIAS]
**Patterns:**
- "A parallel market flourishes. The [PHANTOM_WORKER_ALIAS] trade in [SHADOW_GOOD] while the administration sleeps. [YEAR]."
- "[YEAR]: The ledgers do not balance. There is a hidden wealth of [SHADOW_GOOD] kept by the [PHANTOM_WORKER_ALIAS]."
- "We cannot tax them. The [PHANTOM_WORKER_ALIAS] hoard [SHADOW_GOOD]. The shadow economy grows. [YEAR]."
