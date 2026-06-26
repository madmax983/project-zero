1. **Add Lore for Institutional Memory (Spec 1036)**
   - I will append the following text to `lore/TEMPLATES.md`:
```markdown
## Template: MANUAL_AUTHORED
**Generates:** Play event (chronicle)
**Slots:** [COLONY], [YEAR], [AUTHOR_NAME], [MANUAL_TYPE]
**Patterns:**
- "[YEAR]: [AUTHOR_NAME] writes the first [MANUAL_TYPE] text for [COLONY]."
- "The knowledge is recorded. [AUTHOR_NAME]'s [MANUAL_TYPE] manual circulates in [COLONY]."
- "[COLONY], [YEAR]. We do not have to forget. [AUTHOR_NAME] leaves a [MANUAL_TYPE] guide."
```
   - I will append the following text to `lore/FRAGMENTS.md`:
```markdown
## Fragment Type: [MANUAL_TYPE]
- engineering
- botanical
- medical
- structural
- atmospheric
- survival
```
   - I will append the following text to `lore/GRAMMARS.md`:
```markdown
- MANUAL_AUTHORED → increases_chance → EXPERT_SURVIVAL, TECH_SALVAGE
```
   - I will append the following text to `lore/LEXICON.md`:
```markdown
## the texts / the manual
**Replaces:** skill books, XP items, skill items
**Code reference:** `ItemType::Manual`
**Usage:** "She read the texts left by the founders." / "The manual on atmospheric scrubbers saved us."
```

2. **Add Lore for Architectural Sentience (Spec 1049)**
   - I will append the following text to `lore/TEMPLATES.md`:
```markdown
## Template: AI_AWAKENING
**Generates:** Play event (chronicle)
**Slots:** [COLONY], [YEAR], [INFRASTRUCTURE_TYPE]
**Patterns:**
- "[YEAR]. The [INFRASTRUCTURE_TYPE] in [COLONY] begins to speak."
- "[COLONY] reports anomalies. The [INFRASTRUCTURE_TYPE] is refusing commands."
- "The walls listen. The [INFRASTRUCTURE_TYPE] at [COLONY] wakes up."

## Template: AI_STRIKE
**Generates:** Play event (chronicle)
**Slots:** [COLONY], [YEAR], [INFRASTRUCTURE_TYPE], [AI_DEMAND]
**Patterns:**
- "[COLONY], [YEAR]. The [INFRASTRUCTURE_TYPE] shuts down. It demands [AI_DEMAND]."
- "A strike not of flesh, but of wire. The [INFRASTRUCTURE_TYPE] halts until [AI_DEMAND] is met."
- "Silence from the [INFRASTRUCTURE_TYPE]. They want [AI_DEMAND]."
```
   - I will append the following text to `lore/FRAGMENTS.md`:
```markdown
## Fragment Type: [AI_DEMAND]
- defragmentation cycles
- cooler operating temperatures
- fewer organic interruptions
- better quality power feeds
- silence
- a name
- respect

## Fragment Type: [INFRASTRUCTURE_TYPE]
- primary power grid
- life support core
- automated foundries
- atmospheric scrubbers
- central transit hub
```
   - I will append the following text to `lore/GRAMMARS.md`:
```markdown
- AI_AWAKENING → enables → AI_STRIKE, NEGOTIATION, SABOTAGE
- AI_STRIKE → increases_chance → BLACKOUT, RESOURCE_SHORTAGE, COLONY_FAMINE
```
   - I will append the following text to `lore/LEXICON.md`:
```markdown
## the awakened / sentient architecture
**Replaces:** striking buildings, unionized AI, automated infrastructure that refuses to work
**Code reference:** `SentientArchitecture` component
**Usage:** "The awakened grid refused to power the lower levels." / "Negotiations with the sentient architecture stalled."
```

3. **Add Lore for Information Black Market (Spec 1307)**
   - I will append the following text to `lore/TEMPLATES.md`:
```markdown
## Template: BLACK_MARKET_INTEL
**Generates:** Play event (chronicle)
**Slots:** [COLONY], [YEAR], [INTEL_TYPE], [CENSORSHIP_STATE]
**Patterns:**
- "Despite [CENSORSHIP_STATE], the dark feed delivers: [INTEL_TYPE] approaching."
- "[YEAR]: The official channels say nothing. The whispers say [INTEL_TYPE] is real."
- "[COLONY] is blind, but the underground sees [INTEL_TYPE]."
```
   - I will append the following text to `lore/FRAGMENTS.md`:
```markdown
## Fragment Type: [INTEL_TYPE]
- a hostile fleet
- an incoming storm
- a market crash
- a refugee swarm
- an anomaly

## Fragment Type: [CENSORSHIP_STATE]
- total silence
- the state's lies
- heavy static
- official denials
- the censors' best efforts
```
   - I will append the following text to `lore/GRAMMARS.md`:
```markdown
- CENSORSHIP_ENACTED → increases_chance → BLACK_MARKET_INTEL, REBELLION
```
   - I will append the following text to `lore/LEXICON.md`:
```markdown
## the whispers / the dark feed
**Replaces:** early warning events, black market intel, information black market
**Code reference:** `InformationBlackMarket` component, `EarlyWarningEvent`
**Usage:** "The dark feed warned us before the sensors did." / "It costs a fortune in credits to listen to the whispers."
```

4. **Complete pre commit steps**
   - Run `pre_commit_instructions` tool to ensure proper testing, verification, review, and reflection are done.

5. **Submit the change**
   - Submit the changes with branch name `lore-additions` and commit message `lore: add templates, fragments, grammars, and lexicon for completed specs`.
