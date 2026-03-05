## [Direct Link Wall Sliding]
**Friction:** Diagonal movement against a blocked corner instantly stopped all movement, feeling rigid, clunky, and unresponsive. Players had to perfectly align their inputs to move past corners, leading to frustration and lost momentum.
**Flow:** Implemented wall sliding to preserve movement intent. If a diagonal move is blocked, the system now evaluates single-axis movement (X-only or Y-only). If one axis is free, the character smoothly slides along the wall, maintaining flow and "Juice" (particle effects) without feeling penalized for imprecise input.

## [Combat Cooldown]
**Friction:** Characters were only able to attack once because their `CombatState::cooldown` was set but never decremented. After the first attack, they stood still and never attacked again, feeling broken and unresponsive.
**Flow:** Implemented `combat_cooldown_system` to decrement `CombatState::cooldown` every tick, allowing characters to attack repeatedly and making combat functional and responsive again.
