# Ludwig's Journal 🎲

## [Colony Balance]
**Friction:** Colonists were starving in 4 seconds (real-time) due to aggressive hunger decay (0.02/tick). This created a panic loop where players couldn't react fast enough.
**Flow:** Slowed hunger decay by 20x (0.001/tick) and balanced food production (0.005/tick) to create a sustainable "Day Cycle" rhythm.

## [Feedback System]
**Friction:** Building actions felt "hollow" because there was no confirmation or error message. Players didn't know why they couldn't place a building.
**Flow:** Implemented a `MessageLog` system that provides colored feedback ("Construction started", "Cannot build on Water"). This closes the feedback loop immediately.

## [Consumption Flow]
**Friction:** Pops were snacking constantly (eating at 70% full), breaking their work rhythm and feeling robotic.
**Flow:** Lowered hunger threshold to 40% to create distinct "Work" and "Eat" phases. Added "Thought" bubbles when eating to provide immediate positive feedback ("Tastes like victory").

## [Resource Loop]
**Friction:** Mining instantly credited resources to the global bank, making the physical act of hauling feel redundant and exploitative (double resource gain bug).
**Flow:** Removed instant resource credit. Resources now spawn as physical items that MUST be hauled to be counted. Added immediate log feedback ("Needs Hauling") to teach the player this new requirement.

## [Juice System]
**Friction:** Actions like mining and combat felt "dry" and numerical. Players only saw numbers change, lacking visceral feedback for their orders.
**Flow:** Added a `Particle` system and integrated `ScreenShake`. Mining now kicks up dust/debris, and combat has impact particles and screen shake. This makes the simulation feel "alive" and responsive.

## [Juice System]
**Friction:** Mining and Chopping felt monotonous with consistent tick rates. Combat lacked weight for heavy hits.
**Flow:** Introduced "Critical Success" (5% chance) for work actions to create variable rewards. Scaled screen shake and particles based on damage output to emphasize impact.

## [Combat Feel]
**Friction:** Heavy hits (15+ damage) felt floaty and identical to light hits. The impact was lost in the continuous simulation tick rate.
**Flow:** Implemented "Hit Stop" (Freeze Frame) for 5 ticks (approx 80ms) on heavy hits. This pauses both the attacker and victim, emphasizing the weight of the blow without interrupting the global simulation.

## [UI Responsiveness]
**Friction:** The build cursor and designation rectangle felt static and lifeless, making it hard to see active selection states against the terrain.
**Flow:** Implemented `WallTime` resource to decouple UI animation from simulation tick rate. Added a smooth sine-wave pulse to the build cursor and designation area, providing immediate visual feedback that the tool is active and ready.

## [Combat Dynamics]
**Friction:** Combat damage felt predictable and linear. Even with basic Hit Stop, "Heavy" hits didn't feel rare or special enough.
**Flow:** Introduced Critical Hits (5% chance, 2x Damage). Scaled Hit Stop dynamically (0/2/5/10 ticks) based on damage severity. Crits now trigger a massive 10-tick freeze, yellow particles, and 0.8 screen shake, creating "High Moments" in battle.

## [Physics & Particles 2.0]
**Friction:** Particles were static and lifeless, disappearing in place. Combat and Mining felt numerical rather than physical.
**Flow:** Implemented a sub-grid physics system for particles. Debris now scatters from mining rocks, wood chips fly from trees, and "blood" sprays from combat hits.

## [Combat Weight]
**Friction:** Light attacks (0-5 damage) felt weightless because they had 0 Hit Stop frames. It felt like "swiping at air".
**Flow:** Increased `HIT_STOP_LIGHT` from 0 to 1 tick. Even the smallest hit now registers a micro-pause, adding subconscious "impact" to every successful attack.

## [Movement Flow]
**Friction:** Pops sometimes stuttered when movement speed was slightly below 1.0 (e.g. 0.96), causing them to miss a tick essentially for rounding errors.
**Flow:** Increased "Coyote Time" threshold for movement accumulator from 0.15 to 0.20. This allows pops to "cheat" the movement cost slightly more often, resulting in fluid, continuous motion rather than stop-start lurching.
