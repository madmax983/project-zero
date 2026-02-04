# Ludwig's Journal 🎲

## [Colony Balance]
**Friction:** Colonists were starving in 4 seconds (real-time) due to aggressive hunger decay (0.02/tick). This created a panic loop where players couldn't react fast enough.
**Flow:** Slowed hunger decay by 20x (0.001/tick) and balanced food production (0.005/tick) to create a sustainable "Day Cycle" rhythm.

## [Feedback System]
**Friction:** Building actions felt "hollow" because there was no confirmation or error message. Players didn't know why they couldn't place a building.
**Flow:** Implemented a `MessageLog` system that provides colored feedback ("Construction started", "Cannot build on Water"). This closes the feedback loop immediately.

## [Starvation Loop]
**Friction:** Players started with 0 resources and no way to gather them (AI not implemented), leading to unavoidable starvation in ~80 seconds.
**Flow:** Added a "Landfall Kit" (50 Food, 50 Wood, 20 Stone) to `ColonyResources::default()`. This allows immediate construction of Farms and Housing, buying time for the economy to start.

## [Survival Feedback]
**Friction:** Colonists would silently starve to death, feeling unfair and sudden.
**Flow:** Implemented a `StarvationWarning` system that logs a bright orange warning when a colonist's hunger drops below 30% (Critical).
