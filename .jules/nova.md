## [Pop Thoughts]
**Concept:** Added `Thought` component to Pops and a system to generate random thoughts based on Needs (Hunger/Rest). Displayed in inspection UI.
**Fate:** Merged
**Lesson:** Adding personality to simulation entities makes them feel more alive ("Souls" vs "Agents").

## [Seasonal Visuals]
**Concept:** Added `seasonal_gfx` module to override terrain colors based on `Season` (Winter=Snow, Autumn=Orange).
**Fate:** Merged
**Lesson:** Visual feedback for simulation state (Seasons) vastly improves immersion without changing core logic.

## [Spectral Resonance]
**Concept:** Implemented `Ghost` entities that spawn when pops die. Ghosts wander, dislike light (take damage), and emit negative `Beauty` (making the colony spooky).
**Fate:** Merged
**Lesson:** Death is not just a resource loss but a narrative event that changes the environment.

## [Oneiric Resonance]
**Concept:** Sleeping pops dream about `Chronicle` events. Legendary events give Leisure/Knowledge bonuses; nightmares give nothing.
**Fate:** Merged
**Lesson:** Connecting history (Chronicle) to individual state (Needs) creates emergent storytelling. The past haunts the present.

## [Miasma]
**Concept:** Added `MiasmaGrid` resource and `Sickness` component. Waste, Corpses, and Landfills emit miasma which diffuses and causes morale loss and sickness in nearby pops.
**Fate:** Merged
**Lesson:** Environmental consequences for resource management (Waste) create natural gameplay loops (Cleanup/Burial) without explicit "Quests".
