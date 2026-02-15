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
